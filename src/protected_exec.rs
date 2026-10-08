// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in the store-capability-migration change evidence and scheduled for the
// standalone hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::assertion_density, tigerstyle::sentinel_fallback)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::io::Read;
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
const ROLE_AUDITED_BOOTSTRAP_SEED: &str = "audited-bootstrap-seed";
const PROVENANCE_OPERATOR_SOURCE_BUILD: &str = "operator-supplied-source-build";
const PROVENANCE_OPERATOR_BOOTSTRAP_SEED: &str = "operator-supplied-bootstrap-seed";
const PROVENANCE_TEST_FIXTURE: &str = "test-fixture";
const PROVENANCE_HOST_PATH_DISCOVERY: &str = "host-path-discovery";
const PROVENANCE_NIX_STORE_DISCOVERY: &str = "nix-store-discovery";
/// Label for protected-phase audit events; this token alone grants no execution authority.
pub const PHASE_PROTECTED: &str = "protected";
const PATH_SEPARATOR: char = '/';
const HASH_BUFFER_KIB: usize = 64;
const KIB_BYTES_USIZE: usize = 1024;
const HASH_BUFFER_BYTES: usize = HASH_BUFFER_KIB.saturating_mul(KIB_BYTES_USIZE);
const MAX_SEED_EXECUTABLES: u32 = 4096;
const MAX_EXECUTABLE_DIGEST_VARIANTS_PER_PATH: u32 = 64;
const MAX_SEED_WALK_DEPTH: u32 = 16;
const MAX_SEED_WALK_ENTRIES: u32 = MAX_SEED_EXECUTABLES.saturating_mul(MAX_SEED_WALK_DEPTH);
const MAX_SOURCE_ENTRIES: u32 = MAX_SEED_EXECUTABLES;
const MAX_SOURCE_URLS: u32 = MAX_SEED_EXECUTABLES;
const INVALID_INVENTORY_DIGEST: &str = "stage0-inventory-serialization-error";
const MAX_VERSION_EVIDENCE_BYTES: u32 = 4096;
const MAX_VERSION_COMMAND_ARGS: u32 = 16;
const MAX_VERSION_ARG_BYTES: u32 = 4096;
const ENV_STAGE0_SEED_SANDBOX_ENTRY: &str = "CRUNCH_STAGE0_SEED_SANDBOX_ENTRY";
const ENV_STAGE0_SEED_SANDBOX_SHELL: &str = "CRUNCH_STAGE0_SEED_SANDBOX_SHELL";
const ENV_STAGE0_SEED_TOOLCHAIN_ROOT: &str = "CRUNCH_STAGE0_SEED_TOOLCHAIN_ROOT";
const ENV_STAGE0_SEED_BUILD_TOOLS: &str = "CRUNCH_STAGE0_SEED_BUILD_TOOLS";
const GENERATED_INVENTORY_HEADER: &str = "# Generated host-tool-free stage0 inventory.\n# Do not edit by hand; regenerate from explicit CRUNCH_STAGE0_SEED_* inputs.\n\n";
const ELF_MAGIC: &[u8] = b"\x7fELF";
const SHEBANG_MAGIC: &[u8] = b"#!";
const DYNAMIC_LINKER_MARKERS: [&[u8]; 4] = [b"ld-linux", b"ld-musl", b"/lib/ld", b"/lib64/ld"];
const EXTRACTION_RULE_SEPARATOR: char = '=';
const EXTRACTION_RULE_FORMAT: &str = "format";
const EXTRACTION_RULE_STRIP_COMPONENTS: &str = "strip-components";
const EXTRACTION_RULE_ROOT: &str = "root";
const KIB_BYTES: u64 = 1024;
const MIB_BYTES: u64 = KIB_BYTES.saturating_mul(KIB_BYTES);
const MAX_SEED_CLOSURE_RISK_SCAN_BYTES: u64 = MIB_BYTES;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DigestSpec {
    pub algorithm: String,
    pub hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interoperability_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoundedVersionEvidence {
    pub command: Vec<String>,
    pub byte_limit: u32,
    pub output_digest: DigestSpec,
    pub output_sample: String,
    pub exit_code: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutableSeedEntry {
    pub schema_version: String,
    pub id: String,
    pub role: String,
    pub phase: String,
    pub executable_path: PathBuf,
    pub digest: DigestSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_evidence: Option<BoundedVersionEvidence>,
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
pub struct SeedClosureRiskReport {
    pub entry_id: String,
    pub executable_path: PathBuf,
    pub risk: SeedClosureRisk,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeedClosureRisk {
    StaticElfCandidate,
    DynamicElfLikely,
    ScriptInterpreter,
    UnknownExecutableFormat,
    Unreadable(String),
}

impl fmt::Display for SeedClosureRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticElfCandidate => write!(f, "static-elf-candidate"),
            Self::DynamicElfLikely => write!(f, "dynamic-elf-likely"),
            Self::ScriptInterpreter => write!(f, "script-interpreter-risk"),
            Self::UnknownExecutableFormat => write!(f, "unknown-executable-format"),
            Self::Unreadable(message) => write!(f, "unreadable:{message}"),
        }
    }
}

type ExecutableVariantsByPath = BTreeMap<PathBuf, BTreeMap<String, ExecutableSeedEntry>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedExecPolicy {
    executables_by_path: ExecutableVariantsByPath,
    allowed_promotion_source_ids: Option<BTreeSet<String>>,
    inventory_digest_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedExecutable {
    pub authorization_id: String,
    pub source_stage_id: String,
    pub path: PathBuf,
    pub digest_hex: String,
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
    MissingVersionEvidence {
        entry_id: String,
    },
    InvalidVersionEvidence {
        entry_id: String,
        reason: String,
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
    AmbiguousRequiredSeedRole {
        role: String,
        count: u32,
    },
    UndeclaredExecutable {
        path: PathBuf,
    },
    DigestMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    InvalidExtractionRule {
        entry_id: String,
        rule: String,
        reason: String,
    },
    PromotionDuplicatePath {
        path: PathBuf,
        source_entry_id: String,
    },
    PromotionEmptySet {
        source_entry_id: String,
    },
    UndeclaredPromotionSource {
        source_entry_id: String,
    },
    DuplicatePromotionSource {
        source_entry_id: String,
    },
    InventoryCollectionLimitExceeded {
        collection: &'static str,
        limit: u32,
    },
    PolicyLockPoisoned,
    InventorySerializationFailed,
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
            Self::MissingVersionEvidence { entry_id } => {
                write!(f, "entry {entry_id} is missing bounded version evidence")
            }
            Self::InvalidVersionEvidence { entry_id, reason } => {
                write!(f, "entry {entry_id} has invalid bounded version evidence: {reason}")
            }
            Self::MissingInteroperabilityReason { entry_id, algorithm } => {
                write!(f, "entry {entry_id} uses non-blake3 digest {algorithm} without interoperability reason")
            }
            Self::InvalidBlake3Digest { entry_id, digest_hex } => {
                write!(f, "entry {entry_id} has invalid blake3 digest {digest_hex}")
            }
            Self::MissingRequiredSeedRole { role } => write!(f, "missing required seed role {role}"),
            Self::AmbiguousRequiredSeedRole { role, count } => {
                write!(f, "required seed role {role} has {count} entries; expected one")
            }
            Self::UndeclaredExecutable { path } => write!(f, "undeclared executable path: {}", path.display()),
            Self::DigestMismatch { path, expected, actual } => {
                write!(f, "digest mismatch for {}: expected {expected}, got {actual}", path.display())
            }
            Self::InvalidExtractionRule { entry_id, rule, reason } => {
                write!(f, "entry {entry_id} has invalid extraction rule {rule}: {reason}")
            }
            Self::PromotionDuplicatePath { path, source_entry_id } => {
                write!(f, "promotion from source {source_entry_id} collides at {}", path.display())
            }
            Self::PromotionEmptySet { source_entry_id } => {
                write!(f, "promotion from source {source_entry_id} produced no executables")
            }
            Self::UndeclaredPromotionSource { source_entry_id } => {
                write!(f, "promotion source is not declared by the protected plan: {source_entry_id}")
            }
            Self::DuplicatePromotionSource { source_entry_id } => {
                write!(f, "duplicate protected promotion source: {source_entry_id}")
            }
            Self::InventoryCollectionLimitExceeded { collection, limit } => {
                write!(f, "protected exec {collection} exceeds limit {limit}")
            }
            Self::PolicyLockPoisoned => write!(f, "protected exec policy lock is poisoned"),
            Self::InventorySerializationFailed => write!(f, "stage0 inventory serialization failed"),
        }
    }
}

impl std::error::Error for ProtectedExecError {}

impl ProtectedExecPolicy {
    pub fn from_inventory(inventory: Stage0Inventory) -> Result<Self, ProtectedExecError> {
        Self::from_inventory_with_required_roles(inventory, &[ROLE_SANDBOX_ENTRY, ROLE_SANDBOX_SHELL], None, false)
    }

    pub fn from_stagex_plan(
        seed_path: PathBuf,
        seed_digest_blake3: String,
        promotion_stage_ids: &[String],
        planned_executables: &[PlannedExecutable],
    ) -> Result<Self, ProtectedExecError> {
        let seed_entry = ExecutableSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: "stagex:seed:hex0".to_string(),
            role: ROLE_AUDITED_BOOTSTRAP_SEED.to_string(),
            phase: PHASE_PROTECTED.to_string(),
            executable_path: seed_path.clone(),
            digest: DigestSpec {
                algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
                hex: seed_digest_blake3.clone(),
                interoperability_reason: None,
            },
            version_evidence: Some(promoted_version_evidence(&seed_path, &seed_digest_blake3)),
            provenance_category: PROVENANCE_OPERATOR_BOOTSTRAP_SEED.to_string(),
            provenance: "checked audited StageX seed bytes".to_string(),
            allowed_reason: "execute the audited StageX hex0 seed after the protected transition".to_string(),
            owner: "mantle-stagex-lineage".to_string(),
            required: true,
        };
        let allowed_promotion_source_ids = validate_promotion_source_ids(promotion_stage_ids)?;
        let mut executable_entries = Vec::with_capacity(planned_executables.len().saturating_add(1));
        executable_entries.push(seed_entry);
        for planned in planned_executables {
            executable_entries.push(planned_executable_entry(planned, &allowed_promotion_source_ids)?);
        }
        let inventory = Stage0Inventory {
            executable_entries,
            source_entries: Vec::new(),
        };
        Self::from_inventory_with_required_roles(
            inventory,
            &[ROLE_AUDITED_BOOTSTRAP_SEED],
            Some(allowed_promotion_source_ids),
            true,
        )
    }

    fn from_inventory_with_required_roles(
        inventory: Stage0Inventory,
        required_roles: &[&'static str],
        allowed_promotion_source_ids: Option<BTreeSet<String>>,
        allow_digest_variants: bool,
    ) -> Result<Self, ProtectedExecError> {
        let max_executables = usize::try_from(MAX_SEED_EXECUTABLES).map_err(|_| {
            ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            }
        })?;
        if inventory.executable_entries.len() > max_executables {
            return Err(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            });
        }
        let max_sources =
            usize::try_from(MAX_SOURCE_ENTRIES).map_err(|_| ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "source entries",
                limit: MAX_SOURCE_ENTRIES,
            })?;
        if inventory.source_entries.len() > max_sources {
            return Err(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "source entries",
                limit: MAX_SOURCE_ENTRIES,
            });
        }
        validate_required_roles(&inventory.executable_entries, required_roles)?;
        let source_url_count = inventory_source_url_count(&inventory.source_entries)?;
        if source_url_count > MAX_SOURCE_URLS {
            return Err(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "source URLs",
                limit: MAX_SOURCE_URLS,
            });
        }
        let inventory_digest_blake3 = stage0_inventory_digest_blake3(&inventory);
        if inventory_digest_blake3 == INVALID_INVENTORY_DIGEST {
            return Err(ProtectedExecError::InventorySerializationFailed);
        }
        let executables_by_path =
            group_executable_entries(inventory.executable_entries, max_executables, allow_digest_variants)?;

        let mut seen_source_urls = BTreeSet::new();
        for entry in &inventory.source_entries {
            validate_source_entry(entry)?;
            for url in &entry.urls {
                if !seen_source_urls.insert(url.as_str()) {
                    return Err(ProtectedExecError::DuplicateSourceUrl { url: url.clone() });
                }
            }
        }

        assert!(!executables_by_path.is_empty(), "policy must contain required executables");
        assert!(is_valid_hex_digest(&inventory_digest_blake3, BLAKE3_HEX_LEN));
        Ok(Self {
            executables_by_path,
            allowed_promotion_source_ids,
            inventory_digest_blake3,
        })
    }

    pub fn inventory_digest_blake3(&self) -> &str {
        &self.inventory_digest_blake3
    }

    fn executable_entry_count(&self) -> usize {
        let count = self.executables_by_path.values().map(BTreeMap::len).sum::<usize>();
        assert!(count <= usize::try_from(MAX_SEED_EXECUTABLES).unwrap_or(usize::MAX));
        assert!(count >= self.executables_by_path.len());
        count
    }

    pub fn decide_exec(&self, request: &ExecRequest) -> Result<ExecDecision, ProtectedExecError> {
        assert!(request.path.is_absolute(), "exec request path must be absolute");
        assert!(!request.digest_hex.is_empty(), "exec request digest must be present");
        let variants =
            self.executables_by_path
                .get(&request.path)
                .ok_or_else(|| ProtectedExecError::UndeclaredExecutable {
                    path: request.path.clone(),
                })?;
        let entry = variants.get(&request.digest_hex).ok_or_else(|| ProtectedExecError::DigestMismatch {
            path: request.path.clone(),
            expected: variants.keys().cloned().collect::<Vec<_>>().join(","),
            actual: request.digest_hex.clone(),
        })?;
        Ok(ExecDecision {
            allowed: true,
            entry_id: Some(entry.id.clone()),
            reason: entry.allowed_reason.clone(),
        })
    }

    pub fn promote_verified_output(
        &mut self,
        source_entry_id: &str,
        extraction_rules: &[String],
        executables: &[PromotedExecutable],
    ) -> Result<OutputPromotionRecord, ProtectedExecError> {
        if self.allowed_promotion_source_ids.as_ref().is_some_and(|allowed| !allowed.contains(source_entry_id)) {
            return Err(ProtectedExecError::UndeclaredPromotionSource {
                source_entry_id: source_entry_id.to_string(),
            });
        }
        if executables.is_empty() {
            return Err(ProtectedExecError::PromotionEmptySet {
                source_entry_id: source_entry_id.to_string(),
            });
        }
        let max_executables = usize::try_from(MAX_SEED_EXECUTABLES).map_err(|_| {
            ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            }
        })?;
        let resulting_entry_count = self
            .executable_entry_count()
            .checked_add(executables.len())
            .filter(|entry_count| *entry_count <= max_executables)
            .ok_or(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            })?;
        let resulting_policy_entries =
            u32::try_from(resulting_entry_count).map_err(|_| ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            })?;
        for exe in executables {
            assert!(exe.path.is_absolute(), "promoted executable path must be absolute");
            assert!(!exe.digest_hex.is_empty(), "promoted executable digest must be present");
            if self.executables_by_path.contains_key(&exe.path) {
                return Err(ProtectedExecError::PromotionDuplicatePath {
                    path: exe.path.clone(),
                    source_entry_id: source_entry_id.to_string(),
                });
            }
        }
        for exe in executables {
            let entry = ExecutableSeedEntry {
                schema_version: SCHEMA_VERSION_V1.to_string(),
                id: format!("promoted:{source_entry_id}:{}", exe.path.display()),
                role: ROLE_BOOTSTRAP_BUILD_TOOL.to_string(),
                phase: "protected".to_string(),
                executable_path: exe.path.clone(),
                digest: DigestSpec {
                    algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
                    hex: exe.digest_hex.clone(),
                    interoperability_reason: None,
                },
                version_evidence: Some(promoted_version_evidence(&exe.path, &exe.digest_hex)),
                provenance_category: PROVENANCE_OPERATOR_SOURCE_BUILD.to_string(),
                provenance: format!("promoted from source {source_entry_id}"),
                allowed_reason: format!("verified output of declared source {source_entry_id}"),
                owner: "crunch-protected-exec".to_string(),
                required: false,
            };
            let digest = entry.digest.hex.clone();
            let previous = self.executables_by_path.insert(exe.path.clone(), BTreeMap::from([(digest, entry)]));
            assert!(previous.is_none(), "promotion paths were checked before insertion");
        }
        Ok(OutputPromotionRecord {
            source_entry_id: source_entry_id.to_string(),
            extraction_rules: extraction_rules.to_vec(),
            promoted_executables: executables.to_vec(),
            promoted_at_policy_size: resulting_policy_entries,
        })
    }

    pub fn required_sandbox_entry(&self) -> Result<&ExecutableSeedEntry, ProtectedExecError> {
        self.required_executable_for_role(ROLE_SANDBOX_ENTRY)
    }

    pub fn required_sandbox_shell(&self) -> Result<&ExecutableSeedEntry, ProtectedExecError> {
        self.required_executable_for_role(ROLE_SANDBOX_SHELL)
    }

    fn required_executable_for_role(&self, role: &'static str) -> Result<&ExecutableSeedEntry, ProtectedExecError> {
        let matches: Vec<&ExecutableSeedEntry> = self
            .executables_by_path
            .values()
            .flat_map(BTreeMap::values)
            .filter(|entry| entry.required && entry.role == role)
            .collect();
        if matches.is_empty() {
            return Err(ProtectedExecError::MissingRequiredSeedRole { role });
        }
        if matches.len() > 1 {
            let count =
                u32::try_from(matches.len()).map_err(|_| ProtectedExecError::InventoryCollectionLimitExceeded {
                    collection: "required role matches",
                    limit: MAX_SEED_EXECUTABLES,
                })?;
            return Err(ProtectedExecError::AmbiguousRequiredSeedRole {
                role: role.to_string(),
                count,
            });
        }
        Ok(matches[0])
    }
}

