#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Effect boundary for the explicitly supported iCE40 synthesis cohort.
//! Uses the simulation adapter's network-denied, timeout-bounded sandbox runner,
//! never its simulation evidence or correctness claims.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_fpga_core::FileInput;
use crunch_fpga_core::NetlistRequest;
use crunch_fpga_core::PlannedProfile;
use crunch_fpga_core::Profile;
use crunch_fpga_core::Report;
use crunch_fpga_core::ReportRequest;
use crunch_fpga_core::StageIdentities;
use crunch_fpga_core::Tool;
use crunch_hardware_simulation::ReadOnlyBinding;
use crunch_hardware_simulation::StrictActionRequest;
use crunch_hardware_simulation::run_strict_action;
use serde::Deserialize;
use serde::Serialize;

pub type Result<T, E = String> = std::result::Result<T, E>;

/// The exact artifacts one synthesis run must produce, relative to its root.
const ARTIFACT_PATHS: &[&str] = &[
    "synthesis/netlist.json",
    "synthesis/stdout.log",
    "synthesis/stderr.log",
    "place-route/routed.asc",
    "place-route/routed.json",
    "place-route/report.json",
    "place-route/stdout.log",
    "place-route/stderr.log",
    "packing/bitstream.bin",
    "packing/stdout.log",
    "packing/stderr.log",
];

/// The exact synthesis recipe and constraint text a profile must reproduce.
struct Recipe<'a> {
    synthesis_script: &'a str,
    constraints: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub path: String,
    pub blake3: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub profile: Profile,
    pub stages: StageIdentities,
    pub report: Report,
    pub artifacts: Vec<Artifact>,
    pub synthesis_completed: bool,
    pub declared_clock_timing_accepted: bool,
    pub bitstream_packed: bool,
    pub simulation_validated: bool,
    pub physical_board_validated: bool,
    pub non_claims: Vec<String>,
}

pub fn read_profile(path: &Path) -> Result<Profile> {
    let bytes = read_bounded(path, crunch_fpga_core::MAX_TEXT_BYTES as u64)?;
    let profile: Profile = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    crunch_fpga_core::validate(profile).map_err(|error| error.to_string())
}

/// Streaming binary measurement: nextpnr carries a large embedded chipdb.
pub fn hash_file(path: &Path, maximum_bytes: u64) -> Result<(String, u64)> {
    let mut file = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum_bytes {
        return Err(format!("file-type-or-byte-bound:{}", path.display()));
    }
    let mut buffer = [0_u8; 65_536];
    let mut hasher = blake3::Hasher::new();
    let mut total_bytes = 0_u64;
    while total_bytes <= maximum_bytes {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        total_bytes = total_bytes.checked_add(count as u64).ok_or("file-size-overflow")?;
        if total_bytes > maximum_bytes {
            return Err("file-grew-past-byte-bound".into());
        }
        hasher.update(&buffer[..count]);
    }
    if total_bytes != metadata.len() {
        return Err("file-changed-during-observation".into());
    }
    debug_assert!(metadata.is_file());
    debug_assert!(total_bytes <= maximum_bytes);
    Ok((hasher.finalize().to_hex().to_string(), total_bytes))
}

