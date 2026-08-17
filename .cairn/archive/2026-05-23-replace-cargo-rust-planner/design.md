## Context

Mantle's package build path is already a native derivation pipeline:

```text
Nickel expression
  -> CrunchDerivation record
  -> nix_compat::Derivation
  -> BuildRequest
  -> fetch service or sandboxed build service
  -> snix store/pathinfo/export
```

Rust package support should fit that model by producing explicit derivations. `unit2nix` is a useful transitional reference because it asks Cargo for the true unit graph and serializes a reviewable plan. Anthropic's `cargo-nix-plugin` is a useful proof that native Cargo resolution can be embedded elsewhere, but its Nix evaluator/plugin integration is the wrong seam for Mantle.

The target state is for Mantle to replace Cargo as planner/orchestrator while continuing to use the Rust toolchain (`rustc`, standard library, linker, and target support) as compiler/runtime inputs. Cargo remains an oracle during the compatibility ramp, not a permanent hidden build engine.

## Decisions

### 1. Keep Mantle core language-neutral

**Choice:** Implement Rust package support as a frontend/planner that emits ordinary Mantle derivations and receipts.

**Rationale:** The core builder should build declared derivations; it should not acquire Cargo-specific dependency resolution, feature, or manifest semantics.

### 2. Use Cargo as an oracle before replacing it

**Choice:** The first implementation slice records `cargo metadata` plus `cargo build --unit-graph` output, normalizes it into a Mantle Rust plan, and compares Mantle-computed graph fragments against Cargo until parity is proven.

**Rationale:** Cargo compatibility is subtle. Oracle parity gives a bounded correctness ladder and prevents Mantle from silently inventing incompatible feature, cfg, target, proc-macro, or build-script behavior.

### 3. Own the source closure explicitly

**Choice:** Mantle's Rust planner must represent registry, git, and path dependencies as explicit source inputs with lockfile identities and content hashes before those sources become build units.

**Rationale:** Build proofs need source receipts, not just package names. Offline and remote builds also need a source closure independent of ambient Cargo caches.

### 4. Emit one derivation per Rust unit

**Choice:** The planner emits distinct Mantle derivations for libraries, binaries, tests/examples as supported, build scripts, proc macros, and generated build-script output consumers.

**Rationale:** Per-unit derivations enable cache reuse, transparent host/target separation, precise rebuild reasons, and reviewable `rustc` invocations.

### 5. Fail closed on unsupported Cargo behavior

**Choice:** Unsupported features such as complex build-script output, cfg-dependent target discovery, native link probing, workspace inheritance edge cases, doctests, or unstable Cargo behavior must be reported as deterministic unsupported statuses until implemented.

**Rationale:** Mantle should not overclaim Cargo replacement. A package that crosses an unsupported boundary must produce a clear blocker receipt rather than falling back to hidden Cargo execution.

## Risks / Trade-offs

- Full Cargo compatibility is large, especially resolver v2 features, build scripts, proc macros, target-specific dependencies, native linking metadata, and exact `rustc` flags.
- Initial oracle-driven plans still require Cargo and nightly unit-graph support, so early phases improve reviewability before they eliminate Cargo.
- Per-unit derivations may expose more graph nodes and require careful receipt compression for human review.
- Replacing Cargo's planner does not prove compiler correctness; `rustc` remains a trusted compiler/toolchain input unless separately proven.
