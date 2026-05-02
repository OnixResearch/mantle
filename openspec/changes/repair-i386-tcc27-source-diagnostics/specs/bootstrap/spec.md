## ADDED Requirements

### Requirement: i386 TinyCC 0.9.27 source diagnostics [r[bootstrap.i386-tcc27-source-diagnostics.narrowing]]

Crunch MUST keep the i386 TinyCC 0.9.27 handoff diagnostic narrow enough to distinguish predecessor compiler source-emission failures from Mes runtime-library failures.

#### Scenario: Focused diagnostic matrix is preserved [r[bootstrap.i386-tcc27-source-diagnostics.evidence]]

- GIVEN the sibling i386 Mes runtime-layout proof has built `runtime_libtcc1_object` and `runtime_libtcc1_archive` successfully
- WHEN the TinyCC 0.9.27 pass1 object probe fails
- THEN the proof output MUST include diagnostic return codes for no-line preprocessing, line-marker preprocessing, representative unit compiles, and the gating `tcc27_compile_object` step.

### Requirement: i386 TinyCC 0.9.27 pass1 parity patches [r[bootstrap.i386-tcc27-source-diagnostics.pass1-parity]]

Crunch SHOULD keep the sibling tcc27 pass1 probe aligned with live-bootstrap pass1 source edits and flags when narrowing the handoff blocker.

#### Scenario: Missing pass1 edit and unrelated flags are corrected [r[bootstrap.i386-tcc27-source-diagnostics.pass1-parity.flags]]

- GIVEN the tcc27 pass1 sibling proof patches TinyCC 0.9.27 sources
- WHEN it reaches the pass1 object and link probes
- THEN it SHOULD include the live-bootstrap `check-reloc-null` edit and avoid unrelated Mes feature toggles on the tcc27 source compile/link commands.
