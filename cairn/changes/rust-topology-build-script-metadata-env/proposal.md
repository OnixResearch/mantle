## Why

Native topology now compiles `async-stream-impl`, then blocks when running `aws-lc-rs`'s build script. That script expects Cargo-provided `DEP_AWS_LC_...` variables exported from the linked `aws-lc-sys` build script metadata. Mantle currently parses only rustc-oriented build-script metadata and does not model package `links` metadata or package-level custom build paths such as `build = "builder/main.rs"`.

## What Changes

- Parse package-level `build` and `links` fields from native manifests.
- Capture safe custom build-script metadata key/value output such as `cargo:include=...`.
- Propagate metadata from immediate linked dependencies into dependent build-script environments as deterministic `DEP_<LINKS>_<KEY>` variables.
- Order linked dependency build scripts before dependent build scripts that need their metadata.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, archived evidence.
- **Testing**: focused rust-plan tests, `cairn validate`, `git diff --check`, and clean self-probe receipt.
