## ADDED Requirements

### Requirement: i386 TinyCC 0.9.27 handoff probe

r[i386_tinycc27.handoff_probe] Mantle MUST keep the `tcc26-i386` to `tcc27-i386` handoff as a bounded diagnostic proof until real TinyCC 0.9.27 object emission succeeds with real i386 Mes runtime artifacts.

#### Scenario: handoff summary records success or first blocker

GIVEN a sibling i386 handoff derivation runs with the selected TinyCC 0.9.27 source input and real i386 Mes runtime archive
WHEN the derivation finishes or reaches the first compiler/runtime blocker
THEN it MUST write a compact summary naming status, selected input, runtime artifacts, output digest when successful, rc or signal when failing, and next action
AND the summary MUST distinguish host-unsupported i386 execution from TinyCC handoff failure.

#### Scenario: real object emission is required for success

GIVEN the probe emits a TinyCC 0.9.27 object with the real i386 runtime inputs
WHEN handoff evidence is accepted
THEN the evidence MUST record the object path or digest and smoke boundary used to prove object creation
AND it MUST remain a diagnostic handoff claim rather than a production bootstrap route switch.

#### Scenario: placeholders and segfaults stay blocked

GIVEN the probe uses placeholder archives, header-only continuation, missing runtime artifacts, or a signal-derived object failure
WHEN evidence is summarized
THEN Mantle MUST classify the result as blocked
AND it MUST NOT claim TinyCC 0.9.27 i386 handoff success.

#### Scenario: Make remains out of scope

GIVEN the i386 TinyCC 0.9.27 handoff is not yet successful
WHEN planning downstream live-bootstrap work
THEN Mantle MUST NOT treat GNU Make 3.82 or later i386 bootstrap stages as unblocked by this probe
AND it MUST keep the next action below Make until the compiler handoff evidence succeeds.