fn group_executable_entries(
    entries: Vec<ExecutableSeedEntry>,
    max_executables: usize,
    allow_digest_variants: bool,
) -> Result<ExecutableVariantsByPath, ProtectedExecError> {
    let max_digest_variants = usize::try_from(MAX_EXECUTABLE_DIGEST_VARIANTS_PER_PATH).map_err(|_| {
        ProtectedExecError::InventoryCollectionLimitExceeded {
            collection: "executable digest variants per path",
            limit: MAX_EXECUTABLE_DIGEST_VARIANTS_PER_PATH,
        }
    })?;
    let mut grouped = ExecutableVariantsByPath::new();
    let mut executable_entry_count = 0_usize;
    for entry in entries {
        if executable_entry_count >= max_executables {
            return Err(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            });
        }
        validate_executable_entry(&entry)?;
        let path = entry.executable_path.clone();
        let digest = entry.digest.hex.clone();
        let variants = grouped.entry(path.clone()).or_default();
        if !variants.is_empty() && !allow_digest_variants {
            return Err(ProtectedExecError::DuplicateExecutablePath { path });
        }
        if variants.len() >= max_digest_variants {
            return Err(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable digest variants per path",
                limit: MAX_EXECUTABLE_DIGEST_VARIANTS_PER_PATH,
            });
        }
        if variants.insert(digest, entry).is_some() {
            return Err(ProtectedExecError::DuplicateExecutablePath { path });
        }
        executable_entry_count =
            executable_entry_count.checked_add(1).ok_or(ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "executable entries",
                limit: MAX_SEED_EXECUTABLES,
            })?;
    }
    assert_eq!(executable_entry_count, grouped.values().map(BTreeMap::len).sum::<usize>());
    assert!(grouped.values().all(|variants| !variants.is_empty()));
    Ok(grouped)
}

fn validate_promotion_source_ids(source_ids: &[String]) -> Result<BTreeSet<String>, ProtectedExecError> {
    if source_ids.is_empty() {
        return Err(ProtectedExecError::EmptyField {
            entry_id: "stagex-plan".to_string(),
            field: "promotion_stage_ids",
        });
    }
    let max_sources =
        usize::try_from(MAX_SOURCE_ENTRIES).map_err(|_| ProtectedExecError::InventoryCollectionLimitExceeded {
            collection: "promotion source IDs",
            limit: MAX_SOURCE_ENTRIES,
        })?;
    if source_ids.len() > max_sources {
        return Err(ProtectedExecError::InventoryCollectionLimitExceeded {
            collection: "promotion source IDs",
            limit: MAX_SOURCE_ENTRIES,
        });
    }
    let mut unique = BTreeSet::new();
    for source_id in source_ids {
        if source_id.trim().is_empty() {
            return Err(ProtectedExecError::EmptyField {
                entry_id: "stagex-plan".to_string(),
                field: "promotion_stage_ids[]",
            });
        }
        if !unique.insert(source_id.clone()) {
            return Err(ProtectedExecError::DuplicatePromotionSource {
                source_entry_id: source_id.clone(),
            });
        }
    }
    Ok(unique)
}

fn validate_required_roles(
    entries: &[ExecutableSeedEntry],
    required_roles: &[&'static str],
) -> Result<(), ProtectedExecError> {
    assert!(!entries.is_empty(), "inventory must include executable entries");
    assert!(!required_roles.is_empty(), "required executable roles must not be empty");
    let roles: BTreeSet<&str> =
        entries.iter().filter(|entry| entry.required).map(|entry| entry.role.as_str()).collect();
    for role in required_roles {
        if !roles.contains(role) {
            return Err(ProtectedExecError::MissingRequiredSeedRole { role });
        }
    }
    Ok(())
}

struct CommonFields<'entry> {
    schema_version: &'entry str,
    entry_id: &'entry str,
    role: &'entry str,
    phase: &'entry str,
    digest: &'entry DigestSpec,
    provenance_category: &'entry str,
    provenance: &'entry str,
    allowed_reason: &'entry str,
    owner: &'entry str,
}

struct RequiredField<'entry> {
    entry_id: &'entry str,
    field: &'static str,
    value: &'entry str,
}

#[derive(Clone, Copy)]
struct ExtractionRuleInput<'rule> {
    entry_id: &'rule str,
    rule: &'rule str,
}

struct ExtractionRuleValue<'rule> {
    input: ExtractionRuleInput<'rule>,
    key: &'rule str,
    value: &'rule str,
}

struct ProvenanceInput<'entry> {
    entry_id: &'entry str,
    category: &'entry str,
}

fn validate_executable_entry(entry: &ExecutableSeedEntry) -> Result<(), ProtectedExecError> {
    validate_common_fields(CommonFields {
        schema_version: &entry.schema_version,
        entry_id: &entry.id,
        role: &entry.role,
        phase: &entry.phase,
        digest: &entry.digest,
        provenance_category: &entry.provenance_category,
        provenance: &entry.provenance,
        allowed_reason: &entry.allowed_reason,
        owner: &entry.owner,
    })?;
    let version_evidence =
        entry.version_evidence.as_ref().ok_or_else(|| ProtectedExecError::MissingVersionEvidence {
            entry_id: entry.id.clone(),
        })?;
    validate_version_evidence(entry.id.as_str(), version_evidence)?;
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
    assert_eq!(entry.schema_version, SCHEMA_VERSION_V1);
    assert!(entry.executable_path.is_absolute());
    Ok(())
}

