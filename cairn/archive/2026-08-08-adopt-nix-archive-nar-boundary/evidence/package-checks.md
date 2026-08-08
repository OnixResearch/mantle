# Package checks

Each section contains the exact combined standard output and error from the required command.

## Adapter and store package tests

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-nar -p crunch-store
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_nar-823e82fef187a0c2)

running 6 tests
test tests::cutover_gate_accepts_complete_parity_and_rejects_stale_or_missing_evidence ... ok
test tests::worker_result_preserves_observation_errors_and_maps_worker_failures ... ok
test tests::bounded_encoding_rejects_a_complete_archive_over_the_limit ... ok
test tests::request_rejects_empty_paths_zero_limits_and_unknown_algorithms ... ok
test tests::fact_comparison_accepts_equal_facts_and_rejects_case_hack_drift ... ok
test tests::root_substitution_after_admission_fails_closed ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/parity.rs (/home/brittonr/.cargo-target/debug/deps/parity-8c6cccef0f84a383)

running 8 tests
test generated_case_planning_rejects_the_first_case_over_the_bound ... ok
test retained_fixture_manifest_binds_source_and_payload_identities ... ok
test explicit_case_hack_collision_and_read_failure_are_typed_errors ... ok
test retained_upstream_goldens_match_filesystem_encoding ... ok
test actual_fixture_evidence_satisfies_the_cutover_gate ... ok
test nix_archive_and_snix_match_on_byte_safe_fixture_and_all_recursive_hashes ... ok
test nix_oracle_is_reported_as_matched_or_unavailable ... ok
test generated_tree_parity_is_deterministic_and_bounded ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_store-b14c7d60e8245d3c)

