## MODIFIED Requirements

### Requirement: GCC version ladder
GCC 4.0 `genrecog` boundary promotions MUST be derivation-checked: the `genrecog` path MUST emit a named empty-recognition source boundary, MUST reject the prior generic generated-source stub label, and MUST NOT mark real native `genrecog` correctness complete.

#### Scenario: GCC 4.0 genrecog emits checked empty-recognition boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `genrecog` before native recognizer-generator correctness is complete
- WHEN generated recognizer source is emitted
- THEN the source contains a named empty-recognition boundary symbol
- AND the source contains an empty-recognition source boundary marker
- AND the source does not contain the prior generic generated-source stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