fn validate_source_entry(entry: &SourceSeedEntry) -> Result<(), ProtectedExecError> {
    validate_common_fields(CommonFields {
        schema_version: &entry.schema_version,
        entry_id: &entry.id,
        role: &entry.role,
        phase: &entry.phase,
        digest: &entry.digest,
        provenance_category: &entry.provenance_category,
        provenance: &entry.provenance,
        allowed_reason: &entry.allowed_reason,
        owner: &entry.owner,
    })?;
    let first_url = entry.urls.first().map_or("", String::as_str);
    validate_non_empty(RequiredField {
        entry_id: entry.id.as_str(),
        field: "urls",
        value: first_url,
    })?;
    for url in &entry.urls {
        validate_non_empty(RequiredField {
            entry_id: entry.id.as_str(),
            field: "urls[]",
            value: url,
        })?;
    }
    let first_rule = entry.extraction_rules.first().map_or("", String::as_str);
    validate_non_empty(RequiredField {
        entry_id: entry.id.as_str(),
        field: "extraction_rules",
        value: first_rule,
    })?;
    for rule in &entry.extraction_rules {
        validate_non_empty(RequiredField {
            entry_id: entry.id.as_str(),
            field: "extraction_rules[]",
            value: rule,
        })?;
    }
    validate_extraction_rules(entry.id.as_str(), &entry.extraction_rules)?;
    assert!(!entry.urls.is_empty());
    assert!(!entry.extraction_rules.is_empty());
    Ok(())
}

fn validate_extraction_rules(entry_id: &str, rules: &[String]) -> Result<(), ProtectedExecError> {
    let mut seen = BTreeSet::new();
    for rule in rules {
        let input = ExtractionRuleInput { entry_id, rule };
        let (key, value) = parse_extraction_rule(input)?;
        if !is_allowed_extraction_rule_key(key) {
            return invalid_extraction_rule(input, format!("unsupported key {key}"));
        }
        if !seen.insert(key.to_string()) {
            return invalid_extraction_rule(input, format!("duplicate key {key}"));
        }
        validate_extraction_rule_value(ExtractionRuleValue { input, key, value })?;
    }
    Ok(())
}

fn parse_extraction_rule(input: ExtractionRuleInput<'_>) -> Result<(&str, &str), ProtectedExecError> {
    let mut parts = input.rule.split(EXTRACTION_RULE_SEPARATOR);
    let key = parts.next().unwrap_or_default();
    let value = parts.next().unwrap_or_default();
    if parts.next().is_some() {
        return invalid_extraction_rule(input, "multiple separators".to_string());
    }
    if key.is_empty() {
        return invalid_extraction_rule(input, "empty key".to_string());
    }
    if value.is_empty() {
        return invalid_extraction_rule(input, "empty value".to_string());
    }
    Ok((key, value))
}

fn is_allowed_extraction_rule_key(key: &str) -> bool {
    matches!(key, EXTRACTION_RULE_FORMAT | EXTRACTION_RULE_STRIP_COMPONENTS | EXTRACTION_RULE_ROOT)
}

fn validate_extraction_rule_value(rule: ExtractionRuleValue<'_>) -> Result<(), ProtectedExecError> {
    if rule.key == EXTRACTION_RULE_STRIP_COMPONENTS && rule.value.parse::<u32>().is_err() {
        return invalid_extraction_rule(rule.input, "strip-components must be an unsigned integer".to_string());
    }
    if rule.key == EXTRACTION_RULE_ROOT && rule.value.contains("..") {
        return invalid_extraction_rule(rule.input, "root must not contain parent traversal".to_string());
    }
    Ok(())
}

fn invalid_extraction_rule<T>(input: ExtractionRuleInput<'_>, reason: String) -> Result<T, ProtectedExecError> {
    Err(ProtectedExecError::InvalidExtractionRule {
        entry_id: input.entry_id.to_string(),
        rule: input.rule.to_string(),
        reason,
    })
}

fn validate_common_fields(fields: CommonFields<'_>) -> Result<(), ProtectedExecError> {
    validate_entry_id(fields.entry_id)?;
    for field in [
        RequiredField {
            entry_id: fields.entry_id,
            field: "role",
            value: fields.role,
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "phase",
            value: fields.phase,
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "digest.algorithm",
            value: fields.digest.algorithm.as_str(),
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "digest.hex",
            value: fields.digest.hex.as_str(),
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "provenance_category",
            value: fields.provenance_category,
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "provenance",
            value: fields.provenance,
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "allowed_reason",
            value: fields.allowed_reason,
        },
        RequiredField {
            entry_id: fields.entry_id,
            field: "owner",
            value: fields.owner,
        },
    ] {
        validate_non_empty(field)?;
    }
    if fields.schema_version != SCHEMA_VERSION_V1 {
        return Err(ProtectedExecError::UnsupportedSchemaVersion {
            entry_id: fields.entry_id.to_string(),
            actual: fields.schema_version.to_string(),
        });
    }
    if fields.phase != PHASE_PROTECTED {
        return Err(ProtectedExecError::DisallowedRole {
            entry_id: fields.entry_id.to_string(),
            role: fields.phase.to_string(),
        });
    }
    validate_digest(fields.entry_id, fields.digest)?;
    validate_provenance(ProvenanceInput {
        entry_id: fields.entry_id,
        category: fields.provenance_category,
    })?;
    assert_eq!(fields.schema_version, SCHEMA_VERSION_V1);
    assert_eq!(fields.phase, PHASE_PROTECTED);
    Ok(())
}

fn validate_entry_id(entry_id: &str) -> Result<(), ProtectedExecError> {
    validate_non_empty(RequiredField {
        entry_id,
        field: "id",
        value: entry_id,
    })
}

fn validate_non_empty(field: RequiredField<'_>) -> Result<(), ProtectedExecError> {
    if field.value.is_empty() {
        return Err(ProtectedExecError::EmptyField {
            entry_id: field.entry_id.to_string(),
            field: field.field,
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
        Some(_) | None => Err(ProtectedExecError::MissingInteroperabilityReason {
            entry_id: entry_id.to_string(),
            algorithm: digest.algorithm.clone(),
        }),
    }
}

fn validate_version_evidence(entry_id: &str, evidence: &BoundedVersionEvidence) -> Result<(), ProtectedExecError> {
    let command_len =
        u32::try_from(evidence.command.len()).map_err(|_| ProtectedExecError::InvalidVersionEvidence {
            entry_id: entry_id.to_string(),
            reason: "command argument count cannot be represented as u32".to_string(),
        })?;
    if command_len == 0 {
        return invalid_version_evidence(entry_id, "command must not be empty");
    }
    if command_len > MAX_VERSION_COMMAND_ARGS {
        return invalid_version_evidence(
            entry_id,
            format!("command has {command_len} args; limit {MAX_VERSION_COMMAND_ARGS}"),
        );
    }
    for arg in &evidence.command {
        if arg.is_empty() {
            return invalid_version_evidence(entry_id, "command args must not be empty");
        }
        let arg_len = u32::try_from(arg.len()).map_err(|_| ProtectedExecError::InvalidVersionEvidence {
            entry_id: entry_id.to_string(),
            reason: "command argument length cannot be represented as u32".to_string(),
        })?;
        if arg_len > MAX_VERSION_ARG_BYTES {
            return invalid_version_evidence(
                entry_id,
                format!("command arg has {arg_len} bytes; limit {MAX_VERSION_ARG_BYTES}"),
            );
        }
    }
    if evidence.byte_limit == 0 || evidence.byte_limit > MAX_VERSION_EVIDENCE_BYTES {
        return invalid_version_evidence(
            entry_id,
            format!("byte_limit must be 1..={MAX_VERSION_EVIDENCE_BYTES}, got {}", evidence.byte_limit),
        );
    }
    let sample_len =
        u32::try_from(evidence.output_sample.len()).map_err(|_| ProtectedExecError::InvalidVersionEvidence {
            entry_id: entry_id.to_string(),
            reason: "output sample length cannot be represented as u32".to_string(),
        })?;
    if sample_len > evidence.byte_limit {
        return invalid_version_evidence(
            entry_id,
            format!("output sample has {sample_len} bytes; limit {}", evidence.byte_limit),
        );
    }
    validate_digest(entry_id, &evidence.output_digest)?;
    if evidence.output_digest.algorithm == DIGEST_ALGORITHM_BLAKE3 {
        let actual = blake3::hash(evidence.output_sample.as_bytes()).to_hex().to_string();
        if actual != evidence.output_digest.hex {
            return invalid_version_evidence(entry_id, "output digest does not match bounded output sample");
        }
    }
    assert!(!evidence.command.is_empty());
    assert!(evidence.byte_limit <= MAX_VERSION_EVIDENCE_BYTES);
    Ok(())
}

fn invalid_version_evidence<T>(entry_id: &str, reason: impl Into<String>) -> Result<T, ProtectedExecError> {
    Err(ProtectedExecError::InvalidVersionEvidence {
        entry_id: entry_id.to_string(),
        reason: reason.into(),
    })
}

fn validate_provenance(input: ProvenanceInput<'_>) -> Result<(), ProtectedExecError> {
    if input.category == PROVENANCE_HOST_PATH_DISCOVERY || input.category == PROVENANCE_NIX_STORE_DISCOVERY {
        return Err(ProtectedExecError::DisallowedProvenance {
            entry_id: input.entry_id.to_string(),
            provenance_category: input.category.to_string(),
        });
    }
    if matches!(
        input.category,
        PROVENANCE_OPERATOR_SOURCE_BUILD | PROVENANCE_OPERATOR_BOOTSTRAP_SEED | PROVENANCE_TEST_FIXTURE
    ) {
        return Ok(());
    }
    Err(ProtectedExecError::DisallowedProvenance {
        entry_id: input.entry_id.to_string(),
        provenance_category: input.category.to_string(),
    })
}

fn inventory_source_url_count(entries: &[SourceSeedEntry]) -> Result<u32, ProtectedExecError> {
    let mut total = 0_u32;
    for entry in entries {
        let count =
            u32::try_from(entry.urls.len()).map_err(|_| ProtectedExecError::InventoryCollectionLimitExceeded {
                collection: "source URLs",
                limit: MAX_SOURCE_URLS,
            })?;
        total = total.checked_add(count).ok_or(ProtectedExecError::InventoryCollectionLimitExceeded {
            collection: "source URLs",
            limit: MAX_SOURCE_URLS,
        })?;
    }
    Ok(total)
}

fn is_allowed_executable_role(role: &str) -> bool {
    matches!(
        role,
        ROLE_STAGE0_CRUNCH
            | ROLE_SANDBOX_ENTRY
            | ROLE_SANDBOX_SHELL
            | ROLE_BOOTSTRAP_TOOLCHAIN_TOOL
            | ROLE_BOOTSTRAP_BUILD_TOOL
            | ROLE_AUDITED_BOOTSTRAP_SEED
    )
}

fn is_forbidden_nix_executable(path: &Path) -> bool {
    match path.file_name().and_then(|name| name.to_str()) {
        Some("nix") | Some("nix-build") | Some("nix-store") | Some("nix-shell") | Some("nix-develop") => true,
        Some(name) => name.starts_with("nix") && name.contains("develop"),
        None => false,
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
        .filter(|component| !component.is_empty() && *component != "/")
        .collect::<Vec<_>>()
        .join(PATH_SEPARATOR.encode_utf8(&mut [0; 4]));
    if path.is_absolute() {
        return format!("/{body}");
    }
    body
}

pub fn stage0_inventory_digest_blake3(inventory: &Stage0Inventory) -> String {
    let mut canonical = inventory.clone();
    canonical.executable_entries.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.executable_path.cmp(&right.executable_path))
            .then_with(|| left.role.cmp(&right.role))
    });
    canonical
        .source_entries
        .sort_by(|left, right| left.id.cmp(&right.id).then_with(|| left.urls.cmp(&right.urls)));
    let bytes = match serde_json::to_vec(&canonical) {
        Ok(bytes) => bytes,
        Err(_) => return INVALID_INVENTORY_DIGEST.to_string(),
    };
    blake3::hash(&bytes).to_hex().to_string()
}

