## ADDED Requirements

### Requirement: TinyCC 0.9.27 amd64 static link succeeds [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link]]
The amd64 bootstrap TinyCC 0.9.27 output MUST support statically linking a trivial object with its declared Mes runtime inputs.

#### Scenario: Trivial static executable links and runs [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link.scenario.hello]]
- GIVEN the Crunch-built `bootstrap/tinycc.ncl` output on amd64
- WHEN `tcc -c hello.c` and `tcc -static -o hello hello.o` run in a Crunch diagnostic derivation
- THEN the link exits successfully
- AND executing `./hello` exits 0
- AND no host compiler, host libc, or undeclared runtime object is used

### Requirement: Make validation resumes after TinyCC link repair [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link.make-handoff]]
The TinyCC link repair MUST hand control back to the Make 3.82 runtime-validation successor after the compiler link smoke passes.

#### Scenario: Make runtime validation is unblocked or reclassified [r[bootstrap.compiler.tinycc.0.9.27.amd64.static-link.make-handoff.scenario.resume]]
- GIVEN the TinyCC link smoke passes
- WHEN `bootstrap/make-tcc.ncl` runtime validation is rerun
- THEN either the Make output path is produced for smoke testing
- OR the next concrete Make-specific blocker is recorded with transcript evidence
