#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const SCHEMA: &str = "mantle-fpga-profile-v1";
pub const PART: &str = "ice40-hx1k";
pub const PACKAGE: &str = "tq144";
pub const MAX_INPUTS: usize = 48;
pub const MAX_CLOSURE_PATHS: usize = 256;
pub const MAX_FILE_BYTES: u64 = 268_435_456;
pub const MAX_ARTIFACT_BYTES: u64 = 16_777_216;
pub const MAX_TEXT_BYTES: usize = 1_048_576;
pub const NON_CLAIMS: &[&str] = &[
    "No board programming or physical-board validation",
    "No formal equivalence or RTL correctness claim",
    "Timing closure is only the declared nextpnr clock model, not board or IO timing",
    "Simulation observations are separate from synthesis and timing acceptance",
    "Pinned Nix tool closures are seed inputs, not source-bootstrap proof",
    "Repeat equality is only for the declared host architecture and exact cohort",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileInput {
    pub name: String,
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub executable: FileInput,
    pub version: String,
    /// Exact sorted immutable Nix closure roots; no ambient PATH or library search.
    pub closure: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedDatabase {
    pub part: String,
    pub package: String,
    /// nextpnr 0.10 ice40 embeds its chipdb in this exact executable.
    pub nextpnr_binary_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    pub port: String,
    /// Exact nextpnr report name, not an inferred or fuzzy clock alias.
    pub report_net: String,
    pub frequency_khz: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SynthesisPass {
    Check,
    Opt,
    Clean,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: String,
    pub part: String,
    pub package: String,
    pub rtl: Vec<FileInput>,
    pub includes: Vec<FileInput>,
    pub include_dirs: Vec<String>,
    pub top: String,
    pub defines: BTreeMap<String, String>,
    pub parameters: BTreeMap<String, u32>,
    /// Generated recipe bytes must equal this pinned script exactly.
    pub synthesis_script: FileInput,
    pub synthesis_passes: Vec<SynthesisPass>,
    pub flatten: bool,
    pub pins: BTreeMap<String, String>,
    pub clocks: Vec<Clock>,
    /// PCF bytes must equal the complete typed pin/clock declaration.
    pub constraints: FileInput,
    pub seed: u32,
    pub threads: u32,
    pub yosys: Tool,
    pub nextpnr: Tool,
    pub packer: Tool,
    pub shell: Tool,
    pub bwrap: Tool,
    pub database: EmbeddedDatabase,
    pub utilization_limits: BTreeMap<String, u32>,
    pub timeout_ms: u64,
    pub max_log_bytes: u64,
    pub max_output_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageIdentities {
    pub synthesis: String,
    pub place_route: String,
    pub packing: String,
}

fn invalid(message: &str) -> Result<(), String> {
    Err(message.to_string())
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
        && value.bytes().all(|c| c.is_ascii_alphanumeric() || b"_/.-[]$".contains(&c))
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().next().is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && value.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

fn nix_root(value: &str) -> bool {
    value.strip_prefix("/nix/store/").is_some_and(|tail| {
        tail.len() > 33
            && tail.as_bytes()[32] == b'-'
            && tail.bytes().all(|c| c.is_ascii_alphanumeric() || b"+-._?=".contains(&c))
    })
}

fn validate_tool(tool: &Tool, version: &str, executable: &str) -> Result<(), String> {
    if tool.version != version || tool.executable.name != executable {
        return invalid("unsupported-tool-version-or-executable");
    }
    if tool.closure.is_empty()
        || tool.closure.len() > MAX_CLOSURE_PATHS
        || !tool.closure.iter().all(|p| nix_root(p))
        || !tool.closure.windows(2).all(|w| w[0] < w[1])
    {
        return invalid("invalid-explicit-tool-closure");
    }
    if !is_digest(&tool.executable.blake3)
        || !tool.closure.iter().any(|p| tool.executable.path == format!("{p}/bin/{executable}"))
    {
        return invalid("tool-executable-outside-declared-closure");
    }
    Ok(())
}

pub fn validate(profile: Profile) -> Result<Profile, String> {
    validate_profile(&profile)?;
    Ok(profile)
}

fn validate_profile(profile: &Profile) -> Result<(), String> {
    if profile.schema != SCHEMA || profile.part != PART || profile.package != PACKAGE {
        return invalid("unsupported-schema-part-or-package");
    }
    if profile.database.part != PART
        || profile.database.package != PACKAGE
        || profile.database.nextpnr_binary_blake3 != profile.nextpnr.executable.blake3
    {
        return invalid("wrong-embedded-device-database");
    }
    validate_tool(&profile.yosys, "0.68", "yosys")?;
    validate_tool(&profile.nextpnr, "0.10", "nextpnr-ice40")?;
    validate_tool(&profile.packer, "0-unstable-2025-06-03", "icepack")?;
    validate_tool(&profile.shell, "1.37.0", "sh")?;
    validate_tool(&profile.bwrap, &profile.bwrap.version, "bwrap")?;
    if profile.bwrap.version.is_empty() {
        return invalid("missing-bwrap-version");
    }
    validate_inputs(profile)?;
    validate_constraints(profile)?;
    if !identifier(&profile.top) || profile.threads != 1 || profile.seed == 0 {
        return invalid("invalid-top-seed-or-threads-only-one-supported");
    }
    if profile.timeout_ms == 0
        || profile.timeout_ms > 600_000
        || profile.max_log_bytes == 0
        || profile.max_log_bytes > MAX_TEXT_BYTES as u64
        || profile.max_output_bytes < 1024
        || profile.max_output_bytes > MAX_ARTIFACT_BYTES
    {
        return invalid("invalid-stage-bounds");
    }
    if profile.synthesis_passes.len() > 16
        || profile.defines.len() > 32
        || profile.parameters.len() > 32
        || profile.include_dirs.len() > 32
        || profile.utilization_limits.is_empty()
        || profile.utilization_limits.len() > 32
    {
        return invalid("option-count-bound");
    }
    for (name, value) in &profile.defines {
        if !identifier(name) || value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
            return invalid("unsupported-define-requires-identifier-and-unsigned-decimal");
        }
    }
    if !profile.parameters.keys().all(|n| identifier(n)) || !profile.include_dirs.iter().all(|n| safe_name(n)) {
        return invalid("invalid-parameter-or-include-directory");
    }
    Ok(())
}

fn validate_inputs(profile: &Profile) -> Result<(), String> {
    if profile.rtl.is_empty() || profile.rtl.len() + profile.includes.len() + 2 > MAX_INPUTS {
        return invalid("source-count-bound");
    }
    let mut names = BTreeSet::new();
    for file in profile.rtl.iter().chain(&profile.includes).chain([&profile.synthesis_script, &profile.constraints]) {
        if !safe_name(&file.name)
            || !file.path.starts_with('/')
            || !is_digest(&file.blake3)
            || !names.insert(&file.name)
            || file.name == "netlist.json"
            || file.name == "routed.asc"
            || file.name == "stdout.log"
            || file.name == "stderr.log"
        {
            return invalid("invalid-or-duplicate-source-input");
        }
    }
    Ok(())
}

fn validate_constraints(profile: &Profile) -> Result<(), String> {
    if profile.pins.is_empty() || profile.pins.len() > 96 || profile.clocks.is_empty() || profile.clocks.len() > 8 {
        return invalid("missing-or-excess-pin-clock-constraints");
    }
    let mut used = BTreeSet::new();
    for (port, pin) in &profile.pins {
        if !safe_name(port) || !pin.bytes().all(|c| c.is_ascii_digit()) || pin.is_empty() || !used.insert(pin) {
            return invalid("invalid-or-duplicate-pin-constraint");
        }
    }
    let mut clocks = BTreeSet::new();
    let mut reports = BTreeSet::new();
    for clock in &profile.clocks {
        if !profile.pins.contains_key(&clock.port)
            || !identifier(&clock.port)
            || !safe_name(&clock.report_net)
            || clock.frequency_khz == 0
            || clock.frequency_khz > 1_000_000
            || !clocks.insert(&clock.port)
            || !reports.insert(&clock.report_net)
        {
            return invalid("invalid-or-unconstrained-required-clock");
        }
    }
    Ok(())
}

fn digest<T: Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_TEXT_BYTES {
        return Err("identity-byte-bound".to_string());
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

/// An admitted profile and its pure stage identities and exact recipe bytes.
#[derive(Debug)]
pub struct PlannedProfile {
    pub profile: Profile,
    pub stages: StageIdentities,
    pub synthesis_script: String,
    pub constraints: String,
}

/// Merkle stages deliberately exclude downstream configuration from synthesis.
pub fn identities(profile: Profile) -> Result<PlannedProfile, String> {
    validate_profile(&profile)?;
    let synthesis = digest(
        "mantle.fpga.synthesis.v1",
        &(
            &profile.rtl,
            &profile.includes,
            &profile.include_dirs,
            &profile.top,
            &profile.defines,
            &profile.parameters,
            &profile.synthesis_script,
            &profile.synthesis_passes,
            profile.flatten,
            &profile.yosys,
            &profile.shell,
            &profile.bwrap,
            profile.timeout_ms,
            profile.max_log_bytes,
            profile.max_output_bytes,
        ),
    )?;
    let place_route = digest(
        "mantle.fpga.place-route.v1",
        &(
            &synthesis,
            &profile.part,
            &profile.package,
            &profile.database,
            &profile.nextpnr,
            &profile.constraints,
            &profile.pins,
            &profile.clocks,
            profile.seed,
            profile.threads,
            &profile.utilization_limits,
        ),
    )?;
    let packing = digest("mantle.fpga.packing.v1", &(&place_route, &profile.packer))?;
    let synthesis_script = synthesis_script(&profile);
    let constraints = constraints(&profile);
    Ok(PlannedProfile {
        profile,
        stages: StageIdentities {
            synthesis,
            place_route,
            packing,
        },
        synthesis_script,
        constraints,
    })
}

fn synthesis_script(profile: &Profile) -> String {
    let mut script = String::from("read_verilog -sv");
    for (name, value) in &profile.defines {
        script.push_str(&format!(" -D{name}={value}"));
    }
    for directory in &profile.include_dirs {
        script.push_str(&format!(" -I{directory}"));
    }
    for file in &profile.rtl {
        script.push_str(&format!(" {}", file.name));
    }
    script.push('\n');
    for (name, value) in &profile.parameters {
        script.push_str(&format!("chparam -set {name} {value} {}\n", profile.top));
    }
    for pass in &profile.synthesis_passes {
        script.push_str(match pass {
            SynthesisPass::Check => "check\n",
            SynthesisPass::Opt => "opt\n",
            SynthesisPass::Clean => "clean\n",
        });
    }
    script.push_str(&format!(
        "synth_ice40 -top {}{} -json netlist.json\ncheck -assert\n",
        profile.top,
        if profile.flatten { "" } else { " -noflatten" }
    ));
    script
}

fn constraints(profile: &Profile) -> String {
    let mut pcf = String::new();
    for (port, pin) in &profile.pins {
        pcf.push_str(&format!("set_io {port} {pin}\n"));
    }
    for clock in &profile.clocks {
        pcf.push_str(&format!(
            "set_frequency {} {}.{:03}\n",
            clock.port,
            clock.frequency_khz / 1000,
            clock.frequency_khz % 1000
        ));
    }
    pcf
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrequencyFact {
    pub achieved: f64,
    pub constraint: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtilizationFact {
    pub used: u32,
    pub available: u32,
}

/// Owned JSON facts retained verbatim in the report wire representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub utilization: BTreeMap<String, UtilizationFact>,
    pub fmax: BTreeMap<String, FrequencyFact>,
    pub critical_paths: Vec<JsonValue>,
    #[serde(default)]
    pub detailed_net_timings: Vec<JsonValue>,
}

#[derive(Debug)]
pub struct ReportRequest {
    pub profile: Profile,
    pub report: Report,
}

pub fn accept_report(request: ReportRequest) -> Result<ReportRequest, String> {
    validate_profile(&request.profile)?;
    accept_report_facts(&request.profile, &request.report)?;
    Ok(request)
}

fn accept_report_facts(profile: &Profile, report: &Report) -> Result<(), String> {
    if report.fmax.len() != profile.clocks.len()
        || report.utilization.is_empty()
        || report.utilization.len() > 64
        || report.critical_paths.len() > 256
    {
        return invalid("missing-or-unconstrained-clock-report");
    }
    for clock in &profile.clocks {
        let fact = report
            .fmax
            .get(&clock.report_net)
            .ok_or_else(|| format!("required-clock-not-reported:{}", clock.report_net))?;
        let target = f64::from(clock.frequency_khz) / 1000.0;
        if !fact.achieved.is_finite()
            || !fact.constraint.is_finite()
            || fact.achieved <= 0.0
            || (fact.constraint - target).abs() > 0.000_1
        {
            return Err(format!("wrong-or-missing-clock-constraint:{}", clock.report_net));
        }
        if fact.achieved < target {
            return Err(format!("declared-timing-violation:{}:{}<{target}", clock.report_net, fact.achieved));
        }
    }
    for (name, fact) in &report.utilization {
        if fact.used > fact.available {
            return Err(format!("device-capacity-exceeded:{name}"));
        }
    }
    for (name, limit) in &profile.utilization_limits {
        let fact = report.utilization.get(name).ok_or_else(|| format!("required-utilization-not-reported:{name}"))?;
        if fact.used > *limit {
            return Err(format!("declared-utilization-violation:{name}"));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct NetlistRequest {
    pub profile: Profile,
    pub netlist: Vec<u8>,
}

/// Ensure every actual top-level bit has a declared pin; no phantom PCF entries.
pub fn accept_netlist(request: NetlistRequest) -> Result<NetlistRequest, String> {
    validate_profile(&request.profile)?;
    let netlist = serde_json::from_slice(&request.netlist).map_err(|error| error.to_string())?;
    accept_netlist_ports(&request.profile, &netlist)?;
    Ok(request)
}

fn accept_netlist_ports(profile: &Profile, netlist: &serde_json::Value) -> Result<(), String> {
    let ports = netlist
        .get("modules")
        .and_then(|v| v.get(&profile.top))
        .and_then(|v| v.get("ports"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "missing-synthesized-top-ports".to_string())?;
    if ports.is_empty() || ports.len() > 96 {
        return invalid("netlist-port-count-bound");
    }
    let mut actual = BTreeSet::new();
    for (name, port) in ports {
        let bits = port
            .get("bits")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "invalid-synthesized-port".to_string())?;
        if bits.is_empty() || bits.len() > 96 || port.get("upto").and_then(serde_json::Value::as_u64).unwrap_or(0) != 0
        {
            return invalid("unsupported-empty-ascending-or-wide-port");
        }
        let offset = port.get("offset").and_then(serde_json::Value::as_u64).unwrap_or(0);
        if offset > u64::from(u32::MAX) {
            return invalid("unsupported-port-offset");
        }
        for index in 0..bits.len() {
            let pin = if bits.len() == 1 {
                name.clone()
            } else {
                format!("{name}[{}]", offset + index as u64)
            };
            actual.insert(pin);
            if actual.len() > 96 {
                return invalid("netlist-pin-count-bound");
            }
        }
    }
    if actual != profile.pins.keys().cloned().collect() {
        return invalid("unconstrained-or-unknown-required-pin");
    }
    for clock in &profile.clocks {
        let port = ports.get(&clock.port).ok_or_else(|| "missing-required-clock-port".to_string())?;
        if port.get("direction").and_then(serde_json::Value::as_str) != Some("input") {
            return invalid("required-clock-is-not-input");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
