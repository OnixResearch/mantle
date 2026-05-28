## Context

`bind_all_host_artifacts()` rewrites a host artifact into a target dependency only when the target already has a matching `RustDependencyArtifact` placeholder. The clean profile-env probe shows `darling@0.20.11` has `consumed_host_artifacts` for `darling_macro`, but its dependency artifacts include only `darling_core`. The target rustc args therefore never receive `--extern darling_macro=...`.

## Decisions

### 1. Bind missing proc-macro externs at host-artifact binding time

**Choice:** Extend `bind_all_host_artifacts()` so each consumed host artifact ensures a matching `--extern <crate>=<produced-host-path>` exists after the artifact path is known.

**Rationale:** The binding point already has the produced host artifact path, target derivation, and host artifact crate name. This keeps graph planning unchanged and fixes execution without inventing fake target dependency artifacts.

### 2. Preserve dependency placeholder rewrite semantics

**Choice:** Keep the current dependency placeholder rewrite when a matching dependency artifact exists, and only append the host-artifact extern when the bound args do not already contain it.

**Rationale:** Existing proc-macro tests and receipts with explicit dependency placeholders remain stable. The new behavior fills only the missing target rustc surface.

### 3. Keep fail-closed material checks unchanged

**Choice:** Do not weaken host artifact digest checks or producer ordering. The target still blocks before rustc if the declared consumed host artifact is missing.

**Rationale:** The fix must not search Cargo caches or sysroot paths to repair missing proc-macro material.

## Risks / Trade-offs

- Some Cargo unit shapes may expose proc-macro dependencies through dependency artifacts and some only through host artifacts. The no-duplicate test prevents double `--extern` args when both surfaces are present.
- This does not claim broad proc-macro/Cargo compatibility; it only binds already-planned consumed host artifacts into rustc args.
