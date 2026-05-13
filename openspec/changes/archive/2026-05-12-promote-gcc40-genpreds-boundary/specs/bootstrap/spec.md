## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 `genpreds` boundary promotions MUST be derivation-checked: the `genpreds -h` path MUST emit a guarded empty-predicate boundary header, the source path MUST emit a named empty-predicate boundary source, both paths MUST reject the prior generic `genpreds` stub labels, and neither path MUST mark real native `genpreds` correctness complete. This promotion MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by itself.

#### Scenario: GCC 4.0 genpreds emits checked empty-predicate boundaries

- GIVEN the GCC 4.0 pass1 derivation bridges `genpreds` before native generator correctness is complete
- WHEN `tm-preds.h` and predicate source are emitted
- THEN the header contains the `GCC_TM_PREDS_H` include guard
- AND both outputs contain empty-predicate boundary markers
- AND neither output contains the prior generic `genpreds` stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven
