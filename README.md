# axiom-proof

Creates and verifies an append-only hash chain over proof receipts and state transitions. It is the v0.1 seed of the
Axiom Proof DAG.

> **Maturity:** research prototype v0.1. The default verifier proves properties by exhaustive evaluation over an
> explicitly finite input domain. A VALID receipt is therefore a theorem about that bounded model, not a claim of
> unbounded program correctness.

```bash
cargo run -- append ledger.axdag candidate.axproof --label initial-verification
cargo run -- verify ledger.axdag
```
