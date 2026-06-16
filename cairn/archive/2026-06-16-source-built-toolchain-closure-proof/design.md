## Context

`--rust-source-provider` already validates source-built Rust provider metadata, rejects prebuilt Rust provenance, resolves the provider `bin/rustc`, and records a separate `rust_source_provider` block. `source_built_toolchain_closure`, however, is driven only by `--toolchain-closure`; when no closure manifest is supplied it stays `status: not-provided` and keeps the stale `not-source-built-toolchain-closure` non-claim even if the Rust provider is validated and used for both fixed-point stages.

## Decisions

### 1. Provider-derived closure status

**Choice:** Add a pure status constructor that derives `source_built_toolchain_closure.status = provided` from validated Rust provider metadata.

**Rationale:** The Rust provider validator is already the authority for Rust compiler/sysroot source provenance. Reusing its policy digest, metadata path, artifact count, and source-built/prebuilt checks avoids inventing a parallel manifest while still keeping the broader explicit `--toolchain-closure` enforcement path intact.

### 2. Manifest closure still wins when supplied

**Choice:** If `--toolchain-closure` is provided, keep the existing manifest validation/enforcement semantics and policy digest comparison.

**Rationale:** The explicit closure manifest covers broader compiler, linker, C toolchain, sysroot, crt, runtime, and helper roles. A Rust provider must not weaken that stricter path.

### 3. Non-claims are status-derived

**Choice:** Generate one-shot and fixed-point non-claims from the effective `SourceBuiltToolchainClosureStatus` instead of a fixed list.

**Rationale:** Absent or invalid provider evidence must retain `not-source-built-toolchain-closure`, while a validated provider-backed claim must remove only that stale non-claim and retain unrelated bounds such as not release reproducibility and not full Cargo compatibility.

## Risks / Trade-offs

- Provider-backed status proves the Rust compiler/sysroot provider, not a fully minimized bootstrap trust root or release reproducibility. Those remain explicit non-claims until separate evidence exists.
- The existing explicit closure manifest remains the path for broader native toolchain closure enforcement; this change does not replace it.
