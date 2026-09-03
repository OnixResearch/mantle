use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

const DIGEST_HEX_CHARS: usize = 64;

fn digest(byte: char) -> String {
    core::iter::repeat_n(byte, DIGEST_HEX_CHARS).collect()
}

fn workspace_facts() -> mantle_rust_plan_core::StructuralWorkspaceFacts {
    mantle_rust_plan_core::StructuralWorkspaceFacts {
        schema: mantle_rust_plan_core::WORKSPACE_FACTS_SCHEMA.to_string(),
        workspace_identity: digest('a'),
        packages: vec![mantle_rust_plan_core::PackageFact {
            package_id: "app".to_string(),
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            manifest_identity: digest('b'),
            source_identity: digest('c'),
            workspace_member: true,
            targets: vec![mantle_rust_plan_core::TargetFact {
                name: "app".to_string(),
                crate_name: "app".to_string(),
                kind: mantle_rust_plan_core::TargetKind::Library,
                crate_types: vec!["lib".to_string()],
                source_identity: digest('d'),
                edition: "2024".to_string(),
            }],
            dependencies: Vec::new(),
            features: Vec::new(),
        }],
    }
}

fn command(execute_units: bool, use_cargo_oracle: bool) -> crate::RustPlanApplicationCommand {
    crate::RustPlanApplicationCommand {
        workspace_hint: "fixture-workspace".to_string(),
        compiler_hint: "fixture-rustc".to_string(),
        target_triple: "wasm32-unknown-unknown".to_string(),
        request: mantle_rust_plan_core::RustPlanRequest {
            workspace_members: vec!["app".to_string()],
            features: Vec::new(),
            all_features: false,
            no_default_features: false,
            profile: "release".to_string(),
        },
        use_cargo_oracle,
        execute_units,
    }
}

struct WorkspaceStub;

impl crate::WorkspaceFactsPort for WorkspaceStub {
    fn load_workspace(
        &mut self,
        _request: crate::WorkspaceFactsRequest,
    ) -> Result<crate::WorkspaceFactsObservation, crate::RustPlanPortError> {
        Ok(crate::WorkspaceFactsObservation {
            facts: workspace_facts(),
        })
    }
}

struct OracleStub {
    calls: u32,
}

impl crate::CargoOraclePort for OracleStub {
    fn capture_oracle(
        &mut self,
        _request: crate::CargoOracleRequest,
    ) -> Result<crate::CargoOracleObservation, crate::RustPlanPortError> {
        self.calls = self.calls.saturating_add(1);
        Ok(crate::CargoOracleObservation {
            facts: mantle_rust_plan_core::CargoOracleFacts {
                metadata_identity: digest('e'),
                unit_graph_identity: digest('f'),
                package_ids: vec!["app".to_string()],
                unit_ids: vec!["app:lib".to_string()],
            },
        })
    }
}

struct CompilerStub;

impl crate::CompilerInspectionPort for CompilerStub {
    fn inspect_compiler(
        &mut self,
        request: crate::CompilerInspectionRequest,
    ) -> Result<crate::CompilerInspectionObservation, crate::RustPlanPortError> {
        Ok(crate::CompilerInspectionObservation {
            fact: mantle_rust_plan_core::ToolchainFact {
                compiler_identity: digest('1'),
                compiler_version_identity: digest('2'),
                host_triple: "x86_64-unknown-linux-gnu".to_string(),
                target_triple: request.target_triple,
            },
        })
    }
}

struct ExecutorStub {
    calls: u32,
    status_code: i32,
}

impl crate::UnitExecutionPort for ExecutorStub {
    fn execute_unit(
        &mut self,
        request: crate::UnitExecutionRequest,
    ) -> Result<crate::UnitExecutionObservation, crate::RustPlanPortError> {
        self.calls = self.calls.saturating_add(1);
        let artifacts = if self.status_code == 0 {
            vec![mantle_rust_plan_core::ObservedArtifact {
                name: request.effect.expected_outputs[0].clone(),
                digest_blake3: digest('3'),
            }]
        } else {
            Vec::new()
        };
        Ok(crate::UnitExecutionObservation {
            observation: mantle_rust_plan_core::RustUnitObservation {
                effect_id: request.effect.effect_id,
                status_code: self.status_code,
                stdout_blake3: digest('4'),
                stderr_blake3: digest('5'),
                stdout_bytes: 0,
                stderr_bytes: 0,
                artifacts,
            },
        })
    }
}

#[derive(Clone, Copy)]
enum CacheMode {
    Miss,
    Hit,
    Fail,
}

struct CacheStub {
    mode: CacheMode,
    lookups: u32,
    publications: u32,
}

