# Mantle trellis-job bridge evidence

Change: `add-trellis-job-bridge-mode` (WP8.9, design D-E).
Base: `ead7b8dac` (`origin/main`).
Worktree: `/home/brittonr/wt/mantle-trellis-job`, branch
`add-trellis-job-bridge-mode`.

All commands ran from the worktree root inside the checked-in dev shell:

```sh
env RUSTC_WRAPPER= CARGO_TARGET_DIR=target nix --option min-free 0 --option builders '' develop -c <command>
```

## CLI usage

```sh
mantle-verification-bridge trellis-job OWNER_OUTPUT PRODUCER_EXPORT SOURCE_REVISION OUTPUT
```

- `OWNER_OUTPUT`: the Trellis producer output root (`profile.json`, the
  receipt at `verus.receipt_path`, `producer-report.json`, `proof-cards.json`,
  `evidence.json`, `source/`, `artifacts/`).
- `PRODUCER_EXPORT`: the acquisition-only export (`job.json`,
  `tool-closure.derivation-json.json`, `root-derivation`,
  `flake-metadata.json`, `flake.lock`).
- `SOURCE_REVISION`: the immutable 40-hex revision the profile/export bind.
- `OUTPUT`: the handoff destination, or `-` to inspect without exporting.

Existing modes are unchanged: `<octet|octet-verified|trellis> ROOT PROFILE
RECEIPT SOURCE_REVISION OUTPUT` and `octet-job OWNER_OUTPUT PRODUCER_EXPORT
SOURCE_REVISION OUTPUT`.

## Test list

`crates/crunch-verification-bridge/tests/trellis_job.rs` (19 tests) builds the
producer output contract in temporary directories:

- `synthetic_trellis_job_observation_binds_exact_bytes` (positive)
- `exported_trellis_job_handoff_replays_from_its_own_output` (positive)
- `mutated_profile_source_identity_fails_closed`
- `failed_receipt_status_fails_closed`
- `mutated_receipt_source_binding_fails_closed`
- `mutated_receipt_artifact_row_fails_closed`
- `unsupported_trellis_job_profile_fails_closed`
- `unsupported_verus_receipt_fails_closed`
- `unsupported_producer_report_fails_closed`
- `wrong_job_schema_fails_closed`
- `wrong_job_owner_fails_closed`
- `wrong_rail_fails_closed`
- `network_allowed_execution_fails_closed`
- `graph_missing_root_fails_closed`
- `incomplete_tool_graph_fails_closed`
- `revision_mismatch_fails_closed`
- `tampered_artifact_bytes_fail_closed`
- `missing_producer_report_fails_closed`
- `missing_evidence_document_fails_closed`

The existing `boundaries.rs` (8 tests) and core unit tests (13 tests) run in the
same command and are unchanged.

## Focused tests

`cargo test -p crunch-verification-bridge-core -p crunch-verification-bridge`
(`focused-tests.txt`): exit 0. boundaries 8 passed, trellis_job 19 passed,
core unit 13 passed.

## Clippy and fmt

- `cargo clippy -p crunch-verification-bridge-core -p crunch-verification-bridge
  --all-targets --no-deps -- -D warnings` (`clippy.txt`): exit 0, no warnings.
- `cargo fmt --check -p crunch-verification-bridge-core` and
  `cargo fmt --check -p crunch-verification-bridge` (`fmt.txt`): exit 0.

## no-std core

`./scripts/check-no-std-core.sh` (`check-no-std-core.txt`): steps 1-11 pass;
step 12 (`check-no-std-core-ownership.sh`) fails with 13 missing touched-std
classification entries in `openspec/specs/functional-core/evidence/ownership-review.md`.

Baseline (`no-std-origin-main-step12.txt`): the same script at a pristine
`origin/main` worktree (`git worktree add --detach /tmp/mantle-origin-main-baseline
origin/main`) exits 1 with the same 13 entries. The failure predates this
change and is not fixed here.

## Cairn

`cairn validate --root .` exits 0 with `valid: true` and no issues; the
proposal, design, and tasks gates each report `verdict: PASS` with no issues
(`cairn-gates.txt`).

## Scope

The synthetic fixture is not the real Terrane producer run and no Verus proof
executed here. Mantle still grants no theorem admission:
`proof_acceptance` is `not-evaluated-by-mantle`. The realized producer output,
its Mantle realization, and the Cairn `current` row are WP8 stage 2. The change
is not archived in this stage.
