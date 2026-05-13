## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 `genoutput` boundary promotions MUST be derivation-checked: the `genoutput` bridge path MUST emit a named empty-output source boundary, MUST reject the prior generic generated-source stub label, and MUST NOT mark real native `genoutput` or GCC 4.0 correctness complete.

#### Scenario: GCC 4.0 genoutput emits checked empty-output boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `genoutput` before native instruction-output generator correctness is complete
- WHEN generated instruction-output source is emitted
- THEN the source contains a named empty-output boundary symbol
- AND the source contains an empty-output source boundary marker
- AND the source does not contain the prior generic generated-source stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
