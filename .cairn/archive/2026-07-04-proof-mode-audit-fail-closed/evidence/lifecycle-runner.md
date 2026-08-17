# Lifecycle evidence: proof-mode-audit-fail-closed

Date: 2026-07-04
Change: `proof-mode-audit-fail-closed`
Requirement: `verification_evidence.proof_mode_hermeticity_audit_gate`
Archive path: `cairn/archive/2026-07-04-proof-mode-audit-fail-closed`

## Baseline

Command:

```text
nix develop -c cargo test -p crunch-release-core
```

Output:

```text
test source_archive::tests::submodules_are_rejected ... ok
test source_archive::tests::symlink_targets_must_stay_relative ... ok
test source_archive::tests::unsafe_paths_are_rejected ... ok
test source_archive::tests::unordered_entries_produce_stable_member_order ... ok
test nix_witness::tests::canonicalization_preserves_flag_order_and_sorts_artifact_sets ... ok
test manifest::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok

test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_release_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Focused implementation validation

Command:

```text
nix develop -c cargo test -p crunch-release-core
```

Output:

```text
test source_archive::tests::source_paths_with_target_named_components_are_preserved ... ok
test source_archive::tests::private_runtime_and_lifecycle_paths_are_excluded ... ok
test source_archive::tests::unordered_entries_produce_stable_member_order ... ok
test source_archive::tests::submodules_are_rejected ... ok
test source_archive::tests::symlink_targets_must_stay_relative ... ok
test source_archive::tests::unsafe_paths_are_rejected ... ok

test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_release_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command:

```text
nix develop -c cargo test -p mantle --bin mantle provider_fixed_point_verifier -- --test-threads=1
```

Output:

```text
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.88s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-1e92931af0c4e32a)

running 7 tests
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_informational_proof_audit_events ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_accepts_valid_bounded_bundle ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rebases_copied_bundle_stage_paths ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_bounded_non_claims ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_missing_enforced_closure ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_stage_digest_mismatch ... ok
test cargo_free_self_build::tests::provider_fixed_point_verifier_rejects_unapproved_proof_audit_event ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.00s
```

Command:

```text
nix develop -c cargo test -p mantle --bin mantle cargo_free_self_build:: -- --test-threads=1
```

Output:

```text
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_target_prefixed_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_member_name_conflict ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_unsafe_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_env_omits_ambient_path_entries ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test cargo_free_self_build::tests::rustc_wrapper_script_strips_link_self_contained_runtime_args ... ok
test cargo_free_self_build::tests::safe_path_component_replaces_unsafe_path_bytes ... ok
test cargo_free_self_build::tests::self_build_non_claims_omit_closure_non_claim_when_provider_supplies_claim ... ok

test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 0.08s
```

Command:

```text
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
```

Output:

```text
operator proof guide drift check passed
```

## Cairn pre-archive checks

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
for gate in proposal design tasks; do
  nix run path:/home/brittonr/git/cairn#cairn -- gate "$gate" proof-mode-audit-fail-closed --root /home/brittonr/git/mantle
done
```

Observed output ended with the tasks gate PASS receipt:

```text
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1e8cdca607c024688466be1f6be4c5b3e34f0e4698b868a3b53eb208ba8d4bfb",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "959d512fa7aef078e182e4a6e8b103aa318829feee49954c9e29ec85e10e8c04",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync proof-mode-audit-fail-closed --root /home/brittonr/git/mantle
```

Output:

```text
  "change": "proof-mode-audit-fail-closed",
  "delta_specs": [
    "/home/brittonr/git/mantle/cairn/changes/proof-mode-audit-fail-closed/specs/verification-evidence/spec.md"
  ],
  "dry_run": true,
  "input_hash": "8d880a4a131e02dfb18ab1cefc4413e99ae499a8a451d6948c7f5a882a4ce221",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "c45a48bb7e273f08a8b27bb588197174434ad5097ac69737a8b4d80d6b4db676",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "282edd0852e011fcbe63bcf948e93f4d6fd1a5d99693c272ff48094959e910f5"
}
```

## Archive execution note

The repo lifecycle runner was executed with `--archive-date $(date +%F)` after a clean dry-run. It completed sync/archive mutation but failed while trying to write to the active change evidence path after the active change had already been moved:

```text
error: write ./cairn/changes/proof-mode-audit-fail-closed/evidence/lifecycle-runner.md: No such file or directory (os error 2)
```

This transcript was therefore written manually under the archived change after verifying the archive location and accepted spec sync.

## Accepted spec sync check

Command:

```text
rg -n "proof_mode_hermeticity_audit_gate|Proof-mode hermeticity audit gate" cairn/specs/verification-evidence/spec.md
```

Output:

```text
cairn/specs/verification-evidence/spec.md:1112:### Requirement: Proof-mode hermeticity audit gate
cairn/specs/verification-evidence/spec.md:1114:r[verification_evidence.proof_mode_hermeticity_audit_gate] Mantle proof-mode evidence MUST evaluate typed hermeticity audit events against an explicit closed-by-default policy, fail admission on unapproved events, and report any approved downgrade without satisfying stricter proof claims.
```

## Post-archive validation

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
```

Output:

```json
{
  "change_issues": [],
  "changes": 14,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 30,
  "valid": true
}
```

## Final same-run status

Command:

```text
git status --short --branch
```

Output:

```text
## main...origin/main [ahead 23]
 M cairn/specs/verification-evidence/spec.md
 M crates/crunch-release-core/src/determinism.rs
 M crates/crunch-release-core/src/lib.rs
 M docs/operator-proof-guide.md
 M src/cargo_free_self_build.rs
?? cairn/archive/2026-07-04-proof-mode-audit-fail-closed/
?? cairn/changes/
?? crates/crunch-release-core/src/proof_audit.rs
```

## Final post-transcript validation

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
```

Output:

```json
{
  "change_issues": [],
  "changes": 14,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 30,
  "valid": true
}
```

Command:

```text
git status --short --branch
```

Output:

```text
## main...origin/main [ahead 23]
 M cairn/specs/verification-evidence/spec.md
 M crates/crunch-release-core/src/determinism.rs
 M crates/crunch-release-core/src/lib.rs
 M docs/operator-proof-guide.md
 M src/cargo_free_self_build.rs
?? cairn/archive/2026-07-04-proof-mode-audit-fail-closed/
?? cairn/changes/
?? crates/crunch-release-core/src/proof_audit.rs
```
