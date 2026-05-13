## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 `gengtype` boundary promotions MUST be derivation-checked: the `gengtype` path MUST emit named empty-GTY generated-header and descriptor boundaries, MUST reject the prior generic `gengtype` stub label, and MUST NOT mark real native `gengtype` correctness complete. This promotion MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by itself.

#### Scenario: GCC 4.0 gengtype emits checked empty-GTY boundaries

- GIVEN the GCC 4.0 pass1 derivation bridges `gengtype` before native generator correctness is complete
- WHEN representative generated GTY files are emitted
- THEN a generated header contains an empty-GTY header boundary marker
- AND `gtype-desc.c` contains an empty-GTY descriptor boundary marker
- AND neither output contains the prior generic `gengtype` stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
