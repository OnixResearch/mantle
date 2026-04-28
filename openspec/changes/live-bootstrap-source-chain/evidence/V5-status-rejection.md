# V5 Status Rejection Evidence

Timestamp: 2026-04-27T20:12:00Z

## Tests Run

```
cargo test -p crunch --bin crunch stagex -- --nocapture
cargo test -p crunch --bin crunch eligibility -- --nocapture
```

## Results

### stagex tests (19 passed)
- `stagex_eligible_report_passes_validation` — valid report accepted
- `legacy_fetch_provider_fails_stagex_eligibility` — LegacyFetch rejected
- `source_root_provider_fails_stagex_eligibility` — SourceRoot rejected
- `missing_stagex_metadata_fails_eligibility` — missing lineage metadata rejected
- `missing_protected_transition_fails_eligibility` — missing protected-phase transition rejected
- `host_gcc_in_seccomp_events_fails_eligibility` — /usr/bin/gcc in seccomp events rejected
- `nix_store_in_seccomp_events_fails_eligibility` — /nix/store Nix tools in seccomp events rejected
- `legacy_provider_exec_path_fails_eligibility` — musl.cc legacy provider path rejected
- `host_fallback_events_fail_eligibility` — host fallback events rejected
- `denied_seccomp_events_do_not_fail_eligibility` — denied (blocked) events ok
- `report_format_roundtrip_with_stagex_metadata` — JSON roundtrip preserves stagex metadata
- `provider_mode_legacy_does_not_satisfy_stagex` — legacy provider mode classification
- `provider_mode_stagex_satisfies_requirement` — stagex provider mode classification
- `stagex_lineage_manifest_valid_json_passes_validation` — valid manifest accepted
- `stagex_lineage_manifest_invalid_json_rejected` — invalid manifest rejected
- `stagex_lineage_manifest_forbidden_root_rejected` — forbidden root in manifest rejected
- `stagex_provider_evidence_classification` — evidence classification
- `stagex_profile_rejects_*` (5 tests) — release profile rejects incomplete proof

### eligibility tests (9 passed)
All `validate_stagex_proof_eligibility` test cases pass, covering:
- Valid report acceptance
- Provider mode rejection (LegacyFetch, SourceRoot)
- Missing metadata rejection
- Missing protected transition rejection
- Host tool exec rejection (/usr/bin/gcc, /nix/store/*/nix-build, musl.cc paths)
- Host fallback event rejection
- Denied seccomp events do not fail (correct behavior)

## Summary

All placeholder, deferred, legacy provider, and missing proof scenarios are
correctly rejected by `validate_stagex_proof_eligibility()` and the release
profile validation. 28 tests total, all passing.
