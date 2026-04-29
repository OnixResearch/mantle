## MODIFIED Requirements

### Requirement: Live-bootstrap part tinycc 0.9.26 is independently tracked
Crunch MUST track the live-bootstrap part `tinycc 0.9.26` as an independent bootstrap change bound to `bootstrap/tinycc-mes.ncl`.
ID: bootstrap.part.tinycc.0.9.26

#### Scenario: First self-compile is required evidence

- GIVEN `bootstrap/tinycc-mes.ncl` builds `tcc-mes`
- WHEN the part is marked complete
- THEN the evidence MUST show `tcc-mes` compiles at least `tcc-boot0` without segfaulting
- AND `tcc-mes -version` alone MUST NOT be accepted as the smoke boundary
