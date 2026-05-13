## Context

`genpeep` was split out of the generated-source family, leaving `genopinit` and later generators in the generic wrapper. The generic wrapper still emits `generated_bootstrap_stub`, which hides which generator boundary is currently being bridged.

## Decision

Add a dedicated `build/genopinit` wrapper that emits a small C source containing a named empty-opinit boundary symbol and marker. Keep the output intentionally conservative and keep the parity row partial.

## Alternatives

- Implement real native `genopinit`: higher-value long term, but too broad for this bounded increment.
- Leave grouped wrapper untouched: preserves ambiguity and makes later parity evidence less precise.

## Risks

- OpenSpec archive can narrow the cumulative GCC ladder requirement. After archive, inspect and restore cumulative scenarios before committing.
- Placeholder inventory line numbers can drift; recompute from actual derivation content.
