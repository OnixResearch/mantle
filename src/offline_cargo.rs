use std::collections::BTreeSet;

pub const OFFLINE_CARGO_EVIDENCE_SCHEMA: &str = "mantle-offline-cargo-build-evidence-v1";
pub const OFFLINE_CARGO_EVIDENCE_CLASS: &str = "cargo-inside-mantle-sandbox";
pub const OFFLINE_CARGO_PROJECT_BUILD_STATUS: &str = "default-project-build-lane";
pub const OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH: &str = "share/mantle/offline-cargo-build.json";

const MAX_SOURCE_CLOSURE_ENTRIES: usize = 32;
const MAX_NAME_BYTES: usize = 128;
const BLAKE3_HEX_BYTES: usize = 64;
const MIN_TARGET_SEGMENTS: usize = 3;
const REQUIRED_SOURCE_ROLES: [&str; 4] = ["package-source", "rust-toolchain", "seed-toolchain", "musl-runtime"];
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

pub fn plan_offline_cargo_package(request: RustOfflineCargoPackageRequest) -> RustOfflineCargoPlan {
    let mut blockers = Vec::new();
    validate_name("package name", &request.package_name, "invalid-package-name", &mut blockers);
    validate_name("binary name", &request.binary_name, "missing-selected-binary", &mut blockers);
    validate_target_triple(&request.target_triple, &mut blockers);
    validate_profile(&request.profile, &mut blockers);
    validate_lockfile_digest(
        request.lockfile_digest_blake3.as_deref(),
        request.expected_lockfile_digest_blake3.as_deref(),
        &mut blockers,
    );
    validate_network_policy(request.allow_network, &mut blockers);
    validate_source_closure(&request.source_closure, &mut blockers);

    RustOfflineCargoPlan {
        ready: blockers.is_empty(),
        package_name: request.package_name,
        binary_name: request.binary_name,
        target_triple: request.target_triple,
        profile: request.profile,
        claim_class: OFFLINE_CARGO_EVIDENCE_CLASS.to_string(),
        project_build_status: OFFLINE_CARGO_PROJECT_BUILD_STATUS.to_string(),
        source_closure: request.source_closure,
        non_claims: OFFLINE_CARGO_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect(),
        blockers,
    }
}

fn validate_name(label: &str, value: &str, class: &str, blockers: &mut Vec<RustOfflineCargoBlocker>) {
    if is_valid_derivation_name(value) {
        return;
    }
    blockers.push(blocker(class, &format!("{label} must be a non-empty derivation-compatible name")));
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
        validate_name("source role", &source.role, "invalid-source-role", blockers);
        validate_name("source name", &source.name, "invalid-source-name", blockers);
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

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_BYTES && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn blocker(class: &str, message: &str) -> RustOfflineCargoBlocker {
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
}
