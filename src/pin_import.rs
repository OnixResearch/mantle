use std::fmt::Write as _;
use std::io::Read;
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
use std::path::Component;
use std::path::Path;

use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use cap_std::fs::OpenOptions;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use serde::Deserialize;

use crate::errors::RunError;

const DEFAULT_PINS_FILE: &str = "nixtamal-pins.json";
const MAX_PIN_IMPORT_FILE_BYTES: u64 = 1_048_576;
const MAX_PIN_IMPORT_OUTPUT_FILE_BYTES: u64 = 16 * 1_048_576;
const PIN_IMPORT_BLOCKED_EXIT_CODE: u8 = 3;
const PIN_IMPORT_READ_EFFECT: &str = "pin-import-input-read";
const PIN_IMPORT_WRITE_EFFECT: &str = "pin-import-output-write";
const PIN_IMPORT_READBACK_EFFECT: &str = "pin-import-output-readback";
const PIN_IMPORT_OUTPUT_LIMIT: u32 = 3;

pub const PIN_IMPORTER_NIXTAMAL: &str = crunch_project::PIN_IMPORT_SUPPORTED_IMPORTER;
pub const PIN_IMPORTER_FLAKE: &str = "flake";
pub const PIN_IMPORTER_NPINS: &str = "npins";
pub const PIN_IMPORTER_NIV: &str = "niv";

pub struct PinImportShellOptions<'a> {
    pub root: &'a Path,
    pub importer: &'a str,
    pub pins_file: &'a Path,
    pub project_file: &'a str,
    pub lock_file: &'a str,
    pub inputs_file: &'a str,
    pub apply: bool,
    pub json: bool,
}

impl<'a> PinImportShellOptions<'a> {
    pub fn default_pins_file() -> &'static str {
        DEFAULT_PINS_FILE
    }
}

pub(crate) trait ImportOutputCapability {
    fn readback(&self, relative: &str) -> Result<Option<(Vec<u8>, String)>, CapabilityError>;
    fn read(&self, relative: &str) -> Result<Option<Vec<u8>>, CapabilityError>;
    fn ensure_contents(&self, relative: &str, bytes: &[u8]) -> Result<bool, CapabilityError>;
}

pub(crate) fn import_capability_error(error: CapabilityError) -> RunError {
    RunError::Internal(format!("{}: {}", error.code, error.detail))
}
pub(crate) fn push_import_target(identity: &mut String, path: &str) {
    let _ = write!(identity, "{}:", path.len());
    identity.push_str(path);
    identity.push(';');
}

pub(crate) fn import_targets_identity(paths: &[&str]) -> String {
    let mut identity = String::new();
    for path in paths {
        push_import_target(&mut identity, path);
    }
    identity
}

#[derive(Debug, Deserialize)]
struct NixtamalFixture {
    #[serde(default = "empty_list")]
    unsupported_semantics: Vec<String>,
    #[serde(default = "empty_list")]
    inputs: Vec<NixtamalInput>,
    #[serde(default = "empty_list")]
    patches: Vec<NixtamalPatch>,
}

