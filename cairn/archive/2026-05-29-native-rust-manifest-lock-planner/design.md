# Design: Native manifest and lockfile planner

## Context

The current rust-plan pipeline relies on Cargo metadata. A real Cargo alternative needs a deterministic parser and resolver input model owned by Mantle.

## Decisions

### 1. Split pure parser from filesystem shell

**Choice:** Pure functions parse owned manifest/lockfile text into normalized package, workspace, dependency, and target facts. A thin shell walks files and reads text.

**Rationale:** Parser behavior can be tested without invoking Cargo or touching the filesystem.

### 2. Fail closed on unsupported manifest surface

**Choice:** Unsupported fields such as target-specific dependency tables, patch/replace/source replacement, or registry config emit explicit blockers until implemented.

**Rationale:** Silent partial parsing would create false Cargo-free claims.

### 3. Keep Cargo oracle comparison during migration

**Choice:** While Cargo is still available in dev validation, compare native facts against Cargo metadata and record mismatches.

**Rationale:** Oracle comparison gives bounded confidence without making Cargo a runtime dependency for the final mode.

## Risks / Trade-offs

- Cargo manifest semantics are broad; scope must stay explicit.
- Some ecosystem crates will stay blocked until target/config/source replacement support lands.
