# Proposal: Compile foreign derivation graphs into exact Mantle units

## Why

Mantle can admit and plan foreign derivation graphs. It cannot yet compile those graphs into executable native units.

The current translation sorts nodes by ID and replaces store prefixes as text. It also assigns synthetic output identities. A parent can therefore reference a path that does not match the recomputed child path.

Mantle needs a pure, bounded compiler for concrete Guix-like and Nix-like derivation graphs. The compiler must preserve source-format facts while producing exact Mantle target identities.

## What Changes

- Add a prefix-aware ATerm producer for explicit foreign `.drv` bundles.
- Add bounded dependency ordering with explicit cycle and missing-input failures.
- Compile each node after its dependencies and record exact derivation, output, and source path mappings.
- Lower supported foreign fetch builtins without invoking Guix, Nix, or foreign frontend processes.
- Emit `mantle-foreign-executable-plan-v1` with native unit facts, target roots, path maps, policy identities, and non-claims.
- Preserve foreign SHA-256 interoperability facts while using BLAKE3 for Mantle-owned target and receipt identities.

## Dependencies

This change extends the accepted `foreign-derivation-import` specification. It has no dependency on another active change.

## Non-Goals

- Running builders or mutating Mantle store state.
- Acquiring source bytes from networks or source bundles.
- Relaxing sandbox behavior.
- Auditing realized output contents.
- Evaluating Guix packages, Nix expressions, flakes, overlays, or OS configuration.

## Impact

- **Files**: `src/foreign_derivation_import.rs`, `src/foreign_import_cmd.rs`, `crates/crunch-glue/`, foreign fixtures, CLI tests, documentation, and lifecycle evidence.
- **Testing**: prefix-aware ATerm parsing, graph ordering, cycle rejection, exact path mapping, builtin lowering, hash-domain separation, deterministic plan identity, and CLI drift guards.
