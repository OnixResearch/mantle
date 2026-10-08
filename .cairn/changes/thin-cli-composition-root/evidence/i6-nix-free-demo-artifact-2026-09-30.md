# I6 slice: the `nix-free-demo` and `artifact` operations plan before their effects and run them through typed ports

Change: `thin-cli-composition-root`
Task-ID: I6. Still open. This record covers `nix-free-demo generate`, `validate`, and `readme`, and all six `artifact` actions: `export`, `import`, `oci-export`, `oci-import`, `oci-push`, and `oci-pull`.
Subject: `da00f5842` plus uncommitted edits in the main checkout:
`crates/mantle-application-contract/src/{nix_free_demo,artifact}.rs` (new),
`crates/mantle-application-contract/src/lib.rs` (modules and re-exports),
`src/nix_free_demo_cmd.rs`, `src/artifact_cmd.rs`, and `tests/nix_free_demo_cli.rs`.
Status: at the snapshot proven below, all nine `nix-free-demo` and `artifact` operations are migrated. I6 stays open because the other command roots are not. Every claim below cites a run that actually completed.

## Snapshot scope (read first)

Every verification claim in this record applies to the owned-file hashes
recorded with the isolated proofs below. The final proven snapshot was:

| File | sha256 prefix |
| --- | --- |
| `src/nix_free_demo_cmd.rs` | `36a81335` |
| `src/artifact_cmd.rs` | `61656149` |
| `tests/nix_free_demo_cli.rs` | `8a2fbf79` |
| contract `nix_free_demo.rs` | `9b0416f9` |
| contract `artifact.rs` | `ec065449` |
| contract `lib.rs` | `44c22178` |

After that proof, another worker's change, which `tasks.md` records as the
2026-10-01 "bounded envelope cutover", modified several of these files.
File mtimes are from 2026-10-01 01:24 to 02:05 -0400.

- `crates/mantle-application-contract/src/envelope.rs`: `plan_effects` now
  takes `&[EffectSpec]` and returns `Result<EffectPlan, PlanError>`.
- Contract `artifact.rs` is now `a01f7e8a`. `classify_artifact_call`,
  `classify_artifact_registry_transfer`, and `classify_artifact_export` no
  longer exist.
- Contract `nix_free_demo.rs` is now `2a42601c`.
- `src/artifact_cmd.rs` is now `cb61abdf`.

`src/nix_free_demo_cmd.rs` and `tests/nix_free_demo_cli.rs` still match the
proven hashes.

This record makes no claim about the current versions of the changed files.
Their verification belongs to the cutover's own evidence,
`evidence/i6-envelope-2026-10-01.md`. The design descriptions below describe
the proven snapshot.

That envelope record also cites a read-only audit with an acceptance blocker,
B2, which it says is owned in `artifact.rs`/`artifact_cmd.rs`. B2 states that
the artifact call, registry, and export classifiers synthesize output, kind,
and usage instead of observing adapter facts.

The limit applies to this record's snapshot. Its artifact observations carry
only a status and a diagnostic code, both derived from whether the executed
port call returned or failed (or, for export, from the export decision's
first diagnostic). They carry no adapter-observed output identity, effect
kind, or usage. This record does not claim B2 fixed, and it does not claim
the wrong-observation proof that the envelope record still requires.

## Correction: the earlier version of this record is withdrawn

An earlier version of this file claimed that all nine `nix-free-demo` and
`artifact` actions classify typed observations. That claim was false.

- Its `classify_nix_free_demo_effect` and `classify_artifact_effect` helpers
  built an effect plan from a string after the effect had already run. For
  example, `write_generated_bundle` ran before `classify_nix_free_demo_effect`,
  and artifact imports and exports ran before `classify_artifact_effect`.
- No port executed those plans.
- The observations were synthesized from a report flag.

Those helpers, their constants, and their three new `artifact_cmd` unit tests
were removed. `src/artifact_cmd.rs` was returned to its `da00f5842` content,
and only then was the `artifact import` migration below applied to it.

## Not migrated (I6 remains open for these)

- Every other command root listed as unclassified in the I6 coverage notes in
  `tasks.md` is unchanged. This record covers only the `nix-free-demo` and
  `artifact` roots.

## What changed for `nix-free-demo validate` and `readme`

Both operations consume one summary file. The contract adds:

- `nix_free_demo_summary_effect_plan()`, a `plan_effects` plan with one
  `read-files` effect;
