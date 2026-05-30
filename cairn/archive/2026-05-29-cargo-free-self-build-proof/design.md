# Design: Cargo-free self-build proof

## Context

The bounded Cargo-free proof exercises only a generated path workspace. The real Mantle workspace also needs declared local registry sources and captured git sources from `vendor-deps/`. Cargo-free mode must derive those from checked-in manifests, `Cargo.lock`, and declared vendor roots, not from `cargo metadata`, `cargo build --unit-graph`, network, or the ambient Cargo cache.

## Decisions

### 1. Native lockfile source packages feed no-Cargo planning

**Choice:** In `--no-cargo-oracle` mode, synthesize registry/git package identities from `Cargo.lock`, bind registry packages to declared vendor roots with checksum validation, bind git packages to captured local source material, then feed those manifests into native package and unit graph planning.

**Rationale:** The self-build proof must not call Cargo, but it still needs the full checked-in source closure.

### 2. Proof runner owns the imperative shell

**Choice:** Keep filesystem setup, failing Cargo shim, tool discovery, command execution, and smoke checks in `scripts/prove-cargo-free-rust-plan.sh`. Keep source binding and graph construction in pure Rust planner functions.

**Rationale:** The core remains testable with in-memory/tempdir fixtures; the shell only orchestrates evidence capture.

### 3. Self-build proof is bounded evidence

**Choice:** A successful self-build proof can claim Mantle's native Rust topology built and smoke-checked a Mantle CLI binary with Cargo forbidden. It must not claim Crunch fixed-point self-hosting, source-built bootstrap roots, or release reproducibility.

**Rationale:** This is the next proof rung, not the final bootstrap proof.

## Risks / Trade-offs

- Native feature resolution remains bounded and may expose blockers before a full self-build succeeds.
- Host toolchain/linker availability still matters for direct `rustc` execution.
- Vendored source binding is intentionally fail-closed; missing or mismatched local source material blocks the proof.
