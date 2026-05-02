## ADDED Requirements

### Requirement: i386 TinyCC 0.9.27 Make pass1 spike [r[bootstrap.i386-tcc27-make-pass1.spike]]

Crunch MUST provide a bounded sibling proof for the i386 live-bootstrap sequence from TinyCC 0.9.26 through TinyCC 0.9.27 to GNU Make 3.82 pass1 before changing production bootstrap routing.

#### Scenario: First blocker is recorded [r[bootstrap.i386-tcc27-make-pass1.spike.blocker]]

- GIVEN the proven `tcc26-i386` predecessor compiler
- WHEN the sibling proof attempts TinyCC 0.9.27 and Make 3.82 pass1
- THEN it MUST either pass the Make smoke or save the first concrete failing step with logs.

#### Scenario: Make smoke is required for success [r[bootstrap.i386-tcc27-make-pass1.spike.make-smoke]]

- GIVEN the sibling proof produces a `make` binary
- WHEN the proof claims success
- THEN `make --version` and a trivial Makefile execution MUST both pass inside Crunch's sandbox.