#[derive(Debug, Deserialize)]
struct NixtamalInput {
    name: String,
    #[serde(alias = "source_kind")]
    kind: String,
    #[serde(default = "absent_value")]
    url: Option<String>,
    #[serde(default = "absent_value")]
    repository: Option<String>,
    #[serde(default = "absent_value")]
    reference: Option<String>,
    #[serde(default = "absent_value")]
    rev: Option<String>,
    #[serde(default = "default_hash_algo")]
    hash_algo: String,
    #[serde(default = "absent_value", alias = "expected_hash")]
    hash: Option<String>,
    #[serde(default)]
    frozen: bool,
    #[serde(default = "empty_list")]
    mirrors: Vec<String>,
    #[serde(default = "empty_list")]
    patches: Vec<String>,
    #[serde(default = "absent_value")]
    freshness: Option<String>,
    #[serde(default = "absent_value")]
    fetch_policy: Option<String>,
    #[serde(default = "absent_value")]
    trust_policy: Option<String>,
    #[serde(default = "empty_list")]
    composition_semantics: Vec<String>,
    #[serde(default = "absent_value")]
    lock_identity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NixtamalPatch {
    name: String,
    #[serde(alias = "source_kind")]
    kind: String,
    #[serde(default = "absent_value")]
    path: Option<String>,
    #[serde(default = "absent_value")]
    url: Option<String>,
    #[serde(default = "default_hash_algo")]
    hash_algo: String,
    #[serde(default = "absent_value", alias = "expected_hash")]
    hash: Option<String>,
}

pub(crate) fn import_effect_plan(
    read_id: &'static str,
    write_id: &'static str,
    readback_id: &'static str,
    verified_output: &str,
    read_calls: u32,
    output_limit: u32,
    apply: bool,
) -> Result<EffectPlan, RunError> {
    let specs = [
        EffectSpec {
            effect_id: read_id,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Calls(read_calls),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: write_id,
            kind: EffectKind::WriteFiles,
            limit: EffectMeasure::Items(output_limit),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: readback_id,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Items(output_limit),
            expected_output: ExpectedOutput::Identity(verified_output),
        },
    ];
    let selected = if apply { &specs[..] } else { &specs[..1] };
    plan_effects(CommandFamily::SourceProvenance, selected)
        .map_err(|err| RunError::Internal(format!("planning import effects: {}", err.code())))
}

pub(crate) fn import_observation(
    effect_id: &str,
    kind: EffectKind,
    status: ObservationStatus,
    usage: EffectMeasure,
    output: EffectOutput,
) -> Observation {
    Observation {
        effect_id: EffectId(effect_id.to_string()),
        kind,
        status,
        usage,
        output,
        diagnostics_code: None,
    }
}

pub(crate) fn classify_import_effects(plan: &EffectPlan, observations: &[Observation]) -> Result<(), RunError> {
    match classify_observations(plan, observations) {
        ApplicationOutcome::Completed => Ok(()),
        result => Err(RunError::Internal(format!("import effect observation contradicted plan: {result:?}"))),
    }
}
pub(crate) fn classify_blocked_import_effects(plan: &EffectPlan, read: &Observation) -> Result<(), RunError> {
    if plan.effects.len() == 1 {
        return match classify_observations(plan, std::slice::from_ref(read)) {
            ApplicationOutcome::Failed { .. } => Ok(()),
            result => {
                Err(RunError::Internal(format!("import blocked effect observation contradicted plan: {result:?}")))
            }
        };
    }
    let mut observations = Vec::with_capacity(plan.effects.len());
    observations.push(read.clone());
    for effect in plan.effects.iter().skip(1) {
        let usage = match effect.limit {
            EffectMeasure::Calls(_) => EffectMeasure::Calls(0),
            EffectMeasure::Items(_) => EffectMeasure::Items(0),
            EffectMeasure::Bytes(_) => EffectMeasure::Bytes(0),
        };
        observations.push(import_observation(
            &effect.effect_id.0,
            effect.kind,
            ObservationStatus::Skipped,
            usage,
            EffectOutput::None,
        ));
    }
    match classify_observations(plan, &observations) {
        ApplicationOutcome::Failed { .. } => Ok(()),
        result => Err(RunError::Internal(format!("import blocked effect observation contradicted plan: {result:?}"))),
    }
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum ImportApplyPhase {
    Preflight,
    Write,
    Readback,
}

#[derive(Debug)]
pub(crate) struct ImportApplyError {
    pub phase: ImportApplyPhase,
    pub facts: ImportApplyFacts,
    pub error: RunError,
}

pub(crate) fn import_apply_error(
    phase: ImportApplyPhase,
    facts: ImportApplyFacts,
    error: RunError,
) -> ImportApplyError {
    ImportApplyError { phase, facts, error }
}

pub(crate) fn classify_import_failure(plan: &EffectPlan, read: Observation, failure: ImportApplyError) -> RunError {
    let write_status = match failure.phase {
        ImportApplyPhase::Preflight => ObservationStatus::Skipped,
        ImportApplyPhase::Write => ObservationStatus::Failed,
        ImportApplyPhase::Readback => ObservationStatus::Succeeded,
    };
    let readback_status = match failure.phase {
        ImportApplyPhase::Readback => ObservationStatus::Failed,
        ImportApplyPhase::Preflight | ImportApplyPhase::Write => ObservationStatus::Skipped,
    };
    let observations = [
        read,
        import_observation(
            &plan.effects[1].effect_id.0,
            EffectKind::WriteFiles,
            write_status,
            EffectMeasure::Items(failure.facts.writes),
            EffectOutput::None,
        ),
        import_observation(
            &plan.effects[2].effect_id.0,
            EffectKind::ReadFiles,
            readback_status,
            EffectMeasure::Items(failure.facts.verified),
            EffectOutput::None,
        ),
    ];
    match classify_observations(plan, &observations) {
        ApplicationOutcome::Failed { .. } => failure.error,
        other => RunError::Internal(format!("import failed effect observation contradicted plan: {other:?}")),
    }
}

pub fn run_pin_import(options: PinImportShellOptions<'_>) -> Result<(), RunError> {
    let plan_options = crunch_project::PinImportOptions {
        importer: options.importer.to_string(),
        project_file: options.project_file.to_string(),
        lock_file: options.lock_file.to_string(),
        inputs_file: options.inputs_file.to_string(),
    };
    let targets = [options.project_file, options.lock_file, options.inputs_file];
    let target_identity = import_targets_identity(&targets);
    let effects = import_effect_plan(
        PIN_IMPORT_READ_EFFECT,
        PIN_IMPORT_WRITE_EFFECT,
        PIN_IMPORT_READBACK_EFFECT,
        &target_identity,
        1,
        PIN_IMPORT_OUTPUT_LIMIT,
        options.apply,
    )?;
    let output = ImportOutputPort::open(options.root)?;
    let input = FilesystemPinImportPort { output: &output };
    let (pin_set, read_calls) =
        input.load(options.root, options.pins_file, &plan_options).map_err(import_capability_error)?;
    let plan = crunch_project::build_pin_import_plan(pin_set, plan_options);
    let read = import_observation(
        PIN_IMPORT_READ_EFFECT,
        EffectKind::ReadFiles,
        if !plan.can_apply() {
            ObservationStatus::Failed
        } else if read_calls == 0 {
            ObservationStatus::Skipped
        } else {
            ObservationStatus::Succeeded
        },
        EffectMeasure::Calls(read_calls),
        EffectOutput::None,
    );
    debug_assert_eq!(plan.importer, options.importer);
    debug_assert_eq!(plan.source_label, options.pins_file.display().to_string());

    if !options.apply {
        if plan.can_apply() {
            classify_import_effects(&effects, std::slice::from_ref(&read))?;
        } else {
            classify_blocked_import_effects(&effects, &read)?;
        }
    }
    if options.apply {
        if !plan.can_apply() {
            classify_blocked_import_effects(&effects, &read)?;
            emit_pin_import_plan(&plan, PinImportDisplay {
                is_applied: false,
                is_json: options.json,
            })?;
            return Err(RunError::Reported(PIN_IMPORT_BLOCKED_EXIT_CODE));
        }
        let applied = match apply_pin_import_plan(&output, &plan, &targets) {
            Ok(facts) => facts,
            Err(failure) => return Err(classify_import_failure(&effects, read, failure)),
        };
        let observations = [
            read,
            import_observation(
                PIN_IMPORT_WRITE_EFFECT,
                EffectKind::WriteFiles,
                ObservationStatus::Succeeded,
                EffectMeasure::Items(applied.facts.writes),
                EffectOutput::None,
            ),
            import_observation(
                PIN_IMPORT_READBACK_EFFECT,
                EffectKind::ReadFiles,
                ObservationStatus::Succeeded,
                EffectMeasure::Items(applied.facts.verified),
                EffectOutput::Identity(applied.target_identity),
            ),
        ];
        classify_import_effects(&effects, &observations)?;
        emit_pin_import_plan(&plan, PinImportDisplay {
            is_applied: true,
            is_json: options.json,
        })?;
        return Ok(());
    }

    emit_pin_import_plan(&plan, PinImportDisplay {
        is_applied: false,
        is_json: options.json,
    })?;
    if plan.can_apply() {
        return Ok(());
    }
    Err(RunError::Reported(PIN_IMPORT_BLOCKED_EXIT_CODE))
}

trait PinImportInputPort {
    fn load(
        &self,
        root: &Path,
        pins_file: &Path,
        options: &crunch_project::PinImportOptions,
    ) -> Result<(crunch_project::ExternalPinSet, u32), CapabilityError>;
}

struct FilesystemPinImportPort<'a> {
    output: &'a dyn ImportOutputCapability,
}

impl PinImportInputPort for FilesystemPinImportPort<'_> {
    fn load(
        &self,
        root: &Path,
        pins_file: &Path,
        options: &crunch_project::PinImportOptions,
    ) -> Result<(crunch_project::ExternalPinSet, u32), CapabilityError> {
        load_pin_set(root, pins_file, options, self.output)
            .map_err(|error| CapabilityError::new("pin-import-input-read", error.message()))
    }
}

