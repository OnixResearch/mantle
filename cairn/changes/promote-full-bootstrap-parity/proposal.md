## Why

Even after the construction work succeeds, Mantle must not claim full bootstrap from ad hoc transcripts or manually edited status. The parity report, release evidence, operator documentation, and independent checkers must converge on the same current evidence and reject stale, mixed-authority, scaffold, or narrower fixed-point bundles.

This final change is promotion-only. It consumes completed native parity rows, a complete StageX lineage receipt, and a current source-built Mantle v2 fixed-point proof. It must not repair missing construction by weakening gates.

## What Changes

- Make bootstrap parity consume the completed row-specific, StageX, and v2 self-build receipts and report complete axes only when every axis-specific requirement is independently satisfied.
- Add an independent verifier for the promoted evidence bundle and mutation-style negative fixtures for every authority and digest edge.
- Export release evidence and operator summaries with exact bounded claims and non-claims.
- Preserve compatibility row/schema identifiers unless a separately versioned migration is required.

## Dependencies

- `close-early-native-bootstrap-parity`.
- `close-final-native-toolchain-parity`.
- `materialize-stagex-lineage-provider`.
- `prove-source-built-mantle-fixed-point`.

## Impact

- **Files**: `src/bootstrap_parity.rs`, parity CLI/tests, bootstrap evidence receipts/checkers, release evidence, operator documentation, README status, and lifecycle evidence.
- **Testing**: parity require modes; independent bundle verification; tamper/stale/mixed-authority fixtures; release checks; machine contracts; Cairn/Tracey/Nix gates.