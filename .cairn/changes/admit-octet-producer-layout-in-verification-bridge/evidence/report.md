# Verification bridge producer-layout evidence

Change: `admit-octet-producer-layout-in-verification-bridge`
Base: `b99ed8637` (head of `raise-foreign-import-graph-bounds`).
Worktree: `/home/brittonr/wt/mantle-bridge`, branch
`admit-octet-producer-layout-in-verification-bridge`.

All commands ran from the worktree root inside the checked-in dev shell:

```sh
env RUSTC_WRAPPER= CARGO_TARGET_DIR=target nix --option min-free 0 develop -c <command>
```

## Focused tests

`focused-tests.txt` — `cargo test -p crunch-verification-bridge-core -p crunch-verification-bridge`, exit 0.

Core unit tests (13):

| Test | Result |
| --- | --- |
| `producer::tests::versioned_tool_graph_with_basenames_is_admitted` | ok |
| `producer::tests::legacy_tool_graph_with_store_paths_is_admitted` | ok |
| `producer::tests::versioned_node_carrying_legacy_inputs_fails_closed` | ok |
| `producer::tests::unsupported_top_level_graph_version_fails_closed` | ok |
| `producer::tests::versioned_root_absent_fails_closed` | ok |
| `producer::tests::versioned_dependency_outside_the_closure_fails_closed` | ok |
| `producer::tests::versioned_dynamic_output_selection_fails_closed` | ok |
| `producer::tests::legacy_node_carrying_versioned_keys_fails_closed` | ok |
| `provider_profile_tests::rail_named_provider_profile_locator_is_selected` | ok |
| `provider_profile_tests::absent_provider_profile_row_fails_closed` | ok |
| `provider_profile_tests::duplicate_provider_profile_rows_fail_closed` | ok |
| `provider_profile_tests::provider_profile_digest_mismatch_fails_closed` | ok |
| `provider_profile_tests::escaping_provider_profile_locator_fails_closed` | ok |

Shell integration tests (8, all ok):

| Test | Result |
| --- | --- |
| `real_owner_handoffs_replay_with_exact_receipts_without_promoting_authority` | ok (recorded fixture still passes) |
| `rail_named_provider_profile_layout_is_admitted_from_the_receipt` | ok (real-layout copy) |
| `provider_profile_row_must_be_unique_and_match_the_provider_identity` | ok (missing/duplicate/mismatch negatives) |
| `stale_source_profile_tool_and_options_cannot_reuse_association` | ok |
| `missing_tampered_and_escaped_payloads_fail_without_publication` | ok |
| `external_failure_and_unknown_versions_do_not_become_proof_acceptance` | ok |
| `symbolic_link_substitution_is_not_a_payload` | ok |
| `selected_fixture_cannot_stand_in_for_a_complete_owner_verification_bundle` | ok |

## Static gates

| Command | Transcript | Exit |
| --- | --- | --- |
| `cargo clippy -p crunch-verification-bridge-core -p crunch-verification-bridge --all-targets --no-deps -- -D warnings` | `clippy.txt` | 0 |
| `cargo fmt --check -p crunch-verification-bridge-core` | `fmt.txt` | 0 |
| `cargo fmt --check -p crunch-verification-bridge` | `fmt.txt` | 0 |
| `./scripts/check-no-std-core.sh` | `check-no-std-core.txt` | see below |

`./scripts/check-no-std-core.sh` step 12 fails at `origin/main` for files this
work does not touch; the failure is pre-existing and is recorded, not fixed.
`no-std-origin-main-step12.txt` is the same ownership check run in a detached
worktree of `origin/main` (`cd4ad3f43`), which fails on the same classification
entries.

## Cairn gates

`cairn-gates.txt`:

| Command | Exit | Verdict |
| --- | --- | --- |
| `cairn validate --root .` | 0 | no issues/findings |
| `cairn gate proposal admit-octet-producer-layout-in-verification-bridge --root .` | 0 | PASS |
| `cairn gate design admit-octet-producer-layout-in-verification-bridge --root .` | 0 | PASS |
| `cairn gate tasks admit-octet-producer-layout-in-verification-bridge --root .` | 0 | PASS |

## Real producer-output run

`real-run.txt` records the command and exit code 0;
`real-run-mantle-observation.json` is the exact `mantle-observation.json` the
run wrote (the temporary handoff directory was then deleted).

```
target/release/mantle-verification-bridge octet-job \
  /datapool/mantle-store/convergence-pilot-20261119/store/zjq6whhf2c641vkz89kgwwpbajjq5cbq-octet-mantle-aeneas-consumer-job-v1 \
  /home/brittonr/wt/convergence-evidence/realization/producer-export \
  d5b34c2d64b1227417a46df8ed8427f2d10ed3c8 /tmp/bridge-layout-check-handoff
```

Exit code: 0.

Identities in the observation:

| Field | Value |
| --- | --- |
| `proof_acceptance` | `externally-reported-by-octet` |
| `receipt_identity` | `b3:84d2e77717013a203a4daf6ce649de8e501c7386f142b7abc66f32d08c33420e` |
| `producer_job_identity` | `b3:a14ec403025db0330379bad1e1904fae23835ccd1ab95017a80e3e5ca715b318` |
| `tool_graph_identity` | `b3:d0ebfcbae2f8b44866a45df068cec42cf5be5b6379ff72cb91bddbe919697808` |
| `external_report_identity` | `b3:bfc3e95c55fae9aad7099183d2a2325319b26c37998c7267d158b86b545ab444` |
| `association_identity` | `b3:9290b4ea5319a7255026877505613eefe70e575406fd858d5c805228efd5873f` |

The run read `/home/brittonr/wt/convergence-evidence/realization/producer-export`
and the pilot store path; it wrote only `/tmp/bridge-layout-check-handoff`.
No write occurred under the pilot realization directories or the pilot store.
Mantle gained no proof authority: the observation states external acceptance by
Octet and the checked byte links only.
