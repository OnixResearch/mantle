## Context

`--rust-source-provider` can now bind a source-built Rust compiler/sysroot provider and remove `not-source-built-toolchain-closure` when no explicit toolchain closure is supplied. However, when an operator supplies `--toolchain-closure`, that manifest is intentionally authoritative. Today all explicit manifests report `claim = false`, even if the manifest is enforced, contains no seed exceptions, and all members are classified as source-built.

The next native closure step needs two pieces: a code path that can honestly promote a complete explicit closure, and evidence showing whether the current host/target native tools can satisfy that criterion.

## Decisions

### 1. Promotion is derived from validation accounting

**Choice:** `enforced_source_built_toolchain_closure(...)` returns a claim only when:

- enforcement observed the requested Rust/tool inputs,
- `seed_exception_count == 0`, and
- `source_built_member_count == member_count`.

**Rationale:** The manifest validator already normalizes and counts source-built vs seed-exception members. Promotion should be a deterministic pure-core decision, not a string check or caller convention.

### 2. Validated-only manifests never claim

**Choice:** A manifest that was parsed/validated but not enforced stays `claim = false`, even with zero seed exceptions.

**Rationale:** Parsing a manifest proves shape only. The claim requires checking that the execution inputs match the declared closure.

### 3. Any seed exception preserves the non-claim

**Choice:** If any member is a seed exception, Mantle keeps `not-source-built-toolchain-closure` and reports the seed count.

**Rationale:** This keeps the existing musl-target proof honest: target aliases are proven, but native host C/linker/sysroot closure is not fully source-built yet.

### 4. Current blocker evidence is first-class

**Choice:** The change records the no-seed/native proof frontier if the real host GNU compiler/libc/linker closure is unavailable.

**Rationale:** A blocked proof with exact evidence is better than relabeling Nix clang/glibc or a wrapper as source-built. It preserves the next implementation seam: source-built host-compatible C/linker/libc closure or a musl-host Rust provider that can run proc macros without GNU host linking.

## Risks / Trade-offs

- A zero-seed manifest still depends on the manifest's source/build-receipt identities being true; this change does not magically audit arbitrary external receipts.
- Full no-seed closure may remain blocked until Mantle has source-built host GNU native tooling or changes the Rust provider topology to a host ABI that can run all host units without GNU seed tools.
