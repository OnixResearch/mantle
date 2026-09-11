//! Application fixtures with in-memory port fakes.

use mantle_rust_plan_app::AdapterError;
use mantle_rust_plan_app::ApplicationOutcome;
use mantle_rust_plan_app::CacheDisposition;
use mantle_rust_plan_app::CacheLookup;
use mantle_rust_plan_app::CacheLookupRequest;
use mantle_rust_plan_app::CargoOracleCapture;
use mantle_rust_plan_app::CompilerFacts;
use mantle_rust_plan_app::CompilerInspection;
use mantle_rust_plan_app::CompilerInspectionRequest;
use mantle_rust_plan_app::OracleCaptureRequest;
use mantle_rust_plan_app::OracleFacts;
use mantle_rust_plan_app::RustCacheAccess;
use mantle_rust_plan_app::RustPlanApplication;
use mantle_rust_plan_app::UnitDisposition;
use mantle_rust_plan_app::UnitExecutor;
use mantle_rust_plan_app::WorkspaceFactsRequest;
use mantle_rust_plan_app::WorkspaceFactsSource;
use mantle_rust_plan_app::WorkspaceFactsView;
use mantle_rust_plan_core::BuildProfile;
use mantle_rust_plan_core::PackageFacts;
use mantle_rust_plan_core::PackageSource;
use mantle_rust_plan_core::PackageSourceKind;
use mantle_rust_plan_core::PlanExecutionOutcome;
use mantle_rust_plan_core::TargetFacts;
use mantle_rust_plan_core::TargetKind;
use mantle_rust_plan_core::UnitEffect;
use mantle_rust_plan_core::UnitObservation;
use mantle_rust_plan_core::UnitObservationStatus;

fn lib_target(name: &str) -> TargetFacts {
    TargetFacts {
        name: String::from(name),
        kind: TargetKind::Lib,
        crate_types: vec![String::from("rlib")],
        required_features: Vec::new(),
    }
}

fn package(name: &str) -> PackageFacts {
    PackageFacts {
        name: String::from(name),
        version: String::from("0.1.0"),
        source: PackageSource {
            kind: PackageSourceKind::WorkspaceMember,
            identity: String::from("workspace"),
        },
        targets: vec![lib_target(name)],
        dependencies: Vec::new(),
        features: Vec::new(),
        links: None,
    }
}

/// Workspace facts fake.
struct FakeWorkspace {
    fail: bool,
    calls: usize,
}

impl WorkspaceFactsSource for FakeWorkspace {
    fn load_workspace_facts(&mut self, _request: &WorkspaceFactsRequest) -> Result<WorkspaceFactsView, AdapterError> {
        self.calls = self.calls.saturating_add(1);
        if self.fail {
            return Err(AdapterError::new("workspace-facts-unavailable", "fixture failure"));
        }
        Ok(WorkspaceFactsView {
            packages: vec![package("app"), package("core")],
            feature_requests: Vec::new(),
        })
    }
}

/// Oracle fake.
struct FakeOracle {
    calls: usize,
}

impl CargoOracleCapture for FakeOracle {
    fn capture_oracle(&mut self, _request: &OracleCaptureRequest) -> Result<OracleFacts, AdapterError> {
        self.calls = self.calls.saturating_add(1);
        Ok(OracleFacts {
            oracle_identity: String::from("fixture-oracle"),
            selected_packages: vec![String::from("app@0.1.0")],
        })
    }
}

/// Compiler inspection fake.
struct FakeCompiler {
    calls: usize,
}

impl CompilerInspection for FakeCompiler {
    fn inspect_compiler(&mut self, _request: &CompilerInspectionRequest) -> Result<CompilerFacts, AdapterError> {
        self.calls = self.calls.saturating_add(1);
        Ok(CompilerFacts {
            compiler_identity: String::from("fixture-rustc"),
            target_triple: String::from("x86_64-unknown-linux-gnu"),
        })
    }
}

/// Cache fake with a configurable hit set.
struct FakeCache {
    hits: u32,
    remaining_hits: u32,
    fail: bool,
    calls: usize,
}

impl RustCacheAccess for FakeCache {
    fn lookup_unit(&mut self, _request: &CacheLookupRequest) -> Result<CacheLookup, AdapterError> {
        self.calls = self.calls.saturating_add(1);
        if self.fail {
            return Err(AdapterError::new("cache-unavailable", "fixture cache failure"));
        }
        if self.remaining_hits > 0 {
            self.remaining_hits = self.remaining_hits.saturating_sub(1);
            self.hits = self.hits.saturating_add(1);
            return Ok(CacheLookup::Cached {
                output_identity: String::from("fixture-output"),
            });
        }
        Ok(CacheLookup::NotCached)
    }
}

/// Executor fake with configurable observation behavior.
struct FakeExecutor {
    calls: usize,
    fail: bool,
    fail_first_observation: bool,
    substitute_unknown_effect: bool,
}

