# V3 smoke evidence: blocked by make build failure

Task-ID: V3
Covers: bootstrap.part.make.3.82.runtime-validation

The smoke test could not run because `crunch bootstrap validate bootstrap/make-tcc.ncl`
failed before producing a GNU Make 3.82 output.

Evidence:

- `evidence/validation-summary.json` reports `status = build-failed`, `failure_class = build`, and `build_exit_code = 1` for the validation runner.
- `evidence/build.stdout.log` contains the structured Crunch build report for failed root `make-3.82-tcc`.
- `evidence/build.derivation.log` contains the builder transcript ending in `Segmentation fault (core dumped)`.

Pending repair target: the Make 3.82 TinyCC/Mes builder segfault must be repaired before `make --version` and simple Makefile execution can be claimed.