fn load_pin_set(
    root: &Path,
    pins_file: &Path,
    options: &crunch_project::PinImportOptions,
    output: &dyn ImportOutputCapability,
) -> Result<(crunch_project::ExternalPinSet, u32), RunError> {
    if options.importer != PIN_IMPORTER_NIXTAMAL {
        return Ok((future_adapter_pin_set(pins_file, &options.importer), 0));
    }
    let fixture = load_nixtamal_fixture(root, pins_file)?;
    let existing_files =
        read_existing_files(output, &[&options.project_file, &options.lock_file, &options.inputs_file])?;
    Ok((nixtamal_fixture_to_pin_set(fixture, pins_file, existing_files), 1))
}

fn future_adapter_pin_set(pins_file: &Path, importer: &str) -> crunch_project::ExternalPinSet {
    crunch_project::ExternalPinSet {
        importer: importer.to_string(),
        source_label: pins_file.display().to_string(),
        pins: Vec::new(),
        patches: Vec::new(),
        existing_files: Vec::new(),
        unsupported_semantics: vec!["future adapter is intentionally blocker-only in this change".to_string()],
    }
}

fn load_nixtamal_fixture(root: &Path, pins_file: &Path) -> Result<NixtamalFixture, RunError> {
    let path = root.join(pins_file);
    let file = std::fs::File::open(&path)
        .map_err(|err| RunError::Internal(format!("reading Nixtamal fixture {}: {err}", path.display())))?;
    let mut bytes = Vec::new();
    file.take(MAX_PIN_IMPORT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| RunError::Internal(format!("reading Nixtamal fixture {}: {err}", path.display())))?;
    if bytes.len() as u64 > MAX_PIN_IMPORT_FILE_BYTES {
        return Err(RunError::Internal(format!("{} exceeds pin import fixture size limit", path.display())));
    }
    serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing Nixtamal fixture {}: {err}", path.display())))
}

