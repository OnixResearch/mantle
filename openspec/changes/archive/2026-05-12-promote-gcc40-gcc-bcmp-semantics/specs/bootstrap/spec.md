## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. This promotion MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by itself.

#### Scenario: `__gcc_bcmp` distinguishes equal and unequal byte ranges

- GIVEN the GCC 4.0 pass1 libgcc member sources are generated
- WHEN the derivation compiles and runs the `__gcc_bcmp` smoke
- THEN equal byte ranges return `0`
- AND unequal byte ranges return non-zero
- AND the `gcc.4.0` parity row remains partial until broader native correctness evidence exists
