# Source-built negative matrix

## Question

Does the current source-built proof code reject every negative case listed in V1?

## Inspected evidence

The deterministic matrix passed. This evidence does not prove V57 success or complete root action trust.

Pueue task `10914` serialized all 62 `cargo_free_self_build::tests::` tests with one test thread. All 62 tests passed.

Pueue task `10928` ran the remaining focused modules with one test thread. Every executed test passed.

The first `self_build::tests::` run exposed a stale fixture. The fixture omitted the required `mantlepkgs/` source entry.

The repaired fixture now creates and checks `mantlepkgs/`. The final `self_build::tests::` filter passed 176 tests.

Pueue task `10941` ran `fresh_clone_fixed_point::tests::` through the library target. All four tests passed.

Pueue task `10952` passed Clippy with `-D warnings`. It used only the three existing named allowances recorded in `clippy.log`.

Pueue task `10956` passed `cargo fmt -p mantle --check` and `git diff --check`.

Pueue task `10957` reached the existing Cairn policy parser blocker. `cairn-validation.log` records the invalid `task_marker_policy.markers` field.

## Matrix coverage

| Required negative case | Deterministic evidence |
|---|---|
| Missing source records or substituted provider inputs | `rejects_missing_required_source_role`, `source_built_fixed_point_profile_rejects_missing_native_manifest_authority`, `source_built_fixed_point_profile_rejects_missing_native_source_closure`, and `source_built_fixed_point_profile_rejects_provider_manifest_substitution` |
| Nonempty initial authority, imported completion, or cache-only completion | `rejects_imported_provider_as_source_authority`, `rejects_nonempty_initial_output_authority`, `provider_cache_rejects_stale_or_mismatched_digests_as_hard_miss`, and `action_documents_reject_cache_only_and_plan_digest_drift` |
| Incomplete adapters, path-only authority, or producerless executables | `root_documents_sum_complete_adapters_and_reject_partial_coverage`, `rust_child_plan_rejects_unknown_producer_and_path_only_authority`, and `eager_plan_rejects_missing_producer_and_unbound_builder` |
| Unknown, missing, denied, remote, duplicate, or excessive events | `rust_child_reconciliation_rejects_denied_unknown_and_missing_events`, `action_reconciliation_rejects_unknown_remote_and_missing_events`, `eager_reconciliation_rejects_unknown_missing_and_duplicate_events`, and `bounded_sum_rejects_count_overflow` |
| Executable, producer, plan, audit, payload, or closure drift | `rust_child_authority_is_deterministic_and_rejects_digest_drift`, `eager_plan_composition_deduplicates_exact_actions_and_rejects_drift`, checkpoint rejection tests, closure relocation rejection tests, and binding substitution tests |
| Undeclared input, Cargo use, live fetch, ambient discovery, or protected-exec violation | Toolchain enforcement rejects undeclared host tools. `child_blocker_fails_when_cargo_guard_was_invoked`, `report_rejects_live_fetch_event`, ambient binding tests, and the seccomp test reject these paths. |
| V1 receipt, wrong provider kind, practical fallback, or stage2 host orchestration | Bootstrap parity rejects the V1 workflow. Source profile and release checks reject wrong kinds. The proof plan rejects fallback and host-owned stage2. |
| Output mismatch, timeout, disk shortfall, or partial alias publication | Release comparison tests reject mismatches. `runtime_bounds_reject_elapsed_timeout_before_disk_probe`, `disk_capacity_accepts_sufficient_space_and_rejects_shortfall`, and alias transaction tests reject the remaining cases. |

## Selector note

`negative-matrix-tests.log` retains one incorrect bin selector for `fresh_clone_fixed_point::tests::`. That selector ran zero tests and is not evidence.

`fresh-clone-fixed-point-tests.log` contains the correct library-target run. It supersedes the zero-test selector with four executed tests.

## Evidence files

- `cargo-free-self-build-tests.log`
- `negative-matrix-tests.log`
- `fresh-clone-fixed-point-tests.log`
- `clippy.log`
- `clippy-unallowlisted-baseline.log`
- `cairn-validation.log`

## Non-claims

This matrix does not replace the promoted proof. It does not close I3, I4, I5, or V2.

The full bootstrap-parity suite retains unrelated stale-evidence failures. This matrix ran only the V1 provider-kind and V2-receipt rejection cases.

## Decision

Mark V1 complete. Keep I3, I4, I5, and V2 open until a promoted proof reports complete trust.

## Owner

Mantle source authority, action planning, proof receipts, and fixed-point publication.

## Next action

Preserve V57 without interruption. If V57 publishes an eligible checkpoint, restore it for the final source generation.
