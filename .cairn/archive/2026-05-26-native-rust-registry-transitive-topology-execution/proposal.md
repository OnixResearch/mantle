# Native Rust registry transitive topology execution

## Summary

Extend Mantle's bounded native registry topology execution proof from a single vendored registry package to a transitive vendored registry closure.

The current registry slices prove local roots can execute topology graphs with one vendored registry library, one registry `custom-build` host producer, or one registry proc-macro host producer. This change proves the next package-planning boundary: a local root depending on a registry package that itself depends on another registry package, with both registry source facts ready and all dependency artifacts bound through explicit topology receipts.

## Motivation

Real registry packages rarely appear as isolated leaves. Even tiny crates commonly depend on another registry crate. Mantle needs evidence that ready vendored registry source facts compose across a small registry closure without falling back to Cargo orchestration, Cargo registry caches, network access, or version solving.

## Scope

- Model a bounded topology shape: `local root -> registry package A -> registry package B`.
- Require ready native registry source facts for every registry package in the executed graph.
- Execute registry package B before registry package A, then execute the local root target.
- Bind produced registry dependency artifacts into downstream `rustc` material and receipts.
- Preserve deterministic BLAKE3 source/output/dependency artifact evidence for each executed unit.
- Emit deterministic pre-`rustc` blockers for missing, stale, unsupported, or ambient-cache-dependent transitive registry material.

## Non-goals

- No Cargo orchestration.
- No network/index fetch.
- No `$CARGO_HOME` or ambient registry cache fallback.
- No version solving.
- No broad Cargo registry compatibility claim.
- No feature unification or complex registry resolver behavior beyond the bounded vendored closure fixture.
- No new host-artifact semantics beyond existing supported build-script/proc-macro rails.
