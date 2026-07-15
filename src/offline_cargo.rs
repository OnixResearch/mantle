use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub const OFFLINE_CARGO_EVIDENCE_SCHEMA_V1: &str = "mantle-offline-cargo-build-evidence-v1";
pub const OFFLINE_CARGO_EVIDENCE_SCHEMA_V2: &str = "mantle-offline-cargo-build-evidence-v2";
pub const OFFLINE_CARGO_EVIDENCE_SCHEMA: &str = OFFLINE_CARGO_EVIDENCE_SCHEMA_V2;
pub const OFFLINE_CARGO_EVIDENCE_VERSION_V1: u32 = 1;
pub const OFFLINE_CARGO_EVIDENCE_VERSION_V2: u32 = 2;
pub const OFFLINE_CARGO_EVIDENCE_KIND_LEGACY: &str = "legacy-path-evidence";
pub const OFFLINE_CARGO_EVIDENCE_KIND_DIGEST_BOUND: &str = "digest-bound-evidence";
pub const OFFLINE_CARGO_EVIDENCE_CLASS: &str = "cargo-inside-mantle-sandbox";
pub const OFFLINE_CARGO_PROJECT_BUILD_STATUS: &str = "default-project-build-lane";
pub const OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH: &str = "share/mantle/offline-cargo-build.json";
pub const OFFLINE_CARGO_COMMAND_PROGRAM: &str = "cargo";
pub const OFFLINE_CARGO_NETWORK_MODE: &str = "offline";
pub const OFFLINE_CARGO_NETWORK_RESULT: &str = "undeclared-network-denied";
pub const OFFLINE_CARGO_LOCKFILE_ROLE: &str = "cargo-lockfile";
pub const OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT: &str = "blake3-content";
pub const OFFLINE_CARGO_IDENTITY_STORE_PATH: &str = "store-path";

