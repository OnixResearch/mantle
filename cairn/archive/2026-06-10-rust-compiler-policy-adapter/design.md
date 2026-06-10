## Context

Mantle already plans Rust package units as explicit derivation nodes and executes supported units by invoking `rustc` with receipt-owned arguments, environment, inputs, and outputs. Octet currently owns the rule authority: lint definitions, Dylint driver integration, policy registry, source-shape standards checks, and suppression rules. The integration should therefore be an adapter boundary, not a direct dependency on Octet's rustc-private lint crate.

## Approach

1. Add a pure compiler-policy adapter model to Rust planning. The core model resolves a base compiler path, unit args, adapter selection, and policy mode into a deterministic compiler invocation description plus an adapter identity object.
2. Keep the imperative shell thin. Filesystem checks, executable discovery, digesting adapter files, command construction, and process execution remain in the Rust unit execution shell.
3. Treat Octet as one adapter provider. The Octet adapter consumes a provider manifest or explicit paths for the Dylint driver, lint library, lint config, standards policy artifact, and profile metadata. The adapter invokes the driver for direct-rustc execution and applies the required environment such as `DYLINT_LIBS` and `DYLINT_NO_DEPS`.
4. Add policy modes:
   - `plain`: no adapter claim.
   - `audit`: run adapter and receipt findings where available without failing the unit on warnings.
   - `deny`: fail the unit when configured Octet lints or standards checks emit error-level findings.
   - `required`: fail before compilation if any adapter artifact, config, digest, scope, or standards gate input is absent or inconsistent.
5. Bind the adapter identity into output reuse. Reuse of prior unit outputs is allowed only when the base toolchain identity, source/dependency digests, rustc args/env digests, and compiler-policy adapter identity all match.
6. Keep source-shape standards separate from compiler lints. Mantle may run an Octet standards gate before accepting a topology, but it should receipt that gate as policy evidence rather than folding source-shape checks into rustc execution.

## Interface Sketch

```rust
pub(crate) enum RustCompilerPolicyKind {
    Plain,
    OctetDylint,
}

pub(crate) enum RustCompilerPolicyMode {
    Audit,
    Deny,
    Required,
}

pub(crate) struct RustCompilerPolicyConfig {
    pub(crate) kind: RustCompilerPolicyKind,
    pub(crate) mode: RustCompilerPolicyMode,
    pub(crate) provider_manifest: Option<PathBuf>,
}

pub(crate) struct RustCompilerPolicyIdentity {
    pub(crate) kind: String,
    pub(crate) mode: String,
    pub(crate) provider_manifest_blake3: Option<String>,
    pub(crate) driver_blake3: Option<String>,
    pub(crate) lint_lib_blake3: Option<String>,
    pub(crate) config_blake3: Option<String>,
    pub(crate) standards_policy_blake3: Option<String>,
}
```

The exact Rust API may differ, but the boundary must stay generic: Mantle owns invocation, cache identity, and receipts; Octet owns rules and provider artifacts.

## Risks

- Dylint/rustc-private compatibility is toolchain-sensitive. Required mode must fail closed when the Octet provider does not match the selected rustc.
- Lint-level and source suppression behavior can accidentally weaken the proof. Mantle must receipt the exact config and Octet must own structured-waiver validation.
- Direct-rustc execution differs from Cargo wrapper execution. Mantle should validate the direct-driver invocation path with focused fixtures before promoting the mode to self-build or fixed-point proofs.
- Overclaiming is easy. Receipts must say "compiled under configured policy profile" rather than "program is safe" or "architecture is fully proved."

## Validation

- Pure adapter-resolution tests cover plain, audit, deny, required, missing artifact, digest mismatch, and cache-key distinction.
- Execution tests prove required mode invokes the adapter before rustc output acceptance and refuses raw-rustc output reuse.
- Receipt tests prove adapter identity appears in successful, failed, and reused unit receipts.
- CLI tests prove operator flags select the intended policy mode without affecting default plain execution.
