Task-ID: I3
Covers: bootstrap.part.tcc.musl.v2

Status: pass

## Changes

- Added first-consumer and upstream provenance comments to the tcc 0.9.27 source pin.
- Made `libtcc1.a` a required output-contract artifact instead of copying it conditionally.
- Added installed binary checks for `tcc` and `tcc-0.9.27-musl-v2`.
- Added an in-derivation compile smoke that uses the final self-hosted musl-v2 `tcc` to compile a trivial C program.

## Verification

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl-v2.ncl`
Exit status: 0
Transcript: `evidence/I3-source-pin-recheck.log`

Result: source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

Verified: 2026-04-30T22:58:00Z
