# Pre-existing operator contract drift on main and its standalone repair (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins-baseline/flagdrift-20261008T004239Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-flagdrift.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T00:42:39Z free=1299272896512 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline head=e24bbbc2f803f59872e2e59370bd8ce0829c918e CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins-baseline CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-flagdrift.sh
phase=running launcher=752813 pgid=752813
phase=child_exited exit=0 elapsed_s=346 free=1217064484864
phase=final exit=0 group_survivors=0 end=2026-10-08T00:48:25Z elapsed_s=346 free=1216998715392
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline' is dirty
head=e24bbbc2f803f59872e2e59370bd8ce0829c918e
+ nickel typecheck config/operator-surfaces.ncl
ncl_typecheck_exit=0 ncl_typecheck_wall_s=0
+ bash -c nickel export --format json config/operator-surfaces.ncl > config/operator-surfaces.json
ncl_export_surfaces_exit=0 ncl_export_surfaces_wall_s=0
+ cargo build -p mantle --bin mantle
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 19s
build_mantle_exit=0 build_mantle_wall_s=80
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/mantle __operator-contract --mode raw-descriptors > config/operator-command-descriptors.json
gen_descriptors_exit=0 gen_descriptors_wall_s=1
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/mantle __operator-contract --mode catalog > config/operator-command-catalog.json
gen_catalog_exit=0 gen_catalog_wall_s=0
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/mantle __operator-contract --mode reference > docs/generated/operator-command-reference.md
gen_reference_exit=0 gen_reference_wall_s=1
+ bash -c /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/mantle __operator-contract --mode workflow > docs/generated/canonical-operator-workflow.md
gen_workflow_exit=0 gen_workflow_wall_s=0
 M config/operator-command-catalog.json
 M config/operator-command-descriptors.json
 M config/operator-surfaces.json
 M config/operator-surfaces.ncl
 M docs/generated/operator-command-reference.md
 config/operator-command-catalog.json         | 4 +++-
 config/operator-command-descriptors.json     | 1 +
 config/operator-surfaces.json                | 1 +
 config/operator-surfaces.ncl                 | 2 +-
 docs/generated/operator-command-reference.md | 2 +-
 5 files changed, 7 insertions(+), 3 deletions(-)
+ ./scripts/check-operator-command-contract.sh
operator command contract generator self-test: PASS
operator command contract: PASS (commands=176)
operator_contract_exit=0 operator_contract_wall_s=9
+ cargo test -p mantle --test operator_diagnostics
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins-baseline/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 04s
     Running tests/operator_diagnostics.rs (/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins-baseline/debug/deps/operator_diagnostics-44b0508e7b7f3ff7)

running 18 tests
test operator_contract_generator_runs_positive_and_negative_self_test ... ok
test doctor_json_failure_reports_missing_prerequisite ... ok
test doctor_default_profile_is_build_and_does_not_mutate_paths ... ok
test operator_contract_exports_sorted_public_clap_paths ... ok
test doctor_json_output_reports_selected_profile ... ok
test doctor_explicit_self_build_profile_checks_nightly_visibility ... ok
test operator_contract_checked_files_match_clap_and_policy ... ok
test doctor_failure_names_missing_prerequisite_and_profile ... ok
test build_json_preflight_failure_omits_saved_log_path ... ok
test build_plan_reports_preflight_error_for_missing_store_dir ... ok
test build_plan_json_reports_action_schema ... ok
test build_plan_reports_build_for_uncached_root ... ok
test build_json_failure_envelope_includes_saved_log_path ... ok
test build_human_failure_reports_log_persistence_failure_without_saved_log_path ... ok
test build_json_success_reports_log_persistence_failure_without_fake_log_file ... ok
test build_without_plan_still_executes_fetchurl ... ok
test build_plan_reports_cached_after_fetchurl_build ... ok
test build_human_failure_summary_matches_json_facts ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.84s

operator_diag_exit=0 operator_diag_wall_s=70
flagdrift_done
```
