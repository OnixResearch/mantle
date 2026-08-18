## ADDED Requirements

### Requirement: Source-built provider snix sandbox-shell compile environment

r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env] Mantle MUST handle vendored `snix-build` sandbox-shell compile-time defaults without requiring undeclared ambient `SNIX_BUILD_SANDBOX_SHELL` during provider-backed Cargo-free topology execution under a source-built native closure.

#### Scenario: Missing compile-time shell env is a placeholder, not a proof claim

GIVEN provider-backed Cargo-free topology execution compiles vendored `snix-build` or Mantle diagnostics with `SNIX_BUILD_SANDBOX_SHELL` absent from the child environment
WHEN the code needs a compile-time sandbox-shell default
THEN Mantle MUST compile using an explicit placeholder default rather than failing at `env!("SNIX_BUILD_SANDBOX_SHELL")`.
AND Mantle MUST NOT claim the placeholder is a source-built sandbox shell or source-built runtime sandbox execution evidence.

#### Scenario: Runtime shell selection remains explicit

GIVEN a compiled Mantle binary later performs sandboxed builds
WHEN runtime `SNIX_BUILD_SANDBOX_SHELL` is set to a non-placeholder executable
THEN runtime shell selection MUST prefer that explicit runtime value over the compile-time placeholder.
AND absence of a runtime shell MUST remain bounded by existing runtime discovery or failure behavior rather than provider fixed-point proof success.

#### Scenario: Provider proof frontier is rerun honestly

GIVEN the compile-time sandbox-shell blocker has been removed
WHEN the provider-backed fixed-point proof is rerun from current code
THEN Mantle MUST record whether fixed-point succeeds or the next deterministic blocker appears.
AND any blocked run MUST retain bounded non-claims instead of reporting provider fixed-point release artifact evidence.
