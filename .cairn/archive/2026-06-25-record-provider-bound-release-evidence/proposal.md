## Why

Mantle now verifies that provider fixed-point proof evidence matches a packaged release binary, but the durable release-evidence transcript still needs a fresh bundle produced under that stricter rule. The previous release evidence remains useful history, but the provider proof and release artifact binding should be recorded as current evidence before claiming a provider-backed release artifact.

## What Changes

- Produce a fresh release evidence bundle whose provider fixed-point proof stage binary digest matches a packaged release binary artifact.
- Run deterministic release proof over the complete packaged binary set.
- Replay verification from a copied artifact set with deterministic-release and provider fixed-point gates enabled.
- Record concise tracked Cairn evidence with generated artifact paths, digests, verifier status, and bounded non-claims.

## Impact

- **Files**: Cairn evidence and accepted verification-evidence spec text only.
- **Generated artifacts**: release bundle, proof receipts, replay scratch, and JSON outputs under ignored `target/` paths.
- **Testing**: required verifier command, receipt checker, portable replay positive and negative checks, Cairn validation/gates.
