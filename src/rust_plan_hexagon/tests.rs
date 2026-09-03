use mantle_rust_plan::CargoOraclePort;
use mantle_rust_plan::CompilerInspectionPort;
use mantle_rust_plan::RustCachePort;
use mantle_rust_plan::UnitExecutionPort;
use mantle_rust_plan::WorkspaceFactsPort;

const BLAKE3_HEX_CHARS: usize = 64;

fn digest(byte: char) -> String {
    std::iter::repeat_n(byte, BLAKE3_HEX_CHARS).collect()
}

fn effect() -> mantle_rust_plan_core::RustUnitEffect {
    mantle_rust_plan_core::RustUnitEffect {
        effect_id: digest('a'),
        unit_id: "unit".to_string(),
        compiler_identity: digest('b'),
        arguments: vec!["--crate-name".to_string(), "unit".to_string()],
        environment: std::collections::BTreeMap::new(),
        input_identities: Vec::new(),
        expected_outputs: vec!["unit:lib".to_string()],
        limits: mantle_rust_plan_core::RustUnitLimits::default(),
    }
}

#[test]
fn supplied_fact_adapters_return_application_owned_observations() {
    let mut workspace = super::workspace_adapter::SuppliedWorkspaceFactsAdapter {
        facts: mantle_rust_plan_core::StructuralWorkspaceFacts {
            schema: mantle_rust_plan_core::WORKSPACE_FACTS_SCHEMA.to_string(),
            workspace_identity: digest('c'),
            packages: Vec::new(),
        },
    };
    let mut cargo = super::cargo_adapter::SuppliedCargoOracleAdapter {
        facts: mantle_rust_plan_core::CargoOracleFacts {
            metadata_identity: digest('d'),
            unit_graph_identity: digest('e'),
            package_ids: Vec::new(),
            unit_ids: Vec::new(),
        },
    };
    let mut compiler = super::compiler_adapter::SuppliedCompilerInspectionAdapter {
        fact: mantle_rust_plan_core::ToolchainFact {
            compiler_identity: digest('f'),
            compiler_version_identity: digest('1'),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "wasm32-unknown-unknown".to_string(),
        },
    };
    let workspace_observation = workspace
        .load_workspace(mantle_rust_plan::WorkspaceFactsRequest {
            workspace_hint: "fixture".to_string(),
        })
        .unwrap();
    let cargo_observation = cargo
        .capture_oracle(mantle_rust_plan::CargoOracleRequest {
            workspace_identity: workspace_observation.facts.workspace_identity.clone(),
            profile: "release".to_string(),
        })
        .unwrap();
    let compiler_observation = compiler
        .inspect_compiler(mantle_rust_plan::CompilerInspectionRequest {
            compiler_hint: "fixture-rustc".to_string(),
            target_triple: "wasm32-unknown-unknown".to_string(),
        })
        .unwrap();
    assert_eq!(cargo_observation.facts.metadata_identity, digest('d'));
    assert_eq!(compiler_observation.fact.compiler_identity, digest('f'));
}

#[test]
fn cache_adapter_rejects_wrong_publication_effect() {
    let mut cache = super::cache_adapter::InMemoryRustPlanCacheAdapter::default();
    let request_effect = effect();
    let mut outcome_effect = request_effect.clone();
    outcome_effect.effect_id = digest('2');
    let error = cache
        .publish(mantle_rust_plan::CachePublishRequest {
            effect: request_effect,
            outcome: mantle_rust_plan_core::RustUnitOutcome {
                unit_id: outcome_effect.unit_id,
                effect_id: outcome_effect.effect_id,
                status: "success".to_string(),
                artifacts: Vec::new(),
                blocker: None,
                receipt_blake3: digest('3'),
            },
        })
        .unwrap_err();
    assert_eq!(error.capability, mantle_rust_plan::RustPlanCapability::RustCache);
    assert_eq!(cache.publications.len(), 0);
}

#[test]
fn missing_execution_observation_fails_without_process_fallback() {
    let mut executor = super::execution_adapter::ObservedUnitExecutionAdapter::default();
    let error = executor.execute_unit(mantle_rust_plan::UnitExecutionRequest { effect: effect() }).unwrap_err();
    assert_eq!(error.capability, mantle_rust_plan::RustPlanCapability::UnitExecution);
    assert_eq!(error.code, "unit-observation-missing");
}
