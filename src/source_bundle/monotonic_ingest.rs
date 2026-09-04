use std::fs;
use std::io::BufReader;
#[cfg(not(target_os = "linux"))]
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_source_core::SourceIngestDisposition;
use crunch_source_core::SourceIngestPlan;
use crunch_source_core::SourceIngestRequest;
use crunch_source_core::SourceObservationWire;
use crunch_source_core::SourceRecordFact;
use crunch_source_core::plan_source_ingest;
use crunch_source_core::validate_source_ingest_plan;
#[cfg(target_os = "linux")]
use durable_file_publication::core::CleanupDisposition;
#[cfg(target_os = "linux")]
use durable_file_publication::core::DurabilityMode;
#[cfg(target_os = "linux")]
use durable_file_publication::core::PublicationDisposition as DurablePublicationDisposition;
#[cfg(target_os = "linux")]
use durable_file_publication::core::PublicationRequest;
#[cfg(target_os = "linux")]
use durable_file_publication::core::ReplacementMode;
#[cfg(target_os = "linux")]
use durable_file_publication::shell::LinuxDirectory;
#[cfg(target_os = "linux")]
use durable_file_publication::shell::StageNameSource;
#[cfg(target_os = "linux")]
use durable_file_publication::shell::publish_one_file;
use serde::Serialize;

use super::MAX_SOURCE_RECORDS;
use super::SOURCE_BUNDLE_NON_CLAIM;
use super::SourceBundleImportReport;
use super::SourceBundleManifest;
use super::SourceObservationImportSummary;
use super::SourceRecord;
use super::read_imported_source_records;
use super::source_observation_adapter::ProjectedSourceRecord;
use super::source_observation_adapter::project_source_record;
use super::source_observations_dir;
use super::source_pins_dir;
use super::source_records_dir;
use super::summary_for_record;
use crate::errors::RunError;

#[cfg(not(target_os = "linux"))]
const SOURCE_INGEST_STAGE_EXTENSION: &str = "source-ingest-stage";
#[cfg(target_os = "linux")]
const SOURCE_PUBLICATION_COLLISION_ATTEMPTS_MAX: usize = 8;
#[cfg(target_os = "linux")]
const SOURCE_PUBLICATION_FILE_MODE: u32 = 0o600;
#[cfg(target_os = "linux")]
const SOURCE_PUBLICATION_ANCESTOR_SYNC_COUNT: usize = 3;
#[cfg(target_os = "linux")]
const SOURCE_INGEST_SERIALIZED_BYTES_MAX: u64 = 2_199_023_255_552;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceIngestHookPoint {
    BeforeRecord(u32),
    AfterObservation(u32),
    AfterRecord(u32),
    BeforePin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PublishDisposition {
    Created,
    ExistingEqual,
}

#[cfg(target_os = "linux")]
struct SourceStageNames {
    identity: String,
    next_index: usize,
}

#[cfg(target_os = "linux")]
impl StageNameSource for SourceStageNames {
    fn next_stage_leaf(&mut self) -> String {
        assert!(self.next_index < SOURCE_PUBLICATION_COLLISION_ATTEMPTS_MAX);
        assert!(!self.identity.is_empty());
        let leaf = format!(".mantle-source-ingest-{}-{}.tmp", self.identity, self.next_index);
        self.next_index = self.next_index.saturating_add(1);
        leaf
    }
}

#[derive(Debug, Clone)]
struct PreparedRecord {
    record: SourceRecord,
    projection: ProjectedSourceRecord,
    plan: SourceIngestPlan,
    record_target: PathBuf,
    observation_target: Option<PathBuf>,
}

#[derive(Debug)]
struct PreparedImport {
    records: Vec<PreparedRecord>,
    pin_target: Option<PathBuf>,
    pin_exists_equal: bool,
}

#[derive(Debug)]
struct CreatedState {
    paths: Vec<PathBuf>,
    state_dir_existed: bool,
    source_root_existed: bool,
    records_dir_existed: bool,
    observations_dir_existed: bool,
    pins_dir_existed: bool,
}

pub(super) fn import_source_bundle_monotonic(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
    pin: bool,
) -> Result<SourceBundleImportReport, RunError> {
    import_source_bundle_with_hook(manifest, state_dir, pin, &mut no_ingest_hook)
}

pub(super) fn import_source_bundle_with_hook(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
    pin: bool,
    hook: &mut impl FnMut(SourceIngestHookPoint) -> Result<(), RunError>,
) -> Result<SourceBundleImportReport, RunError> {
    let prepared = prepare_import(manifest, state_dir, pin)?;
    execute_import(manifest, state_dir, prepared, hook)
}

fn prepare_import(manifest: &SourceBundleManifest, state_dir: &Path, pin: bool) -> Result<PreparedImport, RunError> {
    let existing_records = read_imported_source_records(state_dir)?;
    let mut existing_facts = project_existing_facts(state_dir, &existing_records)?;
    let mut prepared_records = Vec::with_capacity(manifest.records.len());
    for record in &manifest.records {
        let projection = project_source_record(record)?;
        let plan = plan_source_ingest(SourceIngestRequest {
            candidate: &projection.fact,
            existing: &existing_facts,
        })
        .map_err(|error| RunError::Internal(format!("planning source ingest: {error:?}")))?;
        validate_source_ingest_plan(&plan)
            .map_err(|error| RunError::Internal(format!("validating source ingest plan: {error:?}")))?;
        require_admitted_plan(&plan)?;
        let record_target = source_records_dir(state_dir).join(format!("{}.json", record.content_blake3));
        let observation_target = observation_target(state_dir, projection.observation.as_ref());
        validate_existing_observation(observation_target.as_deref(), projection.observation.as_ref())?;
        prepared_records.push(PreparedRecord {
            record: record.clone(),
            projection: projection.clone(),
            plan,
            record_target,
            observation_target,
        });
        existing_facts.push(projection.fact);
    }
    let (pin_target, pin_exists_equal) = prepare_pin(manifest, state_dir, pin)?;
    debug_assert_eq!(prepared_records.len(), manifest.records.len());
    debug_assert!(prepared_records.len() <= MAX_SOURCE_RECORDS);
    Ok(PreparedImport {
        records: prepared_records,
        pin_target,
        pin_exists_equal,
    })
}

fn execute_import(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
    prepared: PreparedImport,
    hook: &mut impl FnMut(SourceIngestHookPoint) -> Result<(), RunError>,
) -> Result<SourceBundleImportReport, RunError> {
    let mut created = created_state(state_dir);
    let result = execute_prepared_records(&prepared.records, &mut created, hook)
        .and_then(|_| execute_prepared_pin(manifest, &prepared, &mut created, hook));
    if let Err(error) = result {
        return Err(rollback_created_state(state_dir, &created, error));
    }
    build_import_report(manifest, &prepared)
}

fn execute_prepared_records(
    prepared: &[PreparedRecord],
    created: &mut CreatedState,
    hook: &mut impl FnMut(SourceIngestHookPoint) -> Result<(), RunError>,
) -> Result<(), RunError> {
    for (index, item) in prepared.iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| RunError::Internal("source ingest index overflow".to_string()))?;
        hook(SourceIngestHookPoint::BeforeRecord(index))?;
        if item.plan.disposition == SourceIngestDisposition::Add {
            publish_prepared_record(item, index, created, hook)?;
        }
        hook(SourceIngestHookPoint::AfterRecord(index))?;
    }
    Ok(())
}

