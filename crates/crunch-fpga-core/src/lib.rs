// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
//
// `Report::detailed_net_timings` keeps `#[serde(default)]`: serde emits
// `Default::default()` for the absent field, and dropping the attribute would
// reject report documents that omit this optional field.
#![allow(tigerstyle::explicit_defaults)]
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
use serde_json::Error;

pub const SCHEMA: &str = "mantle-fpga-profile-v1";
pub const PART: &str = "ice40-hx1k";
pub const PACKAGE: &str = "tq144";
pub const MAX_INPUTS: usize = 48;
pub const MAX_CLOSURE_PATHS: usize = 256;
pub const MAX_FILE_BYTES: u64 = 268_435_456;
pub const MAX_ARTIFACT_BYTES: u64 = 16_777_216;
pub const MAX_TEXT_BYTES: usize = 1_048_576;
/// The artifact names a synthesis stage owns, and no source may claim.
const RESERVED_NAMES: &[&str] = &["netlist.json", "routed.asc", "stdout.log", "stderr.log"];
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

/// Profile, identity and measured-output failures at the pure FPGA boundary.
#[derive(Debug)]
pub enum AdmissionError {
    UnsupportedTool,
    ToolClosure,
    ToolExecutable,
    UnsupportedDevice,
    EmbeddedDatabase,
    MissingBwrapVersion,
    TopSeedOrThreads,
    StageBounds,
    OptionCount,
    Define,
    ParameterOrIncludeDirectory,
    SourceCount,
    SourceInput,
    PinClockConstraints,
    PinConstraint,
    RequiredClock,
    IdentitySerialization(Error),
    IdentityByteBound,
    ClockReport,
    ClockNotReported(String),
    ClockConstraint(String),
    TimingViolation { clock: String, achieved: f64, target: f64 },
    DeviceCapacity(String),
    UtilizationNotReported(String),
    UtilizationViolation(String),
    NetlistJson(Error),
    MissingTopPorts,
    PortCount,
    InvalidPort,
    UnsupportedPortShape,
    PortOffset,
    PinCount,
    UnconstrainedPin,
    MissingClockPort,
    ClockNotInput,
}

impl core::fmt::Display for AdmissionError {
    fn fmt(&self, output: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedTool => output.write_str("unsupported-tool-version-or-executable"),
            Self::ToolClosure => output.write_str("invalid-explicit-tool-closure"),
            Self::ToolExecutable => output.write_str("tool-executable-outside-declared-closure"),
            Self::UnsupportedDevice => output.write_str("unsupported-schema-part-or-package"),
            Self::EmbeddedDatabase => output.write_str("wrong-embedded-device-database"),
            Self::MissingBwrapVersion => output.write_str("missing-bwrap-version"),
            Self::TopSeedOrThreads => output.write_str("invalid-top-seed-or-threads-only-one-supported"),
            Self::StageBounds => output.write_str("invalid-stage-bounds"),
            Self::OptionCount => output.write_str("option-count-bound"),
            Self::Define => output.write_str("unsupported-define-requires-identifier-and-unsigned-decimal"),
            Self::ParameterOrIncludeDirectory => output.write_str("invalid-parameter-or-include-directory"),
            Self::SourceCount => output.write_str("source-count-bound"),
            Self::SourceInput => output.write_str("invalid-or-duplicate-source-input"),
            Self::PinClockConstraints => output.write_str("missing-or-excess-pin-clock-constraints"),
            Self::PinConstraint => output.write_str("invalid-or-duplicate-pin-constraint"),
            Self::RequiredClock => output.write_str("invalid-or-unconstrained-required-clock"),
            Self::IdentitySerialization(error) | Self::NetlistJson(error) => write!(output, "{error}"),
            Self::IdentityByteBound => output.write_str("identity-byte-bound"),
            Self::ClockReport => output.write_str("missing-or-unconstrained-clock-report"),
            Self::ClockNotReported(clock) => write!(output, "required-clock-not-reported:{clock}"),
            Self::ClockConstraint(clock) => write!(output, "wrong-or-missing-clock-constraint:{clock}"),
            Self::TimingViolation {
                clock,
                achieved,
                target,
            } => write!(output, "declared-timing-violation:{clock}:{achieved}<{target}"),
            Self::DeviceCapacity(name) => write!(output, "device-capacity-exceeded:{name}"),
            Self::UtilizationNotReported(name) => write!(output, "required-utilization-not-reported:{name}"),
            Self::UtilizationViolation(name) => write!(output, "declared-utilization-violation:{name}"),
            Self::MissingTopPorts => output.write_str("missing-synthesized-top-ports"),
            Self::PortCount => output.write_str("netlist-port-count-bound"),
            Self::InvalidPort => output.write_str("invalid-synthesized-port"),
            Self::UnsupportedPortShape => output.write_str("unsupported-empty-ascending-or-wide-port"),
            Self::PortOffset => output.write_str("unsupported-port-offset"),
            Self::PinCount => output.write_str("netlist-pin-count-bound"),
            Self::UnconstrainedPin => output.write_str("unconstrained-or-unknown-required-pin"),
            Self::MissingClockPort => output.write_str("missing-required-clock-port"),
            Self::ClockNotInput => output.write_str("required-clock-is-not-input"),
        }
    }
}