- the port `NixFreeDemoSummaryPort`, whose single method `read_summary` reads
  only the path the operator named;
- `classify_nix_free_demo_summary_read`, which classifies the port call's
  actual result.

In `src/nix_free_demo_cmd.rs`, `read_summary` builds the plan before it calls
the `SummaryFile` adapter, then classifies that read before `validate` or
`readme` prints anything. A read the port refuses returns `RunError::Internal`
with the `da00f5842` message (`reading Nix-free demo summary <path>: <error>`).
The claimability decision and the malformed-summary report stay where they
were at `da00f5842`. They are domain decisions over the summary that was read,
not observations of the read.

## What changed for `nix-free-demo generate`

The contract module `nix_free_demo.rs` is `no_std` and has no effects. It provides:

- `nix_free_demo_generate_effect_plan()`, which builds the plan through
  `plan_effects` with a `read-files` effect followed by a `write-files` effect;
- the capability-scoped port `NixFreeDemoBundlePort`, with `probe_output`,
  `read_transcript` by declaration index, and `write_bundle`;
- `NixFreeDemoInputFacts`, which the read records as it runs;
- `classify_nix_free_demo_rejection`, which takes the read observed from the
  facts, records the write as `Skipped`, and never returns complete;
- `classify_nix_free_demo_generate`, which may be called only after the write
  port ran. It fails closed when the facts do not back the bundle, or when the
  port reports fewer transcripts or documents than planned. It builds its
  observations as a stack array.

The shell (`src/nix_free_demo_cmd.rs`) works in this order:

1. It builds the plan before the first port call.
2. The adapter `GenerateBundleFiles` holds only `--out` and the declared
   transcript paths.
3. The admission reads go through the port, in the same order and with the same
   messages as at `da00f5842`: the output probe, then each transcript in
   declaration order, stopping at the first one that cannot be admitted.
4. A rejected request classifies its read with the write skipped, then prints
   the same rejected report with `Reported(1)`. No write runs.
5. An admitted request serializes the documents first. It then calls
   `write_bundle` once and classifies that executed result before reporting.
6. A write the port refuses returns `RunError::Internal` with the adapter's
   message, which is identical to the message at `da00f5842`.

Diagnostic precedence is kept on purpose. The output-directory probe and the
transcript reads happen before pure argument rejection (guards, receipt and
artifact digests, bundle validation), exactly as at `da00f5842`. So a request
with malformed arguments still performs those read-only probes, and it never
writes.

## Verification

Environment: the main checkout `/home/brittonr/git/OnixResearch/mantle` inside
`nix develop`. `CARGO_TARGET_DIR` and `TMPDIR` pointed at
`/home/brittonr/scratch/mantle-i6/{target,tmp}`, never the shared
`~/.cargo-target`, and `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`. The rail was pueue
task 255 running `scratch/mantle-i6/run-main-focused.sh after-generate`. Raw
logs are in `scratch/mantle-i6/runs/after-generate/`. Scratch paths are
host-local, not durable.

| Leg | Command | Result |
| --- | --- | --- |
| contract unit | `cargo test -p mantle-application-contract --lib -- --test-threads=1` | `ok. 67 passed; 0 failed`, including the 6 new `nix_free_demo::tests` |
| contract fixtures | `cargo test -p mantle-application-contract --tests -- --test-threads=1` | `ok. 67 passed`, `ok. 7 passed`, `ok. 8 passed`; `exit=0` |
| build | `cargo build -p mantle --bin mantle` | `exit=0`; the copy `bin/mantle-after-generate` has sha256 `7be7531e…` |
| shell unit | `cargo test -p mantle --bin mantle nix_free_demo_cmd:: -- --test-threads=1` | `ok. 4 passed; 0 failed` |
| CLI | `cargo test -p mantle --test nix_free_demo_cli -- --test-threads=1` | `ok. 12 passed; 0 failed`; `exit=0` |
| architecture | `cargo -Zscript scripts/check-cli-architecture.rs` | `cli architecture: PASS`; `exit=0` |

The pueue rail ended with `rail-exit=0`. The source sha256 values are recorded
in `runs/after-generate/subject.txt`.

The 6 new contract tests cover these behaviors:

- a complete write of admitted inputs completes;
- a write the port refuses fails the generation;
- a write that reports fewer transcripts or documents than planned fails closed;
- a bundle not backed by an observed free output and a read of every copied
  transcript fails closed;
- a rejection records the write as skipped and is never complete, with
  `Failed{1}` after a clean read and `Failed{2}` after an unreadable input;