fn publish_prepared_record(
    item: &PreparedRecord,
    index: u32,
    created: &mut CreatedState,
    hook: &mut impl FnMut(SourceIngestHookPoint) -> Result<(), RunError>,
) -> Result<(), RunError> {
    if let (Some(target), Some(observation)) = (&item.observation_target, &item.projection.observation) {
        match publish_json_no_replace(target, observation, "source observation")? {
            PublishDisposition::Created => created.paths.push(target.clone()),
            PublishDisposition::ExistingEqual => {}
        }
        hook(SourceIngestHookPoint::AfterObservation(index))?;
    }
    match publish_json_no_replace(&item.record_target, &item.record, "source record")? {
        PublishDisposition::Created => created.paths.push(item.record_target.clone()),
        PublishDisposition::ExistingEqual => {
            return Err(RunError::Internal(format!(
                "source ingest plan became stale before record publication: {}",
                item.record.identity
            )));
        }
    }
    Ok(())
}

fn execute_prepared_pin(
    manifest: &SourceBundleManifest,
    prepared: &PreparedImport,
    created: &mut CreatedState,
    hook: &mut impl FnMut(SourceIngestHookPoint) -> Result<(), RunError>,
) -> Result<(), RunError> {
    let Some(target) = prepared.pin_target.as_ref() else {
        return Ok(());
    };
    hook(SourceIngestHookPoint::BeforePin)?;
    if prepared.pin_exists_equal {
        return Ok(());
    }
    match publish_json_no_replace(target, manifest, "source pin")? {
        PublishDisposition::Created => created.paths.push(target.clone()),
        PublishDisposition::ExistingEqual => {
            return Err(RunError::Internal("source pin plan became stale before publication".to_string()));
        }
    }
    Ok(())
}

