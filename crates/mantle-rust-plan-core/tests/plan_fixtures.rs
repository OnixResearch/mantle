//! Native Rust-plan core fixtures: selected topology, admission and observation.

use std::collections::BTreeMap;

use mantle_rust_plan_core::CompilerRestoreKind;
use mantle_rust_plan_core::ExistingTopologyUnit;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::NativeFeatureDefinition;
use mantle_rust_plan_core::NativeFeatureSelectionRequest;
use mantle_rust_plan_core::NativeInheritedValue;
use mantle_rust_plan_core::NativeTargetCandidate;
use mantle_rust_plan_core::NormalizedPackageFacts;
use mantle_rust_plan_core::PlanExecutionOutcome;
use mantle_rust_plan_core::ProcessEffectRole;
use mantle_rust_plan_core::ProcessWord;
use mantle_rust_plan_core::ProducerArtifactObservation;
use mantle_rust_plan_core::ResolvedProcessObservation;
use mantle_rust_plan_core::ResolvedUnitFacts;
use mantle_rust_plan_core::RestoredCompilerArtifactObservation;
use mantle_rust_plan_core::UnitId;
use mantle_rust_plan_core::UnitObservation;
use mantle_rust_plan_core::UnitObservationStatus;
use mantle_rust_plan_core::admit_native_targets;
use mantle_rust_plan_core::admit_resolved_build_script_after_cache;
use mantle_rust_plan_core::admit_resolved_unit_effect;
use mantle_rust_plan_core::classify_cargo_free_planning_blockers;
use mantle_rust_plan_core::classify_existing_unit_observations;
use mantle_rust_plan_core::classify_native_dependency_from_selected_cfg;
use mantle_rust_plan_core::classify_native_target_triple;
use mantle_rust_plan_core::classify_resolved_process_observation;
use mantle_rust_plan_core::classify_restored_compiler_artifact;
use mantle_rust_plan_core::classify_rust_plan_compatibility;
use mantle_rust_plan_core::evaluate_supported_target_cfg;
use mantle_rust_plan_core::hash_legacy_receipt_preimage;
use mantle_rust_plan_core::order_existing_unit_topology;
use mantle_rust_plan_core::plan_existing_unit_effects;
use mantle_rust_plan_core::plan_native_runtime_arguments;
use mantle_rust_plan_core::resolve_native_feature_selection;
use mantle_rust_plan_core::resolve_native_package_edition;
use mantle_rust_plan_core::resolve_native_package_version;
use mantle_rust_plan_core::select_native_execution_triple;
use mantle_rust_plan_core::select_normalized_package_closure;

fn real_unit(id: &str, rank: u32, dependencies: &[&str]) -> ExistingUnitFacts {
    ExistingUnitFacts {
        unit_id: id.to_string(),
        package_id: format!("path+file:///workspace#{id}@1.0.0"),
        target_name: id.to_string(),
        target_kind: String::from("lib"),
        execution_kind: String::from("target"),
        dependency_unit_ids: dependencies.iter().map(|id| (*id).to_string()).collect(),
        arguments: vec![String::from("--crate-name"), id.to_string()],
        environment: vec![(String::from("CARGO_MANIFEST_DIR"), String::from("/workspace"))],
        input_identities: vec![format!("source:{id}")],
        expected_outputs: vec![String::from("out")],
        execution_order: rank,
    }
}

#[test]
fn native_effects_derive_topology_and_preserve_actual_derivation_material() {
    let target = real_unit("target-unit-id", 0, &["host-unit-id", "dependency-unit-id"]);
    let mut host = real_unit("host-unit-id", 2, &[]);
    host.execution_kind = String::from("host");
    host.target_kind = String::from("proc-macro");
    let dependency = real_unit("dependency-unit-id", 1, &[]);
    let a = plan_existing_unit_effects(vec![target.clone(), host.clone(), dependency.clone()]).unwrap();
    let b = plan_existing_unit_effects(vec![dependency.clone(), target, host.clone()]).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.iter().map(|effect| effect.unit_id.0.as_str()).collect::<Vec<_>>(), vec![
        "dependency-unit-id",
        "host-unit-id",
        "target-unit-id"
    ]);
    assert_eq!(a[1].arguments, host.arguments);
    assert_eq!(a[1].environment, host.environment);
    assert_eq!(a[1].expected_outputs, host.expected_outputs);
    assert_eq!(a[1].input_identities, host.input_identities);
    assert_eq!(a[1].unit_id.0, host.unit_id);
}

