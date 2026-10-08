// machine-artifact-public: import.command-reports
use std::path::Path;
use std::path::PathBuf;

use mantle_application_contract::CapabilityError;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::ObservationStatus;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use toml::Value;

use crate::cargo_profile_manifest::parse_profile_table;
use crate::cargo_profile_manifest::resolve_profile;
use crate::cargo_profile_manifest::select_command_profile;
use crate::errors::RunError;
use crate::pin_import::ImportApplied;
use crate::pin_import::ImportApplyError;
use crate::pin_import::ImportApplyFacts;
use crate::pin_import::ImportApplyPhase;
use crate::pin_import::ImportOutputCapability;
use crate::pin_import::ImportOutputPort;
use crate::pin_import::classify_blocked_import_effects;
use crate::pin_import::classify_import_effects;
use crate::pin_import::classify_import_failure;
use crate::pin_import::import_apply_error;
use crate::pin_import::import_capability_error;
use crate::pin_import::import_effect_plan;
use crate::pin_import::import_observation;
use crate::pin_import::import_targets_identity;
use crate::pin_import::verify_import_readback;

pub const CARGO_IMPORT_PLAN_SCHEMA: &str = "mantle-cargo-import-plan-v1";
const CARGO_IMPORT_READ_EFFECT: &str = "cargo-import-workspace-read";
const CARGO_IMPORT_WRITE_EFFECT: &str = "cargo-import-output-write";
const CARGO_IMPORT_READBACK_EFFECT: &str = "cargo-import-output-readback";
const CARGO_IMPORT_BLOCKED_EXIT_CODE: u8 = 3;

const DEFAULT_PROJECT_FILE: &str = "mantle-project.ncl";
const DEFAULT_INPUTS_FILE: &str = ".mantle/inputs.ncl";
const DEFAULT_TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";
#[cfg(test)]
const DEFAULT_PROFILE: &str = "release";
const SOURCE_FIELD_SUFFIX: &str = "_src";
const RUST_FIELD_NAME: &str = "rust";
const SEED_FIELD_NAME: &str = "musl_seed_toolchain";
const MUSL_FIELD_NAME: &str = "musl";
const RUST_INPUT_NAME: &str = "rust";
const SEED_INPUT_NAME: &str = "musl-seed-toolchain";
const MUSL_INPUT_NAME: &str = "musl";
const MAX_PACKAGES: usize = 64;
const MAX_BINARIES_PER_PACKAGE: usize = 16;
const MAX_DEPENDENCIES_PER_PACKAGE: usize = 128;
const BLAKE3_HEX_BYTES: usize = 64;
const CARGO_SHA256_HEX_BYTES: usize = 64;
const MAX_NAME_BYTES: usize = 128;
const MIN_TARGET_SEGMENTS: usize = 3;
const MAX_VENDOR_PACKAGES: usize = 512;
const MAX_VENDOR_DIGEST_NODES: usize = 100_000;
const MAX_SOURCE_INPUTS: usize = 5;
const GENERATED_FILE_OPERATION_COUNT: usize = 2;
const CARGO_CONFIG_TOML: &str = ".cargo/config.toml";
const CARGO_CONFIG_LEGACY: &str = ".cargo/config";
const CARGO_SOURCE_TABLE: &str = "source";
const CARGO_CRATES_IO_SOURCE: &str = "crates-io";
const CARGO_REPLACE_WITH_FIELD: &str = "replace-with";
const CARGO_DIRECTORY_FIELD: &str = "directory";
const CARGO_CHECKSUM_FILE: &str = ".cargo-checksum.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoImportOptions {
    pub selected_package: Option<String>,
    pub selected_binary: Option<String>,
    pub project_file: String,
    pub inputs_file: String,
    pub target_triple: String,
    pub profile: String,
}

