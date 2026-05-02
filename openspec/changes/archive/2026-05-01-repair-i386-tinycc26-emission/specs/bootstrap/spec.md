## ADDED Requirements

### Requirement: i386 TinyCC 0.9.26 emission diagnostics [r[bootstrap.i386-tinycc26-emission.diagnostics]]

Crunch MUST isolate the i386 TinyCC 0.9.26 output-generation blocker before using the i386 path as evidence for Make 3.82 runtime validation.

#### Scenario: Emission stages are separated [r[bootstrap.i386-tinycc26-emission.diagnostics.stages]]

- GIVEN a x86_64-hosted/i386-targeting TinyCC 0.9.26 built by Crunch
- WHEN Crunch evaluates the i386 emission proof
- THEN it MUST record version, assemble-only, link-from-assembly, link-from-object, and run-output results separately.

#### Scenario: Diagnostic failure does not imply production pivot [r[bootstrap.i386-tinycc26-emission.diagnostics.no-production-pivot]]

- GIVEN any i386 emission stage fails or segfaults
- WHEN the diagnostic evidence is recorded
- THEN Crunch MUST keep production Make 3.82 validation blocked rather than claiming the i386 path is production-ready.

### Requirement: i386 TinyCC 0.9.26 repair decision [r[bootstrap.i386-tinycc26-emission.decision]]

Crunch MUST record the next repair target after the diagnostic stage identifies where `tcc26-i386` fails.

#### Scenario: Next repair target is evidence-backed [r[bootstrap.i386-tinycc26-emission.decision.target]]

- GIVEN diagnostic transcript evidence for each emission stage
- WHEN choosing the next implementation slice
- THEN Crunch MUST identify whether the next target is assembly parsing, object emission, static linking, ELF materialization, or runtime execution.
