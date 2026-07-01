use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::InputFetchPolicy;
use crate::generate::generate_inputs_ncl;
use crate::lock::LockEntry;
use crate::lock::LockedHash;
use crate::lock::LockedKind;
use crate::lock::LockedPatch;
use crate::lock::LockedPatchSource;
use crate::lock::Lockfile;
use crate::manifest::GitReference;
use crate::manifest::HashAlgo;
use crate::manifest::HashSpec;
use crate::manifest::InputKind;
use crate::manifest::ManifestInput;
use crate::manifest::PatchDef;
use crate::manifest::PatchSource;
use crate::manifest::ProjectManifest;
use crate::version::SchemaVersion;

pub const PIN_IMPORT_PLAN_SCHEMA: &str = "mantle-project-pin-import-plan-v1";
pub const PIN_IMPORT_SUPPORTED_IMPORTER: &str = "nixtamal";
pub const PIN_IMPORT_DEFAULT_PROJECT_FILE: &str = "mantle-project.ncl";
pub const PIN_IMPORT_DEFAULT_LOCK_FILE: &str = "mantle.lock";
pub const PIN_IMPORT_DEFAULT_INPUTS_FILE: &str = ".mantle/inputs.ncl";

const IMPORT_MANIFEST_VERSION: &str = "1.0.0";
const MAX_IMPORT_OUTPUT_PATH_BYTES: usize = 256;
const MAX_IMPORT_NAME_BYTES: usize = 128;
const MAX_IMPORT_SEMANTICS_PER_PIN: usize = 32;
const MAX_IMPORT_EXISTING_FILES: usize = 16;
const HASH_ALGO_SHA256: &str = "sha256";
const HASH_ALGO_SHA512: &str = "sha512";
const HASH_ALGO_BLAKE3: &str = "blake3";
const KIND_FILE: &str = "file";
const KIND_TARBALL: &str = "tarball";
const KIND_GIT: &str = "git";
const PATCH_KIND_LOCAL: &str = "local";
const PATCH_KIND_REMOTE: &str = "remote";
const SEMANTIC_FRESHNESS_LOCKED: &str = "locked";
const SEMANTIC_FRESHNESS_NONE: &str = "none";
const SEMANTIC_FETCH_REFRESH: &str = "refresh";
const SEMANTIC_FETCH_GENERATION: &str = "generation";
const SEMANTIC_TRUST_CONTENT_HASH: &str = "content-hash";
const SEMANTIC_TRUST_HASH_ONLY: &str = "hash-only";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportOptions {
    pub importer: String,
    pub project_file: String,
    pub lock_file: String,
    pub inputs_file: String,
}

