# V1 gate transcripts (2026-10-22)

Subject: revision `a09814bff590809f10688ec2684ab799953feaa0` with a clean
tree (`identity.txt`). Its source equals `a272ea71e`; `a09814bff` changes only
this change's lifecycle files. Toolchain: `rustc 1.96.0-nightly (c75612477
2026-04-07)` from `nix develop`. All Cargo runs used
`CARGO_TARGET_DIR=/tmp/target-mantle-elf` and `CARGO_INCREMENTAL=0`
(`docs/toolchain-incremental-ice.md`).

## Tiger Style

`../tigerstyle-final-green.txt`: the flake's `crunch-tigerstyle check` over
`workspace.metadata.tigerstyle.default_scope` on a fresh target, revision
`a272ea71e`, exit 0.

## Ordinary first-party gate

Command: `nix develop -c ./scripts/check-first-party-quality.sh`, transcript
`first-party-gate.log`, exit 101 (`first-party-gate.exit`).

| Step | Outcome |
|---|---|
| bounded GCC 4.0 configure bridge self-test | pass |
| first-party rustfmt | pass |
| strict Clippy, vendored members excluded | pass |
| workspace lib and tests, serialized | fail |

The test step passed 111 suites, then failed in the `crunch` binary unit
tests: `test result: FAILED. 2734 passed; 16 failed; 92 ignored`. The failing
tests are listed in `crunch-bin-failing-tests.txt`. Cargo stops at the first
failing test binary, so the remaining suites were run separately.

## Affected-package format checks

`affected-package-fmt.txt`: `cargo fmt --check -p` for each of the nine
packages changed by this change and outside the gate's format selection, each
exit 0.

## Suites after the first failing binary

`head-remaining-suites/summary.txt`: every `tests/*.rs` target of `mantle`
and every package ordered after `mantle`, one `cargo test` invocation each.
46 of the 63 `mantle` integration targets passed and 17 failed. The 14
packages after `mantle` passed; `mantle-lock-vendor-producer` has no library,
so its first invocation with `--lib` was refused by Cargo and its rerun with
`--tests` reported 3 passed. The `mantle` binary unit tests compile the same
`src/main.rs` as the `crunch` binary and were not rerun.

## Baseline comparison

The same failing tests were run on `684e31623`, the revision before the first
commit of this change (`baseline-684e31623/`):

- all 16 `crunch` binary failures fail identically on the baseline
  (`head-crunch-bin-16-isolated.log`, `baseline-684e31623/crunch-bin-16.log`);
- 16 of the 17 failing integration targets fail with the same failing test
  names on the baseline;
- `evaluator_budget_cli` failed once on this revision and passed once on the
  baseline. Its failing test,
  `process_fixtures_bound_stderr_and_make_late_results_terminal`, asserts that
  a late worker exits within the shutdown grace period. Repeated runs fail with
  the same `kill_sent` assertion 1 of 5 times on this revision and 3 of 8 times
  on the baseline (`head-evaluator-teardown-rerun-*.log`,
  `baseline-684e31623/evaluator-teardown-rerun-*.log`).

## Non-claims

The ordinary gate does not pass on this revision. Its failures exist on the
baseline revision before this change, so they do not count against or for
this change; they block the reconciliation change's G1 instead. A green Tiger
Style gate is not a correctness, release or adapter claim.