- only an absent or empty output is free.

An earlier plan-shape test was removed because it checked wiring rather than
behavior.

Real-binary smoke ran `scratch/mantle-i6/smoke-generate.sh` with the
unmodified `da00f5842` binary as `before` and the patched binary as `after`.
The `before` binary was built from a detached scratch worktree at the same
HEAD; its sha256 is `062a596d…`. Both runs used one fixed work root. For each
scenario the smoke recorded:

- stdout and stderr;
- the exit code;
- the output tree, with type, mode, and sha256 per file.

`compare-smoke.sh` reported all 14 scenarios identical:

| Scenario | Exit | Output state (both binaries) |
| --- | --- | --- |
| `success-json`, `success-human`, `success-empty-out` | 0 | `README.md`, `manifest.json`, `summary.json`, `validation.json`, `transcripts/proof.log`. The JSON report lists exactly these files. |
| `reject-missing-transcript`, `reject-transcript-is-directory`, `reject-bad-guard`, `reject-bad-receipt-digest`, `reject-contradictory-status` | 1 | output absent |
| `reject-output-occupied`, `precedence-occupied-over-bad-guard` | 1 | only the pre-existing `unrelated.txt`, unchanged; code `output-conflict` |
| `reject-output-is-file` | 1 | the file unchanged; `output-conflict` with `read output dir: Not a directory (os error 20)` |
| `precedence-missing-transcript-over-bad-guard` | 1 | output absent; code `missing-transcript` |
| `write-fails-creating-out` | 3 | no stdout; the error is `creating …/bundle: Not a directory (os error 20)` |
| `write-fails-after-partial-copy` | 3 | no stdout; `copying transcript …/second/proof.log: Permission denied (os error 13)`. Only `transcripts/proof.log`, mode 444, exists, and no document was written. |

The new CLI tests in `tests/nix_free_demo_cli.rs` cover the same behaviors:

- rejected requests create no output, and an occupied output is left as it was;
- unreadable inputs are reported before argument errors;
- the reported `files` equal the files on disk, and the manifest digest is the
  transcript's BLAKE3;
- a partial-write failure exits 3 with no report. This test skips when the user
  can write read-only files.

All 12 CLI tests passed, including the 4 new ones.

[INFERENCE] The partial-write test may have taken its own skip branch, which
would also report as passing. That branch is unlikely here: the smoke run
recorded `Permission denied (os error 13)` for this same scenario as the same
user.

Quality checks run on the changed paths:

- `rustfmt --edition 2024` on `src/nix_free_demo_cmd.rs`,
  `tests/nix_free_demo_cli.rs`, and the new contract module: `rustfmt-exit=0`.
  Crate-root `lib.rs` was not formatted, to avoid cascading into sibling
  modules.
- `git diff --check` on the tracked changed paths: `exit=0`.
- `git diff --no-index --check` for the new contract file listed no whitespace
  errors.

### `validate` and `readme` (rail `after-summary`, pueue task 256)

The same script and environment ran over the tree with the summary port. Logs
are in `scratch/mantle-i6/runs/after-summary/`, and the rail ended with
`rail-exit=0`.

| Leg | Result |
| --- | --- |
| contract unit | `ok. 68 passed; 0 failed`. This included `a_summary_read_completes_only_when_the_port_read_it`, which was later removed because it only fed fabricated `Ok`/`Err` values to the classifier. The summary paths are covered by the CLI tests below. |
| contract fixtures | `ok. 68 passed`, `ok. 7 passed`, `ok. 8 passed`; `exit=0` |
| build | `exit=0`; `bin/mantle-after-summary` has sha256 `1998a294…` |
| shell unit | `ok. 4 passed; 0 failed` |
| CLI `nix_free_demo_cli` | `ok. 12 passed; 0 failed` |
| architecture | `cli architecture: PASS` |

The CLI tests cover the summary paths:

- `nix_free_demo_cli_validates_claimable_bundle_as_json` (exit 0);
- `…_rejects_missing_fixed_point_without_success_claim` and
  `…_rejects_missing_guard_as_json` (exit 1);
- `…_renders_readme_from_summary` and
  `…_readme_of_unclaimable_summary_renders_and_exits_one`;
- `…_keeps_distinct_exit_classes_for_malformed_and_unreadable_summaries`: a
  malformed summary exits 1 with the `malformed-summary` report, and a summary
  the port cannot read exits 3 with no stdout and the read error on stderr.

