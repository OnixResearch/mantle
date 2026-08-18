## Context

`build_script_child_env()` currently passes `OUT_DIR`, `CARGO_MANIFEST_DIR`, `CARGO_PKG_*`, `RUSTC`, `HOST`, `TARGET`, `PROFILE`, and linked dependency `DEP_*` values. Cargo also exposes target cfg variables to build scripts as `CARGO_CFG_*`. Real build scripts use those values for target-specific source generation; `aws-lc-sys` immediately requires `CARGO_CFG_TARGET_ARCH`.

## Decisions

### 1. Derive target cfg env in pure helpers

**Choice:** Add pure helpers that parse Mantle's active target triple and return a bounded `BTreeMap<String, String>` of Cargo-compatible cfg env values.

**Rationale:** Keeps target cfg logic testable without launching build scripts or reading ambient Cargo env.

### 2. Support common host/bootstrap target classes explicitly

**Choice:** Cover the triples used by current native topology and tests (`x86_64-unknown-linux-gnu`, `wasm32-unknown-unknown`) plus adjacent common Rust triples where the mapping is unambiguous. Unknown fields become deterministic empty strings where Cargo-compatible.

**Rationale:** This fixes the known blocker without pretending full target-spec parity. Future target-specific cfgs can be added with evidence.

### 3. Runtime-only for build scripts

**Choice:** Feed `CARGO_CFG_*` into build-script child execution, not rustc compilation env.

**Rationale:** The current blocker is runtime `std::env::var("CARGO_CFG_TARGET_ARCH")`; rustc compile-time cfg already comes from rustc target handling and explicit args.

## Risks / Trade-offs

- The env surface is intentionally bounded and does not yet include every Cargo/rustc cfg such as atomics, features, or panic strategy.
- Incorrect triple parsing could mislead target-specific build scripts; focused tests pin current known triples.