impl UnitExecutor for FakeExecutor {
    fn execute_unit(&mut self, effect: &UnitEffect) -> Result<UnitObservation, AdapterError> {
        self.calls = self.calls.saturating_add(1);
        if self.fail {
            return Err(AdapterError::new("unit-execution-unavailable", "fixture executor failure"));
        }
        let is_first = self.calls == 1;
        let status = if self.fail_first_observation && is_first {
            UnitObservationStatus::Failed
        } else {
            UnitObservationStatus::Succeeded
        };
        let effect_id = if self.substitute_unknown_effect {
            mantle_rust_plan_core::EffectId(String::from("effect:unknown"))
        } else {
            effect.effect_id.clone()
        };
        Ok(UnitObservation {
            effect_id,
            unit_id: effect.unit_id.clone(),
            status,
            exit_code: Some(if status == UnitObservationStatus::Succeeded {
                0
            } else {
                101
            }),
            diagnostics_code: None,
        })
    }
}

#[test]
fn successful_run_executes_every_unit_and_builds_a_receipt() {
    let mut app = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: false,
            substitute_unknown_effect: false,
        },
    );
    let outcome = app.run(vec![String::from("app")], BuildProfile::Release, true).expect("ports succeed");
    let ApplicationOutcome::Executed(executed) = outcome else {
        panic!("expected an executed outcome")
    };
    let receipt_preimage = executed.receipt_preimage;
    let execution = executed.execution;
    let cache = executed.cache;
    let dispositions = executed.dispositions;
    let observations = executed.observations;
    let plan = executed.plan;
    assert_eq!(execution, PlanExecutionOutcome::Completed);
    assert_eq!(cache, CacheDisposition::AllMissed);
    assert!(receipt_preimage.is_some());
    assert_eq!(dispositions.len(), plan.effects.len());
    assert!(observations.iter().all(|observation| observation.exit_code == Some(0)));
}

#[test]
fn cached_units_are_not_executed_and_are_reported() {
    let mut app = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 1,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: false,
            substitute_unknown_effect: false,
        },
    );
    let outcome = app.run(vec![String::from("app")], BuildProfile::Dev, false).expect("ports succeed");
    let ApplicationOutcome::Executed(executed) = outcome else {
        panic!("expected an executed outcome")
    };
    let cache = executed.cache;
    let dispositions = executed.dispositions;
    let observations = executed.observations;
    let execution = executed.execution;
    assert_eq!(cache, CacheDisposition::PartiallyHit);
    assert_eq!(execution, PlanExecutionOutcome::Completed);
    assert_eq!(dispositions.iter().filter(|d| **d == UnitDisposition::CacheHit).count(), 1);
    assert!(observations.iter().any(|observation| observation.diagnostics_code.as_deref() == Some("cache-hit")));
}

#[test]
fn blocked_plans_perform_no_work() {
    let mut app = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: false,
            substitute_unknown_effect: false,
        },
    );
    let outcome = app.run(Vec::new(), BuildProfile::Dev, false).expect("ports succeed");
    let ApplicationOutcome::Blocked { blockers } = outcome else {
        panic!("expected a blocked outcome")
    };
    assert!(blockers.iter().any(|blocker| blocker.code == "missing-roots"));
}

#[test]
fn adapter_failures_propagate_as_typed_errors() {
    let mut app = RustPlanApplication::new(
        FakeWorkspace { fail: true, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: false,
            substitute_unknown_effect: false,
        },
    );
    let error = app
        .run(vec![String::from("app")], BuildProfile::Dev, false)
        .expect_err("workspace failure must propagate");
    assert_eq!(error.code, "workspace-facts-unavailable");

    let mut app = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: true,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: false,
            substitute_unknown_effect: false,
        },
    );
    let error = app
        .run(vec![String::from("app")], BuildProfile::Dev, false)
        .expect_err("cache failure must propagate");
    assert_eq!(error.code, "cache-unavailable");

    let mut app = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: true,
            fail_first_observation: false,
            substitute_unknown_effect: false,
        },
    );
    let error = app
        .run(vec![String::from("app")], BuildProfile::Dev, false)
        .expect_err("executor failure must propagate");
    assert_eq!(error.code, "unit-execution-unavailable");
}

#[test]
fn failed_and_substituted_observations_classify_exactly() {
    let mut failing = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: true,
            substitute_unknown_effect: false,
        },
    );
    let outcome = failing.run(vec![String::from("app")], BuildProfile::Dev, false).expect("ports succeed");
    let ApplicationOutcome::Executed(executed) = outcome else {
        panic!("expected an executed outcome")
    };
    let execution = executed.execution;
    assert_eq!(execution, PlanExecutionOutcome::Failed { failed_effect_count: 1 });

    let mut substituted = RustPlanApplication::new(
        FakeWorkspace { fail: false, calls: 0 },
        FakeOracle { calls: 0 },
        FakeCompiler { calls: 0 },
        FakeCache {
            hits: 0,
            remaining_hits: 0,
            fail: false,
            calls: 0,
        },
        FakeExecutor {
            calls: 0,
            fail: false,
            fail_first_observation: false,
            substitute_unknown_effect: true,
        },
    );
    let outcome = substituted.run(vec![String::from("app")], BuildProfile::Dev, false).expect("ports succeed");
    let ApplicationOutcome::Executed(executed) = outcome else {
        panic!("expected an executed outcome")
    };
    let execution = executed.execution;
    assert!(matches!(execution, PlanExecutionOutcome::Rejected { .. }));
}