pub fn bounded_version_evidence(command: Vec<String>, output_sample: String, exit_code: i32) -> BoundedVersionEvidence {
    let output_digest = DigestSpec {
        algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
        hex: blake3::hash(output_sample.as_bytes()).to_hex().to_string(),
        interoperability_reason: None,
    };
    BoundedVersionEvidence {
        command,
        byte_limit: MAX_VERSION_EVIDENCE_BYTES,
        output_digest,
        output_sample,
        exit_code,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtectedSeccompAuditEvent {
    pub pid: u32,
    pub syscall: String,
    /// Resolved host path whose bytes were hashed and checked against policy.
    pub executable_path: PathBuf,
    /// Raw executable path read from tracee memory before namespace resolution.
    pub tracee_path: PathBuf,
    /// Host path resolved through the tracee root namespace.
    pub resolved_host_path: PathBuf,
    pub digest_hex: String,
    pub reason: String,
    pub phase: String,
    pub inventory_entry_id: Option<String>,
    pub policy_decision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedLaunchAuditEvent {
    pub executable_path: PathBuf,
    pub digest_hex: String,
    pub reason: String,
    pub phase: String,
    pub inventory_entry_id: String,
    pub policy_decision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedLaunchPlan {
    pub executable_path: PathBuf,
    pub digest_hex: String,
    pub inventory_entry_id: String,
    pub reason: String,
    pub phase: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromotedExecutable {
    pub path: PathBuf,
    pub digest_hex: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputPromotionRecord {
    pub source_entry_id: String,
    pub extraction_rules: Vec<String>,
    pub promoted_executables: Vec<PromotedExecutable>,
    pub promoted_at_policy_size: u32,
}

impl ProtectedLaunchPlan {
    pub fn audit_event(&self) -> ProtectedLaunchAuditEvent {
        ProtectedLaunchAuditEvent {
            executable_path: self.executable_path.clone(),
            digest_hex: self.digest_hex.clone(),
            reason: self.reason.clone(),
            phase: self.phase.clone(),
            inventory_entry_id: self.inventory_entry_id.clone(),
            policy_decision: "allowed".to_string(),
        }
    }
}

pub fn plan_protected_launch(
    policy: &ProtectedExecPolicy,
    executable_path: &Path,
) -> Result<ProtectedLaunchPlan, Stage0InventoryGenerationError> {
    assert!(executable_path.is_absolute(), "protected launch path must be absolute");
    let digest_hex = blake3_file_hex(executable_path)?;
    let decision = policy.decide_exec(&ExecRequest {
        path: executable_path.to_path_buf(),
        digest_hex: digest_hex.clone(),
    })?;
    let entry_id = decision.entry_id.unwrap_or_default();
    assert!(!entry_id.is_empty(), "allowed protected launch must name inventory entry");
    Ok(ProtectedLaunchPlan {
        executable_path: executable_path.to_path_buf(),
        digest_hex,
        inventory_entry_id: entry_id,
        reason: decision.reason,
        phase: PHASE_PROTECTED.to_string(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredSandboxSeed {
    pub sandbox_entry: ProtectedLaunchPlan,
    pub sandbox_shell: ProtectedLaunchPlan,
}

impl DeclaredSandboxSeed {
    pub fn audit_events(&self) -> [ProtectedLaunchAuditEvent; 2] {
        [self.sandbox_entry.audit_event(), self.sandbox_shell.audit_event()]
    }
}

pub fn select_declared_sandbox_seed(
    policy: &ProtectedExecPolicy,
) -> Result<DeclaredSandboxSeed, Stage0InventoryGenerationError> {
    let sandbox_entry = policy.required_sandbox_entry()?;
    let sandbox_shell = policy.required_sandbox_shell()?;
    Ok(DeclaredSandboxSeed {
        sandbox_entry: plan_protected_launch(policy, &sandbox_entry.executable_path)?,
        sandbox_shell: plan_protected_launch(policy, &sandbox_shell.executable_path)?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage0InventorySeedConfig {
    pub sandbox_entry: PathBuf,
    pub sandbox_shell: PathBuf,
    pub toolchain_root: PathBuf,
    pub build_tool_inputs: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stage0InventoryGenerationError {
    MissingEnv { var: &'static str },
    EmptyEnv { var: &'static str },
    NonUtf8Path { path: PathBuf },
    RelativeInputPath { path: PathBuf },
    MissingInputPath { path: PathBuf },
    NotExecutable { path: PathBuf },
    TooManySeedExecutables { limit: u32, actual: u32 },
    WalkDepthExceeded { path: PathBuf, limit: u32 },
    WalkEntryLimitExceeded { path: PathBuf, limit: u32 },
    Io { path: PathBuf, message: String },
    Policy(ProtectedExecError),
}

impl fmt::Display for Stage0InventoryGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { var } => write!(f, "missing required environment variable {var}"),
            Self::EmptyEnv { var } => write!(f, "environment variable {var} must not be empty"),
            Self::NonUtf8Path { path } => write!(f, "path is not valid UTF-8: {}", path.display()),
            Self::RelativeInputPath { path } => write!(f, "seed input path must be absolute: {}", path.display()),
            Self::MissingInputPath { path } => write!(f, "seed input path does not exist: {}", path.display()),
            Self::NotExecutable { path } => write!(f, "seed input path is not executable: {}", path.display()),
            Self::TooManySeedExecutables { limit, actual } => {
                write!(f, "too many seed executables: {actual} > {limit}")
            }
            Self::WalkDepthExceeded { path, limit } => {
                write!(f, "seed executable walk exceeded depth {limit} at {}", path.display())
            }
            Self::WalkEntryLimitExceeded { path, limit } => {
                write!(f, "seed executable walk exceeded entry limit {limit} at {}", path.display())
            }
            Self::Io { path, message } => write!(f, "I/O error at {}: {message}", path.display()),
            Self::Policy(err) => write!(f, "stage0 inventory policy error: {err}"),
        }
    }
}

impl std::error::Error for Stage0InventoryGenerationError {}

impl From<ProtectedExecError> for Stage0InventoryGenerationError {
    fn from(value: ProtectedExecError) -> Self {
        Self::Policy(value)
    }
}

pub fn stage0_seed_config_from_process_env() -> Result<Stage0InventorySeedConfig, Stage0InventoryGenerationError> {
    let env: BTreeMap<OsString, OsString> = std::env::vars_os().collect();
    stage0_seed_config_from_env_map(&env)
}

pub fn stage0_seed_config_from_env_map(
    env: &BTreeMap<OsString, OsString>,
) -> Result<Stage0InventorySeedConfig, Stage0InventoryGenerationError> {
    let sandbox_entry = required_env_path(env, ENV_STAGE0_SEED_SANDBOX_ENTRY)?;
    let sandbox_shell = required_env_path(env, ENV_STAGE0_SEED_SANDBOX_SHELL)?;
    let toolchain_root = required_env_path(env, ENV_STAGE0_SEED_TOOLCHAIN_ROOT)?;
    let build_tool_inputs = required_env_path_list(env, ENV_STAGE0_SEED_BUILD_TOOLS)?;
    Ok(Stage0InventorySeedConfig {
        sandbox_entry,
        sandbox_shell,
        toolchain_root,
        build_tool_inputs,
    })
}

pub fn build_stage0_inventory_from_seed_config(
    config: &Stage0InventorySeedConfig,
) -> Result<Stage0Inventory, Stage0InventoryGenerationError> {
    validate_absolute_existing_path(&config.sandbox_entry)?;
    validate_absolute_existing_path(&config.sandbox_shell)?;
    validate_absolute_existing_path(&config.toolchain_root)?;
    let mut executable_entries = Vec::new();
    executable_entries.push(seed_executable_entry(SeedExecutableSpec {
        id: "sandbox-entry",
        role: ROLE_SANDBOX_ENTRY,
        path: &config.sandbox_entry,
        required: true,
        allowed_reason: "operator supplied protected sandbox entry",
    })?);
    executable_entries.push(seed_executable_entry(SeedExecutableSpec {
        id: "sandbox-shell",
        role: ROLE_SANDBOX_SHELL,
        path: &config.sandbox_shell,
        required: true,
        allowed_reason: "operator supplied protected sandbox shell",
    })?);

    let toolchain_executables = collect_seed_executables(&config.toolchain_root)?;
    append_seed_executable_entries(SeedExecutableAppend {
        entries: &mut executable_entries,
        role: ROLE_BOOTSTRAP_TOOLCHAIN_TOOL,
        prefix: "toolchain",
        root: &config.toolchain_root,
        executable_paths: &toolchain_executables,
    })?;

    for input in &config.build_tool_inputs {
        validate_absolute_existing_path(input)?;
        let build_tools = collect_seed_executables(input)?;
        append_seed_executable_entries(SeedExecutableAppend {
            entries: &mut executable_entries,
            role: ROLE_BOOTSTRAP_BUILD_TOOL,
            prefix: "build-tool",
            root: input,
            executable_paths: &build_tools,
        })?;
    }

    executable_entries.sort_by(|left, right| left.id.cmp(&right.id));
    let inventory = Stage0Inventory {
        executable_entries,
        source_entries: Vec::new(),
    };
    ProtectedExecPolicy::from_inventory(inventory.clone())?;
    assert!(inventory.executable_entries.len() >= 2);
    assert!(inventory.source_entries.is_empty());
    Ok(inventory)
}

pub fn write_generated_stage0_inventory(
    config: &Stage0InventorySeedConfig,
    output_path: &Path,
) -> Result<Stage0Inventory, Stage0InventoryGenerationError> {
    let inventory = build_stage0_inventory_from_seed_config(config)?;
    let rendered = render_stage0_inventory_nickel(&inventory)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    }
    fs::write(output_path, rendered).map_err(|err| io_error(output_path, err))?;
    Ok(inventory)
}

pub fn write_generated_stage0_inventory_from_env(
    output_path: &Path,
) -> Result<Stage0Inventory, Stage0InventoryGenerationError> {
    let config = stage0_seed_config_from_process_env()?;
    write_generated_stage0_inventory(&config, output_path)
}

pub fn seed_closure_risk_report(inventory: &Stage0Inventory) -> Vec<SeedClosureRiskReport> {
    inventory
        .executable_entries
        .iter()
        .map(|entry| SeedClosureRiskReport {
            entry_id: entry.id.clone(),
            executable_path: entry.executable_path.clone(),
            risk: classify_seed_closure_risk(&entry.executable_path),
        })
        .collect()
}

fn classify_seed_closure_risk(path: &Path) -> SeedClosureRisk {
    match read_seed_risk_prefix(path) {
        Ok(bytes) => classify_seed_closure_risk_bytes(&bytes),
        Err(err) => SeedClosureRisk::Unreadable(err.to_string()),
    }
}

fn read_seed_risk_prefix(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut limited_bytes = (&mut file).take(MAX_SEED_CLOSURE_RISK_SCAN_BYTES);
    let mut bytes = Vec::new();
    limited_bytes.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Classify supplied seed bytes by file signature and linker markers.
/// Input bounds belong to the caller; `StaticElfCandidate` does not prove static linkage.
pub fn classify_seed_closure_risk_bytes(bytes: &[u8]) -> SeedClosureRisk {
    if bytes.starts_with(SHEBANG_MAGIC) {
        return SeedClosureRisk::ScriptInterpreter;
    }
    if !bytes.starts_with(ELF_MAGIC) {
        return SeedClosureRisk::UnknownExecutableFormat;
    }
    if DYNAMIC_LINKER_MARKERS
        .iter()
        .any(|marker| bytes.windows(marker.len()).any(|window| window == *marker))
    {
        return SeedClosureRisk::DynamicElfLikely;
    }
    SeedClosureRisk::StaticElfCandidate
}

pub fn render_stage0_inventory_nickel(inventory: &Stage0Inventory) -> Result<String, Stage0InventoryGenerationError> {
    let mut out = String::from(GENERATED_INVENTORY_HEADER);
    out.push_str("{\n");
    out.push_str("  executable_entries = [\n");
    for entry in &inventory.executable_entries {
        push_executable_entry_nickel(&mut out, entry)?;
    }
    out.push_str("  ],\n");
    out.push_str("  source_entries = [\n");
    for entry in &inventory.source_entries {
        push_source_entry_nickel(&mut out, entry);
    }
    out.push_str("  ],\n");
    out.push_str("}\n");
    Ok(out)
}

fn required_env_path(
    env: &BTreeMap<OsString, OsString>,
    var: &'static str,
) -> Result<PathBuf, Stage0InventoryGenerationError> {
    let value = env.get(OsStr::new(var)).ok_or(Stage0InventoryGenerationError::MissingEnv { var })?;
    if value.is_empty() {
        return Err(Stage0InventoryGenerationError::EmptyEnv { var });
    }
    let path = PathBuf::from(value);
    validate_absolute_path_only(&path)?;
    Ok(path)
}

fn required_env_path_list(
    env: &BTreeMap<OsString, OsString>,
    var: &'static str,
) -> Result<Vec<PathBuf>, Stage0InventoryGenerationError> {
    let value = env.get(OsStr::new(var)).ok_or(Stage0InventoryGenerationError::MissingEnv { var })?;
    if value.is_empty() {
        return Err(Stage0InventoryGenerationError::EmptyEnv { var });
    }
    let paths: Vec<PathBuf> = std::env::split_paths(value).collect();
    if paths.is_empty() {
        return Err(Stage0InventoryGenerationError::EmptyEnv { var });
    }
    for path in &paths {
        validate_absolute_path_only(path)?;
    }
    Ok(paths)
}

fn validate_absolute_path_only(path: &Path) -> Result<(), Stage0InventoryGenerationError> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(Stage0InventoryGenerationError::RelativeInputPath {
        path: path.to_path_buf(),
    })
}

fn validate_absolute_existing_path(path: &Path) -> Result<(), Stage0InventoryGenerationError> {
    validate_absolute_path_only(path)?;
    if path.exists() {
        return Ok(());
    }
    Err(Stage0InventoryGenerationError::MissingInputPath {
        path: path.to_path_buf(),
    })
}

fn validate_executable_file(path: &Path) -> Result<(), Stage0InventoryGenerationError> {
    validate_absolute_existing_path(path)?;
    let metadata = fs::metadata(path).map_err(|err| io_error(path, err))?;
    if !metadata.is_file() {
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: path.to_path_buf(),
        });
    }
    if !is_executable_metadata(&metadata) {
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

struct SeedExecutableAppend<'seed> {
    entries: &'seed mut Vec<ExecutableSeedEntry>,
    role: &'seed str,
    prefix: &'seed str,
    root: &'seed Path,
    executable_paths: &'seed [PathBuf],
}

struct SeedExecutableSpec<'seed> {
    id: &'seed str,
    role: &'seed str,
    path: &'seed Path,
    required: bool,
    allowed_reason: &'seed str,
}

fn append_seed_executable_entries(input: SeedExecutableAppend<'_>) -> Result<(), Stage0InventoryGenerationError> {
    assert!(!input.role.is_empty(), "seed role must not be empty");
    assert!(!input.prefix.is_empty(), "seed id prefix must not be empty");
    let max_entries =
        usize::try_from(MAX_SEED_EXECUTABLES).map_err(|_| Stage0InventoryGenerationError::TooManySeedExecutables {
            limit: MAX_SEED_EXECUTABLES,
            actual: MAX_SEED_EXECUTABLES.saturating_add(1),
        })?;
    for path in input.executable_paths {
        if input.entries.len() >= max_entries {
            return Err(Stage0InventoryGenerationError::TooManySeedExecutables {
                limit: MAX_SEED_EXECUTABLES,
                actual: MAX_SEED_EXECUTABLES.saturating_add(1),
            });
        }
        let id = seed_id_for_path(input.prefix, input.root, path);
        input.entries.push(seed_executable_entry(SeedExecutableSpec {
            id: &id,
            role: input.role,
            path,
            required: true,
            allowed_reason: "operator supplied protected bootstrap seed",
        })?);
    }
    Ok(())
}

fn seed_executable_entry(input: SeedExecutableSpec<'_>) -> Result<ExecutableSeedEntry, Stage0InventoryGenerationError> {
    assert!(!input.id.is_empty(), "seed id must not be empty");
    assert!(!input.role.is_empty(), "seed role must not be empty");
    validate_executable_file(input.path)?;
    let digest_hex = blake3_file_hex(input.path)?;
    Ok(ExecutableSeedEntry {
        schema_version: SCHEMA_VERSION_V1.to_string(),
        id: input.id.to_string(),
        role: input.role.to_string(),
        phase: PHASE_PROTECTED.to_string(),
        executable_path: input.path.to_path_buf(),
        digest: DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            hex: digest_hex.clone(),
            interoperability_reason: None,
        },
        version_evidence: Some(seed_version_evidence(input.path, &digest_hex)),
        provenance_category: PROVENANCE_OPERATOR_BOOTSTRAP_SEED.to_string(),
        provenance: "explicit operator-supplied seed path".to_string(),
        allowed_reason: input.allowed_reason.to_string(),
        owner: "bootstrap".to_string(),
        required: input.required,
    })
}

fn seed_version_evidence(path: &Path, digest_hex: &str) -> BoundedVersionEvidence {
    assert!(!digest_hex.is_empty(), "seed version evidence must bind digest");
    let sample = format!("stage0 seed executable bytes are bound by blake3 {digest_hex}\n");
    bounded_version_evidence(vec![path.display().to_string(), "--version".to_string()], sample, 0)
}

fn planned_executable_entry(
    planned: &PlannedExecutable,
    allowed_source_ids: &BTreeSet<String>,
) -> Result<ExecutableSeedEntry, ProtectedExecError> {
    if !allowed_source_ids.contains(&planned.source_stage_id) {
        return Err(ProtectedExecError::UndeclaredPromotionSource {
            source_entry_id: planned.source_stage_id.clone(),
        });
    }
    let entry = ExecutableSeedEntry {
        schema_version: SCHEMA_VERSION_V1.to_string(),
        id: format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id),
        role: ROLE_BOOTSTRAP_BUILD_TOOL.to_string(),
        phase: PHASE_PROTECTED.to_string(),
        executable_path: planned.path.clone(),
        digest: DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            hex: planned.digest_hex.clone(),
            interoperability_reason: None,
        },
        version_evidence: Some(planned_version_evidence(&planned.path, &planned.digest_hex)),
        provenance_category: PROVENANCE_OPERATOR_SOURCE_BUILD.to_string(),
        provenance: format!("planned output of StageX stage {}", planned.source_stage_id),
        allowed_reason: format!(
            "execute only after the StageX plan produces the exact digest for stage {}",
            planned.source_stage_id
        ),
        owner: "mantle-stagex-lineage".to_string(),
        required: false,
    };
    validate_executable_entry(&entry)?;
    assert!(entry.executable_path.is_absolute());
    assert_eq!(entry.digest.hex, planned.digest_hex);
    Ok(entry)
}

fn promoted_version_evidence(path: &Path, digest_hex: &str) -> BoundedVersionEvidence {
    assert!(!digest_hex.is_empty(), "promoted version evidence must bind digest");
    let sample = format!("promoted executable bytes are bound by blake3 {digest_hex}\n");
    bounded_version_evidence(vec![path.display().to_string(), "--version".to_string()], sample, 0)
}

fn planned_version_evidence(path: &Path, digest_hex: &str) -> BoundedVersionEvidence {
    assert!(!digest_hex.is_empty(), "planned version evidence must bind digest");
    let sample = format!("planned executable bytes must match blake3 {digest_hex} at exec notification\n");
    bounded_version_evidence(vec![path.display().to_string(), "--version".to_string()], sample, 0)
}

fn collect_seed_executables(root: &Path) -> Result<Vec<PathBuf>, Stage0InventoryGenerationError> {
    validate_absolute_existing_path(root)?;
    let mut paths = Vec::new();
    let mut pending = vec![(root.to_path_buf(), 0_u32)];
    let mut scheduled_entries = 1_u32;
    while let Some((path, depth)) = pending.pop() {
        if depth > MAX_SEED_WALK_DEPTH {
            return Err(Stage0InventoryGenerationError::WalkDepthExceeded {
                path,
                limit: MAX_SEED_WALK_DEPTH,
            });
        }
        let metadata = fs::metadata(&path).map_err(|err| io_error(&path, err))?;
        if metadata.is_file() {
            if is_executable_metadata(&metadata) {
                push_bounded_seed_path(&mut paths, path)?;
                continue;
            }
            return Err(Stage0InventoryGenerationError::NotExecutable { path });
        }
        if metadata.is_dir() {
            let entries = fs::read_dir(&path).map_err(|err| io_error(&path, err))?;
            for entry in entries {
                let entry = entry.map_err(|err| io_error(&path, err))?;
                scheduled_entries = scheduled_entries.checked_add(1).ok_or_else(|| {
                    Stage0InventoryGenerationError::WalkEntryLimitExceeded {
                        path: path.clone(),
                        limit: MAX_SEED_WALK_ENTRIES,
                    }
                })?;
                if scheduled_entries > MAX_SEED_WALK_ENTRIES {
                    return Err(Stage0InventoryGenerationError::WalkEntryLimitExceeded {
                        path,
                        limit: MAX_SEED_WALK_ENTRIES,
                    });
                }
                pending.push((entry.path(), depth.saturating_add(1)));
            }
        }
    }
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: root.to_path_buf(),
        });
    }
    assert!(root.is_absolute());
    assert!(scheduled_entries <= MAX_SEED_WALK_ENTRIES);
    Ok(paths)
}

fn push_bounded_seed_path(paths: &mut Vec<PathBuf>, path: PathBuf) -> Result<(), Stage0InventoryGenerationError> {
    let next_len = paths.len().saturating_add(1);
    let actual = match u32::try_from(next_len) {
        Ok(value) => value,
        Err(_) => MAX_SEED_EXECUTABLES.saturating_add(1),
    };
    if actual > MAX_SEED_EXECUTABLES {
        return Err(Stage0InventoryGenerationError::TooManySeedExecutables {
            limit: MAX_SEED_EXECUTABLES,
            actual,
        });
    }
    paths.push(path);
    Ok(())
}

#[cfg(unix)]
fn is_executable_metadata(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    const EXECUTABLE_BITS: u32 = 0o111;
    metadata.permissions().mode() & EXECUTABLE_BITS != 0
}

#[cfg(not(unix))]
fn is_executable_metadata(_metadata: &fs::Metadata) -> bool {
    false
}

/// Hash the bytes observed through one open file, rejecting a size change during
/// the read. The digest is not signature, provenance, or stable-path authority.
pub fn blake3_file_hex(path: &Path) -> Result<String, Stage0InventoryGenerationError> {
    let mut file = fs::File::open(path).map_err(|err| io_error(path, err))?;
    let expected_bytes = file.metadata().map_err(|err| io_error(path, err))?.len();
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    let buffer_bytes =
        u64::try_from(buffer.len()).map_err(|_| changed_file_error(path, "hash buffer length overflow"))?;
    let rounded_bytes = expected_bytes
        .checked_add(buffer_bytes.saturating_sub(1))
        .ok_or_else(|| changed_file_error(path, "file length overflow"))?;
    let data_reads = rounded_bytes
        .checked_div(buffer_bytes)
        .ok_or_else(|| changed_file_error(path, "hash buffer must not be empty"))?;
    let max_reads = data_reads.checked_add(1).ok_or_else(|| changed_file_error(path, "file read bound overflow"))?;
    let mut total_bytes = 0_u64;
    for _ in 0..max_reads {
        let read = file.read(&mut buffer).map_err(|err| io_error(path, err))?;
        if read == 0 {
            break;
        }
        let read_bytes = u64::try_from(read).map_err(|_| changed_file_error(path, "read length overflow"))?;
        total_bytes = total_bytes
            .checked_add(read_bytes)
            .ok_or_else(|| changed_file_error(path, "total read length overflow"))?;
        if total_bytes > expected_bytes {
            return Err(changed_file_error(path, "file grew while its BLAKE3 digest was computed"));
        }
        hasher.update(&buffer[..read]);
    }
    if total_bytes != expected_bytes {
        return Err(changed_file_error(path, "file changed while its BLAKE3 digest was computed"));
    }
    assert!(!buffer.is_empty());
    assert!(max_reads > 0);
    Ok(hasher.finalize().to_hex().to_string())
}

fn changed_file_error(path: &Path, message: &'static str) -> Stage0InventoryGenerationError {
    io_error(path, io::Error::new(io::ErrorKind::InvalidData, message))
}

fn seed_id_for_path(prefix: &str, root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let normalized = normalized_path_id(relative);
    let safe = normalized
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if safe.is_empty() {
        return prefix.to_string();
    }
    format!("{prefix}-{safe}")
}

struct NickelStringField<'field> {
    out: &'field mut String,
    name: &'field str,
    value: &'field str,
}

fn push_executable_entry_nickel(
    out: &mut String,
    entry: &ExecutableSeedEntry,
) -> Result<(), Stage0InventoryGenerationError> {
    let initial_bytes = out.len();
    out.push_str("    {\n");
    push_nickel_field(NickelStringField {
        out,
        name: "schema_version",
        value: &entry.schema_version,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "id",
        value: &entry.id,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "role",
        value: &entry.role,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "phase",
        value: &entry.phase,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "executable_path",
        value: path_to_str(&entry.executable_path)?,
    });
    push_digest_nickel(out, &entry.digest);
    push_version_evidence_nickel(out, entry.version_evidence.as_ref());
    push_nickel_field(NickelStringField {
        out,
        name: "provenance_category",
        value: &entry.provenance_category,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "provenance",
        value: &entry.provenance,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "allowed_reason",
        value: &entry.allowed_reason,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "owner",
        value: &entry.owner,
    });
    push_nickel_bool_field(out, "required", entry.required);
    out.push_str("    },\n");
    assert!(out.len() > initial_bytes);
    assert!(out.ends_with("    },\n"));
    Ok(())
}

fn push_source_entry_nickel(out: &mut String, entry: &SourceSeedEntry) {
    let initial_bytes = out.len();
    out.push_str("    {\n");
    for (name, value) in [
        ("schema_version", entry.schema_version.as_str()),
        ("id", entry.id.as_str()),
        ("role", entry.role.as_str()),
        ("phase", entry.phase.as_str()),
    ] {
        push_nickel_field(NickelStringField { out, name, value });
    }
    push_nickel_string_array(out, "urls", &entry.urls);
    push_nickel_string_array(out, "extraction_rules", &entry.extraction_rules);
    push_digest_nickel(out, &entry.digest);
    for (name, value) in [
        ("provenance_category", entry.provenance_category.as_str()),
        ("provenance", entry.provenance.as_str()),
        ("allowed_reason", entry.allowed_reason.as_str()),
        ("owner", entry.owner.as_str()),
    ] {
        push_nickel_field(NickelStringField { out, name, value });
    }
    push_nickel_bool_field(out, "required", entry.required);
    out.push_str("    },\n");
    assert!(out.len() > initial_bytes);
    assert!(out.ends_with("    },\n"));
}

fn push_digest_nickel(out: &mut String, digest: &DigestSpec) {
    let initial_bytes = out.len();
    out.push_str("      digest = {\n");
    push_nickel_field(NickelStringField {
        out,
        name: "algorithm",
        value: &digest.algorithm,
    });
    push_nickel_field(NickelStringField {
        out,
        name: "hex",
        value: &digest.hex,
    });
    if let Some(reason) = &digest.interoperability_reason {
        push_nickel_field(NickelStringField {
            out,
            name: "interoperability_reason",
            value: reason,
        });
    } else {
        out.push_str("        interoperability_reason = null,\n");
    }
    out.push_str("      },\n");
    assert!(out.len() > initial_bytes);
    assert!(out.ends_with("      },\n"));
}

fn push_version_evidence_nickel(out: &mut String, evidence: Option<&BoundedVersionEvidence>) {
    match evidence {
        Some(evidence) => {
            out.push_str("      version_evidence = {\n");
            push_nickel_string_array(out, "command", &evidence.command);
            push_nickel_u32_field(out, "byte_limit", evidence.byte_limit);
            out.push_str("        output_digest = {\n");
            push_nickel_field(NickelStringField {
                out,
                name: "algorithm",
                value: &evidence.output_digest.algorithm,
            });
            push_nickel_field(NickelStringField {
                out,
                name: "hex",
                value: &evidence.output_digest.hex,
            });
            if let Some(reason) = &evidence.output_digest.interoperability_reason {
                push_nickel_field(NickelStringField {
                    out,
                    name: "interoperability_reason",
                    value: reason,
                });
            } else {
                out.push_str("          interoperability_reason = null,\n");
            }
            out.push_str("        },\n");
            push_nickel_field(NickelStringField {
                out,
                name: "output_sample",
                value: &evidence.output_sample,
            });
            push_nickel_i32_field(out, "exit_code", evidence.exit_code);
            out.push_str("      },\n");
        }
        None => out.push_str("      version_evidence = null,\n"),
    }
}

fn push_nickel_field(field: NickelStringField<'_>) {
    field.out.push_str("      ");
    field.out.push_str(field.name);
    field.out.push_str(" = ");
    field.out.push_str(&nickel_string(field.value));
    field.out.push_str(",\n");
}

fn push_nickel_bool_field(out: &mut String, name: &str, value: bool) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(if value { "true" } else { "false" });
    out.push_str(",\n");
}

fn push_nickel_u32_field(out: &mut String, name: &str, value: u32) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(&value.to_string());
    out.push_str(",\n");
}

fn push_nickel_i32_field(out: &mut String, name: &str, value: i32) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(&value.to_string());
    out.push_str(",\n");
}

