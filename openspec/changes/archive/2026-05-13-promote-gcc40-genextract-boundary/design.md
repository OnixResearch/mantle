## Context

`genrecog` was split out of the generated-source family, leaving `genextract` and later generators in the generic wrapper. The generic wrapper still emits `generated_bootstrap_stub`, which makes the boundary less auditable for a specific generator.

## Decision

Add a dedicated `build/genextract` wrapper that emits a small C source containing a named empty-extraction boundary symbol and marker. Keep the output intentionally conservative and keep the parity row partial.

## Alternatives

- Implement real native `genextract`: higher-value long term, but too broad for this bounded increment.
- Leave grouped wrapper untouched: preserves ambiguity and makes later parity evidence less precise.

## Risks

- OpenSpec archive can narrow the cumulative GCC ladder requirement. After archive, inspect and restore cumulative scenarios before committing.
- Placeholder inventory line numbers can drift; recompute from actual derivation content.
