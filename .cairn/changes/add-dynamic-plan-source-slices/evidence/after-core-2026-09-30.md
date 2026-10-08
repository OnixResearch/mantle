# Dynamic-plan v2 implementation evidence (2026-09-30)

The first checkpoint below covers only pure `crunch-build::dynamic_plan`
v2 decoding/admission and bounded slice planning, **not** worker admission,
signed PathInfo publication, or an end-to-end build receipt. Its scratch
target was `/home/brittonr/scratch/mantle-dynamic-plan-after-target`;
those Cargo commands ran in the live Mantle checkout inside
`nix develop --offline --no-write-lock-file` with
`TMPDIR=/home/brittonr/scratch`. Baseline was the clean, detached
`origin/main` worktree recorded in `baseline-2026-09-30.md`. Subsequent
sections record later worker/store, root JSON, and CLI evidence explicitly.

## Commands and observed results

```text
# Baseline, clean detached worktree:
CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-baseline-target CARGO_BUILD_JOBS=4 TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command cargo test -p crunch-build --lib dynamic_plan::tests:: -- --test-threads=1
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 633 filtered out; finished in 0.03s

# After, live checkout, following correction of a test assertion that expected pre-canonical source order:
CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target CARGO_BUILD_JOBS=4 TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command cargo test -p crunch-build --lib dynamic_plan::tests:: -- --test-threads=1
test dynamic_plan::tests::v2_slice_canonical_order_and_digest_bind_expected_content ... ok
test dynamic_plan::tests::v2_rejects_bad_subpaths_undeclared_output_and_source_conflict ... ok
test dynamic_plan::tests::v2_enforces_slice_count_depth_and_canonical_byte_ceiling ... ok
test dynamic_plan::tests::slice_planner_accepts_two_and_rejects_without_partial_plan ... ok
test dynamic_plan::tests::wire_projection_preserves_frozen_canonical_bytes_and_plan_digest ... ok
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 633 filtered out; finished in 0.26s

# Final dedup name-boundary assertion (added after the 48-test run):
CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target CARGO_BUILD_JOBS=4 TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command cargo test -p crunch-build --lib dynamic_plan::tests::slice_planner_accepts_two_and_rejects_without_partial_plan -- --exact
test dynamic_plan::tests::slice_planner_accepts_two_and_rejects_without_partial_plan ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 680 filtered out; finished in 0.00s

TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command cargo fmt --check -p crunch-build
# exit 0, no output
CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target CARGO_BUILD_JOBS=4 TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command cargo clippy -p crunch-build --all-targets --no-deps -- -D warnings
    Checking crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-build)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.76s
```

A prior strict Clippy run found `clippy::collapsible_if` in v2 duplicate admission; that condition was corrected and the above final strict run passed. An initial 48-test after run failed one fixture assertion expecting wire insertion order after admission was changed to canonical sorting; corrected subsequent 48-test run passed.

## Golden byte and digest parity

```text
cmp /home/brittonr/scratch/mantle-dynamic-plan-baseline-da00f5/crates/crunch-build/testdata/dynamic-plan-v1-canonical.json crates/crunch-build/testdata/dynamic-plan-v1-canonical.json && b3sum /home/brittonr/scratch/mantle-dynamic-plan-baseline-da00f5/crates/crunch-build/testdata/dynamic-plan-v1-canonical.json crates/crunch-build/testdata/dynamic-plan-v1-canonical.json
dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750  /home/brittonr/scratch/mantle-dynamic-plan-baseline-da00f5/crates/crunch-build/testdata/dynamic-plan-v1-canonical.json
dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750  crates/crunch-build/testdata/dynamic-plan-v1-canonical.json
```

`cmp` exited zero. The v1 golden test passed before and after; canonical bytes and BLAKE3 plan digest match this frozen fixture. v2 uses a separate DTO and schema discriminator, not a v1 serializer change.

## External public-API smoke

A temporary Rust consumer in `/home/brittonr/scratch` imported
`crunch_build::dynamic_plan::{decode_validated_plan_v2, plan_slices}` and
the public slice types. It adapted the frozen v1 JSON into a valid v2
document declaring two distinct source ids with the same content/name at
different subpaths, supplied two tree facts, then supplied a mismatched
observed NAR BLAKE3. It was compiled against the built `crunch-build`
library in the scratch target with:

