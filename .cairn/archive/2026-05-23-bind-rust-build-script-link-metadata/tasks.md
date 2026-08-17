# Tasks: bind-rust-build-script-link-metadata

## Implementation

- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.link_search_binding] Bind supported `rustc-link-search` metadata as deterministic `-L` rustc args.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.link_lib_binding] Bind supported `rustc-link-lib` metadata as deterministic `-l` rustc args.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.link_metadata_blockers] Fail closed on malformed or unsupported link metadata.

## Verification

- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.link_lib_binding] Add a positive CLI fixture whose target needs build-script-supplied native link metadata.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.link_metadata_blockers] Add a negative CLI fixture for unsupported link metadata.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.link_search_binding] Run focused Rust-plan/CLI tests plus Cairn gates.
