# Proposal: Preserve transitive rustc search paths for native topology execution

## Problem

Native topology execution binds direct dependency artifacts by package ID, but the `-L dependency=...` search path set is also derived from package-keyed maps. When Cargo has multiple unit variants for the same package, later variants can overwrite earlier artifact directories. A downstream crate can then receive the correct direct `--extern` path but lose a transitive crate metadata search path needed to load that rlib.

The current clean Mantle self-probe reaches `crunch-system` and fails with `can't find crate for crunch_glue`; reproducing the command succeeds when every previously produced lib/proc-macro artifact directory remains on `-L dependency`, and fails with the package-keyed map-only search set.

## Change

Track all produced target-lib and proc-macro artifact paths as search-path material, while retaining package-keyed maps only for direct artifact binding.

## Success criteria

- Direct dependency binding remains package-keyed and deterministic.
- `-L dependency` includes all produced artifact parent directories needed for transitive metadata loading, including earlier same-package variants.
- Duplicate search paths are not emitted repeatedly.
- Focused tests cover retained duplicate-variant search paths and deduplication.
- Clean self-probe advances past the `crunch_glue` blocker or reaches a later deterministic blocker.