```text
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command rustc --edition 2024 -L dependency=/home/brittonr/scratch/mantle-dynamic-plan-after-target/debug/deps --extern crunch_build=/home/brittonr/scratch/mantle-dynamic-plan-after-target/debug/deps/libcrunch_build-9e31adbadb7610d7.rlib -o /home/brittonr/scratch/mantle-dynamic-plan-smoke /home/brittonr/scratch/mantle-dynamic-plan-smoke.rs
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file --command /home/brittonr/scratch/mantle-dynamic-plan-smoke
v2-core-smoke: admitted=2 canonical_digest=842e08fa11fff7bb8b00a0f15f6253773af287173436536bb42933a052f82d85
source=src.main publication_owner=src.main subpath=packages/a
source=src.other publication_owner=src.main subpath=packages/b
rejected=slice-digest-mismatch source=src.other
```

The temporary source and binary were removed after this successful run.
The output demonstrates public pure-core behavior, not store publication.

## Measured fixture boundaries

- `v2_slice_canonical_order_and_digest_bind_expected_content`: two wire slices canonicalize by source id regardless of input order; reordering preserves canonical bytes/digest, changing one declared NAR digest changes plan digest.
- `slice_planner_accepts_two_and_rejects_without_partial_plan`: two subpaths with the same declared name and NAR digest share one `publication_source_id` but keep two source ids; changing one name yields separate publication ids. Rejected wrong subpath fact, absent subtree, intermediate symlink, mismatched digest, aggregate admitted bytes exceeding 1 GiB, undeclared output, and over-limit tree facts return typed errors and no candidate vector.
- `v2_rejects_bad_subpaths_undeclared_output_and_source_conflict`: rejects absolute, `..`, dot, empty and trailing components, over-4-KiB subpaths, invalid Nix store names, conflicting duplicate source ids, and slice fields in v1; an identical duplicate slice id collapses to one admitted source.
- `v2_enforces_slice_count_depth_and_canonical_byte_ceiling`: depth 32 accepted / 33 rejected; 257 slices rejected against limit 256; oversized canonical projection rejects beyond 4 MiB.

## Lifecycle and claim boundary

The exact lifecycle commands (with `TMPDIR=/home/brittonr/scratch`) were:

```text
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-dynamic-plan-source-slices --root .
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-dynamic-plan-source-slices --root .
nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-dynamic-plan-source-slices --root .
```

Proposal, design, and tasks each returned `"verdict": "PASS"`; the tasks gate
was rerun after checkbox changes and reported `"task_done": 9`,
`"task_todo": 7`, `"issues": []`. Global validation reported no
change-specific issues but the pre-existing
`task_ordering.progress.unsatisfied` in
`.cairn/changes/thin-cli-composition-root/tasks.md:88` (I7 checked before
I6). This change did not edit that other Cairn.

Contract description: [`docs/nominal-dynamic-plan-types.md`](../../../../docs/nominal-dynamic-plan-types.md). Proposed decision: [`adr/0084-dynamic-plan-source-slices.md`](../../../../adr/0084-dynamic-plan-source-slices.md), indexed after ADR 0083. **Time-scoped prerequisite (pure-core checkpoint):** the SourceAdmission capability currently exposes only per-object `preflight` and `ingest`; a verified-source batch wrapper has not yet been integrated. The underlying PathInfo service supports `put_batch_atomic`, so this is not an architectural impossibility. The worker still decodes v1 only. T3.1–T3.4, T4.1, T4.4, and T4.5 remain open; no actual v2 worker build, unit derivation identity, report row, or archive claim is made.

## Subsequent worker/store integration checkpoint

