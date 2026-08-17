# Design: execute-rust-build-script-metadata

## Approach

Reuse the existing host-artifact topology executor. After a supported `custom-build` host unit compiles successfully, Mantle locates the produced executable, creates a deterministic `OUT_DIR` under the execution output root, runs the executable with a minimal explicit environment, parses bounded UTF-8 stdout lines, and records a build-script run receipt.

The parsed metadata becomes the host artifact for target consumers:
- `OUT_DIR` is added to target rustc environment.
- `cargo:rustc-env=K=V` entries are added to target rustc environment.
- `cargo:rustc-cfg=...` entries are appended as `--cfg ...` rustc args.
- link and rerun surfaces are receipt-bound but not used for native link probing.

## Determinism

Metadata lists are sorted/deduplicated. Run receipts hash the parsed metadata with self-reference fields cleared. Diagnostics are bounded/redacted. The rail does not inspect Cargo target directories or ambient Cargo caches.

## Failure behavior

The executor blocks before target `rustc` if a required custom-build unit fails to compile, fails to run, emits malformed metadata, or if a target consumes build-script metadata that has not been produced.