running 338 tests
test action_result::tests::offline_discovery_never_opens_remote_sources ... ok
test action_result::tests::aggregate_candidate_limit_rejects_the_offending_source_response ... ok
test archive::tests::import_action_core_rejects_untrusted_and_skips_present ... ok
test archive::tests::archive_list_rejects_bad_magic ... ok
test archive::tests::archive_list_rejects_header_end_count_mismatch ... ok
test action_result::tests::source_limit_bounds_empty_or_failing_remote_sources ... ok
test archive::tests::pathinfo_fixture_service_is_bounded ... ok
test action_result::tests::duplicate_detached_signatures_are_rejected_before_publication ... ok
test action_result::tests::local_conflicting_existing_record_is_not_overwritten ... ok
test action_result::tests::local_dangling_or_poisoned_index_fails_closed ... ok
test action_result::tests::local_interrupted_publication_is_not_discoverable ... ok
test action_result::tests::gc_retains_metadata_only_while_outputs_are_independently_live ... ok
test attestation::tests::artifact_attestation_file_uses_canonical_bytes ... ok
test attestation::tests::artifact_attestation_round_trips_by_logical_store_path ... ok
test build_io::tests::equal_length_replacement_preserves_binary_shape_and_occurrences ... ok
test action_result::tests::local_publish_is_atomic_no_clobber_and_duplicate_safe ... ok
test build_io::tests::missing_blob_hash_fails_closed ... ok
test build_io::tests::read_file_node_rejects_non_file_and_over_limit_inputs ... ok
test action_result::tests::discovery_only_base_is_readable_but_never_receives_publication ... ok
test ca_mapping::tests::insert_and_get ... ok
test build_io::tests::unequal_length_replacement_is_rejected - should panic ... ok
test ca_mapping::tests::load_missing_file_returns_empty ... ok
test ca_mapping::tests::save_and_load_roundtrip ... ok
test capability::tests::missing_output_does_not_create_a_root ... ok
test capability::tests::output_lookup_and_selected_root_registration_share_exact_identity ... ok
test attestation::tests::runtime_closure_attestation_synthesizes_missing_member_artifact ... ok
test attestation::tests::runtime_closure_attestation_refreshes_after_member_digest_changes ... ok
test closure::tests::cycle_terminates ... ok
test closure::tests::diamond_dedup ... ok
test closure::tests::local_query_error_can_still_use_remote_refs ... ok
test closure::tests::linear_chain ... ok
test attestation::tests::runtime_closure_attestation_is_cached_by_root_selection ... ok
test closure::tests::practical_mode_records_degraded_child_lookup ... ok
test attestation::tests::runtime_closure_attestation_uses_stored_artifact_digests ... ok
test closure::tests::single_path_no_refs ... ok
test closure::tests::practical_mode_records_degraded_root_lookup ... ok
test closure::tests::remote_fallback ... ok
test closure::tests::remote_metadata_lookup_failure_degrades_practical_mode ... ok
test closure::tests::self_reference ... ok
test closure::tests::strict_mode_rejects_missing_root_closure_facts ... ok
test completeness::tests::blob_node_rejects_declared_size_mismatch ... ok
test completeness::tests::blob_node_requires_blob_presence ... ok
test completeness::tests::chunked_blob_metadata_must_match_declared_size ... ok
test completeness::tests::directory_with_missing_blob_child_is_incomplete ... ok
test completeness::tests::completeness_rechecks_and_rejects_removed_directory ... ok
test completeness::tests::symlink_is_always_complete ... ok
test completeness::tests::node_visit_limit_accepts_last_supported_node_and_rejects_overflow ... ok
test completeness::tests::empty_directory_requires_existence ... ok
test completeness::tests::bounded_depth_rejects_extremely_deep_trees ... ok
test archive::tests::ca_path_identity_accepts_reference_aware_standard_path ... ok
test archive::tests::archive_export_list_round_trip_preserves_metadata_before_payload ... ok
test action_result::tests::http_urls_strip_query_fragment_and_preserve_cache_subpath ... ok
test archive::tests::archive_export_rejects_stale_final_nar_facts_before_writing ... ok
test archive::tests::archive_export_refuses_unsigned_without_escape_hatch ... ok
test closure::tests::depth_limit_enforced ... ok
test archive::tests::missing_closure_reference_fails_export_plan ... ok
test archive::tests::export_closure_includes_references_deterministically ... ok
test archive::tests::archive_import_rejects_store_prefix_mismatch_before_persisting ... ok
test build_io::tests::blob_and_nar_hashes_cover_all_supported_algorithms ... ok
test archive::tests::archive_list_drains_non_seekable_payloads_in_bounded_chunks ... ok
test archive::tests::archive_import_rejects_ca_metadata_for_another_store_path ... ok
test archive::tests::archive_import_rejects_unsupported_ca_metadata_without_persisting ... ok
test build_io::tests::rewrite_and_nar_hash_preserve_named_operation_behavior ... ok
test archive::tests::archive_import_rejects_conflicting_local_pathinfo ... ok
test archive::tests::archive_import_rejects_untrusted_signature_without_persisting ... ok
test archive::tests::archive_import_rejects_tampered_payload_without_persisting ... ok
test archive::tests::archive_import_rejects_existing_path_with_stale_final_nar_facts ... ok
test gc::tests::file_cleanup_reports_failure_after_attempting_later_independent_paths ... ok
test export::tests::export_directory_creates_files ... ok
test archive::tests::archive_round_trip_preserves_distinct_marker_ca_and_final_nar_identities ... ok
test export::tests::export_directory_sets_permissions_and_mtime ... ok
test gc::tests::path_explanation_rejects_retaining_root_links_above_policy_limit ... ok
test export::tests::export_directory_with_symlink ... ok
test gc::tests::reclaim_observation_preserves_unknown_bytes_and_plan_identity_binds_shape ... ok
test gc::tests::remove_path_accepts_an_already_missing_export ... ok
test gc::tests::remove_path_does_not_follow_a_symlink_outside_the_export_tree ... ok
test gc::tests::remove_path_removes_nested_read_only_export_tree ... ok
test archive::tests::archive_import_rejects_truncated_payload_without_persisting ... ok
test archive::tests::archive_import_round_trip_and_skip_existing_are_idempotent ... ok
test export::tests::export_file_executable_sets_permissions ... ok
test gc::tests::snapshot_pathinfos_finishes_before_later_mutation ... ok
test gc::tests::gc_aborts_when_root_registry_is_corrupt ... ok
test export::tests::export_file_non_executable_sets_permissions ... ok
test export::tests::export_file_rejects_short_blob ... ok
test handle::tests::action_result_nar_byte_accounting_distinguishes_transfer_reuse_and_overflow ... ok
test gc::tests::operation_reporting_preserves_first_failure_and_records_later_work ... ok
test export::tests::export_file_sets_mtime ... ok
test export::tests::export_file_empty_content ... ok
test export::tests::export_missing_blob_returns_error ... ok
test export::tests::export_file_writes_content ... ok
test export::tests::export_file_creates_parent_directories ... ok
test export::tests::export_symlink_creates_link ... ok
test export::tests::export_missing_directory_returns_error ... ok
test handle::tests::delta_capability_url_preserves_cache_subpath_with_trailing_slash ... ok
test handle::tests::delta_capability_url_preserves_cache_subpath_without_trailing_slash ... ok
test handle::tests::delta_capability_url_uses_root_cache_authority ... ok
test handle::tests::local_protocol_v1_matches_crunch_delta_wire_contract ... ok
test gc::tests::directory_outputs_survive_reopen_and_gc ... ok
test export::tests::export_nested_directory ... ok
test export::tests::export_symlink_sets_lmtime ... ok
test gc::tests::gc_aborts_when_retained_root_pathinfo_is_missing ... ok
test export::tests::export_moderate_depth_succeeds ... ok
test gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed ... ok
test gc::tests::pathinfo_rewrite_failure_stops_before_export_deletion ... ok
test gc::tests::gc_operation_order_matches_design ... ok
test gc::tests::stale_plan_is_rejected_after_root_change_without_deletion ... ok
test gc::tests::stale_plan_is_rejected_after_export_symlink_substitution ... ok
test handle::tests::overlay_missing_base_fails_closed ... ok
test handle::tests::overlay_duplicate_base_declaration_fails_before_overlay_creation ... ok
test gc::tests::dry_run_reports_same_candidates_as_real_run ... ok
test gc::tests::shared_blob_survives_when_reachable_path_still_references_it ... ok
test gc::tests::retained_root_keeps_transitive_closure_and_sidecars ... ok
test handle::tests::action_result_admission_preserves_current_detailed_artifact_attestation ... ok
test action_result::tests::http_interrupted_index_publication_leaves_record_undiscoverable ... ok
test action_result::tests::http_corrupt_record_and_poisoned_index_fail_closed ... ok
test handle::tests::cached_node_for_path_reuses_local_pathinfo_node ... ok
test handle::tests::check_cache_accepts_ca_mapping_with_custom_store_prefix ... ok
test gc::tests::unreachable_output_removes_pathinfo_exports_and_attestations ... ok
test handle::tests::check_cache_directory_output_with_missing_child_is_castore_incomplete ... ok
test handle::tests::failed_persist_does_not_register_root ... ok
test handle::tests::overlay_prefix_mismatch_fails_closed ... ok
test handle::tests::overlay_base_state_mutation_blocks_output_admission ... ok
test handle::tests::overlay_rejects_base_with_write_permission ... ok
test handle::tests::cached_node_for_path_rejects_incomplete_session_node ... ok
test build_io::tests::host_path_hash_is_deterministic_and_missing_paths_fail_closed ... ok
test handle::tests::overlay_generation_drift_blocks_later_read ... ok
test handle::tests::noop_publisher_is_default_and_skips_all_outputs ... ok
test handle::tests::check_cache_preserves_current_detailed_artifact_attestation ... ok
test handle::tests::check_cache_replaces_stale_artifact_attestation_facts ... ok
test action_result::tests::http_publication_is_record_first_discoverable_and_duplicate_safe ... ok
test handle::tests::overlay_ca_mapping_precedence_and_publication_are_layer_bounded ... ok
test handle::tests::overlay_incomplete_shadow_blocks_complete_base_fallback ... ok
test handle::tests::overlay_gc_execution_rejects_stale_base_generation ... ok
test handle::tests::overlay_read_through_base_hit_does_not_mutate_overlay ... ok
test handle::tests::overlay_shadowed_path_does_not_inherit_base_trust ... ok
test handle::tests::persist_signed_output_registers_self_build_root ... ok
test handle::tests::persistent_output_does_not_fail_on_publisher_error ... ok
test handle::tests::persist_signed_output_writes_artifact_attestation ... ok
test handle::tests::persist_signed_output_rejects_store_path_mismatch ... ok
test handle::tests::practical_pathinfo_open_fallback_records_audit_event ... ok
test handle::tests::persistent_output_calls_configured_publisher ... ok
test handle::tests::overlay_writes_route_to_overlay_only ... ok
test handle::tests::persist_signed_output_rejects_unsigned_pathinfo ... ok
test handle::tests::remote_trusted_key_parser_accepts_indexed_keys_and_rejects_duplicate_indexes ... ok
test handle::tests::resolve_same_authority_endpoint_rejects_cross_origin_urls ... ok
test handle::tests::persist_signed_output_registers_build_root ... ok
test handle::tests::strict_pathinfo_open_fallback_is_rejected ... ok
test handle::tests::overlay_rejects_base_pathinfo_with_invalid_layer_signature ... ok
test handle::tests::overlay_gc_rejects_base_to_overlay_reference ... ok
test handle::tests::overlay_shadows_base_pathinfo ... ok
test handle::tests::overlay_gc_retains_base_reachability_without_base_mutation ... ok
test handle::tests::try_substitute_remote_negative_miss_prevents_repeat_probe ... ok
test handle::tests::try_substitute_remote_records_metadata_cache_on_hit ... ok
test handle::tests::root_export_refreshes_stale_materialized_path ... ok
test handle::tests::remote_substitution_writes_artifact_attestation ... ok
test handle::tests::remote_substitution_without_cache_url_skips_probe_and_full_fetches ... ok
test handle::tests::remote_substitution_registers_bootstrap_root ... ok
test http_closure::tests::conflicting_digest_path_identity_is_rejected ... ok
test http_closure::tests::depth_limit_fails_closed ... ok
test http_closure::tests::duplicate_reference_is_rejected ... ok
test http_closure::tests::incomplete_plan_and_active_request_fail_closed ... ok
test http_closure::tests::invalid_limits_and_identity_are_rejected ... ok
test http_closure::tests::observation_without_active_request_is_rejected ... ok
test http_closure::tests::member_limit_fails_closed ... ok
test http_closure::tests::diamond_and_cycle_are_deduplicated ... ok
test http_closure::tests::reference_limit_fails_before_pending_members_change ... ok
test http_closure::tests::linear_plan_imports_root_last ... ok
test http_closure::tests::one_member_plan_is_stable_and_root_last ... ok
test http_closure::tests::reference_order_does_not_change_plan_identity ... ok
test http_closure::tests::returned_path_mismatch_is_rejected ... ok
test layer::tests::store_layer_display_includes_exact_base_index ... ok
test http_closure::tests::total_nar_size_limit_fails_closed ... ok
test layer::tests::layered_value_preserves_exact_index_through_map ... ok
test layer::tests::zero_service_index_is_overlay ... ok
test layer::tests::store_layer_booleans_are_disjoint ... ok
test metadata_cache::tests::evict_expired_removes_only_expired_entries ... ok
test metadata_cache::tests::force_refresh_disables_get ... ok
test metadata_cache::tests::load_corrupt_file_returns_empty ... ok
test metadata_cache::tests::load_empty_cache_from_nonexistent_file ... ok
test metadata_cache::tests::put_and_get_roundtrip ... ok
test metadata_cache::tests::put_replaces_existing_entry ... ok
test metadata_cache::tests::remove_removes_existing_entry ... ok
test metadata_cache::tests::remove_returns_false_for_missing_key ... ok
test overlay::tests::embedded_overlay_policy_is_typed_and_bounded ... ok
test mutation_lock::tests::try_acquire_rejects_second_mutator ... ok
test metadata_cache::tests::save_and_reload_persists_entries ... ok
test metadata_cache::tests::save_evicts_excess_entries ... ok
test overlay::tests::generation_observation_rejects_symlink_members ... ok
test overlay::tests::identity_record_rejects_prefix_drift ... ok
test path_identity::tests::marker_normalized_ca_path_is_accepted ... ok
test path_identity::tests::mismatched_ca_path_is_rejected ... ok
test provenance::tests::byte_reference_admission_requires_a_valid_store_path_digest ... ok
test handle::tests::verified_local_output_adoption_rejects_a_missing_physical_path ... ok
test overlay::tests::writable_base_is_rejected_before_generation_admission ... ok
test provenance::tests::classifier_covers_data_elf_script_and_unknown_executable ... ok
test provenance::tests::cpio_and_gzip_initrd_readers_classify_nested_executable_scripts ... ok
test provenance::tests::equivalent_observation_order_canonicalizes_deterministically ... ok
test provenance::tests::exact_reference_resolution_accepts_declared_target_and_rejects_foreign_unknown_and_escape ... ok
test provenance::tests::generic_compressed_streams_are_bounded_and_scanned_as_single_payloads ... ok
test provenance::tests::identity_shape_is_bounded_and_stable ... ok
test provenance::tests::castore_scan_rejects_untrusted_and_receipt_inconsistent_pathinfo_before_blob_reads ... ok
test provenance::tests::castore_scan_reports_missing_blob_before_claiming_complete_traversal ... ok
test provenance::tests::named_limits_all_fail_closed ... ok
test provenance::tests::malformed_and_bounded_containers_fail_closed ... ok
test provenance::tests::policy_accepts_bounded_sorted_profile_paths_and_rejects_bad_limits ... ok
test provenance::tests::preserved_identity_paths_are_targets_not_untranslated_foreign_references ... ok
test provenance::tests::shebang_accepts_profile_and_rejects_relative_missing_and_non_utf8_targets ... ok
test provenance::tests::symlink_resolution_rejects_escape_missing_and_loop ... ok
test provenance::tests::tar_reader_finds_hidden_unclassified_executable_and_path_escape ... ok
test provenance::tests::castore_scan_accepts_complete_signed_blob_without_host_fallback ... ok
test handle::tests::verified_local_output_adoption_rejects_changed_existing_content ... ok
test provenance::tests::castore_directory_scan_detects_symlink_loop_without_following_links ... ok
test handle::tests::overlay_report_records_selected_base_descriptor ... ok
test archive::tests::large_archive_payload_stays_on_the_chunked_castore_ingest_path ... ok
test handle::tests::overlay_reads_file_and_directory_content_from_distinct_bases_without_backfill ... ok
test handle::tests::verified_local_output_adoption_ingests_signs_and_persists ... ok
test handle::tests::verified_source_ingest_rejects_conflicting_existing_content ... ok
test export::tests::export_depth_limit_enforced ... ok
test handle::tests::overlay_two_bases_stack_in_declaration_order ... ok
test provenance::tests::castore_scan_counts_duplicate_nodes_and_enforces_duplicate_limit ... ok
test action_result::tests::http_timeout_rejects_source_without_fabricating_lookup ... ok
test handle::tests::try_substitute_remote_fallback_to_subsequent_url_when_primary_missing ... ok
test handle::tests::verified_source_ingest_preserves_exact_path_and_reuses_matching_content ... ok
test handle::tests::remote_substitution_404_delta_probe_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_probe_error_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_malformed_delta_capability_json_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_cross_authority_capability_falls_back_to_full_fetch ... ok
test handle::tests::delta_and_full_substitution_record_same_attestation_and_root_metadata ... ok
test handle::tests::remote_substitution_accepts_delta_chunk_stream_without_full_fetch ... ok
test provenance::tests::lexical_relative_resolution_never_returns_a_path_outside_root ... ok
test handle::tests::remote_substitution_directory_delta_without_local_directory_closure_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_malformed_stream_json_falls_back_to_full_fetch ... ok
test handle::tests::remote_substitution_missing_local_backing_content_is_absent_from_receiver_manifest ... ok
test handle::tests::remote_substitution_closure_scoped_delta_accepts_requested_output_and_reports_reuse ... ok
test handle::tests::remote_substitution_receiver_manifest_stays_bounded_to_requested_output ... ok
test handle::tests::remote_substitution_untrusted_delta_pathinfo_falls_back_to_full_fetch ... ok
test pull::tests::http_closure_duplicate_reference_fails_before_nar_download ... ok
test pull::tests::http_closure_narinfo_limit_fails_before_nar_download ... ok
test pull::tests::http_closure_plan_validator_rejects_before_nar_download_or_store_mutation ... ok
test pull::tests::http_closure_path_mismatch_fails_before_nar_download ... ok
test pull::tests::http_closure_changed_dependency_nar_keeps_closure_absent ... ok
test pull::tests::http_closure_total_nar_limit_fails_before_nar_download ... ok
test pull::tests::http_pull_detects_store_path_mismatch_and_client_metadata ... ok
test pull::tests::http_closure_missing_dependency_fails_before_nar_download ... ok
test pull::tests::http_closure_untrusted_root_fails_before_nar_download ... ok
test pull::tests::http_pull_rejects_cache_base_url_userinfo ... ok
test pull::tests::http_closure_dependency_content_failure_keeps_root_absent ... ok
test pull::tests::http_pull_does_not_recurse_into_missing_references ... ok
test pull::tests::http_pull_accepts_unsigned_when_trust_unsigned ... ok
test pull::tests::http_pull_accepts_unknown_key_signature_when_trust_unsigned ... ok
test pull::tests::http_pull_blocks_cross_scheme_redirects ... ok
test handle::tests::remote_substitution_probes_delta_capability_once_per_session ... ok
test pull::tests::http_closure_pull_discovers_all_metadata_and_imports_root_last ... ok
test pull::tests::http_pull_maps_narinfo_http_5xx_to_parse_error_count ... ok
test export::tests::export_file_allows_many_small_reads ... ok
test pull::tests::http_pull_handles_compressed_xz_nar ... ok
test pull::tests::http_pull_continues_after_narinfo_fetch_transport_failure ... ok
test pull::tests::pull_nonexistent_source_returns_error ... ok
test pull::tests::http_pull_maps_nar_http_failure_to_missing_nar_count ... ok
test pull::tests::http_pull_maps_narinfo_http_403_to_missing_nar_count ... ok
test pull::tests::http_closure_refetches_incomplete_local_dependency ... ok
test pull::tests::pull_accepts_unsigned_when_trust_unsigned ... ok
test pull::tests::pull_detects_nar_hash_mismatch ... ok
test pull::tests::http_pull_export_failure_after_persistence_is_fatal ... ok
test pull::tests::pull_rejects_store_dir_mismatch ... ok
test push::tests::deriver_normalization_rejects_an_empty_base_name ... ok
test pull::tests::http_pull_rejects_absolute_nar_url_without_download ... ok
test pull::tests::pull_rejects_untrusted_signature ... ok
test pull::tests::http_pull_rejects_malformed_narinfo_text ... ok
test handle::tests::remote_substitution_stream_failure_falls_back_through_real_http_cache ... ok
test push::tests::push_custom_store_dir_narinfo_uses_correct_prefix ... ok
test push::tests::push_idempotent_skip ... ok
test push::tests::push_includes_unsigned_when_trusted ... ok
test pull::tests::pull_single_signed_path_round_trip ... ok
test query::tests::store_sign_adds_signature ... ok
test query::tests::store_sign_all_skips_already_signed_entries ... ok
test pull::tests::http_pull_parses_references_with_local_store_prefix ... ok
test pull::tests::pull_skips_missing_nar ... ok
test push::tests::push_narinfo_references_match ... ok
test pull::tests::pull_skips_already_present ... ok
test query::tests::store_sign_appends_different_key_signature ... ok
test query::tests::store_sign_replaces_same_key_signature ... ok
test query::tests::store_verify_missing_for_path_not_on_disk_in_custom_store_dir ... ok
test query::tests::store_verify_ok_for_exported_path_in_custom_store_dir ... ok
test query::tests::store_verify_mismatch_for_tampered_disk_content ... ok
test query::tests::store_verify_read_failure_does_not_persist_pathinfo ... ok
test query::tests::verify_signatures_reports_untrusted_signer ... ok
test push::tests::push_normalizes_deriver_suffix_and_writes_parseable_narinfo ... ok
test query::tests::store_sign_then_verify_roundtrips_under_custom_prefix ... ok
test query::tests::store_verify_signatures_rejects_wrong_prefix ... ok
test query::tests::verify_signatures_accepts_trusted_key ... ok
test pull::tests::http_pull_rejects_unsigned_when_trust_unsigned_is_false ... ok
test push::tests::push_skips_unsigned_by_default ... ok
test push::tests::push_single_signed_path ... ok
test push::tests::push_preserves_existing_nix_cache_info ... ok
test repair::tests::pure_plan_distinguishes_current_and_stale_facts ... ok
test push::tests::push_multiple_paths ... ok
test repair::tests::pure_plan_rejects_each_unsafe_candidate ... ok
test retention::tests::embedded_policy_is_typed_and_bounded ... ok
test pull::tests::http_pull_nix_cache_info_redirect_rejection_warns_and_proceeds ... ok
test roots::tests::corrupt_registry_is_rejected_without_replacement ... ok
test roots::tests::pin_rejects_nonexistent_and_unreadable_paths ... ok
test roots::tests::legacy_record_migrates_to_protected_unmanaged_state ... ok
test roots::tests::project_generation_reuses_identity_and_advances_new_identity ... ok
test roots::tests::shell_lease_renewal_is_bounded_and_requires_lease_facts ... ok
test roots::tests::unmanaged_remote_registration_uses_remote_owner_scope ... ok
test roots::tests::managed_generation_batch_is_atomic_and_shares_generation_number ... ok
test roots::tests::register_root_survives_reload_with_versioned_provenance ... ok
test pull::tests::pull_multiple_paths ... ok
test roots::tests::unpin_removes_existing_versioned_record ... ok
test pull::tests::http_pull_rejects_narinfo_store_path_prefix_mismatch_after_matching_preflight ... ok
test pull::tests::http_closure_diamond_fetches_shared_member_once ... ok
test repair::tests::current_pathinfo_is_an_idempotent_no_op ... ok
test repair::tests::dry_run_does_not_mutate_stale_pathinfo_or_attestation ... ok
test repair::tests::missing_exact_pathinfo_is_rejected ... ok
test pull::tests::http_pull_pathinfo_persistence_failure_is_fatal ... ok
test repair::tests::invalid_ca_identity_is_rejected_without_pathinfo_mutation ... ok
test repair::tests::incomplete_content_is_rejected_without_pathinfo_mutation ... ok
test repair::tests::unsigned_stale_pathinfo_is_rejected_without_mutation ... ok
test repair::tests::stale_artifact_attestation_is_rejected_without_pathinfo_mutation ... ok
test pull::tests::pull_with_path_filter ... ok
test repair::tests::execute_repairs_facts_replaces_signatures_and_preserves_attestation_graph ... ok
test pull::tests::http_pull_rejects_untrusted_signature ... ok
test pull::tests::http_pull_network_failure_continues_for_remaining_paths ... ok
test pull::tests::http_pull_store_dir_mismatch_is_hard_error_before_narinfo_fetch ... ok
test pull::tests::http_pull_skips_missing_narinfo_404 ... ok
test repair::tests::pathinfo_persistence_failure_is_reported_without_mutation ... ok
test pull::tests::http_closure_pull_reuses_complete_dependency ... ok
test pull::tests::http_pull_single_signed_path_round_trip ... ok
test provenance::tests::payload_classification_is_deterministic_for_arbitrary_bytes ... ok
test pull::tests::http_pull_skips_already_present_without_narinfo_request ... ok
test pull::tests::http_pull_missing_and_malformed_nix_cache_info_both_proceed ... ok
test pull::tests::http_pull_nix_cache_info_client_error_warning_proceeds ... ok
test pull::tests::http_pull_maps_other_narinfo_http_statuses_to_parse_error_count ... ok
test pull::tests::http_pull_rejects_malformed_or_wrong_prefix_references_before_persistence ... ok
test pull::tests::http_pull_handles_gzip_bzip2_and_zstd_nar ... ok
test pull::tests::http_pull_maps_other_nar_http_statuses_to_missing_nar_count ... ok
test pull::tests::http_pull_keeps_path_prefixed_cache_urls_stable ... ok

