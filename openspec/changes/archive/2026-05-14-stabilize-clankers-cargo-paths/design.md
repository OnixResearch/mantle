## Context

Commit `d225be90` recorded a successful fresh rebuild with verdict `mismatch`:

- original binary BLAKE3: `e1e8e1c36e0979a2534bcb8c394d4c360068985b0700b0e73ee32f1bd23917ff`
- rebuilt binary BLAKE3: `82e569fcb115baf84efba01930b30085f0556d6ec9c3bc7c5ef45379e6aa5863`
- first observed differing path: `/tmp/cargo-target/x86_64-unknown-linux-musl/release/build/cranelift-codegen-<hash>/out/inst_builder.rs`

That path is generated below Cargo's build-script `OUT_DIR`, and it can leak into final Rust binaries via generated source spans/panic metadata even when the ELF is stripped.

## Goals / Non-Goals

**Goals:**
- Remove or normalize embedded `/tmp/cargo-target/.../build/<crate>-<hash>/out/...` paths from the Clankers output binary.
- Prove the fixed derivation rebuilds to a stable binary BLAKE3 across at least two fresh stores.
- Record fail-closed evidence if a mismatch remains.

**Non-Goals:**
- Change Clankers features or dependency set.
- Prove cross-machine reproducibility.
- Claim no-Nix bootstrap; `nix-shell` may still provide outer host tools.

## Decisions

### 1. Use Rust path remapping at the derivation boundary

**Choice:** Add `--remap-path-prefix` flags through `RUSTFLAGS` for `/tmp/build`, `/tmp/cargo-home`, `/tmp/cargo-target`, `/build`, and relevant logical input roots.

**Rationale:** The nondeterministic evidence is an embedded Rust source path. Rust's remap flag is intended for reproducible builds and should normalize source/debug/panic paths without patching upstream crates.

**Alternative:** Patch Wasmtime/Cranelift build scripts. Rejected for this slice because it is dependency-specific and less reusable.

### 2. Verify by fresh-store pair, not against stale original only

**Choice:** After the fix, compare two fresh rebuilt binaries from the same committed derivation. If they match, update proof metadata to the new stable digest and record the previous mismatch as historical evidence.

**Rationale:** The original binary was built before the reproducibility fix. A deterministic fix may intentionally produce a new binary digest; the proof should bind the current deterministic derivation, not hide the prior mismatch.

## Risks / Trade-offs

**Remap insufficient** → Record mismatch and the next embedded-difference source rather than updating the proof.

**Long build time** → Use managed background execution and reuse the existing helper script pattern.
