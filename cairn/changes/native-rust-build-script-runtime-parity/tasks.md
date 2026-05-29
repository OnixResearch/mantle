# Tasks

## Spec

- [ ] [serial] Add build-script runtime parity requirement and design. r[rust_package_planning.native_build_script_runtime]

## Implementation

- [ ] [serial] Centralize build-script environment derivation from native facts. r[rust_package_planning.native_build_script_runtime]
- [ ] [serial] Complete typed parser for supported Cargo metadata lines. r[rust_package_planning.native_build_script_runtime]
- [ ] [serial] Thread typed metadata into dependent rustc args/env/link search. r[rust_package_planning.native_build_script_runtime]
- [ ] [serial] Add deterministic blockers for unsupported host probing and malformed metadata. r[rust_package_planning.native_build_script_runtime]
- [ ] [serial] Add fixtures for cc-rs-like, links metadata, and malformed metadata cases. r[rust_package_planning.native_build_script_runtime]

## Verification

- [ ] [serial] Run build-script env/metadata tests, topology self-probe, and Cairn validation. r[rust_package_planning.native_build_script_runtime]
