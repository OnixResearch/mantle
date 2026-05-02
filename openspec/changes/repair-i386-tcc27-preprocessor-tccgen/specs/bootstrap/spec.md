## MODIFIED Requirements

### Requirement: i386 TinyCC 0.9.27 handoff diagnostics [r[i386-tcc27-handoff-diagnostics]]

The system MUST maintain a sibling i386 TinyCC 0.9.27 handoff diagnostic derivation that records source-emission, per-unit compile, full object compile, link, and downstream Make smoke boundaries before production bootstrap routing depends on that path.

#### Scenario: predecessor source-emission blocker is repaired or narrowed [r[i386-tcc27-handoff-diagnostics.source-emission]]

- GIVEN the sibling i386 Mes runtime layout derivation
- WHEN the diagnostic derivation runs after a source-normalization patch
- THEN the evidence MUST show whether line-marker preprocessing, `tccgen.c` compilation, full `ONE_SOURCE=1` compilation, and the first subsequent blocker pass or fail with captured rc/stdout/stderr logs.
