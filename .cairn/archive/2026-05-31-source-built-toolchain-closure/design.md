# Design: Source-built toolchain closure proof

## Context

The current first-class fixed-point command proves a bounded native Rust topology fixed point: host Mantle builds stage1, stage1 builds stage2, Cargo is guarded, and the two Mantle binaries match. The proof still depends on a host-selected Rust toolchain and records `not-source-built-toolchain-closure`.

This change defines the next claim boundary. It does not claim Mantle can already build Rust from source. It makes the acceptance criteria explicit so later implementation cannot accidentally promote host-tool or placeholder evidence into a source-built-toolchain claim.

## Decisions

### 1. Treat the toolchain as a closure, not one `rustc` path

**Choice:** The proof bundle records a `toolchain_closure` made of compiler, linker, C toolchain, sysroot/crt objects, native helper tools, and dynamic/static runtime inputs. Each member has a logical role, source identity, build receipt identity, binary/content BLAKE3 digest, execution path, and trust classification.

**Rationale:** A `rustc` binary alone is not enough. Native Rust units also depend on linker behavior, C toolchain behavior, sysroot contents, and build-script helper tools. The proof claim must match what actually influenced outputs.

### 2. Fail closed on ambient host tools

**Choice:** Source-built closure mode rejects ambient host `rustc`, Cargo, linkers, C compilers, pkg-config, Nix profile tools, and undeclared PATH helpers unless they are listed as explicit seed exceptions in the closure manifest.

**Rationale:** Current fixed-point proof already guards Cargo. The stronger claim must also prevent host compiler/linker leakage from being mistaken for source-built provenance.

### 3. Keep seed exceptions explicit and bounded

**Choice:** The closure may contain an initial seed/trust root, but every seed exception must be named, hashed with BLAKE3, justified, and excluded from the source-built portion of the claim. Completion requires a real normalized seed/source-root provider, not a placeholder.

**Rationale:** Bootstrap starts from some trust root. Hiding that root creates a false stronger claim. Naming it makes the remaining source-built closure reviewable.

### 4. Reuse fixed-point stages with stricter toolchain input policy

**Choice:** The source-built proof extends the existing `mantle self-build --cargo-free --fixed-point --out <bundle-dir>` stage model with a receipt-bound toolchain selection. Stage1 and stage2 must both use the same closure policy, and the final success condition remains matching stage binary BLAKE3 digests.

**Rationale:** This avoids creating another unrelated proof rail. The existing fixed-point rail already has durable stage receipts, smoke outputs, binary digests, Cargo guards, and non-claims.

### 5. Separate proof metadata from implementation shortcuts

**Choice:** Fixture tests may use tiny fake toolchain closures for parser/validator behavior, but any task claiming source-built closure completion must cite a real end-to-end proof bundle with real built tools.

**Rationale:** Unit tests can prove fail-closed logic; they cannot prove real compiler provenance.

## Risks / Trade-offs

- Real Rust compiler/toolchain source builds are expensive; this change must preserve smaller negative/fixture tests for fast iteration.
- Seed provenance can become vague unless the manifest schema forces explicit trust classifications and digest checks.
- Toolchain-path normalization must not rewrite host paths into proof-looking store paths unless receipt digests prove the content and source provenance.
- Current fixed-point success remains a bounded proof only; this change must not weaken or relabel existing evidence.