#[test]
fn native_effects_reject_missing_producers_cycles_and_ambiguous_units() {
    let missing = plan_existing_unit_effects(vec![real_unit("app", 0, &["absent"])]).unwrap_err();
    assert!(missing.iter().any(|blocker| blocker.code == "missing-unit-producer"));
    let cycle = plan_existing_unit_effects(vec![real_unit("a", 0, &["b"]), real_unit("b", 1, &["a"])]).unwrap_err();
    assert!(cycle.iter().all(|blocker| blocker.code == "dependency-cycle"));
    let duplicate = plan_existing_unit_effects(vec![real_unit("a", 0, &[]), real_unit("a", 1, &[])]).unwrap_err();
    assert!(duplicate.iter().any(|blocker| blocker.code == "duplicate-unit"));
}

#[test]
fn native_effects_enforce_bounded_declarations_without_inventing_outputs() {
    let mut effect = real_unit("a", 0, &[]);
    effect.expected_outputs.clear();
    assert!(
        plan_existing_unit_effects(vec![effect.clone()])
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "incomplete-unit-effect")
    );
    effect.expected_outputs.push(String::from("out"));
    effect.environment.push((String::from("CARGO_MANIFEST_DIR"), String::from("/other")));
    assert!(
        plan_existing_unit_effects(vec![effect.clone()])
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "invalid-effect-environment")
    );
    effect.environment.pop();
    effect.arguments = vec![String::from("arg"); 513];
    assert!(
        plan_existing_unit_effects(vec![effect])
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "effect-argument-limit")
    );
    let mut unsupported = real_unit("unsupported", 0, &[]);
    unsupported.execution_kind = String::from("ambient");
    assert!(
        plan_existing_unit_effects(vec![unsupported])
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "unsupported-execution-kind")
    );
}

#[test]
fn native_observations_require_matching_effect_and_unit_and_consistent_exit() {
    let effects = plan_existing_unit_effects(vec![real_unit("real-unit-id", 0, &[])]).unwrap();
    let mut observed = vec![UnitObservation {
        effect_id: effects[0].effect_id.clone(),
        unit_id: effects[0].unit_id.clone(),
        status: UnitObservationStatus::Succeeded,
        exit_code: None,
        diagnostics_code: None,
    }];
    assert_eq!(classify_existing_unit_observations(&effects, &observed), PlanExecutionOutcome::Completed);
    observed[0].unit_id = UnitId(String::from("different"));
    assert_eq!(classify_existing_unit_observations(&effects, &observed), PlanExecutionOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 1
    });
    observed[0].unit_id = effects[0].unit_id.clone();
    observed[0].exit_code = Some(101);
    assert_eq!(classify_existing_unit_observations(&effects, &observed), PlanExecutionOutcome::Rejected {
        unknown_effect_count: 1,
        missing_effect_count: 1
    });
}

#[test]
fn accepted_compatibility_claims_distinguish_oracle_and_bounded_surface() {
    let oracle = classify_rust_plan_compatibility(false, &[String::from("unsupported")]);
    assert_eq!(oracle.compatibility_class, "cargo-oracle-evidence");
    assert_eq!(oracle.status, "oracle");
    assert_eq!(oracle.surface_ids, ["cargo-oracle-reference"]);
    let supported = classify_rust_plan_compatibility(true, &[]);
    assert_eq!(supported.compatibility_class, "cargo-free-bounded-topology");
    assert_eq!(supported.status, "supported");
    assert_eq!(supported.surface_ids, [
        "local-path-dependency",
        "path-workspace-basic",
        "source-closure-digest",
        "unit-graph-facts",
        "workspace-inheritance",
    ]);
    let blocked = classify_rust_plan_compatibility(true, &[String::from("missing-lock")]);
    assert_eq!(blocked.compatibility_class, "blocked-unsupported-surface");
    assert_eq!(blocked.status, "blocked");
    assert_eq!(blocked.blocker_classes, ["missing-lock"]);
    assert_eq!(blocked.surface_ids, ["blocked-unsupported-surface"]);
}

#[test]
fn native_feature_selection_preserves_explicit_seed_and_dependency_edges() {
    let mut request = NativeFeatureSelectionRequest {
        feature_definitions: vec![
            NativeFeatureDefinition {
                name: String::from("default"),
                entries: vec![String::from("full")],
            },
            NativeFeatureDefinition {
                name: String::from("full"),
                entries: vec![
                    String::from("dep:plugin"),
                    String::from("plugin?/serde"),
                    String::from("cli"),
                ],
            },
            NativeFeatureDefinition {
                name: String::from("cli"),
                entries: Vec::new(),
            },
        ],
        optional_dependencies: vec![String::from("plugin")],
        explicit_features: Vec::new(),
        all_features: false,
        no_default_features: false,
    };
    let default = resolve_native_feature_selection(&request);
    assert!(default.blockers.is_empty());
    assert_eq!(default.selected_features, ["cli", "default", "full"]);
    assert_eq!(default.activated_optional_dependencies, ["plugin"]);
    assert_eq!(default.dependency_feature_edges[0].dependency, "plugin");
    assert_eq!(default.dependency_feature_edges[0].feature, "serde");
    assert!(default.dependency_feature_edges[0].weak);
    request.explicit_features = vec![String::from("cli")];
    let explicit = resolve_native_feature_selection(&request);
    assert_eq!(explicit.selected_features, ["cli"]);
    assert!(explicit.activated_optional_dependencies.is_empty());
    request.explicit_features.clear();
    request.all_features = true;
    let all = resolve_native_feature_selection(&request);
    assert_eq!(all.selected_features, ["cli", "default", "full", "plugin"]);
}

