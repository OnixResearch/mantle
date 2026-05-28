## Why

The clean native rust-plan self-probe now reaches a deterministic build-script metadata parser blocker:
`malformed-build-script-metadata: build-script metadata line 75 is malformed: rustc-link-lib name must be a safe token`.
Cargo accepts richer `cargo:rustc-link-lib` forms from registry build scripts, including modifier-bearing static-link directives. Mantle currently treats those accepted link directives as malformed metadata, so topology stops before the next real Rust-unit frontier.

## What Changes

- Accept bounded Cargo-compatible `rustc-link-lib` metadata forms needed by current native topology, without allowing whitespace, paths, empty components, or arbitrary shell-like values.
- Preserve deterministic rejection for malformed link metadata.
- Add positive and negative parser tests, then rerun the focused parser tests and clean self-probe.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`
- **Testing**: focused metadata parser tests, clean native rust-plan self-probe, `cairn validate`, Cairn tasks gate.