Real-binary smoke `scratch/mantle-i6/smoke-summary.sh` ran with the
`da00f5842` binary (`062a596d…`) as `before` and `mantle-after-summary` as
`after`. It covered 18 scenarios: `validate` and `readme` × claimable, blocked,
and malformed summaries in JSON and human mode, plus a missing summary (human
and JSON) and a directory given as the summary. stdout, stderr, and the exit
code were byte-identical in all 18. The exits were 0 for claimable, 1 for
blocked and malformed, and 3 for missing and directory.

`rustfmt --edition 2024` on `src/nix_free_demo_cmd.rs` and the contract module
exited 0.

Not run in this rail: Clippy, Tiger Style, `nix flake check`, Cairn
validation and gates.

### `artifact import` (rail `after-import`, pueue task 258)

Contract `artifact.rs` provides:

- `artifact_import_effect_plan()`, a `plan_effects` plan with one `write-files`
  effect;
- `ArtifactImportPort`, with a single `import_artifact` method;
- `classify_artifact_import`, which classifies the executed call's result.

The operation boundary is one composite port call. The `ArtifactImportFiles`
adapter runs the existing `import_frontend_artifact`, which reads the source
tree and writes the store content and manifest, and then writes the requested
`--report-out` report. `cmd_artifact_import` builds the plan before that call
and classifies the call before it prints anything. A refused call returns
`RunError::Internal` with the `da00f5842` message.

Non-claims: the single observation does not say whether a failure happened in
the source read, the store write, or the report write. A failed observation
does not mean nothing was written. The report write runs after the store
import, so a failed report write leaves the artifact in the store, as at
`da00f5842`.

Logs are in `scratch/mantle-i6/runs/after-import/`, and the rail ended with
`rail-exit=0`:

| Leg | Result |
| --- | --- |
| contract unit | `ok. 67 passed; 0 failed`. `artifact.rs` has no unit test, because the classifier only maps the port result. |
| build | `exit=0`; `bin/mantle-after-import` has sha256 `a7e9ac0c…` |
| `artifact_cmd::` unit | `ok. 15 passed; 0 failed` |
| `kernel_bundle_oci_cli` | `ok. 4 passed; 0 failed`. Its fixtures run `mantle --json artifact import … --report-out …` through the binary. |
| architecture | `cli architecture: PASS` |

The 3 new `artifact_cmd` unit tests run the real operation against real files:

- `artifact_import_of_a_missing_source_writes_neither_store_nor_report`: the
  error starts `importing frontend artifact: `, the state directory is never
  created, and no report is written.
- `artifact_import_report_write_failure_fails_after_the_store_import`: the
  error starts `creating `, no report is written, and
  `frontend_artifact_is_available` reports the artifact in the store.
- `artifact_import_stores_the_source_identity_and_reports_it`: the artifact is
  absent before the import and available after it. The written report's
  `artifact_ref` equals `frontend_artifact_identity(source)`, and the stored
  bytes equal the source.

Real-binary smoke `scratch/mantle-i6/smoke-artifact-import.sh` ran with the
`da00f5842` binary (`062a596d…`) as `before` and `mantle-after-import` as
`after`. For each scenario it recorded stdout, stderr, the exit code, and the
state and report trees with sha256 per file. All 6 scenarios were identical:

| Scenario | Exit | State |
| --- | --- | --- |
| `json`, `reimport` (a second import into the same state) | 0 | content and manifest in the store |
| `human-report` (report under a parent that does not exist yet) | 0 | store files plus the report |
| `json-tree-report` (a directory source) | 0 | store tree plus the report |
| `missing-source` | 3 | `importing frontend artifact: frontend artifact source does not exist: …`; no state and no report |
| `report-under-file` | 3 | `creating …/report-parent-is-a-file: File exists (os error 17)`; content and manifest are in the store, and there is no report |

`rustfmt --edition 2024 --check` on `src/artifact_cmd.rs`, `artifact.rs`, and
`nix_free_demo.rs` exited 0.

Not run: Clippy, Tiger Style, `nix flake check`, Cairn validation and gates,
and the `kernel_bundle_oci_registry_cli` suite.

### `artifact oci-export` and `oci-import` (rail `after-oci-layout`, pueue task 259)

After the import slice passed, the contract seam was renamed for its three
single-call users. `artifact.rs` now provides:

- `artifact_call_effect_plan()`, which keeps the one `write-files` effect;
- `classify_artifact_call`;
- the ports `ArtifactImportPort`, `OciLayoutExportPort` (`export_layout`), and
  `OciLayoutImportPort` (`import_layout`).

