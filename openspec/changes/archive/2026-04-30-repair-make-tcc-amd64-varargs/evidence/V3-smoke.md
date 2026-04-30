Task-ID: V3
Covers: bootstrap.part.make.3.82.amd64.execution

Status: deferred
Deferred to: repair-make-tcc-amd64-varargs-runtime-validation

The version, simple Makefile positive smoke, and missing-target negative smoke require a completed `bootstrap/make-tcc.ncl` build output. V2 timed out locally before producing an output path, so running these smoke checks now would be a false proof.

The runtime-validation successor must first obtain the built make output, then run:

- `bin/make --version` and require `GNU Make 3.82`;
- a simple Makefile whose `all` target echoes `make-smoke-ok` and exits successfully;
- a missing-target invocation that exits nonzero without a signal/segfault status.

Verified: 2026-04-30T23:47:11Z