#[test]
fn native_feature_selection_blocks_unknown_and_malformed_references() {
    let mut request = NativeFeatureSelectionRequest {
        feature_definitions: vec![NativeFeatureDefinition {
            name: String::from("default"),
            entries: vec![String::from("missing"), String::from("?/")],
        }],
        optional_dependencies: Vec::new(),
        explicit_features: Vec::new(),
        all_features: false,
        no_default_features: false,
    };
    let outcome = resolve_native_feature_selection(&request);
    let codes: Vec<_> = outcome.blockers.iter().map(|blocker| blocker.code.as_str()).collect();
    assert!(codes.contains(&"unknown-feature-entry"));
    assert!(codes.contains(&"unsupported-feature-entry"));
    request.no_default_features = true;
    assert!(resolve_native_feature_selection(&request).blockers.is_empty());
}

#[test]
fn native_execution_role_selects_declared_host_or_target_triple() {
    let host = "x86_64-unknown-linux-gnu";
    let target = "wasm32-unknown-unknown";
    assert_eq!(select_native_execution_triple("target", host, target), target);
    assert_eq!(select_native_execution_triple("host", host, target), host);
    assert_eq!(select_native_execution_triple("host-dependency", host, target), host);
    let host_cfg = classify_native_target_triple(host);
    assert_eq!((host_cfg.os, host_cfg.arch, host_cfg.vendor, host_cfg.env), ("linux", "x86_64", "unknown", "gnu"));
    assert_eq!((host_cfg.family, host_cfg.pointer_width, host_cfg.features), ("unix", "64", "fxsr,sse,sse2,x87"));
    let target_cfg = classify_native_target_triple(target);
    assert_eq!((target_cfg.os, target_cfg.family, target_cfg.pointer_width), ("unknown", "wasm", "32"));
    assert_eq!(
        target_cfg.features,
        "bulk-memory,multivalue,mutable-globals,nontrapping-fptoint,reference-types,sign-ext"
    );
}

#[test]
fn native_triple_classification_preserves_bounded_os_precedence() {
    let windows = classify_native_target_triple("x86_64-pc-windows-msvc");
    assert_eq!((windows.os, windows.family, windows.env), ("windows", "windows", "msvc"));
    let darwin = classify_native_target_triple("aarch64-apple-darwin");
    assert_eq!((darwin.os, darwin.family), ("darwin", "unix"));
    let linux_android = classify_native_target_triple("aarch64-linux-android");
    assert_eq!((linux_android.os, linux_android.family), ("linux", "unix"));
}

#[test]
fn native_dfs_topology_preserves_root_and_dependency_order() {
    let units = vec![
        ExistingTopologyUnit {
            unit_id: String::from("target"),
            dependencies: vec![String::from("host"), String::from("library")],
            execution_rank: 0,
        },
        ExistingTopologyUnit {
            unit_id: String::from("host"),
            dependencies: Vec::new(),
            execution_rank: 1,
        },
        ExistingTopologyUnit {
            unit_id: String::from("library"),
            dependencies: Vec::new(),
            execution_rank: 2,
        },
    ];
    let roots = vec![String::from("target"), String::from("library")];
    let reordered = vec![units[2].clone(), units[0].clone(), units[1].clone()];
    let expected = vec!["host", "library", "target"];
    assert_eq!(order_existing_unit_topology(&roots, &units).unwrap(), expected);
    assert_eq!(order_existing_unit_topology(&roots, &reordered).unwrap(), expected);
    let mut cyclic = units;
    cyclic[1].dependencies.push(String::from("target"));
    let blocker = order_existing_unit_topology(&roots, &cyclic).unwrap_err();
    assert_eq!(blocker.code, "dependency-cycle");
    assert_eq!(blocker.subject, "target");
}

fn native_package(id: &str, name: &str, order: &str, dependencies: &[&str]) -> NormalizedPackageFacts {
    NormalizedPackageFacts {
        package_id: id.to_string(),
        package_name: name.to_string(),
        version: String::from("1.0.0"),
        source_identity: format!("source:{id}"),
        order_key: order.to_string(),
        selected_features: vec![String::from("default")],
        selected_dependencies: dependencies.iter().map(|id| (*id).to_string()).collect(),
    }
}