impl core::error::Error for AdmissionError {}

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

/// The exact version and executable name a declared tool must carry.
struct ToolExpectation<'a> {
    version: &'a str,
    executable: &'a str,
}

impl<'a> ToolExpectation<'a> {
    fn new(version: &'a str, executable: &'a str) -> Self {
        Self { version, executable }
    }
}

/// One declared tool must match its expectation and stay inside its closure.
fn validate_tool(tool: &Tool, expectation: &ToolExpectation<'_>) -> Result<(), AdmissionError> {
    let is_version = tool.version == expectation.version;
    let is_executable = tool.executable.name == expectation.executable;
    if [is_version, is_executable].iter().any(|fact| !*fact) {
        return Err(AdmissionError::UnsupportedTool);
    }
    let is_closure_present = !tool.closure.is_empty();
    let is_closure_bounded = tool.closure.len() <= MAX_CLOSURE_PATHS;
    let is_closure_admitted = tool.closure.iter().all(|path| nix_root(path));
    let is_closure_sorted = tool.closure.windows(2).all(|pair| pair[0] < pair[1]);
    if [
        is_closure_present,
        is_closure_bounded,
        is_closure_admitted,
        is_closure_sorted,
    ]
    .iter()
    .any(|fact| !*fact)
    {
        return Err(AdmissionError::ToolClosure);
    }
    debug_assert!(tool.closure.windows(2).all(|pair| pair[0] < pair[1]));
    let is_executable_digest = is_digest(&tool.executable.blake3);
    let is_executable_admitted = tool
        .closure
        .iter()
        .any(|path| tool.executable.path == format!("{path}/bin/{}", expectation.executable));
    if [is_executable_digest, is_executable_admitted].iter().any(|fact| !*fact) {
        return Err(AdmissionError::ToolExecutable);
    }
    debug_assert!(is_digest(&tool.executable.blake3));
    Ok(())
}

pub fn validate(profile: Profile) -> Result<Profile, AdmissionError> {
    validate_profile(&profile)?;
    Ok(profile)
}

fn validate_profile(profile: &Profile) -> Result<(), AdmissionError> {
    validate_identity(profile)?;
    validate_tools(profile)?;
    validate_inputs(profile)?;
    validate_constraints(profile)?;
    validate_bounds(profile)
}

/// The profile identity and the embedded device database it must carry.
fn validate_identity(profile: &Profile) -> Result<(), AdmissionError> {
    let is_schema = profile.schema == SCHEMA;
    let is_part = profile.part == PART;
    let is_package = profile.package == PACKAGE;
    if [is_schema, is_part, is_package].iter().any(|fact| !*fact) {
        return Err(AdmissionError::UnsupportedDevice);
    }
    let is_database_part = profile.database.part == PART;
    let is_database_package = profile.database.package == PACKAGE;
    let is_database_digest = profile.database.nextpnr_binary_blake3 == profile.nextpnr.executable.blake3;
    if [is_database_part, is_database_package, is_database_digest].iter().any(|fact| !*fact) {
        return Err(AdmissionError::EmbeddedDatabase);
    }
    debug_assert_eq!(profile.part, PART);
    debug_assert_eq!(profile.package, PACKAGE);
    Ok(())
}