In `src/artifact_cmd.rs`:

- `cmd_oci_export` and `cmd_oci_import` build the plan before calling their
  adapters. `OciLayoutExportFiles` and `OciLayoutImportFiles` hold only the
  declared `OciExportRequest` and `OciImportRequest`.
- Both classify the executed call through `classified_artifact_call`, which
  `cmd_artifact_import` now shares, before they print anything.
- A refused call returns `RunError::Internal` with the `da00f5842` message
  (`exporting OCI layout: …` / `importing OCI layout: …`).

The module doc carries the same non-claims as import. The observation does not
attribute a failure to reading, validating, or writing. An adapter's own
admission rejection, such as an existing output or a failed layout
validation, is observed as a failed call, not as a blocked outcome.

Logs are in `scratch/mantle-i6/runs/after-oci-layout/`, and the rail ended with
`rail-exit=0`:

| Leg | Result |
| --- | --- |
| contract unit | `ok. 67 passed; 0 failed` |
| build | `exit=0`; `bin/mantle-after-oci-layout` has sha256 `5ffd56b4…` |
| `artifact_cmd::` unit | `ok. 17 passed; 0 failed`, including two new tests: `artifact_oci_export_into_an_existing_output_fails_and_leaves_it_untouched` (the error starts `exporting OCI layout: OCI output already exists: `, and the output keeps exactly its one prior file and bytes) and `artifact_oci_import_of_a_missing_layout_writes_no_report` |
| `kernel_bundle_oci_cli` | `ok. 4 passed; 0 failed`: the round trip through `oci-export` and `oci-import`, a tampered layer rejected before the import report, and an unsealed projection rejected before object reads |
| `kernel_bundle_oci_registry_cli` | `ok. 10 passed; 0 failed; 1 ignored`. Its fixtures run `artifact import` and `oci-export` through the binary. The ignored test needs the pinned external registry binary, as in the `da00f5842` baseline logs. |
| architecture | `cli architecture: PASS` |

Real-binary smoke against the `da00f5842` binary:

- `scratch/mantle-i6/smoke-oci-layout.sh` covered 5 failure scenarios with
  identical stdout, stderr, exit code, and work tree: an export into an
  existing output, a missing projection, a malformed projection, an import of
  a missing layout, and an empty layout. All exit 3.
- `smoke-artifact-import.sh` ran again with `mantle-after-oci-layout` and was
  identical in all 6 scenarios.

`rustfmt --edition 2024` on `src/artifact_cmd.rs` and `artifact.rs` exited 0.
Scoped `git diff --check` exited 0. `git diff --no-index --check` on the
untracked contract and evidence files listed no whitespace errors.

Not run: Clippy, Tiger Style, `nix flake check`, Cairn validation and gates.

### `artifact oci-push` and `oci-pull`

Contract additions in `artifact.rs`:

- the ports `OciRegistryPushPort` and `OciRegistryPullPort`, each with three
  methods: `read_inputs`, then `push` or `pull` over an admitted request, then
  `write_receipt`;
- `ArtifactRegistryProgress`, with the variants `InputsRefused`, `Rejected`,
  `TransferFailed`, `ReceiptFailed`, and `Recorded`;
- `artifact_registry_effect_plan()`, a `plan_effects` plan with `read-files`,
  then `use-network`, then `write-files`;
- `classify_artifact_registry_transfer`.

For each variant, the transfer and receipt effects after the stop are
`Skipped`.

In `src/artifact_cmd.rs`, `cmd_oci_push` and `cmd_oci_pull` work in this order:

1. They build the plan before the first port call.
2. `read_inputs` runs the `da00f5842` `require_absent_output` receipt probe,
   then the trust-policy load, and for a push then the signing-key load, each
   with its `da00f5842` message. A failure classifies as `InputsRefused`: the
   read is observed failed, and the transfer and receipt are skipped.
3. Pure target admission runs next. A rejection classifies as `Rejected` only
   after `read_inputs` returned successfully, and it keeps the `da00f5842`
   `validating OCI registry target: …` error.
4. The transfer runs. A failure classifies as `TransferFailed`, and the receipt
   is skipped.
5. The receipt write runs. A failure classifies as `ReceiptFailed`.
6. `Recorded` is classified only after the receipt write has returned
   successfully, and only then is the report printed.

