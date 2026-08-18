## Context

Current Rust planning captures `cargo metadata` and `cargo build --unit-graph` as oracle material, then derives source closures, unit derivations, execution receipts, topology receipts, and reuse receipts from those normalized inputs. That gives strong downstream evidence, but it does not yet prove Mantle can own the package/target planning step that Cargo normally performs.

This change keeps the oracle during the transition and introduces only the first native fragment: package manifest and target facts for a tiny supported subset. The goal is not full Cargo compatibility; it is a reviewable seam where Mantle computes a deterministic fragment, compares it to Cargo, and refuses unsupported or divergent inputs.

## Decisions

### 1. Start with a package/target fragment, not a scheduler

**Choice:** Implement native planning for simple workspace/package facts before expanding graph scheduling: package identity, manifest path, target names/kinds/src paths, selected features for the supported default/no-extra-feature subset, and path-source closure linkage.

**Rationale:** The execution topology already exists. Owning the earliest planner facts removes a Cargo dependency at the root without perturbing the verified execution rails.

### 2. Keep Cargo oracle as comparison evidence

**Choice:** Retain Cargo metadata/unit-graph capture for this slice and compare Mantle-computed fragments against the oracle for the same root/profile/features.

**Rationale:** This gives deterministic review evidence while avoiding overclaiming that Cargo is gone. A mismatch is a planning blocker, not a fallback to Cargo truth.

### 3. Fail closed on unsupported shapes

**Choice:** Unsupported target kinds/modes, build scripts/proc macros in the native fragment, workspace inheritance/features outside the bounded subset, missing manifests, unreadable source paths, or mismatched oracle facts produce structured blockers before emitting a Cargo-free planner claim.

**Rationale:** The replacement path must remain bounded and explicit. Silent `Ok(None)` or hidden Cargo fallback would undermine the existing receipt model.

### 4. Receipt-first integration

**Choice:** Surface native planner fragment facts and blockers in `rust-plan` JSON before using them as the sole source for later unit derivation graph construction.

**Rationale:** The first slice should be inspectable and easy to compare. Later changes can switch downstream derivation construction to require the Mantle-owned fragment once the evidence is stable.

## Risks / Trade-offs

- The first supported subset is intentionally small and may reject real Cargo workspaces that current oracle-backed planning can inspect.
- Comparing against Cargo keeps Cargo present for this slice, so it is replacement evidence rather than a full Cargo-free planning path.
- Feature resolution can expand quickly; keep this change to default/simple feature facts and record unsupported feature surfaces as blockers.
