## Why

The strict Tiger Style gate now reaches `crunch-pipeline`. It reports eight
findings in `crates/crunch-pipeline/src/lib.rs`: two assertion-density findings,
two unbounded-growth findings, two long interfaces, one boolean name, and one
ambiguous string interface.

Mantle must repair this bounded debt without lint allowances, reduced check
scope, warning budgets, finding baselines, or pipeline-semantic changes. The
repair must preserve eager evaluation completeness, managed root publication,
cache-only observation, registered-build ordering, failure-key normalization,
and public compatibility.

## What Changes

- Bound eager root collection by the root count admitted before streaming.
- Derive managed-generation capacity from admitted outputs and source paths.
- Separate the pure managed-generation plan from root-registry effects.
- Pass existing builder bundles and named private requests instead of long
  positional interfaces.
- Keep cache-only service construction small and explicit.
- Replace the private ambiguous failure-key arguments with a named input.
- Run focused and repository Tiger Style checks without allowances.

## Impact

- **Files:** `crates/crunch-pipeline/src/lib.rs`, focused tests, one architecture
  record if the decision constrains later work, and lifecycle evidence.
- **Testing:** pre-change and post-change pipeline tests, focused Tiger Style,
  repository Tiger Style, strict Clippy, formatting, caller compilation, Nix
  evaluation, full Nix checks, Cairn validation, Tracey coverage, and gates.