The adapters are `OciPushFiles` and `OciPullFiles`. Each holds only the
declared paths, the expected digests, and the bearer-token path.

The pull transfer is composite, as the module doc states: `pull_registry_layout`
fetches the image, then materializes and admits the fetched layout and writes
its import report, all in one call. The adapters' own checks inside push and
pull, such as the policy repository, revocation, and digests, are observed as
a failed transfer call.

Interim rail `after-registry` (pueue 260) was stopped before its CLI suites
because the shared checkout changed under it. Its completed legs were
contract unit `ok. 68 passed` and `artifact_cmd::` unit `ok. 19 passed`. The
19 include `artifact_oci_push_with_a_taken_receipt_stops_before_trust_inputs_and_keeps_the_receipt`
(the error is exactly `OCI registry push receipt already exists: <path>`, and
the receipt bytes are unchanged) and
`artifact_oci_pull_with_an_unloadable_trust_policy_writes_no_outputs`.

### Final state (rail `final`, pueue 262)

This rail ran once over every owned path after the last source edit. That edit
replaced a `clone()` of the skipped observation with a zero-cost helper.

The owned-source hashes recorded at the start equal the hashes at the end:

| File | sha256 prefix |
| --- | --- |
| `src/nix_free_demo_cmd.rs` | `36a81335` |
| `src/artifact_cmd.rs` | `c3234d34` |
| `tests/nix_free_demo_cli.rs` | `8a2fbf79` |
| contract `nix_free_demo.rs` | `9b0416f9` |
| contract `artifact.rs` | `98eebcdd` |
| contract `lib.rs` | `dfac7edd` |

The whole-tree fingerprint moved from `9113850c…` to `177f58c7…`, because
other workers edited unrelated files while the rail ran. That drift is a
non-claim for the full suites.

Legs that ran on a coherent build:

| Leg | Result |
| --- | --- |
| contract unit | `ok. 68 passed; 0 failed` |
| contract fixtures | `ok. 68 passed`, `ok. 7 passed`, `ok. 8 passed` |
| build | `exit=0`; `bin/mantle-final` has sha256 `f99a6244…` |
| `nix_free_demo_cmd::` unit | `ok. 4 passed; 0 failed` |
| `rustfmt --edition 2024 --check` on the five owned Rust files | `exit=0` |

Legs broken by concurrent unrelated edits, not by owned code:

- `artifact_cmd::` unit and `nix_free_demo_cli` failed with
  `error[E0583]: file not found for module daemon` at
  `src/remote_build/live_state.rs:14`. Its owner has since removed that line.
- `kernel_bundle_oci_cli` and `kernel_bundle_oci_registry_cli` failed to
  compile `crunch-build`, which referenced `crunch_store` items not yet
  present: `ObservedSourceSlice`, `VerifiedSourceBatchResult`,
  `VerifiedSourceBatchEntry`, `observe_source_slice`, and
  `admit_verified_source_batch`.
- `check-cli-architecture` reported six "core error ownership" findings, all
  in `crates/crunch-project-core/src/bootstrap_pins.rs`, which is not an owned
  file. No finding names an owned file.

A rerun of only those legs in the shared checkout (pueue 263, phase
`final-rerun`) could not compile either. The owned-file fingerprint was
unchanged (`f842e804…` at start and end), but `crunch-store` failed with
`error[E0599]: no method named as_bytes found for struct GenericArray<…>`
in another worker's in-flight change.

### Isolated proof: HEAD plus the six owned files (pueue 264, phase `isolated`)

To get one coherent snapshot without concurrent edits, `run-isolated.sh` made
a detached worktree at `da00f5842` and copied in only the six owned files.
Their hashes are the same six prefixes listed above, and `git status --short`
in that worktree shows only those six paths. It ran the legs with the scratch
target, then removed the worktree (`worktree-removed=0`). Logs are in
`scratch/mantle-i6/runs/isolated/`, and the rail ended with `rail-exit=0`:

| Leg | Result |
| --- | --- |
| contract unit | `ok. 68 passed; 0 failed` |
| build | `exit=0`; `bin/mantle-isolated` has sha256 `7da4fb5b…` |
| `nix_free_demo_cmd::` unit | `ok. 4 passed; 0 failed` |
| `artifact_cmd::` unit | `ok. 19 passed; 0 failed` |
| `nix_free_demo_cli` | `ok. 12 passed; 0 failed` |
| `kernel_bundle_oci_cli` | `ok. 4 passed; 0 failed` |
| `kernel_bundle_oci_registry_cli` | `ok. 10 passed; 0 failed; 1 ignored` |
| architecture | `cli architecture: PASS` |

