// machine-artifact-public: realization.route-plan-report
//! Compatibility facade for the extracted build-planning core.

pub use crunch_build_planning_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn selected_route(input: RoutePlannerInput) -> RouteClass {
        plan_realization_route(input).selected_route
    }

    #[test]
    fn unknown_build_action_fails_closed_as_preflight_error() {
        let report = route_plan_for_existing_build_action("unknown-action", Some("operator input"));

        assert_eq!(report.selected_route, RouteClass::PreflightError);
        assert_eq!(report.selected_reason_code, "unknown-build-plan-action");
        assert_eq!(report.selected_detail.as_deref(), Some("operator input"));
    }

    #[test]
    fn selects_cached_local_before_remote_candidates() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            RouteCandidateFacts::eligible(RouteClass::CachedLocal, "local-hit"),
            RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "remote-hit").requiring_network(),
        ]));

        assert_eq!(report.selected_route, RouteClass::CachedLocal);
        assert_eq!(report.selected_reason_code, "local-hit");
        assert!(report.rejected_routes.is_empty());
    }

    #[test]
    fn candidate_order_does_not_change_tie_breaker_result() {
        let policy = RoutePolicy::practical_online();
        let left = selected_route(RoutePlannerInput::new(policy, vec![
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            RouteCandidateFacts::eligible(RouteClass::ArchiveImport, "archive-hit"),
            RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit").requiring_network(),
        ]));
        let right = selected_route(RoutePlannerInput::new(policy, vec![
            RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit").requiring_network(),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            RouteCandidateFacts::eligible(RouteClass::ArchiveImport, "archive-hit"),
        ]));

        assert_eq!(left, RouteClass::ArchiveImport);
        assert_eq!(right, RouteClass::ArchiveImport);
    }

    #[test]
    fn offline_policy_rejects_network_routes_before_local_build() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Offline,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::local_only(),
            },
            vec![
                RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-miss"),
                RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "remote-hit").requiring_network(),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[1].route, RouteClass::TrustedSubstitute);
        assert_eq!(report.rejected_routes[1].reason_code, "offline-network-required");
    }

    #[test]
    fn strong_claim_policy_rejects_practical_only_remote_build() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::StrongActionCorrectness,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit")
                    .requiring_network()
                    .practical_only()
                    .with_upload_summary(UploadSummary::new(
                        vec![UploadClass::Source, UploadClass::StoreObject],
                        2,
                        4_096,
                    )),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[0].route, RouteClass::P2pRemoteBuilder);
        assert_eq!(report.rejected_routes[0].reason_code, "claim-strength-mismatch");
    }

    #[test]
    fn upload_policy_rejects_forbidden_secret_descriptors() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit")
                    .requiring_network()
                    .practical_only()
                    .with_upload_summary(UploadSummary::new(vec![UploadClass::SecretDescriptor], 1, 128)),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[0].reason_code, "upload-privacy-denied");
        assert!(report.rejected_routes[0].detail.as_deref().unwrap_or_default().contains("SecretDescriptor"));
    }

    #[test]
    fn reports_preflight_when_no_realization_route_is_eligible() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-miss"),
            RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "remote-miss"),
            RouteCandidateFacts::rejected(RouteClass::LocalBuild, "local-preflight-failed"),
        ]));

        assert_eq!(report.selected_route, RouteClass::PreflightError);
        assert_eq!(report.selected_reason_code, "no-route-eligible");
        assert_eq!(report.rejected_routes.len(), 3);
    }

    #[test]
    fn existing_action_adapter_records_selected_and_rejected_routes() {
        let report = route_plan_for_existing_build_action("build", Some("out=build"));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert!(report.rejected_routes.iter().any(|route| route.route == RouteClass::TrustedSubstitute));
        assert!(report.rejected_routes.iter().all(|route| route.reason_code.len() <= MAX_REASON_CODE_BYTES));
    }

    #[test]
    fn selects_trusted_substitute_when_local_cache_misses() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::eligible(RouteClass::TrustedSubstitute, "trusted-remote-pathinfo-available")
                .requiring_network(),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::TrustedSubstitute);
        assert_eq!(report.rejected_routes[0].reason_code, "local-output-missing");
    }

    #[test]
    fn selects_archive_import_when_archive_candidate_is_eligible() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
            RouteCandidateFacts::eligible(RouteClass::ArchiveImport, "archive-prefix-and-signature-match"),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::ArchiveImport);
        assert_eq!(report.selected_reason_code, "archive-prefix-and-signature-match");
    }

    #[test]
    fn selects_source_bundle_route_when_source_material_is_required() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
            RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "archive-candidate-missing"),
            RouteCandidateFacts::eligible(RouteClass::SourceBundle, "declared-source-bundle-ready"),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::SourceBundle);
        assert_eq!(report.rejected_routes.len(), 3);
    }

    // r[verify realization_routing.source_bundle_route_execution]
    #[test]
    fn offline_ready_source_bundle_facts_select_input_realization_route() {
        let source_facts = SourceBundleRouteFacts {
            required: true,
            ready: true,
            reason_code: "declared-source-bundle-ready",
            detail: Some("source_state_blake3=abc123; non_claim=input-realization-only".to_string()),
        };
        let remote =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");

        let report = route_plan_for_build_action_with_remote_and_source(
            "build",
            Some("out=build"),
            Some(&remote),
            Some(&source_facts),
        );

        assert_eq!(report.policy.network, NetworkPolicy::Offline);
        assert_eq!(report.selected_route, RouteClass::SourceBundle);
        assert_eq!(report.selected_reason_code, "declared-source-bundle-ready");
        assert!(report.selected_detail.as_deref().unwrap_or_default().contains("source_state_blake3"));
    }

    // r[verify realization_routing.source_bundle_route_execution]
    #[test]
    fn offline_incomplete_source_bundle_facts_reject_network_and_local_fetch_routes() {
        let source_facts = SourceBundleRouteFacts {
            required: true,
            ready: false,
            reason_code: "missing-source-state",
            detail: Some("record=fixed-url-demo".to_string()),
        };
        let remote =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");

        let report = route_plan_for_build_action_with_remote_and_source(
            "build",
            Some("out=build"),
            Some(&remote),
            Some(&source_facts),
        );

        assert_eq!(report.policy.network, NetworkPolicy::Offline);
        assert_eq!(report.selected_route, RouteClass::PreflightError);
        assert!(
            report.rejected_routes.iter().any(|route| {
                route.route == RouteClass::SourceBundle && route.reason_code == "missing-source-state"
            })
        );
        assert!(report.rejected_routes.iter().any(|route| {
            route.route == RouteClass::P2pRemoteBuilder && route.reason_code == "offline-network-required"
        }));
        assert!(report.rejected_routes.iter().any(|route| {
            route.route == RouteClass::LocalBuild && route.reason_code == "offline-source-readiness-incomplete"
        }));
    }

    #[test]
    fn selects_remote_builder_with_privacy_safe_upload_summary() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
                RouteCandidateFacts::rejected(RouteClass::TrustedSubstitute, "trusted-substitute-missing"),
                RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "archive-candidate-missing"),
                RouteCandidateFacts::rejected(RouteClass::SourceBundle, "source-bundle-unavailable"),
                RouteCandidateFacts::eligible(
                    RouteClass::P2pRemoteBuilder,
                    "builder-capability-and-output-trust-match",
                )
                .requiring_network()
                .practical_only()
                .with_upload_summary(UploadSummary::new(
                    vec![UploadClass::StoreObject, UploadClass::Source],
                    12,
                    8_192,
                )),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
            ],
        ));

        let upload = report.upload_summary.as_ref().expect("upload summary");
        assert_eq!(report.selected_route, RouteClass::P2pRemoteBuilder);
        assert_eq!(upload.object_count, 12);
        assert_eq!(upload.byte_count, 8_192);
        assert!(upload.classes.contains(&UploadClass::Source));
    }

    #[test]
    fn cli_remote_builder_plan_facts_select_remote_route_without_secrets() {
        let facts =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");
        let report = route_plan_for_build_action_with_remote("build", Some("out=build"), Some(&facts));
        let json = serde_json::to_string(&report).expect("route report serializes");

        assert_eq!(report.selected_route, RouteClass::P2pRemoteBuilder);
        assert_eq!(report.selected_reason_code, "builder-capability-and-output-trust-match");
        assert!(report.selected_detail.as_deref().unwrap_or_default().contains("endpoint=builder-1"));
        assert!(report.upload_summary.is_some());
        assert!(!json.contains("secret"));
        assert!(!json.contains("ticket"));
    }

    #[test]
    fn cli_remote_builder_plan_rejects_missing_output_trust_before_local_build() {
        let facts =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 0).expect("remote facts configured");
        let report = route_plan_for_build_action_with_remote("build", Some("out=build"), Some(&facts));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert!(report.rejected_routes.iter().any(|route| {
            route.route == RouteClass::P2pRemoteBuilder && route.reason_code == "output-trust-missing"
        }));
    }

    #[test]
    fn remote_builder_plan_facts_reject_missing_input_capability_and_source_readiness() {
        let mut missing_inputs =
            RemoteBuilderPlanFacts::cli_configured(Some("builder-1"), true, 1).expect("remote facts configured");
        missing_inputs.concrete_inputs = false;
        let mut capability_mismatch = missing_inputs.clone();
        capability_mismatch.concrete_inputs = true;
        capability_mismatch.capabilities_match = false;
        let mut source_missing = capability_mismatch.clone();
        source_missing.capabilities_match = true;
        source_missing.source_inputs_ready = false;

        assert_eq!(remote_builder_candidate_from_facts(&missing_inputs).reason_code, "concrete-inputs-missing");
        assert_eq!(
            remote_builder_candidate_from_facts(&capability_mismatch).reason_code,
            "builder-capability-mismatch"
        );
        assert_eq!(remote_builder_candidate_from_facts(&source_missing).reason_code, "source-readiness-missing");
    }

    #[test]
    fn strong_claim_compatible_route_can_be_selected() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::StrongActionCorrectness,
                upload_policy: UploadPolicy::local_only(),
            },
            vec![
                RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
                RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-strong-receipts-ready"),
            ],
        ));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.policy.requested_claim_strength, ClaimStrength::StrongActionCorrectness);
    }

    #[test]
    fn negative_route_facts_keep_specific_blocker_codes() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::ArchiveImport, "archive-prefix-mismatch"),
            RouteCandidateFacts::rejected(RouteClass::SourceBundle, "missing-source-state"),
            RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "builder-capability-mismatch"),
            RouteCandidateFacts::rejected(RouteClass::LocalBuild, "local-preflight-failed"),
        ]));

        let reasons = report.rejected_routes.iter().map(|route| route.reason_code.as_str()).collect::<Vec<_>>();
        assert!(reasons.contains(&"archive-prefix-mismatch"));
        assert!(reasons.contains(&"missing-source-state"));
        assert!(reasons.contains(&"builder-capability-mismatch"));
        assert!(reasons.contains(&"local-preflight-failed"));
    }

    #[test]
    fn remote_builder_ticket_without_output_trust_is_rejected() {
        let report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::rejected(RouteClass::CachedLocal, "local-output-missing"),
            RouteCandidateFacts::rejected(RouteClass::P2pRemoteBuilder, "untrusted-builder-output-key")
                .requiring_network(),
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-preflight-ok"),
        ]));

        assert_eq!(report.selected_route, RouteClass::LocalBuild);
        assert_eq!(report.rejected_routes[1].reason_code, "untrusted-builder-output-key");
    }

    #[test]
    fn upload_summary_rejects_overflow_without_panicking() {
        let err =
            UploadSummary::try_new(vec![UploadClass::Source], MAX_UPLOAD_OBJECTS.saturating_add(1), MAX_UPLOAD_BYTES)
                .expect_err("object limit must fail");

        assert_eq!(err, RoutePlanError::UploadObjectLimitExceeded {
            actual: MAX_UPLOAD_OBJECTS.saturating_add(1),
            limit: MAX_UPLOAD_OBJECTS,
        });
    }

    #[test]
    fn route_report_binds_base_descriptor_generation_and_selected_layer() {
        let digest = blake3::hash(b"route-store-evidence").to_hex().to_string();
        let mut report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::eligible(RouteClass::CachedLocal, "base-cache-hit"),
        ]));

        report
            .bind_store_evidence(
                Some(RouteStoreOverlayEvidence {
                    plan_blake3: digest.clone(),
                    bases: vec![RouteStoreOverlayBaseEvidence {
                        layer_index: MIN_ROUTE_STORE_LAYER_INDEX,
                        descriptor_blake3: digest.clone(),
                        generation_blake3: digest.clone(),
                    }],
                }),
                vec![RouteStoreLayerEvidence {
                    store_path: "/mantle/store/example".to_string(),
                    selected_layer: "base[1]".to_string(),
                    base_descriptor_blake3: Some(digest.clone()),
                    base_generation_blake3: Some(digest),
                }],
            )
            .expect("bounded route store evidence must bind");

        assert_eq!(report.selected_store_layers[0].selected_layer, "base[1]");
        assert!(report.selected_store_layers[0].base_descriptor_blake3.is_some());
    }

    #[test]
    fn route_report_rejects_oversized_layer_label() {
        let mut report = plan_realization_route(RoutePlannerInput::new(RoutePolicy::practical_online(), vec![
            RouteCandidateFacts::eligible(RouteClass::LocalBuild, "local-build"),
        ]));
        let oversized_label = "x".repeat(MAX_ROUTE_STORE_LAYER_LABEL_BYTES.saturating_add(1));

        let error = report
            .bind_store_evidence(None, vec![RouteStoreLayerEvidence {
                store_path: "/mantle/store/example".to_string(),
                selected_layer: oversized_label,
                base_descriptor_blake3: None,
                base_generation_blake3: None,
            }])
            .expect_err("oversized route layer label must fail");

        assert_eq!(error, "route store layer label is invalid");
        assert!(report.selected_store_layers.is_empty());
    }

    #[test]
    fn route_reports_do_not_include_raw_secret_values() {
        let report = plan_realization_route(RoutePlannerInput::new(
            RoutePolicy {
                network: NetworkPolicy::Online,
                requested_claim_strength: ClaimStrength::Practical,
                upload_policy: UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                RouteCandidateFacts::eligible(RouteClass::P2pRemoteBuilder, "remote-builder-hit")
                    .requiring_network()
                    .with_upload_summary(UploadSummary::new(vec![UploadClass::Source], 1, 128)),
            ],
        ));
        let json = serde_json::to_string(&report).expect("route report serializes");

        assert_eq!(report.selected_route, RouteClass::P2pRemoteBuilder);
        assert!(report.non_claim.contains("advisory"));
        assert!(!report.non_claim.contains("build success"));
        assert!(!json.contains("token"));
        assert!(!json.contains("private"));
    }
}
