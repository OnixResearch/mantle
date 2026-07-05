# Drain Session Evidence — 2026-07-05

## Committed Implementation

19 files committed (1242 insertions, 81 deletions) covering:
- `src/cache_substitution.rs` — pure model: CacheCandidate, CacheAdmissionReason, CacheAdmissionEvent, sanitization, parsing, trust-conflict detection
- `src/build_plan.rs` — cache admission in plan entries
- `src/build_report.rs` — BuildJsonCacheAdmission, admission diagnostics
- `src/main.rs` — CLI threading: substituter_urls Vec, plan/build dispatch
- `crates/crunch-store/src/completeness.rs` — recursive castore completeness + global markers
- `crates/crunch-store/src/metadata_cache.rs` — AdvisoryMetadataCache persistence, validity checks, refresh policy
- `crates/crunch-store/src/handle.rs` — castore_has_complete_content, remote_cache_urls plural
- `crates/crunch-store/src/lib.rs` — module + pub use exports
- `src/attest_cmd.rs`, `src/build_cmd.rs`, `src/remote_build.rs`, `src/self_build.rs`, `src/store_cmd.rs` — renames/wiring
- `tests/*.rs` — remote_cache_url → remote_cache_urls in test store configs

## Test Results

### cache_substitution (39 tests)
```
test cache_substitution::tests::parse_orders_candidates_by_configuration_order ... ok
test cache_substitution::tests::parse_deduplicates_same_identity_same_trust ... ok
test cache_substitution::tests::parse_rejects_malformed_url ... ok
test cache_substitution::tests::parse_rejects_too_many_candidates ... ok
test cache_substitution::tests::parse_rejects_invalid_store_prefix ... ok
test cache_substitution::tests::parse_rejects_invalid_trust_policy_digest ... ok
test cache_substitution::tests::parse_marks_https_url_as_requiring_network ... ok
test cache_substitution::tests::parse_marks_file_url_as_not_requiring_network ... ok
test cache_substitution::tests::trust_conflict_detected_for_same_identity_different_digest ... ok
test cache_substitution::tests::no_trust_conflict_for_different_identity ... ok
test cache_substitution::tests::no_trust_conflict_for_same_identity_same_digest ... ok
test cache_substitution::tests::sanitize_strips_query_fragment_and_userinfo ... ok
test cache_substitution::tests::sanitize_preserves_port ... ok
test cache_substitution::tests::sanitize_strips_trailing_slash_root ... ok
test cache_substitution::tests::sanitize_rejects_missing_scheme ... ok
test cache_substitution::tests::sanitize_rejects_missing_host_and_scheme ... ok
test cache_substitution::tests::split_substituter_urls_splits_commas_and_trims ... ok
test cache_substitution::tests::split_substituter_urls_skips_empty_entries ... ok
test cache_substitution::tests::split_substituter_urls_returns_single ... ok
test cache_substitution::tests::split_substituter_urls_returns_empty_for_empty_input ... ok
test cache_substitution::tests::admission_reason_as_str_is_stable ... ok
test cache_substitution::tests::admission_reason_is_hit_classifies_local_and_remote ... ok
test cache_substitution::tests::admission_event_serializes_stable_reason_code ... ok
test cache_substitution::tests::metadata_class_as_str_is_stable ... ok
test cache_substitution::tests::metadata_schema_version_current_is_one ... ok
test cache_substitution::tests::build_metadata_cache_key_includes_all_dimensions ... ok
test cache_substitution::tests::metadata_ttl_negative_miss_is_shorter ... ok
test cache_substitution::tests::new_metadata_entry_sets_expiry_from_ttl ... ok
test cache_substitution::tests::check_metadata_validity_returns_fresh_for_matching_entry ... ok
test cache_substitution::tests::check_metadata_validity_expired_when_past_expiry ... ok
test cache_substitution::tests::check_metadata_validity_force_refresh_bypasses_fresh_entry ... ok
test cache_substitution::tests::check_metadata_validity_detects_schema_mismatch ... ok
test cache_substitution::tests::check_metadata_validity_detects_trust_policy_mismatch ... ok
test cache_substitution::tests::check_metadata_validity_detects_store_prefix_mismatch ... ok
test cache_substitution::tests::check_metadata_validity_detects_identity_mismatch ... ok
test cache_substitution::tests::check_metadata_validity_detects_output_digest_mismatch ... ok
test cache_substitution::tests::admission_summary_from_reason_does_not_claim_metadata_reuse ... ok
test cache_substitution::tests::admission_summary_with_reused_metadata_records_validity ... ok
test cache_substitution::tests::metadata_validity_as_str_is_stable ... ok
```
All pass (39/39).

### completeness (8 tests)
All pass — recursive DFS, marker store, depth bounds, missing children.

### metadata_cache persistence (8 tests)
All pass — put/get/remove, eviction, save/reload, force-refresh, corrupt file.

## Cairn Validation

```
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
valid: true, changes: 2, specs_validated: 18

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal cache-substitution-reuse-diagnostics --root .  verdict: PASS
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design cache-substitution-reuse-diagnostics --root .     verdict: PASS
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks cache-substitution-reuse-diagnostics --root .     verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal production-remote-build-farm --root .  verdict: PASS
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design production-remote-build-farm --root .     verdict: PASS
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks production-remote-build-farm --root .     verdict: PASS
```

## Remaining Work

### cache-substitution-reuse-diagnostics
- [ ] V1 — priority-ordered multi-cache selection (needs store-layer multi-URL iteration + integration test)
- [ ] V2 — repeated build--plan metadata reuse (needs build-plan-level integration test)
- [ ] V3 — cached metadata substitution + final verification (needs store-level integration)
- [ ] V4 — negative: untrusted signatures, prefix mismatches, fixed-output hits (some model tests exist, need store integration)
- [ ] V5 — castore-incomplete rejection at store level (model-level tests exist)
- [ ] V6 — stale/schema-mismatch metadata handling (model-level tests exist)

### production-remote-build-farm
- All I1-I7 implementation tasks unchecked
- All V1-V13 verification tasks unchecked
- Massive scope — no work started