The registry suite drives the migrated `oci-push` and `oci-pull` through the
binary against its in-process loopback registry. These tests passed:

- `registry_cli_pushes_and_pulls_exact_admitted_layout_into_fresh_state`
- `registry_push_requires_credentials_and_reuses_blobs_after_interruption`
- `registry_pull_rejects_wrong_repository_and_revoked_policy_before_network_outputs`
- `registry_pull_rejects_mutable_image_tag_before_layout_or_admission`
- `registry_pull_rejects_tampered_metadata_blob_before_layout_or_admission`
- `registry_pull_rejects_signature_tag_drift_before_layout_or_admission`
- `registry_pull_rejects_unknown_signer_before_content_closure_or_admission`
- `registry_pull_rejects_bad_signature_bytes_before_content_closure_or_admission`

The ignored test is `registry_cli_interoperates_with_pinned_distribution_and_rejects_wrong_signature_digest`.
Its unchanged reason is "requires the repository-pinned independent OCI
Distribution registry binary".

Smoke with `bin/mantle-isolated` against the `da00f5842` binary was also
identical in every scenario: generate 14, validate/readme 18, import 6,
oci-export/oci-import 5, and push/pull refusals 6.

### `artifact export` (isolated rail `isolated-export`, pueue 265)

`artifact export` was the last unmigrated action here. A single `write-files`
effect could not describe it truthfully, for two reasons:

- In store-backed mode it copies the stored artifact into `--out` before its
  pure export validation, then writes the receipt after that validation.
- `plan_effects` keys effects by their kind name.

The HEAD smoke observed this ordering directly. With an attestation digest
that differs from the ref digest, the `da00f5842` binary wrote
`outs/other-digest/exported.txt` and then reported
`frontend-artifact-export-digest-mismatch` with exit 1 and no receipt.

Contract additions in `artifact.rs`:

- `artifact_export_effect_plan(source, receipt_requested)` builds `Effect`
  values with their own identities:
  - `read-attestation` (`ReadFiles`);
  - `resolve-content`, which is `ReadFiles` for a `--materialized-path` probe
    and `WriteFiles` for the store copy;
  - `write-receipt` (`WriteFiles`), only when `--receipt-out` is given.
- The source is fixed from the request before any effect runs.
- `ArtifactExportPort` has three methods: `read_attestation`,
  `resolve_content`, and `write_receipt`.
- `classify_artifact_export` maps each `ArtifactExportProgress` stop to its
  observations:
  - `AttestationUnavailable` and `ContentFailed` fail the call that ran;
  - `RejectedBeforeContent` skips the content resolution;
  - `ContentInadmissible`, set when the export decision rejects the content
    that was resolved, fails the content effect with the first diagnostic
    code;
  - `ReceiptFailed` fails the receipt write;
  - a planned receipt write after any stop is skipped.

A rejected export therefore never completes, even when no receipt was
planned.

In `src/artifact_cmd.rs`, `cmd_artifact_export` builds the plan before the
attestation read. The adapter `ArtifactExportFiles` holds only the declared
attestation, content, output, and receipt paths. Admission keeps the
`da00f5842` order:

1. the attestation read;
2. the provenance parse;
3. the preflight;
4. the storage-ref check;
5. the content-source check (`artifact export requires --materialized-path or
   storage-backed --out`).

Only after all five does the content resolution run. Each stop is classified
before anything is printed. Failed reports keep `RunError::Reported(1)` in JSON
mode and the `Build` diagnostics error in human mode, and port failures keep
their exact internal messages.

The removed helpers `resolve_export_content`, `export_content_if_available`,
and `emit_export_report` have no remaining callers. Their logic now lives in
the adapter and in `emit_exported_report`.

Isolated rail on HEAD plus the six owned files: `artifact_cmd.rs` is
`61656149…`, contract `artifact.rs` is `ec065449…`, and `lib.rs` is
`44c22178…`; the other three are unchanged. It ended with `rail-exit=0` and
`worktree-removed=0`. The worktree was restored and then removed without
`--force`.