impl Default for CargoImportOptions {
    fn default() -> Self {
        Self {
            selected_package: None,
            selected_binary: None,
            project_file: DEFAULT_PROJECT_FILE.to_string(),
            inputs_file: DEFAULT_INPUTS_FILE.to_string(),
            target_triple: DEFAULT_TARGET_TRIPLE.to_string(),
            profile: select_command_profile("install", false, None).expect("built-in install command profile resolves"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoWorkspaceFacts {
    pub workspace_root: String,
    pub lockfile_digest_blake3: Option<String>,
    pub lock_packages: Vec<CargoLockPackageFact>,
    pub vendor_source: Option<CargoVendorSourceFact>,
    pub packages: Vec<CargoPackageFact>,
    pub existing_files: Vec<ExistingProjectFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoPackageFact {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub package_root: String,
    pub binaries: Vec<String>,
    pub dependencies: Vec<CargoDependencyFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoDependencyFact {
    pub name: String,
    pub source: CargoDependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoLockPackageFact {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoVendorSourceFact {
    pub root: String,
    pub root_digest_blake3: Option<String>,
    pub replacement_source: String,
    pub packages: Vec<CargoVendorPackageFact>,
    pub blockers: Vec<CargoImportBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoVendorPackageFact {
    pub name: String,
    pub version: String,
    pub directory: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub digest_blake3: Option<String>,
    pub checksum_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CargoDependencySource {
    LocalPath(String),
    Registry,
    Git(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingProjectFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportPlan {
    pub schema: String,
    pub selected_package: Option<String>,
    pub selected_binary: Option<String>,
    pub target_triple: String,
    pub profile: String,
    pub file_operations: Vec<CargoImportFileOperation>,
    pub source_inputs: Vec<CargoImportSourceInput>,
    pub vendor_source: Option<CargoImportVendorSource>,
    pub blockers: Vec<CargoImportBlocker>,
    pub non_claims: Vec<String>,
    pub build_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportVendorSource {
    pub role: String,
    pub field: String,
    pub name: String,
    pub root: String,
    pub digest_blake3: Option<String>,
    pub replacement_source: String,
    pub packages: Vec<CargoVendorPackageFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportFileOperation {
    pub path: String,
    pub action: String,
    pub digest_blake3: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportSourceInput {
    pub role: String,
    pub field: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportBlocker {
    pub class: String,
    pub message: String,
}

pub struct CargoImportShellOptions<'a> {
    pub root: &'a Path,
    pub selected_package: Option<&'a str>,
    pub selected_binary: Option<&'a str>,
    pub project_file: &'a str,
    pub inputs_file: &'a str,
    pub target_triple: &'a str,
    pub profile: &'a str,
    pub apply: bool,
    pub json: bool,
}

struct WorkspaceFactLoadRequest<'a> {
    root: &'a Path,
    output: &'a dyn ImportOutputCapability,
    project_file: &'a str,
    inputs_file: &'a str,
}

trait CargoImportInputPort {
    fn load(
        &self,
        request: WorkspaceFactLoadRequest<'_>,
        profile: &str,
    ) -> Result<(CargoWorkspaceFacts, u32), CapabilityError>;
}

struct FilesystemCargoImportPort;

impl CargoImportInputPort for FilesystemCargoImportPort {
    fn load(
        &self,
        request: WorkspaceFactLoadRequest<'_>,
        profile: &str,
    ) -> Result<(CargoWorkspaceFacts, u32), CapabilityError> {
        let root = request.root;
        let facts = load_workspace_facts(request)
            .map_err(|error| CapabilityError::new("cargo-import-workspace-read", error.message()))?;
        validate_root_profile(root, profile)
            .map_err(|error| CapabilityError::new("cargo-import-profile-read", error.message()))?;
        Ok((facts, 2))
    }
}

struct RelativeOutputPathValidation<'a> {
    path: &'a str,
    blocker_class: &'a str,
}

struct NameValidation<'a> {
    label: &'a str,
    value: &'a str,
    blocker_class: &'a str,
}

struct CargoImportEmissionOptions {
    is_applied: bool,
    is_json: bool,
}

pub fn run_cargo_import(options: CargoImportShellOptions<'_>) -> Result<(), RunError> {
    let targets = [options.project_file, options.inputs_file];
    let target_identity = import_targets_identity(&targets);
    let effects = import_effect_plan(
        CARGO_IMPORT_READ_EFFECT,
        CARGO_IMPORT_WRITE_EFFECT,
        CARGO_IMPORT_READBACK_EFFECT,
        &target_identity,
        2,
        GENERATED_FILE_OPERATION_COUNT as u32,
        options.apply,
    )?;
    let output = ImportOutputPort::open(options.root)?;
    let input = FilesystemCargoImportPort;
    let (facts, read_calls) = input
        .load(
            WorkspaceFactLoadRequest {
                root: options.root,
                output: &output,
                project_file: options.project_file,
                inputs_file: options.inputs_file,
            },
            options.profile,
        )
        .map_err(import_capability_error)?;
    let plan = build_cargo_import_plan(facts, CargoImportOptions {
        selected_package: options.selected_package.map(ToOwned::to_owned),
        selected_binary: options.selected_binary.map(ToOwned::to_owned),
        project_file: options.project_file.to_string(),
        inputs_file: options.inputs_file.to_string(),
        target_triple: options.target_triple.to_string(),
        profile: options.profile.to_string(),
    });
    debug_assert_eq!(plan.schema, CARGO_IMPORT_PLAN_SCHEMA);
    debug_assert_eq!(plan.target_triple.as_str(), options.target_triple);
    let read = import_observation(
        CARGO_IMPORT_READ_EFFECT,
        EffectKind::ReadFiles,
        if plan.blockers.is_empty() {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        EffectMeasure::Calls(read_calls),
        EffectOutput::None,
    );
    let emission = CargoImportEmissionOptions {
        is_applied: options.apply,
        is_json: options.json,
    };

    if options.apply {
        if !plan.blockers.is_empty() {
            classify_blocked_import_effects(&effects, &read)?;
            emit_cargo_import_plan(&plan, &CargoImportEmissionOptions {
                is_applied: false,
                is_json: options.json,
            })?;
            return Err(RunError::Reported(CARGO_IMPORT_BLOCKED_EXIT_CODE));
        }
        let applied = match apply_cargo_import_plan(&output, &plan, &targets) {
            Ok(facts) => facts,
            Err(failure) => return Err(classify_import_failure(&effects, read, failure)),
        };
        let observations = [
            read,
            import_observation(
                CARGO_IMPORT_WRITE_EFFECT,
                EffectKind::WriteFiles,
                ObservationStatus::Succeeded,
                EffectMeasure::Items(applied.facts.writes),
                EffectOutput::None,
            ),
            import_observation(
                CARGO_IMPORT_READBACK_EFFECT,
                EffectKind::ReadFiles,
                ObservationStatus::Succeeded,
                EffectMeasure::Items(applied.facts.verified),
                EffectOutput::Identity(applied.target_identity),
            ),
        ];
        classify_import_effects(&effects, &observations)?;
        emit_cargo_import_plan(&plan, &emission)?;
        return Ok(());
    }
    if plan.blockers.is_empty() {
        classify_import_effects(&effects, std::slice::from_ref(&read))?;
    } else {
        classify_blocked_import_effects(&effects, &read)?;
    }
    emit_cargo_import_plan(&plan, &emission)?;
    if plan.blockers.is_empty() {
        return Ok(());
    }
    Err(RunError::Reported(CARGO_IMPORT_BLOCKED_EXIT_CODE))
}

pub fn build_cargo_import_plan(facts: CargoWorkspaceFacts, options: CargoImportOptions) -> CargoImportPlan {
    let mut blockers = Vec::new();
    validate_workspace_shape(&facts, &mut blockers);
    validate_import_options(&options, &mut blockers);
    validate_vendor_source_facts(facts.vendor_source.as_ref(), &mut blockers);

    let selected_package = select_package(&facts, options.selected_package.as_deref(), &mut blockers);
    let selected_binary =
        selected_package.and_then(|package| select_binary(package, options.selected_binary.as_deref(), &mut blockers));
    if let Some(package) = selected_package {
        validate_package_fact(package, &facts.lock_packages, facts.vendor_source.as_ref(), &mut blockers);
    }
    validate_lockfile(facts.lockfile_digest_blake3.as_deref(), &mut blockers);

    let mut file_operations = Vec::new();
    let mut source_inputs = Vec::new();
    let mut vendor_source = None;
    if blockers.is_empty() {
        if let (Some(package), Some(binary)) = (selected_package, selected_binary) {
            vendor_source = accepted_vendor_source(package, facts.vendor_source.as_ref());
            source_inputs = source_inputs_for(package, vendor_source.as_ref());
            file_operations =
                generated_file_operations(package, binary, &options, &source_inputs, vendor_source.as_ref());
            validate_file_conflicts(&mut file_operations, &facts.existing_files, &mut blockers);
        } else {
            blockers.push(blocker(
                "internal-selection-invariant",
                "blocker-free Cargo import selection must identify one package and binary",
            ));
        }
    }

    let plan = CargoImportPlan {
        schema: CARGO_IMPORT_PLAN_SCHEMA.to_string(),
        selected_package: selected_package.map(|package| package.name.clone()),
        selected_binary: selected_binary.map(ToOwned::to_owned),
        target_triple: options.target_triple,
        profile: options.profile,
        file_operations,
        source_inputs,
        vendor_source,
        blockers,
        non_claims: vec![
            "not-cargo-free-execution".to_string(),
            "not-module-layer-semantics".to_string(),
            "not-network-vendoring".to_string(),
            "not-full-cargo-compatibility".to_string(),
        ],
        build_hints: vec![
            "Review the plan, materialize declared source/toolchain inputs, then run `mantle import cargo --apply`."
                .to_string(),
        ],
    };
    debug_assert!(!plan.non_claims.is_empty());
    debug_assert!(!plan.build_hints.is_empty());
    plan
}

fn validate_workspace_shape(facts: &CargoWorkspaceFacts, blockers: &mut Vec<CargoImportBlocker>) {
    if facts.workspace_root.is_empty() {
        blockers.push(blocker("invalid-workspace-root", "workspace root must be valid UTF-8"));
    }
    if facts.packages.is_empty() {
        blockers.push(blocker("missing-package", "Cargo import requires at least one package"));
    }
    if facts.packages.len() > MAX_PACKAGES {
        blockers.push(blocker("too-many-packages", "Cargo import initial surface supports a bounded package count"));
    }
}

fn validate_import_options(options: &CargoImportOptions, blockers: &mut Vec<CargoImportBlocker>) {
    let blocker_count_before = blockers.len();
    validate_relative_output_path(
        RelativeOutputPathValidation {
            path: &options.project_file,
            blocker_class: "invalid-project-file",
        },
        blockers,
    );
    validate_relative_output_path(
        RelativeOutputPathValidation {
            path: &options.inputs_file,
            blocker_class: "invalid-inputs-file",
        },
        blockers,
    );
    if options.target_triple.split('-').filter(|segment| !segment.is_empty()).count() < MIN_TARGET_SEGMENTS {
        blockers.push(blocker("unsupported-target-triple", "target triple must be explicit"));
    }
    if options.profile.is_empty() {
        blockers.push(blocker("unsupported-profile", "import profile must not be empty"));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers[blocker_count_before..].iter().all(|blocker| !blocker.class.is_empty()));
}

fn validate_relative_output_path(validation: RelativeOutputPathValidation<'_>, blockers: &mut Vec<CargoImportBlocker>) {
    let path = validation.path;
    if path.is_empty() || path.starts_with('/') || path.split('/').any(|part| part == ".." || part.is_empty()) {
        blockers.push(blocker(
            validation.blocker_class,
            "generated file path must be a relative path without parent traversal",
        ));
    }
}

fn select_package<'a>(
    facts: &'a CargoWorkspaceFacts,
    selected_package: Option<&str>,
    blockers: &mut Vec<CargoImportBlocker>,
) -> Option<&'a CargoPackageFact> {
    if let Some(name) = selected_package {
        let matches: Vec<&CargoPackageFact> = facts.packages.iter().filter(|package| package.name == name).collect();
        if matches.len() == 1 {
            return matches.first().copied();
        }
        blockers.push(blocker("unknown-package", format!("selected package `{name}` is not in the workspace")));
        return None;
    }
    if facts.packages.len() == 1 {
        return facts.packages.first();
    }
    blockers.push(blocker("ambiguous-default-package", "workspace has multiple packages; select one with --package"));
    None
}

fn select_binary<'a>(
    package: &'a CargoPackageFact,
    selected_binary: Option<&'a str>,
    blockers: &mut Vec<CargoImportBlocker>,
) -> Option<&'a str> {
    let blocker_count_before = blockers.len();
    if let Some(name) = selected_binary {
        if package.binaries.iter().any(|binary| binary == name) {
            debug_assert!(package.binaries.iter().any(|binary| binary == name));
            debug_assert_eq!(blockers.len(), blocker_count_before);
            return Some(name);
        }
        blockers.push(blocker(
            "missing-selected-binary",
            format!("selected binary `{name}` is not declared by package `{}`", package.name),
        ));
        return None;
    }
    if package.binaries.len() == 1 {
        return package.binaries.first().map(String::as_str);
    }
    if package.binaries.is_empty() {
        blockers.push(blocker("missing-selected-binary", "package has no supported binary target"));
    } else {
        blockers.push(blocker(
            "ambiguous-binary-selection",
            "package has multiple binary targets; select one with --binary",
        ));
    }
    None
}

fn validate_package_fact(
    package: &CargoPackageFact,
    lock_packages: &[CargoLockPackageFact],
    vendor_source: Option<&CargoVendorSourceFact>,
    blockers: &mut Vec<CargoImportBlocker>,
) {
    let blocker_count_before = blockers.len();
    validate_name(
        NameValidation {
            label: "package name",
            value: &package.name,
            blocker_class: "malformed-package-name",
        },
        blockers,
    );
    for binary in &package.binaries {
        validate_name(
            NameValidation {
                label: "binary name",
                value: binary,
                blocker_class: "malformed-binary-name",
            },
            blockers,
        );
    }
    if package.binaries.len() > MAX_BINARIES_PER_PACKAGE {
        blockers.push(blocker("too-many-binaries", "package has too many binary targets for initial import"));
    }
    if package.dependencies.len() > MAX_DEPENDENCIES_PER_PACKAGE {
        blockers.push(blocker("too-many-dependencies", "package dependency count exceeds initial import bound"));
    }
    for dependency in &package.dependencies {
        validate_dependency_source(dependency, lock_packages, vendor_source, blockers);
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(
        blockers[blocker_count_before..]
            .iter()
            .all(|blocker| !blocker.class.is_empty() && !blocker.message.is_empty())
    );
}

fn validate_dependency_source(
    dependency: &CargoDependencyFact,
    lock_packages: &[CargoLockPackageFact],
    vendor_source: Option<&CargoVendorSourceFact>,
    blockers: &mut Vec<CargoImportBlocker>,
) {
    match &dependency.source {
        CargoDependencySource::LocalPath(_) => {}
        CargoDependencySource::Registry | CargoDependencySource::Git(_) => {
            validate_vendored_dependency(dependency, lock_packages, vendor_source, blockers);
        }
        CargoDependencySource::Unknown => blockers.push(blocker(
            "unsupported-dependency-source",
            format!("dependency `{}` has an unsupported source declaration", dependency.name),
        )),
    }
}

fn validate_vendor_source_facts(vendor_source: Option<&CargoVendorSourceFact>, blockers: &mut Vec<CargoImportBlocker>) {
    let Some(vendor_source) = vendor_source else {
        return;
    };
    blockers.extend(vendor_source.blockers.iter().cloned());
    if vendor_source.packages.len() > MAX_VENDOR_PACKAGES {
        blockers.push(blocker("too-many-vendored-packages", "vendored source package count exceeds import bound"));
    }
}

fn validate_vendored_dependency(
    dependency: &CargoDependencyFact,
    lock_packages: &[CargoLockPackageFact],
    vendor_source: Option<&CargoVendorSourceFact>,
    blockers: &mut Vec<CargoImportBlocker>,
) {
    debug_assert!(matches!(&dependency.source, CargoDependencySource::Registry | CargoDependencySource::Git(_)));
    debug_assert!(!matches!(&dependency.source, CargoDependencySource::LocalPath(_) | CargoDependencySource::Unknown));
    let lock_package = match unique_lock_package(&dependency.name, lock_packages) {
        Ok(lock_package) => lock_package,
        Err(class) => {
            blockers.push(blocker(class, format!("dependency `{}` has no unique Cargo.lock package", dependency.name)));
            return;
        }
    };
    if !dependency_source_matches_lock(dependency, lock_package) {
        blockers.push(blocker(
            "lock-source-mismatch",
            format!("dependency `{}` source does not match Cargo.lock", dependency.name),
        ));
        return;
    }
    let Some(vendor_source) = vendor_source else {
        blockers.push(blocker(
            "missing-vendored-source",
            format!("dependency `{}` requires declared vendored source material", dependency.name),
        ));
        return;
    };
    let matches: Vec<&CargoVendorPackageFact> = vendor_source
        .packages
        .iter()
        .filter(|package| package.name == lock_package.name && package.version == lock_package.version)
        .collect();
    if matches.is_empty() {
        blockers.push(blocker(
            "missing-vendored-package",
            format!("dependency `{}` has no vendored package entry", dependency.name),
        ));
        return;
    }
    if matches.len() != 1 {
        blockers.push(blocker(
            "ambiguous-vendored-package",
            format!("dependency `{}` maps to multiple vendored package entries", dependency.name),
        ));
        return;
    }
    let vendored = matches[0];
    if vendored.checksum_status != "verified" {
        blockers.push(blocker(
            "stale-vendor-checksum",
            format!("dependency `{}` has unverified Cargo checksum metadata", dependency.name),
        ));
    }
    if lock_package.checksum.is_some() && vendored.checksum.is_none() {
        blockers.push(blocker(
            "missing-vendor-checksum",
            format!("dependency `{}` lockfile checksum is not represented in vendor metadata", dependency.name),
        ));
    }
}

fn unique_lock_package<'a>(
    name: &str,
    lock_packages: &'a [CargoLockPackageFact],
) -> Result<&'a CargoLockPackageFact, &'static str> {
    let matches: Vec<&CargoLockPackageFact> = lock_packages.iter().filter(|package| package.name == name).collect();
    if matches.len() == 1 {
        return Ok(matches[0]);
    }
    if matches.is_empty() {
        return Err("missing-lock-entry");
    }
    Err("ambiguous-lock-entry")
}

fn dependency_source_matches_lock(dependency: &CargoDependencyFact, lock_package: &CargoLockPackageFact) -> bool {
    match &dependency.source {
        CargoDependencySource::Registry => {
            lock_package.source.as_deref().is_some_and(|source| source.starts_with("registry+"))
        }
        CargoDependencySource::Git(_) => {
            lock_package.source.as_deref().is_some_and(|source| source.starts_with("git+"))
        }
        CargoDependencySource::LocalPath(_) => lock_package.source.is_none(),
        CargoDependencySource::Unknown => false,
    }
}

fn validate_lockfile(digest: Option<&str>, blockers: &mut Vec<CargoImportBlocker>) {
    let Some(digest) = digest else {
        blockers.push(blocker("missing-lockfile", "Cargo.lock is required for offline Cargo import"));
        return;
    };
    if digest.len() == BLAKE3_HEX_BYTES && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return;
    }
    blockers.push(blocker("invalid-lockfile-digest", "Cargo.lock digest must be BLAKE3 hex"));
}

fn source_inputs_for(
    package: &CargoPackageFact,
    vendor_source: Option<&CargoImportVendorSource>,
) -> Vec<CargoImportSourceInput> {
    let source_field = source_field_name(&package.name);
    let mut inputs = vec![
        CargoImportSourceInput {
            role: "package-source".to_string(),
            field: source_field,
            name: format!("{}-src", package.name),
        },
        CargoImportSourceInput {
            role: "rust-toolchain".to_string(),
            field: RUST_FIELD_NAME.to_string(),
            name: RUST_INPUT_NAME.to_string(),
        },
        CargoImportSourceInput {
            role: "seed-toolchain".to_string(),
            field: SEED_FIELD_NAME.to_string(),
            name: SEED_INPUT_NAME.to_string(),
        },
        CargoImportSourceInput {
            role: "musl-runtime".to_string(),
            field: MUSL_FIELD_NAME.to_string(),
            name: MUSL_INPUT_NAME.to_string(),
        },
    ];
    if let Some(vendor_source) = vendor_source {
        inputs.push(CargoImportSourceInput {
            role: vendor_source.role.clone(),
            field: vendor_source.field.clone(),
            name: vendor_source.name.clone(),
        });
    }
    let package_source_count = inputs.iter().filter(|input| input.role == "package-source").count();
    debug_assert_eq!(package_source_count, 1);
    debug_assert!(inputs.len() <= MAX_SOURCE_INPUTS);
    inputs
}

fn accepted_vendor_source(
    package: &CargoPackageFact,
    vendor_source: Option<&CargoVendorSourceFact>,
) -> Option<CargoImportVendorSource> {
    let vendor_source = vendor_source?;
    if vendor_source.blockers.is_empty()
        && package
            .dependencies
            .iter()
            .any(|dependency| !matches!(&dependency.source, CargoDependencySource::LocalPath(_)))
    {
        debug_assert!(vendor_source.blockers.is_empty());
        debug_assert!(
            package
                .dependencies
                .iter()
                .any(|dependency| !matches!(&dependency.source, CargoDependencySource::LocalPath(_)))
        );
        return Some(CargoImportVendorSource {
            role: "vendored-dependencies".to_string(),
            field: "vendor_src".to_string(),
            name: format!("{}-vendor", package.name),
            root: vendor_source.root.clone(),
            digest_blake3: vendor_source.root_digest_blake3.clone(),
            replacement_source: vendor_source.replacement_source.clone(),
            packages: vendor_source.packages.clone(),
        });
    }
    None
}

fn generated_file_operations(
    package: &CargoPackageFact,
    binary: &str,
    options: &CargoImportOptions,
    source_inputs: &[CargoImportSourceInput],
    vendor_source: Option<&CargoImportVendorSource>,
) -> Vec<CargoImportFileOperation> {
    let project = render_project_ncl(package, binary, options, source_inputs, vendor_source);
    let inputs = render_inputs_ncl(source_inputs);
    vec![
        file_operation(&options.project_file, project),
        file_operation(&options.inputs_file, inputs),
    ]
}

fn file_operation(path: &str, content: String) -> CargoImportFileOperation {
    CargoImportFileOperation {
        path: path.to_string(),
        action: "write".to_string(),
        digest_blake3: blake3_hex(content.as_bytes()),
        content,
    }
}

fn validate_file_conflicts(
    operations: &mut [CargoImportFileOperation],
    existing_files: &[ExistingProjectFile],
    blockers: &mut Vec<CargoImportBlocker>,
) {
    for operation in operations {
        let Some(existing) = existing_files.iter().find(|existing| existing.path == operation.path) else {
            continue;
        };
        if existing.content == operation.content {
            operation.action = "keep-equivalent".to_string();
            continue;
        }
        blockers.push(blocker("existing-file-conflict", format!("{} exists with different content", operation.path)));
    }
}

fn render_project_ncl(
    package: &CargoPackageFact,
    binary: &str,
    options: &CargoImportOptions,
    source_inputs: &[CargoImportSourceInput],
    vendor_source: Option<&CargoImportVendorSource>,
) -> String {
    let package_field = nickel_field(&package.name);
    let source_field = source_field_name(&package.name);
    let source_name = format!("{}-src", package.name);
    let package_source_count = source_inputs
        .iter()
        .filter(|input| input.role == "package-source" && input.field == source_field && input.name == source_name)
        .count();
    debug_assert_eq!(package_source_count, 1);
    debug_assert!(source_inputs.len() <= MAX_SOURCE_INPUTS);
    let vendor_fields = render_vendor_project_fields(vendor_source);
    format!(
        r#"# Generated by `mantle import cargo --apply`.
# Build-shaped handoff: Cargo runs inside Mantle's sandbox; this is not a module layer.
let mantle = import "lib.ncl" in
let inputs = import {inputs_file} in
{{
  packages.{package_field} = mantle.offlineCargoPackage {{
    name = {package_name},
    src = inputs.{source_field},
    source_name = {source_name},
    rust = inputs.{rust_field},
    rust_name = {rust_name},
    seed_toolchain = inputs.{seed_field},
    seed_toolchain_name = {seed_name},
    musl = inputs.{musl_field},
    musl_name = {musl_name},
{vendor_fields}    binary = {binary},
    target = {target},
    profile = '{profile},
  }},
  default.package = {package_name},
}} | mantle.Project
"#,
        inputs_file = nickel_string(&options.inputs_file),
        package_field = package_field,
        package_name = nickel_string(&package.name),
        source_field = nickel_field(&source_field),
        source_name = nickel_string(&source_name),
        rust_field = nickel_field(RUST_FIELD_NAME),
        rust_name = nickel_string(RUST_INPUT_NAME),
        seed_field = nickel_field(SEED_FIELD_NAME),
        seed_name = nickel_string(SEED_INPUT_NAME),
        musl_field = nickel_field(MUSL_FIELD_NAME),
        musl_name = nickel_string(MUSL_INPUT_NAME),
        vendor_fields = vendor_fields,
        binary = nickel_string(binary),
        target = nickel_string(&options.target_triple),
        profile = options.profile,
    )
}

fn render_vendor_project_fields(vendor_source: Option<&CargoImportVendorSource>) -> String {
    let Some(vendor_source) = vendor_source else {
        return String::new();
    };
    format!(
        "    vendor_src = inputs.{},\n    vendor_name = {},\n",
        nickel_field(&vendor_source.field),
        nickel_string(&vendor_source.name)
    )
}

fn render_inputs_ncl(source_inputs: &[CargoImportSourceInput]) -> String {
    debug_assert!(!source_inputs.is_empty());
    debug_assert!(source_inputs.len() <= MAX_SOURCE_INPUTS);
    let mut fields = String::new();
    for input in source_inputs {
        fields.push_str(&format!(
            "  {} = placeholder {} {},\n",
            nickel_field(&input.field),
            nickel_string(&input.name),
            nickel_string(&format!("materialize `{}` input `{}` before running mantle build", input.role, input.name))
        ));
    }
    format!(
        r#"# Generated by `mantle import cargo --apply`.
# Replace placeholders with admitted source/toolchain derivations before building.
let mantle = import "lib.ncl" in
let placeholder = fun artifact_name message => {{
  name = artifact_name,
  builder = "/bin/sh",
  args = ["-c", "echo \"$MANTLE_PLACEHOLDER_MESSAGE\" >&2; exit 1"],
  env = {{ MANTLE_PLACEHOLDER_MESSAGE = message }},
  addressing_mode = 'input-addressed,
}} | mantle.Derivation in
{{
{fields}}}
"#
    )
}

fn source_field_name(package_name: &str) -> String {
    let mut field = package_name.replace('-', "_");
    field.push_str(SOURCE_FIELD_SUFFIX);
    field
}

fn validate_name(validation: NameValidation<'_>, blockers: &mut Vec<CargoImportBlocker>) {
    if is_valid_derivation_name(validation.value) {
        return;
    }
    blockers.push(blocker(
        validation.blocker_class,
        format!("{} must be a non-empty derivation-compatible name", validation.label),
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

fn nickel_field(value: &str) -> String {
    if is_nickel_identifier(value) {
        return value.to_string();
    }
    nickel_string(value)
}

fn is_nickel_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn nickel_string(value: &str) -> String {
    serde_json::Value::String(value.to_string()).to_string()
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn blocker(class: &str, message: impl ToString) -> CargoImportBlocker {
    CargoImportBlocker {
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn validate_root_profile(root: &Path, profile: &str) -> Result<(), RunError> {
    let manifest_path = root.join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .map_err(|error| RunError::Internal(format!("reading {}: {error}", manifest_path.display())))?;
    let table = parse_profile_table(&manifest).map_err(|error| RunError::Internal(error.to_string()))?;
    resolve_profile(&table, profile).map_err(|error| RunError::Internal(error.to_string()))?;
    Ok(())
}

fn load_workspace_facts(request: WorkspaceFactLoadRequest<'_>) -> Result<CargoWorkspaceFacts, RunError> {
    let root_text = request
        .root
        .to_str()
        .ok_or_else(|| RunError::Internal("cargo import requires a UTF-8 workspace root path".to_string()))?
        .to_string();
    let manifest_path = request.root.join("Cargo.toml");
    let manifest = read_toml_value(&manifest_path)?;
    let lock_digest = read_lock_digest(request.root);
    let lock_packages = read_lock_packages(request.root)?;
    let vendor_source = read_vendor_source_facts(request.root, &lock_packages)?;
    let package_paths = package_manifest_paths(request.root, &manifest)?;
    let package_path_count = package_paths.len();
    let mut packages = Vec::with_capacity(package_path_count);
    for package_path in package_paths {
        packages.push(load_package_fact(&package_path)?);
    }
    let existing_files = read_existing_files(request.output, &[request.project_file, request.inputs_file])?;
    let facts = CargoWorkspaceFacts {
        workspace_root: root_text,
        lockfile_digest_blake3: lock_digest,
        lock_packages,
        vendor_source,
        packages,
        existing_files,
    };
    debug_assert_eq!(facts.packages.len(), package_path_count);
    debug_assert!(facts.existing_files.len() <= GENERATED_FILE_OPERATION_COUNT);
    Ok(facts)
}

fn read_toml_value(path: &Path) -> Result<Value, RunError> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    text.parse::<Value>()
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))
}

fn read_lock_digest(root: &Path) -> Option<String> {
    std::fs::read(root.join("Cargo.lock")).ok().map(|bytes| blake3_hex(&bytes))
}

fn read_lock_packages(root: &Path) -> Result<Vec<CargoLockPackageFact>, RunError> {
    let path = root.join("Cargo.lock");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let lockfile = read_toml_value(&path)?;
    let packages = lockfile.get("package").and_then(Value::as_array).cloned().unwrap_or_default();
    let package_count = packages.len();
    let mut facts = Vec::with_capacity(package_count);
    for package in packages {
        let Some(table) = package.as_table() else {
            continue;
        };
        let Some(name) = table.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(version) = table.get("version").and_then(Value::as_str) else {
            continue;
        };
        facts.push(CargoLockPackageFact {
            name: name.to_string(),
            version: version.to_string(),
            source: table.get("source").and_then(Value::as_str).map(ToOwned::to_owned),
            checksum: table.get("checksum").and_then(Value::as_str).map(ToOwned::to_owned),
        });
    }
    facts.sort_by(|left, right| {
        (&left.name, &left.version, &left.source).cmp(&(&right.name, &right.version, &right.source))
    });
    debug_assert!(facts.len() <= package_count);
    debug_assert!(facts.windows(2).all(|window| {
        (&window[0].name, &window[0].version, &window[0].source)
            <= (&window[1].name, &window[1].version, &window[1].source)
    }));
    Ok(facts)
}

fn read_vendor_source_facts(
    root: &Path,
    lock_packages: &[CargoLockPackageFact],
) -> Result<Option<CargoVendorSourceFact>, RunError> {
    let Some((replacement_source, vendor_root, mut blockers)) = read_vendor_replacement(root)? else {
        return Ok(None);
    };
    let mut packages = Vec::new();
    if !vendor_root.is_dir() {
        blockers.push(blocker(
            "missing-vendor-directory",
            format!("declared Cargo vendor directory {} does not exist", vendor_root.display()),
        ));
    } else {
        packages = read_vendor_packages(root, &vendor_root, lock_packages, &mut blockers)?;
    }
    let root_digest_blake3 = if vendor_root.is_dir() {
        path_digest_blake3(&vendor_root).ok()
    } else {
        None
    };
    let facts = CargoVendorSourceFact {
        root: normalize_relative_path(root, &vendor_root),
        root_digest_blake3,
        replacement_source,
        packages,
        blockers,
    };
    debug_assert!(facts.packages.len() <= MAX_VENDOR_PACKAGES);
    debug_assert!(facts.root_digest_blake3.as_ref().is_none_or(|digest| digest.len() == BLAKE3_HEX_BYTES));
    Ok(Some(facts))
}

fn read_vendor_replacement(root: &Path) -> Result<Option<(String, PathBuf, Vec<CargoImportBlocker>)>, RunError> {
    let Some(config_path) = cargo_config_path(root) else {
        return Ok(None);
    };
    debug_assert!(config_path.is_file());
    debug_assert!(config_path.ends_with(CARGO_CONFIG_TOML) || config_path.ends_with(CARGO_CONFIG_LEGACY));
    let mut blockers = Vec::new();
    let config = read_toml_value(&config_path)?;
    let Some(source_table) = config.get(CARGO_SOURCE_TABLE).and_then(Value::as_table) else {
        return Ok(None);
    };
    let Some(crates_io) = source_table.get(CARGO_CRATES_IO_SOURCE).and_then(Value::as_table) else {
        return Ok(None);
    };
    let Some(replacement_source) = crates_io.get(CARGO_REPLACE_WITH_FIELD).and_then(Value::as_str) else {
        return Ok(None);
    };
    let Some(replacement) = source_table.get(replacement_source).and_then(Value::as_table) else {
        blockers.push(blocker(
            "unsupported-source-replacement",
            format!("Cargo source replacement `{replacement_source}` has no table"),
        ));
        return Ok(Some((replacement_source.to_string(), root.join("vendor"), blockers)));
    };
    let Some(directory) = replacement.get(CARGO_DIRECTORY_FIELD).and_then(Value::as_str) else {
        blockers.push(blocker(
            "unsupported-source-replacement",
            format!("Cargo source replacement `{replacement_source}` is not a local directory source"),
        ));
        return Ok(Some((replacement_source.to_string(), root.join("vendor"), blockers)));
    };
    if directory.is_empty()
        || directory.starts_with('/')
        || directory.split('/').any(|part| part.is_empty() || part == "..")
    {
        blockers.push(blocker("unsafe-vendor-path", "vendored source directory must be a safe relative path"));
        return Ok(Some((replacement_source.to_string(), root.join("vendor"), blockers)));
    }
    Ok(Some((replacement_source.to_string(), root.join(directory), blockers)))
}

fn cargo_config_path(root: &Path) -> Option<PathBuf> {
    let modern = root.join(CARGO_CONFIG_TOML);
    if modern.is_file() {
        return Some(modern);
    }
    let legacy = root.join(CARGO_CONFIG_LEGACY);
    if legacy.is_file() {
        return Some(legacy);
    }
    None
}

fn read_vendor_packages(
    root: &Path,
    vendor_root: &Path,
    lock_packages: &[CargoLockPackageFact],
    blockers: &mut Vec<CargoImportBlocker>,
) -> Result<Vec<CargoVendorPackageFact>, RunError> {
    let mut packages = Vec::with_capacity(MAX_VENDOR_PACKAGES);
    for entry in std::fs::read_dir(vendor_root)
        .map_err(|err| RunError::Internal(format!("reading vendor directory {}: {err}", vendor_root.display())))?
    {
        let entry = entry.map_err(|err| RunError::Internal(format!("reading vendor entry: {err}")))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("Cargo.toml").is_file() {
            continue;
        }
        if packages.len() >= MAX_VENDOR_PACKAGES {
            blockers.push(blocker("too-many-vendored-packages", "vendored source package count exceeds import bound"));
            break;
        }
        packages.push(read_vendor_package(root, &path, lock_packages, blockers)?);
    }
    packages.sort_by(|left, right| {
        (&left.name, &left.version, &left.directory).cmp(&(&right.name, &right.version, &right.directory))
    });
    debug_assert!(packages.len() <= MAX_VENDOR_PACKAGES);
    debug_assert!(packages.windows(2).all(|window| {
        (&window[0].name, &window[0].version, &window[0].directory)
            <= (&window[1].name, &window[1].version, &window[1].directory)
    }));
    Ok(packages)
}

fn read_vendor_package(
    root: &Path,
    package_dir: &Path,
    lock_packages: &[CargoLockPackageFact],
    blockers: &mut Vec<CargoImportBlocker>,
) -> Result<CargoVendorPackageFact, RunError> {
    let manifest_path = package_dir.join("Cargo.toml");
    let manifest = read_toml_value(&manifest_path)?;
    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .ok_or_else(|| RunError::Internal(format!("{} has no [package] table", manifest_path.display())))?;
    let name = required_string(package.get("name"), "package.name", &manifest_path)?;
    let version = optional_string(package.get("version")).unwrap_or_else(|| "0.0.0".to_string());
    let lock = lock_packages.iter().find(|lock| lock.name == name && lock.version == version);
    let checksum = lock.and_then(|lock| lock.checksum.clone());
    let checksum_status = validate_vendor_checksum(package_dir, blockers);
    let fact = CargoVendorPackageFact {
        name,
        version,
        directory: normalize_relative_path(root, package_dir),
        source: lock.and_then(|lock| lock.source.clone()),
        checksum,
        digest_blake3: path_digest_blake3(package_dir).ok(),
        checksum_status,
    };
    debug_assert_eq!(fact.directory, normalize_relative_path(root, package_dir));
    debug_assert!(fact.digest_blake3.as_ref().is_none_or(|digest| digest.len() == BLAKE3_HEX_BYTES));
    Ok(fact)
}

#[derive(Debug, Deserialize)]
struct CargoVendorChecksumFile {
    #[serde(default = "empty_vendor_checksum_files")]
    files: std::collections::BTreeMap<String, String>,
    #[serde(default = "missing_vendor_package_checksum")]
    package: Option<String>,
}

fn empty_vendor_checksum_files() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::new()
}

fn missing_vendor_package_checksum() -> Option<String> {
    None
}

fn validate_vendor_checksum(package_dir: &Path, blockers: &mut Vec<CargoImportBlocker>) -> String {
    let checksum_path = package_dir.join(CARGO_CHECKSUM_FILE);
    debug_assert!(package_dir.is_dir());
    debug_assert!(checksum_path.ends_with(CARGO_CHECKSUM_FILE));
    let text = match std::fs::read_to_string(&checksum_path) {
        Ok(text) => text,
        Err(_) => {
            blockers.push(blocker("missing-vendor-checksum", "vendored package is missing .cargo-checksum.json"));
            return "missing".to_string();
        }
    };
    let parsed: CargoVendorChecksumFile = match serde_json::from_str(&text) {
        Ok(parsed) => parsed,
        Err(_) => {
            blockers.push(blocker("malformed-vendor-checksum", "vendored package checksum metadata is malformed"));
            return "malformed".to_string();
        }
    };
    if parsed.files.is_empty() && parsed.package.is_none() {
        blockers.push(blocker("malformed-vendor-checksum", "vendored package checksum metadata is empty"));
        return "malformed".to_string();
    }
    for (relative, expected) in parsed.files {
        if !is_safe_vendor_relative_path(&relative) || !is_sha256_hex(&expected) {
            blockers.push(blocker("malformed-vendor-checksum", "vendored package checksum entry is unsafe or invalid"));
            return "malformed".to_string();
        }
        let actual = match sha256_file_hex(&package_dir.join(&relative)) {
            Ok(actual) => actual,
            Err(_) => {
                blockers.push(blocker("stale-vendor-checksum", "vendored package checksum references a missing file"));
                return "stale".to_string();
            }
        };
        if actual != expected {
            blockers.push(blocker("stale-vendor-checksum", "vendored package file checksum does not match metadata"));
            return "stale".to_string();
        }
    }
    "verified".to_string()
}

fn normalize_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn is_safe_vendor_relative_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.split('/').any(|part| part.is_empty() || part == "..")
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == CARGO_SHA256_HEX_BYTES
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn sha256_file_hex(path: &Path) -> Result<String, std::io::Error> {
    let bytes = std::fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn path_digest_blake3(path: &Path) -> Result<String, std::io::Error> {
    if path.is_file() {
        return std::fs::read(path).map(|bytes| blake3_hex(&bytes));
    }
    let mut entries = Vec::with_capacity(MAX_VENDOR_PACKAGES);
    collect_digest_entries(path, path, &mut entries)?;
    entries.sort();
    Ok(blake3_hex(entries.join("\n").as_bytes()))
}

fn collect_digest_entries(root: &Path, path: &Path, entries: &mut Vec<String>) -> Result<(), std::io::Error> {
    debug_assert!(root.is_dir());
    debug_assert!(path.starts_with(root));
    let mut directory_worklist = Vec::with_capacity(MAX_VENDOR_PACKAGES);
    directory_worklist.push(path.to_path_buf());
    let mut visited_node_count = 0usize;
    while let Some(directory) = directory_worklist.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            visited_node_count = visited_node_count.checked_add(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "vendor source digest node count overflow")
            })?;
            if visited_node_count > MAX_VENDOR_DIGEST_NODES {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("vendor source digest exceeds {MAX_VENDOR_DIGEST_NODES} nodes"),
                ));
            }
            let child = entry.path();
            let relative = child.strip_prefix(root).unwrap_or(&child).display().to_string();
            let metadata = std::fs::symlink_metadata(&child)?;
            if metadata.is_dir() {
                directory_worklist.push(child);
            } else if metadata.is_file() {
                let digest = std::fs::read(&child).map(|bytes| blake3_hex(&bytes))?;
                entries.push(format!("file:{relative}:{digest}"));
            } else if metadata.file_type().is_symlink() {
                let target = std::fs::read_link(&child)?.display().to_string();
                entries.push(format!("symlink:{relative}:{target}"));
            }
        }
    }
    Ok(())
}

fn package_manifest_paths(root: &Path, manifest: &Value) -> Result<Vec<PathBuf>, RunError> {
    if manifest.get("package").is_some() {
        return Ok(vec![root.join("Cargo.toml")]);
    }
    let Some(workspace) = manifest.get("workspace").and_then(Value::as_table) else {
        return Ok(Vec::new());
    };
    let members = workspace.get("members").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut paths = Vec::with_capacity(members.len());
    for member in members {
        let Some(member_path) = member.as_str() else {
            continue;
        };
        paths.push(root.join(member_path).join("Cargo.toml"));
    }
    Ok(paths)
}

fn load_package_fact(manifest_path: &Path) -> Result<CargoPackageFact, RunError> {
    let manifest = read_toml_value(manifest_path)?;
    debug_assert!(manifest_path.is_file());
    debug_assert!(manifest_path.ends_with("Cargo.toml"));
    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .ok_or_else(|| RunError::Internal(format!("{} has no [package] table", manifest_path.display())))?;
    let name = required_string(package.get("name"), "package.name", manifest_path)?;
    let version = optional_string(package.get("version")).unwrap_or_else(|| "0.0.0".to_string());
    let edition = optional_string(package.get("edition")).unwrap_or_else(|| "2021".to_string());
    let package_root = manifest_path
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| RunError::Internal("package root must be UTF-8".to_string()))?
        .to_string();
    let binaries = package_binaries(manifest_path, &manifest, &name);
    let dependencies = package_dependencies(&manifest);
    Ok(CargoPackageFact {
        name,
        version,
        edition,
        package_root,
        binaries,
        dependencies,
    })
}

fn required_string(value: Option<&Value>, label: &str, manifest_path: &Path) -> Result<String, RunError> {
    value
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| RunError::Internal(format!("{} must contain string field {label}", manifest_path.display())))
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Table(table) => table
            .get("workspace")
            .and_then(Value::as_bool)
            .and_then(|workspace| if workspace { Some("2021".to_string()) } else { None }),
        _ => None,
    }
}

fn package_binaries(manifest_path: &Path, manifest: &Value, package_name: &str) -> Vec<String> {
    let mut binaries = Vec::new();
    if let Some(bin_entries) = manifest.get("bin").and_then(Value::as_array) {
        for entry in bin_entries {
            if let Some(name) = entry.get("name").and_then(Value::as_str) {
                binaries.push(name.to_string());
            }
        }
    }
    if binaries.is_empty() {
        let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        if root.join("src/main.rs").is_file() {
            binaries.push(package_name.to_string());
        }
    }
    binaries.sort();
    binaries.dedup();
    binaries
}

fn package_dependencies(manifest: &Value) -> Vec<CargoDependencyFact> {
    let mut dependencies = Vec::new();
    collect_dependency_table(manifest.get("dependencies"), &mut dependencies);
    collect_dependency_table(manifest.get("build-dependencies"), &mut dependencies);
    collect_dependency_table(manifest.get("dev-dependencies"), &mut dependencies);
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    dependencies
}

fn collect_dependency_table(value: Option<&Value>, dependencies: &mut Vec<CargoDependencyFact>) {
    let Some(table) = value.and_then(Value::as_table) else {
        return;
    };
    for (name, value) in table {
        dependencies.push(CargoDependencyFact {
            name: name.clone(),
            source: dependency_source(value),
        });
    }
}

fn dependency_source(value: &Value) -> CargoDependencySource {
    match value {
        Value::String(_) => CargoDependencySource::Registry,
        Value::Table(table) => {
            if let Some(path) = table.get("path").and_then(Value::as_str) {
                return CargoDependencySource::LocalPath(path.to_string());
            }
            if let Some(git) = table.get("git").and_then(Value::as_str) {
                return CargoDependencySource::Git(git.to_string());
            }
            if table.get("version").is_some() {
                return CargoDependencySource::Registry;
            }
            CargoDependencySource::Unknown
        }
        _ => CargoDependencySource::Unknown,
    }
}

fn read_existing_files(
    output: &dyn ImportOutputCapability,
    paths: &[&str],
) -> Result<Vec<ExistingProjectFile>, RunError> {
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        if let Some(bytes) = output.read(path).map_err(import_capability_error)? {
            let content = String::from_utf8(bytes)
                .map_err(|err| RunError::Internal(format!("reading existing import file {path}: {err}")))?;
            files.push(ExistingProjectFile {
                path: (*path).to_string(),
                content,
            });
        }
    }
    Ok(files)
}

fn apply_cargo_import_plan(
    output: &dyn ImportOutputCapability,
    plan: &CargoImportPlan,
    targets: &[&str],
) -> Result<ImportApplied, ImportApplyError> {
    let mut facts = ImportApplyFacts { writes: 0, verified: 0 };
    if !plan.blockers.is_empty() {
        return Err(import_apply_error(
            ImportApplyPhase::Preflight,
            facts,
            RunError::Internal(format!("refusing to apply cargo import plan with {} blocker(s)", plan.blockers.len())),
        ));
    }
    if plan.file_operations.len() != targets.len()
        || plan.file_operations.iter().zip(targets).any(|(operation, target)| operation.path != *target)
    {
        let error = RunError::Internal("cargo import plan targets differ from declared output paths".to_string());
        return Err(import_apply_error(ImportApplyPhase::Preflight, facts, error));
    }
    debug_assert!(plan.file_operations.len() <= GENERATED_FILE_OPERATION_COUNT);
    for (index, operation) in plan.file_operations.iter().enumerate() {
        for other in plan.file_operations.iter().skip(index + 1) {
            let left = Path::new(&operation.path);
            let right = Path::new(&other.path);
            if left.starts_with(right) || right.starts_with(left) {
                let error = RunError::Internal(format!(
                    "cargo import output targets overlap: {} and {}",
                    operation.path, other.path
                ));
                return Err(import_apply_error(ImportApplyPhase::Preflight, facts, error));
            }
        }
    }
    for operation in &plan.file_operations {
        let observed = output
            .read(&operation.path)
            .map_err(|error| import_apply_error(ImportApplyPhase::Preflight, facts, import_capability_error(error)))?;
        ImportOutputPort::validate_content_size(&operation.path, operation.content.as_bytes())
            .map_err(|error| import_apply_error(ImportApplyPhase::Preflight, facts, error))?;
        match (operation.action.as_str(), observed) {
            ("write", None) => {}
            ("write", Some(bytes)) | ("keep-equivalent", Some(bytes)) if bytes == operation.content.as_bytes() => {}
            _ => {
                let error =
                    RunError::Internal(format!("cargo import output {} changed since planning", operation.path));
                return Err(import_apply_error(ImportApplyPhase::Preflight, facts, error));
            }
        }
    }
    for operation in &plan.file_operations {
        if operation.action == "write" {
            let did_write = output
                .ensure_contents(&operation.path, operation.content.as_bytes())
                .map_err(|error| import_apply_error(ImportApplyPhase::Write, facts, import_capability_error(error)))?;
            if did_write {
                facts.writes = facts.writes.checked_add(1).ok_or_else(|| {
                    import_apply_error(
                        ImportApplyPhase::Write,
                        facts,
                        RunError::Internal("cargo import write count overflow".to_string()),
                    )
                })?;
            }
        }
    }
    let observed_targets = verify_import_readback(
        output,
        plan.file_operations.iter().map(|operation| (operation.path.as_str(), operation.content.as_bytes())),
        &mut facts,
    )?;
    Ok(ImportApplied {
        facts,
        target_identity: observed_targets,
    })
}

fn emit_cargo_import_plan(plan: &CargoImportPlan, options: &CargoImportEmissionOptions) -> Result<(), RunError> {
    debug_assert_eq!(plan.schema, CARGO_IMPORT_PLAN_SCHEMA);
    debug_assert!(plan.file_operations.len() <= GENERATED_FILE_OPERATION_COUNT);
    if options.is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(plan)
                .map_err(|err| RunError::Internal(format!("serializing cargo import plan: {err}")))?
        );
        return Ok(());
    }
    if options.is_applied {
        println!("Applied Cargo import scaffold for {:?}", plan.selected_package);
    } else {
        println!("Cargo import plan for {:?}", plan.selected_package);
    }
    println!("  blockers: {}", plan.blockers.len());
    for blocker in &plan.blockers {
        println!("  - {}: {}", blocker.class, blocker.message);
    }
    for operation in &plan.file_operations {
        println!("  {} {} ({})", operation.action, operation.path, operation.digest_blake3);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const LOCK_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn package(name: &str, binaries: &[&str], dependencies: Vec<CargoDependencyFact>) -> CargoPackageFact {
        CargoPackageFact {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            package_root: "/tmp/demo".to_string(),
            binaries: binaries.iter().map(|binary| (*binary).to_string()).collect(),
            dependencies,
        }
    }

    fn local_dependency(name: &str) -> CargoDependencyFact {
        CargoDependencyFact {
            name: name.to_string(),
            source: CargoDependencySource::LocalPath("../dep".to_string()),
        }
    }

    fn registry_dependency(name: &str) -> CargoDependencyFact {
        CargoDependencyFact {
            name: name.to_string(),
            source: CargoDependencySource::Registry,
        }
    }

    fn lock_package(name: &str, version: &str, source: Option<&str>, checksum: Option<&str>) -> CargoLockPackageFact {
        CargoLockPackageFact {
            name: name.to_string(),
            version: version.to_string(),
            source: source.map(ToOwned::to_owned),
            checksum: checksum.map(ToOwned::to_owned),
        }
    }

    fn verified_vendor_source() -> CargoVendorSourceFact {
        CargoVendorSourceFact {
            root: "vendor".to_string(),
            root_digest_blake3: Some(LOCK_DIGEST.to_string()),
            replacement_source: "vendored-sources".to_string(),
            packages: vec![CargoVendorPackageFact {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                directory: "vendor/serde".to_string(),
                source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
                checksum: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()),
                digest_blake3: Some(LOCK_DIGEST.to_string()),
                checksum_status: "verified".to_string(),
            }],
            blockers: Vec::new(),
        }
    }

    fn facts(packages: Vec<CargoPackageFact>) -> CargoWorkspaceFacts {
        CargoWorkspaceFacts {
            workspace_root: "/tmp/workspace".to_string(),
            lockfile_digest_blake3: Some(LOCK_DIGEST.to_string()),
            lock_packages: vec![
                lock_package("demo", "0.1.0", None, None),
                lock_package("dep", "0.1.0", None, None),
                lock_package(
                    "serde",
                    "1.0.0",
                    Some("registry+https://github.com/rust-lang/crates.io-index"),
                    Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                ),
            ],
            vendor_source: None,
            packages,
            existing_files: Vec::new(),
        }
    }

    #[test]
    fn cargo_import_uses_shared_root_profile_resolution() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[profile.fast]\ninherits = 'release'\nopt-level = 2\n").unwrap();

        validate_root_profile(dir.path(), "fast").unwrap();
        let error = validate_root_profile(dir.path(), "missing").unwrap_err();

        assert!(error.to_string().contains("unknown-profile"));
        assert!(error.to_string().contains("missing"));
    }

    #[test]
    fn cargo_import_plan_generates_build_shaped_files_for_local_binary_workspace() {
        let plan = build_cargo_import_plan(
            facts(vec![package("demo", &["demo"], vec![local_dependency("dep")])]),
            CargoImportOptions::default(),
        );

        assert!(plan.blockers.is_empty(), "blockers: {:#?}", plan.blockers);
        assert_eq!(plan.selected_package.as_deref(), Some("demo"));
        assert_eq!(plan.selected_binary.as_deref(), Some("demo"));
        assert_eq!(plan.file_operations.len(), 2);
        assert!(plan.file_operations[0].content.contains("mantle.offlineCargoPackage"));
        assert!(plan.file_operations[0].content.contains("not a module layer"));
        assert_eq!(plan.file_operations[0].digest_blake3.len(), BLAKE3_HEX_BYTES);
        assert!(plan.non_claims.contains(&"not-cargo-free-execution".to_string()));
    }

    #[test]
    fn cargo_import_plan_blocks_unsupported_and_ambiguous_surfaces() {
        let plan = build_cargo_import_plan(
            facts(vec![
                package("demo", &["demo", "other"], vec![registry_dependency("serde")]),
                package("second", &["second"], Vec::new()),
            ]),
            CargoImportOptions::default(),
        );
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("ambiguous-default-package"), "classes: {classes:?}");
        assert!(plan.file_operations.is_empty());
        assert!(plan.source_inputs.is_empty());
    }

    #[test]
    fn cargo_import_plan_blocks_missing_lockfile_malformed_name_and_registry_dependency() {
        let mut workspace = facts(vec![package("bad/name", &["bad/name"], vec![registry_dependency("serde")])]);
        workspace.lockfile_digest_blake3 = None;
        let plan = build_cargo_import_plan(workspace, CargoImportOptions {
            selected_package: Some("bad/name".to_string()),
            selected_binary: Some("bad/name".to_string()),
            ..CargoImportOptions::default()
        });
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("missing-lockfile"), "classes: {classes:?}");
        assert!(classes.contains("malformed-package-name"), "classes: {classes:?}");
        assert!(classes.contains("malformed-binary-name"), "classes: {classes:?}");
        assert!(classes.contains("missing-vendored-source"), "classes: {classes:?}");
    }

    #[test]
    fn cargo_import_plan_accepts_verified_vendored_registry_source() {
        let mut workspace = facts(vec![package("demo", &["demo"], vec![registry_dependency("serde")])]);
        workspace.vendor_source = Some(verified_vendor_source());

        let plan = build_cargo_import_plan(workspace, CargoImportOptions::default());

        assert!(plan.blockers.is_empty(), "blockers: {:#?}", plan.blockers);
        let vendor = plan.vendor_source.as_ref().expect("vendor source should be accepted");
        assert_eq!(vendor.role, "vendored-dependencies");
        assert_eq!(vendor.field, "vendor_src");
        assert_eq!(vendor.name, "demo-vendor");
        assert!(plan.source_inputs.iter().any(|input| input.role == "vendored-dependencies"));
        assert!(plan.file_operations[0].content.contains("vendor_src = inputs.vendor_src"));
        assert!(plan.file_operations[0].content.contains("vendor_name = \"demo-vendor\""));
        assert!(plan.file_operations[1].content.contains("placeholder \"demo-vendor\""));
        assert!(plan.non_claims.contains(&"not-network-vendoring".to_string()));
        assert!(plan.non_claims.contains(&"not-full-cargo-compatibility".to_string()));
    }

    #[test]
    fn cargo_import_plan_rejects_stale_or_ambiguous_vendor_material() {
        let mut workspace = facts(vec![package("demo", &["demo"], vec![registry_dependency("serde")])]);
        let mut vendor = verified_vendor_source();
        vendor.packages[0].checksum_status = "stale".to_string();
        vendor.packages.push(vendor.packages[0].clone());
        workspace.vendor_source = Some(vendor);

        let plan = build_cargo_import_plan(workspace, CargoImportOptions::default());
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("ambiguous-vendored-package"), "classes: {classes:?}");
        assert!(plan.file_operations.is_empty());
        assert!(plan.vendor_source.is_none());
    }

    #[test]
    fn cargo_import_plan_reports_existing_file_conflicts_without_applying() {
        let mut workspace = facts(vec![package("demo", &["demo"], Vec::new())]);
        workspace.existing_files.push(ExistingProjectFile {
            path: DEFAULT_PROJECT_FILE.to_string(),
            content: "# user content\n".to_string(),
        });
        let plan = build_cargo_import_plan(workspace, CargoImportOptions::default());
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("existing-file-conflict"), "classes: {classes:?}");
        assert_eq!(plan.file_operations.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn cargo_import_shell_rejects_non_utf8_workspace_roots() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let scratch = tempfile::tempdir().unwrap();
        let invalid_root = scratch.path().join(OsString::from_vec(vec![0xff]));
        std::fs::create_dir(&invalid_root).unwrap();
        let err = run_cargo_import(CargoImportShellOptions {
            root: &invalid_root,
            selected_package: None,
            selected_binary: None,
            project_file: DEFAULT_PROJECT_FILE,
            inputs_file: DEFAULT_INPUTS_FILE,
            target_triple: DEFAULT_TARGET_TRIPLE,
            profile: DEFAULT_PROFILE,
            apply: false,
            json: true,
        })
        .unwrap_err();

        assert!(matches!(err, RunError::Internal(_)), "{err}");
        assert!(!invalid_root.join(DEFAULT_PROJECT_FILE).exists());
    }
}