fn build_import_report(
    manifest: &SourceBundleManifest,
    prepared: &PreparedImport,
) -> Result<SourceBundleImportReport, RunError> {
    let imported_count =
        prepared.records.iter().filter(|item| item.plan.disposition == SourceIngestDisposition::Add).count();
    let skipped_count = prepared.records.len().saturating_sub(imported_count);
    let records = prepared
        .records
        .iter()
        .map(|item| summary_for_record(&item.record))
        .collect::<Result<Vec<_>, _>>()?;
    let source_observations = prepared
        .records
        .iter()
        .map(|item| item.projection.summary.clone())
        .collect::<Vec<SourceObservationImportSummary>>();
    debug_assert_eq!(records.len(), source_observations.len());
    debug_assert_eq!(records.len(), manifest.records.len());
    Ok(SourceBundleImportReport {
        imported_count: u32::try_from(imported_count)
            .map_err(|_| RunError::Internal("imported source record count overflow".to_string()))?,
        skipped_present_count: u32::try_from(skipped_count)
            .map_err(|_| RunError::Internal("present source record count overflow".to_string()))?,
        pinned: prepared.pin_target.is_some(),
        manifest_blake3: manifest.manifest_blake3.clone(),
        records,
        source_observations,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

fn project_existing_facts(state_dir: &Path, records: &[SourceRecord]) -> Result<Vec<SourceRecordFact>, RunError> {
    let mut facts = Vec::with_capacity(records.len());
    for record in records {
        let projection = project_source_record(record)?;
        let target = observation_target(state_dir, projection.observation.as_ref());
        validate_existing_observation(target.as_deref(), projection.observation.as_ref())?;
        facts.push(projection.fact);
    }
    debug_assert_eq!(facts.len(), records.len());
    debug_assert!(facts.len() <= MAX_SOURCE_RECORDS);
    Ok(facts)
}

fn require_admitted_plan(plan: &SourceIngestPlan) -> Result<(), RunError> {
    match plan.disposition {
        SourceIngestDisposition::Add | SourceIngestDisposition::IdenticalReuse => Ok(()),
        SourceIngestDisposition::IdentityConflict | SourceIngestDisposition::InvalidRejection => {
            Err(RunError::Internal(format!("source ingest rejected {}: {}", plan.record_identity, plan.reason_code)))
        }
    }
}

fn observation_target(state_dir: &Path, observation: Option<&SourceObservationWire>) -> Option<PathBuf> {
    observation.map(|value| source_observations_dir(state_dir).join(format!("{}.json", value.observation_blake3)))
}

fn validate_existing_observation(
    target: Option<&Path>,
    observation: Option<&SourceObservationWire>,
) -> Result<(), RunError> {
    let (Some(target), Some(observation)) = (target, observation) else {
        return Ok(());
    };
    if !target.exists() {
        return Ok(());
    }
    let existing: SourceObservationWire = read_json(target, "source observation")?;
    crunch_source_core::admit_source_observation(existing.clone())
        .map_err(|error| RunError::Internal(format!("existing source observation is invalid: {error:?}")))?;
    if existing != *observation {
        return Err(RunError::Internal(format!("existing source observation conflicts: {}", target.display())));
    }
    Ok(())
}

fn prepare_pin(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
    pin: bool,
) -> Result<(Option<PathBuf>, bool), RunError> {
    if !pin {
        return Ok((None, false));
    }
    let target = source_pins_dir(state_dir).join(format!("{}.json", manifest.manifest_blake3));
    if !target.exists() {
        return Ok((Some(target), false));
    }
    let existing: SourceBundleManifest = read_json(&target, "source pin")?;
    super::validate_manifest(&existing)?;
    if existing != *manifest {
        return Err(RunError::Internal(format!("existing source pin conflicts: {}", target.display())));
    }
    Ok((Some(target), true))
}

fn publish_json_no_replace<T: Serialize>(
    target: &Path,
    value: &T,
    label: &str,
) -> Result<PublishDisposition, RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Internal(format!("serializing {label}: {error}")))?;
    bytes.push(b'\n');
    publish_bytes_no_replace(target, &bytes, label)
}

#[cfg(target_os = "linux")]
pub(super) fn publish_bytes_no_replace(
    target: &Path,
    bytes: &[u8],
    label: &str,
) -> Result<PublishDisposition, RunError> {
    let parent = target.parent().ok_or_else(|| RunError::Internal(format!("{label} has no parent")))?;
    ensure_source_publication_directory(parent, label)?;
    let destination_leaf = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RunError::Internal(format!("{label} destination name is invalid")))?;
    let identity = target
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RunError::Internal(format!("{label} identity is invalid")))?;
    let maximum_payload_bytes = usize::try_from(SOURCE_INGEST_SERIALIZED_BYTES_MAX)
        .map_err(|_| RunError::Internal("source ingest serialized-byte limit exceeds this platform".to_string()))?;
    let request = PublicationRequest {
        destination_leaf: destination_leaf.to_string(),
        payload_bytes: bytes.len(),
        maximum_payload_bytes,
        collision_attempt_limit: SOURCE_PUBLICATION_COLLISION_ATTEMPTS_MAX,
        final_mode: SOURCE_PUBLICATION_FILE_MODE,
        replacement: ReplacementMode::NoReplace,
        durability: DurabilityMode::DurabilityRequired,
    };
    let directory = open_source_publication_parent(parent, label)?;
    let mut io = LinuxDirectory::from_open_directory(directory)
        .map_err(|error| RunError::Internal(format!("opening {label} publication capability: {error}")))?;
    let mut names = SourceStageNames {
        identity: identity.to_string(),
        next_index: 0,
    };
    let classification = publish_one_file(&mut io, &mut names, &request, bytes)
        .map_err(|error| RunError::Internal(format!("classifying {label} publication: {error:?}")))?;
    interpret_durable_publication(target, bytes, label, &classification.disposition)
}

