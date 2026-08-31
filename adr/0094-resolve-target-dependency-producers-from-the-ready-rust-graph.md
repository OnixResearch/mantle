# ADR 0094: Resolve target dependency producers from the ready Rust graph

## Status

Accepted (2026-08-31)

## Context

V91 passed the consumed-host proc-macro fallback from ADR 0093. It then reached
a target `proc-macro-crate` library unit whose `toml_edit` dependency artifact
had no direct producer.

The ready native unit graph contained the exact target `toml_edit` library unit.
This was not a proc-macro host edge, so the consumer had no consumed host
artifact.

The inspected graph contains 2,203 target dependency artifacts without direct
producer IDs. For all of them, one target library with the same package and
triple exists. Most also match the normalized dependency name. The remaining
renamed dependencies have one package-level target library.

## Decision Drivers

- Use only units in the already ready derivation graph.
- Prefer direct and consumed-host producer facts first.
- Select target libraries, never sibling binaries.
- Keep target triples equal.
- Support hyphen-to-underscore crate-name normalization.
- Support renamed dependencies only when the package candidate is unique.
- Reject missing and ambiguous graph producers.
- Avoid quadratic producer search at the graph limit.

## Decision

Build a bounded producer index once from ready graph units whose execution kind
is `target` and target kind is `lib`.

Index each producer by:

- package identity and selected triple; and
- package identity, selected triple, and normalized target name.

For a dependency artifact without direct or consumed-host authority:

1. Require the consumer execution kind to be `target`.
2. Prefer the exact normalized-name index.
3. Otherwise use the package-and-triple index.
4. Require exactly one producer identity.
5. Bind that identity in action ordering and dependency input authority.

Normalize only `-` to `_`, matching Rust crate naming. Do not strip versions,
change package identities, or cross target triples.

## Alternatives Considered

### Search every graph unit for each dependency

Rejected. The worst case is quadratic at the 16,384-unit bound. A one-pass
index keeps lookup bounded.

### Match package identity without target kind

Rejected. Packages can publish libraries and multiple binaries. Dependencies
consume library artifacts.

### Match dependency name only

Rejected. Different packages can export equal crate names.

### Accept multiple feature variants

Rejected. Ambiguous producer identities must be normalized earlier or fail.

### Treat missing producers as external source inputs

Rejected. The ready graph already declares the concrete producer unit.

## Consequences

- Target dependency edges receive explicit action producer authority.
- Renamed dependencies resolve only through one package-and-triple library.
- Host proc-macro fallback remains separate and higher priority.
- Missing or ambiguous ready-graph producers fail before execution.
- V91 remains failed evidence. A fresh promoted proof must verify the index.
