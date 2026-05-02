## ADDED Requirements

### Requirement: i386 Mes runtime/header layout spike [r[bootstrap.i386-mes-runtime-layout.spike]]

Crunch MUST keep the i386 Mes runtime/header layout investigation as a bounded sibling diagnostic before changing production Make/TinyCC bootstrap routing.

#### Scenario: Layout proof records first blocker [r[bootstrap.i386-mes-runtime-layout.evidence]]

- GIVEN the proven `tcc26-i386` predecessor
- WHEN Crunch builds the i386 Mes runtime/header layout spike
- THEN the output MUST include logs and a summary naming the first blocked step or the next successful handoff boundary.

### Requirement: i386 TinyCC 0.9.27 handoff blocker [r[bootstrap.i386-mes-runtime-layout.blocker]]

Crunch MUST distinguish Mes header/CRT layout progress from complete runtime library availability before attempting the TinyCC 0.9.27 -> Make 3.82 handoff.

#### Scenario: Runtime library blocker is explicit [r[bootstrap.i386-mes-runtime-layout.blocker.runtime-library]]

- GIVEN an i386 Mes header tree and CRT object can be created
- WHEN runtime library or TinyCC 0.9.27 object compilation fails
- THEN the evidence MUST record the exact step, return code, and stderr excerpt.