#[cfg(target_os = "linux")]
fn interpret_durable_publication(
    target: &Path,
    expected: &[u8],
    label: &str,
    disposition: &DurablePublicationDisposition,
) -> Result<PublishDisposition, RunError> {
    match disposition {
        DurablePublicationDisposition::CommittedAndParentSynchronized => Ok(PublishDisposition::Created),
        DurablePublicationDisposition::DestinationExists {
            cleanup: CleanupDisposition::Removed,
        } => compare_existing_publication(target, expected, label),
        DurablePublicationDisposition::DestinationExists { cleanup } => {
            Err(RunError::Internal(format!("{label} destination exists but stage cleanup was {cleanup:?}")))
        }
        DurablePublicationDisposition::NotCommitted { failure, cleanup } => {
            Err(RunError::Internal(format!("{label} was not committed: failure={failure:?} cleanup={cleanup:?}")))
        }
        DurablePublicationDisposition::CommittedVisibilityOnly => Err(RunError::Internal(format!(
            "{label} publication committed visibility without the required payload and parent durability: {}",
            target.display()
        ))),
        DurablePublicationDisposition::CommittedDurabilityUnknown => Err(RunError::Internal(format!(
            "{label} publication committed with unknown parent durability and was retained: {}",
            target.display()
        ))),
    }
}

#[cfg(target_os = "linux")]
fn compare_existing_publication(target: &Path, expected: &[u8], label: &str) -> Result<PublishDisposition, RunError> {
    let metadata = fs::symlink_metadata(target)
        .map_err(|error| RunError::Internal(format!("inspecting existing {label} {}: {error}", target.display())))?;
    let actual_bytes = usize::try_from(metadata.len())
        .map_err(|_| RunError::Internal(format!("existing {label} size exceeds this platform")))?;
    if !metadata.file_type().is_file() || actual_bytes > expected.len() {
        return Err(RunError::Internal(format!("existing {label} conflicts: {}", target.display())));
    }
    let actual = fs::read(target)
        .map_err(|error| RunError::Internal(format!("reading existing {label} {}: {error}", target.display())))?;
    if actual != expected {
        return Err(RunError::Internal(format!("existing {label} conflicts: {}", target.display())));
    }
    Ok(PublishDisposition::ExistingEqual)
}