fn push_nickel_string_array(out: &mut String, name: &str, values: &[String]) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&nickel_string(value));
    }
    out.push_str("],\n");
}

fn nickel_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            value => out.push(value),
        }
    }
    out.push('"');
    out
}

fn path_to_str(path: &Path) -> Result<&str, Stage0InventoryGenerationError> {
    path.to_str().ok_or_else(|| Stage0InventoryGenerationError::NonUtf8Path {
        path: path.to_path_buf(),
    })
}

fn io_error(path: &Path, err: io::Error) -> Stage0InventoryGenerationError {
    Stage0InventoryGenerationError::Io {
        path: path.to_path_buf(),
        message: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    fn digest(hex: &str) -> DigestSpec {
        DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            hex: hex.to_string(),
            interoperability_reason: None,
        }
    }

    fn version_evidence_for_test(id: &str) -> BoundedVersionEvidence {
        bounded_version_evidence(vec![id.to_string(), "--version".to_string()], format!("{id} test-version\n"), 0)
    }

    fn executable(id: &str, role: &str, path: &str, hex: &str) -> ExecutableSeedEntry {
        ExecutableSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: id.to_string(),
            role: role.to_string(),
            phase: PHASE_PROTECTED.to_string(),
            executable_path: PathBuf::from(path),
            digest: digest(hex),
            version_evidence: Some(version_evidence_for_test(id)),
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

    fn make_executable(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    fn env_with_seed_paths(
        sandbox_entry: &Path,
        sandbox_shell: &Path,
        toolchain_root: &Path,
        build_tools: &[PathBuf],
    ) -> BTreeMap<OsString, OsString> {
        let mut env = BTreeMap::new();
        env.insert(OsString::from(ENV_STAGE0_SEED_SANDBOX_ENTRY), sandbox_entry.as_os_str().to_os_string());
        env.insert(OsString::from(ENV_STAGE0_SEED_SANDBOX_SHELL), sandbox_shell.as_os_str().to_os_string());
        env.insert(OsString::from(ENV_STAGE0_SEED_TOOLCHAIN_ROOT), toolchain_root.as_os_str().to_os_string());
        env.insert(OsString::from(ENV_STAGE0_SEED_BUILD_TOOLS), std::env::join_paths(build_tools).unwrap());
        env
    }

    fn inventory_with_current_exe(current_exe: &Path, digest_hex: String) -> Stage0Inventory {
        let mut inv = inventory();
        inv.executable_entries.push(ExecutableSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: "stage0-crunch".to_string(),
            role: ROLE_STAGE0_CRUNCH.to_string(),
            phase: PHASE_PROTECTED.to_string(),
            executable_path: current_exe.to_path_buf(),
            digest: DigestSpec {
                algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
                hex: digest_hex,
                interoperability_reason: None,
            },
            version_evidence: Some(version_evidence_for_test("stage0-crunch")),
            provenance_category: PROVENANCE_TEST_FIXTURE.to_string(),
            provenance: "current test binary".to_string(),
            allowed_reason: "stage0 crunch may launch itself".to_string(),
            owner: "bootstrap".to_string(),
            required: false,
        });
        inv
    }

    #[test]
    fn generated_inventory_ignores_fake_path_host_helper_families() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap-seed");
        let sandbox_shell = temp.path().join("seed/bin/busybox-seed");
        let toolchain_cc = temp.path().join("toolchain/bin/cc");
        let build_make = temp.path().join("build-tools/make");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");

        let fake_path_dir = temp.path().join("fake-path");
        for helper in [
            "git",
            "tar",
            "cp",
            "sh",
            "cargo",
            "bwrap",
            "nix",
            "nix-build",
            "nix-store",
            "nix-shell",
        ] {
            make_executable(&fake_path_dir.join(helper), helper.as_bytes());
        }
        let mut env = env_with_seed_paths(&sandbox_entry, &sandbox_shell, &temp.path().join("toolchain"), &[temp
            .path()
            .join("build-tools")]);
        env.insert(OsString::from("PATH"), fake_path_dir.as_os_str().to_os_string());
        let config = stage0_seed_config_from_env_map(&env).unwrap();
        let inventory = build_stage0_inventory_from_seed_config(&config).unwrap();
        let policy = ProtectedExecPolicy::from_inventory(inventory).unwrap();

        for helper in [
            "git",
            "tar",
            "cp",
            "sh",
            "cargo",
            "bwrap",
            "nix",
            "nix-build",
            "nix-store",
            "nix-shell",
        ] {
            let path = fake_path_dir.join(helper);
            let digest_hex = blake3_file_hex(&path).unwrap();
            let err = policy
                .decide_exec(&ExecRequest {
                    path: path.clone(),
                    digest_hex,
                })
                .unwrap_err();
            assert_eq!(err, ProtectedExecError::UndeclaredExecutable { path });
        }
    }

    #[test]
    fn generated_inventory_missing_required_seed_path_fails_closed() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_shell = temp.path().join("seed/bin/busybox-seed");
        let toolchain_cc = temp.path().join("toolchain/bin/cc");
        let build_make = temp.path().join("build-tools/make");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");
        let config = Stage0InventorySeedConfig {
            sandbox_entry: temp.path().join("missing/bwrap-seed"),
            sandbox_shell,
            toolchain_root: temp.path().join("toolchain"),
            build_tool_inputs: vec![temp.path().join("build-tools")],
        };

        let err = build_stage0_inventory_from_seed_config(&config).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::MissingInputPath {
            path: temp.path().join("missing/bwrap-seed")
        });
    }

    #[test]
    fn protected_launch_plan_authorizes_content_bound_executable() {
        let current_exe = std::env::current_exe().unwrap();
        let digest_hex = blake3_file_hex(&current_exe).unwrap();
        let inv = inventory_with_current_exe(&current_exe, digest_hex.clone());
        let policy = ProtectedExecPolicy::from_inventory(inv).unwrap();

        let plan = plan_protected_launch(&policy, &current_exe).unwrap();
        let event = plan.audit_event();
        assert_eq!(event.executable_path, current_exe);
        assert_eq!(event.digest_hex, digest_hex);
        assert_eq!(event.inventory_entry_id, "stage0-crunch");
        assert_eq!(event.policy_decision, "allowed");
        assert_eq!(event.phase, PHASE_PROTECTED);
    }

    #[test]
    fn protected_launch_plan_rejects_changed_executable_before_launch() {
        let current_exe = std::env::current_exe().unwrap();
        let actual_digest = blake3_file_hex(&current_exe).unwrap();
        let expected_digest = if actual_digest == DIGEST_A { DIGEST_B } else { DIGEST_A };
        let inv = inventory_with_current_exe(&current_exe, expected_digest.to_string());
        let policy = ProtectedExecPolicy::from_inventory(inv).unwrap();

        let err = plan_protected_launch(&policy, &current_exe).unwrap_err();
        assert!(matches!(err, Stage0InventoryGenerationError::Policy(
            ProtectedExecError::DigestMismatch { actual, expected, .. }
        ) if actual == actual_digest && expected == expected_digest));
    }

    #[test]
    fn seed_config_requires_explicit_environment_inputs() {
        let env = BTreeMap::new();
        let err = stage0_seed_config_from_env_map(&env).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::MissingEnv {
            var: ENV_STAGE0_SEED_SANDBOX_ENTRY
        });

        let mut partial = BTreeMap::new();
        partial.insert(OsString::from(ENV_STAGE0_SEED_SANDBOX_ENTRY), OsString::from("/seed/bin/bwrap"));
        let err = stage0_seed_config_from_env_map(&partial).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::MissingEnv {
            var: ENV_STAGE0_SEED_SANDBOX_SHELL
        });
    }

    #[test]
    fn generated_inventory_expands_explicit_seed_tools_and_ignores_path() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        let toolchain_cc = temp.path().join("toolchain/bin/x86_64-linux-musl-gcc");
        let build_make = temp.path().join("build-tools/make");
        let fake_path_tool = temp.path().join("fake-path/git");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");
        make_executable(&fake_path_tool, b"must-not-appear");

        let mut env = env_with_seed_paths(&sandbox_entry, &sandbox_shell, &temp.path().join("toolchain"), &[temp
            .path()
            .join("build-tools")]);
        env.insert(OsString::from("PATH"), temp.path().join("fake-path").as_os_str().to_os_string());
        env.insert(OsString::from("NIX_STORE"), OsString::from("/nix/store"));

        let config = stage0_seed_config_from_env_map(&env).unwrap();
        let inventory = build_stage0_inventory_from_seed_config(&config).unwrap();
        let paths: BTreeSet<PathBuf> =
            inventory.executable_entries.iter().map(|entry| entry.executable_path.clone()).collect();

        assert!(paths.contains(&sandbox_entry));
        assert!(paths.contains(&sandbox_shell));
        assert!(paths.contains(&toolchain_cc));
        assert!(paths.contains(&build_make));
        assert!(!paths.contains(&fake_path_tool));
        assert!(inventory.executable_entries.iter().any(|entry| entry.role == ROLE_BOOTSTRAP_TOOLCHAIN_TOOL));
        assert!(inventory.executable_entries.iter().any(|entry| entry.role == ROLE_BOOTSTRAP_BUILD_TOOL));
        assert_eq!(
            inventory
                .executable_entries
                .iter()
                .find(|entry| entry.executable_path == sandbox_entry)
                .unwrap()
                .digest
                .hex,
            blake3::hash(b"sandbox-entry").to_hex().to_string()
        );
    }

    #[test]
    fn seed_closure_risk_reports_static_dynamic_and_script_shapes() {
        let static_elf = [ELF_MAGIC, b"static payload"].concat();
        let dynamic_elf = [ELF_MAGIC, b"/lib64/ld-linux-x86-64.so.2"].concat();
        let script = b"#!/bin/sh\nexit 0\n";
        let unknown = b"plain executable bytes";

        assert_eq!(classify_seed_closure_risk_bytes(&static_elf), SeedClosureRisk::StaticElfCandidate);
        assert_eq!(classify_seed_closure_risk_bytes(&dynamic_elf), SeedClosureRisk::DynamicElfLikely);
        assert_eq!(classify_seed_closure_risk_bytes(script), SeedClosureRisk::ScriptInterpreter);
        assert_eq!(classify_seed_closure_risk_bytes(unknown), SeedClosureRisk::UnknownExecutableFormat);
    }

    #[test]
    fn seed_closure_risk_report_names_inventory_entries() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        make_executable(&sandbox_entry, &[ELF_MAGIC, b"static bwrap"].concat());
        make_executable(&sandbox_shell, b"#!/bin/sh\nexit 0\n");
        let inventory = Stage0Inventory {
            executable_entries: vec![
                executable("sandbox-entry", ROLE_SANDBOX_ENTRY, sandbox_entry.to_str().unwrap(), DIGEST_A),
                executable("sandbox-shell", ROLE_SANDBOX_SHELL, sandbox_shell.to_str().unwrap(), DIGEST_B),
            ],
            source_entries: Vec::new(),
        };

        let report = seed_closure_risk_report(&inventory);

        assert_eq!(report.len(), 2);
        assert_eq!(report[0].entry_id, "sandbox-entry");
        assert_eq!(report[0].risk, SeedClosureRisk::StaticElfCandidate);
        assert_eq!(report[1].entry_id, "sandbox-shell");
        assert_eq!(report[1].risk, SeedClosureRisk::ScriptInterpreter);
    }

    #[test]
    fn generated_inventory_file_is_nickel_and_policy_valid() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        let toolchain_cc = temp.path().join("toolchain/bin/cc");
        let build_make = temp.path().join("build-tools/make");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");
        let config = Stage0InventorySeedConfig {
            sandbox_entry: sandbox_entry.clone(),
            sandbox_shell,
            toolchain_root: temp.path().join("toolchain"),
            build_tool_inputs: vec![temp.path().join("build-tools")],
        };
        let output_path = temp.path().join("target/host-tool-free-stage0/stage0-inventory.ncl");

        let inventory = write_generated_stage0_inventory(&config, &output_path).unwrap();
        let text = fs::read_to_string(&output_path).unwrap();
        let policy = ProtectedExecPolicy::from_inventory(inventory).unwrap();

        assert!(text.contains("executable_entries"));
        assert!(text.contains("sandbox-entry"));
        assert!(text.contains("bootstrap-toolchain-tool"));
        assert!(text.contains("bootstrap-build-tool"));
        assert!(
            policy
                .decide_exec(&ExecRequest {
                    path: sandbox_entry,
                    digest_hex: blake3::hash(b"sandbox-entry").to_hex().to_string(),
                })
                .unwrap()
                .allowed
        );
    }

    #[test]
    fn generated_inventory_rejects_empty_or_non_executable_tool_roots() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        fs::create_dir_all(temp.path().join("empty-toolchain")).unwrap();
        let config = Stage0InventorySeedConfig {
            sandbox_entry,
            sandbox_shell,
            toolchain_root: temp.path().join("empty-toolchain"),
            build_tool_inputs: vec![temp.path().join("empty-toolchain")],
        };

        let err = build_stage0_inventory_from_seed_config(&config).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::NotExecutable {
            path: temp.path().join("empty-toolchain")
        });
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
    fn inventory_requires_bounded_version_evidence() {
        let mut missing = inventory();
        missing.executable_entries[0].version_evidence = None;
        let err = ProtectedExecPolicy::from_inventory(missing).unwrap_err();
        assert!(matches!(err, ProtectedExecError::MissingVersionEvidence { .. }));

        let mut invalid = inventory();
        invalid.executable_entries[0].version_evidence = Some(BoundedVersionEvidence {
            command: Vec::new(),
            byte_limit: MAX_VERSION_EVIDENCE_BYTES,
            output_digest: digest(DIGEST_A),
            output_sample: "bwrap 1.0\n".to_string(),
            exit_code: 0,
        });
        let err = ProtectedExecPolicy::from_inventory(invalid).unwrap_err();
        assert!(matches!(err, ProtectedExecError::InvalidVersionEvidence { .. }));
    }

    #[test]
    fn inventory_digest_binds_version_evidence_and_order_is_stable() {
        let mut left = inventory();
        let mut right = inventory();
        right.executable_entries.reverse();
        assert_eq!(stage0_inventory_digest_blake3(&left), stage0_inventory_digest_blake3(&right));

        left.executable_entries[0].version_evidence = Some(version_evidence_for_test("changed"));
        assert_ne!(stage0_inventory_digest_blake3(&left), stage0_inventory_digest_blake3(&right));
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
    fn stagex_seed_policy_authorizes_only_the_bound_seed() {
        let seed_path = PathBuf::from("/stagex/seed/hex0-seed");
        let promotion_stage_ids = vec!["hex0-reproduction".to_string()];
        let policy =
            ProtectedExecPolicy::from_stagex_plan(seed_path.clone(), DIGEST_A.to_string(), &promotion_stage_ids, &[])
                .unwrap();
        let decision = policy
            .decide_exec(&ExecRequest {
                path: seed_path,
                digest_hex: DIGEST_A.to_string(),
            })
            .unwrap();
        let undeclared = policy.decide_exec(&ExecRequest {
            path: PathBuf::from("/bin/sh"),
            digest_hex: DIGEST_A.to_string(),
        });
        assert!(decision.allowed);
        assert_eq!(decision.entry_id.as_deref(), Some("stagex:seed:hex0"));
        assert!(matches!(undeclared, Err(ProtectedExecError::UndeclaredExecutable { .. })));
    }

    #[test]
    fn stagex_plan_preauthorizes_exact_future_output_digest() {
        let output_path = PathBuf::from("/stagex/out/hex1");
        let source_stage_ids = vec!["stage0-hex1".to_string()];
        let planned = vec![PlannedExecutable {
            authorization_id: "exec:stage0-hex1".to_string(),
            source_stage_id: "stage0-hex1".to_string(),
            path: output_path.clone(),
            digest_hex: DIGEST_B.to_string(),
        }];
        let policy = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            DIGEST_A.to_string(),
            &source_stage_ids,
            &planned,
        )
        .unwrap();

        let allowed = policy.decide_exec(&ExecRequest {
            path: output_path.clone(),
            digest_hex: DIGEST_B.to_string(),
        });
        let substituted = policy.decide_exec(&ExecRequest {
            path: output_path,
            digest_hex: DIGEST_A.to_string(),
        });

        assert!(allowed.unwrap().allowed);
        assert!(matches!(substituted, Err(ProtectedExecError::DigestMismatch { .. })));
    }

    #[test]
    fn stagex_plan_allows_bounded_digest_variants_at_one_path() {
        let output_path = PathBuf::from("/stagex/out/conftest");
        let source_stage_ids = vec!["configure-probes".to_string()];
        let planned = vec![
            PlannedExecutable {
                authorization_id: "exec:configure-probe:a".to_string(),
                source_stage_id: "configure-probes".to_string(),
                path: output_path.clone(),
                digest_hex: DIGEST_B.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:configure-probe:b".to_string(),
                source_stage_id: "configure-probes".to_string(),
                path: output_path.clone(),
                digest_hex: DIGEST_C.to_string(),
            },
        ];
        let policy = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            DIGEST_A.to_string(),
            &source_stage_ids,
            &planned,
        )
        .unwrap();
        let first = policy
            .decide_exec(&ExecRequest {
                path: output_path.clone(),
                digest_hex: DIGEST_B.to_string(),
            })
            .unwrap();
        let second = policy
            .decide_exec(&ExecRequest {
                path: output_path,
                digest_hex: DIGEST_C.to_string(),
            })
            .unwrap();
        assert_eq!(first.entry_id.as_deref(), Some("planned:configure-probes:exec:configure-probe:a"));
        assert_eq!(second.entry_id.as_deref(), Some("planned:configure-probes:exec:configure-probe:b"));
    }

    #[test]
    fn inventory_rejects_digest_variants_at_one_path() {
        let mut inv = inventory();
        let mut variant = inv.executable_entries[0].clone();
        variant.id = "duplicate-path-variant".to_string();
        variant.digest.hex = DIGEST_C.to_string();
        inv.executable_entries.push(variant);
        let error = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert!(matches!(error, ProtectedExecError::DuplicateExecutablePath { .. }));
        assert!(!error.to_string().contains("allowed"));
    }

    #[test]
    fn stagex_plan_rejects_duplicate_and_excessive_path_variants() {
        let output_path = PathBuf::from("/stagex/out/conftest");
        let source_stage_ids = vec!["configure-probes".to_string()];
        let duplicate = vec![
            PlannedExecutable {
                authorization_id: "exec:configure-probe:a".to_string(),
                source_stage_id: "configure-probes".to_string(),
                path: output_path.clone(),
                digest_hex: DIGEST_B.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:configure-probe:duplicate".to_string(),
                source_stage_id: "configure-probes".to_string(),
                path: output_path.clone(),
                digest_hex: DIGEST_B.to_string(),
            },
        ];
        let duplicate_error = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            DIGEST_A.to_string(),
            &source_stage_ids,
            &duplicate,
        )
        .unwrap_err();
        let excessive_count = usize::try_from(MAX_EXECUTABLE_DIGEST_VARIANTS_PER_PATH).unwrap().saturating_add(1);
        let digest_width = usize::try_from(BLAKE3_HEX_LEN).unwrap();
        let excessive = (0..excessive_count)
            .map(|index| PlannedExecutable {
                authorization_id: format!("exec:configure-probe:{index}"),
                source_stage_id: "configure-probes".to_string(),
                path: output_path.clone(),
                digest_hex: format!("{index:0width$x}", width = digest_width),
            })
            .collect::<Vec<_>>();
        let excessive_error = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            DIGEST_A.to_string(),
            &source_stage_ids,
            &excessive,
        )
        .unwrap_err();
        assert!(matches!(duplicate_error, ProtectedExecError::DuplicateExecutablePath { .. }));
        assert!(matches!(excessive_error, ProtectedExecError::InventoryCollectionLimitExceeded { .. }));
    }

    #[test]
    fn stagex_plan_rejects_future_output_from_undeclared_stage() {
        let source_stage_ids = vec!["stage0-hex1".to_string()];
        let planned = vec![PlannedExecutable {
            authorization_id: "exec:ambient".to_string(),
            source_stage_id: "host-tools".to_string(),
            path: PathBuf::from("/stagex/out/ambient"),
            digest_hex: DIGEST_B.to_string(),
        }];

        let error = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            DIGEST_A.to_string(),
            &source_stage_ids,
            &planned,
        )
        .unwrap_err();

        assert!(matches!(error, ProtectedExecError::UndeclaredPromotionSource { .. }));
        assert!(!error.to_string().contains("digest mismatch"));
    }

    #[test]
    fn stagex_seed_policy_rejects_undeclared_output_promotion() {
        let promotion_stage_ids = vec!["hex0-reproduction".to_string()];
        let mut policy = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            DIGEST_A.to_string(),
            &promotion_stage_ids,
            &[],
        )
        .unwrap();
        let result = policy.promote_verified_output("host-tools", &["format=raw".to_string()], &[PromotedExecutable {
            path: PathBuf::from("/stagex/out/host-tool"),
            digest_hex: DIGEST_B.to_string(),
        }]);
        assert!(matches!(result, Err(ProtectedExecError::UndeclaredPromotionSource { .. })));
    }

    #[test]
    fn stagex_seed_policy_rejects_relative_path_and_bad_digest() {
        let promotion_stage_ids = vec!["hex0-reproduction".to_string()];
        let relative = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("hex0-seed"),
            DIGEST_A.to_string(),
            &promotion_stage_ids,
            &[],
        );
        let malformed = ProtectedExecPolicy::from_stagex_plan(
            PathBuf::from("/stagex/seed/hex0-seed"),
            "bad".to_string(),
            &promotion_stage_ids,
            &[],
        );
        assert!(matches!(relative, Err(ProtectedExecError::RelativeExecutablePath { .. })));
        assert!(matches!(malformed, Err(ProtectedExecError::InvalidBlake3Digest { .. })));
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
    fn inventory_rejects_malformed_source_entries() {
        let mut inv = inventory();
        inv.source_entries[0].urls = vec![String::new()];
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert_eq!(err, ProtectedExecError::EmptyField {
            entry_id: "musl".to_string(),
            field: "urls"
        });

        let mut missing_rule = inventory();
        missing_rule.source_entries[0].extraction_rules = Vec::new();
        let err = ProtectedExecPolicy::from_inventory(missing_rule).unwrap_err();
        assert_eq!(err, ProtectedExecError::EmptyField {
            entry_id: "musl".to_string(),
            field: "extraction_rules"
        });
    }

    #[test]
    fn inventory_rejects_duplicate_source_urls_across_entries() {
        let mut inv = inventory();
        let mut second = inv.source_entries[0].clone();
        second.id = "musl-copy".to_string();
        inv.source_entries.push(second);

        assert_eq!(ProtectedExecPolicy::from_inventory(inv).unwrap_err(), ProtectedExecError::DuplicateSourceUrl {
            url: "https://example.invalid/musl.tar.xz".to_string()
        });
    }

    #[test]
    fn inventory_rejects_source_entry_count_above_bound() {
        let mut inv = inventory();
        let source_entry = inv.source_entries[0].clone();
        let excessive_entry_count = usize::try_from(MAX_SOURCE_ENTRIES).unwrap().saturating_add(1);
        inv.source_entries = vec![source_entry; excessive_entry_count];

        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();

        assert_eq!(err, ProtectedExecError::InventoryCollectionLimitExceeded {
            collection: "source entries",
            limit: MAX_SOURCE_ENTRIES,
        });
    }

    #[test]
    fn inventory_rejects_malformed_or_ambiguous_extraction_rules() {
        for (rule, expected_reason) in [
            ("strip-components", "empty value"),
            ("strip-components=1=2", "multiple separators"),
            ("unknown=value", "unsupported key unknown"),
            ("strip-components=abc", "unsigned integer"),
            ("root=../escape", "parent traversal"),
        ] {
            let mut inv = inventory();
            inv.source_entries[0].extraction_rules = vec![rule.to_string()];
            let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
            match err {
                ProtectedExecError::InvalidExtractionRule { reason, .. } => {
                    assert!(reason.contains(expected_reason), "rule={rule} reason={reason}");
                }
                other => panic!("rule={rule} expected InvalidExtractionRule, got {other:?}"),
            }
        }

        let mut duplicate = inventory();
        duplicate.source_entries[0].extraction_rules = vec!["format=tar".to_string(), "format=tar.xz".to_string()];
        let err = ProtectedExecPolicy::from_inventory(duplicate).unwrap_err();
        assert!(
            matches!(err, ProtectedExecError::InvalidExtractionRule { reason, .. } if reason.contains("duplicate key"))
        );
    }

    #[test]
    fn normalized_path_id_is_deterministic() {
        let id = normalized_path_id(Path::new("/seed/bin/bwrap"));
        assert_eq!(id, "/seed/bin/bwrap");
        assert_ne!(id, "/seed/bin/busybox");
    }

    #[test]
    fn promote_verified_output_extends_policy_and_records_audit() {
        let mut policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let initial_size = policy.executables_by_path.len();

        let promoted = vec![
            PromotedExecutable {
                path: PathBuf::from("/build/out/bin/make"),
                digest_hex: "a".repeat(64),
            },
            PromotedExecutable {
                path: PathBuf::from("/build/out/bin/gcc"),
                digest_hex: "b".repeat(64),
            },
        ];
        let record = policy.promote_verified_output("src-gnu-make", &["format=tar".to_string()], &promoted).unwrap();

        assert_eq!(record.source_entry_id, "src-gnu-make");
        assert_eq!(record.promoted_executables.len(), 2);
        assert_eq!(record.extraction_rules, vec!["format=tar"]);
        assert_eq!(record.promoted_at_policy_size, (initial_size + 2) as u32);

        let make_request = ExecRequest {
            path: PathBuf::from("/build/out/bin/make"),
            digest_hex: "a".repeat(64),
        };
        let decision = policy.decide_exec(&make_request).unwrap();
        assert!(decision.allowed);
        assert!(decision.entry_id.unwrap().contains("promoted:src-gnu-make"));
    }

    #[test]
    fn promote_verified_output_rejects_empty_set() {
        let mut policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let err = policy.promote_verified_output("src-empty", &[], &[]).unwrap_err();
        match err {
            ProtectedExecError::PromotionEmptySet { source_entry_id } => {
                assert_eq!(source_entry_id, "src-empty");
            }
            other => panic!("expected PromotionEmptySet, got {other:?}"),
        }
    }

    #[test]
    fn promote_verified_output_rejects_duplicate_path() {
        let mut policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let existing_path = policy.executables_by_path.keys().next().cloned().unwrap();
        let promoted = vec![PromotedExecutable {
            path: existing_path.clone(),
            digest_hex: "c".repeat(64),
        }];
        let err = policy.promote_verified_output("src-dup", &[], &promoted).unwrap_err();
        match err {
            ProtectedExecError::PromotionDuplicatePath { path, source_entry_id } => {
                assert_eq!(path, existing_path);
                assert_eq!(source_entry_id, "src-dup");
            }
            other => panic!("expected PromotionDuplicatePath, got {other:?}"),
        }
    }

    #[test]
    fn unpromoted_executable_is_denied_after_promotion() {
        let mut policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let promoted = vec![PromotedExecutable {
            path: PathBuf::from("/build/out/bin/promoted-tool"),
            digest_hex: "d".repeat(64),
        }];
        policy.promote_verified_output("src-x", &[], &promoted).unwrap();

        let unpromoted_request = ExecRequest {
            path: PathBuf::from("/build/out/bin/not-promoted"),
            digest_hex: "e".repeat(64),
        };
        let err = policy.decide_exec(&unpromoted_request).unwrap_err();
        assert!(matches!(err, ProtectedExecError::UndeclaredExecutable { .. }));
    }
}
