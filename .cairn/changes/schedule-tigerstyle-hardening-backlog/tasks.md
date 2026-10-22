# Tasks: Schedule the Tiger Style hardening backlog

## Scope

- [x] [task:S1][serial] Record the per-package Tiger Style findings for every package in `workspace.metadata.tigerstyle.default_scope`, one package per run so a failing package does not hide the packages behind it. r[mantle.tigerstyle_backlog.scan]
  - Evidence: `evidence/tigerstyle-backlog-2026-10-22.log` records the per-package scan as a summary, 477 findings in 24 packages; the raw per-package transcripts of that scan were not retained. The `mantle` library was rescanned after each repair, because a library that fails type checking under lint does not report every finding.
- [x] [task:S2][after:S1] Close the mechanical findings in the source across the 24 affected packages: assertion density, compound conditions, numeric unit naming, bool naming, explicit defaults, unchecked unsigned arithmetic, function length, nested conditionals, parameter count, ambiguous parameters, collection growth and loop bounds. r[mantle.tigerstyle_backlog.fix]
  - Evidence: the `style:` commits from `7b3f46173` through `a272ea71e`, and `237c5b5d0`, which restores the publication error that `30340c74f` had turned into a debug-build abort and adds its regression test. Focused checks on the source of `7f4da80d1`, which `835702dfc` only reformats and `a272ea71e` only adjusts for strict Clippy: the serialized `mantle` library tests reported 239 passed and 0 failed, the `mantle` binary `source_built_fixed_point` and `bootstrap::` filters reported 128 passed (1 ignored) and 18 passed, and the ELF relocation and reference-scanner suites reported 6 and 1 passed. The new unwritable-directory relocation test aborts on `468c50910`.
- [x] [task:S3][after:S1] Record every finding that cannot be removed without changing observable behaviour, changing a published public API, or restructuring code outside the affected package, as a file-scoped `#![allow(tigerstyle::...)]` with a `HARDENING-BACKLOG` comment naming this change's evidence file. Do not extend any existing backlog block beyond its recorded categories. r[mantle.tigerstyle_backlog.record]
  - Evidence: every file carrying a `HARDENING-BACKLOG 2026-10-22` block has a row with its categories and reason in `evidence/tigerstyle-backlog-2026-10-22.log`. No block dated before this change gained a category, and every allow added in the range is a `tigerstyle::` allow.
- [x] [task:S4][after:S2][after:S3] Write the deferred-finding log and the green gate transcript into this change's evidence. r[mantle.tigerstyle_backlog.evidence]
  - Evidence: `evidence/tigerstyle-backlog-2026-10-22.log` and `evidence/tigerstyle-final-green.txt`, the cold-target run of the flake's `crunch-tigerstyle check` over the declared scope on `a272ea71e` with a clean source tree, which exits 0.
- [x] [task:V1][after:S4] Run `cargo tigerstyle check` over the declared scope, the first-party strict Clippy gate, the workspace lib and tests, and the affected packages' format checks, and retain the exact output. r[mantle.tigerstyle_backlog.verify]
  - Evidence: `evidence/v1-gates-2026-10-22/summary.md` and the transcripts beside it, on `a09814bff`, whose source equals `a272ea71e`. Tiger Style, the GCC 4.0 bridge, first-party rustfmt, strict Clippy and the nine affected-package format checks pass. The serialized workspace tests fail: 16 `crunch` binary unit tests and 17 `mantle` integration targets, every one of which fails the same way on `684e31623`, the revision before this change; one of them, the evaluator teardown test, is timing-flaky on both revisions. The ordinary gate therefore does not pass, and those pre-existing failures remain open outside this change.

## Boundaries

- `dylint.toml` is consumer-owned and is not changed by this change.
- No non-`tigerstyle` allow is added anywhere.
- No finding is suppressed by removing a test, weakening a check or
  accepting a different input.
