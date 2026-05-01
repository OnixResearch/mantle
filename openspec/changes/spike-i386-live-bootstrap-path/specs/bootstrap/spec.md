## ADDED Requirements

### Requirement: i386 live-bootstrap path spike [r[bootstrap.i386-live-bootstrap-spike]]
Crunch MUST provide decision evidence before pivoting Make 3.82 runtime validation from the current amd64 TinyCC/Mes repair path to an i386-first live-bootstrap path.

#### Scenario: Reference audit is not runtime proof [r[bootstrap.i386-live-bootstrap-spike.reference-audit]]
- GIVEN a StageX or upstream live-bootstrap reference sequence that builds on `linux/386`
- WHEN Crunch records that sequence as evidence
- THEN Crunch MUST classify it as reference evidence only until a Crunch-local proof target runs.

#### Scenario: Pivot decision has explicit criteria [r[bootstrap.i386-live-bootstrap-spike.decision]]
- GIVEN the amd64 Make 3.82 path remains blocked by runtime segfaults
- WHEN the i386 proof target is evaluated
- THEN Crunch MUST record whether to pivot, continue amd64 repair, or carry both paths with explicit scope boundaries.
