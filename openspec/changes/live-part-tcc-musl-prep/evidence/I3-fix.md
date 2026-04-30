Task-ID: I3
Covers: bootstrap.part.tcc.musl.prep

Status: pass

## Changes

- Added first-consumer and upstream provenance comments to the tcc 0.9.27 source pin.
- Added output-contract checks for both installed bridge compiler names: `tcc` and `tcc-musl-prep`.
- Added checks that the carried Mes libc archive and headers are installed for the bridge compiler.
- Added a `tcc -v` smoke check inside the derivation.

## Verification

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl-prep.ncl`
Exit status: 0
Transcript: `evidence/I3-source-pin-recheck.log`

Result: source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

Verified: 2026-04-30T22:51:00Z
