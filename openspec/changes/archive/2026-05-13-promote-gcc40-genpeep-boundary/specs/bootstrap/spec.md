## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 `genpeep` boundary promotions MUST be derivation-checked: the `genpeep` bridge path MUST emit a named empty-peephole source boundary, MUST reject the prior generic generated-source stub label, and MUST NOT mark real native `genpeep` or GCC 4.0 correctness complete.

#### Scenario: GCC 4.0 genpeep emits checked empty-peephole boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `genpeep` before native peephole-generator correctness is complete
- WHEN generated peephole source is emitted
- THEN the source contains a named empty-peephole boundary symbol
- AND the source contains an empty-peephole source boundary marker
- AND the source does not contain the prior generic generated-source stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
