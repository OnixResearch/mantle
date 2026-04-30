Task-ID: I3
Covers: bootstrap.part.grep.2.4

Status: pass

## Changes

- Added first-consumer and upstream provenance comments to the `grep-2.4` source pin.
- Removed per-source compile suppression from `bootstrap/grep-2.4-musl.ncl`.
- Added fail-closed object checks for every source in the manual compile set before linking.
- Added output-contract checks for `grep`, `egrep`, and `fgrep`.

## Verification

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/grep-2.4-musl.ncl`
Exit status: 0
Transcript: `evidence/I3-source-pin-recheck.log`

Result: source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

Verified: 2026-04-30T21:55:51Z