The preceding pure-core prerequisite was time-scoped. The current worker
decodes v1 or v2 with a borrowed schema probe, observes each v2 source
subtree through read-only castore access, plans the complete batch, and uses
the Builder's separate signed `SliceAdmission` capability before registering
units. Accepted and rejected native reports now contain canonical slice rows.
The store's logical PathInfo batch transaction does not promise physical
castore rollback or cross-process signer no-clobber; an uncertain publication
error aborts the worker instead of claiming a clean rejection.

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target nix develop --offline --no-write-lock-file -c cargo test -p crunch-build --lib native_v2_slice -- --test-threads=1
test worker::tests::native_v2_slice_aggregate_byte_limit_preserves_scheduler_and_publication ... ok
test worker::tests::native_v2_slice_failure_leaves_store_registry_goals_scheduler_and_report_unadmitted ... ok
test worker::tests::native_v2_slice_publication_survives_two_producer_reruns ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 681 filtered out; finished in 0.36s
```

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target nix develop --offline --no-write-lock-file -c cargo test -p crunch-build --lib dynamic_plan::tests:: -- --test-threads=1
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 636 filtered out; finished in 0.41s

TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target nix develop --offline --no-write-lock-file -c cargo test -p crunch-build --lib -- --test-threads=1
test result: ok. 684 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.58s
```

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target nix develop --offline --no-write-lock-file -c cargo test -p crunch-pipeline --lib -- --test-threads=1
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.10s
```

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-live-state-current-target nix develop --offline --no-write-lock-file -c cargo test -p mantle --bin mantle build_report::tests::build_json_report_includes_artifact_attestation_reference -- --test-threads=1
test build_report::tests::build_json_report_includes_artifact_attestation_reference ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2626 filtered out; finished in 0.00s
```

```text
nix develop --offline --no-write-lock-file --command env CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-backend-status/target cargo test -p crunch-store --lib capability::tests::source_slice_batch_publishes_both_backends_without_partial_conflicts -- --exact
test capability::tests::source_slice_batch_publishes_both_backends_without_partial_conflicts ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 401 filtered out
```

The backend owner's scoped test exercised real Snix and Casita signed
source-slice batches, idempotence, wrong-key rejection, absent/final and
intermediate symlink traversal, the 257-entry limit, and a second-candidate
conflict without publishing the first candidate PathInfo. This test's
logical transaction claim does not establish physical castore cleanup or
cross-process signer no-clobber.

The two-run worker test changed producer bytes outside `packages/a` and
`packages/b`, observed distinct producer roots, checked both declared slice
ids reuse one trusted, signed CA PathInfo, checked the unit derivation key
and its source input/argument stay equal across runs, and ran the registered
goals. The negative test checked digest mismatch, absent subtree, final and
intermediate symlink, plan-artifact output, and undeclared producer output:
the first candidate PathInfo, registry, goals, scheduling epoch, and success
report all remained unadmitted. A separate 256-slice fixture exceeded the
1 GiB aggregate byte limit before publication and likewise kept scheduler
and store state unchanged. These scoped tests do **not** establish a real
sandbox CLI build; that acceptance check follows separately.

## External two-run CLI and sandbox receipt

An external Nickel producer under the scratch path below declared an `out`,
`plan`, and `sources` output. Its native plan declared `src.main` at
`sources/packages/a` and `src.other` at `sources/packages/b`, with the same
`unchanged-package` store name and expected NAR BLAKE3. A real shell
builder at the admitted `src.main` path read `$SOURCE/file.txt` inside
the sandbox and wrote `slice:package-content-stable` to the unit output.
The first CLI run built both producer and unit (`cached: false` for each);
its JSON `failed` list was empty. Only the producer's write to the
*unbound* `sources/outside.txt` changed from `outside-first` to
`outside-second` in the definitive rerun; the producer name, `out` marker,
package files, and plan were unchanged. The second run reused the same
state, store, and compiled CLI binary, built the producer under a distinct
derivation key, and reported the unit cached with an empty `failed` list.

```text
env CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-live-state-current-target TMPDIR=/home/brittonr/scratch nix develop --offline -c cargo build -p mantle --bin mantle
# Shared binary build: exit 0, supplied by live-state owner.
env TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c /home/brittonr/scratch/mantle-live-state-current-target/debug/mantle --nix-compat --store /home/brittonr/scratch/mantle-v2-cli-fixture/store --state-dir /home/brittonr/scratch/mantle-v2-cli-fixture/state --json build /home/brittonr/scratch/mantle-v2-cli-fixture/run-one/producer.ncl --no-substitute
# Same invocation after changing only the outside.txt write from outside-first to outside-second.
```

Observed JSON and physical outputs, in first/second order:

