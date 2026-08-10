# Portfolio search: native dynamic derivation admission

Date: 2026-08-10

## Question

Which bounded admission design removes sentinel identities without mixing effects, hash domains, or logical store prefixes?

## Inspected evidence

- `crates/crunch-build/src/dynamic.rs` before this change;
- `crates/crunch-build/src/worker.rs` discovery and scheduling path;
- `crates/crunch-build/src/registry.rs` path and HDM maps;
- `nix_compat::Derivation` native hashing and configured-prefix functions;
- ADR 0002 and ADR 0077;
- baseline and zero-fallback characterization logs;
- positive, negative, compatibility, depth, collision, and no-effect tests.

## Candidate mechanisms

### Replace only the zero digest

This option would return an error when the parent lookup fails.
It would keep parsing, hashing, insertion, and goal creation in one mutable path.

Decision: reject.

Reason: later validation failures could still follow earlier mutation.
The approach does not create a type boundary for complete parent facts.

### Parse and register each candidate in discovery order

This option would validate each candidate before its own insertion.
It would keep a small Worker diff.

Decision: reject.

Reason: a later collision could leave earlier candidates registered.
Discovery order could select the surviving candidate.

### Use staged private states and batch preflight

This option separates parsed, validated, identity-resolved, and registry-ready values.
The Worker observes parent and registry facts between pure stages.
The Worker applies a complete batch only after collision checks.

Decision: accept.

Reason: each state proves the facts needed by the next operation.
Only the final state can cross the registry mutation boundary.

### Use `nix-derivation` for all dynamic inputs

This option would reuse the compatibility adapter for native dynamic inputs.

Decision: reject.

Reason: ADR 0077 limits that crate to Nix compatibility metadata.
Mantle-native BLAKE3 identity and configured prefixes must remain independent.

### Reject every versioned form

This option would keep only covered traditional derivations.

Decision: reject.

Reason: the accepted requirement needs one explicit bounded versioned path.
The selected path supports only `xp-dyn-drv` and input-addressed outputs.

## Adversarial audit

The first custom-prefix projection wrote direct parent paths with `/nix/store`.
A new custom-prefix versioned test exposed the mixed-prefix failure.
The renderer now uses the selected logical prefix.

The first batch check accepted two exact duplicate insertion plans.
The second insertion would have failed after the first registry mutation.
The core now selects one representative before the Worker applies the batch.
A Worker test proves one registry insertion for an exact duplicate batch.

The first unsupported-output fixture changed only the declared output path.
The legacy parser rejected it before semantic output validation.
The final fixture creates a valid fixed-output derivation first.
It then proves the stable unsupported-output blocker.

A pre-existing registry entry can share the planned path without a dynamic identity.
A Worker test proves this collision changes no registry or Worker scheduler state.

## Oracle checkpoint

Question: Does the staged design prevent partial registration for known failure paths?

Inspected evidence: batch selection code, registry type gate, missing-parent test, exact-duplicate test, and collision no-effect test.

Decision: yes, for the bounded tested admission path.

Owner: Mantle `crunch-build` Worker and dynamic admission core.

Next action: run package, Clippy, boundary, workspace, and Cairn gates before archive.

## Claim boundary

This review supports the selected admission structure only.
It does not prove arbitrary ATerm compatibility, compiler correctness, sandboxing, output behavior, scheduling quality, or release eligibility.
