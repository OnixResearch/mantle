//! Direct-rustc whole-unit baseline for the frozen small fixtures. No provider or
//! early-metadata protocol is implemented here; every dependency finishes first.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crunch_rustc_wrapper::observed_process::Capture;
use crunch_rustc_wrapper::waits::DiagnosticOutcome;
use crunch_rustc_wrapper::waits::NativeWaitCollector;
use mantle_rust_differential_core::Cell;
use mantle_rust_differential_core::Coverage;
use mantle_rust_differential_core::CoverageSummary;
use mantle_rust_differential_core::Digest;
use mantle_rust_differential_core::Identity;
use mantle_rust_differential_core::Manifest;
use mantle_rust_differential_core::Observation;
use mantle_rust_differential_core::Operation;
use mantle_rust_differential_core::Status;
use mantle_rust_differential_core::UnitRecords;
use mantle_rust_differential_core::Verdict;
use mantle_rust_differential_core::waits::ProcessRole;
use mantle_rust_differential_core::waits::UnitIdentity;
use mantle_rust_differential_core::waits::{self};
use mantle_rust_differential_core::{self as core};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::Result;
use crate::err;
use crate::fixtures;
use crate::fixtures::Fixture;
use crate::fixtures::Step;
use crate::process;
use crate::runner;
use crate::runner::Artifact;
use crate::runner::Options;
use crate::runner::State;

