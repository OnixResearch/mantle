# Design: Harden native dynamic derivation admission

## Context

`crates/crunch-build/src/dynamic.rs` is already intended as a pure core. The Worker reads candidate bytes from castore and later mutates the derivation registry and scheduler.

The current core returns a general `nix_compat::Derivation`. Registration then calculates identity. Unknown parent derivations receive an all-zero modulo hash. The parser recognizes only `Derive(`.

Native Mantle derivation identity differs from Nix compatibility identity. This change therefore adapts useful state and validation patterns without importing `nix-derivation` into the native path.

## State model

```text
CandidateBytes
  -> ParsedDynamicDerivation
  -> ValidatedDynamicDerivation
  -> IdentityResolvedDynamicDerivation
  -> RegistryReadyDynamicDerivation
  -> shell-owned registry and scheduler mutation
```

Each admitted state uses a private Rust wrapper. Constructors enforce the transition invariants. Callers cannot construct a registry-ready value from parsed bytes alone.

The pure core receives bytes, the candidate output identity, the logical store prefix, named limits, supported version policy, and explicit parent-hash facts. It returns an admitted value or a stable typed blocker.

## Functional core and shell

The core performs these actions:

- candidate shape and size admission;
- traditional or versioned prefix classification;
- syntax parsing;
- semantic and output-form validation;
- dynamic-input depth and collection validation;
- parent-reference completeness checks;
- BLAKE3 derivation identity and configured-prefix path calculation;
- deterministic duplicate and registration planning.

The Worker shell performs these actions:

- output and castore discovery;
- bounded blob reads;
- registry lookups that produce an immutable parent-hash fact map;
- log and diagnostic presentation;
- registry mutation;
- goal creation, waiter changes, and scheduler dispatch.

The core does not access castore, the registry, tracing, clocks, files, networks, or processes.

## Complete parent identity

Identity resolution requires one explicit hash fact for every direct parent derivation. Missing, duplicate, wrong-prefix, or conflicting parent facts return stable blockers.

The core never substitutes zero bytes or another sentinel as a parent hash. External parents can enter only through an explicit admitted parent fact supplied by another compatibility or import boundary.

The registry remains unchanged after any parent-resolution error.

## Versioned dynamic inputs

Candidate detection recognizes the traditional and declared versioned prefixes. The first supported version policy covers only reviewed tags and shapes.

Recursive dynamic inputs use a named maximum depth. Parsing, construction, traversal, and validation enforce the same bound. Unknown versions and over-depth inputs fail before identity calculation.

Parsing a versioned form does not authorize execution. Unsupported output or dynamic-input semantics produce a typed unsupported blocker.

## Prefix and hash domain

The logical store prefix is an explicit input to parsing, identity, path calculation, registry keys, and diagnostics. Mixed-prefix references fail before registration.

Native derivation identities continue to use Mantle BLAKE3 rules. The Nix compatibility adapter retains Nix-required algorithms. No digest type crosses those domains without an explicit conversion that validation can reject.

## Scheduler boundary

The scheduler receives only `RegistryReadyDynamicDerivation`. A failed parse, validation, parent resolution, or identity calculation cannot create a root, waiter, registry entry, or success report.

Duplicate discovery is idempotent only when the existing registry entry has the same full admitted identity. A path collision with different identity fails closed.

## Compatibility and rollback

Golden fixtures bind accepted ATerm bytes, configured prefix, parent facts, BLAKE3 identity, derivation path, and registry plan. Positive parity preserves covered native identities.

The change records the prior dynamic-admission implementation. Rollback restores the prior core and Worker adapter together. Rollback evidence must retain the known zero-fallback risk.

## Claims

Typed staged admission proves only that selected derivation metadata passed the declared local rules. It does not prove builder safety, sandboxing, source trust, output correctness, scheduler optimality, or release eligibility.
