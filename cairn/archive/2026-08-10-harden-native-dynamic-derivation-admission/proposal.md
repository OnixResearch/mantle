# Change: Harden native dynamic derivation admission

## Why

Mantle detects traditional `Derive(...)` files after builds and registers them as new scheduler goals. The current path parses and registers one mutable derivation value, supports only the traditional prefix, and substitutes an all-zero parent hash when a referenced parent is unknown.

That fallback creates a deterministic but unsupported identity. It can schedule work under an output identity that does not match the declared parent graph.

The native path also needs explicit states for syntax, semantic validity, parent-hash completeness, and registry eligibility. These rules belong to Mantle because native derivations use BLAKE3 and configurable logical store prefixes.

## What Changes

- Add a pure staged admission model for native dynamic derivations.
- Keep castore reads, output discovery, logging, registry mutation, and scheduler effects in the Worker shell.
- Detect supported traditional and versioned derivation prefixes under named byte and depth bounds.
- Separate parsed, validated, identity-resolved, and registry-ready states with private Rust types.
- Require explicit parent derivation hash facts before identity resolution.
- Remove the all-zero unknown-parent fallback.
- Preserve Mantle-native BLAKE3 hashing and explicit logical store prefixes.
- Classify unsupported output forms and version tags before registry mutation.
- Add positive and negative tests for every state transition and failure boundary.

## Non-Goals

- Replacing the complete native derivation, store, scheduler, or evaluator model.
- Using Nix SHA-256 derivation identity for native Mantle paths.
- Adding full import-from-derivation evaluator suspension.
- Treating every Nix versioned derivation feature as supported execution behavior.
- Claiming that typed admission proves builder correctness, output correctness, or release eligibility.

## Impact

- **New spec:** `dynamic-derivation-admission`.
- **Planned code:** `crates/crunch-build/src/dynamic.rs`, a pure dynamic-admission core module, registry interfaces, Worker orchestration, tests, ADR 0002, and evidence.
- **Compatibility:** valid covered native dynamic derivations retain their configured-prefix and BLAKE3 identities.
- **Behavior change:** a missing parent hash becomes a fail-closed blocker instead of a warning and zero digest.
- **Testing:** traditional and versioned forms, known parents, duplicate admission, custom prefixes, malformed data, size and depth limits, unknown versions, unsupported outputs, missing parents, and registry non-mutation.
