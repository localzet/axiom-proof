use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{env, fs, path::Path};
fn main() -> Result<()> {
    let mut a = env::args().skip(1);
    match a.next().as_deref() {
        Some("append") => {
            let ledger = a.next().context("missing ledger")?;
            let proof = a.next().context("missing proof")?;
            if a.next().as_deref() != Some("--label") {
                bail!("expected --label");
            }
            let label = a.next().context("missing label")?;
            append(&ledger, &proof, &label)?
        }
        Some("verify") => verify(&a.next().context("missing ledger")?)?,
        _ => bail!("usage: axiom-proof append ledger proof --label LABEL | verify ledger"),
    };
    Ok(())
}
fn append(ledger: &str, proof: &str, label: &str) -> Result<()> {
    let proof_raw = fs::read_to_string(proof)?;
    if !proof_raw.starts_with("AXIOM-PROOF/1\n") {
        bail!("not an AXIOM-PROOF/1 receipt");
    }
    let prev = if Path::new(ledger).exists() {
        last_hash(&fs::read_to_string(ledger)?)?
    } else {
        "0".repeat(64)
    };
    let ph = hex(&hash(proof_raw.as_bytes()));
    let payload = format!("prev={prev};proof={ph};label={label}");
    let node = hex(&hash(payload.as_bytes()));
    let line = format!("{node}\t{prev}\t{ph}\t{label}\n");
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger)?
        .write_all(line.as_bytes())?;
    println!("{node}");
    Ok(())
}
use std::io::Write;
fn verify(path: &str) -> Result<()> {
    let raw = fs::read_to_string(path)?;
    let mut expected = "0".repeat(64);
    let mut count = 0usize;
    for (i, l) in raw.lines().enumerate() {
        let p: Vec<_> = l.split('\t').collect();
        if p.len() != 4 {
            bail!("line {} malformed", i + 1)
        }
        let node = p[0];
        let prev = p[1];
        let proof = p[2];
        let label = p[3];
        if prev != expected {
            bail!("line {} breaks chain", i + 1)
        }
        let calc = hex(&hash(
            format!("prev={prev};proof={proof};label={label}").as_bytes(),
        ));
        if node != calc {
            bail!("line {} hash mismatch", i + 1)
        }
        expected = node.to_owned();
        count += 1;
    }
    println!("VALID DAG CHAIN: {count} node(s), head={expected}");
    Ok(())
}
fn last_hash(raw: &str) -> Result<String> {
    Ok(raw
        .lines()
        .last()
        .context("empty ledger")?
        .split('\t')
        .next()
        .context("bad ledger")?
        .to_owned())
}
fn hash(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
fn hex(h: &[u8; 32]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}
