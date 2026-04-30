Task-ID: I3
Covers: bootstrap.part.musl.1.1.24.tcc

Status: pass

## Changes

- Added first-consumer and upstream provenance comments to the musl 1.1.24 source pin.
- Tightened the startup-object output contract: the derivation now fails if neither `crt1.o` nor `Scrt1.o` is installed.
- Preserved existing required checks for `lib/libc.a` and `include/stdio.h`.

## Verification

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/musl-1.1.24-tcc.ncl`
Exit status: 0
Transcript: `evidence/I3-source-pin-recheck.log`

Result: source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

Verified: 2026-04-30T22:25:00Z
