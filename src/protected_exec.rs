use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

const BLAKE3_HEX_LEN: u32 = 64;
const SCHEMA_VERSION_V1: &str = "host-tool-free-stage0-v1";
const DIGEST_ALGORITHM_BLAKE3: &str = "blake3";
const ROLE_STAGE0_CRUNCH: &str = "stage0-crunch";
const ROLE_SANDBOX_ENTRY: &str = "sandbox-entry";
const ROLE_SANDBOX_SHELL: &str = "sandbox-shell";
const ROLE_BOOTSTRAP_TOOLCHAIN_TOOL: &str = "bootstrap-toolchain-tool";
const ROLE_BOOTSTRAP_BUILD_TOOL: &str = "bootstrap-build-tool";
const PROVENANCE_OPERATOR_SOURCE_BUILD: &str = "operator-supplied-source-build";
const PROVENANCE_OPERATOR_BOOTSTRAP_SEED: &str = "operator-supplied-bootstrap-seed";
const PROVENANCE_TEST_FIXTURE: &str = "test-fixture";
const PROVENANCE_HOST_PATH_DISCOVERY: &str = "host-path-discovery";
const PROVENANCE_NIX_STORE_DISCOVERY: &str = "nix-store-discovery";
const PHASE_PROTECTED: &str = "protected";
const PATH_SEPARATOR: char = '/';

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DigestSpec {
    pub algorithm: String,
    pub hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interoperability_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutableSeedEntry {
    pub schema_version: String,
    pub id: String,
    pub role: String,
    pub phase: String,
    pub executable_path: PathBuf,
    pub digest: DigestSpec,
    pub provenance_category: String,
    pub provenance: String,
    pub allowed_reason: String,
    pub owner: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceSeedEntry {
    pub schema_version: String,
    pub id: String,
    pub role: String,
    pub phase: String,
    pub urls: Vec<String>,
    pub extraction_rules: Vec<String>,
    pub digest: DigestSpec,
    pub provenance_category: String,
    pub provenance: String,
    pub allowed_reason: String,
    pub owner: String,
    pub required: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Stage0Inventory {
    pub executable_entries: Vec<ExecutableSeedEntry>,
    pub source_entries: Vec<SourceSeedEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecRequest {
    pub path: PathBuf,
    pub digest_hex: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecDecision {
    pub allowed: bool,
    pub entry_id: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedExecPolicy {
    executables_by_path: BTreeMap<PathBuf, ExecutableSeedEntry>,
    source_urls: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedExecError {
    DuplicateExecutablePath {
        path: PathBuf,
    },
    DuplicateSourceUrl {
        url: String,
    },
    EmptyField {
        entry_id: String,
        field: &'static str,
    },
    UnsupportedSchemaVersion {
        entry_id: String,
        actual: String,
    },
    RelativeExecutablePath {
        entry_id: String,
        path: PathBuf,
    },
    DisallowedRole {
        entry_id: String,
        role: String,
    },
    DisallowedNixExecutable {
        entry_id: String,
        path: PathBuf,
    },
    DisallowedProvenance {
        entry_id: String,
        provenance_category: String,
    },
    MissingInteroperabilityReason {
        entry_id: String,
        algorithm: String,
    },
    InvalidBlake3Digest {
        entry_id: String,
        digest_hex: String,
    },
    MissingRequiredSeedRole {
        role: &'static str,
    },
    UndeclaredExecutable {
        path: PathBuf,
    },
    DigestMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    UndeclaredSourceUrl {
        url: String,
    },
}

impl fmt::Display for ProtectedExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateExecutablePath { path } => write!(f, "duplicate executable path: {}", path.display()),
            Self::DuplicateSourceUrl { url } => write!(f, "duplicate source url: {url}"),
            Self::EmptyField { entry_id, field } => write!(f, "entry {entry_id} has empty field {field}"),
            Self::UnsupportedSchemaVersion { entry_id, actual } => {
                write!(f, "entry {entry_id} has unsupported schema version {actual}")
            }
            Self::RelativeExecutablePath { entry_id, path } => {
                write!(f, "entry {entry_id} executable path is not absolute: {}", path.display())
            }
            Self::DisallowedRole { entry_id, role } => write!(f, "entry {entry_id} uses disallowed role {role}"),
            Self::DisallowedNixExecutable { entry_id, path } => {
                write!(f, "entry {entry_id} declares forbidden nix executable {}", path.display())
            }
            Self::DisallowedProvenance {
                entry_id,
                provenance_category,
            } => write!(f, "entry {entry_id} has disallowed provenance {provenance_category}"),
            Self::MissingInteroperabilityReason { entry_id, algorithm } => {
                write!(f, "entry {entry_id} uses non-blake3 digest {algorithm} without interoperability reason")
            }
            Self::InvalidBlake3Digest { entry_id, digest_hex } => {
                write!(f, "entry {entry_id} has invalid blake3 digest {digest_hex}")
            }
            Self::MissingRequiredSeedRole { role } => write!(f, "missing required seed role {role}"),
            Self::UndeclaredExecutable { path } => write!(f, "undeclared executable path: {}", path.display()),
            Self::DigestMismatch { path, expected, actual } => {
                write!(f, "digest mismatch for {}: expected {expected}, got {actual}", path.display())
            }
            Self::UndeclaredSourceUrl { url } => write!(f, "undeclared protected source url: {url}"),
        }
    }
}

impl std::error::Error for ProtectedExecError {}

impl ProtectedExecPolicy {
    pub fn from_inventory(inventory: Stage0Inventory) -> Result<Self, ProtectedExecError> {
        validate_required_roles(&inventory.executable_entries)?;
        let mut executables_by_path = BTreeMap::new();
        for entry in inventory.executable_entries {
            validate_executable_entry(&entry)?;
            let previous = executables_by_path.insert(entry.executable_path.clone(), entry);
            if previous.is_some() {
                let path = executables_by_path.keys().next_back().cloned().unwrap_or_default();
                return Err(ProtectedExecError::DuplicateExecutablePath { path });
            }
        }

        let mut source_urls = BTreeSet::new();
        for entry in inventory.source_entries {
            validate_source_entry(&entry)?;
            for url in entry.urls {
                if !source_urls.insert(url.clone()) {
                    return Err(ProtectedExecError::DuplicateSourceUrl { url });
                }
            }
        }

        assert!(!executables_by_path.is_empty(), "policy must contain required executables");
        Ok(Self {
            executables_by_path,
            source_urls,
        })
    }

    pub fn decide_exec(&self, request: &ExecRequest) -> Result<ExecDecision, ProtectedExecError> {
        assert!(request.path.is_absolute(), "exec request path must be absolute");
        assert!(!request.digest_hex.is_empty(), "exec request digest must be present");
        let entry =
            self.executables_by_path
                .get(&request.path)
                .ok_or_else(|| ProtectedExecError::UndeclaredExecutable {
                    path: request.path.clone(),
                })?;
        if entry.digest.hex != request.digest_hex {
            return Err(ProtectedExecError::DigestMismatch {
                path: request.path.clone(),
                expected: entry.digest.hex.clone(),
                actual: request.digest_hex.clone(),
            });
        }
        Ok(ExecDecision {
            allowed: true,
            entry_id: Some(entry.id.clone()),
            reason: entry.allowed_reason.clone(),
        })
    }

    pub fn ensure_source_url_allowed(&self, url: &str) -> Result<(), ProtectedExecError> {
        assert!(!url.is_empty(), "source url check must not be empty");
        if self.source_urls.contains(url) {
            return Ok(());
        }
        Err(ProtectedExecError::UndeclaredSourceUrl { url: url.to_string() })
    }
}

fn validate_required_roles(entries: &[ExecutableSeedEntry]) -> Result<(), ProtectedExecError> {
    assert!(!entries.is_empty(), "inventory must include executable entries");
    let roles: BTreeSet<&str> =
        entries.iter().filter(|entry| entry.required).map(|entry| entry.role.as_str()).collect();
    for role in [ROLE_SANDBOX_ENTRY, ROLE_SANDBOX_SHELL] {
        if !roles.contains(role) {
            return Err(ProtectedExecError::MissingRequiredSeedRole { role });
        }
    }
    Ok(())
}

fn validate_executable_entry(entry: &ExecutableSeedEntry) -> Result<(), ProtectedExecError> {
    validate_common_fields(
        &entry.schema_version,
        &entry.id,
        &entry.role,
        &entry.phase,
        &entry.digest,
        &entry.provenance_category,
        &entry.provenance,
        &entry.allowed_reason,
        &entry.owner,
    )?;
    if !entry.executable_path.is_absolute() {
        return Err(ProtectedExecError::RelativeExecutablePath {
            entry_id: entry.id.clone(),
            path: entry.executable_path.clone(),
        });
    }
    if !is_allowed_executable_role(&entry.role) {
        return Err(ProtectedExecError::DisallowedRole {
            entry_id: entry.id.clone(),
            role: entry.role.clone(),
        });
    }
    if is_forbidden_nix_executable(&entry.executable_path) {
        return Err(ProtectedExecError::DisallowedNixExecutable {
            entry_id: entry.id.clone(),
            path: entry.executable_path.clone(),
        });
    }
    Ok(())
}

fn validate_source_entry(entry: &SourceSeedEntry) -> Result<(), ProtectedExecError> {
    validate_common_fields(
        &entry.schema_version,
        &entry.id,
        &entry.role,
        &entry.phase,
        &entry.digest,
        &entry.provenance_category,
        &entry.provenance,
        &entry.allowed_reason,
        &entry.owner,
    )?;
    validate_non_empty(entry.id.as_str(), "urls", entry.urls.first().map(String::as_str).unwrap_or_default())?;
    validate_non_empty(
        entry.id.as_str(),
        "extraction_rules",
        entry.extraction_rules.first().map(String::as_str).unwrap_or_default(),
    )?;
    Ok(())
}

fn validate_common_fields(
    schema_version: &str,
    entry_id: &str,
    role: &str,
    phase: &str,
    digest: &DigestSpec,
    provenance_category: &str,
    provenance: &str,
    allowed_reason: &str,
    owner: &str,
) -> Result<(), ProtectedExecError> {
    validate_non_empty(entry_id, "id", entry_id)?;
    validate_non_empty(entry_id, "role", role)?;
    validate_non_empty(entry_id, "phase", phase)?;
    validate_non_empty(entry_id, "digest.algorithm", digest.algorithm.as_str())?;
    validate_non_empty(entry_id, "digest.hex", digest.hex.as_str())?;
    validate_non_empty(entry_id, "provenance_category", provenance_category)?;
    validate_non_empty(entry_id, "provenance", provenance)?;
    validate_non_empty(entry_id, "allowed_reason", allowed_reason)?;
    validate_non_empty(entry_id, "owner", owner)?;
    if schema_version != SCHEMA_VERSION_V1 {
        return Err(ProtectedExecError::UnsupportedSchemaVersion {
            entry_id: entry_id.to_string(),
            actual: schema_version.to_string(),
        });
    }
    if phase != PHASE_PROTECTED {
        return Err(ProtectedExecError::DisallowedRole {
            entry_id: entry_id.to_string(),
            role: phase.to_string(),
        });
    }
    validate_digest(entry_id, digest)?;
    validate_provenance(entry_id, provenance_category)?;
    Ok(())
}

fn validate_non_empty(entry_id: &str, field: &'static str, value: &str) -> Result<(), ProtectedExecError> {
    if value.is_empty() {
        return Err(ProtectedExecError::EmptyField {
            entry_id: entry_id.to_string(),
            field,
        });
    }
    Ok(())
}

fn validate_digest(entry_id: &str, digest: &DigestSpec) -> Result<(), ProtectedExecError> {
    if digest.algorithm == DIGEST_ALGORITHM_BLAKE3 {
        if is_valid_hex_digest(&digest.hex, BLAKE3_HEX_LEN) {
            return Ok(());
        }
        return Err(ProtectedExecError::InvalidBlake3Digest {
            entry_id: entry_id.to_string(),
            digest_hex: digest.hex.clone(),
        });
    }
    match &digest.interoperability_reason {
        Some(reason) if !reason.is_empty() => Ok(()),
        _ => Err(ProtectedExecError::MissingInteroperabilityReason {
            entry_id: entry_id.to_string(),
            algorithm: digest.algorithm.clone(),
        }),
    }
}

fn validate_provenance(entry_id: &str, provenance_category: &str) -> Result<(), ProtectedExecError> {
    if provenance_category == PROVENANCE_HOST_PATH_DISCOVERY || provenance_category == PROVENANCE_NIX_STORE_DISCOVERY {
        return Err(ProtectedExecError::DisallowedProvenance {
            entry_id: entry_id.to_string(),
            provenance_category: provenance_category.to_string(),
        });
    }
    if matches!(
        provenance_category,
        PROVENANCE_OPERATOR_SOURCE_BUILD | PROVENANCE_OPERATOR_BOOTSTRAP_SEED | PROVENANCE_TEST_FIXTURE
    ) {
        return Ok(());
    }
    Err(ProtectedExecError::DisallowedProvenance {
        entry_id: entry_id.to_string(),
        provenance_category: provenance_category.to_string(),
    })
}

fn is_allowed_executable_role(role: &str) -> bool {
    matches!(
        role,
        ROLE_STAGE0_CRUNCH
            | ROLE_SANDBOX_ENTRY
            | ROLE_SANDBOX_SHELL
            | ROLE_BOOTSTRAP_TOOLCHAIN_TOOL
            | ROLE_BOOTSTRAP_BUILD_TOOL
    )
}

fn is_forbidden_nix_executable(path: &Path) -> bool {
    match path.file_name().and_then(|name| name.to_str()) {
        Some("nix") | Some("nix-build") | Some("nix-store") | Some("nix-shell") | Some("nix-develop") => true,
        Some(name) if name.starts_with("nix") && name.contains("develop") => true,
        _ => false,
    }
}

fn is_valid_hex_digest(value: &str, expected_len: u32) -> bool {
    let Ok(actual_len) = u32::try_from(value.len()) else {
        return false;
    };
    if actual_len != expected_len {
        return false;
    }
    value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn normalized_path_id(path: &Path) -> String {
    let body = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .filter(|component| *component != "" && *component != "/")
        .collect::<Vec<_>>()
        .join(PATH_SEPARATOR.encode_utf8(&mut [0; 4]));
    if path.is_absolute() {
        return format!("/{body}");
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn digest(hex: &str) -> DigestSpec {
        DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            hex: hex.to_string(),
            interoperability_reason: None,
        }
    }

    fn executable(id: &str, role: &str, path: &str, hex: &str) -> ExecutableSeedEntry {
        ExecutableSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: id.to_string(),
            role: role.to_string(),
            phase: PHASE_PROTECTED.to_string(),
            executable_path: PathBuf::from(path),
            digest: digest(hex),
            provenance_category: PROVENANCE_TEST_FIXTURE.to_string(),
            provenance: "fixture-built seed".to_string(),
            allowed_reason: "needed for protected sandbox".to_string(),
            owner: "bootstrap".to_string(),
            required: true,
        }
    }

    fn source(id: &str, url: &str) -> SourceSeedEntry {
        SourceSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: id.to_string(),
            role: "bootstrap-source".to_string(),
            phase: PHASE_PROTECTED.to_string(),
            urls: vec![url.to_string()],
            extraction_rules: vec!["strip-components=1".to_string()],
            digest: digest(DIGEST_A),
            provenance_category: PROVENANCE_TEST_FIXTURE.to_string(),
            provenance: "fixture source".to_string(),
            allowed_reason: "bootstrap source".to_string(),
            owner: "bootstrap".to_string(),
            required: true,
        }
    }

    fn inventory() -> Stage0Inventory {
        Stage0Inventory {
            executable_entries: vec![
                executable("sandbox", ROLE_SANDBOX_ENTRY, "/seed/bin/bwrap", DIGEST_A),
                executable("shell", ROLE_SANDBOX_SHELL, "/seed/bin/busybox", DIGEST_B),
            ],
            source_entries: vec![source("musl", "https://example.invalid/musl.tar.xz")],
        }
    }

    #[test]
    fn policy_allows_declared_seed_with_matching_digest() {
        let policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let decision = policy
            .decide_exec(&ExecRequest {
                path: PathBuf::from("/seed/bin/bwrap"),
                digest_hex: DIGEST_A.to_string(),
            })
            .unwrap();

        assert!(decision.allowed);
        assert_eq!(decision.entry_id.as_deref(), Some("sandbox"));
        assert_eq!(decision.reason, "needed for protected sandbox");
    }

    #[test]
    fn policy_rejects_undeclared_and_mismatched_executables() {
        let policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let undeclared = policy
            .decide_exec(&ExecRequest {
                path: PathBuf::from("/usr/bin/tar"),
                digest_hex: DIGEST_A.to_string(),
            })
            .unwrap_err();
        assert_eq!(undeclared, ProtectedExecError::UndeclaredExecutable {
            path: PathBuf::from("/usr/bin/tar")
        });

        let mismatch = policy
            .decide_exec(&ExecRequest {
                path: PathBuf::from("/seed/bin/bwrap"),
                digest_hex: DIGEST_B.to_string(),
            })
            .unwrap_err();
        assert!(matches!(mismatch, ProtectedExecError::DigestMismatch { .. }));
    }

    #[test]
    fn inventory_rejects_forbidden_nix_even_with_digest() {
        let mut inv = inventory();
        inv.executable_entries.push(executable("nix", ROLE_BOOTSTRAP_BUILD_TOOL, "/seed/bin/nix", DIGEST_A));
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert!(matches!(err, ProtectedExecError::DisallowedNixExecutable { .. }));
    }

    #[test]
    fn inventory_rejects_bad_provenance_and_relative_paths() {
        let mut bad_provenance = inventory();
        bad_provenance.executable_entries[0].provenance_category = PROVENANCE_NIX_STORE_DISCOVERY.to_string();
        let err = ProtectedExecPolicy::from_inventory(bad_provenance).unwrap_err();
        assert!(matches!(err, ProtectedExecError::DisallowedProvenance { .. }));

        let mut relative = inventory();
        relative.executable_entries[0].executable_path = PathBuf::from("bin/bwrap");
        let err = ProtectedExecPolicy::from_inventory(relative).unwrap_err();
        assert!(matches!(err, ProtectedExecError::RelativeExecutablePath { .. }));
    }

    #[test]
    fn inventory_requires_interoperability_reason_for_non_blake3_digest() {
        let mut inv = inventory();
        inv.executable_entries[0].digest.algorithm = "sha256".to_string();
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert!(matches!(err, ProtectedExecError::MissingInteroperabilityReason { .. }));

        let mut allowed = inventory();
        allowed.executable_entries[0].digest.algorithm = "sha256".to_string();
        allowed.executable_entries[0].digest.interoperability_reason = Some("Cargo package checksum".to_string());
        let policy = ProtectedExecPolicy::from_inventory(allowed).unwrap();
        assert!(policy.executables_by_path.contains_key(Path::new("/seed/bin/bwrap")));
    }

    #[test]
    fn policy_allows_only_declared_source_urls() {
        let policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        assert!(policy.ensure_source_url_allowed("https://example.invalid/musl.tar.xz").is_ok());
        let err = policy.ensure_source_url_allowed("https://example.invalid/other.tar.xz").unwrap_err();
        assert_eq!(err, ProtectedExecError::UndeclaredSourceUrl {
            url: "https://example.invalid/other.tar.xz".to_string()
        });
    }

    #[test]
    fn inventory_requires_sandbox_entry_and_shell() {
        let inv = Stage0Inventory {
            executable_entries: vec![executable("sandbox", ROLE_SANDBOX_ENTRY, "/seed/bin/bwrap", DIGEST_A)],
            source_entries: Vec::new(),
        };
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert_eq!(err, ProtectedExecError::MissingRequiredSeedRole {
            role: ROLE_SANDBOX_SHELL
        });
    }

    #[test]
    fn normalized_path_id_is_deterministic() {
        let id = normalized_path_id(Path::new("/seed/bin/bwrap"));
        assert_eq!(id, "/seed/bin/bwrap");
        assert_ne!(id, "/seed/bin/busybox");
    }
}
