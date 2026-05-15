## Why

`bootstrap parity-report` still reports `full-musl-binutils` as a live-bootstrap and Guix blocker with only prose evidence. The row should fail closed on an explicit checked contract for the final musl 1.2.5 and binutils 2.41 handoff instead of relying on narrative claims in the derivations.

## What Changes

- Add a compact provider-contract receipt for the full musl/binutils handoff.
- Validate that the receipt schema, derivation paths, status, parity effect, and required markers still match `bootstrap/musl-full.ncl` and `bootstrap/binutils-full.ncl`.
- Surface the row as evidence-backed partial while preserving live-bootstrap and Guix blocker status until native/full-source proof exists.
- Add positive and fail-closed negative regressions for missing receipt and marker drift.

## Scope

In scope: parity-report validation and checked receipt plumbing for the existing derivations.

Out of scope: proving full musl/binutils correctness, changing provider kind semantics, or marking live-bootstrap/Guix parity complete.

## Verification

Run targeted bootstrap parity unit tests, CLI parity tests, `git diff --check`, and `openspec validate check-full-musl-binutils-provider-contract --strict`.