| Observation | First | Second |
| --- | --- | --- |
| Producer derivation | `/nix/store/4pb4rymbi0zbjl78ag6pzvlx5llcq6zx-v2-external-producer-first.drv` | `/nix/store/fcqn428lmjrd3w7y74h6xam0bs83n1ml-v2-external-producer-first.drv` |
| Producer `sources` physical root | `1v5h7z5k14jmxzl38xlcyijkc3q5hr19-v2-external-producer-first-sources` | `7kga5vzgnkd4wa6srxsic2vq06d8bwq1-v2-external-producer-first-sources` |
| `sources/outside.txt` bytes | `outside-first` | `outside-second` |
| Producer `out` marker and physical path | `producer-first`; `y1f0z30c27k1p63c9ic1nshqbl0k0hc1-v2-external-producer-first` | identical |
| Producer `plan` output physical path | `yyxfg2nn0hvb657fzvsgk5mjf1y4qx2z-v2-external-producer-first-plan` | identical |
| `src.main` and `src.other` observed NAR BLAKE3 | `a8758b396c10d15c46e5a5b6f7f97d835d7610f5734775a9d08755ba2c264d5a` | identical |
| Both admitted store paths | `/nix/store/p3s7rhbb2czcg6y4wjc19sq1l93q9i3c-unchanged-package` | identical |
| Dynamic unit derivation | `/nix/store/dy9i266awqmmxl5h4cjh23ifcjnxixs7-v2-slice-unit.drv` | identical |
| Dynamic unit output | `/nix/store/19bkcn0zj0z7176xqrl2y93bb6zwns8h-v2-slice-unit` | identical, cached |

Both JSON reports contained source-slice rows in `src.main`, `src.other`
order with declared and observed digests equal, `disposition: admitted`,
one shared admitted path, `scheduler_action: registered-roots`, and
`canonical_plan_digest: 25c37fd7b709ef5a2595ff9eb3725e739a2a976a0993dda492a44f3b4fcc505c`.
The first physical unit output was read as `slice:package-content-stable`.
For both ids the planner selected `src.main` as publication owner; the
worker sends one signed batch request for that owner and binds `src.other`
to its returned path. The report's shared path is thus backed by one CA
publication candidate per plan rather than duplicate requests for both ids.
This proves the observed two-run CLI/sandbox scenario, not source
correctness beyond the admitted digest or release eligibility.

## Package and Cairn gates after worker integration

```text
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c cargo fmt --check -p crunch-build
# exit 0
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c cargo fmt --check -p crunch-pipeline
# exit 0
TMPDIR=/home/brittonr/scratch nix develop --offline --no-write-lock-file -c rustfmt --edition 2024 --check src/build_report.rs
# exit 0 after formatting the touched report fixture
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target nix develop --offline --no-write-lock-file -c cargo clippy -p crunch-build --all-targets --no-deps -- -D warnings
# Finished dev profile, exit 0
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-dynamic-plan-after-target nix develop --offline --no-write-lock-file -c cargo clippy -p crunch-pipeline --all-targets --no-deps -- -D warnings
# Finished dev profile, exit 0
```

After the final worker publication-error branch avoided formatting the
error string solely to classify a known uncertain `Error::Store` payload,
the focused `native_v2_slice` command above was rerun: 3 passed,
0 failed, 681 filtered, 0.47 s. The crunch-build formatter check and
strict Clippy command above were rerun after that edit; both exited zero.

`cairn gate proposal`, `gate design`, and `gate tasks` all returned
`"verdict": "PASS"` and `"issues": []` for
`add-dynamic-plan-source-slices` after integration; the tasks receipt
reported 14 done of 16 (strict final root gate and isolated-worktree
archive remain open). `cairn validate --root .` reported
`"change_issues": []`, but the repository-wide verdict retains the
unrelated `thin-cli-composition-root/tasks.md:88` predecessor-order
issue recorded at the pure-core checkpoint.

Strict root `cargo clippy -p mantle --bin mantle --no-deps -- -D warnings`
could not complete while a separate in-flight Rust-plan core/app boundary
made `PlanBlocker::new` private to its app callers (E0624 at
`crates/mantle-rust-plan-app/src/application.rs:194,201,251`). The owning
agent is repairing that boundary; this first attempt is **not** a passed
root Clippy gate.

The read-only Cairn `sync add-dynamic-plan-source-slices --root .` preview
returned `blocked: false`, `mutated: false`: five ADDED requirement blocks
would populate the absent
`.cairn/specs/dynamic-plan-source-slices/spec.md`, with no merge-preflight
diagnostics. The read-only `archive ... --root .` preview returned
`blocked: true`, `mutated: false` while T4.4 and T4.5 are still unchecked
(`tasks not archive-ready (todo: 2)`). Promotion belongs in the later
isolated branch/worktree, not this shared checkout.