/// The declared toolchain must match the admitted cohort exactly.
fn validate_tools(profile: &Profile) -> Result<(), AdmissionError> {
    validate_tool(&profile.yosys, &ToolExpectation::new("0.68", "yosys"))?;
    validate_tool(&profile.nextpnr, &ToolExpectation::new("0.10", "nextpnr-ice40"))?;
    validate_tool(&profile.packer, &ToolExpectation::new("0-unstable-2025-06-03", "icepack"))?;
    validate_tool(&profile.shell, &ToolExpectation::new("1.37.0", "sh"))?;
    validate_tool(&profile.bwrap, &ToolExpectation::new(&profile.bwrap.version, "bwrap"))?;
    if profile.bwrap.version.is_empty() {
        return Err(AdmissionError::MissingBwrapVersion);
    }
    Ok(())
}

/// The stage bounds, option counts and option values.
fn validate_bounds(profile: &Profile) -> Result<(), AdmissionError> {
    let is_top = identifier(&profile.top);
    let is_threads = profile.threads == 1;
    let is_seed = profile.seed != 0;
    if [is_top, is_threads, is_seed].iter().any(|fact| !*fact) {
        return Err(AdmissionError::TopSeedOrThreads);
    }
    let has_timeout_ms = profile.timeout_ms != 0;
    let is_admitted_timeout_ms = profile.timeout_ms <= 600_000;
    let is_log_present = profile.max_log_bytes != 0;
    let is_log_bounded = profile.max_log_bytes <= MAX_TEXT_BYTES as u64;
    let is_output_present = profile.max_output_bytes >= 1024;
    let is_output_bounded = profile.max_output_bytes <= MAX_ARTIFACT_BYTES;
    if [
        has_timeout_ms,
        is_admitted_timeout_ms,
        is_log_present,
        is_log_bounded,
        is_output_present,
        is_output_bounded,
    ]
    .iter()
    .any(|fact| !*fact)
    {
        return Err(AdmissionError::StageBounds);
    }
    debug_assert!(profile.timeout_ms != 0);
    debug_assert!(profile.max_output_bytes >= 1024);
    let is_passes_bounded = profile.synthesis_passes.len() <= 16;
    let is_defines_bounded = profile.defines.len() <= 32;
    let is_parameters_bounded = profile.parameters.len() <= 32;
    let is_include_dirs_bounded = profile.include_dirs.len() <= 32;
    let is_utilization_present = !profile.utilization_limits.is_empty();
    let is_utilization_bounded = profile.utilization_limits.len() <= 32;
    if [
        is_passes_bounded,
        is_defines_bounded,
        is_parameters_bounded,
        is_include_dirs_bounded,
        is_utilization_present,
        is_utilization_bounded,
    ]
    .iter()
    .any(|fact| !*fact)
    {
        return Err(AdmissionError::OptionCount);
    }
    for (name, value) in &profile.defines {
        let is_name = identifier(name);
        let is_value = !value.is_empty();
        let is_decimal = value.bytes().all(|byte| byte.is_ascii_digit());
        if [is_name, is_value, is_decimal].iter().any(|fact| !*fact) {
            return Err(AdmissionError::Define);
        }
    }
    let is_parameters = profile.parameters.keys().all(|name| identifier(name));
    let is_include_dirs = profile.include_dirs.iter().all(|name| safe_name(name));
    if [is_parameters, is_include_dirs].iter().any(|fact| !*fact) {
        return Err(AdmissionError::ParameterOrIncludeDirectory);
    }
    Ok(())
}

fn validate_inputs(profile: &Profile) -> Result<(), AdmissionError> {
    let source_count = profile.rtl.len().saturating_add(profile.includes.len());
    let is_source_present = !profile.rtl.is_empty();
    let is_source_bounded = source_count.saturating_add(2) <= MAX_INPUTS;
    if [is_source_present, is_source_bounded].iter().any(|fact| !*fact) {
        return Err(AdmissionError::SourceCount);
    }
    let mut names = BTreeSet::new();
    for file in profile.rtl.iter().chain(&profile.includes).chain([&profile.synthesis_script, &profile.constraints]) {
        let is_name = safe_name(&file.name);
        let is_absolute = file.path.starts_with('/');
        let is_digest_value = is_digest(&file.blake3);
        let is_unique = names.insert(&file.name);
        let is_unreserved = !RESERVED_NAMES.contains(&file.name.as_str());
        if [is_name, is_absolute, is_digest_value, is_unique, is_unreserved].iter().any(|fact| !*fact) {
            return Err(AdmissionError::SourceInput);
        }
    }
    debug_assert!(!names.is_empty());
    debug_assert_eq!(names.len(), source_count.saturating_add(2));
    Ok(())
}

