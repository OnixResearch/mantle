## MODIFIED Requirements

### Requirement: GCC version ladder
GCC 4.0 `genemit` boundary promotions MUST be derivation-checked: the `genemit` path MUST emit a named empty-emit source boundary, MUST reject the prior generic `genemit` stub label, and MUST NOT mark real native `genemit` correctness complete.

#### Scenario: GCC 4.0 genemit emits checked empty-emit boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `genemit` before native generator correctness is complete
- WHEN generated emit source is emitted
- THEN the source contains a named empty-emit boundary symbol
- AND the source contains an empty-emit source boundary marker
- AND the source does not contain the prior generic `genemit` stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
