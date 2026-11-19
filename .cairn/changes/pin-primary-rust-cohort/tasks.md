# Tasks: Pin the primary Rust compiler cohort

## Phase 1: Pin and document

- [x] [serial] T1.1 Pin `rust-toolchain.toml` to `nightly-2026-04-08`. r[toolchain_cohort.dated_primary]
- [x] [serial] T1.2 Document the update workflow and add ADR 0165. r[toolchain_cohort.update_workflow]
- [x] [serial] T1.3 Record `rustc -Vv` in the flake-check workflow. r[toolchain_cohort.observable_identity]

## Phase 2: Verify

- [x] [serial] T2.1 Confirm direct `cargo -Vv`/`rustc -Vv` and `nix develop -c rustc -Vv` agree on version and commit. r[toolchain_cohort.observable_identity]
- [x] [serial] T2.2 Run a focused workspace check under the pinned compiler. r[toolchain_cohort.dated_primary]
