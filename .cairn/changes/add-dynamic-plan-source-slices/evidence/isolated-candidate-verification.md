# Isolated source-slice candidate verification

The private, uncommitted candidate is based on `75ec931c8c5ae6e831d76ea12a93031f17e347f2` on `audit/v2-source-slices-unpublished`. The original shared checkout was not edited. At transfer from `/tmp/mantle-v2-source-candidate-yccAkTr2` into private device-46 `/home/brittonr/.cargo-target/mantle-v2-source-candidate-DY5UW249`, the complete tracked-plus-untracked source-list/content SHA-256 was `11f8a4906aeb10cbb060441c1cd093d86a43d05dc57072e5d102d68d95ddcdce` on both sides, `.git/config` SHA-256 was `509e3d599331308fba5b0819a37415441ae6d267426a54c1a3baec4b1f6ce8a2` on both sides, and both HEADs matched the pinned revision. Later dev46 source edits intentionally change only the dev46 candidate.

All final Cargo commands used private dev46 `HOME`, `TMPDIR`, `XDG_CACHE_HOME`, and `CARGO_TARGET_DIR`, with `CARGO_INCREMENTAL=0`, debug information disabled for dev/test, and `nix develop -c` from the candidate. Exact command output and both full JSON CLI receipts are retained alongside this note.

| Check | Observed result | Raw receipt |
| --- | --- | --- |
| `cargo check -q -p mantle --bin mantle` | Passed on dev46; unrelated root dead-code warnings emitted. The later rebuilt CLI binary also executed the V2 fixture. | CLI JSON receipts below |
| `cargo test -q -p crunch-build --lib --test export_api -- --test-threads=2` | Earlier candidate run passed 687 library tests and one external API test after the active-prefix fix, before the additional QA negatives below. | [pre-QA crunch-build](dev46-final-crunch-build-tests.txt) |
| `cargo test -q -p crunch-build -p crunch-store -p crunch-pipeline --lib -- --test-threads=2` | `crunch-build` 686/686, `crunch-pipeline` 45/45, `crunch-store` 396/397; the only failure is untouched `handle::tests::overlay_prefix_mismatch_fails_closed` at `handle.rs:7378`, expecting `PrefixMismatch`. That failure was not rerun. | [all-touched-libraries](dev46-all-touched-lib-tests.txt) |
| `cargo test -q -p mantle --bin mantle build_report::tests::build_json_report_includes_artifact_attestation_reference -- --test-threads=1` | Earlier accepted-slice JSON test passed 1/1; the later QA run also passes with a rejected absent-slice row. | [pre-QA root report](dev46-root-report-json-test.txt), [QA focused tests](dev46-qa-negative-focused-tests.txt) |
| `cargo test -q -p mantle --test integration_build --no-run` | Passed compilation of the migrated integration-build capability caller; this did not run its test cases. | Empty compiler output |
| `cargo test -q -p crunch-build --lib worker::tests::native_v2_custom_store_prefix_admits_declared_producer_output -- --test-threads=1` | Passed 1/1 after a real CLI rejection exposed the custom-prefix declaration lookup bug. | [custom-prefix-regression](dev46-v2-custom-prefix-test.txt) |
| Focused V2 QA negatives: registry collision/capacity, injected `put_batch_atomic` failure, and root rejected absent-slice JSON | 2/2, 1/1, and 1/1 passed respectively; both distinct batch entries remained absent on definite rejection. | [QA focused tests](dev46-qa-negative-focused-tests.txt) |
| `cargo clippy -q --no-deps --lib -p crunch-build -p crunch-store -p crunch-pipeline -- -D warnings` | Passed both before and after the QA regressions, with empty diagnostics. | No output |
| `cargo clippy -q --no-deps -p mantle --bin mantle -- -D warnings` | Failed on 34 pinned-main diagnostics in untouched root files; none named changed `bootstrap.rs`, `build_report.rs`, or `remote_build.rs`. | [root-strict-Clippy](dev46-root-strict-clippy.txt) |
| `cargo clippy -q --lib -p crunch-build -p crunch-store -p crunch-pipeline -- -D warnings` without `--no-deps` | Failed earlier on 36 existing vendored `fuse-backend-rs` diagnostics, before checking the first-party crates. | [vendored-Clippy](dev46-vendor-strict-clippy.txt) |
| Cairn scoped proposal/design/tasks gates | Historical pre-baseline gates passed (13 done, three todo). After T1.1 was checked, the available Nix-packaged Cairn CLI could not parse pinned policy's `outcome_machine`, so a fresh scoped tasks gate is **not** claimed green. | [historical tasks-gate](dev46-cairn-tasks-gate.json), [current parser error](dev46-qa-cairn-tasks-parser.txt) |
| Global `cairn validate --root .` | Not green: pinned main's unrelated `.cairn/changes/thin-cli-composition-root/tasks.md:88` has I7 marked done ahead of predecessor I6. This known failure was not rerun. | Prior candidate command output |

## Clean pinned-main V1 baseline (T1.1)

A separate clean detached clone at
`/home/brittonr/.cargo-target/mantle-v2-source-candidate-DY5UW249/baseline-main/source`
used pinned published HEAD `75ec931c8c5ae6e831d76ea12a93031f17e347f2`,
its own dev46 Cargo target, HOME, TMPDIR and XDG cache, and **the same**
`Cargo.lock` SHA-256 `f670020c83e393e077e297dcc91dba30b37fbc20fd26a889e548567e072feb08`
as the private candidate. Its `git status --porcelain` was empty after the
focused run. The accepted V1 canonical fixture is exactly
[these 916 bytes](dev46-before-v1-canonical.json); SHA-256
`2f5579a542909e6d35f59f39e380a707bad9828d75f01fe80e209388ef49d8d3`
and BLAKE3 plan digest
`dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750`.
`cmp` proved its bytes equal the private candidate fixture; `b3sum`
independently returned the same digest on both revisions.

