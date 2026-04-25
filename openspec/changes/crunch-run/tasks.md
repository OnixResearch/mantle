## 1. CLI target and binary selection

- [x] Add `--bin <name>` to the `crunch run` CLI surface and pass it into run orchestration.
- [x] Route explicit Nickel file targets (`./`, `../`, `/`, or `.ncl` suffix) through the same build pipeline as `crunch build` and reject file targets that do not produce exactly one runnable derivation outcome.
- [x] Keep project-default, bare project package name, simple project-selector, and nested `.#category.name` selector targets working through existing project resolution, with selector/bare-name parsing before filesystem probing.
- [x] Add resolver-precedence coverage for selector-before-filesystem and same-name bare package versus path ambiguity.
- [x] Add target-resolution failure coverage for missing `crunch.ncl`, missing default package, unknown selector, unknown bare package, nonexistent explicit file path, non-derivation file, and multi-entry record-valued file rejection; assert errors mention `crunch.ncl` or the rejected target as specified.
- [x] Add guardrail coverage proving `crunch run` uses `crunch.ncl`, not `crunch-project.ncl`, including an only-`crunch-project.ncl` fixture and a both-files fixture.
- [x] Implement selected-output resolution for multi-output derivations: prefer `out`, otherwise first output name sorted lexicographically; add regression coverage for both branches.
- [x] Extract deterministic binary selection into a small helper with positive and negative unit coverage.
- [x] Make missing `bin/`, empty `bin/`, missing requested `--bin <name>`, explicit `--bin` directory target, and non-executable explicit binary selections fail with clear errors, while default mode skips directories and non-executable regular files before choosing a sorted executable candidate.
- [x] Reject invalid `--bin` names (`""`, absolute paths, names with path separators, `.`, and `..`) before joining with `$out/bin`; add unit coverage for each case.
- [x] Cover positive explicit `--bin` selection of an executable symlink plus sorted symlink candidates in binary-selection tests.
- [x] Cover negative symlink cases for explicit `--bin` and fallback: broken symlink, symlink to directory, and symlink to non-executable target.
- [x] Cover fallback success when invalid symlink entries sort before a later valid executable candidate.

## 2. Execution behavior

- [x] Preserve `--` argument passthrough to the selected binary.
- [x] Preserve child exit status as the `crunch run` exit status, including non-zero fallback when the child terminates without an exit code.
- [x] Preserve child stdio, environment, and current-working-directory inheritance.
- [x] Keep `--import-path`, `--jobs`, `--no-substitute`, `--signing-key`, `--trust-unsigned`, `--store`, `--state-dir`, `--store-prefix`, and `--nix-compat` wired through the build step.
- [x] Keep `crunch run` in practical hermeticity mode and verify `--strict-hermetic` is not accepted for `run`.
- [x] Preserve run as ephemeral store-only execution with no profile or generation mutation.

## 3. Validation

- [x] Run `openspec validate crunch-run --strict`. ✅ 2026-04-25T02:49Z 1s
- [x] Run focused run-command tests for default package, bare project package name, simple and nested selectors, file target, record-valued file rejection, `--bin`, missing/non-executable binary failures, default skip semantics, symlink handling, argv passthrough, child exit-status/no-code propagation, child inheritance, profile non-mutation, practical hermeticity/no-strict flag, and all run/build flag forwarding. ✅ `cargo test -p crunch --bin crunch run_ -- --nocapture` (7 passed), pueue#52 `cargo test -p crunch --test project_build_smoke run -- --nocapture` (11 passed, including singleton-record file target), and pueue#51 `cargo test -p crunch --bin crunch -- --nocapture` (203 passed)
- [x] Update the README command summary and verify `crunch run --help` / top-level help show `run` with project-selector/file-target purpose, `--bin`, and `--` passthrough coverage. ✅ README updated; `run_help_lists_bin_and_passthrough` and `run_rejects_strict_hermetic_flag` pass in pueue#48
- [x] Run formatting for touched Rust and documentation files. ✅ `rustfmt --check src/main.rs src/build_cmd.rs src/build_report.rs src/self_build.rs tests/project_build_smoke.rs`
