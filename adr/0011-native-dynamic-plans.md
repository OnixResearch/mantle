# ADR 0011: Native dynamic plans

## Status

Proposed (2026-05-30)

## Context

Mantle already has a compatibility implementation for dynamic derivations: a
build can produce a Nix `.drv` ATerm file, `crunch-build` parses it, registers the
resulting `nix_compat::Derivation`, and the lazy worker schedules it. That proves
the scheduler can grow the graph during a build, but it makes the native dynamic
story depend on Nix file formats.

Mantle needs native dynamic graph growth for bootstrap stages, package-manager
resolver outputs, generated package sets, and frontend-produced build plans. The
user direction for this decision is explicit: no Steel now, and no requirement to
stay Nix-compatible.

## Decision

Mantle dynamic derivations will use native dynamic build plans, not Nix `.drv` as
the core API. A producer build declares outputs that may contain `mantle-plan-v1`
artifacts. After the producer completes, the worker reads only those declared
outputs, validates the plan with Rust-owned typed data, computes a canonical
BLAKE3 digest, registers accepted units as native goals, and continues through
the existing lazy scheduler.

Nickel remains the declarative authoring surface for normal derivations and for
declaring dynamic-plan outputs. Rust owns the canonical plan schema, validation,
limits, scheduling, and provenance. Steel may become an optional plan producer in
a later ADR, but it is not part of the core dynamic-plan dependency chain.

The existing `.drv` output detector may remain as labeled compatibility/debug
behavior. It must not define the native ABI.

## Consequences

Dynamic graph growth becomes frontend-neutral and Mantle-shaped. Sandboxed tools
can generate plans without re-entering Nickel evaluation, and reports can record
which producer emitted which plan digest and which units were accepted or
rejected.

The native ABI needs explicit versioning, bounds, policy inheritance rules, and
negative tests. First implementation should therefore start with the pure plan
validator before touching worker scheduling.

Compatibility with Nix dynamic derivation path calculations is no longer the
goal for this native path. Any `.drv` support must be documented and reported as
compatibility mode.

## Alternatives Considered

### Keep `.drv` as the only dynamic format

Rejected. It preserves the compatibility seam but keeps Mantle's core dynamic
story coupled to Nix ATerm and `nix_compat::Derivation`.

### Add Steel as the dynamic-plan language now

Rejected. Steel can be a producer later, but making it a core dependency would
add a second evaluator/runtime before the data ABI is stable.

### Implement Nix-style import from derivation

Rejected. Evaluator suspension and arbitrary output import are more powerful but
much riskier. Native dynamic plans are bounded data, validated before scheduling.
