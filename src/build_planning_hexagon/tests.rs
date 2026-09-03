use std::cell::Cell;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INPUT_BYTES_MAX: u64 = 1_024;
const OUTPUT_BYTES_MAX: u64 = 2_048;
const BUILD_TIME_MS_MAX: u64 = 60_000;
const CPU_UNITS_MAX: u32 = 4;
const MEMORY_BYTES_MAX: u64 = 1_048_576;
const ATTEMPTS_MAX: u32 = 2;

#[test]
fn remote_contract_projects_candidate_without_opening_a_session() {
    let observation =
        super::remote_adapter::project_remote_candidate(super::remote_adapter::RemoteCandidateProjection {
            command: remote_command(),
            configured: true,
            credential_present: true,
            capabilities_match: true,
            source_inputs_ready: true,
            output_trusted: true,
            requires_network: true,
            upload_classes: vec![crunch_build_planning_core::UploadClass::StoreObject],
            upload_object_count: 1,
            upload_bytes: INPUT_BYTES_MAX,
        })
        .unwrap();
    assert_eq!(observation.candidate_identity_blake3.len(), crunch_remote_core::BLAKE3_HEX_LENGTH);
    assert!(observation.output_trusted);
    assert_eq!(observation.upload_summary.object_count, 1);
}

#[test]
fn stale_plan_rejects_before_effect_executor_runs() {
    let decision = planning_decision();
    let effect = decision.effects[1].clone();
    let calls = Cell::new(0_u32);
    let result = super::execution_adapter::execute_planned_effect(
        &decision,
        &effect,
        &"b".repeat(crunch_remote_core::BLAKE3_HEX_LENGTH),
        |_| {
            calls.set(calls.get().saturating_add(1));
            Ok::<_, ()>(((), successful_observation(&effect)))
        },
    );
    assert_eq!(calls.get(), 0);
    assert_eq!(
        result,
        Err(super::execution_adapter::PlannedEffectExecutionError::PlanningBlocker(
            crunch_build_planning_core::BuildPlanningBlocker::StalePlan,
        ))
    );
}

#[test]
fn accepted_plan_executes_exact_effect_and_records_observation() {
    let decision = planning_decision();
    let effect = decision.effects[1].clone();
    let calls = Cell::new(0_u32);
    let result = super::execution_adapter::execute_planned_effect(&decision, &effect, &decision.facts_blake3, |_| {
        calls.set(calls.get().saturating_add(1));
        Ok::<_, ()>(("done", successful_observation(&effect)))
    })
    .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(result.0, "done");
    assert!(result.1.succeeded);
}

fn planning_decision() -> crunch_build_planning_core::BuildPlanningDecision {
    super::observation_adapter::SuppliedBuildPlanningObservations {
        facts: planning_facts(),
        policy: crunch_build_planning_core::BuildPlanningPolicy::practical_local(),
        parallelism: crunch_build_planning_core::ParallelismFacts::legacy_compatible(
            Some(1),
            crunch_build_planning_core::ParallelismObservation::Unavailable,
        ),
    }
    .plan()
    .unwrap()
}

fn planning_facts() -> crunch_build_planning_core::BuildPlanningFacts {
    use crunch_build_planning_core::ObservedFact::Present;
    crunch_build_planning_core::BuildPlanningFacts {
        local_output: Present(crunch_build_planning_core::LocalOutputObservation {
            present: false,
            content_complete: false,
            trusted: false,
        }),
        source_readiness: Present(crunch_build_planning_core::SourceReadinessObservation {
            required: false,
            ready: true,
            source_identity_blake3: Some(DIGEST.to_string()),
        }),
        substituter: Present(crunch_build_planning_core::SubstituterObservation {
            configured: false,
            candidate_available: false,
            trusted: false,
            requires_network: true,
        }),
        archive: Present(crunch_build_planning_core::ArchiveObservation {
            configured: false,
            candidate_available: false,
            prefix_matches: false,
            signature_trusted: false,
        }),
        remote_candidate: Present(crunch_build_planning_core::RemoteCandidateObservation {
            candidate_identity_blake3: DIGEST.to_string(),
            configured: false,
            credential_present: false,
            capabilities_match: false,
            source_inputs_ready: false,
            output_trusted: false,
            requires_network: true,
            upload_summary: crunch_build_planning_core::UploadSummary {
                classes: Vec::new(),
                object_count: 0,
                byte_count: 0,
            },
        }),
        doctor: Present(crunch_build_planning_core::DoctorObservation {
            preflight_ok: true,
            report_identity_blake3: DIGEST.to_string(),
        }),
        platform: Present(crunch_build_planning_core::PlatformObservation {
            supported: true,
            platform_identity: "x86_64-linux".to_string(),
        }),
        trust: Present(crunch_build_planning_core::TrustObservation {
            trusted_output_key_count: 1,
            trust_policy_identity_blake3: DIGEST.to_string(),
        }),
        network: Present(crunch_build_planning_core::NetworkObservation { online: true }),
        executor: Present(crunch_build_planning_core::ExecutorObservation {
            local_available: true,
            remote_available: false,
            executor_limit: Some(1),
        }),
    }
}

fn successful_observation(
    effect: &crunch_build_planning_core::BuildPlanningEffect,
) -> crunch_build_planning_core::BuildPlanningEffectObservation {
    crunch_build_planning_core::BuildPlanningEffectObservation {
        effect_id_blake3: effect.effect_id_blake3.clone(),
        selected_route: effect.selected_route,
        kind: effect.kind,
        succeeded: true,
        reason_code: "adapter-observed-success".to_string(),
    }
}

fn remote_command() -> crunch_remote_core::RemoteCommand {
    crunch_remote_core::RemoteCommand {
        schema: crunch_remote_core::REMOTE_COMMAND_SCHEMA.to_string(),
        job_id: "job-a".to_string(),
        request_blake3: DIGEST.to_string(),
        worker_id: "worker-a".to_string(),
        fence_generation: 1,
        input_refs: vec!["source-a".to_string()],
        expected_outputs: vec![crunch_remote_core::RemoteOutputExpectation {
            name: "out".to_string(),
            logical_path: Some("/mantle/store/example".to_string()),
        }],
        policy: crunch_remote_core::RemotePolicy {
            attempts_max: ATTEMPTS_MAX,
            input_bytes_max: INPUT_BYTES_MAX,
            output_bytes_max: OUTPUT_BYTES_MAX,
            build_time_ms_max: BUILD_TIME_MS_MAX,
            cpu_units_max: CPU_UNITS_MAX,
            memory_bytes_max: MEMORY_BYTES_MAX,
            allow_transient_retry: true,
            require_output_trust: true,
        },
    }
}
