# Native Rust registry topology execution

## Summary

Mantle now records supported registry-backed Rust package sources as explicit native source facts from `Cargo.lock` plus declared local/vendor source roots. The next boundary is to consume those facts in the native unit/topology execution rail so vendored registry dependencies can participate in Cargo-free execution without falling back to `$CARGO_HOME`, Cargo registry caches, network fetches, or Cargo orchestration.

## Why

The current registry work proves source evidence exists, but the execution claim remains bounded unless downstream package, unit graph, derivation, and topology execution receipts can prove they used that evidence. A vendored registry dependency should be executable like a path dependency only after Mantle has bound its lockfile identity, checksum, declared source root, source digest, oracle comparison, and explicit rustc material.

## Scope

- Thread ready `native_registry_source_planning` facts into native package/target, unit graph, derivation, and `rust-plan --execute-topology` evidence for the supported vendored registry dependency shape.
- Execute a bounded topology containing a local root crate and one or more vendored registry-backed `lib` dependencies from explicit Mantle receipts.
- Preserve deterministic blockers for missing/stale vendor material, missing checksum evidence, unsupported registry layouts, source/oracle mismatches, and any attempted ambient Cargo cache fallback.
- Add focused CLI JSON fixtures proving successful registry-backed topology execution and fail-closed missing/stale vendor behavior.

## Non-goals

- No network fetch, remote registry protocol, Cargo cache lookup, or `$CARGO_HOME` fallback.
- No version solving beyond the retained lockfile identity.
- No broad Cargo ecosystem compatibility claim.
- No general scheduler beyond the existing bounded topology execution rail.
- No expansion to unsupported target kinds, tests/doctests/examples, git registry source layouts, or native-link probing unless separate Cairn changes cover them.
