# Design

## Boundary

The functional core extends existing planning summaries and metadata parsing with deterministic data: package `links`, package custom-build path, generic build-script metadata, and explicit metadata dependency edges. The imperative shell remains limited to executing rustc/build-script subprocesses and applying the already-derived environment.

## Approach

1. Extend manifest parsing so `[package] build = "..."` creates a custom-build target with Cargo-compatible `build-script-<stem>` naming; `[package] links = "..."` is recorded on package facts.
2. Extend build-script metadata parsing to store safe non-rustc `cargo:key=value` lines while continuing to handle rustc cfg/env/link/search and rerun directives.
3. Add metadata dependency facts for custom-build units whose immediate normal dependencies have `links` metadata.
4. In combined topology ordering, add host edges from dependent build scripts to linked dependency custom-build producers.
5. Before running a build script, translate linked dependency metadata to `DEP_<LINKS>_<KEY>` env vars, uppercasing and underscore-normalizing the manifest links/key tokens.
6. Prove with focused positive/negative tests and a clean self-probe.

## Risks

- Not every `cargo:` line is dependency metadata. The parser should ignore known non-metadata control lines and validate keys before exporting.
- Manifest-only dependency edges can over-approximate Cargo. For this slice, metadata dependencies are limited to immediate normal dependencies with `links` package metadata.
- Running linked dependency build scripts may expose the next missing host-tool/env blocker before `aws-lc-rs` reaches success; evidence must record the new deterministic frontier without claiming broader Cargo parity.