In the clean baseline, `worker.rs` maps declared source IDs to store paths,
binds source placeholders to their logical paths, puts source inputs into
`input_sources` and registered unit outputs into `input_derivations`.
`orchestrate.rs` resolves PathInfo closures for external sources, reuses
current-session outputs, and merges source and dependency closures for sandbox
inputs. The V1 native-registration test asserted the exact source environment
path, dependent output environment path, and dependent output argument; an
unknown content-addressed output was rejected rather than guessed. The
[exact focused before-test output](dev46-before-focused-crunch-build-tests.txt)
records 44 golden/core, eight native worker, two source, two sandbox-merge,
one imported-source closure, one current-session source, and one explicit
dependency-registration invocation, all passing. Full dependency-fetch noise
from that private Cargo HOME remains outside the candidate in
`baseline-main/logs/focused-before-tests.log`. The prior private candidate
suite passed the same V1 golden and native worker tests, not a separate
before-change CLI build.

## Real two-run CLI and regression

A private Snix state/store under the dev46 lane, runnable `bwrap`, one separately built and signed stable Busybox tool store path, and two Nickel producers exercised the actual `mantle --json build --no-substitute` process and sandbox. The producers emitted the same `mantle-plan-v2`, the same `sources/package/file.txt`, and different bytes in `sources/outside.txt`. An initial CLI run exposed the previously untested active-prefix lookup defect: a declared `sources` output was rejected as `slice-output-undeclared` because its producer key was formatted with `/nix/store` while the registry used `/mantle/store`. The V2 worker now resolves the producer derivation path with the registry's configured prefix before checking declared outputs. [Pre-fix JSON](dev46-cli-before-prefix-fix.json) records the consumer-visible rejection.

Both corrected processes accepted the V2 plan and admitted the slice. The second producer had a different derivation and whole-output path, but reused the exact slice path `/mantle/store/pgr4dlvr8na9ilwi252zhz76pycg82r7-v2-cli-package` and unit derivation `/mantle/store/7kkfk95lgzwmqrsajd3vqa5r1nj9dffc-v2-cli-unit.drv`; the unit was cached on run two. Declared and observed slice NAR BLAKE3 both equaled `85b54411643cbcae7a4474c95e91fc66f29b72ccd9411906dac32b04637c6a35`. [First accepted CLI report](dev46-cli-first-accepted.json), [second accepted CLI report](dev46-cli-second-accepted.json), and [machine-extracted two-run comparison](dev46-cli-two-run-proof.json) retain exact observations. The comparison's seven boolean invariants were all true under `jq -e`.

A fresh-process `mantle --json store info` query for the admitted slice returned
one signature, `ca: Nar(Sha256(...))`, and matching NAR SHA-256 metadata. This
proves the signed PathInfo persisted; it does not prove source trust.
[Signed slice store-info receipt](dev46-cli-slice-store-info.json) records it.

## Consumer-visible QA negative controls

New V2 worker tests cause a real prospective derivation collision and leave
only one registry slot for two V2 units. Both reject **before** signed slice
publication and preserve pre-existing registry, ready goals, scheduler epoch,
and rejected report rows without admitted paths. A fault-injected
`PathInfoService::put_batch_atomic` adapter is reached through the actual
V2 `SliceAdmission` boundary with two distinct source contents and store
names: the test observes **both** expected digests in one attempted batch,
forces a definite pre-commit error, and confirms neither signed PathInfo
exists and no registry/goal mutation or falsely admitted report follows.
[All four focused regression results](dev46-qa-negative-focused-tests.txt)
include the root JSON serialization test for an absent rejected slice. This
controlled pre-commit failure does **not** establish universal atomicity
under faulty or uncertain backends; independent QA reported a separate
existing Casita backend atomic test failing, which was not rerun or disguised.

An actual sandboxed CLI build emitting an absent `packages/missing` source
slice returned `scheduler_action: rejected`, `slice-absent` and
`admitted_store_path: null`; the producer itself succeeded. [Full rejected
CLI report](dev46-qa-cli-rejected-absent.json) is retained. Replaying both
accepted CLI producers against the same private store after the QA tests
returned the identical prior slice path and unit `.drv` path, both cached;
an eleven-condition `jq -e` comparison including the rejected null path
returned `true`. [First parity report](dev46-qa-cli-parity-one.json) and
[second parity report](dev46-qa-cli-parity-two.json) are exact outputs.
The first absent-fixture attempt outside the Nix shell lacked `bwrap`;
the successful actual CLI run used `nix develop -c` to provide the sandbox.

The earlier dynamic-plan core suite passed 48/48, including V1 golden parity
and three V2 core fixtures. Native V1 worker compatibility tests passed 7/7;
initial V2 worker tests passed 5/5; a focused source-batch test passed 1/1.
The **pre-QA** 687-test crunch-build suite included the active-prefix
regression, and the subsequent three V2 negatives and root JSON regression
passed in focused runs; a new full candidate suite has not been claimed here.
No foreign content-addressed-unit placeholder path was changed, and
`Cargo.lock` is untouched.

T1.1 is now checked from the clean pinned-main before-change receipt above. T4.4 (all clean gates, including strict root Clippy, the current Cairn parser error, and global Cairn validation) and T4.5 (sync/archive) remain unchecked. This source reconstruction is **not** an archive, an accepted ADR, or evidence that the unrelated red gates passed. It makes no source-trust, compiler-correctness, universal-backend-atomicity, or release-eligibility claim. No stage, commit, push, or sync occurred.
