# V3 smoke evidence: blocked by make build failure

Task-ID: V3
Covers: bootstrap.part.make.3.82.runtime-validation

The smoke test could not run because `crunch bootstrap validate bootstrap/make-tcc.ncl`
failed before producing a GNU Make 3.82 output.

Evidence:

- `evidence/validation-summary.json` reports `status = build-failed`, `failure_class = build`, and `build_exit_code = 1` for the validation runner.
- `evidence/build.stdout.log` contains the structured Crunch build report for failed root `make-3.82-tcc`.
- `evidence/build.derivation.log` and `evidence/build.derivation-2026-05-02.log` contain builder transcripts ending in `Segmentation fault (core dumped)`.
- `evidence/V2-build-rerun-2026-05-02.md` records the latest reproduced `make-3.82-tcc` exit 139.

Pending repair target: the TinyCC/Mes link-only diagnostic now reaches the builder and produces a static executable, but that executable still exits 139 under a host runtime probe. Repair the TinyCC/Mes static executable runtime startup/relocation path before returning to GNU Make `make --version` and simple Makefile smokes.
