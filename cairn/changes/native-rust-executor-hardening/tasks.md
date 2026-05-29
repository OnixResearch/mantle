# Tasks

## Spec

- [ ] [serial] Add native executor hardening requirement and design. r[rust_package_planning.native_executor_hardening]

## Implementation

- [ ] [serial] Extend receipt cache checks to all declared input/artifact/env/toolchain facts. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Validate dependency and host artifact files and digests before rustc. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Normalize deterministic blocker classes for stale, missing, and mismatched material. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Redact and stabilize rustc diagnostics in failure receipts. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Add positive cache-hit and negative stale-output tests. r[rust_package_planning.native_executor_hardening]

## Verification

- [ ] [serial] Run executor cache/rebuild tests, receipt replay tests, topology self-probe, and Cairn validation. r[rust_package_planning.native_executor_hardening]
