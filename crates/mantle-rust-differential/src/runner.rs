//! Frozen baseline orchestration. Runs no experimental compiler protocols.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crunch_rustc_wrapper::observed_process::Capture;
use crunch_rustc_wrapper::observed_process::Fault;
use crunch_rustc_wrapper::observed_process::Injection;
use crunch_rustc_wrapper::waits::DiagnosticOutcome;
use crunch_rustc_wrapper::waits::RunOutcome;
use crunch_rustc_wrapper::waits::WaitCollector;
use crunch_rustc_wrapper::waits::WaitContext;
use mantle_rust_differential_core::Cell;
use mantle_rust_differential_core::Coverage;
use mantle_rust_differential_core::CoverageSummary;
use mantle_rust_differential_core::Denominator;
use mantle_rust_differential_core::Digest;
use mantle_rust_differential_core::Expected;
use mantle_rust_differential_core::Identity;
use mantle_rust_differential_core::Manifest;
use mantle_rust_differential_core::Member;
use mantle_rust_differential_core::Observation;
use mantle_rust_differential_core::Operation;
use mantle_rust_differential_core::Status;
use mantle_rust_differential_core::UnitRecords;
use mantle_rust_differential_core::Verdict;
use mantle_rust_differential_core::{self as core};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::Result;
use crate::err;
use crate::fixtures::Fixture;
use crate::fixtures::Step;
use crate::fixtures::{self};
use crate::process::{self};