#[test]
fn native_package_selection_uses_actual_ids_and_selected_resolved_edges() {
    let root = native_package("path+root#app@1", "app", "workspace/Cargo.toml", &["registry+a#lib@1"]);
    let dependency = native_package("registry+a#lib@1", "lib", "vendor/lib/Cargo.toml", &[]);
    let other_version = native_package("registry+b#lib@1", "lib", "vendor/other/Cargo.toml", &[]);
    let packages = vec![other_version, root.clone(), dependency.clone()];
    let roots = vec![root.package_id.clone()];
    let selected = select_normalized_package_closure(&roots, &packages).unwrap();
    assert_eq!(selected, [dependency.package_id.clone(), root.package_id.clone()]);
    let reversed = packages.into_iter().rev().collect::<Vec<_>>();
    assert_eq!(select_normalized_package_closure(&roots, &reversed).unwrap(), selected);
}

#[test]
fn native_package_admission_rejects_duplicate_and_missing_selected_facts() {
    let mut root = native_package("root", "app", "workspace/Cargo.toml", &["missing"]);
    let blockers = select_normalized_package_closure(&[String::from("root")], &[root.clone()]).unwrap_err();
    assert!(blockers.iter().any(|blocker| blocker.code == "missing-dependency-package"));
    root.selected_dependencies.clear();
    let blockers = select_normalized_package_closure(&[String::from("root")], &[root.clone(), root]).unwrap_err();
    assert!(blockers.iter().any(|blocker| blocker.code == "duplicate-package"));
    let root = native_package("root", "app", "workspace/Cargo.toml", &[]);
    let blockers = select_normalized_package_closure(&[String::new()], &[root]).unwrap_err();
    assert!(blockers.iter().any(|blocker| blocker.code == "missing-package-root" && blocker.subject == "roots"));
}

#[test]
fn native_package_selection_rejects_oversized_scalar_facts_before_traversal() {
    let oversized = "x".repeat(1024 * 1024 + 1);
    let root = String::from("root");
    let package = native_package("root", "app", "workspace/Cargo.toml", &[]);
    let blockers = select_normalized_package_closure(std::slice::from_ref(&oversized), std::slice::from_ref(&package))
        .unwrap_err();
    assert_eq!(blockers[0].code, "package-root-limit");
    assert_eq!(blockers[0].subject, "roots");

    for field in ["package_id", "package_name", "version", "source_identity", "order_key"] {
        let mut malformed = package.clone();
        match field {
            "package_id" => malformed.package_id = oversized.clone(),
            "package_name" => malformed.package_name = oversized.clone(),
            "version" => malformed.version = oversized.clone(),
            "source_identity" => malformed.source_identity = oversized.clone(),
            "order_key" => malformed.order_key = oversized.clone(),
            _ => unreachable!(),
        }
        let blockers = select_normalized_package_closure(std::slice::from_ref(&root), &[malformed]).unwrap_err();
        assert_eq!(blockers[0].code, "package-identity-limit", "{field}");
        assert_eq!(blockers[0].subject, "packages", "{field}");
    }
    let mut malformed = package.clone();
    malformed.selected_features = vec![oversized.clone()];
    let blockers = select_normalized_package_closure(std::slice::from_ref(&root), &[malformed]).unwrap_err();
    assert_eq!(blockers[0].code, "package-selection-identity-limit");
    let mut malformed = package.clone();
    malformed.selected_dependencies = vec![oversized];
    let blockers = select_normalized_package_closure(std::slice::from_ref(&root), &[malformed]).unwrap_err();
    assert_eq!(blockers[0].code, "package-selection-identity-limit");

    let mut admitted = package;
    admitted.selected_features = vec!["x".repeat(1024 * 1024)];
    assert_eq!(select_normalized_package_closure(std::slice::from_ref(&root), &[admitted]).unwrap(), vec![root]);
}

fn resolved_facts(effect: &mantle_rust_plan_core::ExistingUnitEffect) -> ResolvedUnitFacts {
    ResolvedUnitFacts {
        effect_id: effect.effect_id.clone(),
        unit_id: effect.unit_id.clone(),
        role: ProcessEffectRole::RustCompiler,
        attempt: 0,
        prior_attempt_identity: None,
        executable: ProcessWord::Opaque {
            encoding: String::from("unix-os-bytes"),
            bytes: b"/tool/rustc".to_vec(),
        },
        toolchain_identity: String::from("rustc:toolchain"),
        arguments: effect.arguments.iter().cloned().map(ProcessWord::Utf8).collect(),
        environment: vec![(String::from("PATH"), ProcessWord::Opaque {
            encoding: String::from("unix-os-bytes"),
            bytes: vec![b'/', 0xff, b'/'],
        })],
        working_directory: Some(ProcessWord::Utf8(String::from("/workspace"))),
        input_identities: effect.input_identities.clone(),
        expected_outputs: effect.expected_outputs.clone(),
        dependency_observations: Vec::new(),
    }
}