impl crate::RustCachePort for CacheStub {
    fn lookup(
        &mut self,
        request: crate::CacheLookupRequest,
    ) -> Result<crate::CacheLookupObservation, crate::RustPlanPortError> {
        self.lookups = self.lookups.saturating_add(1);
        let (status, artifacts, failure_code) = match self.mode {
            CacheMode::Miss => (mantle_rust_plan_core::CacheObservationStatus::Miss, Vec::new(), None),
            CacheMode::Hit => (
                mantle_rust_plan_core::CacheObservationStatus::Hit,
                vec![mantle_rust_plan_core::ObservedArtifact {
                    name: request.effect.expected_outputs[0].clone(),
                    digest_blake3: digest('6'),
                }],
                None,
            ),
            CacheMode::Fail => (
                mantle_rust_plan_core::CacheObservationStatus::Failed,
                Vec::new(),
                Some("cache-unavailable".to_string()),
            ),
        };
        Ok(crate::CacheLookupObservation {
            observation: mantle_rust_plan_core::CacheObservation {
                effect_id: request.effect.effect_id,
                status,
                artifacts,
                failure_code,
            },
        })
    }

    fn publish(&mut self, _request: crate::CachePublishRequest) -> Result<(), crate::RustPlanPortError> {
        self.publications = self.publications.saturating_add(1);
        Ok(())
    }
}

fn run_with(
    command: crate::RustPlanApplicationCommand,
    oracle: &mut OracleStub,
    executor: &mut ExecutorStub,
    cache: &mut CacheStub,
) -> Result<crate::RustPlanApplicationOutcome, crate::RustPlanApplicationFailure> {
    let mut workspace = WorkspaceStub;
    let mut compiler = CompilerStub;
    let mut ports = crate::RustPlanPortSet {
        workspace: &mut workspace,
        cargo_oracle: oracle,
        compiler: &mut compiler,
        executor,
        cache,
    };
    crate::run(command, &mut ports)
}

#[test]
fn application_executes_declared_effect_and_publishes_success() {
    let mut oracle = OracleStub { calls: 0 };
    let mut executor = ExecutorStub {
        calls: 0,
        status_code: 0,
    };
    let mut cache = CacheStub {
        mode: CacheMode::Miss,
        lookups: 0,
        publications: 0,
    };
    let outcome = run_with(command(true, true), &mut oracle, &mut executor, &mut cache).unwrap();
    assert_eq!(oracle.calls, 1);
    assert_eq!(executor.calls, 1);
    assert_eq!(cache.publications, 1);
    assert_eq!(outcome.unit_outcomes[0].status, "success");
}

#[test]
fn planning_only_does_not_execute_units() {
    let mut oracle = OracleStub { calls: 0 };
    let mut executor = ExecutorStub {
        calls: 0,
        status_code: 0,
    };
    let mut cache = CacheStub {
        mode: CacheMode::Miss,
        lookups: 0,
        publications: 0,
    };
    let outcome = run_with(command(false, false), &mut oracle, &mut executor, &mut cache).unwrap();
    assert_eq!(oracle.calls, 0);
    assert_eq!(executor.calls, 0);
    assert!(outcome.unit_outcomes.is_empty());
    assert!(outcome.execution_blockers.is_empty());
}

#[test]
fn admitted_cache_hit_skips_executor() {
    let mut oracle = OracleStub { calls: 0 };
    let mut executor = ExecutorStub {
        calls: 0,
        status_code: 0,
    };
    let mut cache = CacheStub {
        mode: CacheMode::Hit,
        lookups: 0,
        publications: 0,
    };
    let outcome = run_with(command(true, false), &mut oracle, &mut executor, &mut cache).unwrap();
    assert_eq!(executor.calls, 0);
    assert_eq!(outcome.unit_outcomes[0].status, "reused");
    assert_eq!(cache.publications, 0);
}

#[test]
fn compiler_failure_stops_dependent_execution() {
    let mut oracle = OracleStub { calls: 0 };
    let mut executor = ExecutorStub {
        calls: 0,
        status_code: 7,
    };
    let mut cache = CacheStub {
        mode: CacheMode::Miss,
        lookups: 0,
        publications: 0,
    };
    let outcome = run_with(command(true, false), &mut oracle, &mut executor, &mut cache).unwrap();
    assert_eq!(executor.calls, 1);
    assert_eq!(outcome.unit_outcomes[0].status, "failed");
    assert_eq!(outcome.execution_blockers[0].class, "compiler-execution-failed");
}

#[test]
fn cache_fault_fails_closed_before_execution() {
    let mut oracle = OracleStub { calls: 0 };
    let mut executor = ExecutorStub {
        calls: 0,
        status_code: 0,
    };
    let mut cache = CacheStub {
        mode: CacheMode::Fail,
        lookups: 0,
        publications: 0,
    };
    let outcome = run_with(command(true, false), &mut oracle, &mut executor, &mut cache).unwrap();
    assert_eq!(executor.calls, 0);
    assert_eq!(outcome.execution_blockers[0].class, "cache-capability-failed");
    assert!(outcome.unit_outcomes.is_empty());
}

#[test]
fn mismatched_port_capability_is_not_reclassified() {
    let error = crate::RustPlanPortError::new(crate::RustPlanCapability::CargoOracle, "wrong-port", false);
    assert_eq!(error.capability, crate::RustPlanCapability::CargoOracle);
    assert!(!error.retryable);
}