fn read_existing_files(
    output: &dyn ImportOutputCapability,
    paths: &[&str],
) -> Result<Vec<crunch_project::ExternalExistingFile>, RunError> {
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        if let Some(bytes) = output.read(path).map_err(import_capability_error)? {
            let content = String::from_utf8(bytes)
                .map_err(|err| RunError::Internal(format!("reading existing import file {path}: {err}")))?;
            files.push(crunch_project::ExternalExistingFile {
                path: (*path).to_string(),
                content,
            });
        }
    }
    Ok(files)
}

type CheckedImportOutput = (Vec<u8>, Option<String>);

/// A single, directory-descriptor-rooted output capability shared by plan
/// reads, writes, and the independent post-write read-back.
pub(crate) struct ImportOutputPort {
    root: Dir,
}

impl ImportOutputPort {
    pub(crate) fn open(root: &Path) -> Result<Self, RunError> {
        let absolute = std::path::absolute(root)
            .map_err(|err| RunError::Internal(format!("resolving import root {}: {err}", root.display())))?;
        let mut directory = Dir::open_ambient_dir("/", ambient_authority())
            .map_err(|err| RunError::Internal(format!("opening import root anchor: {err}")))?;
        for component in absolute.components() {
            match component {
                Component::Normal(name) => {
                    directory = directory.open_dir_nofollow(name).map_err(|err| {
                        RunError::Internal(format!("opening no-follow import root {}: {err}", absolute.display()))
                    })?;
                }
                Component::ParentDir => {
                    return Err(RunError::Internal(format!(
                        "refusing import root with parent traversal: {}",
                        absolute.display()
                    )));
                }
                Component::RootDir | Component::CurDir | Component::Prefix(_) => {}
            }
        }
        Ok(Self { root: directory })
    }