#[cfg(target_os = "linux")]
fn ensure_source_publication_directory(path: &Path, label: &str) -> Result<(), RunError> {
    fs::create_dir_all(path)
        .map_err(|error| RunError::Internal(format!("creating {label} parent {}: {error}", path.display())))?;
    sync_source_directory(path, label)?;
    let mut ancestor = path.parent();
    for _ in 0..SOURCE_PUBLICATION_ANCESTOR_SYNC_COUNT {
        let Some(directory) = ancestor else {
            break;
        };
        sync_source_directory(directory, label)?;
        ancestor = directory.parent();
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn open_source_publication_parent(parent: &Path, label: &str) -> Result<fs::File, RunError> {
    use std::os::unix::fs::OpenOptionsExt as _;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent)
        .map_err(|error| RunError::Internal(format!("opening {label} parent without following links: {error}")))
}

#[cfg(target_os = "linux")]
fn sync_source_directory(path: &Path, label: &str) -> Result<(), RunError> {
    let directory = open_source_publication_parent(path, label)?;
    let metadata = directory
        .metadata()
        .map_err(|error| RunError::Internal(format!("inspecting {label} parent {}: {error}", path.display())))?;
    if !metadata.is_dir() {
        return Err(RunError::Internal(format!("{label} parent is not a directory: {}", path.display())));
    }
    directory
        .sync_all()
        .map_err(|error| RunError::Internal(format!("syncing {label} parent {}: {error}", path.display())))
}

#[cfg(not(target_os = "linux"))]
pub(super) fn publish_bytes_no_replace(
    target: &Path,
    bytes: &[u8],
    label: &str,
) -> Result<PublishDisposition, RunError> {
    let parent = target.parent().ok_or_else(|| RunError::Internal(format!("{label} has no parent")))?;
    fs::create_dir_all(parent)
        .map_err(|error| RunError::Internal(format!("creating {label} parent {}: {error}", parent.display())))?;
    let stage = target.with_extension(SOURCE_INGEST_STAGE_EXTENSION);
    write_stage(&stage, bytes, label)?;
    match crate::linux_rename::rename_path_no_replace(&stage, target) {
        Ok(()) => finish_created_publication(target, parent, label),
        Err(error) => interpret_publish_collision(&stage, target, bytes, label, error),
    }
}

#[cfg(not(target_os = "linux"))]
fn write_stage(stage: &Path, bytes: &[u8], label: &str) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(stage)
        .map_err(|error| RunError::Internal(format!("creating {label} stage {}: {error}", stage.display())))?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.flush()).and_then(|_| file.sync_all()) {
        let primary = RunError::Internal(format!("writing and syncing {label} stage {}: {error}", stage.display()));
        return Err(cleanup_failed_stage(stage, label, primary));
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn finish_created_publication(target: &Path, parent: &Path, label: &str) -> Result<PublishDisposition, RunError> {
    if let Err(error) = sync_parent(parent, label) {
        return Err(cleanup_failed_publication(target, parent, label, error));
    }
    Ok(PublishDisposition::Created)
}

#[cfg(not(target_os = "linux"))]
fn cleanup_failed_publication(target: &Path, parent: &Path, label: &str, primary: RunError) -> RunError {
    match fs::remove_file(target) {
        Ok(()) => match sync_parent(parent, label) {
            Ok(()) => primary,
            Err(cleanup) => RunError::Internal(format!("{primary}; cleanup sync failed: {cleanup}")),
        },
        Err(cleanup) => RunError::Internal(format!(
            "{primary}; cleanup of newly published {label} {} failed: {cleanup}",
            target.display()
        )),
    }
}

#[cfg(not(target_os = "linux"))]
fn cleanup_failed_stage(stage: &Path, label: &str, primary: RunError) -> RunError {
    match fs::remove_file(stage) {
        Ok(()) => {
            let Some(parent) = stage.parent() else {
                return primary;
            };
            match sync_parent(parent, label) {
                Ok(()) => primary,
                Err(cleanup) => RunError::Internal(format!("{primary}; cleanup sync failed: {cleanup}")),
            }
        }
        Err(cleanup) => {
            RunError::Internal(format!("{primary}; cleanup of {label} stage {} failed: {cleanup}", stage.display()))
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn interpret_publish_collision(
    stage: &Path,
    target: &Path,
    expected: &[u8],
    label: &str,
    error: impl core::fmt::Display,
) -> Result<PublishDisposition, RunError> {
    fs::remove_file(stage).map_err(|cleanup| {
        RunError::Internal(format!(
            "publishing {label} {} without replacement: {error}; cleanup of stage {} failed: {cleanup}",
            target.display(),
            stage.display()
        ))
    })?;
    if let Some(parent) = stage.parent() {
        sync_parent(parent, label)?;
    }
    if target.is_file() {
        let actual = fs::read(target).map_err(|read_error| {
            RunError::Internal(format!("reading existing {label} {}: {read_error}", target.display()))
        })?;
        if actual == expected {
            return Ok(PublishDisposition::ExistingEqual);
        }
    }
    Err(RunError::Internal(format!("publishing {label} {} without replacement: {error}", target.display())))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T, RunError> {
    let file = fs::File::open(path)
        .map_err(|error| RunError::Internal(format!("reading {label} {}: {error}", path.display())))?;
    serde_json::from_reader(BufReader::new(file))
        .map_err(|error| RunError::Internal(format!("parsing {label} {}: {error}", path.display())))
}

#[cfg(target_os = "linux")]
fn sync_parent(parent: &Path, label: &str) -> Result<(), RunError> {
    sync_source_directory(parent, label)
}

#[cfg(not(target_os = "linux"))]
fn sync_parent(parent: &Path, label: &str) -> Result<(), RunError> {
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| RunError::Internal(format!("syncing {label} parent {}: {error}", parent.display())))
}

fn created_state(state_dir: &Path) -> CreatedState {
    CreatedState {
        paths: Vec::new(),
        state_dir_existed: state_dir.is_dir(),
        source_root_existed: state_dir.join(super::SOURCE_STATE_DIR).is_dir(),
        records_dir_existed: source_records_dir(state_dir).is_dir(),
        observations_dir_existed: source_observations_dir(state_dir).is_dir(),
        pins_dir_existed: source_pins_dir(state_dir).is_dir(),
    }
}

fn rollback_created_state(state_dir: &Path, created: &CreatedState, primary: RunError) -> RunError {
    let mut cleanup_errors = Vec::new();
    for path in created.paths.iter().rev() {
        if let Err(error) = fs::remove_file(path) {
            cleanup_errors.push(format!("{}: {error}", path.display()));
            continue;
        }
        if let Some(parent) = path.parent()
            && let Err(error) = sync_parent(parent, "source ingest rollback")
        {
            cleanup_errors.push(error.to_string());
        }
    }
    remove_created_directory(&source_pins_dir(state_dir), created.pins_dir_existed, &mut cleanup_errors);
    remove_created_directory(
        &source_observations_dir(state_dir),
        created.observations_dir_existed,
        &mut cleanup_errors,
    );
    remove_created_directory(&source_records_dir(state_dir), created.records_dir_existed, &mut cleanup_errors);
    remove_created_directory(
        &state_dir.join(super::SOURCE_STATE_DIR),
        created.source_root_existed,
        &mut cleanup_errors,
    );
    remove_created_directory(state_dir, created.state_dir_existed, &mut cleanup_errors);
    if cleanup_errors.is_empty() {
        return primary;
    }
    RunError::Internal(format!("{primary}; source ingest rollback failed: {}", cleanup_errors.join("; ")))
}

fn remove_created_directory(path: &Path, existed: bool, errors: &mut Vec<String>) {
    if existed || !path.exists() {
        return;
    }
    if let Err(error) = fs::remove_dir(path) {
        errors.push(format!("{}: {error}", path.display()));
        return;
    }
    if let Some(parent) = path.parent()
        && let Err(error) = sync_parent(parent, "source ingest rollback")
    {
        errors.push(error.to_string());
    }
}

fn no_ingest_hook(_point: SourceIngestHookPoint) -> Result<(), RunError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::SourceIngestHookPoint;
    use super::import_source_bundle_with_hook;
    use crate::errors::RunError;
    use crate::source_bundle::FETCH_ENV_REV_KEY;
    use crate::source_bundle::RECORD_METADATA_URL_KEY;
    use crate::source_bundle::SourceFileEntry;
    use crate::source_bundle::SourceFileType;
    use crate::source_bundle::SourceRecord;
    use crate::source_bundle::SourceRecordKind;
    use crate::source_bundle::assemble_source_bundle;
    use crate::source_bundle::digest_source_record_content;
    use crate::source_bundle::import_source_bundle;

    const TEST_STORE_PREFIX: &str = "/mantle/store";
    const TEST_FIRST_PAYLOAD: &[u8] = b"first-source";
    const TEST_SECOND_PAYLOAD: &[u8] = b"second-source";
    const TEST_SECOND_RECORD_INDEX: u32 = 1;

    #[test]
    fn fresh_import_publishes_record_observation_and_pin() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let manifest = manifest(vec![fixed_record(
            "source-a",
            "https://example.invalid/source-a",
            TEST_FIRST_PAYLOAD,
        )]);
        let report = import_source_bundle(&manifest, &state_dir, true).unwrap();
        let observation_id = report.source_observations[0].observation_blake3.as_ref().unwrap();
        let observation_path = state_dir.join("source-bundles/observations").join(format!("{observation_id}.json"));
        let record_path = state_dir
            .join("source-bundles/records")
            .join(format!("{}.json", manifest.records[0].content_blake3));
        let pin_path = state_dir.join("source-bundles/pins").join(format!("{}.json", manifest.manifest_blake3));
        assert_eq!(report.imported_count, 1);
        assert_eq!(report.skipped_present_count, 0);
        assert!(observation_path.is_file());
        assert!(record_path.is_file());
        assert!(pin_path.is_file());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_component_is_create_new_idempotent_and_conflict_safe() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("objects/source.json");
        let first = serde_json::json!({"source": "first"});
        let second = serde_json::json!({"source": "second"});
        let created = super::publish_json_no_replace(&target, &first, "test source").unwrap();
        let first_bytes = std::fs::read(&target).unwrap();
        let reused = super::publish_json_no_replace(&target, &first, "test source").unwrap();
        let conflict = super::publish_json_no_replace(&target, &second, "test source").unwrap_err();
        assert_eq!(created, super::PublishDisposition::Created);
        assert_eq!(reused, super::PublishDisposition::ExistingEqual);
        assert!(conflict.to_string().contains("conflicts"));
        assert_eq!(std::fs::read(&target).unwrap(), first_bytes);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn committed_unknown_destination_is_retained_for_recovery() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("source.json");
        let expected = b"committed-source";
        std::fs::write(&target, expected).unwrap();
        let error = super::interpret_durable_publication(
            &target,
            expected,
            "test source",
            &durable_file_publication::core::PublicationDisposition::CommittedDurabilityUnknown,
        )
        .unwrap_err();
        assert!(error.to_string().contains("committed with unknown parent durability"));
        assert_eq!(std::fs::read(&target).unwrap(), expected);
        assert!(target.is_file());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_component_uses_next_bounded_stage_name_after_collision() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("objects");
        std::fs::create_dir_all(&parent).unwrap();
        let stale_stage = parent.join(".mantle-source-ingest-source-0.tmp");
        std::fs::write(&stale_stage, b"preexisting-stage").unwrap();
        let target = parent.join("source.json");
        let created =
            super::publish_json_no_replace(&target, &serde_json::json!({"source": "value"}), "test source").unwrap();
        assert_eq!(created, super::PublishDisposition::Created);
        assert_eq!(std::fs::read(&stale_stage).unwrap(), b"preexisting-stage");
        assert!(target.is_file());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_component_rejects_symlink_parent_without_outside_mutation() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let outside = temp.path().join("outside");
        let state = temp.path().join("state");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        symlink(&outside, state.join("objects")).unwrap();
        let target = state.join("objects/source.json");
        let error = super::publish_json_no_replace(&target, &serde_json::json!({"source": "value"}), "test source")
            .unwrap_err();
        assert!(error.to_string().contains("without following links"));
        assert!(!outside.join("source.json").exists());
        assert!(std::fs::read_dir(&outside).unwrap().next().is_none());
    }

    #[test]
    fn adapters_project_each_supported_source_family() {
        const SHA1_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
        let fixed = fixed_record("fixed", "https://example.invalid/fixed", TEST_FIRST_PAYLOAD);
        let git = git_record("git", SHA1_REVISION, TEST_FIRST_PAYLOAD);
        let local = nested_record(SourceRecordKind::LocalPath, "local", TEST_FIRST_PAYLOAD);
        let mirror = nested_record(SourceRecordKind::PackageMirror, "mirror", TEST_FIRST_PAYLOAD);
        let opaque = source_record(SourceRecordKind::ProofInput, "opaque", BTreeMap::new(), TEST_FIRST_PAYLOAD);
        let cases = [
            (fixed, crunch_source_core::SourceKind::FixedUrl),
            (git, crunch_source_core::SourceKind::VcsSnapshot),
            (local, crunch_source_core::SourceKind::LocalLogical),
            (mirror, crunch_source_core::SourceKind::PackageMirror),
            (opaque, crunch_source_core::SourceKind::OpaqueAdapter),
        ];
        for (record, expected_kind) in cases {
            let projection = crate::source_bundle::source_observation_adapter::project_source_record(&record).unwrap();
            let observation = projection.observation.unwrap();
            assert_eq!(observation.source_kind, expected_kind);
            assert_eq!(observation.provenance, crunch_source_core::ProvenanceDisposition::Complete);
        }
    }

    #[test]
    fn mirror_location_does_not_change_observed_content_or_identity() {
        let first = fixed_record("mirror-a", "https://mirror-a.invalid/source", TEST_FIRST_PAYLOAD);
        let second = fixed_record("mirror-b", "https://mirror-b.invalid/source", TEST_FIRST_PAYLOAD);
        let first_projection = crate::source_bundle::source_observation_adapter::project_source_record(&first).unwrap();
        let second_projection =
            crate::source_bundle::source_observation_adapter::project_source_record(&second).unwrap();
        assert_ne!(first.content_blake3, second.content_blake3);
        assert_eq!(first_projection.summary.content_blake3, second_projection.summary.content_blake3);
        assert_eq!(first_projection.summary.observation_blake3, second_projection.summary.observation_blake3);
    }

    #[test]
    fn identical_reuse_is_write_free() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let manifest = manifest(vec![fixed_record(
            "source-a",
            "https://example.invalid/source-a",
            TEST_FIRST_PAYLOAD,
        )]);
        import_source_bundle(&manifest, &state_dir, true).unwrap();
        let before = snapshot_tree(&state_dir);
        let report = import_source_bundle(&manifest, &state_dir, true).unwrap();
        let after = snapshot_tree(&state_dir);
        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_present_count, 1);
        assert_eq!(before, after);
        assert_eq!(report.source_observations[0].disposition, "complete");
    }

    #[test]
    fn identity_conflict_rejects_before_state_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let first = manifest(vec![fixed_record(
            "source-a",
            "https://example.invalid/source-a",
            TEST_FIRST_PAYLOAD,
        )]);
        import_source_bundle(&first, &state_dir, true).unwrap();
        std::fs::write(state_dir.join("roots.json"), b"preserved-root").unwrap();
        std::fs::write(state_dir.join("readiness.json"), b"preserved-readiness").unwrap();
        std::fs::write(state_dir.join("release-evidence.json"), b"preserved-release").unwrap();
        let before = snapshot_tree(&state_dir);
        let conflict = manifest(vec![fixed_record(
            "source-a",
            "https://example.invalid/source-b",
            TEST_SECOND_PAYLOAD,
        )]);
        let error = import_source_bundle(&conflict, &state_dir, true).unwrap_err();
        let after = snapshot_tree(&state_dir);
        assert!(error.to_string().contains("identity-conflict"));
        assert_eq!(before, after);
        assert!(!state_dir.join("source-bundles/pins").join(format!("{}.json", conflict.manifest_blake3)).exists());
    }

    #[test]
    fn interruption_between_observation_and_record_rolls_back_sidecar() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("new-state");
        let manifest = manifest(vec![fixed_record(
            "source-a",
            "https://example.invalid/source-a",
            TEST_FIRST_PAYLOAD,
        )]);
        let mut hook = |point| {
            if point == SourceIngestHookPoint::AfterObservation(0) {
                return Err(RunError::Internal("injected observation fault".to_string()));
            }
            Ok(())
        };
        let error = import_source_bundle_with_hook(&manifest, &state_dir, true, &mut hook).unwrap_err();
        assert!(error.to_string().contains("injected observation fault"));
        assert!(!state_dir.exists());
    }

    #[test]
    fn interrupted_record_import_rolls_back_all_created_state() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("new-state");
        let manifest = manifest(vec![
            fixed_record("source-a", "https://example.invalid/source-a", TEST_FIRST_PAYLOAD),
            fixed_record("source-b", "https://example.invalid/source-b", TEST_SECOND_PAYLOAD),
        ]);
        let mut hook = |point| {
            if point == SourceIngestHookPoint::BeforeRecord(TEST_SECOND_RECORD_INDEX) {
                return Err(RunError::Internal("injected record fault".to_string()));
            }
            Ok(())
        };
        let error = import_source_bundle_with_hook(&manifest, &state_dir, true, &mut hook).unwrap_err();
        assert!(error.to_string().contains("injected record fault"));
        assert!(!state_dir.exists());
    }

    #[test]
    fn interrupted_extension_preserves_all_existing_state() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let base = manifest(vec![fixed_record(
            "source-base",
            "https://example.invalid/source-base",
            TEST_FIRST_PAYLOAD,
        )]);
        import_source_bundle(&base, &state_dir, true).unwrap();
        std::fs::write(state_dir.join("roots.json"), b"preserved-root").unwrap();
        std::fs::write(state_dir.join("readiness.json"), b"preserved-readiness").unwrap();
        std::fs::write(state_dir.join("release-evidence.json"), b"preserved-release").unwrap();
        let before = snapshot_tree(&state_dir);
        let extension = manifest(vec![fixed_record(
            "source-extension",
            "https://example.invalid/source-extension",
            TEST_SECOND_PAYLOAD,
        )]);
        let mut hook = |point| {
            if point == SourceIngestHookPoint::AfterRecord(0) {
                return Err(RunError::Internal("injected extension fault".to_string()));
            }
            Ok(())
        };
        let error = import_source_bundle_with_hook(&extension, &state_dir, true, &mut hook).unwrap_err();
        let after = snapshot_tree(&state_dir);
        assert!(error.to_string().contains("injected extension fault"));
        assert_eq!(before, after);
        assert!(!state_dir.join("source-bundles/pins").join(format!("{}.json", extension.manifest_blake3)).exists());
    }

    #[test]
    fn interrupted_pin_import_rolls_back_records_and_observations() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("new-state");
        let manifest = manifest(vec![fixed_record(
            "source-a",
            "https://example.invalid/source-a",
            TEST_FIRST_PAYLOAD,
        )]);
        let mut hook = |point| {
            if point == SourceIngestHookPoint::BeforePin {
                return Err(RunError::Internal("injected pin fault".to_string()));
            }
            Ok(())
        };
        let error = import_source_bundle_with_hook(&manifest, &state_dir, true, &mut hook).unwrap_err();
        assert!(error.to_string().contains("injected pin fault"));
        assert!(!state_dir.exists());
    }

    #[test]
    fn legacy_mutable_git_reference_imports_without_invented_observation() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let manifest = manifest(vec![git_record("legacy-git", "refs/heads/main", TEST_FIRST_PAYLOAD)]);
        let report = import_source_bundle(&manifest, &state_dir, false).unwrap();
        assert_eq!(report.imported_count, 1);
        assert_eq!(report.source_observations[0].disposition, "provenance-unavailable-legacy-v1");
        assert!(report.source_observations[0].observation_blake3.is_none());
        assert!(!state_dir.join("source-bundles/observations").exists());
    }

    #[test]
    fn ambiguous_legacy_local_file_keeps_provenance_unavailable() {
        let record = source_record(SourceRecordKind::LocalPath, "legacy-local", BTreeMap::new(), TEST_FIRST_PAYLOAD);
        let expected_content = crate::source_bundle::digest_source_entries(&record.files).unwrap();
        let projection = crate::source_bundle::source_observation_adapter::project_source_record(&record).unwrap();
        assert_eq!(projection.summary.disposition, "provenance-unavailable-legacy-v1");
        assert!(projection.observation.is_none());
        assert_eq!(projection.summary.content_blake3, expected_content);
    }

    #[test]
    fn secret_bearing_locator_rejects_before_state_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let manifest = manifest(vec![fixed_record(
            "source-secret",
            "https://example.invalid/source?access_token=private",
            TEST_FIRST_PAYLOAD,
        )]);
        let error = import_source_bundle(&manifest, &state_dir, true).unwrap_err();
        assert!(error.to_string().contains("SecretBearingLocator"));
        assert!(!state_dir.exists());
    }

    fn manifest(records: Vec<SourceRecord>) -> crate::source_bundle::SourceBundleManifest {
        assemble_source_bundle(records, TEST_STORE_PREFIX).unwrap()
    }

    fn fixed_record(identity: &str, url: &str, payload: &[u8]) -> SourceRecord {
        let mut metadata = BTreeMap::new();
        metadata.insert(RECORD_METADATA_URL_KEY.to_string(), url.to_string());
        source_record(SourceRecordKind::FixedUrl, identity, metadata, payload)
    }

    fn git_record(identity: &str, revision: &str, payload: &[u8]) -> SourceRecord {
        let mut metadata = BTreeMap::new();
        metadata.insert(RECORD_METADATA_URL_KEY.to_string(), "https://example.invalid/source.git".to_string());
        metadata.insert(FETCH_ENV_REV_KEY.to_string(), revision.to_string());
        source_record(SourceRecordKind::VcsSnapshot, identity, metadata, payload)
    }

    fn nested_record(kind: SourceRecordKind, identity: &str, payload: &[u8]) -> SourceRecord {
        let mut record = source_record(kind, identity, BTreeMap::new(), payload);
        record.files[0].path = "tree/source".to_string();
        record.content_blake3 = digest_source_record_content(&record.kind, &record.metadata, &record.files).unwrap();
        record
    }

    fn source_record(
        kind: SourceRecordKind,
        identity: &str,
        metadata: BTreeMap<String, String>,
        payload: &[u8],
    ) -> SourceRecord {
        let file = SourceFileEntry {
            path: "source".to_string(),
            file_type: SourceFileType::Regular,
            executable: false,
            size: u64::try_from(payload.len()).unwrap(),
            content_hex: Some(encode_hex(payload)),
            symlink_target: None,
            chunk_index: None,
            chunk_count: None,
            blake3: blake3::hash(payload).to_hex().to_string(),
        };
        let files = vec![file];
        let content_blake3 = digest_source_record_content(&kind, &metadata, &files).unwrap();
        SourceRecord {
            kind,
            identity: identity.to_string(),
            store_prefix: None,
            adapter: None,
            metadata,
            payload_bytes: u64::try_from(payload.len()).unwrap(),
            content_blake3,
            files,
        }
    }

    fn encode_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn snapshot_tree(root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut entries = Vec::new();
        snapshot_directory(root, root, &mut entries);
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries
    }

    fn snapshot_directory(root: &Path, current: &Path, entries: &mut Vec<(String, Vec<u8>)>) {
        let mut paths = std::fs::read_dir(current).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                snapshot_directory(root, &path, entries);
            } else {
                let relative = path.strip_prefix(root).unwrap().to_string_lossy().to_string();
                entries.push((relative, std::fs::read(path).unwrap()));
            }
        }
    }
}
