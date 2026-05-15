## Context

`full-musl-binutils` is the row that represents the final musl 1.2.5 plus binutils 2.41 handoff. The derivations already contain configure/build/install checks and smoke probes, but the parity row currently has no machine-checked receipt tying those markers to a bounded claim.

## Decisions

### Checked combined receipt

Use one receipt at `bootstrap/evidence/full-musl-binutils-provider-contract.json` with schema `mantle-full-musl-binutils-provider-contract-v1`. It records separate derivation paths and marker arrays for `musl-full` and `binutils-full` so either side can drift independently and fail closed.

### Evidence-backed partial only

A valid receipt clears the row's evidence-check failure and documents a checked provider contract, but `expected_complete` stays `false`. The row remains a live-bootstrap/Guix blocker until actual full toolchain correctness/source-root proof exists.

## Risks

- Marker-only contracts can overstate correctness. Mitigation: receipt and row wording explicitly say contract-only/evidence-backed partial and do not unblock parity axes.
- Combined receipt drift could hide which derivation changed. Mitigation: validation error messages name `musl-full` or `binutils-full` and the missing marker.

## Validation Plan

Add positive and negative unit tests for the validator, plus a CLI regression that the real report exposes checked partial evidence while still blocking live-bootstrap/Guix.
