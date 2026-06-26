## Context

A fresh provider-backed fixed-point proof now reaches the vendored `snix-build` library and blocks in direct `rustc` execution because `vendor/snix-build/src/buildservice/{bwrap,oci}.rs` use `env!("SNIX_BUILD_SANDBOX_SHELL")`. The direct Cargo-free topology runner deliberately clears ambient environment and forwards only explicit, bounded values, so this compile-time requirement is not available unless Mantle smuggles in caller state.

`snix-build` already treats `/bin/sh` as a placeholder at runtime in the bwrap backend: runtime `SNIX_BUILD_SANDBOX_SHELL` wins, non-placeholder compile defaults are used only if they still exist, and otherwise it searches for static busybox before falling back to `/bin/sh`. The proof only needs compilation to avoid an undeclared ambient compile-time dependency; it does not prove runtime sandbox-shell provenance.

## Decisions

### 1. Make compile-time shell defaults optional

**Choice:** Use `option_env!("SNIX_BUILD_SANDBOX_SHELL")` with `/bin/sh` as the placeholder when the variable is absent.

**Rationale:** This preserves the existing placeholder semantics while allowing direct rustc topology execution to compile the crate without requiring undeclared caller environment.

### 2. Do not claim a source-built sandbox shell from the placeholder

**Choice:** The placeholder is not a source-built toolchain member and is not recorded as a successful source-built sandbox-shell claim.

**Rationale:** The native closure currently binds compilers, linker, sysroot, target helpers, and runtime libraries, not a sandbox shell. Treating `/bin/sh` as a proven source-built shell would overclaim.

### 3. Keep runtime selection behavior separate from compile-time availability

**Choice:** Compile-time optionality only removes the `env!` compilation blocker. Runtime sandbox execution continues to prefer runtime `SNIX_BUILD_SANDBOX_SHELL` and existing discovery/failure paths.

**Rationale:** The provider fixed-point proof is a Cargo-free topology build proof. It must not silently convert compile success into a broader sandbox execution or deployability claim.

## Risks / Trade-offs

- The proof may move to another deterministic Rust topology blocker after `snix-build` compiles; that is expected and must be recorded honestly.
- Runtime sandbox shell provenance remains a separate problem if future release claims require source-built sandbox execution rather than compile-only topology evidence.
