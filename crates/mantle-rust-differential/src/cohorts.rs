//! Predeclared five-pair AB/BA ordinary-Cargo replay cohorts, never speedup claims.
use std::time::Instant;

use mantle_rust_differential_core::Cell;
use mantle_rust_differential_core::Operation;
use mantle_rust_differential_core::Verdict;
use mantle_rust_differential_core::waits::*;
use mantle_rust_differential_core::{self as core};
use serde::Deserialize;
use serde::Serialize;

use crate::Result;
use crate::err;
use crate::fixtures;
use crate::process;
use crate::runner::Attempt;
use crate::runner::Options;
use crate::runner::{self};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cohort {
    pub fixture: String,
    pub cache: CacheState,
    pub ledger: LedgerReport,
    pub attempts: Vec<Attempt>,
    pub errors: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cohorts {
    pub schema: String,
    pub options: Options,
    pub cohorts: Vec<Cohort>,
    pub qualified: bool,
    pub policy: String,
}
fn controls(
    options: &Options,
    fixture: &fixtures::Fixture,
    cache: &CacheState,
    instrumentation: &core::Digest,
) -> Vec<Control> {
    [
        "compiler",
        "cargo",
        "flags",
        "source",
        "lock",
        "inputs",
        "configuration",
        "resources",
        "instrumentation",
        "cache",
        "provider",
    ]
    .into_iter()
    .map(|role| Control {
        role: role.into(),
        identity: match role {
            "compiler" | "cargo" => options.toolchain.clone().expect("pinned"),
            "source" | "inputs" => runner::digest(fixture),
            "lock" => runner::digest(&fixture.steps[0].snapshot["Cargo.lock"]),
            "instrumentation" => instrumentation.clone(),
            "cache" => runner::digest(cache),
            "resources" => runner::digest(&(
                2,
                crunch_rustc_wrapper::observed_process::BUILD_MS,
                crunch_rustc_wrapper::observed_process::SCRATCH_BYTES,
                "ambient-affinity-and-page-cache-uncontrolled",
            )),
            "configuration" => runner::digest(&(&options.host, &options.path)),
            "provider" => runner::digest(&"ordinary-Cargo-full-metadata"),
            _ => runner::digest(&"dev-incremental-locked-offline-v1"),
        },
    })
    .collect()
}
/// Diagnostics are identically enabled on A and B. This is repeatability, not an optimization
/// treatment.
pub fn execute(mut options: Options) -> Result<Cohorts> {
    if options.wrapper.is_none() {
        return Err("cohorts-require-explicit-wait-wrapper".into());
    }
    if options.baseline.is_some() {
        return Err("cohorts-use-predeclared-plans-not-coverage-baselines".into());
    }
    if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } != 0 {
        return Err("subreaper-unavailable".into());
    }
    crunch_rustc_wrapper::observed_process::private_dir(&options.root)?;
    runner::pin(&mut options)?;
    let total = Instant::now();
    let mut cohorts = vec![];
    runner::probe_tools(&options, &fixtures::generate("chain")?, total)?;
    let instrumentation = runner::digest(&(
        process::hash_closure(options.wrapper.as_ref().ok_or("wait-wrapper-required")?)?,
        include_str!("../../crunch-rustc-wrapper/src/waits.rs"),
    ));
    for fixture_id in ["chain", "wide"] {
        for cache in [CacheState::Clean, CacheState::Warm, CacheState::Incremental] {
            let fixture = fixtures::generate(fixture_id)?;
            let cell = Cell {
                id: format!("{fixture_id}-cohort"),
                fixture: fixture_id.into(),
                operation: Operation::Build,
                profile: "dev".into(),
                lto: None,
                features: vec![],
                required: true,
                mode: "baseline-replay".into(),
            };
            let cohort_name = format!("{fixture_id}-{}", match cache {
                CacheState::Clean => "clean",
                CacheState::Warm => "warm",
                CacheState::Incremental => "incremental",
            });
            let cohort_root = options.root.join(&cohort_name);
            crunch_rustc_wrapper::observed_process::private_dir(&cohort_root)?;
            let measured_step = if cache == CacheState::Incremental {
                &fixture.steps[3]
            } else {
                &fixture.steps[0]
            };
            let scenario = if cache == CacheState::Clean {
                Scenario::Cold
            } else if cache == CacheState::Warm {
                Scenario::Unchanged
            } else {
                Scenario::FirstEdit
            };
            let mut plans = vec![];
            let mut slots = vec![];
            for pair in 0..5u32 {
                for treatment_b in [pair % 2 == 1, pair % 2 == 0] {
                    let slot = format!("pair-{pair}-{}", if treatment_b { "b" } else { "a" });
                    let session = runner::digest(&(&cohort_name, &slot));
                    let controls = controls(&options, &fixture, &cache, &instrumentation);
                    let base = &fixture.steps[0];
                    let setup_id = runner::digest(&(&cohort_name, &slot, "setup"));
                    plans.push(RunPlan {
                        id: setup_id.clone(),
                        pair: None,
                        treatment_b,
                        scenario: Scenario::Setup,
                        controls: controls.clone(),
                        cache: cache.clone(),
                        before: base.digest.clone(),
                        after: base.digest.clone(),
                        predecessor: None,
                        session: session.clone(),
                        expectation: runner::digest(&base.expected),
                        clean_reference: base.digest.clone(),
                        required: true,
                    });
                    let mut predecessor = setup_id;
                    if cache != CacheState::Clean {
                        let warmup = runner::digest(&(&cohort_name, &slot, "warmup"));
                        plans.push(RunPlan {
                            id: warmup.clone(),
                            pair: None,
                            treatment_b,
                            scenario: Scenario::Warmup,
                            controls: controls.clone(),
                            cache: cache.clone(),
                            before: base.digest.clone(),
                            after: base.digest.clone(),
                            predecessor: Some(predecessor),
                            session: session.clone(),
                            expectation: runner::digest(&base.expected),
                            clean_reference: base.digest.clone(),
                            required: true,
                        });
                        predecessor = warmup;
                    }
                    let id = runner::digest(&(&cohort_name, &slot, "measured"));
                    plans.push(RunPlan {
                        id: id.clone(),
                        pair: Some(pair),
                        treatment_b,
                        scenario: scenario.clone(),
                        controls,
                        cache: cache.clone(),
                        before: base.digest.clone(),
                        after: measured_step.digest.clone(),
                        predecessor: Some(predecessor),
                        session,
                        expectation: runner::digest(&measured_step.expected),
                        clean_reference: measured_step.digest.clone(),
                        required: true,
                    });
                    slots.push((slot, id));
                }
            }
            runner::save(&cohort_root.join("frozen-plans.json"), &plans)?;
            let mut samples = vec![];
            let mut attempts = vec![];
            let mut errors = vec![];
            for (slot, id) in slots {
                let slot_root = cohort_root.join(&slot);
                let start = Instant::now();
                let setup = plans
                    .iter()
                    .find(|p| p.session == runner::digest(&(&cohort_name, &slot)) && p.scenario == Scenario::Setup)
                    .ok_or("setup-plan")?;
                let prepared = runner::prepare_state(&options, &slot_root, &fixture.steps[0].snapshot, total);
                let (state, metadata) = match prepared {
                    Ok(v) => v,
                    Err(e) => {
                        let capture = process::read_bounded(&slot_root.join("metadata/capture.json"), 64 * 1024 * 1024)
                            .ok()
                            .and_then(|bytes| {
                                serde_json::from_slice::<crunch_rustc_wrapper::observed_process::Capture>(&bytes).ok()
                            });
                        let outcome = if capture.as_ref().is_some_and(|c| c.failure.as_deref() == Some("deadline")) {
                            RunOutcome::Timeout
                        } else {
                            RunOutcome::Infrastructure
                        };
                        samples.push(RunSample {
                            plan: setup.id.clone(),
                            invocation: Some(runner::digest(&(&slot_root, "setup"))),
                            outcome,
                            elapsed_ns: Some(crunch_rustc_wrapper::observed_process::ns(start)),
                            equality_qualified: false,
                            work_qualified: false,
                            exclusion: Some(Unavailable::MissingEndpoint),
                        });
                        errors.push(e);
                        continue;
                    }
                };
                samples.push(RunSample {
                    plan: setup.id.clone(),
                    invocation: Some(runner::digest(&(&slot_root, "setup"))),
                    outcome: RunOutcome::Success,
                    elapsed_ns: Some(crunch_rustc_wrapper::observed_process::ns(start)),
                    equality_qualified: true,
                    work_qualified: metadata.code == Some(0),
                    exclusion: None,
                });
                let result = (|| -> Result<()> {
                    if cache != CacheState::Clean {
                        let warmup = runner::attempt(
                            &options,
                            &state,
                            &cell,
                            &fixture.steps[0],
                            "warmup",
                            total,
                            None,
                            crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                        )?;
                        let plan = plans
                            .iter()
                            .find(|p| p.session == setup.session && p.scenario == Scenario::Warmup)
                            .ok_or("warmup-plan")?;
                        let valid = core::validate_oracle(&warmup.observation, &fixture.steps[0].expected).is_ok();
                        samples.push(RunSample {
                            plan: plan.id.clone(),
                            invocation: Some(runner::digest(&warmup.root)),
                            outcome: runner::outcome(warmup.observation.status),
                            elapsed_ns: Some(warmup.cargo.elapsed_ns),
                            equality_qualified: valid,
                            work_qualified: valid,
                            exclusion: (!valid).then_some(Unavailable::EqualityUnqualified),
                        });
                        attempts.push(warmup);
                        if !valid {
                            return Err("warmup-correctness-failed".into());
                        }
                    }
                    if cache == CacheState::Incremental {
                        process::apply(&state.source, &fixture.steps[0].snapshot, measured_step)?;
                    }
                    let observed = runner::attempt(
                        &options,
                        &state,
                        &cell,
                        measured_step,
                        "measured",
                        total,
                        None,
                        crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                    )?;
                    samples.push(RunSample {
                        plan: id.clone(),
                        invocation: Some(runner::digest(&observed.root)),
                        outcome: runner::outcome(observed.observation.status),
                        elapsed_ns: Some(observed.cargo.elapsed_ns),
                        equality_qualified: false,
                        work_qualified: false,
                        exclusion: Some(Unavailable::MissingEndpoint),
                    });
                    let observed_index = attempts.len();
                    attempts.push(observed);
                    let clean_root = slot_root.join("clean-reference");
                    let (clean_state, _) =
                        runner::prepare_state(&options, &clean_root, &measured_step.snapshot, total)?;
                    let clean = runner::attempt(
                        &options,
                        &clean_state,
                        &cell,
                        measured_step,
                        "compile",
                        total,
                        None,
                        crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                    )?;
                    let observed = &attempts[observed_index];
                    let valid = core::compare(&observed.observation, &clean.observation, &measured_step.expected)
                        == Verdict::Equivalent;
                    let work = valid
                        && !observed.artifacts.is_empty()
                        && observed.diagnostic.as_ref().is_some_and(|d| {
                            !d.report.sink_failed
                                && d.publication_error.is_none()
                                && d.report.reduction_error.is_none()
                                && d.report.timeline.complete
                                && d.report
                                    .timeline
                                    .attempts
                                    .iter()
                                    .all(|a| a.role == ProcessRole::ToolProbe || a.unit.is_some())
                        });
                    let sample = samples.iter_mut().find(|s| s.plan == id).ok_or("measured-sample-missing")?;
                    sample.equality_qualified = valid;
                    sample.work_qualified = work;
                    sample.exclusion = if !valid {
                        Some(Unavailable::EqualityUnqualified)
                    } else if !work {
                        Some(Unavailable::WorkUnqualified)
                    } else {
                        None
                    };
                    let teardown = observed.cargo.teardown_complete && clean.cargo.teardown_complete;
                    attempts.push(clean);
                    if teardown {
                        process::remove_owned_tree(&clean_state.target)?;
                        process::remove_owned_tree(&state.target)?;
                    }
                    Ok(())
                })();
                if let Err(e) = result {
                    errors.push(e);
                }
            }
            let ledger = reduce_ledger(&plans, &samples).map_err(err)?;
            let cohort = Cohort {
                fixture: fixture_id.into(),
                cache: cache.clone(),
                ledger,
                attempts,
                errors,
            };
            runner::save(&cohort_root.join("cohort.json"), &cohort)?;
            cohorts.push(cohort);
        }
    }
    let qualified = cohorts.iter().all(|c| c.ledger.comparison_qualified && c.errors.is_empty());
    let report=Cohorts{schema:"mantle-rust-wait-cohorts-v1".into(),options,cohorts,qualified,policy:"five alternating AB/BA same-instrumentation baseline replay pairs per fixture/cache; median/min/max; no percentiles; OS page cache and affinity uncontrolled; no speedup claim".into()};
    runner::save(&report.options.root.join("cohorts.json"), &report)?;
    Ok(report)
}