test result: ok. 338 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s

     Running tests/authority_source_policy.rs (/home/brittonr/.cargo-target/debug/deps/authority_source_policy-d6caab10be9df2e6)

running 2 tests
test production_sources_keep_store_authority_narrow ... ok
test source_policy_rejects_each_authority_escape ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

   Doc-tests crunch_nar

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_store

running 7 tests
test crates/crunch-store/src/capability.rs - capability::BuildStore (line 44) - compile fail ... ok
test crates/crunch-store/src/capability.rs - capability::BuildStore (line 60) - compile fail ... ok
test crates/crunch-store/src/capability.rs - capability::BuildStore (line 52) - compile fail ... ok
test crates/crunch-store/src/capability.rs - capability::BuildStore (line 68) - compile fail ... ok
test crates/crunch-store/src/capability.rs - capability::BuildStore (line 36) - compile fail ... ok
test crates/crunch-store/src/capability.rs - capability::BuildServiceStore (line 128) - compile fail ... ok
test crates/crunch-store/src/capability.rs - capability::ActionResultPort (line 81) - compile fail ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

```

## Project refresh CLI tests

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test project_refresh_cli
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running tests/project_refresh_cli.rs (/home/brittonr/.cargo-target/debug/deps/project_refresh_cli-c865a5ec18198f18)

running 14 tests
test failed_recursive_observation_does_not_mutate_lock_or_generated_inputs ... ok
test list_stale_reports_stale_and_failed_without_mutating_files ... ok
test refresh_partial_failure_writes_successes_and_exits_nonzero ... ok
test freshness_no_network_mode_does_not_contact_http_probe ... ok
test freshness_local_directory_probe_updates_lock_digest ... ok
test refresh_trusted_file_input_with_wrong_signature_rejects_lock_write ... ok
test freshness_http_json_template_refreshes_selected_stale_input ... ok
test freshness_git_ref_probe_reports_stale_without_mutating_lock ... ok
test refresh_git_input_locks_resolved_rev_and_tree_hash ... ok
test build_fetch_policy_refresh_uses_expected_hash_without_network_resolution ... ok
test refresh_trusted_file_input_writes_lock_evidence_and_show_reports_claim ... ok
test refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes ... ok
test freshness_command_missing_timeout_and_output_limit_fail_deterministically ... ok
test refresh_tarball_supports_each_recursive_hash_algorithm ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

```