#[test]
fn resolved_process_effect_preserves_opaque_child_environment_and_observed_outputs() {
    let planned = plan_existing_unit_effects(vec![real_unit("real-unit", 0, &[])]).unwrap();
    let admitted = admit_resolved_unit_effect(&planned[0], resolved_facts(&planned[0]), None).unwrap();
    assert!(
        matches!(&admitted.facts.environment[0].1, ProcessWord::Opaque { bytes, .. } if bytes == &[b'/', 0xff, b'/'])
    );
    let observed = ResolvedProcessObservation {
        effect_id: admitted.facts.effect_id.clone(),
        unit_id: admitted.facts.unit_id.clone(),
        resolved_blake3: admitted.resolved_blake3.clone(),
        role: ProcessEffectRole::RustCompiler,
        attempt: 0,
        status: UnitObservationStatus::Succeeded,
        exit_code: Some(0),
        output_identities: vec![(String::from("out"), String::from("artifact-digest"))],
    };
    assert!(classify_resolved_process_observation(&admitted, &observed).is_ok());
    let mut drift = observed.clone();
    drift.resolved_blake3 = mantle_rust_plan_core::Blake3Digest::from_slice(b"other");
    assert_eq!(
        classify_resolved_process_observation(&admitted, &drift).unwrap_err().code,
        "wrong-resolved-observation"
    );
    drift = observed;
    drift.output_identities.clear();
    assert_eq!(
        classify_resolved_process_observation(&admitted, &drift).unwrap_err().code,
        "missing-observed-output"
    );
}

#[test]
fn resolved_process_rejects_missing_producer_and_unrelated_inputs() {
    let planned =
        plan_existing_unit_effects(vec![real_unit("consumer", 1, &["producer"]), real_unit("producer", 0, &[])])
            .unwrap();
    let consumer = planned.iter().find(|effect| effect.unit_id.0 == "consumer").unwrap();
    let mut facts = resolved_facts(consumer);
    assert!(
        admit_resolved_unit_effect(consumer, facts.clone(), None)
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "missing-producer-observation")
    );
    facts.dependency_observations = vec![ProducerArtifactObservation {
        producer_unit_id: UnitId(String::from("producer")),
        artifact_identity: String::from("artifact:producer"),
    }];
    assert!(
        admit_resolved_unit_effect(consumer, facts.clone(), None)
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "invalid-producer-observation")
    );
    facts.input_identities.push(String::from("artifact:producer"));
    assert!(admit_resolved_unit_effect(consumer, facts.clone(), None).is_ok());
    facts.input_identities.clear();
    assert!(
        admit_resolved_unit_effect(consumer, facts, None)
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "missing-resolved-input")
    );
}

#[test]
fn resolved_process_byte_bound_includes_all_matching_producer_artifact_observations() {
    let mut units = (0..9).map(|rank| real_unit(&format!("producer-{rank}"), rank, &[])).collect::<Vec<_>>();
    let mut consumer = real_unit("consumer", 9, &[]);
    consumer.dependency_unit_ids = units.iter().map(|unit| unit.unit_id.clone()).collect();
    units.push(consumer);
    let planned = plan_existing_unit_effects(units).unwrap();
    let consumer = planned.iter().find(|effect| effect.unit_id.0 == "consumer").unwrap();
    let mut facts = resolved_facts(consumer);
    let payload = "a".repeat(480_000);
    for producer in &consumer.dependency_unit_ids {
        let artifact_identity = format!("{}:{payload}", producer.0);
        facts.input_identities.push(artifact_identity.clone());
        facts.dependency_observations.push(ProducerArtifactObservation {
            producer_unit_id: producer.clone(),
            artifact_identity,
        });
    }
    let blockers = admit_resolved_unit_effect(consumer, facts, None).unwrap_err();
    assert_eq!(blockers.iter().map(|blocker| blocker.code.as_str()).collect::<Vec<_>>(), vec![
        "resolved-effect-byte-limit"
    ]);
}

