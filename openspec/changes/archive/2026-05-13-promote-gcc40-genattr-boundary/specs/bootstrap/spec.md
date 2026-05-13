## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 `genattr` boundary promotions MUST be derivation-checked: the `genattr` path MUST emit a guarded empty-attribute boundary header, MUST retain the disabled `HAVE_ATTR_enabled` contract, MUST reject the prior generic `genattr` stub label, and MUST NOT mark real native `genattr` correctness complete. This promotion MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by itself.

#### Scenario: GCC 4.0 genattr emits a checked empty-attribute boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `genattr` before native generator correctness is complete
- WHEN `insn-attr.h` is emitted
- THEN the header contains the `GCC_INSN_ATTR_H` include guard
- AND the header contains an empty-attribute boundary marker
- AND the header disables `HAVE_ATTR_enabled`
- AND the header does not contain the prior generic `genattr` stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