| Leg | Result |
| --- | --- |
| contract unit | `ok. 69 passed; 0 failed`, adding `a_stopped_export_never_completes_and_skips_every_later_effect` |
| build | `exit=0`; `bin/mantle-isolated-export` has sha256 `ad5f5b6f…` |
| `nix_free_demo_cmd::` unit | `ok. 4 passed; 0 failed` |
| `artifact_cmd::` unit | `ok. 22 passed; 0 failed` |
| `nix_free_demo_cli` | `ok. 12 passed; 0 failed` |
| `kernel_bundle_oci_cli` | `ok. 4 passed; 0 failed` |
| `kernel_bundle_oci_registry_cli` | `ok. 10 passed; 0 failed; 1 ignored` (same reason as above) |
| architecture | `cli architecture: PASS` |

The 22 `artifact_cmd::` unit tests include the 10 `da00f5842` export tests,
which run the migrated path, and 3 new ones:

- `artifact_export_with_an_unreadable_attestation_resolves_no_content`: the
  error starts `reading <path>: `, and there is no `--out` copy and no receipt.
- `artifact_export_receipt_write_failure_keeps_the_store_copy_and_writes_no_receipt`.
- `artifact_export_of_inadmissible_store_content_keeps_the_copy_and_reports_exit_one`:
  the result is `Reported(1)`, the copy keeps the stored bytes, and there is
  no receipt.

Real-binary smoke `scratch/mantle-i6/smoke-artifact-export.sh` compared the
`da00f5842` binary with `mantle-isolated-export`. All 14 scenarios were
identical in stdout, stderr, exit code, and output tree with per-file sha256:

| Scenario | Exit |
| --- | --- |
| `store-ok-json` (copy and receipt), `store-ok-human`, `legacy-ok-human` | 0 |
| `hidden-fallback-json`, `hidden-fallback-human` (rejected before the copy, so no copy exists) | 1 |
| `store-missing-content-json`, `legacy-missing-content-json`, `legacy-missing-content-human` | 1 |
| `attestation-digest-differs-json` (the copy exists, with no receipt) | 1 |
| `attestation-missing`, `attestation-malformed`, `bad-provenance`, `neither-content-source` | 3 |
| `store-receipt-under-file` (the copy exists, with no receipt) | 3 |

The other five smokes, run again with `mantle-isolated-export`, were also all
identical: generate 14, validate/readme 18, import 6, OCI layout 5, and
registry refusals 6.

Real-binary smoke with `bin/mantle-final` against the `da00f5842` binary was
identical in every scenario: generate 14 of 14, validate/readme 18 of 18,
import 6 of 6, oci-export/oci-import 5 of 5, and push/pull input refusals 6 of
6. The push/pull refusal scenarios were: a taken receipt (push and pull,
checked before the trust policy), a trust policy that is not `.ncl`, and a
missing `.ncl` policy. Each exits 3 with the `da00f5842` message and leaves
the receipt bytes unchanged.

## Non-claims

- I6 stays unchecked. This record did not edit `tasks.md`, and nothing was
  archived or committed.
- Lifecycle ordering. When this record was written, I7 was checked with
  `[after:I6]` while I6 was open, and `cairn validate` reported
  `task_ordering.progress.unsatisfied` for I7. This record changed no
  checkbox and no dependency annotation.
  - Later, another edit re-annotated I7 as `[after:I5]`. That edit was not
    made by this record; `tasks.md` mtime is 2026-10-01 00:05 -0400. Its
    note says the checker shipped during I6 migration and that I6 remains
    open.
  - Cairn builds its ordering graph only from `[task:]` and `[after:]`
    markers (`cairn-core` `validation/task_ordering.rs`). `[serial]` and
    `[parallel]` create no edges.
  - Re-checked on 2026-10-01 against `tasks.md` sha256 `e4e29b65…`. The
    command `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn --
    validate --root .` exited 0 with `issues: []`, under policy
    `mantle-default` with 40 changes (receipt `a0efb932…`).
  - `cairn gate tasks thin-cli-composition-root --root .` returned `PASS`
    with a valid ordering graph (receipt `9c51513b…`): I6 is todo and ready,
    I7 is done after I5, and V1 is todo and ready.
  - No task now declares I6 as a predecessor. V1–V5 are therefore not
    ordered after I6, and only the unchecked I6 itself blocks archive.
- This record does not claim that any command root outside `nix-free-demo`
  and `artifact` plans before its effects or runs through a port.
- The full-suite results do not claim a frozen worktree. Other workers edited
  `src/main.rs`, `src/remote_build/`, `crates/crunch-build`,
  `crates/crunch-store`, and `crates/crunch-project-core` during these rails.
  Coherence is claimed only for the owned files by hash.
- Parity is claimed only for the scenarios exercised in this record.