const MAX_SOURCE_CLOSURE_ENTRIES: usize = 32;
const MAX_NAME_BYTES: usize = 128;
const MAX_PATH_BYTES: usize = 4_096;
const MAX_COMMAND_ARGS: usize = 16;
const BLAKE3_HEX_BYTES: usize = 64;
const MIN_TARGET_SEGMENTS: usize = 3;
const REQUIRED_SOURCE_ROLES: [&str; 4] = ["package-source", "rust-toolchain", "seed-toolchain", "musl-runtime"];
const REQUIRED_DIGEST_SOURCE_ROLES: [&str; 2] = ["package-source", "vendored-dependencies"];
const OFFLINE_CARGO_NON_CLAIMS: [&str; 5] = [
    "not-cargo-free-execution",
    "not-full-cargo-compatibility",
    "not-compiler-correctness",
    "not-release-reproducibility",
    "not-bootstrap-correctness",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustOfflineCargoPackageRequest {
    pub package_name: String,
    pub binary_name: String,
    pub target_triple: String,
    pub profile: String,
    pub lockfile_digest_blake3: Option<String>,
    pub expected_lockfile_digest_blake3: Option<String>,
    pub allow_network: bool,
    pub source_closure: Vec<RustOfflineCargoSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustOfflineCargoSource {
    pub role: String,
    pub name: String,
    pub digest_blake3: Option<String>,
    pub expected_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustOfflineCargoPlan {
    pub ready: bool,
    pub package_name: String,
    pub binary_name: String,
    pub target_triple: String,
    pub profile: String,
    pub claim_class: String,
    pub project_build_status: String,
    pub source_closure: Vec<RustOfflineCargoSource>,
    pub non_claims: Vec<String>,
    pub blockers: Vec<RustOfflineCargoBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustOfflineCargoBlocker {
    pub class: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceV1File {
    pub schema: String,
    pub claim_class: String,
    pub project_build_status: String,
    pub source_closure: Vec<OfflineCargoEvidenceInput>,
    pub toolchain: OfflineCargoEvidenceToolchain,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceV2File {
    pub schema: String,
    pub evidence_version: u32,
    pub claim_class: String,
    pub project_build_status: String,
    pub target: String,
    pub profile: String,
    pub binary: String,
    pub lockfile: OfflineCargoEvidenceInput,
    pub source_closure: Vec<OfflineCargoEvidenceInput>,
    pub toolchain: OfflineCargoEvidenceToolchain,
    pub cargo_command: OfflineCargoEvidenceCommand,
    pub network_policy: OfflineCargoEvidenceNetworkPolicy,
    pub output: OfflineCargoEvidenceOutput,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceInput {
    pub role: String,
    pub path: String,
    #[serde(default = "no_evidence_digest", skip_serializing_if = "Option::is_none")]
    pub digest_blake3: Option<String>,
    #[serde(default = "no_evidence_digest", skip_serializing_if = "Option::is_none")]
    pub expected_digest_blake3: Option<String>,
    #[serde(default = "empty_identity_class", skip_serializing_if = "String::is_empty")]
    pub identity_class: String,
}

fn no_evidence_digest() -> Option<String> {
    None
}

fn empty_identity_class() -> String {
    String::new()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceToolchain {
    pub cargo: String,
    pub rustc: String,
    pub linker: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceNetworkPolicy {
    pub mode: String,
    pub result: String,
    pub allow_undeclared_network: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OfflineCargoEvidenceOutput {
    pub binary: String,
    pub path: String,
}

pub fn offline_cargo_non_claims() -> Vec<String> {
    OFFLINE_CARGO_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect()
}

pub fn plan_offline_cargo_package(request: RustOfflineCargoPackageRequest) -> RustOfflineCargoPlan {
    debug_assert!(!OFFLINE_CARGO_EVIDENCE_CLASS.is_empty());
    debug_assert!(!OFFLINE_CARGO_PROJECT_BUILD_STATUS.is_empty());
    let mut blockers = Vec::new();
    validate_name(
        ValidationField {
            label: "package name",
            value: &request.package_name,
            blocker_class: "invalid-package-name",
        },
        &mut blockers,
    );
    validate_name(
        ValidationField {
            label: "binary name",
            value: &request.binary_name,
            blocker_class: "missing-selected-binary",
        },
        &mut blockers,
    );
    validate_target_triple(&request.target_triple, &mut blockers);
    validate_profile(&request.profile, &mut blockers);
    validate_lockfile_digest(
        request.lockfile_digest_blake3.as_deref(),
        request.expected_lockfile_digest_blake3.as_deref(),
        &mut blockers,
    );
    validate_network_policy(request.allow_network, &mut blockers);
    validate_source_closure(&request.source_closure, &mut blockers);

    let plan = RustOfflineCargoPlan {
        ready: blockers.is_empty(),
        package_name: request.package_name,
        binary_name: request.binary_name,
        target_triple: request.target_triple,
        profile: request.profile,
        claim_class: OFFLINE_CARGO_EVIDENCE_CLASS.to_string(),
        project_build_status: OFFLINE_CARGO_PROJECT_BUILD_STATUS.to_string(),
        source_closure: request.source_closure,
        non_claims: offline_cargo_non_claims(),
        blockers,
    };
    debug_assert_eq!(plan.ready, plan.blockers.is_empty());
    debug_assert_eq!(plan.non_claims.len(), OFFLINE_CARGO_NON_CLAIMS.len());
    plan
}

pub fn validate_digest_bound_evidence(evidence: &OfflineCargoEvidenceV2File) -> Vec<RustOfflineCargoBlocker> {
    let mut blockers = Vec::new();
    validate_v2_header(evidence, &mut blockers);
    validate_name(
        ValidationField {
            label: "binary name",
            value: &evidence.binary,
            blocker_class: "missing-selected-binary",
        },
        &mut blockers,
    );
    validate_target_triple(&evidence.target, &mut blockers);
    validate_profile(&evidence.profile, &mut blockers);
    validate_evidence_lockfile(&evidence.lockfile, &mut blockers);
    validate_evidence_source_inputs(&evidence.source_closure, &mut blockers);
    validate_toolchain_paths(&evidence.toolchain, &mut blockers);
    validate_cargo_command(evidence, &mut blockers);
    validate_evidence_network_policy(&evidence.network_policy, &mut blockers);
    validate_evidence_output(evidence, &mut blockers);
    validate_non_claims(&evidence.non_claims, &mut blockers);
    blockers
}

pub fn validate_legacy_evidence(evidence: &OfflineCargoEvidenceV1File) -> Vec<RustOfflineCargoBlocker> {
    let mut blockers = Vec::new();
    if evidence.schema != OFFLINE_CARGO_EVIDENCE_SCHEMA_V1 {
        blockers.push(blocker("unsupported-offline-cargo-evidence-schema", "legacy evidence schema is unsupported"));
    }
    if evidence.claim_class != OFFLINE_CARGO_EVIDENCE_CLASS {
        blockers.push(blocker("unexpected-offline-cargo-claim-class", "legacy evidence claim class is unsupported"));
    }
    if evidence.project_build_status != OFFLINE_CARGO_PROJECT_BUILD_STATUS {
        blockers.push(blocker("unexpected-offline-cargo-build-status", "legacy evidence build status is unsupported"));
    }
    validate_non_claims(&evidence.non_claims, &mut blockers);
    blockers
}

fn validate_v2_header(evidence: &OfflineCargoEvidenceV2File, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if evidence.schema != OFFLINE_CARGO_EVIDENCE_SCHEMA_V2 {
        blockers
            .push(blocker("unsupported-offline-cargo-evidence-schema", "offline Cargo evidence schema is unsupported"));
    }
    if evidence.evidence_version != OFFLINE_CARGO_EVIDENCE_VERSION_V2 {
        blockers.push(blocker(
            "unsupported-offline-cargo-evidence-version",
            "offline Cargo evidence version is unsupported",
        ));
    }
    if evidence.claim_class != OFFLINE_CARGO_EVIDENCE_CLASS {
        blockers
            .push(blocker("unexpected-offline-cargo-claim-class", "offline Cargo evidence claim class is unsupported"));
    }
    if evidence.project_build_status != OFFLINE_CARGO_PROJECT_BUILD_STATUS {
        blockers.push(blocker("unexpected-offline-cargo-build-status", "offline Cargo build status is unsupported"));
    }
}

fn validate_evidence_lockfile(input: &OfflineCargoEvidenceInput, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if input.role != OFFLINE_CARGO_LOCKFILE_ROLE {
        blockers.push(blocker("unexpected-lockfile-role", "Cargo.lock evidence role is unsupported"));
    }
    validate_absolute_path(
        ValidationField {
            label: "Cargo.lock path",
            value: &input.path,
            blocker_class: "invalid-lockfile-path",
        },
        blockers,
    );
    validate_identity_class(input, OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT, blockers);
    validate_lockfile_digest(input.digest_blake3.as_deref(), input.expected_digest_blake3.as_deref(), blockers);
}

fn validate_evidence_source_inputs(inputs: &[OfflineCargoEvidenceInput], blockers: &mut Vec<RustOfflineCargoBlocker>) {
    debug_assert!(MAX_SOURCE_CLOSURE_ENTRIES >= REQUIRED_SOURCE_ROLES.len());
    debug_assert!(MAX_SOURCE_CLOSURE_ENTRIES >= REQUIRED_DIGEST_SOURCE_ROLES.len());
    if inputs.len() > MAX_SOURCE_CLOSURE_ENTRIES {
        blockers.push(blocker(
            "source-closure-too-large",
            "source closure exceeds the bounded offline Cargo evidence surface",
        ));
        return;
    }
    let mut roles_seen = BTreeSet::new();
    let mut paths_seen = BTreeSet::new();
    for input in inputs {
        validate_name(
            ValidationField {
                label: "source role",
                value: &input.role,
                blocker_class: "invalid-source-role",
            },
            blockers,
        );
        validate_absolute_path(
            ValidationField {
                label: "source path",
                value: &input.path,
                blocker_class: "invalid-source-path",
            },
            blockers,
        );
        validate_input_identity(input, blockers);
        if !roles_seen.insert(input.role.as_str()) {
            blockers.push(blocker(
                "duplicate-source-role",
                &format!("source role `{}` appears more than once", input.role),
            ));
        }
        if !paths_seen.insert(input.path.as_str()) {
            blockers.push(blocker(
                "duplicate-source-path",
                &format!("source path `{}` appears more than once", input.path),
            ));
        }
    }
    validate_required_roles(&roles_seen, blockers);
}

fn validate_required_roles(roles_seen: &BTreeSet<&str>, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    for required_role in REQUIRED_SOURCE_ROLES {
        if roles_seen.contains(required_role) {
            continue;
        }
        blockers.push(blocker(
            "missing-source-material",
            &format!("required source closure role `{required_role}` is missing"),
        ));
    }
}

fn validate_input_identity(input: &OfflineCargoEvidenceInput, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if REQUIRED_DIGEST_SOURCE_ROLES.contains(&input.role.as_str()) {
        validate_identity_class(input, OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT, blockers);
    }
    match input.identity_class.as_str() {
        OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT => validate_source_identity_digest(input, blockers),
        OFFLINE_CARGO_IDENTITY_STORE_PATH => {}
        "" => blockers.push(blocker("missing-source-identity-class", "source identity class is required")),
        _ => blockers.push(blocker("unsupported-source-identity-class", "source identity class is unsupported")),
    }
}

fn validate_source_identity_digest(input: &OfflineCargoEvidenceInput, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    let source = RustOfflineCargoSource {
        role: input.role.clone(),
        name: input.role.clone(),
        digest_blake3: input.digest_blake3.clone(),
        expected_digest_blake3: input.expected_digest_blake3.clone(),
    };
    validate_source_digest(&source, blockers);
    if input.digest_blake3.is_none() {
        blockers.push(blocker("missing-source-digest", &format!("source `{}` digest is required", input.role)));
    }
}

fn validate_identity_class(
    input: &OfflineCargoEvidenceInput,
    expected: &str,
    blockers: &mut Vec<RustOfflineCargoBlocker>,
) {
    if input.identity_class == expected {
        return;
    }
    blockers.push(blocker(
        "unexpected-source-identity-class",
        &format!("source `{}` must use `{expected}` identity", input.role),
    ));
}

fn validate_toolchain_paths(toolchain: &OfflineCargoEvidenceToolchain, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    for (label, value) in [
        ("cargo path", toolchain.cargo.as_str()),
        ("rustc path", toolchain.rustc.as_str()),
        ("linker path", toolchain.linker.as_str()),
    ] {
        validate_absolute_path(
            ValidationField {
                label,
                value,
                blocker_class: "invalid-toolchain-path",
            },
            blockers,
        );
    }
}

fn validate_cargo_command(evidence: &OfflineCargoEvidenceV2File, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    let command = &evidence.cargo_command;
    if command.program != OFFLINE_CARGO_COMMAND_PROGRAM {
        blockers.push(blocker("unexpected-cargo-command", "offline Cargo evidence must run cargo"));
    }
    if command.args.len() > MAX_COMMAND_ARGS {
        blockers.push(blocker("cargo-command-too-large", "offline Cargo command args exceed the bounded surface"));
        return;
    }
    require_command_arg(command, "build", blockers);
    require_command_arg(command, "--locked", blockers);
    require_command_arg(command, "--offline", blockers);
    require_command_pair(
        command,
        CommandPair {
            flag: "--bin",
            value: &evidence.binary,
        },
        blockers,
    );
    require_command_pair(
        command,
        CommandPair {
            flag: "--target",
            value: &evidence.target,
        },
        blockers,
    );
    validate_profile_command_arg(evidence, blockers);
}

fn validate_profile_command_arg(evidence: &OfflineCargoEvidenceV2File, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    let has_release = evidence.cargo_command.args.iter().any(|arg| arg == "--release");
    if evidence.profile == "release" && !has_release {
        blockers.push(blocker("missing-cargo-profile-arg", "release evidence must record --release"));
    }
    if evidence.profile == "debug" && has_release {
        blockers.push(blocker("unexpected-cargo-profile-arg", "debug evidence must not record --release"));
    }
}

fn require_command_arg(command: &OfflineCargoEvidenceCommand, arg: &str, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if command.args.iter().any(|candidate| candidate == arg) {
        return;
    }
    blockers.push(blocker("missing-cargo-command-arg", &format!("offline Cargo command is missing `{arg}`")));
}

struct CommandPair<'a> {
    flag: &'a str,
    value: &'a str,
}

fn require_command_pair(
    command: &OfflineCargoEvidenceCommand,
    required: CommandPair<'_>,
    blockers: &mut Vec<RustOfflineCargoBlocker>,
) {
    for pair in command.args.windows(2) {
        if pair[0] == required.flag && pair[1] == required.value {
            return;
        }
    }
    blockers.push(blocker(
        "missing-cargo-command-arg",
        format!("offline Cargo command is missing `{} {}`", required.flag, required.value),
    ));
}

fn validate_evidence_network_policy(
    policy: &OfflineCargoEvidenceNetworkPolicy,
    blockers: &mut Vec<RustOfflineCargoBlocker>,
) {
    if policy.mode != OFFLINE_CARGO_NETWORK_MODE {
        blockers.push(blocker("unsupported-network-policy", "offline Cargo evidence must record offline network mode"));
    }
    if policy.result != OFFLINE_CARGO_NETWORK_RESULT {
        blockers.push(blocker(
            "unexpected-network-policy-result",
            "offline Cargo evidence must record denied undeclared network",
        ));
    }
    if policy.allow_undeclared_network {
        blockers
            .push(blocker("unsupported-network-policy", "offline Cargo evidence must not allow undeclared network"));
    }
}

fn validate_evidence_output(evidence: &OfflineCargoEvidenceV2File, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if evidence.output.binary != evidence.binary {
        blockers.push(blocker("offline-cargo-output-mismatch", "output binary does not match selected binary"));
    }
    validate_absolute_path(
        ValidationField {
            label: "output path",
            value: &evidence.output.path,
            blocker_class: "invalid-output-path",
        },
        blockers,
    );
}

fn validate_non_claims(non_claims: &[String], blockers: &mut Vec<RustOfflineCargoBlocker>) {
    let present = non_claims.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for required in OFFLINE_CARGO_NON_CLAIMS {
        if present.contains(required) {
            continue;
        }
        blockers.push(blocker(
            "missing-offline-cargo-non-claim",
            &format!("offline Cargo evidence is missing `{required}`"),
        ));
    }
}

struct ValidationField<'a> {
    label: &'a str,
    value: &'a str,
    blocker_class: &'a str,
}

fn validate_name(field: ValidationField<'_>, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if is_valid_derivation_name(field.value) {
        return;
    }
    blockers.push(blocker(
        field.blocker_class,
        format!("{} must be a non-empty derivation-compatible name", field.label),
    ));
}

fn validate_absolute_path(field: ValidationField<'_>, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if field.value.starts_with('/') && field.value.len() <= MAX_PATH_BYTES {
        return;
    }
    blockers.push(blocker(field.blocker_class, format!("{} must be an absolute bounded path", field.label)));
}

fn validate_target_triple(target: &str, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if target.split('-').filter(|segment| !segment.is_empty()).count() >= MIN_TARGET_SEGMENTS {
        return;
    }
    blockers.push(blocker(
        "unsupported-target-triple",
        "target triple must be explicit enough for Cargo output selection",
    ));
}

fn validate_profile(profile: &str, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if matches!(profile, "debug" | "release") {
        return;
    }
    blockers.push(blocker(
        "unsupported-profile",
        "offline Cargo project builds initially support debug or release profiles",
    ));
}

fn validate_lockfile_digest(
    digest: Option<&str>,
    expected_digest: Option<&str>,
    blockers: &mut Vec<RustOfflineCargoBlocker>,
) {
    debug_assert_eq!(BLAKE3_HEX_BYTES, blake3::OUT_LEN.saturating_mul(2));
    debug_assert!(MAX_NAME_BYTES > 0);
    let Some(digest) = digest else {
        blockers.push(blocker(
            "missing-lockfile-digest",
            "Cargo.lock identity must be recorded before accepting an offline package build",
        ));
        return;
    };
    if !is_blake3_hex(digest) {
        blockers.push(blocker("invalid-lockfile-digest", "Cargo.lock digest must be lowercase BLAKE3 hex"));
        return;
    }
    let Some(expected) = expected_digest else {
        return;
    };
    if !is_blake3_hex(expected) {
        blockers.push(blocker(
            "invalid-expected-lockfile-digest",
            "expected Cargo.lock digest must be lowercase BLAKE3 hex",
        ));
        return;
    }
    if digest == expected {
        return;
    }
    blockers.push(blocker("stale-lockfile-digest", "Cargo.lock digest differs from the recorded project expectation"));
}

fn validate_network_policy(allow_network: bool, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if !allow_network {
        return;
    }
    blockers.push(blocker(
        "unsupported-network-policy",
        "offline Cargo project builds must not enable undeclared network access",
    ));
}

fn validate_source_closure(sources: &[RustOfflineCargoSource], blockers: &mut Vec<RustOfflineCargoBlocker>) {
    debug_assert!(MAX_SOURCE_CLOSURE_ENTRIES >= REQUIRED_SOURCE_ROLES.len());
    debug_assert!(MAX_SOURCE_CLOSURE_ENTRIES >= REQUIRED_DIGEST_SOURCE_ROLES.len());
    if sources.len() > MAX_SOURCE_CLOSURE_ENTRIES {
        blockers.push(blocker(
            "source-closure-too-large",
            "source closure exceeds the bounded initial offline Cargo package surface",
        ));
        return;
    }

    let mut roles_seen = BTreeSet::new();
    let mut names_seen = BTreeSet::new();
    for source in sources {
        validate_name(
            ValidationField {
                label: "source role",
                value: &source.role,
                blocker_class: "invalid-source-role",
            },
            blockers,
        );
        validate_name(
            ValidationField {
                label: "source name",
                value: &source.name,
                blocker_class: "invalid-source-name",
            },
            blockers,
        );
        if !roles_seen.insert(source.role.as_str()) {
            blockers.push(blocker(
                "duplicate-source-role",
                &format!("source role `{}` appears more than once", source.role),
            ));
        }
        if !names_seen.insert(source.name.as_str()) {
            blockers
                .push(blocker("duplicate-source-name", &format!("source `{}` appears more than once", source.name)));
        }
        validate_source_digest(source, blockers);
    }

    for required_role in REQUIRED_SOURCE_ROLES {
        if roles_seen.contains(required_role) {
            continue;
        }
        blockers.push(blocker(
            "missing-source-material",
            &format!("required source closure role `{required_role}` is missing"),
        ));
    }
}

fn validate_source_digest(source: &RustOfflineCargoSource, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    debug_assert_eq!(BLAKE3_HEX_BYTES, blake3::OUT_LEN.saturating_mul(2));
    debug_assert!(MAX_NAME_BYTES > 0);
    let Some(digest) = source.digest_blake3.as_deref() else {
        return;
    };
    if !is_blake3_hex(digest) {
        blockers.push(blocker(
            "invalid-source-digest",
            &format!("source `{}` digest must be lowercase BLAKE3 hex", source.name),
        ));
        return;
    }
    let Some(expected) = source.expected_digest_blake3.as_deref() else {
        return;
    };
    if !is_blake3_hex(expected) {
        blockers.push(blocker(
            "invalid-expected-source-digest",
            &format!("source `{}` expected digest must be lowercase BLAKE3 hex", source.name),
        ));
        return;
    }
    if digest == expected {
        return;
    }
    blockers.push(blocker(
        "stale-source-digest",
        &format!("source `{}` digest differs from the recorded project expectation", source.name),
    ));
}

fn is_valid_derivation_name(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_NAME_BYTES {
        return false;
    }
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'_' | b'?' | b'=' | b'-'))
}

pub fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_BYTES && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn blocker(class: &str, message: impl AsRef<str>) -> RustOfflineCargoBlocker {
    let message = message.as_ref();
    RustOfflineCargoBlocker {
        class: class.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const DIGEST_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

    fn source(role: &str, name: &str, digest: &str) -> RustOfflineCargoSource {
        RustOfflineCargoSource {
            role: role.to_string(),
            name: name.to_string(),
            digest_blake3: Some(digest.to_string()),
            expected_digest_blake3: Some(digest.to_string()),
        }
    }

    fn evidence_input(role: &str, path: &str, digest: Option<&str>, identity_class: &str) -> OfflineCargoEvidenceInput {
        OfflineCargoEvidenceInput {
            role: role.to_string(),
            path: path.to_string(),
            digest_blake3: digest.map(str::to_string),
            expected_digest_blake3: digest.map(str::to_string),
            identity_class: identity_class.to_string(),
        }
    }

    fn valid_request() -> RustOfflineCargoPackageRequest {
        RustOfflineCargoPackageRequest {
            package_name: "demo".to_string(),
            binary_name: "demo".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            profile: "release".to_string(),
            lockfile_digest_blake3: Some(DIGEST_A.to_string()),
            expected_lockfile_digest_blake3: Some(DIGEST_A.to_string()),
            allow_network: false,
            source_closure: vec![
                source("package-source", "demo-src", DIGEST_B),
                source("rust-toolchain", "rust", DIGEST_C),
                source("seed-toolchain", "musl-seed-toolchain", DIGEST_D),
                source("musl-runtime", "musl", DIGEST_E),
            ],
        }
    }

    fn valid_evidence_v2() -> OfflineCargoEvidenceV2File {
        OfflineCargoEvidenceV2File {
            schema: OFFLINE_CARGO_EVIDENCE_SCHEMA_V2.to_string(),
            evidence_version: OFFLINE_CARGO_EVIDENCE_VERSION_V2,
            claim_class: OFFLINE_CARGO_EVIDENCE_CLASS.to_string(),
            project_build_status: OFFLINE_CARGO_PROJECT_BUILD_STATUS.to_string(),
            target: "x86_64-unknown-linux-musl".to_string(),
            profile: "release".to_string(),
            binary: "demo".to_string(),
            lockfile: evidence_input(
                OFFLINE_CARGO_LOCKFILE_ROLE,
                "/src/Cargo.lock",
                Some(DIGEST_A),
                OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
            ),
            source_closure: vec![
                evidence_input("package-source", "/src", Some(DIGEST_B), OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT),
                evidence_input("rust-toolchain", "/toolchain/rust", None, OFFLINE_CARGO_IDENTITY_STORE_PATH),
                evidence_input("seed-toolchain", "/toolchain/seed", None, OFFLINE_CARGO_IDENTITY_STORE_PATH),
                evidence_input("musl-runtime", "/toolchain/musl", None, OFFLINE_CARGO_IDENTITY_STORE_PATH),
                evidence_input(
                    "vendored-dependencies",
                    "/vendor",
                    Some(DIGEST_C),
                    OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
                ),
            ],
            toolchain: OfflineCargoEvidenceToolchain {
                cargo: "/toolchain/rust/bin/cargo".to_string(),
                rustc: "/toolchain/rust/bin/rustc".to_string(),
                linker: "/toolchain/seed/bin/x86_64-linux-musl-gcc".to_string(),
            },
            cargo_command: OfflineCargoEvidenceCommand {
                program: OFFLINE_CARGO_COMMAND_PROGRAM.to_string(),
                args: vec![
                    "build".to_string(),
                    "--locked".to_string(),
                    "--offline".to_string(),
                    "--release".to_string(),
                    "--bin".to_string(),
                    "demo".to_string(),
                    "--target".to_string(),
                    "x86_64-unknown-linux-musl".to_string(),
                ],
            },
            network_policy: OfflineCargoEvidenceNetworkPolicy {
                mode: OFFLINE_CARGO_NETWORK_MODE.to_string(),
                result: OFFLINE_CARGO_NETWORK_RESULT.to_string(),
                allow_undeclared_network: false,
            },
            output: OfflineCargoEvidenceOutput {
                binary: "demo".to_string(),
                path: "/out/bin/demo".to_string(),
            },
            non_claims: offline_cargo_non_claims(),
        }
    }

    #[test]
    fn valid_offline_cargo_plan_records_bounded_claims() {
        let plan = plan_offline_cargo_package(valid_request());

        assert!(plan.ready, "valid package should be ready: {:#?}", plan.blockers);
        assert_eq!(plan.claim_class, OFFLINE_CARGO_EVIDENCE_CLASS);
        assert_eq!(plan.project_build_status, OFFLINE_CARGO_PROJECT_BUILD_STATUS);
        assert!(plan.blockers.is_empty());
        assert!(plan.non_claims.contains(&"not-cargo-free-execution".to_string()));
        assert!(plan.non_claims.contains(&"not-full-cargo-compatibility".to_string()));
    }

    #[test]
    fn invalid_offline_cargo_plan_fails_closed_without_network_fallback() {
        let mut request = valid_request();
        request.binary_name.clear();
        request.lockfile_digest_blake3 = None;
        request.allow_network = true;
        request.source_closure.retain(|source| source.role != "package-source");

        let plan = plan_offline_cargo_package(request);
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(!plan.ready);
        assert!(classes.contains("missing-selected-binary"), "classes: {classes:?}");
        assert!(classes.contains("missing-lockfile-digest"), "classes: {classes:?}");
        assert!(classes.contains("unsupported-network-policy"), "classes: {classes:?}");
        assert!(classes.contains("missing-source-material"), "classes: {classes:?}");
    }

    #[test]
    fn stale_offline_cargo_digests_fail_closed_before_accepting_outputs() {
        let mut request = valid_request();
        request.expected_lockfile_digest_blake3 = Some(DIGEST_B.to_string());
        request.source_closure[0].expected_digest_blake3 = Some(DIGEST_C.to_string());

        let plan = plan_offline_cargo_package(request);
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(!plan.ready);
        assert!(classes.contains("stale-lockfile-digest"), "classes: {classes:?}");
        assert!(classes.contains("stale-source-digest"), "classes: {classes:?}");
    }

    #[test]
    fn digest_bound_evidence_accepts_content_and_store_path_identities() {
        let blockers = validate_digest_bound_evidence(&valid_evidence_v2());

        assert!(blockers.is_empty(), "blockers: {blockers:#?}");
    }

    #[test]
    fn digest_bound_evidence_rejects_stale_or_impure_claims() {
        let mut evidence = valid_evidence_v2();
        evidence.lockfile.expected_digest_blake3 = Some(DIGEST_B.to_string());
        evidence.source_closure[0].digest_blake3 = None;
        evidence.network_policy.allow_undeclared_network = true;
        evidence.non_claims.retain(|claim| claim != "not-compiler-correctness");

        let blockers = validate_digest_bound_evidence(&evidence);
        let classes = blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("stale-lockfile-digest"), "classes: {classes:?}");
        assert!(classes.contains("missing-source-digest"), "classes: {classes:?}");
        assert!(classes.contains("unsupported-network-policy"), "classes: {classes:?}");
        assert!(classes.contains("missing-offline-cargo-non-claim"), "classes: {classes:?}");
    }
}
