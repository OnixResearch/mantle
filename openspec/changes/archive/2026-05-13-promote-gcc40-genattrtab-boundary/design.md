## Context

The late GCC 4.0 generated-source family was previously grouped behind a generic wrapper. `genemit`, `genrecog`, `genextract`, `genpeep`, `genopinit`, and `genoutput` now have named checked boundaries. `genattrtab` remains as the final generic generated-source member.

## Decision

Give `build/genattrtab` its own shell wrapper that emits a conservative empty-attrtab C source with stable marker text and a stable symbol. Keep checks local to the derivation and parity tests.

## Alternatives

- Implement real native `genattrtab`: desirable later, but too broad for this bounded increment.
- Keep the generic wrapper: leaves ambiguous debt and weakens parity evidence.

## Risks

- OpenSpec archive can replace the cumulative GCC ladder text with only the new delta. Inspect and restore cumulative scenarios after archive.
- Placeholder inventory line numbers can drift; recompute from actual derivation content.
