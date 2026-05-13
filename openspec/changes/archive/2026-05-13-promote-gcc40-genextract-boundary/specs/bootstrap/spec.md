## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 `genextract` boundary promotions MUST be derivation-checked: the `genextract` bridge path MUST emit a named empty-extraction source boundary, MUST reject the prior generic generated-source stub label, and MUST NOT mark real native `genextract` or GCC 4.0 correctness complete.

#### Scenario: GCC 4.0 genextract emits checked empty-extraction boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `genextract` before native extractor-generator correctness is complete
- WHEN generated extraction source is emitted
- THEN the source contains a named empty-extraction boundary symbol
- AND the source contains an empty-extraction source boundary marker
- AND the source does not contain the prior generic generated-source stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