    fn parent<'a>(&self, relative: &'a str, create: bool) -> Result<Option<(Dir, &'a str)>, RunError> {
        if !is_safe_relative_path(relative) {
            return Err(RunError::Internal(format!("refusing unsafe import output path {relative}")));
        }
        let (parent_path, file_name) = relative.rsplit_once('/').unwrap_or(("", relative));
        let mut directory = self
            .root
            .try_clone()
            .map_err(|err| RunError::Internal(format!("cloning import root capability: {err}")))?;
        for component in Path::new(parent_path).components() {
            let name = component.as_os_str();
            match directory.symlink_metadata(name) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(RunError::Internal(format!("refusing symlink import output parent {relative}")));
                }
                Ok(metadata) if !metadata.is_dir() => {
                    return Err(RunError::Internal(format!("import output parent is not a directory: {relative}")));
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound && !create => return Ok(None),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    directory
                        .create_dir(name)
                        .or_else(|err| {
                            if err.kind() == std::io::ErrorKind::AlreadyExists {
                                Ok(())
                            } else {
                                Err(err)
                            }
                        })
                        .map_err(|err| {
                            RunError::Internal(format!("creating import output parent {relative}: {err}"))
                        })?;
                }
                Err(err) => {
                    return Err(RunError::Internal(format!("inspecting import output parent {relative}: {err}")));
                }
                Ok(_) => {}
            }
            directory = directory.open_dir_nofollow(name).map_err(|err| {
                RunError::Internal(format!("opening no-follow import output parent {relative}: {err}"))
            })?;
        }
        Ok(Some((directory, file_name)))
    }

    pub(crate) fn read(&self, relative: &str) -> Result<Option<Vec<u8>>, RunError> {
        Ok(self.read_checked(relative, false)?.map(|(bytes, _)| bytes))
    }

    fn readback(&self, relative: &str) -> Result<Option<(Vec<u8>, String)>, RunError> {
        self.read_checked(relative, true)?
            .map(|(bytes, path)| {
                path.map(|path| (bytes, path))
                    .ok_or_else(|| RunError::Internal(format!("import read-back target missing for {relative}")))
            })
            .transpose()
    }

    fn read_checked(&self, relative: &str, observe_path: bool) -> Result<Option<CheckedImportOutput>, RunError> {
        let Some((parent, name)) = self.parent(relative, false)? else {
            return Ok(None);
        };
        let metadata = match parent.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(RunError::Internal(format!("inspecting import output {relative}: {err}"))),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(RunError::Internal(format!("refusing symlink or non-file import output {relative}")));
        }
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        let mut file = parent
            .open_with(name, &options)
            .map_err(|err| RunError::Internal(format!("opening no-follow import output {relative}: {err}")))?;
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_PIN_IMPORT_OUTPUT_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|err| RunError::Internal(format!("reading import output {relative}: {err}")))?;
        if bytes.len() as u64 > MAX_PIN_IMPORT_OUTPUT_FILE_BYTES {
            return Err(RunError::Internal(format!("import output {relative} exceeds file size limit")));
        }
        let opened_path = if observe_path {
            Some(self.opened_target(&file)?)
        } else {
            None
        };
        Ok(Some((bytes, opened_path)))
    }

    #[cfg(target_os = "linux")]
    fn opened_target(&self, file: &cap_std::fs::File) -> Result<String, RunError> {
        let root = std::fs::read_link(format!("/proc/self/fd/{}", self.root.as_raw_fd()))
            .map_err(|error| RunError::Internal(format!("observing import root descriptor: {error}")))?;
        let opened = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
            .map_err(|error| RunError::Internal(format!("observing import output descriptor: {error}")))?;
        let relative = opened
            .strip_prefix(root)
            .map_err(|_| RunError::Internal(format!("observed import output outside root: {}", opened.display())))?;
        let path = relative
            .to_str()
            .ok_or_else(|| RunError::Internal("observed import output path is not UTF-8".to_string()))?;
        if !is_safe_relative_path(path) {
            return Err(RunError::Internal(format!("observed unsafe import output path: {path}")));
        }
        Ok(path.to_owned())
    }

    #[cfg(not(target_os = "linux"))]
    fn opened_target(&self, _file: &cap_std::fs::File) -> Result<String, RunError> {
        Err(RunError::Internal("independent import output target observation requires Linux".to_string()))
    }

    pub(crate) fn validate_content_size(relative: &str, bytes: &[u8]) -> Result<(), RunError> {
        if bytes.len() as u64 > MAX_PIN_IMPORT_OUTPUT_FILE_BYTES {
            return Err(RunError::Internal(format!("import output {relative} exceeds file size limit")));
        }
        Ok(())
    }

    fn write_new(&self, relative: &str, bytes: &[u8]) -> Result<(), RunError> {
        Self::validate_content_size(relative, bytes)?;
        let (parent, name) = self
            .parent(relative, true)?
            .ok_or_else(|| RunError::Internal(format!("import output parent missing for {relative}")))?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).follow(FollowSymlinks::No);
        let mut file = parent.open_with(name, &options).map_err(|err| {
            RunError::Internal(format!(
                "creating no-follow import output {relative}: {err}; prior import writes may remain"
            ))
        })?;
        file.write_all(bytes).and_then(|()| file.flush()).map_err(|err| {
            RunError::Internal(format!("writing import output {relative}: {err}; import may be partial"))
        })
    }

    pub(crate) fn ensure_contents(&self, relative: &str, bytes: &[u8]) -> Result<bool, RunError> {
        match self.read(relative)? {
            Some(existing) if existing == bytes => Ok(false),
            Some(_) => Err(RunError::Internal(format!(
                "import output {relative} changed since planning; import may be partial"
            ))),
            None => {
                self.write_new(relative, bytes)?;
                Ok(true)
            }
        }
    }
}