#[test]
fn compiler_fallback_and_build_script_run_require_causal_attempts() {
    let mut unit = real_unit("build-script-unit", 0, &[]);
    unit.target_kind = String::from("custom-build");
    let planned = plan_existing_unit_effects(vec![unit]).unwrap();
    let effect = &planned[0];
    let primary = admit_resolved_unit_effect(effect, resolved_facts(effect), None).unwrap();
    let failed = ResolvedProcessObservation {
        effect_id: effect.effect_id.clone(),
        unit_id: effect.unit_id.clone(),
        resolved_blake3: primary.resolved_blake3.clone(),
        role: ProcessEffectRole::RustCompiler,
        attempt: 0,
        status: UnitObservationStatus::Failed,
        exit_code: Some(101),
        output_identities: Vec::new(),
    };
    let failed_id = classify_resolved_process_observation(&primary, &failed).unwrap();
    let mut fallback = resolved_facts(effect);
    fallback.attempt = 1;
    assert!(
        admit_resolved_unit_effect(effect, fallback.clone(), Some(&failed))
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "resolved-effect-predecessor")
    );
    fallback.prior_attempt_identity = Some(failed_id);
    assert!(admit_resolved_unit_effect(effect, fallback, Some(&failed)).is_ok());

    let success = ResolvedProcessObservation {
        status: UnitObservationStatus::Succeeded,
        exit_code: Some(0),
        output_identities: vec![(String::from("out"), String::from("compiled-script"))],
        ..failed
    };
    let success_id = classify_resolved_process_observation(&primary, &success).unwrap();
    let mut run = resolved_facts(effect);
    run.role = ProcessEffectRole::BuildScriptRun;
    run.arguments.clear();
    run.expected_outputs = vec![String::from("OUT_DIR"), String::from("metadata-stdout")];
    run.prior_attempt_identity = Some(success_id);
    assert!(admit_resolved_unit_effect(effect, run.clone(), Some(&success)).is_ok());
    run.expected_outputs = vec![String::from("out")];
    assert!(
        admit_resolved_unit_effect(effect, run, Some(&success))
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "resolved-effect-outputs")
    );
}

#[test]
fn resolved_process_rejects_unroundtrippable_host_words() {
    let planned = plan_existing_unit_effects(vec![real_unit("real-unit", 0, &[])]).unwrap();
    let effect = &planned[0];
    let mut facts = resolved_facts(effect);
    for bad_word in [
        ProcessWord::Utf8(String::from("bad\0arg")),
        ProcessWord::Opaque {
            encoding: String::from("arbitrary"),
            bytes: vec![1],
        },
        ProcessWord::Opaque {
            encoding: String::from("unix-os-bytes"),
            bytes: vec![b'a', 0],
        },
        ProcessWord::Opaque {
            encoding: String::from("windows-wtf16le"),
            bytes: vec![1],
        },
        ProcessWord::Opaque {
            encoding: String::from("windows-wtf16le"),
            bytes: vec![0, 0],
        },
    ] {
        facts.arguments.push(bad_word);
        assert!(
            admit_resolved_unit_effect(effect, facts.clone(), None)
                .unwrap_err()
                .iter()
                .any(|blocker| blocker.code == "incomplete-resolved-effect")
        );
        facts.arguments.pop();
    }
    facts.arguments.push(ProcessWord::Opaque {
        encoding: String::from("windows-wtf16le"),
        bytes: vec![0x3d, 0xd8, 0, 0xdc],
    });
    assert!(admit_resolved_unit_effect(effect, facts, None).is_ok());
}

#[test]
fn build_script_run_after_cache_hit_requires_real_receipt_and_artifact_identity() {
    let mut unit = real_unit("cached-build-script", 0, &[]);
    unit.target_kind = String::from("custom-build");
    let planned = plan_existing_unit_effects(vec![unit]).unwrap();
    let effect = &planned[0];
    let primary = admit_resolved_unit_effect(effect, resolved_facts(effect), None).unwrap();
    let receipt_hash = mantle_rust_plan_core::Blake3Digest::from_slice(b"actual-cache-receipt");
    let output_set = mantle_rust_plan_core::Blake3Digest::from_slice(b"actual-re-digested-output-list");
    let mut restored = RestoredCompilerArtifactObservation {
        effect_id: effect.effect_id.clone(),
        unit_id: effect.unit_id.clone(),
        compiler_resolved_blake3: primary.resolved_blake3.clone(),
        cache_receipt_identity: receipt_hash.clone(),
        declared_outputs: effect.expected_outputs.clone(),
        output_artifact_set_identity: output_set.clone(),
        output_artifact_count: 1,
        cache_kind: CompilerRestoreKind::RestoredLocal,
    };
    let predecessor = classify_restored_compiler_artifact(effect, &primary, &restored).unwrap();
    let mut run = resolved_facts(effect);
    run.role = ProcessEffectRole::BuildScriptRun;
    run.arguments.clear();
    run.expected_outputs = vec![String::from("OUT_DIR"), String::from("metadata-stdout")];
    run.prior_attempt_identity = Some(predecessor);
    run.input_identities
        .extend([String::from(receipt_hash.as_str()), String::from(output_set.as_str())]);
    assert!(
        admit_resolved_unit_effect(effect, run.clone(), None)
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "resolved-effect-predecessor")
    );
    let admitted = admit_resolved_build_script_after_cache(effect, run.clone(), &primary, &restored).unwrap();
    assert_eq!(admitted.facts.role, ProcessEffectRole::BuildScriptRun);
    run.input_identities.pop();
    assert!(
        admit_resolved_build_script_after_cache(effect, run, &primary, &restored)
            .unwrap_err()
            .iter()
            .any(|blocker| blocker.code == "resolved-effect-predecessor")
    );
    restored.output_artifact_count = 0;
    assert_eq!(
        classify_restored_compiler_artifact(effect, &primary, &restored).unwrap_err().code,
        "restored-compiler-outputs"
    );
    restored.output_artifact_count = 1;
    restored.compiler_resolved_blake3 = mantle_rust_plan_core::Blake3Digest::from_slice(b"different-compiler");
    assert_eq!(
        classify_restored_compiler_artifact(effect, &primary, &restored).unwrap_err().code,
        "restored-compiler-predecessor"
    );
}

