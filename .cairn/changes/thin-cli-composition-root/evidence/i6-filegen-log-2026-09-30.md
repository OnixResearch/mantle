# I6 filegen and log bounded-effect slice (2026-09-30)

Scope: `filegen plan`, `filegen apply`, and `log` only. This is not I6 completion evidence. No external build outcome, deployment safety, or generated-content correctness follows from these observations.

## Origin/main baseline

Revision `da00f58425740adea559ef926c9dfa97b2cb8240` was built in detached `/home/brittonr/scratch/mantle-filegen-log-baseline-da00f58` with `TMPDIR=/home/brittonr/scratch`, `CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-filegen-log-target`, and `nix develop -c cargo build -p mantle --bin mantle` (`Finished dev profile` in 7m24s). Executable SHA-256: `9d053aa457650740707ed12dbb7e9e5a1e5133b480d4f04c7915a6d11d9c3138`.

Actual CLI smoke used a fresh scratch copy of `examples/projects/reviewed-file-generation/{mantle-project.ncl,filegen-schema.ncl}` and real `CRUNCH_LOG_DIR` filesystem entries:

| Scenario | Baseline exit | Baseline observation |
| --- | ---: | --- |
| `--json filegen plan --plan-out` pointing at existing directory | 3 | No stdout; JSON stderr reports `committing generated file ... Is a directory` |
| Successful JSON `filegen plan --plan-out` | 0 | `mantle-project-filegen-plan-v1`, two `create` actions, plan file exists |
| Successful JSON `filegen apply --plan` | 0 | Same two actions; both generated files exist |
| Human `filegen plan` after apply | 0 | Two `unchanged copy` lines with stable target/digest, stderr `filegen plan ok` with non-claim |
| JSON filegen plan after removing one state entry and modifying that generated file | 3 | `conflict`, `unchanged`; reviewed partial plan written, no stderr |
| Apply prior reviewed plan after partial state and edited generated file | 3 | JSON `existing-file-conflict` blocker, no stderr; edited content remains untouched |
| Apply with `.mantle/` mode `0555` after planning | 3 | No stdout; JSON stderr reports permission denial writing temporary state; both generated files exist, state file absent (real partial apply) |
| `log --list`, readable log plus directory named `z-bad.log` | **0** | Partial stdout: `a-good  [success]  smoke`, then bare `z-bad`; no stderr |
| `log --list`, readable log plus dangling `z-missing.log` symlink | **0** | Partial stdout: `a-good  [success]  smoke`, then bare `z-missing`; no stderr |
| `log --list`, only readable log | 0 | `a-good  [success]  smoke` |
| `log a-good`, one readable selected log | 0 | Exact raw log content `# status: success` and `# derivation: smoke` on stdout |
| `log --list`, empty directory | 0 | Empty stdout; stderr names the directory with `No build logs found` |

The failed/missing log rows demonstrate the consumer-visible baseline bug. This smoke does not prove the baseline state read-back race because an ordinary CLI invocation cannot intervene between write and read-back; the scoped real-port fixtures address that boundary.

## First post-cutover binary

After the integrated current-tree root and real binary build, the binary at `/home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle` had SHA-256 `9bff7a785ac1e74bff7966cf95e8f1501daed8ce39095e16c407b14949409eb0`. The same fresh-project and filesystem-log scenarios ran with `python3 /home/brittonr/scratch/mantle-filegen-log-smoke.py /home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle /home/brittonr/git/OnixResearch/mantle` (exit 0).
Separate scratch `CRUNCH_LOG_DIR` fixtures exercised `log z-bad` with a readable log and directory `z-bad.log`, a readable log followed by invalid UTF-8, 16 MiB and 16 MiB + 1 byte log files, two logs jointly at and beyond 16 MiB, and `log --list` with 4096 and 4097 log files using the same binary.