fn read_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("inspect {}: {e}", path.display()))?;
    if !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(format!("regular-file-byte-bound:{}", path.display()));
    }
    let capacity_bytes = usize::try_from(metadata.len()).map_err(|e| e.to_string())?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > maximum_bytes {
        return Err("file-grew-past-byte-bound".into());
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

fn admit_tool(tool: &Tool) -> Result<()> {
    for root in &tool.closure {
        let path = Path::new(root);
        let canonical = fs::canonicalize(path).map_err(|e| format!("missing-declared-tool-closure:{root}:{e}"))?;
        if canonical != path || !path.is_dir() {
            return Err(format!("noncanonical-tool-closure:{root}"));
        }
    }
    admit_executable(&tool.executable, &tool.closure)
}

fn admit_executable(input: &FileInput, closure: &[String]) -> Result<()> {
    let canonical = fs::canonicalize(&input.path).map_err(|e| format!("missing-pinned-tool:{}:{e}", input.path))?;
    if !closure.iter().any(|root| canonical.starts_with(root)) {
        return Err(format!("tool-symlink-outside-closure:{}", input.path));
    }
    if hash_file(&canonical, crunch_fpga_core::MAX_FILE_BYTES)?.0 != input.blake3 {
        return Err(format!("tool-or-embedded-database-digest-mismatch:{}", input.name));
    }
    Ok(())
}

fn snapshot(input: &FileInput, directory: &Path) -> Result<Vec<u8>> {
    let bytes = read_bounded(Path::new(&input.path), crunch_fpga_core::MAX_TEXT_BYTES as u64)?;
    if blake3::hash(&bytes).to_hex().as_str() != input.blake3 {
        return Err(format!("immutable-source-digest-mismatch:{}", input.name));
    }
    write_new(&directory.join(&input.name), &bytes)?;
    Ok(bytes)
}

fn prepare(profile: &Profile, recipe: &Recipe<'_>, root: &Path) -> Result<()> {
    for tool in [&profile.yosys, &profile.nextpnr, &profile.packer, &profile.shell] {
        admit_tool(tool)?;
    }
    admit_tool(&profile.bwrap)?;
    fs::create_dir(root).map_err(|e| format!("fresh-output-root-required:{}:{e}", root.display()))?;
    let synth = root.join("synthesis");
    fs::create_dir(&synth).map_err(|e| e.to_string())?;
    let mut total_bytes = 0_u64;
    for input in profile.rtl.iter().chain(&profile.includes) {
        total_bytes += snapshot(input, &synth)?.len() as u64;
        if total_bytes > crunch_fpga_core::MAX_ARTIFACT_BYTES {
            return Err("aggregate-source-byte-bound".into());
        }
    }
    let script = snapshot(&profile.synthesis_script, &synth)?;
    if script != recipe.synthesis_script.as_bytes() {
        return Err("pinned-synthesis-script-does-not-match-typed-recipe".into());
    }
    debug_assert_eq!(script, recipe.synthesis_script.as_bytes());
    let pnr = root.join("place-route");
    fs::create_dir(&pnr).map_err(|e| e.to_string())?;
    if snapshot(&profile.constraints, &pnr)? != recipe.constraints.as_bytes() {
        return Err("invalid-pcf-does-not-match-complete-typed-constraints".into());
    }
    debug_assert!(total_bytes <= crunch_fpga_core::MAX_ARTIFACT_BYTES);
    Ok(())
}

fn run_stage(profile: &Profile, stage: &str, tool: &Tool, args: Vec<String>, root: &Path) -> Result<()> {
    let mut paths: Vec<PathBuf> = tool.closure.iter().chain(&profile.shell.closure).map(PathBuf::from).collect();
    paths.sort();
    paths.dedup();
    let mut env = BTreeMap::from([
        ("HOME".into(), "/build".into()),
        ("TMPDIR".into(), "/build/tmp".into()),
        ("LC_ALL".into(), "C".into()),
        ("LANG".into(), "C".into()),
        ("TZ".into(), "UTC".into()),
        ("SOURCE_DATE_EPOCH".into(), "0".into()),
        ("OMP_NUM_THREADS".into(), "1".into()),
        ("OPENBLAS_NUM_THREADS".into(), "1".into()),
        ("PYTHONHASHSEED".into(), "0".into()),
    ]);
    // Only admitted executables can be found by Yosys's ABC subprocess.
    let bin_dirs: Vec<String> = tool.closure.iter().map(|p| format!("{p}/bin")).collect();
    debug_assert_eq!(bin_dirs.len(), tool.closure.len());
    env.insert("PATH".into(), bin_dirs.join(":"));
    let mut shell_args = vec![
        "-c".into(),
        format!("ulimit -c 0 || exit 90; ulimit -f {} || exit 91; exec \"$@\"", profile.max_output_bytes / 1024),
        "fpga-stage".into(),
        tool.executable.path.clone(),
    ];
    shell_args.extend(args);
    debug_assert!(!shell_args.is_empty());
    fs::create_dir(root.join("tmp")).map_err(|e| e.to_string())?;
    let result = run_strict_action(StrictActionRequest {
        bwrap: PathBuf::from(&profile.bwrap.executable.path),
        executable: PathBuf::from(&profile.shell.executable.path),
        args: shell_args,
        env,
        read_only_paths: paths,
        read_only_bindings: vec![ReadOnlyBinding {
            source: PathBuf::from(&profile.shell.executable.path),
            guest: PathBuf::from("/bin/sh"),
        }],
        writable_root: root.to_path_buf(),
        timeout_ms: profile.timeout_ms,
        max_log_bytes: profile.max_log_bytes,
    })
    .map_err(|e| format!("{stage}: {e}"))?;
    if result.exit_code != Some(0) {
        return Err(format!("{stage}-failed:{:?}:{}", result.exit_code, String::from_utf8_lossy(&result.stderr)));
    }
    check_stage_bounds(root, profile.max_output_bytes)?;
    Ok(())
}

fn check_stage_bounds(root: &Path, maximum_bytes: u64) -> Result<()> {
    let mut pending = vec![root.to_path_buf()];
    let mut total_bytes = 0_u64;
    let mut count = 0_u32;
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            count += 1;
            if count > 256 {
                return Err("stage-entry-count-bound".into());
            }
            let entry = entry.map_err(|e| e.to_string())?;
            let metadata = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total_bytes = total_bytes.checked_add(metadata.len()).ok_or("stage-byte-overflow")?;
                if total_bytes > maximum_bytes {
                    return Err("stage-total-output-byte-bound".into());
                }
            } else {
                return Err("stage-produced-nonregular-output".into());
            }
        }
    }
    debug_assert!(count <= 256);
    debug_assert!(total_bytes <= maximum_bytes);
    Ok(())
}