#[test]
fn native_package_inheritance_and_target_admission_preserve_manifest_order_and_errors() {
    let inherited = NativeInheritedValue {
        literal: None,
        workspace: true,
        workspace_value: Some("2.4.1"),
    };
    assert_eq!(resolve_native_package_version("my-app", inherited).unwrap(), "2.4.1");
    assert_eq!(
        resolve_native_package_version("my-app", NativeInheritedValue {
            literal: Some("3.0.0"),
            ..inherited
        })
        .unwrap(),
        "3.0.0"
    );
    assert_eq!(
        resolve_native_package_version("my-app", NativeInheritedValue {
            workspace_value: None,
            ..inherited
        })
        .unwrap_err()
        .code,
        "missing-workspace-package-version"
    );
    assert_eq!(
        resolve_native_package_version("my-app", NativeInheritedValue {
            workspace: false,
            ..inherited
        })
        .unwrap_err()
        .code,
        "missing-package-version"
    );
    assert_eq!(
        resolve_native_package_edition("my-app", NativeInheritedValue {
            workspace: false,
            workspace_value: None,
            literal: None,
        })
        .unwrap(),
        "2015"
    );
    assert_eq!(
        resolve_native_package_edition("my-app", NativeInheritedValue {
            workspace_value: None,
            ..inherited
        })
        .unwrap_err()
        .code,
        "missing-workspace-package-edition"
    );

    let lib = NativeTargetCandidate {
        name: String::from("my-app"),
        kind: String::from("lib"),
        source_path: String::from("/workspace/src/lib.rs"),
        edition: String::from("2021"),
        source_failure_display: None,
    };
    let bin = NativeTargetCandidate {
        name: String::from("main"),
        kind: String::from("bin"),
        source_path: String::from("/workspace/src/main.rs"),
        edition: String::from("2021"),
        source_failure_display: None,
    };
    let admitted = admit_native_targets("path+workspace#my-app@2.4.1", vec![lib.clone(), bin, lib.clone()]).unwrap();
    assert_eq!(admitted.iter().map(|target| (target.name.as_str(), target.kind.as_str())).collect::<Vec<_>>(), [
        ("main", "bin"),
        ("my-app", "lib")
    ]);
    assert_eq!(admitted[1].crate_name, "my_app");
    let mut missing = lib;
    missing.source_failure_display = Some(String::from("/workspace/src/lib.rs"));
    let failure = admit_native_targets("path+workspace#my-app@2.4.1", vec![missing]).unwrap_err();
    assert_eq!(failure.code, "missing-target-source");
    assert!(failure.message.contains("/workspace/src/lib.rs"));
    assert_eq!(
        admit_native_targets("path+workspace#my-app@2.4.1", Vec::new()).unwrap_err().code,
        "missing-supported-target"
    );
}

#[test]
fn malformed_native_target_and_package_fields_do_not_cross_admission() {
    let present = NativeTargetCandidate {
        name: String::from("app"),
        kind: String::from("bin"),
        source_path: String::from("/workspace/src/main.rs"),
        edition: String::from("2021"),
        source_failure_display: None,
    };
    let mut malformed = present.clone();
    malformed.name.clear();
    assert_eq!(admit_native_targets("pkg", vec![malformed.clone()]).unwrap_err().code, "target-identity");
    malformed.name = present.name.clone();
    malformed.kind.clear();
    assert_eq!(admit_native_targets("pkg", vec![malformed.clone()]).unwrap_err().code, "target-identity");
    malformed.kind = String::from("example");
    assert_eq!(admit_native_targets("pkg", vec![malformed.clone()]).unwrap_err().code, "unsupported-target-kind");
    malformed.kind = present.kind.clone();
    malformed.source_path.clear();
    assert_eq!(admit_native_targets("pkg", vec![malformed.clone()]).unwrap_err().code, "target-identity");
    malformed.source_path = present.source_path.clone();
    malformed.edition.clear();
    assert_eq!(admit_native_targets("pkg", vec![malformed]).unwrap_err().code, "target-identity");
    assert_eq!(admit_native_targets("", vec![present.clone()]).unwrap_err().code, "package-identity");
    let oversized_source_display = "a".repeat(1024 * 1024 + 1);
    let mut unreadable = present.clone();
    unreadable.source_failure_display = Some(oversized_source_display.clone());
    assert_eq!(admit_native_targets("pkg", vec![unreadable]).unwrap_err().code, "target-identity-limit");
    let mut unreadable = present.clone();
    unreadable.name = oversized_source_display;
    unreadable.source_failure_display = Some(String::from("/workspace/src/main.rs"));
    assert_eq!(admit_native_targets("pkg", vec![unreadable]).unwrap_err().code, "target-identity-limit");
    let mut oversized = present;
    oversized.name = "a".repeat(1024 * 1024 + 1);
    assert_eq!(admit_native_targets("pkg", vec![oversized]).unwrap_err().code, "target-identity-limit");
    assert_eq!(
        resolve_native_package_version("", NativeInheritedValue {
            literal: Some("1.0.0"),
            workspace: false,
            workspace_value: None,
        })
        .unwrap_err()
        .code,
        "package-identity"
    );
    assert_eq!(
        resolve_native_package_edition("app", NativeInheritedValue {
            literal: Some(""),
            workspace: false,
            workspace_value: None,
        })
        .unwrap_err()
        .code,
        "package-identity"
    );
}