fn validate_constraints(profile: &Profile) -> Result<(), AdmissionError> {
    let is_pins_present = !profile.pins.is_empty();
    let is_pins_bounded = profile.pins.len() <= 96;
    let is_clocks_present = !profile.clocks.is_empty();
    let is_clocks_bounded = profile.clocks.len() <= 8;
    if [is_pins_present, is_pins_bounded, is_clocks_present, is_clocks_bounded].iter().any(|fact| !*fact) {
        return Err(AdmissionError::PinClockConstraints);
    }
    let mut used = BTreeSet::new();
    for (port, pin) in &profile.pins {
        let is_name = safe_name(port);
        let is_numeric = pin.bytes().all(|byte| byte.is_ascii_digit());
        let is_present = !pin.is_empty();
        let is_unique = used.insert(pin);
        if [is_name, is_numeric, is_present, is_unique].iter().any(|fact| !*fact) {
            return Err(AdmissionError::PinConstraint);
        }
    }
    debug_assert!(!used.is_empty());
    debug_assert_eq!(used.len(), profile.pins.len());
    let mut clocks = BTreeSet::new();
    let mut clock_nets = BTreeSet::new();
    for clock in &profile.clocks {
        let is_constrained = profile.pins.contains_key(&clock.port);
        let is_name = identifier(&clock.port);
        let is_net = safe_name(&clock.report_net);
        let is_frequency_present = clock.frequency_khz != 0;
        let is_frequency_bounded = clock.frequency_khz <= 1_000_000;
        let is_unique_clock = clocks.insert(&clock.port);
        let is_unique_net = clock_nets.insert(&clock.report_net);
        if [
            is_constrained,
            is_name,
            is_net,
            is_frequency_present,
            is_frequency_bounded,
            is_unique_clock,
            is_unique_net,
        ]
        .iter()
        .any(|fact| !*fact)
        {
            return Err(AdmissionError::RequiredClock);
        }
    }
    debug_assert_eq!(clocks.len(), profile.clocks.len());
    Ok(())
}