impl Default for PinImportOptions {
    fn default() -> Self {
        Self {
            importer: PIN_IMPORT_SUPPORTED_IMPORTER.to_string(),
            project_file: PIN_IMPORT_DEFAULT_PROJECT_FILE.to_string(),
            lock_file: PIN_IMPORT_DEFAULT_LOCK_FILE.to_string(),
            inputs_file: PIN_IMPORT_DEFAULT_INPUTS_FILE.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalPinSet {
    pub importer: String,
    pub source_label: String,
    pub pins: Vec<ExternalPin>,
    pub patches: Vec<ExternalPatch>,
    pub existing_files: Vec<ExternalExistingFile>,
    pub unsupported_semantics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalExistingFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalPin {
    pub name: String,
    pub kind: ExternalPinKind,
    pub hash: ExternalHash,
    pub frozen: bool,
    pub mirrors: Vec<String>,
    pub patches: Vec<String>,
    pub metadata: ExternalPinMetadata,
    pub lock_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExternalPinKind {
    #[serde(rename = "file")]
    File { url: String },
    #[serde(rename = "tarball")]
    Tarball { url: String },
    #[serde(rename = "git")]
    Git {
        repository: String,
        reference: Option<String>,
        rev: Option<String>,
    },
    #[serde(rename = "unsupported")]
    Unsupported { kind: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalHash {
    pub algo: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExternalPinMetadata {
    pub freshness: Option<String>,
    pub fetch_policy: Option<String>,
    pub trust_policy: Option<String>,
    pub composition_semantics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalPatch {
    pub name: String,
    pub source: ExternalPatchSource,
    pub hash: ExternalHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExternalPatchSource {
    #[serde(rename = "local")]
    Local { path: String },
    #[serde(rename = "remote")]
    Remote { url: String },
    #[serde(rename = "unsupported")]
    Unsupported { kind: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportPlan {
    pub schema: String,
    pub importer: String,
    pub source_label: String,
    pub file_operations: Vec<PinImportFileOperation>,
    pub mapped_inputs: Vec<PinImportMappedInput>,
    pub mapped_patches: Vec<PinImportMappedPatch>,
    pub preserved_semantics: Vec<PinImportSemantic>,
    pub rewritten_semantics: Vec<PinImportSemantic>,
    pub blockers: Vec<PinImportBlocker>,
    pub non_claims: Vec<String>,
    pub future_adapters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportFileOperation {
    pub path: String,
    pub action: String,
    pub digest_blake3: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportMappedInput {
    pub name: String,
    pub kind: String,
    pub frozen: bool,
    pub hash_algo: String,
    pub lock_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportMappedPatch {
    pub name: String,
    pub source_kind: String,
    pub hash_algo: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportSemantic {
    pub subject: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinImportBlocker {
    pub class: String,
    pub subject: String,
    pub message: String,
}

impl PinImportPlan {
    pub fn can_apply(&self) -> bool {
        self.blockers.is_empty()
    }
}

struct MappedImport {
    manifest: ProjectManifest,
    lock: Lockfile,
    mapped_inputs: Vec<PinImportMappedInput>,
    mapped_patches: Vec<PinImportMappedPatch>,
    preserved_semantics: Vec<PinImportSemantic>,
    rewritten_semantics: Vec<PinImportSemantic>,
}

// r[impl project_workflows.project_lock_importers]
// r[impl project_workflows.nixtamal_importer]
pub fn build_pin_import_plan(pin_set: ExternalPinSet, options: PinImportOptions) -> PinImportPlan {
    debug_assert!(!PIN_IMPORT_PLAN_SCHEMA.is_empty());
    debug_assert!(!PIN_IMPORT_SUPPORTED_IMPORTER.is_empty());
    let mut blockers = Vec::new();
    validate_importer(&pin_set, &options, &mut blockers);
    validate_options(&options, &mut blockers);
    validate_pin_set_shape(&pin_set, &mut blockers);

    let mapped = map_pin_set(&pin_set, &mut blockers);
    let mut file_operations = Vec::new();
    let mut mapped_inputs = Vec::new();
    let mut mapped_patches = Vec::new();
    let mut preserved_semantics = Vec::new();
    let mut rewritten_semantics = Vec::new();

    if let Some(mapped) = mapped {
        mapped_inputs = mapped.mapped_inputs;
        mapped_patches = mapped.mapped_patches;
        preserved_semantics = mapped.preserved_semantics;
        rewritten_semantics = mapped.rewritten_semantics;
        file_operations = plan_file_operations(mapped.manifest, mapped.lock, &options, &mut blockers);
        validate_file_conflicts(&file_operations, &pin_set.existing_files, &mut blockers);
    }

    PinImportPlan {
        schema: PIN_IMPORT_PLAN_SCHEMA.to_string(),
        importer: options.importer,
        source_label: pin_set.source_label,
        file_operations,
        mapped_inputs,
        mapped_patches,
        preserved_semantics,
        rewritten_semantics,
        blockers,
        non_claims: pin_import_non_claims(),
        future_adapters: vec!["flake".to_string(), "npins".to_string(), "niv".to_string()],
    }
}

fn validate_importer(pin_set: &ExternalPinSet, options: &PinImportOptions, blockers: &mut Vec<PinImportBlocker>) {
    if options.importer != pin_set.importer {
        blockers.push(blocker(
            "importer-mismatch",
            &pin_set.source_label,
            "requested importer does not match normalized pin-set importer",
        ));
    }
    if options.importer != PIN_IMPORT_SUPPORTED_IMPORTER {
        blockers.push(blocker(
            "future-adapter",
            &options.importer,
            "this importer seam is reserved but not implemented; no recursive composition semantics are imported",
        ));
    }
}

fn validate_options(options: &PinImportOptions, blockers: &mut Vec<PinImportBlocker>) {
    validate_relative_output_path(&options.project_file, "invalid-project-file", blockers);
    validate_relative_output_path(&options.lock_file, "invalid-lock-file", blockers);
    validate_relative_output_path(&options.inputs_file, "invalid-inputs-file", blockers);
}

fn validate_relative_output_path(path: &str, class: &str, blockers: &mut Vec<PinImportBlocker>) {
    if path.is_empty() || path.len() > MAX_IMPORT_OUTPUT_PATH_BYTES {
        blockers.push(blocker(class, path, "planned file path must be non-empty and bounded"));
        return;
    }
    if path.starts_with('/') || path.split('/').any(|part| part.is_empty() || part == "..") {
        blockers.push(blocker(class, path, "planned file path must be relative without parent traversal"));
    }
}

fn validate_pin_set_shape(pin_set: &ExternalPinSet, blockers: &mut Vec<PinImportBlocker>) {
    if pin_set.pins.is_empty() {
        blockers.push(blocker("missing-inputs", &pin_set.source_label, "pin import requires at least one input"));
    }
    if pin_set.existing_files.len() > MAX_IMPORT_EXISTING_FILES {
        blockers.push(blocker(
            "too-many-existing-files",
            &pin_set.source_label,
            "existing-file facts exceed importer limit",
        ));
    }
    for semantic in &pin_set.unsupported_semantics {
        blockers.push(blocker("unsupported-import-semantic", &pin_set.source_label, semantic));
    }
}

fn map_pin_set(pin_set: &ExternalPinSet, blockers: &mut Vec<PinImportBlocker>) -> Option<MappedImport> {
    let patch_names = patch_name_set(&pin_set.patches, blockers);
    let mapped_patches = map_patches(&pin_set.patches, blockers);
    let mapped_inputs = map_inputs(&pin_set.pins, &patch_names, blockers);
    if !blockers.is_empty() {
        return None;
    }
    let mapped_patches = mapped_patches.expect("blocker-free patch mapping exists");
    let mapped_inputs = mapped_inputs.expect("blocker-free input mapping exists");
    Some(MappedImport {
        manifest: ProjectManifest {
            version: IMPORT_MANIFEST_VERSION.to_string(),
            inputs: mapped_inputs.manifest_inputs,
            patches: mapped_patches.manifest_patches,
        },
        lock: Lockfile {
            version: SchemaVersion::CURRENT,
            inputs: mapped_inputs.lock_inputs,
            patches: mapped_patches.lock_patches,
        },
        mapped_inputs: mapped_inputs.report_inputs,
        mapped_patches: mapped_patches.report_patches,
        preserved_semantics: mapped_inputs.preserved_semantics,
        rewritten_semantics: mapped_inputs.rewritten_semantics,
    })
}

fn patch_name_set(patches: &[ExternalPatch], blockers: &mut Vec<PinImportBlocker>) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for patch in patches {
        validate_name(&patch.name, "patch", blockers);
        if !names.insert(patch.name.clone()) {
            blockers.push(blocker("duplicate-patch", &patch.name, "patch names must be unique"));
        }
    }
    names
}

struct MappedPatches {
    manifest_patches: Vec<PatchDef>,
    lock_patches: BTreeMap<String, LockedPatch>,
    report_patches: Vec<PinImportMappedPatch>,
}

fn map_patches(patches: &[ExternalPatch], blockers: &mut Vec<PinImportBlocker>) -> Option<MappedPatches> {
    let blocker_count_before = blockers.len();
    let mut manifest_patches = Vec::with_capacity(patches.len());
    let mut lock_patches = BTreeMap::new();
    let mut report_patches = Vec::with_capacity(patches.len());
    for patch in patches {
        if let Some(mapped) = map_patch(patch, blockers) {
            report_patches.push(mapped.report);
            manifest_patches.push(mapped.manifest_patch);
            lock_patches.insert(patch.name.clone(), mapped.lock_patch);
        }
    }
    if blockers.len() != blocker_count_before {
        return None;
    }
    Some(MappedPatches {
        manifest_patches,
        lock_patches,
        report_patches,
    })
}

struct MappedPatch {
    manifest_patch: PatchDef,
    lock_patch: LockedPatch,
    report: PinImportMappedPatch,
}

fn map_patch(patch: &ExternalPatch, blockers: &mut Vec<PinImportBlocker>) -> Option<MappedPatch> {
    let hash = map_locked_hash(&patch.hash, &patch.name, blockers)?;
    let (manifest_source, lock_source, source_kind) = match &patch.source {
        ExternalPatchSource::Local { path } => (
            PatchSource::Local { path: path.clone() },
            LockedPatchSource::Local { path: path.clone() },
            PATCH_KIND_LOCAL.to_string(),
        ),
        ExternalPatchSource::Remote { url } => (
            PatchSource::Remote {
                url: url.clone(),
                hash: HashSpec {
                    algo: hash.algo.clone(),
                    expected: Some(hash.value.clone()),
                },
            },
            LockedPatchSource::Remote { url: url.clone() },
            PATCH_KIND_REMOTE.to_string(),
        ),
        ExternalPatchSource::Unsupported { kind } => {
            blockers.push(blocker(
                "unsupported-patch-source",
                &patch.name,
                &format!("unsupported patch source: {kind}"),
            ));
            return None;
        }
    };
    Some(MappedPatch {
        manifest_patch: PatchDef {
            name: patch.name.clone(),
            source: manifest_source,
        },
        lock_patch: LockedPatch {
            source: lock_source,
            hash: hash.clone(),
        },
        report: PinImportMappedPatch {
            name: patch.name.clone(),
            source_kind,
            hash_algo: hash.algo.to_string(),
        },
    })
}

struct MappedInputs {
    manifest_inputs: Vec<ManifestInput>,
    lock_inputs: BTreeMap<String, LockEntry>,
    report_inputs: Vec<PinImportMappedInput>,
    preserved_semantics: Vec<PinImportSemantic>,
    rewritten_semantics: Vec<PinImportSemantic>,
}

fn map_inputs(
    pins: &[ExternalPin],
    patch_names: &BTreeSet<String>,
    blockers: &mut Vec<PinImportBlocker>,
) -> Option<MappedInputs> {
    let blocker_count_before = blockers.len();
    let mut names = BTreeSet::new();
    let mut manifest_inputs = Vec::with_capacity(pins.len());
    let mut lock_inputs = BTreeMap::new();
    let mut report_inputs = Vec::with_capacity(pins.len());
    let mut preserved_semantics = Vec::new();
    let mut rewritten_semantics = Vec::new();
    for pin in pins {
        validate_pin_identity(pin, &mut names, blockers);
        validate_pin_patches(pin, patch_names, blockers);
        validate_pin_metadata(pin, &mut preserved_semantics, &mut rewritten_semantics, blockers);
        if let Some(mapped) = map_pin(pin, blockers) {
            report_inputs.push(mapped.report);
            manifest_inputs.push(mapped.manifest_input);
            lock_inputs.insert(pin.name.clone(), mapped.lock_entry);
        }
    }
    if blockers.len() != blocker_count_before {
        return None;
    }
    Some(MappedInputs {
        manifest_inputs,
        lock_inputs,
        report_inputs,
        preserved_semantics,
        rewritten_semantics,
    })
}

fn validate_pin_identity(pin: &ExternalPin, names: &mut BTreeSet<String>, blockers: &mut Vec<PinImportBlocker>) {
    validate_name(&pin.name, "input", blockers);
    if !names.insert(pin.name.clone()) {
        blockers.push(blocker("duplicate-input", &pin.name, "input names must be unique"));
    }
}

fn validate_name(name: &str, subject: &str, blockers: &mut Vec<PinImportBlocker>) {
    if name.is_empty() || name.len() > MAX_IMPORT_NAME_BYTES {
        blockers.push(blocker("invalid-name", subject, "name must be non-empty and bounded"));
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        blockers.push(blocker("invalid-name", name, "name must not contain path separators or NUL"));
    }
}

fn validate_pin_patches(pin: &ExternalPin, patch_names: &BTreeSet<String>, blockers: &mut Vec<PinImportBlocker>) {
    let mut seen = BTreeSet::new();
    for patch in &pin.patches {
        if !patch_names.contains(patch) {
            blockers.push(blocker("unknown-patch", &pin.name, &format!("input references unknown patch `{patch}`")));
        }
        if !seen.insert(patch) {
            blockers.push(blocker("duplicate-patch-reference", &pin.name, "input repeats a patch reference"));
        }
    }
}

fn validate_pin_metadata(
    pin: &ExternalPin,
    preserved: &mut Vec<PinImportSemantic>,
    rewritten: &mut Vec<PinImportSemantic>,
    blockers: &mut Vec<PinImportBlocker>,
) {
    validate_semantic_value(
        pin,
        "freshness",
        pin.metadata.freshness.as_deref(),
        supported_freshness,
        preserved,
        blockers,
    );
    validate_semantic_value(
        pin,
        "fetch_policy",
        pin.metadata.fetch_policy.as_deref(),
        supported_fetch_policy,
        preserved,
        blockers,
    );
    validate_semantic_value(
        pin,
        "trust_policy",
        pin.metadata.trust_policy.as_deref(),
        supported_trust_policy,
        preserved,
        blockers,
    );
    if pin.metadata.composition_semantics.len() > MAX_IMPORT_SEMANTICS_PER_PIN {
        blockers.push(blocker(
            "too-many-composition-semantics",
            &pin.name,
            "composition metadata list exceeds importer limit",
        ));
    }
    for semantic in &pin.metadata.composition_semantics {
        blockers.push(blocker("composition-semantic", &pin.name, semantic));
        rewritten.push(PinImportSemantic {
            subject: pin.name.clone(),
            key: "composition".to_string(),
            value: semantic.clone(),
        });
    }
}

fn validate_semantic_value(
    pin: &ExternalPin,
    key: &str,
    value: Option<&str>,
    supported: fn(&str) -> bool,
    preserved: &mut Vec<PinImportSemantic>,
    blockers: &mut Vec<PinImportBlocker>,
) {
    let Some(value) = value else { return };
    if supported(value) {
        preserved.push(PinImportSemantic {
            subject: pin.name.clone(),
            key: key.to_string(),
            value: value.to_string(),
        });
        return;
    }
    blockers.push(blocker("unsupported-metadata", &pin.name, &format!("unsupported {key}: {value}")));
}

fn supported_freshness(value: &str) -> bool {
    matches!(value, SEMANTIC_FRESHNESS_LOCKED | SEMANTIC_FRESHNESS_NONE)
}

fn supported_fetch_policy(value: &str) -> bool {
    matches!(value, SEMANTIC_FETCH_REFRESH | SEMANTIC_FETCH_GENERATION)
}

fn map_external_fetch_policy(metadata: &ExternalPinMetadata) -> InputFetchPolicy {
    match metadata.fetch_policy.as_deref() {
        Some(SEMANTIC_FETCH_GENERATION) => InputFetchPolicy::BuildFetchAction,
        _ => InputFetchPolicy::GenerationMaterial,
    }
}

fn supported_trust_policy(value: &str) -> bool {
    matches!(value, SEMANTIC_TRUST_CONTENT_HASH | SEMANTIC_TRUST_HASH_ONLY)
}

struct MappedPin {
    manifest_input: ManifestInput,
    lock_entry: LockEntry,
    report: PinImportMappedInput,
}

fn map_pin(pin: &ExternalPin, blockers: &mut Vec<PinImportBlocker>) -> Option<MappedPin> {
    let hash = map_locked_hash(&pin.hash, &pin.name, blockers);
    let mapped_kind = map_pin_kind(&pin.kind, &pin.name, blockers);
    let (Some(hash), Some((manifest_kind, lock_kind, kind_label))) = (hash, mapped_kind) else {
        return None;
    };
    let fetch_policy = map_external_fetch_policy(&pin.metadata);
    Some(MappedPin {
        manifest_input: ManifestInput {
            name: pin.name.clone(),
            kind: manifest_kind,
            hash: HashSpec {
                algo: hash.algo.clone(),
                expected: Some(hash.value.clone()),
            },
            frozen: pin.frozen,
            mirrors: pin.mirrors.clone(),
            patches: pin.patches.clone(),
            fetch_policy,
            freshness: None,
        },
        lock_entry: LockEntry {
            kind: lock_kind,
            hash: hash.clone(),
            patches: pin.patches.clone(),
            mirrors: pin.mirrors.clone(),
            fetch_policy,
            freshness: None,
        },
        report: PinImportMappedInput {
            name: pin.name.clone(),
            kind: kind_label,
            frozen: pin.frozen,
            hash_algo: hash.algo.to_string(),
            lock_identity: pin.lock_identity.clone(),
        },
    })
}

fn map_pin_kind(
    kind: &ExternalPinKind,
    subject: &str,
    blockers: &mut Vec<PinImportBlocker>,
) -> Option<(InputKind, LockedKind, String)> {
    match kind {
        ExternalPinKind::File { url } => {
            Some((InputKind::File { url: url.clone() }, LockedKind::File { url: url.clone() }, KIND_FILE.to_string()))
        }
        ExternalPinKind::Tarball { url } => Some((
            InputKind::Tarball { url: url.clone() },
            LockedKind::Tarball { url: url.clone() },
            KIND_TARBALL.to_string(),
        )),
        ExternalPinKind::Git {
            repository,
            reference,
            rev,
        } => map_git_kind(repository, reference.as_deref(), rev.as_deref(), subject, blockers),
        ExternalPinKind::Unsupported { kind } => {
            blockers.push(blocker("unsupported-source-kind", subject, &format!("unsupported source kind: {kind}")));
            None
        }
    }
}

fn map_git_kind(
    repository: &str,
    reference: Option<&str>,
    rev: Option<&str>,
    subject: &str,
    blockers: &mut Vec<PinImportBlocker>,
) -> Option<(InputKind, LockedKind, String)> {
    let Some(rev) = rev else {
        blockers.push(blocker("missing-git-rev", subject, "git inputs require resolved lock rev identity"));
        return None;
    };
    let reference = reference.unwrap_or(rev);
    let manifest_reference = if reference == rev {
        GitReference::Rev(reference.to_string())
    } else {
        GitReference::Branch(reference.to_string())
    };
    Some((
        InputKind::Git {
            repository: repository.to_string(),
            reference: manifest_reference,
        },
        LockedKind::Git {
            repository: repository.to_string(),
            rev: rev.to_string(),
            ref_name: Some(reference.to_string()),
        },
        KIND_GIT.to_string(),
    ))
}

fn map_locked_hash(hash: &ExternalHash, subject: &str, blockers: &mut Vec<PinImportBlocker>) -> Option<LockedHash> {
    let algo = match hash.algo.as_str() {
        HASH_ALGO_SHA256 => HashAlgo::Sha256,
        HASH_ALGO_SHA512 => HashAlgo::Sha512,
        HASH_ALGO_BLAKE3 => HashAlgo::Blake3,
        other => {
            blockers.push(blocker(
                "unsupported-hash-algorithm",
                subject,
                &format!("unsupported hash algorithm: {other}"),
            ));
            return None;
        }
    };
    let Some(value) = &hash.value else {
        blockers.push(blocker("missing-hash", subject, "imported pins must carry resolved expected hashes"));
        return None;
    };
    if value.is_empty() {
        blockers.push(blocker("missing-hash", subject, "imported hash value must be non-empty"));
        return None;
    }
    Some(LockedHash {
        algo,
        value: value.clone(),
    })
}

fn plan_file_operations(
    manifest: ProjectManifest,
    lock: Lockfile,
    options: &PinImportOptions,
    blockers: &mut Vec<PinImportBlocker>,
) -> Vec<PinImportFileOperation> {
    let manifest_content = render_manifest_nickel(&manifest);
    let lock_content = match lock.clone().to_json() {
        Ok(content) => content,
        Err(error) => {
            blockers.push(blocker("lock-serialization", &options.lock_file, &error.to_string()));
            return Vec::new();
        }
    };
    let inputs_content = generate_inputs_ncl(lock);
    vec![
        file_operation(&options.project_file, manifest_content),
        file_operation(&options.lock_file, lock_content),
        file_operation(&options.inputs_file, inputs_content),
    ]
}

fn file_operation(path: &str, content: String) -> PinImportFileOperation {
    let digest = blake3::hash(content.as_bytes()).to_hex().to_string();
    PinImportFileOperation {
        path: path.to_string(),
        action: "write".to_string(),
        digest_blake3: digest,
        content,
    }
}

fn validate_file_conflicts(
    operations: &[PinImportFileOperation],
    existing_files: &[ExternalExistingFile],
    blockers: &mut Vec<PinImportBlocker>,
) {
    let existing = existing_files
        .iter()
        .map(|file| (file.path.as_str(), file.content.as_str()))
        .collect::<BTreeMap<_, _>>();
    for operation in operations {
        if let Some(content) = existing.get(operation.path.as_str()) {
            if *content != operation.content {
                blockers.push(blocker(
                    "existing-file-conflict",
                    &operation.path,
                    "existing file differs from import plan",
                ));
            }
        }
    }
}

fn render_manifest_nickel(manifest: &ProjectManifest) -> String {
    let mut out = String::new();
    out.push_str("# Generated by `mantle import pins apply`. DO NOT EDIT BY HAND WITHOUT REVIEW.\n");
    out.push_str("{\n");
    out.push_str(&format!("  version = {},\n", quote(&manifest.version)));
    render_manifest_inputs(&mut out, &manifest.inputs);
    render_manifest_patches(&mut out, &manifest.patches);
    out.push_str("}\n");
    out
}

fn render_manifest_inputs(out: &mut String, inputs: &[ManifestInput]) {
    out.push_str("  inputs = [\n");
    for input in inputs {
        out.push_str("    {\n");
        out.push_str(&format!("      name = {},\n", quote(&input.name)));
        render_input_kind(out, &input.kind);
        render_hash_spec(out, "hash", &input.hash, "      ");
        out.push_str(&format!("      frozen = {},\n", input.frozen));
        render_string_array(out, "mirrors", &input.mirrors, "      ");
        render_string_array(out, "patches", &input.patches, "      ");
        out.push_str(&format!("      fetch_policy = {},\n", quote(input.fetch_policy.as_str())));
        out.push_str("    },\n");
    }
    out.push_str("  ],\n");
}

fn render_input_kind(out: &mut String, kind: &InputKind) {
    match kind {
        InputKind::File { url } => {
            out.push_str(&format!("      kind = {{ type = {}, url = {} }},\n", quote(KIND_FILE), quote(url)));
        }
        InputKind::Tarball { url } => {
            out.push_str(&format!("      kind = {{ type = {}, url = {} }},\n", quote(KIND_TARBALL), quote(url)));
        }
        InputKind::Git { repository, reference } => {
            out.push_str("      kind = {\n");
            out.push_str(&format!("        type = {},\n", quote(KIND_GIT)));
            out.push_str(&format!("        repository = {},\n", quote(repository)));
            render_git_reference(out, reference);
            out.push_str("      },\n");
        }
    }
}

fn render_git_reference(out: &mut String, reference: &GitReference) {
    let (kind, value) = match reference {
        GitReference::Branch(value) => ("branch", value),
        GitReference::Tag(value) => ("tag", value),
        GitReference::Rev(value) => ("rev", value),
    };
    out.push_str(&format!("        reference = {{ ref_type = {}, ref_value = {} }},\n", quote(kind), quote(value)));
}

fn render_hash_spec(out: &mut String, field: &str, hash: &HashSpec, indent: &str) {
    out.push_str(&format!("{indent}{field} = {{\n"));
    out.push_str(&format!("{indent}  algo = {},\n", quote(&hash.algo.to_string())));
    if let Some(expected) = &hash.expected {
        out.push_str(&format!("{indent}  expected = {},\n", quote(expected)));
    }
    out.push_str(&format!("{indent}}},\n"));
}

fn render_string_array(out: &mut String, field: &str, values: &[String], indent: &str) {
    let rendered = values.iter().map(|value| quote(value)).collect::<Vec<_>>().join(", ");
    out.push_str(&format!("{indent}{field} = [{rendered}],\n"));
}

fn render_manifest_patches(out: &mut String, patches: &[PatchDef]) {
    out.push_str("  patches = [\n");
    for patch in patches {
        out.push_str("    {\n");
        out.push_str(&format!("      name = {},\n", quote(&patch.name)));
        render_patch_source(out, &patch.source);
        out.push_str("    },\n");
    }
    out.push_str("  ],\n");
}

fn render_patch_source(out: &mut String, source: &PatchSource) {
    match source {
        PatchSource::Local { path } => {
            out.push_str(&format!(
                "      source = {{ type = {}, path = {} }},\n",
                quote(PATCH_KIND_LOCAL),
                quote(path)
            ));
        }
        PatchSource::Remote { url, hash } => {
            out.push_str("      source = {\n");
            out.push_str(&format!("        type = {},\n", quote(PATCH_KIND_REMOTE)));
            out.push_str(&format!("        url = {},\n", quote(url)));
            render_hash_spec(out, "hash", hash, "        ");
            out.push_str("      },\n");
        }
    }
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization is infallible")
}

fn pin_import_non_claims() -> Vec<String> {
    vec![
        "import-plan-does-not-prove-source-availability".to_string(),
        "import-plan-does-not-prove-build-success".to_string(),
        "import-plan-does-not-import-module-or-composition-semantics".to_string(),
    ]
}

fn blocker(class: &str, subject: &str, message: &str) -> PinImportBlocker {
    PinImportBlocker {
        class: class.to_string(),
        subject: subject.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const HASH_SHA256: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const HASH_BLAKE3: &str = "blake3-0000000000000000000000000000000000000000000000000000000000000000";
    const HASH_SHA512: &str = "sha512-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";

    fn hash(algo: &str, value: &str) -> ExternalHash {
        ExternalHash {
            algo: algo.to_string(),
            value: Some(value.to_string()),
        }
    }

    fn options() -> PinImportOptions {
        PinImportOptions::default()
    }

    fn fixture_pin_set() -> ExternalPinSet {
        ExternalPinSet {
            importer: PIN_IMPORT_SUPPORTED_IMPORTER.to_string(),
            source_label: "nixtamal-fixture.json".to_string(),
            pins: vec![
                ExternalPin {
                    name: "tool".to_string(),
                    kind: ExternalPinKind::File {
                        url: "https://example.test/tool".to_string(),
                    },
                    hash: hash(HASH_ALGO_BLAKE3, HASH_BLAKE3),
                    frozen: true,
                    mirrors: vec!["https://mirror.example.test/tool".to_string()],
                    patches: vec![],
                    metadata: ExternalPinMetadata {
                        freshness: Some(SEMANTIC_FRESHNESS_LOCKED.to_string()),
                        fetch_policy: Some(SEMANTIC_FETCH_REFRESH.to_string()),
                        trust_policy: Some(SEMANTIC_TRUST_CONTENT_HASH.to_string()),
                        composition_semantics: vec![],
                    },
                    lock_identity: Some("nixtamal:tool@1".to_string()),
                },
                ExternalPin {
                    name: "archive".to_string(),
                    kind: ExternalPinKind::Tarball {
                        url: "https://example.test/archive.tar.gz".to_string(),
                    },
                    hash: hash(HASH_ALGO_SHA512, HASH_SHA512),
                    frozen: false,
                    mirrors: vec![],
                    patches: vec!["fix".to_string()],
                    metadata: ExternalPinMetadata::default(),
                    lock_identity: None,
                },
                ExternalPin {
                    name: "repo".to_string(),
                    kind: ExternalPinKind::Git {
                        repository: "https://example.test/repo.git".to_string(),
                        reference: Some("main".to_string()),
                        rev: Some("0123456789abcdef".to_string()),
                    },
                    hash: hash(HASH_ALGO_SHA256, HASH_SHA256),
                    frozen: false,
                    mirrors: vec!["https://mirror.example.test/repo.git".to_string()],
                    patches: vec![],
                    metadata: ExternalPinMetadata::default(),
                    lock_identity: None,
                },
            ],
            patches: vec![ExternalPatch {
                name: "fix".to_string(),
                source: ExternalPatchSource::Remote {
                    url: "https://example.test/fix.patch".to_string(),
                },
                hash: hash(HASH_ALGO_BLAKE3, HASH_BLAKE3),
            }],
            existing_files: vec![],
            unsupported_semantics: vec![],
        }
    }

    // r[verify project_workflows.nixtamal_importer]
    #[test]
    fn nixtamal_supported_semantics_map_to_manifest_lock_and_inputs_plan() {
        let plan = build_pin_import_plan(fixture_pin_set(), options());

        assert!(plan.can_apply(), "blockers: {:?}", plan.blockers);
        assert_eq!(plan.mapped_inputs.len(), 3);
        assert_eq!(plan.mapped_patches.len(), 1);
        assert_eq!(plan.file_operations.len(), 3);
        assert!(plan.file_operations.iter().any(|operation| operation.path == PIN_IMPORT_DEFAULT_PROJECT_FILE));
        assert!(plan.file_operations.iter().any(|operation| operation.content.contains("blake3")));
        assert!(plan.preserved_semantics.iter().any(|semantic| semantic.key == "freshness"));
        assert!(plan.preserved_semantics.iter().any(|semantic| semantic.key == "fetch_policy"));
        assert!(plan.preserved_semantics.iter().any(|semantic| semantic.key == "trust_policy"));
    }

    // r[verify project_workflows.project_lock_importers]
    #[test]
    fn future_adapter_seams_block_recursive_composition_semantics() {
        let mut pins = fixture_pin_set();
        pins.importer = "flake".to_string();
        pins.pins[0].metadata.composition_semantics = vec!["follows nixpkgs".to_string()];
        let plan = build_pin_import_plan(pins, PinImportOptions {
            importer: "flake".to_string(),
            ..options()
        });
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(!plan.can_apply());
        assert!(classes.contains("future-adapter"));
        assert!(classes.contains("composition-semantic"));
        assert!(plan.future_adapters.iter().any(|adapter| adapter == "npins"));
    }

    // r[verify project_workflows.project_lock_importers]
    #[test]
    fn negative_surfaces_block_without_partial_file_operations() {
        let mut pins = fixture_pin_set();
        pins.pins[0].kind = ExternalPinKind::Unsupported {
            kind: "darcs".to_string(),
        };
        pins.pins[1].hash.algo = "sha1".to_string();
        pins.pins[2].patches = vec!["missing".to_string()];
        pins.unsupported_semantics = vec!["command freshness probes are not imported yet".to_string()];
        let plan = build_pin_import_plan(pins, options());
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(!plan.can_apply());
        assert!(plan.file_operations.is_empty());
        assert!(classes.contains("unsupported-source-kind"));
        assert!(classes.contains("unsupported-hash-algorithm"));
        assert!(classes.contains("unknown-patch"));
        assert!(classes.contains("unsupported-import-semantic"));
    }

    // r[verify project_workflows.project_lock_importers]
    #[test]
    fn existing_file_conflict_blocks_apply_but_keeps_review_plan() {
        let mut pins = fixture_pin_set();
        pins.existing_files = vec![ExternalExistingFile {
            path: PIN_IMPORT_DEFAULT_LOCK_FILE.to_string(),
            content: "user lock\n".to_string(),
        }];
        let plan = build_pin_import_plan(pins, options());
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(!plan.can_apply());
        assert_eq!(plan.file_operations.len(), 3);
        assert!(classes.contains("existing-file-conflict"));
    }

    // r[verify project_workflows.nixtamal_importer]
    #[test]
    fn malformed_hash_and_git_identity_fail_closed() {
        let mut pins = fixture_pin_set();
        pins.pins[0].hash.value = None;
        pins.pins[2].kind = ExternalPinKind::Git {
            repository: "https://example.test/repo.git".to_string(),
            reference: Some("main".to_string()),
            rev: None,
        };
        let plan = build_pin_import_plan(pins, options());
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(!plan.can_apply());
        assert!(classes.contains("missing-hash"));
        assert!(classes.contains("missing-git-rev"));
    }
}