impl ImportOutputCapability for ImportOutputPort {
    fn readback(&self, relative: &str) -> Result<Option<(Vec<u8>, String)>, CapabilityError> {
        ImportOutputPort::readback(self, relative)
            .map_err(|error| CapabilityError::new("import-output-readback", error.message()))
    }

    fn read(&self, relative: &str) -> Result<Option<Vec<u8>>, CapabilityError> {
        ImportOutputPort::read(self, relative)
            .map_err(|error| CapabilityError::new("import-output-read", error.message()))
    }

    fn ensure_contents(&self, relative: &str, bytes: &[u8]) -> Result<bool, CapabilityError> {
        ImportOutputPort::ensure_contents(self, relative, bytes)
            .map_err(|error| CapabilityError::new("import-output-write", error.message()))
    }
}

fn is_safe_relative_path(relative: &str) -> bool {
    let path = Path::new(relative);
    if relative.is_empty() || path.is_absolute() || relative.split('/').any(str::is_empty) {
        return false;
    }
    path.components().all(|component| matches!(component, Component::Normal(_)))
}

fn nixtamal_fixture_to_pin_set(
    fixture: NixtamalFixture,
    pins_file: &Path,
    existing_files: Vec<crunch_project::ExternalExistingFile>,
) -> crunch_project::ExternalPinSet {
    crunch_project::ExternalPinSet {
        importer: PIN_IMPORTER_NIXTAMAL.to_string(),
        source_label: pins_file.display().to_string(),
        pins: fixture.inputs.into_iter().map(nixtamal_input_to_external_pin).collect(),
        patches: fixture.patches.into_iter().map(nixtamal_patch_to_external_patch).collect(),
        existing_files,
        unsupported_semantics: fixture.unsupported_semantics,
    }
}

fn nixtamal_input_to_external_pin(input: NixtamalInput) -> crunch_project::ExternalPin {
    let expected_name = input.name.clone();
    let expected_hash_algo = input.hash_algo.clone();
    let kind = nixtamal_input_kind(&input);
    let pin = crunch_project::ExternalPin {
        name: input.name,
        kind,
        hash: crunch_project::ExternalHash {
            algo: input.hash_algo,
            value: input.hash,
        },
        frozen: input.frozen,
        mirrors: input.mirrors,
        patches: input.patches,
        metadata: crunch_project::ExternalPinMetadata {
            freshness: input.freshness,
            fetch_policy: input.fetch_policy,
            trust_policy: input.trust_policy,
            composition_semantics: input.composition_semantics,
        },
        lock_identity: input.lock_identity,
    };
    debug_assert_eq!(pin.name, expected_name);
    debug_assert_eq!(pin.hash.algo, expected_hash_algo);
    pin
}

fn nixtamal_input_kind(input: &NixtamalInput) -> crunch_project::ExternalPinKind {
    match input.kind.as_str() {
        "file" => crunch_project::ExternalPinKind::File {
            url: input.url.clone().unwrap_or_default(),
        },
        "tarball" | "archive" => crunch_project::ExternalPinKind::Tarball {
            url: input.url.clone().unwrap_or_default(),
        },
        "git" => crunch_project::ExternalPinKind::Git {
            repository: input.repository.clone().unwrap_or_default(),
            reference: input.reference.clone(),
            rev: input.rev.clone(),
        },
        other => crunch_project::ExternalPinKind::Unsupported {
            kind: other.to_string(),
        },
    }
}