fn copy_artifact(source: &Path, destination: &Path, maximum_bytes: u64) -> Result<()> {
    write_new(destination, &read_bounded(source, maximum_bytes)?)
}

pub fn execute(profile: Profile, root: &Path) -> Result<Manifest> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err("unsupported-host-platform-requires-x86_64-linux".into());
    }
    if !root.is_absolute() {
        return Err("absolute-fresh-output-root-required".into());
    }
    debug_assert!(root.is_absolute());
    let PlannedProfile {
        profile,
        stages,
        synthesis_script,
        constraints,
    } = crunch_fpga_core::identities(profile).map_err(|error| error.to_string())?;
    debug_assert_eq!(stages.synthesis.len(), 64);
    prepare(
        &profile,
        &Recipe {
            synthesis_script: &synthesis_script,
            constraints: &constraints,
        },
        root,
    )?;
    let synthesis = root.join("synthesis");
    run_stage(
        &profile,
        "synthesis",
        &profile.yosys,
        vec![
            "-Q".into(),
            "-T".into(),
            "-s".into(),
            profile.synthesis_script.name.clone(),
        ],
        &synthesis,
    )?;
    let netlist_bytes = read_bounded(&synthesis.join("netlist.json"), profile.max_output_bytes)?;
    let NetlistRequest {
        profile,
        netlist: netlist_bytes,
    } = crunch_fpga_core::accept_netlist(NetlistRequest {
        profile,
        netlist: netlist_bytes,
    })
    .map_err(|error| error.to_string())?;
    let pnr = root.join("place-route");
    write_new(&pnr.join("netlist.json"), &netlist_bytes)?;
    run_stage(&profile, "place-route", &profile.nextpnr, pnr_args(&profile), &pnr)?;
    let facts: Report = serde_json::from_slice(&read_bounded(&pnr.join("report.json"), profile.max_output_bytes)?)
        .map_err(|e| format!("invalid-nextpnr-report:{e}"))?;
    // Packing never starts on successful exit alone: facts must pass policy.
    let ReportRequest { profile, report: facts } =
        crunch_fpga_core::accept_report(ReportRequest { profile, report: facts }).map_err(|error| error.to_string())?;
    let pack = root.join("packing");
    fs::create_dir(&pack).map_err(|e| e.to_string())?;
    copy_artifact(&pnr.join("routed.asc"), &pack.join("routed.asc"), profile.max_output_bytes)?;
    run_stage(&profile, "packing", &profile.packer, vec!["routed.asc".into(), "bitstream.bin".into()], &pack)?;
    finish(profile, stages, facts, root)
}

fn pnr_args(profile: &Profile) -> Vec<String> {
    vec![
        "--hx1k".into(),
        "--package".into(),
        "tq144".into(),
        "--json".into(),
        "netlist.json".into(),
        "--pcf".into(),
        profile.constraints.name.clone(),
        "--asc".into(),
        "routed.asc".into(),
        "--write".into(),
        "routed.json".into(),
        "--report".into(),
        "report.json".into(),
        "--seed".into(),
        profile.seed.to_string(),
        "--threads".into(),
        profile.threads.to_string(),
        "--placer".into(),
        "heap".into(),
        "--router".into(),
        "router1".into(),
        "--timing-allow-fail".into(),
    ]
}

fn finish(profile: Profile, stages: StageIdentities, facts: Report, root: &Path) -> Result<Manifest> {
    let mut artifacts = Vec::with_capacity(ARTIFACT_PATHS.len());
    for name in ARTIFACT_PATHS {
        let path = root.join(name);
        let bytes = read_bounded(&path, profile.max_output_bytes)?;
        if bytes.is_empty() && !name.ends_with(".log") {
            return Err(format!("empty-required-artifact:{name}"));
        }
        artifacts.push(Artifact {
            path: (*name).into(),
            blake3: blake3::hash(&bytes).to_hex().to_string(),
            bytes: bytes.len() as u64,
        });
    }
    debug_assert_eq!(artifacts.len(), ARTIFACT_PATHS.len());
    let manifest = Manifest {
        schema: "mantle-fpga-artifacts-v1".into(),
        profile,
        stages,
        report: facts,
        artifacts,
        synthesis_completed: true,
        declared_clock_timing_accepted: true,
        bitstream_packed: true,
        simulation_validated: false,
        physical_board_validated: false,
        non_claims: crunch_fpga_core::NON_CLAIMS.iter().map(|s| (*s).into()).collect(),
    };
    write_new(&root.join("manifest.json"), &serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)?;
    Ok(manifest)
}
