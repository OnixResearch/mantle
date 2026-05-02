## ADDED Requirements

### Requirement: i386 Mes libtcc1 compile flags [r[bootstrap.i386-mes-libtcc1-flags.repair]]

Crunch MUST compile the i386 Mes `libtcc1.c` proof with flags that avoid unsupported broad float and long-long helper emission in the `tcc26-i386` predecessor.

#### Scenario: Real libtcc1 archive is created [r[bootstrap.i386-mes-libtcc1-flags.evidence]]

- GIVEN the i386 Mes runtime-layout sibling proof
- WHEN it builds Mes `lib/libtcc1.c`
- THEN `runtime_libtcc1_object` and `runtime_libtcc1_archive` MUST exit 0 without using the prior placeholder object path.

### Requirement: Post-libtcc1 TinyCC 0.9.27 blocker [r[bootstrap.i386-mes-libtcc1-flags.next-blocker]]

Crunch MUST record the next TinyCC handoff blocker after real i386 `libtcc1.a` creation.

#### Scenario: Next blocked step is explicit [r[bootstrap.i386-mes-libtcc1-flags.next-blocker.explicit]]

- GIVEN a real i386 Mes `libtcc1.a` archive exists
- WHEN the TinyCC 0.9.27 object probe fails
- THEN the evidence MUST name the failing step, return code, and stderr excerpt.