fn nixtamal_patch_to_external_patch(patch: NixtamalPatch) -> crunch_project::ExternalPatch {
    let source = nixtamal_patch_source(&patch);
    crunch_project::ExternalPatch {
        name: patch.name,
        source,
        hash: crunch_project::ExternalHash {
            algo: patch.hash_algo,
            value: patch.hash,
        },
    }
}

fn nixtamal_patch_source(patch: &NixtamalPatch) -> crunch_project::ExternalPatchSource {
    match patch.kind.as_str() {
        "local" => crunch_project::ExternalPatchSource::Local {
            path: patch.path.clone().unwrap_or_default(),
        },
        "remote" => crunch_project::ExternalPatchSource::Remote {
            url: patch.url.clone().unwrap_or_default(),
        },
        other => crunch_project::ExternalPatchSource::Unsupported {
            kind: other.to_string(),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ImportApplyFacts {
    pub writes: u32,
    pub verified: u32,
}

pub(crate) struct ImportApplied {
    pub facts: ImportApplyFacts,
    pub target_identity: String,
}

pub(crate) fn verify_import_readback<'a>(
    output: &dyn ImportOutputCapability,
    files: impl Iterator<Item = (&'a str, &'a [u8])>,
    facts: &mut ImportApplyFacts,
) -> Result<String, ImportApplyError> {
    let mut observed_targets = String::new();
    for (path, expected) in files {
        let observed = output
            .readback(path)
            .map_err(|error| import_apply_error(ImportApplyPhase::Readback, *facts, import_capability_error(error)))?
            .ok_or_else(|| {
                import_apply_error(
                    ImportApplyPhase::Readback,
                    *facts,
                    RunError::Internal(format!("import output {path} missing on read-back; import may be partial")),
                )
            })?;
        let (observed_bytes, observed_path) = observed;
        if observed_path != path || observed_bytes != expected {
            let error = RunError::Internal(format!(
                "import output {path} differs from planned path or bytes; import may be partial"
            ));
            return Err(import_apply_error(ImportApplyPhase::Readback, *facts, error));
        }
        push_import_target(&mut observed_targets, &observed_path);
        facts.verified = facts.verified.checked_add(1).ok_or_else(|| {
            import_apply_error(
                ImportApplyPhase::Readback,
                *facts,
                RunError::Internal("import read-back count overflow".to_string()),
            )
        })?;
    }
    Ok(observed_targets)
}

fn apply_pin_import_plan(
    output: &dyn ImportOutputCapability,
    plan: &crunch_project::PinImportPlan,
    targets: &[&str],
) -> Result<ImportApplied, ImportApplyError> {
    let mut facts = ImportApplyFacts { writes: 0, verified: 0 };
    if !plan.can_apply() {
        return Err(import_apply_error(
            ImportApplyPhase::Preflight,
            facts,
            RunError::Internal("cannot apply a blocked pin import plan".to_string()),
        ));
    }
    if plan.file_operations.len() != targets.len()
        || plan.file_operations.iter().zip(targets).any(|(operation, target)| operation.path != *target)
    {
        let error = RunError::Internal("pin import plan targets differ from declared output paths".to_string());
        return Err(import_apply_error(ImportApplyPhase::Preflight, facts, error));
    }
    // Reject all known target conflicts before any write; path resolution
    // itself remains relative to the same no-follow directory capability.
    for (index, operation) in plan.file_operations.iter().enumerate() {
        for other in plan.file_operations.iter().skip(index + 1) {
            let left = Path::new(&operation.path);
            let right = Path::new(&other.path);
            if left.starts_with(right) || right.starts_with(left) {
                let error = RunError::Internal(format!(
                    "pin import output targets overlap: {} and {}",
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
                let error = RunError::Internal(format!("pin import output {} changed since planning", operation.path));
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
                        RunError::Internal("pin import write count overflow".to_string()),
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

#[derive(Clone, Copy)]
struct PinImportDisplay {
    is_applied: bool,
    is_json: bool,
}

fn emit_pin_import_plan(plan: &crunch_project::PinImportPlan, display: PinImportDisplay) -> Result<(), RunError> {
    debug_assert!(!display.is_applied || plan.can_apply());
    debug_assert!(plan.file_operations.len() <= plan.file_operations.capacity());
    if display.is_json {
        let rendered = serde_json::to_string_pretty(plan)
            .map_err(|err| RunError::Internal(format!("rendering pin import plan: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    let mode = if display.is_applied { "applied" } else { "plan" };
    println!("pin import {mode}: {} ({})", plan.importer, plan.source_label);
    println!("planned files:");
    for operation in &plan.file_operations {
        println!("  {} {} {}", operation.action, operation.path, operation.digest_blake3);
    }
    println!("mapped inputs: {}", plan.mapped_inputs.len());
    for input in &plan.mapped_inputs {
        println!("  {} {} {}", input.name, input.kind, input.hash_algo);
    }
    println!("mapped patches: {}", plan.mapped_patches.len());
    for patch in &plan.mapped_patches {
        println!("  {} {} {}", patch.name, patch.source_kind, patch.hash_algo);
    }
    if !plan.blockers.is_empty() {
        println!("blockers:");
        for blocker in &plan.blockers {
            println!("  [{}] {}: {}", blocker.class, blocker.subject, blocker.message);
        }
    }
    println!("non-claims:");
    for non_claim in &plan.non_claims {
        println!("  {non_claim}");
    }
    Ok(())
}

fn empty_list<T>() -> Vec<T> {
    Vec::new()
}

fn absent_value<T>() -> Option<T> {
    None
}

fn default_hash_algo() -> String {
    "sha256".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_adapter_pin_set_is_blocker_only() {
        let options = crunch_project::PinImportOptions {
            importer: PIN_IMPORTER_NPINS.to_string(),
            ..crunch_project::PinImportOptions::default()
        };

        let output = ImportOutputPort::open(Path::new(".")).unwrap();
        let (pin_set, _) = load_pin_set(Path::new("."), Path::new("missing.json"), &options, &output).unwrap();
        let plan = crunch_project::build_pin_import_plan(pin_set, options);

        assert!(!plan.can_apply());
        assert!(plan.blockers.iter().any(|blocker| blocker.class == "future-adapter"));
        assert!(plan.file_operations.is_empty());
    }

    #[test]
    fn relative_path_guard_rejects_absolute_and_parent_traversal() {
        assert!(is_safe_relative_path(".mantle/inputs.ncl"));
        assert!(!is_safe_relative_path("/tmp/out"));
        assert!(!is_safe_relative_path("../mantle.lock"));
    }

    #[cfg(target_os = "linux")]
    struct MisroutedOutput<'a>(&'a ImportOutputPort);

    #[cfg(target_os = "linux")]
    impl ImportOutputCapability for MisroutedOutput<'_> {
        fn readback(&self, _relative: &str) -> Result<Option<(Vec<u8>, String)>, CapabilityError> {
            <ImportOutputPort as ImportOutputCapability>::readback(self.0, "other.ncl")
        }

        fn read(&self, relative: &str) -> Result<Option<Vec<u8>>, CapabilityError> {
            <ImportOutputPort as ImportOutputCapability>::read(self.0, relative)
        }

        fn ensure_contents(&self, relative: &str, bytes: &[u8]) -> Result<bool, CapabilityError> {
            <ImportOutputPort as ImportOutputCapability>::ensure_contents(self.0, relative, bytes)
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn readback_checks_real_target_and_bytes_not_just_file_count() {
        let dir = tempfile::tempdir().unwrap();
        let output = ImportOutputPort::open(dir.path()).unwrap();
        output.write_new("reviewed.ncl", b"reviewed bytes").unwrap();
        output.write_new("other.ncl", b"reviewed bytes").unwrap();
        let mut facts = ImportApplyFacts { writes: 1, verified: 0 };

        let wrong_bytes =
            verify_import_readback(&output, [("reviewed.ncl", &b"changed bytes"[..])].into_iter(), &mut facts)
                .unwrap_err();
        assert!(wrong_bytes.error.message().contains("differs from planned path or bytes"));
        assert_eq!(facts.verified, 0);

        let wrong_target = verify_import_readback(
            &MisroutedOutput(&output),
            [("reviewed.ncl", &b"reviewed bytes"[..])].into_iter(),
            &mut facts,
        )
        .unwrap_err();
        assert!(wrong_target.error.message().contains("differs from planned path or bytes"));
        assert_eq!(facts.verified, 0);

        let observed =
            verify_import_readback(&output, [("reviewed.ncl", &b"reviewed bytes"[..])].into_iter(), &mut facts)
                .unwrap();
        assert_eq!(observed, import_targets_identity(&["reviewed.ncl"]));
        assert_eq!(facts.verified, 1);
    }
}
