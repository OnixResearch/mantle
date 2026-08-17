## Context

`build_script_child_env` currently uses `rust_crate_name(&unit.target_name)` for `CARGO_PKG_NAME`. For native custom-build targets the target name is normally `build-script-build`, so the env value becomes `build_script_build` rather than the manifest package name.

## Decisions

### 1. Package name rides in derivation env

**Choice:** Store package-derived `CARGO_PKG_NAME` in the reviewable derivation env when planning native target and host units, then let build-script execution reuse that value.

**Rationale:** The package name is already known at native planning time. Keeping it in the derivation env avoids reparsing package IDs and makes the build-script env helper pure and focused.

### 2. Deterministic fallback remains target-derived

**Choice:** If a legacy or Cargo-oracle unit lacks package-name env data, continue to fall back to the old deterministic `rust_crate_name(target_name)` value.

**Rationale:** This keeps existing non-native unit summaries executable while making native units match Cargo semantics.

## Risks / Trade-offs

- Derivation env/digests change for native units because package-name env becomes explicit.
- This does not add every Cargo package metadata variable; only the reviewed `CARGO_PKG_NAME` mismatch is in scope.
