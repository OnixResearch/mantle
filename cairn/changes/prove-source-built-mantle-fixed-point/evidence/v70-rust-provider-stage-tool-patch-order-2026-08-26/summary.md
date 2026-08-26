# V70 run-rustc stage-tool patch ordering

## Result

V70 validated the V61 native prefix and completed serialized translated-Cargo construction. This proves that the V69 shared-output race repair moved the frontier.

The later `run_rustc` build failed closed with two denied executions:

- `output/prefix-s/bin/rustc`
- `output/prefix-s/bin/cargo`

Action reconciliation recorded 67,339 observed executions, 67,337 matches, two denials, and 107 promotions. Its BLAKE3 is `bbb95bc36e2d867efb271782b1d5e71f2501b46bab74b912ce197a5dbd307ec8`.

## Cause

The generated script contained the correct absolute-path rewrite. However, script line 1,578 ran the host `run_rustc` Make build before lines 1,958–1,996 applied that rewrite.

The source Makefile therefore still contained relative recipe executions when protected execution started. `ordering-excerpt.txt` preserves both script ranges.

## Repair

The full-source pipeline now applies the stage-tool absolute-path rewrite before the `FirstStageRunRustcHost` and `FirstStageRunRustcTarget` phases. Those phases preserve the rewritten recipe lines and then run their builds.

A generated-script ordering assertion requires the binding marker to occur before the host `run_rustc` Make command. Existing negative coverage still requires the compatibility route to omit the protected rewrite.

## Validation

The current transcript is in `post-repair-validation.log`.

- Rust-provider tests: 68 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