## Affected package checks

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo check -p crunch-nar -p crunch-store -p mantle
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-attestation-core)
    Checking snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-tracing)
    Checking nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/nix-compat)
    Checking fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/fuse-backend-rs)
    Checking crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-action-result-core)
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-overlay-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-gc-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-repair-core)
    Checking crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-rust-cache-core)
    Checking crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-eval)
    Checking crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-wasm-component-core)
    Checking crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-shell-core)
    Checking crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-delta-core)
    Checking crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-bootstrap-core)
    Checking mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/mantlepkgs-core)
    Checking crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-release-core)
    Checking crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-shell)
    Checking crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-attestation)
    Checking crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-project-core)
    Checking snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-castore)
    Checking crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-nar)
    Checking crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-glue)
    Checking crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-wasm-component)
    Checking crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-project)
    Checking snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-store)
    Checking snix-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/vendor/snix-build)
    Checking crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store)
    Checking crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-build)
    Checking crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-rust-cache)
    Checking crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-delta)
    Checking crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-rustc-wrapper)
    Checking crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-pipeline)
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 36.09s
```

## Rustfmt check

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt --check -p crunch-nar -p crunch-store -p mantle -v
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-nar/src/lib.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-nar/tests/parity.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store/src/lib.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store/tests/authority_source_policy.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_compare.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_eval_backends.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_eval_smoke.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_lazy_eval.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_scheduler_priority.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_suite.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/hardware_simulation_plan.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/picolibc_compare.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/projects/delta-substitution/demo.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/projects/release-witness-handoff/demo.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/projects/shared-action-result-roundtrip/publish.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/lib.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/attest_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/audit_support.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/benchmark_harness.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/bootstrap_eval.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/bootstrap_parity_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/bootstrap_validate_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/cargo_free_self_build_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/cargo_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/example_projects.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_build.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_eval.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_inventory.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_workflow_gallery.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/foreign_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/freshness_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/identity_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/integration.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/integration_build.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/kernel_bundle_oci_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/kernel_bundle_oci_registry_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/kernelscript_experiment.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/lock_importer_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/machine_schema_contracts.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/nix_free_demo_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/offline_build_runbook_docs.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/offline_cargo_project.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/operator_diagnostics.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/pin_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/project_build_smoke.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/project_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/project_refresh_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/release_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_credentials_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_rail_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_stdio_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_transfer_production.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/removed_system_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/retention_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/rust_compatibility_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/rust_plan_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/scheduling_policy.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/self_hosting.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/smoke.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/source_bundle_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/source_bundle_hydration_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/stdlib_tests.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/store_archive_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/store_gc_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/transcript_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/trust_policy_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/wasm_component_cli.rs"
rustfmt --edition 2024 --check /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-nar/src/lib.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-nar/tests/parity.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store/src/lib.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-store/tests/authority_source_policy.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_compare.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_eval_backends.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_eval_smoke.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_lazy_eval.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_scheduler_priority.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/benchmark_suite.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/hardware_simulation_plan.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/picolibc_compare.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/projects/delta-substitution/demo.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/projects/release-witness-handoff/demo.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/examples/projects/shared-action-result-roundtrip/publish.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/lib.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/attest_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/audit_support.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/benchmark_harness.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/bootstrap_eval.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/bootstrap_parity_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/bootstrap_validate_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/cargo_free_self_build_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/cargo_import_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/example_projects.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_build.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_eval.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_inventory.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/examples_workflow_gallery.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/foreign_import_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/freshness_offline_rail.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/identity_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/integration.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/integration_build.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/kernel_bundle_oci_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/kernel_bundle_oci_registry_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/kernelscript_experiment.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/lock_importer_offline_rail.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/machine_schema_contracts.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/nix_free_demo_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/offline_build_runbook_docs.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/offline_cargo_project.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/operator_diagnostics.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/pin_import_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/project_build_smoke.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/project_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/project_refresh_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/release_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_credentials_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_rail_offline_rail.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_stdio_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/remote_transfer_production.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/removed_system_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/retention_offline_rail.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/rust_compatibility_rail.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/rust_plan_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/scheduling_policy.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/self_hosting.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/smoke.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/source_bundle_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/source_bundle_hydration_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/stdlib_tests.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/store_archive_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/store_gc_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/transcript_cli.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/trust_policy_offline_rail.rs /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/tests/wasm_component_cli.rs
```

Package-check verdict: PASS
