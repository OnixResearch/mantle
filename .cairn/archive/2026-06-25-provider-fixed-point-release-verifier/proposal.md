# provider fixed-point release verifier

## Problem

Mantle can now produce a provider-backed Cargo-free stage1/stage2 fixed-point proof under an explicit source-built Rust/native closure. Operators need a cheap verification boundary for reusing that proof as release-adjacent evidence: a release verification command should be able to validate the proof bundle shape, stage receipts, closure enforcement, binary digest equality, and bounded non-claims before anyone cites the bundle.

Without that verifier, release evidence can point at a fixed-point bundle, but the release verifier does not independently check that the bundle actually proves the bounded fixed-point claim or that it keeps release reproducibility and full Cargo compatibility out of scope.

## Proposed change

Add an optional provider fixed-point proof verifier to `mantle release verify`. The verifier should accept a proof bundle path, validate the existing fixed-point bundle files without rebuilding, and report a structured result in both human and JSON output. A required mode should fail closed when the bundle is absent or invalid.

The verifier remains bounded: success means the supplied bundle is a valid provider-backed Cargo-free fixed-point proof under an enforced source-built closure. It does not promote the release to deterministic/reproducible status.

## Success criteria

- `mantle release verify` accepts a Cargo-free fixed-point proof bundle and reports whether it is valid.
- Required mode fails closed on missing, malformed, mismatched, or overclaiming proof bundles.
- Positive and negative tests cover a valid bundle summary, stage digest mismatch, missing enforced closure, and missing non-claims.
- Evidence records focused tests, build, diff check, Cairn validation, and release verifier behavior.