pub const OUTPUT_POLICY: &str = "cargo-current-operation-semantic-output-v2";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub root: PathBuf,
    pub cargo: PathBuf,
    pub rustc: PathBuf,
    pub sysroot: PathBuf,
    pub linker: PathBuf,
    pub path: String,
    pub closures: Vec<PathBuf>,
    pub wrapper: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub host: String,
    #[serde(default)]
    pub toolchain: Option<Digest>,
    /// Native serial observation only; ordinary Cargo uses the explicit wrapper.
    #[serde(default)]
    pub diagnostics: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub unit: String,
    pub role: Vec<String>,
    pub path: PathBuf,
    pub digest: Digest,
    pub bytes: u64,
    pub fresh: bool,
    pub executable: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub root: PathBuf,
    pub source: Digest,
    pub cargo: Capture,
    pub raw_json: Vec<Value>,
    pub observation: Observation,
    pub program: Option<Capture>,
    pub artifacts: Vec<Artifact>,
    pub diagnostic: Option<DiagnosticOutcome>,
    pub errors: Vec<String>,
    pub reused_units: Vec<String>,
    pub executed_units: Vec<String>,
    pub first_error_timing: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepReport {
    pub cell: String,
    pub side: String,
    pub step: String,
    pub campaign: Attempt,
    pub clean: Attempt,
    pub verdict: Verdict,
    pub expectation: Expected,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FaultReport {
    pub kind: Fault,
    pub seed: Attempt,
    pub current: Attempt,
    pub evidence: core::FaultEvidence,
    pub validation: std::result::Result<(), core::Error>,
    pub recovery: RecoveryReport,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub lineage: core::Recovery,
    pub successor: Attempt,
    pub clean: Attempt,
    pub verdict: Verdict,
    pub validation: std::result::Result<(), core::Error>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema: String,
    pub manifest: Manifest,
    pub baseline: Manifest,
    pub manifest_identity: Digest,
    pub baseline_identity: Digest,
    pub options: Options,
    pub fixtures: Vec<Fixture>,
    pub probes: Vec<Capture>,
    pub steps: Vec<StepReport>,
    pub replay: Vec<(String, String, Verdict)>,
    pub faults: Vec<FaultReport>,
    pub coverage: Vec<Coverage>,
    pub population: std::result::Result<CoverageSummary, core::Error>,
    pub unavailable: Vec<(String, Verdict, String)>,
    pub errors: Vec<String>,
    pub complete: bool,
    pub output_policy: String,
    pub nonclaims: Vec<String>,
}
pub(crate) fn digest<T: Serialize + ?Sized>(value: &T) -> Digest {
    Digest::from_slice(&serde_json::to_vec(value).expect("serializable report value"))
}
fn identity<T: Serialize + ?Sized>(role: &str, value: &T) -> Identity {
    Identity {
        role: role.into(),
        digest: digest(value),
    }
}
pub(crate) fn save<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    process::write_new(path, &serde_json::to_vec_pretty(value).map_err(err)?)
}
fn cells() -> Vec<Cell> {
    let mut cells = vec![];
    for fixture in ["chain", "wide", "proc-macro", "generated"] {
        for operation in [Operation::Build, Operation::Check] {
            cells.push(Cell {
                id: format!(
                    "{fixture}-{}",
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
                mode: "baseline-replay".into(),
            });
        }
    }
    for lto in ["thin", "fat"] {
        cells.push(Cell {
            id: format!("chain-lto-{lto}"),
            fixture: "chain".into(),
            operation: Operation::Build,
            profile: "release".into(),
            lto: Some(lto.into()),
            features: vec![],
            required: true,
            mode: "baseline-replay".into(),
        });
    }
    cells.push(Cell {
        id: "future-early-metadata".into(),
        fixture: "chain".into(),
        operation: Operation::Build,
        profile: "dev".into(),
        lto: None,
        features: vec![],
        required: false,
        mode: "early-metadata-unregistered".into(),
    });
    cells
}
fn member(pop: &mut Vec<Member>, key: String, role: &str, required: bool, eligible: bool) {
    pop.push(Member {
        key,
        role: role.into(),
        required,
        selected: true,
        eligible,
        reason: (!eligible).then(|| "unregistered-capability".into()),
    });
}
pub(crate) fn freeze(options: &Options, fixtures: &[Fixture], probes: &[Capture]) -> Result<Manifest> {
    freeze_cells(options, fixtures, probes, cells(), "ordinary-Cargo-full-metadata-baseline-replay", true)
}

pub(crate) fn freeze_cells(
    options: &Options,
    fixtures: &[Fixture],
    probes: &[Capture],
    cells: Vec<Cell>,
    execution_policy: &str,
    faults: bool,
) -> Result<Manifest> {
    let mut population = vec![];
    for cell in &cells {
        member(&mut population, cell.id.clone(), "cell", cell.required, cell.required);
        if !cell.required {
            continue;
        }
        let fixture = fixtures.iter().find(|f| f.id == cell.fixture).ok_or("fixture-missing")?;
        for side in ["a", "b"] {
            for step in &fixture.steps {
                member(&mut population, format!("{}/{}/{}", cell.id, side, step.id), "step", true, true);
                member(&mut population, format!("oracle/{}/{}/{}", cell.id, side, step.id), "test", true, true);
                for package in &fixture.packages {
                    member(
                        &mut population,
                        format!("unit/{}/{}/{}/{package}", cell.id, side, step.id),
                        "unit",
                        step.expected.status == Status::Success,
                        true,
                    );
                }
            }
        }
    }
    for name in if faults { &["crash", "cancel"][..] } else { &[] } {
        member(&mut population, format!("fault/{name}"), "fault", true, true);
        member(&mut population, format!("recovery/{name}"), "recovery", true, true);
    }
    population.push(Member {
        key: "nonroot-documentation".into(),
        role: "nonroot".into(),
        required: false,
        selected: false,
        eligible: false,
        reason: Some("not-compiler-input".into()),
    });
    let mut denominators = vec![];
    for role in ["cell", "step", "test", "unit", "fault", "recovery"] {
        denominators.push(Denominator {
            role: role.into(),
            members: population
                .iter()
                .filter(|m| m.role == role && m.selected && m.eligible)
                .map(|m| m.key.clone())
                .collect(),
        });
    }
    let mut closure = vec![];
    for root in &options.closures {
        closure.push((root.clone(), process::hash_closure(root)?));
    }
    let tool_ids = (&closure, process::hash_closure(&options.linker)?);
    let mut identities = vec![
        identity("source", fixtures),
        identity("config", &fixtures.iter().map(|f| &f.steps[0].snapshot["Cargo.toml"]).collect::<Vec<_>>()),
        identity("lock", &fixtures.iter().map(|f| &f.steps[0].snapshot["Cargo.lock"]).collect::<Vec<_>>()),
        identity("dependencies", &fixtures.iter().map(|f| &f.packages).collect::<Vec<_>>()),
        Identity {
            role: "rustc".into(),
            digest: process::hash_closure(&options.rustc)?,
        },
        Identity {
            role: "cargo".into(),
            digest: process::hash_closure(&options.cargo)?,
        },
        Identity {
            role: "sysroot".into(),
            digest: process::hash_closure(&options.sysroot)?,
        },
        identity("tools", &tool_ids),
        identity(
            "argv",
            &(
                execution_policy,
                &cells,
                probes.iter().skip(1).map(|p| (&p.stdout, &p.stderr, p.code)).collect::<Vec<_>>(),
            ),
        ),
        identity(
            "environment",
            &(&options.path, &options.host, "env-clear-private-roots-no-implicit-wrappers-C-UTC-v2"),
        ),
        identity("profile", &cells.iter().map(|c| (&c.profile, &c.lto)).collect::<Vec<_>>()),
        identity("features", &Vec::<String>::new()),
        identity("cache", &"side-private-Cargo-incremental-no-result-cache-v1"),
        identity("mode", &execution_policy),
        identity("comparison", &(core::POLICY, OUTPUT_POLICY)),
        identity("edits", fixtures),
        identity(
            "harness",
            &(
                env!("CARGO_PKG_VERSION"),
                include_str!("runner.rs"),
                include_str!("fixtures.rs"),
                include_str!("process.rs"),
                include_str!("../../crunch-rustc-wrapper/src/waits.rs"),
                include_str!("../../crunch-rustc-wrapper/src/observed_process.rs"),
                include_str!("cohorts.rs"),
                include_str!("negative.rs"),
                include_str!("main.rs"),
                include_str!("serial.rs"),
            ),
        ),
    ];
    if let Some(wrapper) = &options.wrapper {
        identities.push(Identity {
            role: "wait-wrapper".into(),
            digest: process::hash_closure(wrapper)?,
        });
    }
    let bounds = core::Bounds {
        source_files: 128,
        source_bytes: process::SOURCE_BYTES,
        processes: 64,
        program_ms: crunch_rustc_wrapper::observed_process::PROGRAM_MS,
        total_ms: crunch_rustc_wrapper::observed_process::TOTAL_MS,
        scratch_bytes: crunch_rustc_wrapper::observed_process::SCRATCH_BYTES,
        ..core::Bounds::default()
    };
    let manifest = Manifest {
        schema: core::SCHEMA.into(),
        policy: core::POLICY.into(),
        bounds,
        identities,
        population,
        denominators,
        cells,
    };
    manifest.validate().map_err(err)?;
    Ok(manifest)
}
#[derive(Clone)]
pub(crate) struct State {
    pub root: PathBuf,
    pub source: PathBuf,
    pub target: PathBuf,
    pub packages: Value,
}
pub(crate) fn environment(options: &Options, state: &State) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for (key, value) in [
        ("PATH", options.path.clone()),
        ("HOME", state.root.join("home").display().to_string()),
        ("CARGO_HOME", state.root.join("cargo-home").display().to_string()),
        ("TMPDIR", state.root.join("tmp").display().to_string()),
        ("CARGO_TARGET_DIR", state.target.display().to_string()),
        ("RUSTC", options.rustc.display().to_string()),
        // Cargo also reads ancestor .cargo/config.toml despite the private CARGO_HOME.
        ("RUSTC_WRAPPER", String::new()),
        ("RUSTC_WORKSPACE_WRAPPER", String::new()),
        ("CARGO_INCREMENTAL", "1".into()),
        ("LC_ALL", "C".into()),
        ("LANG", "C".into()),
        ("TZ", "UTC".into()),
        ("CARGO_NET_OFFLINE", "true".into()),
    ] {
        env.insert(key.into(), value);
    }
    env
}
pub(crate) fn prepare_state(
    options: &Options,
    root: &Path,
    snapshot: &fixtures::Snapshot,
    total: Instant,
) -> Result<(State, Capture)> {
    crunch_rustc_wrapper::observed_process::private_dir(root)?;
    for directory in [
        "home",
        "cargo-home",
        "tmp",
        "target",
        "store",
        "state",
        "cache",
        "protocol",
        "locks",
    ] {
        crunch_rustc_wrapper::observed_process::private_dir(&root.join(directory))?;
    }
    let mut state = State {
        root: root.into(),
        source: root.join("source"),
        target: root.join("target"),
        packages: Value::Null,
    };
    process::materialize(&state.source, snapshot)?;
    let probe = root.join("metadata");
    crunch_rustc_wrapper::observed_process::private_dir(&probe)?;
    let mut cmd = Command::new(&options.cargo);
    cmd.current_dir(&state.source).env_clear().envs(environment(options, &state)).args([
        "metadata",
        "--format-version",
        "1",
        "--no-deps",
        "--offline",
        "--locked",
    ]);
    let capture =
        crunch_rustc_wrapper::observed_process::supervise(cmd, crunch_rustc_wrapper::observed_process::Supervision {
            root: &probe,
            scratch: &options.root,
            timeout_ms: crunch_rustc_wrapper::observed_process::PROBE_MS,
            total_start: total,
            injection: None,
            stream_bound: crunch_rustc_wrapper::observed_process::STREAM_BYTES,
        })?;
    save(&probe.join("capture.json"), &capture)?;
    if capture.code != Some(0) || !capture.complete || !capture.teardown_complete {
        return Err("cargo-metadata-prerequisite".into());
    }
    state.packages = serde_json::from_slice(&capture.stdout).map_err(err)?;
    Ok((state, capture))
}
pub(crate) fn normalize(value: &mut Value, state: &State, options: &Options) {
    match value {
        Value::String(text) => {
            *text = text
                .replace(&state.source.display().to_string(), "$SOURCE")
                .replace(&state.target.display().to_string(), "$TARGET")
                .replace(&state.root.display().to_string(), "$SIDE")
                .replace(&options.sysroot.display().to_string(), "$SYSROOT");
        }
        Value::Array(values) => {
            for value in values {
                normalize(value, state, options);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                normalize(value, state, options);
            }
        }
        _ => {}
    }
}
fn unit(record: &Value, state: &State, cell: &Cell, options: &Options) -> Result<(String, String)> {
    let id = record.get("package_id").and_then(Value::as_str).ok_or("missing-package-id")?;
    let package = state
        .packages
        .get("packages")
        .and_then(Value::as_array)
        .ok_or("metadata-packages")?
        .iter()
        .find(|p| p["id"] == id)
        .ok_or("unmapped-package-id")?;
    let target = record.get("target").ok_or("missing-target")?;
    let targets = package["targets"].as_array().ok_or("metadata-targets")?;
    if targets
        .iter()
        .filter(|t| {
            t["name"] == target["name"]
                && t["kind"] == target["kind"]
                && t["src_path"] == target["src_path"]
                && t["crate_types"] == target["crate_types"]
        })
        .count()
        != 1
    {
        return Err("ambiguous-exact-package-target".into());
    }
    let mut identity = json!({"package":id,"target":target,"operation":cell.operation,"profile":cell.profile,"features":cell.features,"lto":cell.lto,"host_role":target["kind"].as_array().is_some_and(|k|k.iter().any(|v|v=="proc-macro"||v=="custom-build"))});
    normalize(&mut identity, state, options);
    Ok((digest(&identity).as_str().into(), package["name"].as_str().ok_or("package-name")?.into()))
}
pub(crate) fn outcome(status: Status) -> RunOutcome {
    match status {
        Status::Success => RunOutcome::Success,
        Status::Rejected => RunOutcome::Rejected,
        Status::Crash => RunOutcome::Crash,
        Status::Cancelled => RunOutcome::Cancelled,
        Status::Timeout => RunOutcome::Timeout,
        Status::Infrastructure => RunOutcome::Infrastructure,
    }
}
pub(crate) fn attempt(
    options: &Options,
    state: &State,
    cell: &Cell,
    step: &Step,
    id: &str,
    total: Instant,
    injection: Option<Fault>,
    stream_bound: u64,
) -> Result<Attempt> {
    process::verify(&state.source, &step.snapshot)?;
    let root = state.root.join(id);
    crunch_rustc_wrapper::observed_process::private_dir(&root)?;
    let mut cmd = Command::new(&options.cargo);
    cmd.current_dir(&state.source).env_clear().envs(environment(options, state));
    let mut args = vec![
        if step.force {
            "rustc".to_string()
        } else if cell.operation == Operation::Build {
            "build".into()
        } else {
            "check".into()
        },
        "--locked".into(),
        "--offline".into(),
        "--message-format=json".into(),
        "--jobs=2".into(),
        "--package=fixture-app".into(),
    ];
    if cell.profile == "release" {
        args.push("--release".into());
    }
    // Remap paths at compilation, before observing diagnostics/artifacts. Keep side-private incremental
    // state.
    let flags = format!(
        "--sysroot={}\x1f-C\x1flinker={}\x1f--remap-path-prefix={}=/mantle-fixture\x1f--remap-path-prefix={}=/mantle-target",
        options.sysroot.display(),
        options.linker.display(),
        state.source.display(),
        state.target.display()
    );
    cmd.env("CARGO_ENCODED_RUSTFLAGS", flags);
    if let Some(lto) = &cell.lto {
        cmd.env("CARGO_PROFILE_RELEASE_LTO", lto);
    }
    if step.force {
        args.push("--bin=fixture-app".into());
        if cell.operation == Operation::Check {
            args.push("--profile=check".into());
        }
        args.extend(["--".into(), "--cfg=mantle_differential_force_reentry".into()]);
    }
    cmd.args(&args);
    let mut collector = if let Some(wrapper) = &options.wrapper {
        let context = WaitContext {
            invocation: digest(&(id, &root)),
            clock: digest(&(id, &root, "clock")),
            toolchain: options.toolchain.clone().ok_or("unpinned-toolchain")?,
            configuration: digest(&(cell, &args)),
            host: digest(&options.host),
            target: digest(&options.host),
        };
        match WaitCollector::start(&root, wrapper, context) {
            Ok(c) => Some(c),
            Err(e) => {
                save(&root.join("wait-unavailable.json"), &e.to_string())?;
                None
            }
        }
    } else {
        None
    };
    if let Some(c) = &collector {
        c.configure(&mut cmd);
    }
    let injection = injection.map(|fault| Injection {
        fault,
        rustc: options.rustc.clone(),
        manifest: state.source.join("lib0"),
    });
    let capture =
        crunch_rustc_wrapper::observed_process::supervise(cmd, crunch_rustc_wrapper::observed_process::Supervision {
            root: &root,
            scratch: &options.root,
            timeout_ms: crunch_rustc_wrapper::observed_process::BUILD_MS,
            total_start: total,
            injection,
            stream_bound,
        })?;
    let mut status = if capture.failure.as_deref() == Some("deadline") {
        Status::Timeout
    } else if !capture.complete || !capture.teardown_complete {
        Status::Infrastructure
    } else if let Some(trigger) = &capture.trigger {
        if trigger.delivered {
            if trigger.signal == libc::SIGKILL {
                Status::Crash
            } else {
                Status::Cancelled
            }
        } else {
            Status::Infrastructure
        }
    } else if capture.signal.is_some() {
        Status::Crash
    } else if capture.code == Some(0) {
        Status::Success
    } else if capture.code == Some(101) {
        Status::Rejected
    } else {
        Status::Infrastructure
    };
    let mut errors = vec![];
    let mut records = BTreeMap::<String, Vec<Value>>::new();
    let mut raw_json = vec![];
    let mut artifacts = vec![];
    let mut executable = None;
    let mut finished = None;
    let mut reused_units = vec![];
    let mut executed_units = vec![];
    let mut semantic_artifacts = vec![];
    let text = std::str::from_utf8(&capture.stdout).map_err(err)?;
    for line in text.lines() {
        let record: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("malformed-cargo-json:{e}"));
                continue;
            }
        };
        if raw_json.len() >= 16_384 {
            errors.push("cargo-record-bound".into());
            break;
        }
        if let Some(c) = &mut collector {
            c.observe_cargo(&record);
        }
        let reason = record["reason"].as_str().unwrap_or("");
        let result = (|| -> Result<()> {
            match reason {
                "compiler-message" => {
                    let (key, _) = unit(&record, state, cell, options)?;
                    let mut diagnostic = record.get("message").ok_or("missing-diagnostic")?.clone();
                    normalize(&mut diagnostic, state, options);
                    records.entry(key).or_default().push(diagnostic);
                }
                "compiler-artifact" => {
                    let (key, package) = unit(&record, state, cell, options)?;
                    let fresh = record["fresh"].as_bool().ok_or("artifact-freshness")?;
                    if fresh {
                        reused_units.push(package);
                    } else {
                        executed_units.push(package);
                    }
                    let roles = record["target"]["kind"]
                        .as_array()
                        .ok_or("artifact-kinds")?
                        .iter()
                        .map(|v| v.as_str().map(str::to_owned).ok_or("artifact-kind".into()))
                        .collect::<Result<Vec<_>>>()?;
                    let mut semantic = json!({"unit":key,"roles":roles,"profile":record["profile"],"features":record["features"],"successful":true});
                    normalize(&mut semantic, state, options);
                    semantic_artifacts.push(semantic);
                    for name in record["filenames"].as_array().ok_or("artifact-files")? {
                        let path = PathBuf::from(name.as_str().ok_or("artifact-path")?);
                        if !path.starts_with(&state.target) {
                            return Err("artifact-outside-target".into());
                        }
                        let bytes = process::read_bounded(&path, 128 * 1024 * 1024)?;
                        let is_executable = record["executable"].as_str().is_some_and(|s| Path::new(s) == path);
                        if is_executable && record["target"]["name"] == "fixture-app" {
                            executable = Some(path.clone());
                        }
                        artifacts.push(Artifact {
                            unit: key.clone(),
                            role: roles.clone(),
                            path,
                            digest: Digest::from_slice(&bytes),
                            bytes: bytes.len() as u64,
                            fresh,
                            executable: is_executable,
                        });
                    }
                }
                "build-script-executed" => {
                    let id = record["package_id"].as_str().ok_or("build-script-package")?;
                    if !state.packages["packages"].as_array().ok_or("metadata-packages")?.iter().any(|p| p["id"] == id)
                    {
                        return Err("build-script-unmapped".into());
                    }
                }
                "build-finished" => {
                    if finished.replace(record["success"].as_bool().ok_or("build-finished-success")?).is_some() {
                        return Err("duplicate-build-finished".into());
                    }
                }
                _ => return Err("unknown-cargo-record".into()),
            }
            Ok(())
        })();
        if let Err(e) = result {
            errors.push(e);
        }
        raw_json.push(record);
    }
    if finished != Some(capture.code == Some(0)) && capture.trigger.is_none() {
        errors.push("missing-or-inconsistent-build-finished".into());
    }
    let stderr = String::from_utf8_lossy(&capture.stderr);
    let ice = stderr.contains("internal compiler error")
        || stderr.contains("the compiler unexpectedly panicked")
        || records.values().flatten().any(|d| {
            d["level"] == "ice" || d["message"].as_str().is_some_and(|m| m.contains("internal compiler error"))
        });
    if step.force
        && status == Status::Success
        && (!executed_units.iter().any(|p| p == "fixture-app")
            || !capture.processes.iter().any(|p| p.executable.as_ref() == Some(&options.rustc)))
    {
        errors.push("forced-compiler-reentry-unobserved".into());
    }
    if process::verify(&state.source, &step.snapshot).is_err() {
        errors.push("source-changed-during-attempt".into());
    }
    let mut program = None;
    if status == Status::Success && errors.is_empty() && cell.operation == Operation::Build {
        if let Some(executable) = executable {
            let program_root = root.join("program");
            crunch_rustc_wrapper::observed_process::private_dir(&program_root)?;
            let mut command = Command::new(executable);
            command.current_dir(&state.source).env_clear().envs(environment(options, state));
            let output = crunch_rustc_wrapper::observed_process::supervise(
                command,
                crunch_rustc_wrapper::observed_process::Supervision {
                    root: &program_root,
                    scratch: &options.root,
                    timeout_ms: crunch_rustc_wrapper::observed_process::PROGRAM_MS,
                    total_start: total,
                    injection: None,
                    stream_bound: 1_048_576,
                },
            )?;
            if output.code != Some(0) || !output.complete || !output.teardown_complete {
                errors.push("current-program-failed".into());
            }
            program = Some(output);
        } else {
            errors.push("current-success-executable-unavailable".into());
        }
    }
    if !errors.is_empty() {
        status = Status::Infrastructure;
    }
    let mut observation = Observation {
        status,
        units: records.into_iter().map(|(unit, records)| UnitRecords { unit, records }).collect(),
        program_stdout: None,
        program_stderr: None,
        accepted_outputs: vec![],
        complete: capture.complete && errors.is_empty(),
        teardown_complete: capture.teardown_complete && program.as_ref().is_none_or(|p| p.teardown_complete),
        ice,
    };
    if status == Status::Success {
        if let Some(p) = &program {
            observation.program_stdout = Some(p.stdout.clone());
            observation.program_stderr = Some(p.stderr.clone());
        }
        // Explicit operation-specific semantic identity; NOT raw-byte equivalence or a receipt.
        semantic_artifacts.sort_by_key(|v| v["unit"].as_str().unwrap_or("").to_owned());
        observation.accepted_outputs.push(identity(
            OUTPUT_POLICY,
            &(
                &step.digest,
                &options.toolchain,
                cell.operation,
                status,
                &observation.units,
                &observation.program_stdout,
                &observation.program_stderr,
                &semantic_artifacts,
            ),
        ));
    }
    if let Some(c) = &mut collector {
        c.set_resources(capture.resources.clone(), capture.monotonic_origin_ns);
    }
    let diagnostic = collector.map(|c| {
        c.finish(
            outcome(status),
            capture.monotonic_origin_ns,
            capture.elapsed_ns,
            capture.teardown_complete.then_some(capture.last_reap_ns),
        )
    });
    let result = Attempt {
        id: id.into(),
        root: root.clone(),
        source: step.digest.clone(),
        cargo: capture,
        raw_json,
        observation,
        program,
        artifacts,
        diagnostic,
        errors,
        reused_units,
        executed_units,
        first_error_timing:
            "runner completion capture: live first-error unavailable; inspect opted-in wrapper report if supported"
                .into(),
    };
    save(&root.join("attempt.json"), &result)?;
    Ok(result)
}
pub(crate) fn expected(cell: &Cell, step: &Step) -> Expected {
    let mut expected = step.expected.clone();
    if cell.operation == Operation::Check {
        expected.program_stdout = None;
    }
    expected
}
pub(crate) fn initial_coverage(manifest: &Manifest) -> Vec<Coverage> {
    manifest
        .population
        .iter()
        .map(|m| Coverage {
            key: m.key.clone(),
            attempted: false,
            executed: false,
            reused: false,
            fallback: None,
            unsupported: (!m.eligible && m.selected).then(|| "unregistered-capability".into()),
            failed: false,
            unrun: m.selected,
        })
        .collect()
}
pub(crate) fn cover(coverage: &mut [Coverage], key: &str, attempted: bool, executed: bool, reused: bool, failed: bool) {
    if let Some(c) = coverage.iter_mut().find(|c| c.key == key) {
        c.attempted = attempted;
        c.executed = executed;
        c.reused = reused;
        c.failed = failed;
        c.unrun = !executed && !reused;
    }
}
fn run_campaigns(options: &Options, report: &mut Report, total: Instant) -> Result<()> {
    for cell in report.manifest.cells.clone().into_iter().filter(|c| c.required) {
        let fixture = report.fixtures.iter().find(|f| f.id == cell.fixture).ok_or("fixture-missing")?.clone();
        let cell_root = options.root.join(&cell.id);
        crunch_rustc_wrapper::observed_process::private_dir(&cell_root)?;
        let mut cell_failed = false;
        for side in ["a", "b"] {
            let side_root = cell_root.join(side);
            let (state, metadata) = prepare_state(options, &side_root, &fixture.steps[0].snapshot, total)?;
            report.probes.push(metadata);
            let mut previous = &fixture.steps[0].snapshot;
            for (index, step) in fixture.steps.iter().enumerate() {
                if total.elapsed().as_millis() >= u128::from(crunch_rustc_wrapper::observed_process::TOTAL_MS) {
                    return Err("total-deadline".into());
                }
                if index != 0 {
                    process::apply(&state.source, previous, step)?;
                }
                previous = &step.snapshot;
                let campaign = attempt(
                    options,
                    &state,
                    &cell,
                    step,
                    &format!("step-{index:02}-{}", step.id),
                    total,
                    None,
                    crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                )?;
                let clean_root = side_root.join(format!("clean-{index:02}"));
                let (clean_state, metadata) = prepare_state(options, &clean_root, &step.snapshot, total)?;
                report.probes.push(metadata);
                let clean = attempt(
                    options,
                    &clean_state,
                    &cell,
                    step,
                    "compile",
                    total,
                    None,
                    crunch_rustc_wrapper::observed_process::STREAM_BYTES,
                )?;
                let expectation = expected(&cell, step);
                let verdict = core::compare(&campaign.observation, &clean.observation, &expectation);
                let failed = verdict != Verdict::Equivalent;
                cell_failed |= failed;
                let key = format!("{}/{side}/{}", cell.id, step.id);
                cover(&mut report.coverage, &key, true, true, !campaign.reused_units.is_empty(), failed);
                cover(&mut report.coverage, &format!("oracle/{key}"), true, true, false, failed);
                for package in &fixture.packages {
                    let executed = campaign.executed_units.contains(package);
                    let reused = campaign.reused_units.contains(package);
                    cover(&mut report.coverage, &format!("unit/{key}/{package}"), true, executed, reused, false);
                }
                report.steps.push(StepReport {
                    cell: cell.id.clone(),
                    side: side.into(),
                    step: step.id.clone(),
                    campaign,
                    clean,
                    verdict,
                    expectation,
                });
                if index + 1 < fixture.steps.len() {
                    if !report.steps.last().ok_or("missing-finalized-step")?.clean.cargo.teardown_complete {
                        return Err("reference-teardown-incomplete".into());
                    }
                    process::remove_owned_tree(&clean_state.target)?;
                }
            }
        }
        for step in &fixture.steps {
            let a = report
                .steps
                .iter()
                .find(|r| r.cell == cell.id && r.side == "a" && r.step == step.id)
                .ok_or("missing-a")?;
            let b = report
                .steps
                .iter()
                .find(|r| r.cell == cell.id && r.side == "b" && r.step == step.id)
                .ok_or("missing-b")?;
            let verdict = core::compare(&a.campaign.observation, &b.campaign.observation, &a.expectation);
            cell_failed |= verdict != Verdict::Equivalent;
            report.replay.push((cell.id.clone(), step.id.clone(), verdict));
        }
        cover(&mut report.coverage, &cell.id, true, true, false, cell_failed);
    }
    Ok(())
}
fn run_faults(options: &Options, report: &mut Report, total: Instant) -> Result<()> {
    let fixture = fixtures::generate("chain")?;
    let cell = report.manifest.cells.iter().find(|c| c.id == "chain-build").ok_or("fault-cell")?.clone();
    for (name, kind) in [("crash", Fault::Crash), ("cancel", Fault::Cancel)] {
        let root = options.root.join(format!("fault-{name}"));
        let (state, metadata) = prepare_state(options, &root, &fixture.steps[0].snapshot, total)?;
        report.probes.push(metadata);
        let seed = attempt(
            options,
            &state,
            &cell,
            &fixture.steps[0],
            "seed",
            total,
            None,
            crunch_rustc_wrapper::observed_process::STREAM_BYTES,
        )?;
        core::validate_oracle(&seed.observation, &fixture.steps[0].expected).map_err(err)?;
        let prior = seed.artifacts.iter().find(|a| a.executable).ok_or("fault-prior-artifact")?.digest.clone();
        let body = &fixture.steps[3];
        process::apply(&state.source, &fixture.steps[0].snapshot, body)?;
        let current = attempt(
            options,
            &state,
            &cell,
            body,
            "injected",
            total,
            Some(kind),
            crunch_rustc_wrapper::observed_process::STREAM_BYTES,
        )?;
        let expected = if kind == Fault::Crash {
            Status::Crash
        } else {
            Status::Cancelled
        };
        let evidence = core::FaultEvidence {
            expected,
            observed: current.observation.status,
            producer_observed_running: current.cargo.trigger.is_some(),
            signal_requested: current.cargo.trigger.as_ref().is_some_and(|t| t.delivered),
            producer_signal: current.cargo.trigger.as_ref().map(|t| t.signal),
            teardown_complete: current.cargo.teardown_complete,
            current_outputs_accepted: !current.observation.accepted_outputs.is_empty() || current.program.is_some(),
            prior_success_artifact: Some(prior),
        };
        let validation = core::validate_fault(&evidence);
        cover(&mut report.coverage, &format!("fault/{name}"), true, true, false, validation.is_err());
        if !current.cargo.teardown_complete {
            return Err("recovery-predecessor-not-reaped".into());
        }
        let discarded = process::hash_closure(&state.target)?;
        let (successor_state, metadata) =
            prepare_state(options, &root.join("recovery-successor"), &body.snapshot, total)?;
        report.probes.push(metadata);
        let successor = attempt(
            options,
            &successor_state,
            &cell,
            body,
            "compile",
            total,
            None,
            crunch_rustc_wrapper::observed_process::STREAM_BYTES,
        )?;
        let (clean_state, metadata) = prepare_state(options, &root.join("recovery-clean"), &body.snapshot, total)?;
        report.probes.push(metadata);
        let clean = attempt(
            options,
            &clean_state,
            &cell,
            body,
            "compile",
            total,
            None,
            crunch_rustc_wrapper::observed_process::STREAM_BYTES,
        )?;
        let verdict = core::compare(&successor.observation, &clean.observation, &body.expected);
        let lineage = core::Recovery {
            predecessor: digest(&current.root),
            successor: digest(&successor.root),
            cause: format!("injected-{name}"),
            original_status: current.observation.status,
            original_observation: digest(&current.observation),
            discarded: vec![Identity {
                role: "predecessor-cargo-target-not-reused".into(),
                digest: discarded,
            }],
            preserved: vec![
                Identity {
                    role: "source-snapshot".into(),
                    digest: body.digest.clone(),
                },
                identity("failed-attempt-evidence", &current),
            ],
            teardown_complete: current.cargo.teardown_complete,
            continuity_satisfied: false,
            remaining_obligations: vec![format!("uninterrupted-after-{name}-not-demonstrated")],
        };
        let recovery_validation = lineage.validate();
        cover(
            &mut report.coverage,
            &format!("recovery/{name}"),
            true,
            true,
            false,
            recovery_validation.is_err() || verdict != Verdict::Equivalent,
        );
        let recovery = RecoveryReport {
            lineage,
            successor,
            clean,
            verdict,
            validation: recovery_validation,
        };
        report.faults.push(FaultReport {
            kind,
            seed,
            current,
            evidence,
            validation,
            recovery,
        });
    }
    Ok(())
}
/// Single invocation; all paths and bounded failures remain in the retained root.
pub fn execute(mut options: Options) -> Result<Report> {
    if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } != 0 {
        return Err("subreaper-unavailable".into());
    }
    crunch_rustc_wrapper::observed_process::private_dir(&options.root)?;
    let total = Instant::now();
    pin(&mut options)?;
    let fixtures = ["chain", "wide", "proc-macro", "generated"]
        .into_iter()
        .map(fixtures::generate)
        .collect::<Result<Vec<_>>>()?;
    save(&options.root.join("fixture-recipe.json"), &fixtures)?;
    let probes = probe_tools(&options, &fixtures[0], total)?;
    let manifest = freeze(&options, &fixtures, &probes)?;
    save(&options.root.join("manifest.json"), &manifest)?;
    let baseline = if let Some(path) = &options.baseline {
        Manifest::decode(&process::read_bounded(path, core::MAX_MANIFEST_BYTES as u64)?).map_err(err)?
    } else {
        manifest.clone()
    };
    save(&options.root.join("coverage-baseline.json"), &baseline)?;
    // Re-read the separately retained immutable baseline at finalization.
    let mut report=Report{schema:"mantle-rust-differential-report-v1".into(),manifest_identity:manifest.identity().map_err(err)?,baseline_identity:baseline.identity().map_err(err)?,coverage:initial_coverage(&manifest),manifest,baseline,options:options.clone(),fixtures,probes,steps:vec![],replay:vec![],faults:vec![],population:Err(core::Error::Missing),unavailable:vec![("future-early-metadata".into(),Verdict::Unsupported,"no admitted same-binary early-publication/swap capability; rustd and Headstart not implemented".into())],errors:vec![],complete:false,output_policy:OUTPUT_POLICY.into(),nonclaims:vec!["No compiler correctness, raw artifact reproducibility, native per-unit compatibility, Mantle self-hosting, release qualification, or speedup claim".into(),"Ordinary Cargo full-metadata pipelining only; no early-interface phase evidence".into()]};
    if let Err(e) = run_campaigns(&options, &mut report, total) {
        report.errors.push(e);
    }
    if report.errors.is_empty()
        && let Err(e) = run_faults(&options, &mut report, total)
    {
        report.errors.push(e);
    }
    let retained = Manifest::decode(&process::read_bounded(
        &options.root.join("coverage-baseline.json"),
        core::MAX_MANIFEST_BYTES as u64,
    )?)
    .map_err(err)?;
    report.population = core::reconcile(&retained, &report.manifest, &report.coverage);
    report.complete = report.errors.is_empty()
        && report.population.as_ref().is_ok_and(|p| p.complete)
        && report.steps.iter().all(|s| s.verdict == Verdict::Equivalent)
        && report.replay.iter().all(|(_, _, v)| *v == Verdict::Equivalent)
        && report.faults.iter().all(|f| f.validation.is_ok());
    save(&options.root.join("report.json"), &report)?;
    Ok(report)
}
/// Pin executable content and complete declared sysroot closure before any build.
pub(crate) fn pin(options: &mut Options) -> Result<()> {
    if options.closures.is_empty() {
        return Err("explicit-runtime-tool-closures-required".into());
    }
    for tool in [&options.cargo, &options.rustc, &options.linker, &options.sysroot]
        .into_iter()
        .chain(options.wrapper.iter())
        .chain(options.closures.iter())
    {
        if !tool.is_absolute() || fs::canonicalize(tool).map_err(err)? != *tool {
            return Err("tool-path-must-be-resolved-absolute".into());
        }
    }
    if options.path.is_empty() || std::env::split_paths(&options.path).any(|p| !p.is_absolute() || !p.is_dir()) {
        return Err("explicit-absolute-tool-path-required".into());
    }
    let closures = options
        .closures
        .iter()
        .map(|root| Ok((root, process::hash_closure(root)?)))
        .collect::<Result<Vec<_>>>()?;
    let linker = process::hash_closure(&options.linker)?;
    if let Some(wrapper) = &options.wrapper {
        process::hash_closure(wrapper)?;
    }
    if options.host.is_empty() || options.host.len() > 128 {
        return Err("declared-host-triple-required".into());
    }
    options.toolchain = Some(digest(&(
        process::hash_closure(&options.rustc)?,
        process::hash_closure(&options.cargo)?,
        process::hash_closure(&options.sysroot)?,
        linker,
        closures,
    )));
    Ok(())
}
pub(crate) fn probe_tools(options: &Options, fixture: &Fixture, total: Instant) -> Result<Vec<Capture>> {
    let fixtures = std::slice::from_ref(fixture);
    let mut probes = vec![];
    // Probe actual pinned tools in a child-only environment, never the caller's Cargo config.
    let probe_state_root = options.root.join("probe-state");
    let (probe_state, metadata) = prepare_state(options, &probe_state_root, &fixtures[0].steps[0].snapshot, total)?;
    probes.push(metadata);
    for (index, (tool, args)) in [
        (&options.cargo, vec!["--version", "--verbose"]),
        (&options.rustc, vec!["--version", "--verbose"]),
        (&options.rustc, vec!["--print", "sysroot"]),
        (&options.linker, vec!["--version"]),
    ]
    .into_iter()
    .enumerate()
    {
        let root = options.root.join(format!("tool-probe-{index}"));
        crunch_rustc_wrapper::observed_process::private_dir(&root)?;
        let mut cmd = Command::new(tool);
        cmd.current_dir(&probe_state.source).env_clear().envs(environment(options, &probe_state)).args(args);
        let capture = crunch_rustc_wrapper::observed_process::supervise(
            cmd,
            crunch_rustc_wrapper::observed_process::Supervision {
                root: &root,
                scratch: &options.root,
                timeout_ms: crunch_rustc_wrapper::observed_process::PROBE_MS,
                total_start: total,
                injection: None,
                stream_bound: crunch_rustc_wrapper::observed_process::STREAM_BYTES,
            },
        )?;
        save(&root.join("capture.json"), &capture)?;
        if capture.code != Some(0) || !capture.complete || !capture.teardown_complete {
            save(
                &options.root.join("blocked.json"),
                &json!({"verdict":"blocked","reason":"pinned-tool-probe","capture":capture}),
            )?;
            return Err("pinned-tool-prerequisite".into());
        }
        probes.push(capture);
    }
    let observed_sysroot = String::from_utf8_lossy(&probes[3].stdout).trim().to_owned();
    if fs::canonicalize(observed_sysroot).map_err(err)? != options.sysroot {
        return Err("declared-sysroot-mismatch".into());
    }
    if !String::from_utf8_lossy(&probes[2].stdout)
        .lines()
        .any(|line| line == format!("host: {}", options.host))
    {
        return Err("declared-host-mismatch".into());
    }
    Ok(probes)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn ancestor_cargo_wrappers_cannot_replace_the_selected_compiler() {
        let root = std::env::temp_dir().join(format!(
            "mantle-differential-wrappers-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        crunch_rustc_wrapper::observed_process::private_dir(&root).unwrap();
        fs::create_dir(root.join(".cargo")).unwrap();
        let wrapper = root.join("forbidden-wrapper");
        fs::write(&wrapper, b"#!/bin/sh\nexit 97\n").unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            root.join(".cargo/config.toml"),
            format!(
                "[build]\nrustc-wrapper = {:?}\nrustc-workspace-wrapper = {:?}\n",
                wrapper.to_str().unwrap(),
                wrapper.to_str().unwrap()
            ),
        )
        .unwrap();
        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("src")).unwrap();
        fs::write(
            source.join("Cargo.toml"),
            "[package]\nname = \"wrapper-isolation\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
        )
        .unwrap();
        fs::write(
            source.join("Cargo.lock"),
            "version = 4\n[[package]]\nname = \"wrapper-isolation\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(source.join("src/main.rs"), "fn main() { println!(\"42\"); }\n").unwrap();
        let probe = Command::new("rustc").args(["--print", "sysroot"]).output().unwrap();
        assert!(probe.status.success());
        let sysroot = PathBuf::from(String::from_utf8(probe.stdout).unwrap().trim());
        let rustc = fs::canonicalize(sysroot.join("bin/rustc")).unwrap();
        let options = Options {
            root: root.clone(),
            cargo: env!("CARGO").into(),
            rustc,
            sysroot,
            linker: PathBuf::new(),
            path: std::env::var("PATH").unwrap(),
            closures: Vec::new(),
            wrapper: None,
            baseline: None,
            host: String::new(),
            toolchain: None,
            diagnostics: false,
        };
        let state = State {
            root: root.clone(),
            source,
            target: root.join("target"),
            packages: Value::Null,
        };
        let compiled = Command::new(&options.cargo)
            .args(["build", "--locked", "--offline"])
            .current_dir(&state.source)
            .env_clear()
            .envs(environment(&options, &state))
            .env("CARGO_ENCODED_RUSTFLAGS", "")
            .output()
            .unwrap();
        let executed = compiled
            .status
            .success()
            .then(|| Command::new(state.target.join("debug/wrapper-isolation")).output().unwrap());
        fs::remove_dir_all(&root).unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let executed = executed.unwrap();
        assert!(executed.status.success());
        assert_eq!(executed.stdout, b"42\n");
    }
}
