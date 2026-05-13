## Why

After `genoutput` was split from the remaining GCC 4.0 generated-source wrapper, `genattrtab` is the last generic generated-source boundary still emitting `generated_bootstrap_stub`. Splitting it closes this grouped-wrapper debt with a named and checked boundary.

## What Changes

- Replace the generic `genattrtab` generated-source wrapper with a dedicated checked empty-attrtab source boundary.
- Add derivation-local checks for the boundary executable, marker, symbol, and absence of legacy generic labels.
- Refresh GCC 4.0 placeholder inventory and add parity regression coverage.
- Preserve the `gcc.4.0` evidence-backed partial classification.

## Non-Goals

- Proving native `genattrtab` correctness.
- Completing GCC 4.0 native/self-hosted compiler correctness.
- Unblocking live-bootstrap or Guix parity.

## Verification

Run eval/shell shape checks, GCC 4.0 build, bootstrap parity tests, parity report, OpenSpec validation, and `git diff --check`.
