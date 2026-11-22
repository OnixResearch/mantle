## Phase 1: Content-admission refusals name their cause

- [x] [serial] T1.1 Return the HTTP NAR refusal reason as `NarIngestion::Refused(NarRefusal { class, detail })`, bound the detail with a named `MAX_*` byte limit truncated on a UTF-8 boundary, and keep every existing tracing warning. r[foreign_derivation_import.content_admission_diagnostics]
  - Evidence: `crates/crunch-store/src/pull.rs` unit test for the class/detail rendering and the truncation bound.
- [x] [serial] T1.2 Report the strict closure import refusal as `http-closure-content-admission-failed: <path>: <class>: <detail>` and keep the flat pull's skip-and-count behavior unchanged. r[foreign_derivation_import.content_admission_diagnostics]
  - Evidence: `-p crunch-store --lib` tests: HTTP status, ingestion failure, successful import, and skip counter.

## Phase 2: Successful realizations keep their build logs

- [x] [serial] T2.1 Persist every executed outcome log and every retained failed-root log to `<state-dir>/logs/<drv>.log` with the existing `build_log` writer, write nothing for cached or reused outcomes, and collect failures as diagnostic persistence failures. r[foreign_derivation_import.realization_build_logs]
  - Evidence: shell unit test that a failed logs directory yields one `DiagnosticPersistenceFailure` and no error.
- [x] [serial] T2.2 Report collected failures the way `mantle build` does in human output only, leaving `--json` stdout a single receipt and the receipt bytes unchanged. r[foreign_derivation_import.realization_build_logs]
  - Evidence: CLI test asserting the receipt parses on a successful realize.
- [x] [serial] T2.3 Add the CLI regression: a successful realize leaves the root log with the builder's output, `mantle log` shows it, and a reuse run does not rewrite the log. r[foreign_derivation_import.realization_build_logs]
  - Evidence: `tests/cache_inputs_build_route.rs`.

## Phase 3: Verification and records

- [x] [serial] T3.1 Run the focused test legs, fmt, and strict Clippy for the touched packages; record the exact transcripts in `evidence/`. r[foreign_derivation_import.content_admission_diagnostics] r[foreign_derivation_import.realization_build_logs]
  - Evidence: `evidence/checks.md` indexing the captured logs.
- [x] [serial] T3.2 Run `cairn validate`, the proposal/design/tasks gates, and `git diff --check`; commit the change. r[foreign_derivation_import.content_admission_diagnostics] r[foreign_derivation_import.realization_build_logs]
  - Evidence: gate transcripts under `evidence/`.
