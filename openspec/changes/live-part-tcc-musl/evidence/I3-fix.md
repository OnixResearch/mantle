Task-ID: I3
Covers: bootstrap.part.tcc.musl

Status: pass

## Changes

- Added first-consumer and upstream provenance comments to the tcc 0.9.27 source pin.
- Made `libtcc1.a` a required output-contract artifact instead of copying it conditionally.
- Added installed binary checks for `tcc` and `tcc-0.9.27-musl`.
- Added an in-derivation compile smoke that uses the installed musl-linked `tcc` to compile a trivial C program.

## Verification

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl.ncl`
Exit status: 0
Transcript: `evidence/I3-source-pin-recheck.log`

Result: source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

Verified: 2026-04-30T22:45:00Z
