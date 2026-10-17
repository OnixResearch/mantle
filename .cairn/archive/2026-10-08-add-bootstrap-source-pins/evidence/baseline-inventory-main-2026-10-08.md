# T1.1 inventory and refresh-surface baseline on published main e24bbbc2 (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins-baseline/inventory-20261008T004125Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-inventory.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T00:41:25Z free=1322484404224 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline head=e24bbbc2f803f59872e2e59370bd8ce0829c918e CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins-baseline CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-inventory.sh
phase=running launcher=718574 pgid=718574
phase=child_exited exit=0 elapsed_s=109 free=1285176119296
phase=final exit=0 group_survivors=0 end=2026-10-08T00:43:16Z elapsed_s=111 free=1285088018432
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
head=e24bbbc2f803f59872e2e59370bd8ce0829c918e
root_recipes=185
url_literal_files=131 url_literal_occurrences=158
hash_literal_files=131 hash_literal_occurrences=150
exists crates/crunch-project-core/src/refresh.rs: yes
exists crates/crunch-project/src/refresh_adapter.rs: yes
exists src/project_resolve.rs: yes
exists .cairn/specs/mantlepkgs-update-plans/spec.md: yes
79:impl RefreshResolver for LiveResolver {
88:    fn hash_url_content(
+ cargo test -p crunch-project-core --lib refresh -- --nocapture
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/deps/crunch_project_core-5f776a7d4a366768)

running 19 tests
test merge::tests::inputs_needing_refresh_skips_frozen ... ok
test refresh::tests::plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work ... ok
test refresh::tests::refresh_inputs_marks_frozen_without_resolution ... ok
test refresh::tests::refresh_build_fetch_policy_locks_expected_hash_without_resolution ... ok
test refresh::tests::refresh_imported_source_policy_fails_without_source_state ... ok
test merge::tests::inputs_needing_refresh_includes_missing ... ok
test refresh::tests::refresh_inputs_marks_matching_entry_unchanged ... ok
test refresh::tests::refresh_imported_source_policy_requires_ready_source_state ... ok
test refresh::tests::freshness_unchanged_decision_skips_resolution_work ... ok
test refresh::tests::refresh_inputs_marks_resolved_entry_updated ... ok
test refresh::tests::freshness_network_required_decision_fails_closed_for_refresh ... ok
test refresh::tests::apply_outcomes_removes_orphaned_patches ... ok
test refresh::tests::list_stale_separates_changed_and_failed_inputs ... ok
test refresh::tests::apply_outcomes_updates_lock_and_resolves_patches ... ok
test refresh::tests::plan_refresh_inputs_filters_selected_inputs ... ok
test refresh::tests::apply_outcomes_reverts_input_when_patch_resolution_fails ... ok
test refresh::tests::refresh_inputs_rejects_trusted_input_without_evidence ... ok
test refresh::tests::apply_outcomes_rejects_trusted_patch_without_evidence ... ok
test refresh::tests::refresh_inputs_attaches_verified_trust_to_updated_lock ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s

core_refresh_exit=0 core_refresh_wall_s=3
+ cargo test -p crunch-project --lib refresh -- --nocapture
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 33s
     Running unittests src/lib.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/deps/crunch_project-23a69231edac4543)

running 6 tests
test refresh_adapter::tests::failed_freshness_observation_maps_empty_resolver_diagnostic ... ok
test refresh_adapter::tests::adapter_preserves_failed_resolution_behavior ... ok
test refresh_adapter::tests::adapter_resolves_vcs_identities_and_tree_hashes ... ok
test refresh_adapter::tests::default_vcs_hooks_fail_closed_with_unsupported_tool_diagnostic ... ok
test refresh_adapter::tests::identity_validation_preserves_missing_and_empty_behavior ... ok
test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.03s

project_refresh_exit=0 project_refresh_wall_s=100
```
