# ADR 0062: Adapt Ekala package-maintenance patterns without transferring authority

## Status

Proposed

## Context

ADR 0056 defines Mantlepkgs generation from locked concrete package graphs. That boundary does not yet define package domains, base-to-head review evidence, or source-update policy.

The Ekala project contains relevant package-maintenance designs:

- `corepkgs` separates a small core from ecosystem package repositories.
- `corepkgs` uses named variants and selected separate test derivations.
- EkaCI compares base and head package graphs, build outcomes, and closures.
- `ekapkgs-update` models version sources, update rules, OSV observations, and Repology observations.
- `hackage2ekapkgs` generates per-package artifacts and one deterministic index.

These projects use Nix-oriented data and service boundaries. Some inspected code mixes I/O with policy logic. Selected advisory code also converts request failures into empty results.

Mantle already owns stronger boundaries for typed Nickel policy, pure Rust cores, action-result admission, store evidence, and external CI adapters.

## Decision Drivers

- Extend Mantlepkgs without adding a Nix evaluator to consumers.
- Keep package and update policy typed and replayable.
- Preserve functional-core and imperative-shell boundaries.
- Keep forge credentials and deployment authority outside Mantle.
- Preserve unavailable advisory and closure facts without false success.
- Use external references without making young upstream repositories runtime dependencies.

## Decision

Mantle will adapt Ekala package-maintenance patterns through three bounded Cairn changes.

`structure-mantlepkgs-domains` will add typed package domains, deterministic shard composition, explicit variants, and separate validation roots.

`report-mantlepkgs-impact` will add deterministic base-to-head package, outcome, and closure reports. Forge presentation remains an external adapter.

`plan-mantlepkgs-source-updates` will add typed update policy, bounded source observations, pure candidate selection, preimage-bound mutation, and explicit advisory status.

Mantle may use one pinned `corepkgs` revision as an external validation corpus. The corpus must carry an exact revision, license observation, selected package set, producer policy, and artifact identities.

Mantle will not copy Ekala Nix package expressions into product code. It will not adopt derivation-path-only result authority or convert unavailable advisory data into empty findings.

Mantle will defer ecosystem-specific generator code, including `hackage2ekapkgs`, until the generic catalog contracts are stable.

Mantle will not adopt `ekafleet` or EkaOS deployment behavior. Those concerns belong to OnixOS, Lattice, or another frontend under ADR 0010.

Mantle will not adopt the archived Ekala `stdenv` repository or duplicate Cairn and ADR governance with EEP lifecycle files.

## Alternatives Considered

### Adopt EkaCI as Mantle's CI server

Rejected because hosted forge orchestration, credentials, webhooks, and review presentation are outside the build-tool boundary.

### Adopt `ekapkgs-update` directly

Rejected because its Nix subprocess model and selected failure-to-empty advisory behavior do not fit Mantle's evidence boundary.

### Import Ekala package expressions into Nickel

Rejected because Nix package expressions do not become equivalent Nickel recipes through bounded source translation.

### Keep one undifferentiated Mantlepkgs catalog

Rejected because it hides review ownership, package-domain cadence, explicit variant relationships, and validation-root separation.

### Add all maintenance behavior to the active Mantlepkgs change

Rejected because catalog generation, impact reporting, and update mutation have separate contracts, risks, and validation rails.

## Consequences

- Mantle gains three active planning packages after the base Mantlepkgs producer change.
- The changes can progress independently after their declared dependencies stabilize.
- A pinned `corepkgs` corpus can expose Nixpkgs-specific assumptions.
- External services remain observations rather than trust authorities.
- More schemas and receipts require explicit limits and compatibility versions.
- No direct upstream code adoption occurs without a separate pinned license, provenance, and test audit.

## References

- <https://github.com/ekala-project/corepkgs>
- <https://github.com/ekala-project/eka-ci>
- <https://github.com/ekala-project/ekapkgs-update>
- <https://github.com/ekala-project/hackage2ekapkgs>
- [ADR 0010](0010-keep-mantle-build-tool-boundary.md)
- [ADR 0024](0024-separate-action-results-from-cas-and-execution.md)
- [ADR 0056](0056-generate-mantlepkgs-from-concrete-package-graphs.md)
