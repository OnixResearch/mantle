# Design: Build-script runtime parity

## Context

Build scripts are host programs whose outputs affect later rustc invocations. A Cargo alternative must make this behavior explicit and deterministic.

## Decisions

### 1. Define an allowlisted Cargo environment contract

**Choice:** Provide only documented/required `CARGO_*`, profile, target cfg, `OUT_DIR`, and manifest fields from selected native facts.

**Rationale:** Broad ambient env inheritance breaks reproducibility and leaks host state.

### 2. Parse build-script output into typed metadata

**Choice:** Convert `cargo:` / `cargo::` lines into typed receipt fields for rustc cfg/env/link/search/rerun metadata, rejecting malformed or unsupported lines deterministically.

**Rationale:** Stringly metadata should not flow unvalidated into rustc.

### 3. Keep native tool probing explicit

**Choice:** For helper crates such as `cc-rs`, expose target/profile/tool env through declared inputs; unsupported host probing fails closed.

**Rationale:** Native C toolchains are a separate trust boundary.

## Risks / Trade-offs

- More crates will run, but native-link correctness remains bounded by explicit toolchain support.
- Some build scripts intentionally inspect host state; these must stay blockers unless modeled.