| Scenario | Updated exit | Updated observation |
| --- | ---: | --- |
| JSON `filegen plan --plan-out` targeting an existing directory | 3 | No stdout; JSON stderr code 3 and `internal` kind name the directory and failed commit |
| Successful JSON `filegen plan --plan-out` | 0 | Two `create` actions, `mantle-project-filegen-plan-v1`, plan artifact exists |
| Successful JSON `filegen apply --plan` | 0 | Two `create` actions and both managed files exist |
| Human `filegen plan` after apply | 0 | Two `unchanged copy` lines with stable target/digest; stderr retains the limited file-generation non-claim |
| Plan after removing one state entry and editing that generated file | 3 | `conflict`, `unchanged`; partial reviewed plan artifact exists, no stderr |
| Apply old reviewed plan after partial state/edit | 3 | JSON `existing-file-conflict` blocker, no stderr; edited file remains `locally edited` |
| Apply after `.mantle/` mode `0555` | 3 | No stdout; JSON stderr reports the denied state temporary write; both generated files exist but state file does not (partial apply) |
| `log --list`, readable log plus directory named `z-bad.log` | **3** | **Empty stdout**; stderr names `z-bad.log` and `Is a directory` |
| `log --list`, readable log plus dangling `z-missing.log` symlink | **3** | **Empty stdout**; stderr names `z-missing.log` and `No such file or directory` |
| `log z-bad`, one readable log plus directory named `z-bad.log` | **3** | **Empty stdout**; stderr names `z-bad.log` and `Is a directory` |
| `log --list`, readable log plus invalid UTF-8 log | **3** | **Empty stdout**; stderr names `z-invalid.log` and `invalid utf-8 sequence` |
| `log oversized`, selected file has 16 MiB + 1 byte | **3** | **Empty stdout**; stderr reports `log bytes exceed 16777216` |
| `log --list`, 16 MiB - 1 byte plus 2-byte log | **3** | **Empty stdout**; stderr names the second log and `log bytes exceed 16777216` |
| `log --list`, 16 MiB - 1 byte plus 1-byte log | 0 | Both `a-first` and `z-second` listed, no stderr |
| `log --list`, one exactly 16 MiB log | 0 | Single row `exact  [unknown]`, no stderr |
| `log --list`, 4097 log files | **3** | **Empty stdout**; stderr reports `too many build logs: 4097 > 4096` |
| `log --list`, 4096 log files | 0 | Exactly 4096 rows (`0000` through `4095`), no stderr |
| `log --list`, only readable log | 0 | `a-good  [success]  smoke` on stdout, no stderr |
| `log a-good`, one readable selected log | 0 | Exact two-line raw content `# status: success` and `# derivation: smoke` on stdout, no stderr |
| `log --list`, empty directory | 0 | Empty stdout; stderr names the empty directory with `No build logs found` |

The unreadable and missing-log cases no longer report a successful partial listing; the over-budget cases also fail with empty stdout. The CLI run cannot intervene between apply and read-back to prove the state race; the real-filesystem port regressions exercise mismatched state, tampered/missing managed copies, changed symlink targets, and failed plan publication. This first-binary smoke does not imply deployment safety or complete I6 acceptance.

## Focused verification

With `TMPDIR=/home/brittonr/scratch`, private `CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-filegen-log-target`, and `nix develop --offline --no-write-lock-file -c`, the following `cargo test --locked --offline -p mantle` filters passed:

| Target and filter | Result |
| --- | --- |
| `--bin mantle filegen_cmd:: -- --test-threads=1` | 10 passed, 0 failed |
| `--bin mantle log_cmd:: -- --test-threads=1` | 4 passed, 0 failed |
| `--test example_projects reviewed_file_generation -- --test-threads=1` | 5 passed, 0 failed |
| `--test integration log_subcommand -- --test-threads=1` | 7 passed, 0 failed |

`rustfmt --check --edition 2024 src/filegen_cmd.rs src/log_cmd.rs tests/example_projects.rs tests/integration.rs` in the project shell passed. `git diff --check --` over the owned source, tests, README, example, and evidence files passed.

The strict scoped `cargo clippy --locked --offline -p mantle --bin mantle --test example_projects --test integration --no-deps -- -D warnings` has **not** passed: private-target attempts stopped before checking the CLI on unrelated, concurrently edited shared dependencies (`crunch-eval` watch imports and the application-contract artifact helper). The root coordinator paused shared-source compilers until the cross-owner source snapshot is coherent. No lint result for the filegen/log changes is claimed.