#[test]
fn cfg_dependencies_use_selected_triple_once_and_optional_feature_edges() {
    let linux = "x86_64-unknown-linux-gnu";
    let windows = "x86_64-pc-windows-msvc";
    let expression = "cfg(all(target_arch = \"x86_64\", any(unix, not(windows))))";
    assert_eq!(evaluate_supported_target_cfg(expression, linux), Some(true));
    assert_eq!(evaluate_supported_target_cfg(expression, windows), Some(false));
    assert_eq!(evaluate_supported_target_cfg("cfg(unrecognized_predicate)", linux), None);
    assert_eq!(evaluate_supported_target_cfg("x86_64-unknown-linux-gnu", linux), Some(true));
    let oversized = "x".repeat(1024 * 1024 + 1);
    assert_eq!(evaluate_supported_target_cfg(&oversized, linux), None);
    assert_eq!(evaluate_supported_target_cfg(expression, &oversized), None);
    let definitions = BTreeMap::from([
        (String::from("enabled"), vec![String::from("dep:optional?/serde")]),
        (String::from("disabled"), vec![String::from("other")]),
    ]);
    let selected = vec![String::from("enabled")];
    let active = classify_native_dependency_from_selected_cfg(true, "optional", true, &definitions, &selected);
    assert!(active.is_optional_selected && active.is_dependency_selected);
    assert_eq!(active.decision, "selected");
    let inactive = classify_native_dependency_from_selected_cfg(false, "optional", true, &definitions, &selected);
    assert!(!inactive.is_dependency_selected);
    assert_eq!(inactive.decision, "not-selected");
    let optional =
        classify_native_dependency_from_selected_cfg(true, "optional", true, &definitions, &[String::from("disabled")]);
    assert_eq!(optional.decision, "not-selected-optional");
}

#[test]
fn runtime_arguments_and_cargo_free_blockers_preserve_explicit_policy() {
    let reviewable = vec![String::from("--crate-name"), String::from("demo")];
    let mut expected = reviewable.clone();
    expected.extend([String::from("-C"), String::from("link-self-contained=no")]);
    assert_eq!(plan_native_runtime_arguments(&reviewable), expected);
    for explicit in [vec![String::from("-C"), String::from("link-self-contained=yes")], vec![
        String::from("-Clink-self-contained=no"),
    ]] {
        assert_eq!(plan_native_runtime_arguments(&explicit), explicit);
    }
    assert_eq!(plan_native_runtime_arguments(&[String::from("-C"), String::from("link-self-containedness=yes")]), [
        String::from("-C"),
        String::from("link-self-containedness=yes"),
        String::from("-C"),
        String::from("link-self-contained=no")
    ]);
    assert_eq!(classify_cargo_free_planning_blockers([false, true, false, true]), [
        "native-host-unit-graph-planning-blocked",
        "native-package-target-planning-blocked"
    ]);
}

#[test]
fn typed_receipt_preimage_hashes_exact_field_order_without_new_domain() {
    #[derive(serde::Serialize)]
    struct Receipt<'a> {
        unit_id: &'a str,
        receipt_hash: &'a str,
    }
    let receipt = Receipt {
        unit_id: "unit",
        receipt_hash: "",
    };
    let expected = blake3::hash(br#"{"unit_id":"unit","receipt_hash":""}"#).to_hex().to_string();
    assert_eq!(hash_legacy_receipt_preimage(&receipt).unwrap(), expected);
    let differing = Receipt {
        unit_id: "unit",
        receipt_hash: "already-filled",
    };
    assert_ne!(hash_legacy_receipt_preimage(&differing).unwrap(), expected);
}
