use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{env, fs, io::Write, path::Path};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("append") => {
            let ledger = args.next().context("missing ledger")?;
            if args.next().as_deref() != Some("--kind") {
                bail!("expected --kind");
            }
            let kind = args.next().context("missing kind")?;
            if args.next().as_deref() != Some("--artifact") {
                bail!("expected --artifact");
            }
            let artifact = args.next().context("missing artifact")?;
            if args.next().as_deref() != Some("--label") {
                bail!("expected --label");
            }
            let label = args.next().context("missing label")?;
            append(&ledger, &kind, &artifact, &label)?;
        }
        Some("verify") => verify(&args.next().context("missing ledger")?)?,
        _ => bail!("usage: axiom-proof append ledger --kind KIND --artifact FILE --label LABEL | verify ledger"),
    }
    Ok(())
}

fn append(ledger: &str, kind: &str, artifact: &str, label: &str) -> Result<()> {
    let previous = if Path::new(ledger).exists() {
        last_hash(&fs::read_to_string(ledger)?)?
    } else {
        "0".repeat(64)
    };
    let artifact_hash = sha256_hex(&fs::read(artifact)?);
    let payload = format!("prev={previous};kind={kind};artifact={artifact_hash};label={label}");
    let node = sha256_hex(payload.as_bytes());
    writeln!(
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(ledger)?,
        "{node}\t{previous}\t{kind}\t{artifact_hash}\t{label}"
    )?;
    println!("{node}");
    Ok(())
}

fn verify(path: &str) -> Result<()> {
    let raw = fs::read_to_string(path)?;
    let mut previous = "0".repeat(64);
    let mut count = 0usize;
    for (index, line) in raw.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            bail!("line {} malformed", index + 1);
        }
        let node = fields[0];
        let prev = fields[1];
        let kind = fields[2];
        let artifact = fields[3];
        let label = fields[4];
        if prev != previous {
            bail!("line {} breaks the hash chain", index + 1);
        }
        let calculated = sha256_hex(
            format!("prev={prev};kind={kind};artifact={artifact};label={label}").as_bytes(),
        );
        if node != calculated {
            bail!("line {} node hash mismatch", index + 1);
        }
        previous = node.to_owned();
        count += 1;
    }
    println!("VALID PROOF DAG: {count} event(s), head={previous}");
    Ok(())
}

fn last_hash(raw: &str) -> Result<String> {
    Ok(raw
        .lines()
        .last()
        .context("empty ledger")?
        .split('\t')
        .next()
        .context("malformed ledger")?
        .to_owned())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