fn digest<T: Serialize>(domain: &str, value: &T) -> Result<String, AdmissionError> {
    let bytes = serde_json::to_vec(value).map_err(AdmissionError::IdentitySerialization)?;
    if bytes.len() > MAX_TEXT_BYTES {
        return Err(AdmissionError::IdentityByteBound);
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
pub fn identities(profile: Profile) -> Result<PlannedProfile, AdmissionError> {
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
    debug_assert_eq!(synthesis.len(), 64);
    debug_assert_eq!(place_route.len(), 64);
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
    debug_assert!(script.starts_with("read_verilog -sv"));
    debug_assert!(script.ends_with("check -assert\n"));
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

pub fn accept_report(request: ReportRequest) -> Result<ReportRequest, AdmissionError> {
    validate_profile(&request.profile)?;
    accept_report_facts(&request.profile, &request.report)?;
    Ok(request)
}

fn accept_report_facts(profile: &Profile, report: &Report) -> Result<(), AdmissionError> {
    let is_clock_count = report.fmax.len() == profile.clocks.len();
    let is_utilization_present = !report.utilization.is_empty();
    let is_utilization_bounded = report.utilization.len() <= 64;
    let is_paths_bounded = report.critical_paths.len() <= 256;
    if [
        is_clock_count,
        is_utilization_present,
        is_utilization_bounded,
        is_paths_bounded,
    ]
    .iter()
    .any(|fact| !*fact)
    {
        return Err(AdmissionError::ClockReport);
    }
    debug_assert_eq!(report.fmax.len(), profile.clocks.len());
    debug_assert!(!report.utilization.is_empty());
    for clock in &profile.clocks {
        let fact = report
            .fmax
            .get(&clock.report_net)
            .ok_or_else(|| AdmissionError::ClockNotReported(clock.report_net.clone()))?;
        let target = f64::from(clock.frequency_khz) / 1000.0;
        let is_achieved_finite = fact.achieved.is_finite();
        let is_constraint_finite = fact.constraint.is_finite();
        let is_achieved_positive = fact.achieved > 0.0;
        let is_constraint_matched = (fact.constraint - target).abs() <= 0.000_1;
        if [
            is_achieved_finite,
            is_constraint_finite,
            is_achieved_positive,
            is_constraint_matched,
        ]
        .iter()
        .any(|fact| !*fact)
        {
            return Err(AdmissionError::ClockConstraint(clock.report_net.clone()));
        }
        if fact.achieved < target {
            return Err(AdmissionError::TimingViolation {
                clock: clock.report_net.clone(),
                achieved: fact.achieved,
                target,
            });
        }
    }
    for (name, fact) in &report.utilization {
        if fact.used > fact.available {
            return Err(AdmissionError::DeviceCapacity(name.clone()));
        }
    }
    for (name, limit) in &profile.utilization_limits {
        let fact = report.utilization.get(name).ok_or_else(|| AdmissionError::UtilizationNotReported(name.clone()))?;
        if fact.used > *limit {
            return Err(AdmissionError::UtilizationViolation(name.clone()));
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
pub fn accept_netlist(request: NetlistRequest) -> Result<NetlistRequest, AdmissionError> {
    validate_profile(&request.profile)?;
    let netlist = serde_json::from_slice(&request.netlist).map_err(AdmissionError::NetlistJson)?;
    accept_netlist_ports(&request.profile, &netlist)?;
    Ok(request)
}

fn accept_netlist_ports(profile: &Profile, netlist: &serde_json::Value) -> Result<(), AdmissionError> {
    // The synthesized top level, described by one entry per declared port.
    let top_descriptors = netlist
        .get("modules")
        .and_then(|v| v.get(&profile.top))
        .and_then(|v| v.get("ports"))
        .and_then(serde_json::Value::as_object)
        .ok_or(AdmissionError::MissingTopPorts)?;
    let is_present = !top_descriptors.is_empty();
    let is_bounded = top_descriptors.len() <= 96;
    if [is_present, is_bounded].iter().any(|fact| !*fact) {
        return Err(AdmissionError::PortCount);
    }
    let mut actual = BTreeSet::new();
    for (name, descriptor) in top_descriptors {
        let bits = descriptor.get("bits").and_then(serde_json::Value::as_array).ok_or(AdmissionError::InvalidPort)?;
        let is_bits_present = !bits.is_empty();
        let is_bits_bounded = bits.len() <= 96;
        let is_explicit_bits = descriptor.get("upto").and_then(serde_json::Value::as_u64).unwrap_or(0) == 0;
        if [is_bits_present, is_bits_bounded, is_explicit_bits].iter().any(|fact| !*fact) {
            return Err(AdmissionError::UnsupportedPortShape);
        }
        let first_bit = descriptor.get("offset").and_then(serde_json::Value::as_u64).unwrap_or(0);
        if first_bit > u64::from(u32::MAX) {
            return Err(AdmissionError::PortOffset);
        }
        for index in 0..bits.len() {
            let pin = if bits.len() == 1 {
                name.clone()
            } else {
                let bit = first_bit.checked_add(index as u64).ok_or(AdmissionError::PortOffset)?;
                format!("{name}[{bit}]")
            };
            actual.insert(pin);
            if actual.len() > 96 {
                return Err(AdmissionError::PinCount);
            }
        }
    }
    debug_assert!(!actual.is_empty());
    debug_assert!(actual.len() <= 96);
    if actual != profile.pins.keys().cloned().collect() {
        return Err(AdmissionError::UnconstrainedPin);
    }
    for clock in &profile.clocks {
        let descriptor = top_descriptors.get(&clock.port).ok_or(AdmissionError::MissingClockPort)?;
        if descriptor.get("direction").and_then(serde_json::Value::as_str) != Some("input") {
            return Err(AdmissionError::ClockNotInput);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