const MODE: &str = "native-serial-whole-unit-incremental-v1";
const OUTPUT: &str = "native-current-operation-semantic-output-v2";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Compilation {
    pub unit: String,
    pub capture: Capture,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attempt {
    pub root: PathBuf,
    pub source: Digest,
    pub compilations: Vec<Compilation>,
    pub build_script: Option<Capture>,
    pub program: Option<Capture>,
    pub artifacts: Vec<Artifact>,
    pub observation: Observation,
    pub diagnostic: Option<DiagnosticOutcome>,
    pub diagnostic_errors: Vec<String>,
    pub errors: Vec<String>,
    pub planned_units: Vec<String>,
    pub unrun_units: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepReport {
    pub cell: String,
    pub side: String,
    pub step: String,
    pub predecessor: Option<Digest>,
    pub session: Digest,
    pub campaign: Attempt,
    pub clean: Attempt,
    pub verdict: Verdict,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema: String,
    pub options: Options,
    pub manifest: Manifest,
    pub baseline: Manifest,
    pub fixtures: Vec<Fixture>,
    pub probes: Vec<Capture>,
    pub steps: Vec<StepReport>,
    pub replay: Vec<(String, String, Verdict)>,
    pub coverage: Vec<Coverage>,
    pub population: std::result::Result<CoverageSummary, core::Error>,
    pub errors: Vec<String>,
    pub complete: bool,
    pub policy: String,
}

fn status(capture: &Capture) -> Status {
    if capture.failure.as_deref() == Some("deadline") {
        Status::Timeout
    } else if !capture.complete || !capture.teardown_complete {
        Status::Infrastructure
    } else if capture.signal.is_some() {
        Status::Crash
    } else if capture.code == Some(0) {
        Status::Success
    } else if capture.code == Some(1) {
        Status::Rejected
    } else {
        Status::Infrastructure
    }
}

fn native_cells() -> Vec<Cell> {
    ["chain", "wide"]
        .into_iter()
        .flat_map(|fixture| {
            [Operation::Build, Operation::Check].into_iter().map(move |operation| Cell {
                id: format!(
                    "serial-{fixture}-{}",
                    if operation == Operation::Build {
                        "build"
                    } else {
                        "check"
                    }
                ),
                fixture: fixture.into(),
                operation,
                profile: "dev".into(),
                lto: None,
                features: vec![],
                required: true,
                mode: MODE.into(),
            })
        })
        .collect()
}

struct UnitSpec {
    name: String,
    source: String,
    crate_type: &'static str,
    dependencies: Vec<String>,
    host: bool,
}
fn units(fixture: &Fixture) -> Vec<UnitSpec> {
    let count = if fixture.id == "wide" { 8 } else { 4 };
    let mut units = vec![
        UnitSpec {
            name: "fixture-macro".into(),
            source: "fixture-macro/src/lib.rs".into(),
            crate_type: "proc-macro",
            dependencies: vec![],
            host: true,
        },
        UnitSpec {
            name: "lib0-build-script".into(),
            source: "lib0/build.rs".into(),
            crate_type: "bin",
            dependencies: vec![],
            host: true,
        },
    ];
    for n in 0..count {
        units.push(UnitSpec {
            name: format!("lib{n}"),
            source: format!("lib{n}/src/lib.rs"),
            crate_type: "rlib",
            host: false,
            dependencies: if n == 0 {
                vec!["fixture-macro".into(), "lib0-build-script".into()]
            } else if fixture.id == "wide" {
                vec![]
            } else {
                vec![format!("lib{}", n - 1)]
            },
        });
    }
    units.push(UnitSpec {
        name: "fixture-app".into(),
        source: "fixture-app/src/main.rs".into(),
        crate_type: "bin",
        host: false,
        dependencies: (0..count).map(|n| format!("lib{n}")).collect(),
    });
    units
}

fn attempt(
    options: &Options,
    state: &State,
    fixture: &Fixture,
    cell: &Cell,
    step: &Step,
    id: &str,
    total: Instant,
) -> Result<Attempt> {
    process::verify(&state.source, &step.snapshot)?;
    let root = state.root.join(id);
    crunch_rustc_wrapper::observed_process::private_dir(&root)?;
    // Step-owned final outputs prevent a previous successful binary from satisfying
    // a failed current compile. Incremental scratch alone carries across steps.
    let outputs = state.target.join(format!("outputs-{id}"));
    let generated = outputs.join("generated");
    crunch_rustc_wrapper::observed_process::private_dir(&outputs)?;
    crunch_rustc_wrapper::observed_process::private_dir(&generated)?;
    let invocation = runner::digest(&(&root, "native-invocation"));
    let clock = runner::digest(&(&invocation, "monotonic-clock"));
    let mut diagnostic_errors = vec![];
    let mut collector = if options.diagnostics {
        match NativeWaitCollector::start(&root, invocation.clone(), clock.clone()) {
            Ok(c) => Some(c),
            Err(_) => {
                diagnostic_errors.push("native-diagnostic-start-unavailable".into());
                None
            }
        }
    } else {
        None
    };
    let specs = units(fixture);
    let mut result = Attempt {
        root: root.clone(),
        source: step.digest.clone(),
        compilations: vec![],
        build_script: None,
        program: None,
        artifacts: vec![],
        diagnostic: None,
        diagnostic_errors: vec![],
        errors: vec![],
        planned_units: specs.iter().map(|u| u.name.clone()).collect(),
        unrun_units: vec![],
        observation: Observation {
            status: Status::Success,
            units: vec![],
            program_stdout: None,
            program_stderr: None,
            accepted_outputs: vec![],
            complete: true,
            teardown_complete: true,
            ice: false,
        },
    };
    let mut paths = BTreeMap::<String, PathBuf>::new();
    let mut unit_ids = BTreeMap::<String, Digest>::new();
    let execution = (|| -> Result<()> {
        for spec in &specs {
            if result.observation.status != Status::Success {
                break;
            }
            let unit_root = root.join(&spec.name);
            crunch_rustc_wrapper::observed_process::private_dir(&unit_root)?;
            let incremental = state.target.join(format!("incremental-{}", spec.name));
            if !incremental.exists() {
                crunch_rustc_wrapper::observed_process::private_dir(&incremental)?;
            }
            let metadata_only = cell.operation == Operation::Check && !spec.host;
            let output = outputs.join(if spec.crate_type == "proc-macro" {
                "libfixture_macro.so".into()
            } else if spec.crate_type == "rlib" {
                format!("lib{}.{}", spec.name, if metadata_only { "rmeta" } else { "rlib" })
            } else {
                spec.name.clone()
            });
            let mut command = Command::new(&options.rustc);
            command.current_dir(&state.source).env_clear().envs(runner::environment(options, state));
            command.env("OUT_DIR", &generated);
            command.args([
                "--edition=2021",
                "--error-format=json",
                "--crate-name",
                &spec.name.replace('-', "_"),
                "--crate-type",
                spec.crate_type,
            ]);
            command
                .arg(&spec.source)
                .arg("--sysroot")
                .arg(&options.sysroot)
                .arg("-C")
                .arg(format!("linker={}", options.linker.display()))
                .arg("-Cstrip=debuginfo")
                .arg("-C")
                .arg(format!("incremental={}", incremental.display()))
                .arg("--remap-path-prefix")
                .arg(format!("{}=/mantle-fixture", state.source.display()))
                .arg("--remap-path-prefix")
                .arg(format!("{}=/mantle-target", state.target.display()))
                .arg("--remap-path-prefix")
                .arg(format!("{}=/mantle-attempt", root.display()))
                .arg("-L")
                .arg(format!("dependency={}", outputs.display()))
                .arg("--emit")
                .arg(if metadata_only { "metadata" } else { "link" })
                .arg("-o")
                .arg(&output);
            for dependency in &spec.dependencies {
                if dependency != "lib0-build-script" {
                    command.arg("--extern").arg(format!(
                        "{}={}",
                        dependency.replace('-', "_"),
                        paths.get(dependency).ok_or("native-dependency-unavailable")?.display()
                    ));
                }
            }
            let mut observed_attempt = None;
            if let Some(c) = &mut collector {
                let mut args = json!(command.get_args().map(|a| a.to_string_lossy().into_owned()).collect::<Vec<_>>());
                runner::normalize(&mut args, state, options);
                let role = if spec.host {
                    ProcessRole::HostCompiler
                } else {
                    ProcessRole::TargetCompiler
                };
                let facts = UnitIdentity {
                    package: runner::digest(&(&fixture.id, &spec.name)),
                    source: step.digest.clone(),
                    target: runner::digest(&spec.name),
                    target_kind: runner::digest(&spec.crate_type),
                    host: runner::digest(&options.host),
                    target_triple: runner::digest(&options.host),
                    role: role.clone(),
                    features: vec![],
                    profile: runner::digest(&cell.profile),
                    mode: runner::digest(&(MODE, cell.operation)),
                    compiler: options.toolchain.clone().ok_or("unpinned-toolchain")?,
                    arguments: runner::digest(&args),
                    inputs: step.digest.clone(),
                    dependencies: spec
                        .dependencies
                        .iter()
                        .filter_map(|name| unit_ids.get(name))
                        .map(|id| waits::Dependency {
                            unit: id.clone(),
                            role: waits::ArtifactRole::CompletionObserved,
                        })
                        .collect(),
                };
                let process_identity = runner::digest(&(&options.rustc, &args, &root, &spec.name));
                match c.register_unit(facts).and_then(|unit| {
                    unit_ids.insert(spec.name.clone(), unit.clone());
                    c.register_attempt(waits::Attempt {
                        id: runner::digest(&"unassigned"),
                        invocation: invocation.clone(),
                        clock: clock.clone(),
                        unit: Some(unit),
                        role,
                        number: 1,
                        resolved_process: process_identity,
                        prior: None,
                        unmapped: None,
                    })
                }) {
                    Ok(id) => observed_attempt = Some(id),
                    Err(_) => {
                        c.observation_lost();
                        diagnostic_errors.push("native-diagnostic-registration-incomplete".into());
                    }
                }
            }
            let mut observer = |event| {
                if let Some(c) = &mut collector {
                    c.process_event(observed_attempt.as_ref(), event);
                }
            };
            let capture = crunch_rustc_wrapper::observed_process::supervise_observed(
                command,
                crunch_rustc_wrapper::observed_process::Supervision {
                    root: &unit_root,
                    scratch: &options.root,
                    timeout_ms: crunch_rustc_wrapper::observed_process::BUILD_MS,
                    total_start: total,
                    injection: None,
                    stream_bound: crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                },
                &mut observer,
            )
            .inspect_err(|_| result.observation.teardown_complete = false)?;
            runner::save(&unit_root.join("capture.json"), &capture)?;
            result.observation.status = status(&capture);
            result.observation.complete &= capture.complete;
            result.observation.teardown_complete &= capture.teardown_complete;
            result.compilations.push(Compilation {
                unit: spec.name.clone(),
                capture,
            });
            let capture = &result.compilations.last().ok_or("native-missing-capture")?.capture;
            let mut records = vec![];
            for line in std::str::from_utf8(&capture.stderr).map_err(err)?.lines() {
                let mut record: Value =
                    serde_json::from_str(line).map_err(|e| format!("native-diagnostic-json:{e}"))?;
                if record["$message_type"] != "diagnostic" {
                    return Err("native-unknown-rustc-record".into());
                }
                record.as_object_mut().ok_or("native-diagnostic-object")?.remove("$message_type");
                runner::normalize(&mut record, state, options);
                result.observation.ice |= record["level"] == "ice"
                    || record["message"].as_str().is_some_and(|m| m.contains("internal compiler error"));
                records.push(record);
                if records.len() > core::MAX_DIAGNOSTICS {
                    return Err("native-diagnostic-bound".into());
                }
            }
            if !records.is_empty() {
                result.observation.units.push(UnitRecords {
                    unit: spec.name.clone(),
                    records,
                });
            }
            if !capture.stdout.is_empty() {
                result.errors.push("unexpected-native-compiler-stdout".into());
            }
            if result.observation.status != Status::Success {
                break;
            }
            let bytes = process::read_bounded(&output, 128 * 1024 * 1024)?;
            result.artifacts.push(Artifact {
                unit: spec.name.clone(),
                role: vec![spec.crate_type.into()],
                path: output.clone(),
                digest: Digest::from_slice(&bytes),
                bytes: bytes.len() as u64,
                fresh: false,
                executable: spec.crate_type == "bin" && !metadata_only,
            });
            paths.insert(spec.name.clone(), output.clone());
            if spec.name == "lib0-build-script" {
                let script_root = root.join("build-script-execution");
                crunch_rustc_wrapper::observed_process::private_dir(&script_root)?;
                let mut script = Command::new(&output);
                script
                    .current_dir(state.source.join("lib0"))
                    .env_clear()
                    .envs(runner::environment(options, state))
                    .env("OUT_DIR", &generated);
                let mut observer = |event| {
                    if let Some(c) = &mut collector {
                        c.process_event(None, event);
                    }
                };
                let capture = crunch_rustc_wrapper::observed_process::supervise_observed(
                    script,
                    crunch_rustc_wrapper::observed_process::Supervision {
                        root: &script_root,
                        scratch: &options.root,
                        timeout_ms: crunch_rustc_wrapper::observed_process::PROGRAM_MS,
                        total_start: total,
                        injection: None,
                        stream_bound: crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                    },
                    &mut observer,
                )
                .inspect_err(|_| result.observation.teardown_complete = false)?;
                runner::save(&script_root.join("capture.json"), &capture)?;
                result.observation.teardown_complete &= capture.teardown_complete;
                if status(&capture) != Status::Success
                    || capture.stdout != b"cargo:rerun-if-changed=input.txt\n"
                    || !capture.stderr.is_empty()
                {
                    result.errors.push("native-build-script-failed".into());
                }
                let expected = format!(
                    "pub fn generated()->i64 {{{}}}",
                    std::str::from_utf8(&step.snapshot["lib0/input.txt"]).map_err(err)?.trim()
                );
                if process::read_bounded(&generated.join("generated.rs"), process::SOURCE_BYTES)? != expected.as_bytes()
                {
                    result.errors.push("native-generated-input-mismatch".into());
                }
                result.build_script = Some(capture);
                if !result.errors.is_empty() {
                    break;
                }
            }
        }
        if !result.errors.is_empty() {
            result.observation.status = Status::Infrastructure;
            result.observation.complete = false;
        }
        if result.observation.status == Status::Success && cell.operation == Operation::Build {
            let program_root = root.join("program");
            crunch_rustc_wrapper::observed_process::private_dir(&program_root)?;
            let mut program = Command::new(paths.get("fixture-app").ok_or("native-current-executable-unavailable")?);
            program.current_dir(&state.source).env_clear().envs(runner::environment(options, state));
            let mut observer = |event| {
                if let Some(c) = &mut collector {
                    c.process_event(None, event);
                }
            };
            let capture = crunch_rustc_wrapper::observed_process::supervise_observed(
                program,
                crunch_rustc_wrapper::observed_process::Supervision {
                    root: &program_root,
                    scratch: &options.root,
                    timeout_ms: crunch_rustc_wrapper::observed_process::PROGRAM_MS,
                    total_start: total,
                    injection: None,
                    stream_bound: crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                },
                &mut observer,
            )
            .inspect_err(|_| result.observation.teardown_complete = false)?;
            runner::save(&program_root.join("capture.json"), &capture)?;
            if status(&capture) != Status::Success {
                result.observation.status = Status::Infrastructure;
                result.observation.complete = false;
            }
            result.observation.program_stdout = Some(capture.stdout.clone());
            result.observation.program_stderr = Some(capture.stderr.clone());
            result.observation.teardown_complete &= capture.teardown_complete;
            result.program = Some(capture);
        }
        process::verify(&state.source, &step.snapshot)?;
        Ok(())
    })();
    if let Err(error) = execution {
        result.errors.push(error);
        result.observation.status = Status::Infrastructure;
        result.observation.complete = false;
    }
    result.unrun_units = result
        .planned_units
        .iter()
        .filter(|name| !result.compilations.iter().any(|u| &u.unit == *name))
        .cloned()
        .collect();
    if result.observation.status == Status::Success {
        result.observation.accepted_outputs.push(Identity {
            role: OUTPUT.into(),
            digest: runner::digest(&(
                &step.digest,
                &options.toolchain,
                cell.operation,
                &result.planned_units,
                &result.observation.units,
                &result.observation.program_stdout,
                &result.observation.program_stderr,
            )),
        });
    }
    if let Some(c) = collector {
        result.diagnostic = Some(c.finish(runner::outcome(result.observation.status)));
    }
    result.diagnostic_errors = diagnostic_errors;
    runner::save(&root.join("attempt.json"), &result)?;
    Ok(result)
}

/// Same-lane A/A replay plus independently materialized clean references. No
/// Cargo/native artifact-byte or timing equivalence is inferred from this lane.
pub fn execute(mut options: Options) -> Result<Report> {
    if options.wrapper.is_some() {
        return Err("native-serial-uses-diagnostics-option-not-Cargo-wrapper".into());
    }
    if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } != 0 {
        return Err("subreaper-unavailable".into());
    }
    crunch_rustc_wrapper::observed_process::private_dir(&options.root)?;
    let total = Instant::now();
    runner::pin(&mut options)?;
    let mut fixtures = ["chain", "wide"].into_iter().map(fixtures::generate).collect::<Result<Vec<_>>>()?;
    for fixture in &mut fixtures {
        fixture.packages.push("lib0-build-script".into());
    }
    runner::save(&options.root.join("fixture-recipe.json"), &fixtures)?;
    let probes = runner::probe_tools(&options, &fixtures[0], total)?;
    let mut manifest = runner::freeze_cells(&options, &fixtures, &probes, native_cells(), MODE, false)?;
    for id in &mut manifest.identities {
        if id.role == "cache" {
            id.digest = runner::digest(&"native-side-private-rustc-incremental-no-result-cache-v1");
        }
        if id.role == "comparison" {
            id.digest = runner::digest(&(core::POLICY, OUTPUT));
        }
    }
    runner::save(&options.root.join("manifest.json"), &manifest)?;
    let baseline = match &options.baseline {
        Some(path) => Manifest::decode(&process::read_bounded(path, core::MAX_MANIFEST_BYTES as u64)?).map_err(err)?,
        None => manifest.clone(),
    };
    runner::save(&options.root.join("coverage-baseline.json"), &baseline)?;
    let mut report = Report {
        schema: "mantle-rust-serial-differential-v1".into(), options: options.clone(), coverage: runner::initial_coverage(&manifest),
        manifest, baseline, fixtures, probes, steps: vec![], replay: vec![], population: Err(core::Error::Missing), errors: vec![], complete: false,
        policy: "Same-lane serial full-unit baseline only; all regular units compile on every step with retained private rustc incremental scratch. No disk-result cache or resident provider exists. Completion-only diagnostics: live first error, metadata readiness, ready queue, pause/resume and resident queries unavailable. Unit capture resources have separate clocks and are not summed into a session peak. Semantic accepted-output projection excludes raw artifact bytes, retained separately; no cross-adapter artifact equivalence, sandbox, release, self-hosting, speedup or scheduling/cache/publication authority.".into(),
    };
    let execution = (|| -> Result<()> {
        for cell in report.manifest.cells.clone() {
            let fixture =
                report.fixtures.iter().find(|f| f.id == cell.fixture).ok_or("native-fixture-missing")?.clone();
            let cell_root = options.root.join(&cell.id);
            crunch_rustc_wrapper::observed_process::private_dir(&cell_root)?;
            let mut failed = false;
            for side in ["a", "b"] {
                let (state, metadata) =
                    runner::prepare_state(&options, &cell_root.join(side), &fixture.steps[0].snapshot, total)?;
                report.probes.push(metadata);
                let mut previous = &fixture.steps[0].snapshot;
                for (index, step) in fixture.steps.iter().enumerate() {
                    if index != 0 {
                        process::apply(&state.source, previous, step)?;
                    }
                    previous = &step.snapshot;
                    let campaign =
                        attempt(&options, &state, &fixture, &cell, step, &format!("step-{index:02}"), total)?;
                    if !campaign.observation.teardown_complete
                        || !matches!(campaign.observation.status, Status::Success | Status::Rejected)
                    {
                        return Err("native-campaign-stopped-after-incomplete-attempt".into());
                    }
                    let (clean_state, metadata) = runner::prepare_state(
                        &options,
                        &state.root.join(format!("clean-{index:02}")),
                        &step.snapshot,
                        total,
                    )?;
                    report.probes.push(metadata);
                    let clean = attempt(&options, &clean_state, &fixture, &cell, step, "compile", total)?;
                    let verdict =
                        core::compare(&campaign.observation, &clean.observation, &runner::expected(&cell, step));
                    failed |= verdict != Verdict::Equivalent;
                    let key = format!("{}/{side}/{}", cell.id, step.id);
                    runner::cover(&mut report.coverage, &key, true, true, false, verdict != Verdict::Equivalent);
                    runner::cover(
                        &mut report.coverage,
                        &format!("oracle/{key}"),
                        true,
                        true,
                        false,
                        verdict != Verdict::Equivalent,
                    );
                    for package in &fixture.packages {
                        let unit = campaign.compilations.iter().find(|u| &u.unit == package);
                        runner::cover(
                            &mut report.coverage,
                            &format!("unit/{key}/{package}"),
                            unit.is_some(),
                            unit.is_some(),
                            false,
                            unit.is_some_and(|u| status(&u.capture) != Status::Success),
                        );
                    }
                    report.steps.push(StepReport {
                        cell: cell.id.clone(),
                        side: side.into(),
                        step: step.id.clone(),
                        predecessor: index.checked_sub(1).map(|i| fixture.steps[i].digest.clone()),
                        session: runner::digest(&(&state.root, MODE)),
                        campaign,
                        clean,
                        verdict,
                    });
                    if index + 1 < fixture.steps.len()
                        && report.steps.last().is_some_and(|r| r.clean.observation.teardown_complete)
                    {
                        process::remove_owned_tree(&clean_state.target)?;
                    }
                }
            }
            for step in &fixture.steps {
                let a = report
                    .steps
                    .iter()
                    .find(|r| r.cell == cell.id && r.side == "a" && r.step == step.id)
                    .ok_or("native-missing-a")?;
                let b = report
                    .steps
                    .iter()
                    .find(|r| r.cell == cell.id && r.side == "b" && r.step == step.id)
                    .ok_or("native-missing-b")?;
                let verdict =
                    core::compare(&a.campaign.observation, &b.campaign.observation, &runner::expected(&cell, step));
                failed |= verdict != Verdict::Equivalent;
                report.replay.push((cell.id.clone(), step.id.clone(), verdict));
            }
            runner::cover(&mut report.coverage, &cell.id, true, true, false, failed);
        }
        Ok(())
    })();
    if let Err(error) = execution {
        report.errors.push(error);
    }
    let retained = Manifest::decode(&process::read_bounded(
        &options.root.join("coverage-baseline.json"),
        core::MAX_MANIFEST_BYTES as u64,
    )?)
    .map_err(err)?;
    report.population = core::reconcile(&retained, &report.manifest, &report.coverage);
    report.complete = report.errors.is_empty()
        && report.population.as_ref().is_ok_and(|c| c.complete)
        && report.steps.iter().all(|s| s.verdict == Verdict::Equivalent)
        && report.replay.iter().all(|(_, _, v)| *v == Verdict::Equivalent);
    runner::save(&options.root.join("serial.json"), &report)?;
    Ok(report)
}
