Task-ID: I3
Covers: bootstrap.part.patch.2.5.9

Status: pass

## Changes

- Added first-consumer and upstream provenance comments to the patch 2.5.9 source pin.
- Added fail-closed object checks for every manually compiled source file.
- Added an in-derivation smoke test that applies a simple unified diff and verifies the edited file content.
- Preserved the installed output check for `bin/patch` and added an installed `patch --version` check.

## Verification

Command: `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/patch-tcc.ncl`
Exit status: 0
Transcript: `evidence/I3-source-pin-recheck.log`

Result: source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

Verified: 2026-04-30T22:38:00Z
