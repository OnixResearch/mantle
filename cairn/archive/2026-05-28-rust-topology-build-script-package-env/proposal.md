## Why

Native topology now reaches `aws-lc-sys` build-script compilation, but direct `rustc` fails because the build script uses `env!("CARGO_PKG_VERSION")`. Cargo provides package metadata variables to rustc at compile time. Mantle currently records only package name for build-script runtime env and misses compile-time package metadata.

## What Changes

- Capture a bounded Cargo package environment from native manifests.
- Thread package env data through native target and host unit facts.
- Set Cargo-compatible package env vars for rustc compilation and build-script execution.
- Keep absent optional manifest fields as deterministic empty strings.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, evidence.
- **Testing**: focused rust-plan tests, `cairn validate`, `git diff --check`, clean self-probe.
