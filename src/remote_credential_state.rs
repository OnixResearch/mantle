use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use fs2::FileExt as _;
use rand::RngCore as _;
use rand::rngs::OsRng;
use serde::Serialize;
use zeroize::Zeroize as _;
use zeroize::Zeroizing;

use crate::errors::RunError;
use crate::remote_credentials::MAX_INVALIDATED_LEGACY_TICKET_IDS;
use crate::remote_credentials::REMOTE_TICKET_STATE_SCHEMA_VERSION;
use crate::remote_credentials::RemoteTicketState;

pub const REMOTE_TICKET_STATE_DIRECTORY: &str = "remote-builders";
pub const REMOTE_TICKET_STATE_FILENAME: &str = "tickets.json";
pub const REMOTE_TICKET_STATE_BYTES_MAX: u64 = 16_777_216;
const REMOTE_TICKET_STATE_LOCK_FILENAME: &str = "tickets.lock";
const REMOTE_TICKET_STATE_LOCK_WAIT_TIMEOUT_MS: u64 = 5_000;
const REMOTE_TICKET_STATE_LOCK_POLL_INTERVAL_MS: u64 = 10;

const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;
const PRIVATE_DIRECTORY_REQUIRED_OWNER_BITS: u32 = 0o700;
const PRIVATE_FILE_REQUIRED_OWNER_BITS: u32 = 0o600;
const NON_OWNER_PERMISSION_BITS: u32 = 0o077;
const NON_OWNER_WRITE_BITS: u32 = 0o022;
const STATE_TEMP_RANDOM_BYTES: usize = 16;
const HEX_CHARS_PER_BYTE: usize = 2;
const STATE_TEMP_PREFIX: &str = ".tickets.json.tmp-";
const STATE_TEMP_OPEN_ATTEMPTS_MAX: u32 = 16;
const LEGACY_TICKET_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyTicketMigrationReport {
    pub legacy_schema_version: u32,
    pub next_schema_version: u32,
    pub invalidated_ticket_count: u32,
    pub invalidated_ticket_ids: Vec<String>,
    pub replacement_issuance_required: bool,
    pub executed: bool,
}

#[derive(Debug)]
pub struct TicketStateMutationGuard {
    file: File,
}

impl Drop for TicketStateMutationGuard {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

pub fn acquire_ticket_state_mutation_guard(state_dir: &Path) -> Result<TicketStateMutationGuard, RunError> {
    acquire_ticket_state_mutation_guard_with_wait(
        state_dir,
        Duration::from_millis(REMOTE_TICKET_STATE_LOCK_WAIT_TIMEOUT_MS),
        Duration::from_millis(REMOTE_TICKET_STATE_LOCK_POLL_INTERVAL_MS),
    )
}

fn acquire_ticket_state_mutation_guard_with_wait(
    state_dir: &Path,
    wait_timeout: Duration,
    poll_interval: Duration,
) -> Result<TicketStateMutationGuard, RunError> {
    let parent = state_dir.join(REMOTE_TICKET_STATE_DIRECTORY);
    ensure_private_state_directory(&parent)?;
    let lock_path = parent.join(REMOTE_TICKET_STATE_LOCK_FILENAME);
    let file = open_private_state_lock(&lock_path).map_err(|_| state_error("lock-open-failed"))?;
    let metadata = file.metadata().map_err(|_| state_error("lock-metadata-failed"))?;
    validate_private_state_file_metadata(&metadata)?;
    let started = Instant::now();
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(TicketStateMutationGuard { file }),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if started.elapsed() >= wait_timeout {
                    return Err(state_error("lock-timeout"));
                }
                thread::sleep(poll_interval);
            }
            Err(_) => return Err(state_error("lock-acquire-failed")),
        }
    }
}

pub fn ticket_state_path(state_dir: &Path) -> PathBuf {
    state_dir.join(REMOTE_TICKET_STATE_DIRECTORY).join(REMOTE_TICKET_STATE_FILENAME)
}

pub fn load_ticket_state(state_dir: &Path) -> Result<RemoteTicketState, RunError> {
    let path = ticket_state_path(state_dir);
    let Some(bytes) = read_private_state_file(&path)? else {
        return Ok(RemoteTicketState::default());
    };
    let mut value = ZeroizingJsonValue::parse(&bytes)?;
    let schema_version = state_schema_version(value.as_value())?;
    if schema_version == LEGACY_TICKET_STATE_SCHEMA_VERSION {
        return Err(state_error("legacy-migration-required"));
    }
    if schema_version != REMOTE_TICKET_STATE_SCHEMA_VERSION {
        return Err(state_error("schema-version-unsupported"));
    }
    if json_contains_field(value.as_value(), "secret") {
        return Err(state_error("plaintext-field-forbidden"));
    }
    let state: RemoteTicketState = serde_json::from_value(value.take()).map_err(|_| state_error("malformed"))?;
    state.validate().map_err(|_| state_error("validation-failed"))?;
    Ok(state)
}

pub fn save_ticket_state(state_dir: &Path, state: &RemoteTicketState) -> Result<(), RunError> {
    state.validate().map_err(|_| state_error("validation-failed"))?;
    let path = ticket_state_path(state_dir);
    let parent = path.parent().ok_or_else(|| state_error("parent-missing"))?;
    ensure_private_state_directory(parent)?;
    validate_existing_state_target(&path)?;
    let mut rendered = Zeroizing::new(Vec::new());
    serde_json::to_writer_pretty(&mut *rendered, state).map_err(|_| state_error("serialization-failed"))?;
    rendered.push(b'\n');
    let rendered_bytes = u64::try_from(rendered.len()).map_err(|_| state_error("size-conversion-failed"))?;
    if rendered_bytes > REMOTE_TICKET_STATE_BYTES_MAX {
        return Err(state_error("size-limit-exceeded"));
    }
    let (tmp_path, mut file) = create_private_state_temp(parent)?;
    let write_result = write_and_commit_state(&path, parent, &tmp_path, &mut file, &rendered);
    if write_result.is_err() {
        let _cleanup_result = fs::remove_file(&tmp_path);
    }
    write_result
}

pub fn migrate_legacy_ticket_state(
    state_dir: &Path,
    invalidate_legacy: bool,
    dry_run: bool,
) -> Result<LegacyTicketMigrationReport, RunError> {
    if !invalidate_legacy {
        return Err(state_error("migration-confirmation-required"));
    }
    let path = ticket_state_path(state_dir);
    let bytes = read_explicit_migration_file(&path)?;
    let value = ZeroizingJsonValue::parse(&bytes)?;
    let invalidated_ticket_ids = match state_schema_version(value.as_value()) {
        Ok(LEGACY_TICKET_STATE_SCHEMA_VERSION) => legacy_ticket_ids(value.as_value())?,
        Ok(_) => return Err(state_error("legacy-state-not-found")),
        Err(error) => return Err(error),
    };
    let invalidated_ticket_count = u32::try_from(invalidated_ticket_ids.len())
        .map_err(|_| state_error("legacy-ticket-count-conversion-failed"))?;
    let report = LegacyTicketMigrationReport {
        legacy_schema_version: LEGACY_TICKET_STATE_SCHEMA_VERSION,
        next_schema_version: REMOTE_TICKET_STATE_SCHEMA_VERSION,
        invalidated_ticket_count,
        invalidated_ticket_ids: invalidated_ticket_ids.clone(),
        replacement_issuance_required: true,
        executed: !dry_run,
    };
    if dry_run {
        return Ok(report);
    }
    harden_explicit_migration_permissions(&path)?;
    let _guard = acquire_ticket_state_mutation_guard(state_dir)?;
    let locked_bytes = read_explicit_migration_file(&path)?;
    let locked_value = ZeroizingJsonValue::parse(&locked_bytes)?;
    let locked_ids = match state_schema_version(locked_value.as_value()) {
        Ok(LEGACY_TICKET_STATE_SCHEMA_VERSION) => legacy_ticket_ids(locked_value.as_value())?,
        Ok(_) => return Err(state_error("legacy-state-not-found")),
        Err(error) => return Err(error),
    };
    if locked_ids != invalidated_ticket_ids {
        return Err(state_error("legacy-state-changed-during-migration"));
    }
    let next_state = RemoteTicketState {
        invalidated_legacy_ticket_ids: locked_ids.into_iter().collect(),
        ..RemoteTicketState::default()
    };
    save_ticket_state(state_dir, &next_state)?;
    Ok(report)
}

fn read_explicit_migration_file(path: &Path) -> Result<Zeroizing<Vec<u8>>, RunError> {
    let parent = path.parent().ok_or_else(|| state_error("parent-missing"))?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|_| state_error("parent-inspection-failed"))?;
    validate_migration_source_directory(&parent_metadata)?;
    let mut file = open_state_read_no_follow(path).map_err(|_| state_error("open-failed"))?;
    let metadata = file.metadata().map_err(|_| state_error("metadata-failed"))?;
    validate_migration_source_file(&metadata)?;
    let limit_with_probe =
        REMOTE_TICKET_STATE_BYTES_MAX.checked_add(1).ok_or_else(|| state_error("size-limit-overflow"))?;
    let mut bytes = Zeroizing::new(Vec::new());
    std::io::Read::by_ref(&mut file)
        .take(limit_with_probe)
        .read_to_end(&mut bytes)
        .map_err(|_| state_error("read-failed"))?;
    let observed_bytes = u64::try_from(bytes.len()).map_err(|_| state_error("size-conversion-failed"))?;
    if observed_bytes > REMOTE_TICKET_STATE_BYTES_MAX {
        return Err(state_error("size-limit-exceeded"));
    }
    Ok(bytes)
}

#[cfg(unix)]
fn validate_migration_source_directory(metadata: &fs::Metadata) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt as _;

    if !metadata.file_type().is_dir() || metadata.uid() != unsafe { libc::geteuid() } {
        return Err(state_error("migration-parent-unsafe"));
    }
    if metadata.mode() & NON_OWNER_WRITE_BITS != 0 {
        return Err(state_error("migration-parent-writable-by-non-owner"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_migration_source_directory(metadata: &fs::Metadata) -> Result<(), RunError> {
    if !metadata.file_type().is_dir() {
        return Err(state_error("migration-parent-unsafe"));
    }
    Ok(())
}

#[cfg(unix)]
fn validate_migration_source_file(metadata: &fs::Metadata) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt as _;

    if !metadata.file_type().is_file() || metadata.uid() != unsafe { libc::geteuid() } {
        return Err(state_error("migration-target-unsafe"));
    }
    if metadata.mode() & NON_OWNER_WRITE_BITS != 0 {
        return Err(state_error("migration-target-writable-by-non-owner"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_migration_source_file(metadata: &fs::Metadata) -> Result<(), RunError> {
    if !metadata.file_type().is_file() {
        return Err(state_error("migration-target-unsafe"));
    }
    Ok(())
}

#[cfg(unix)]
fn harden_explicit_migration_permissions(path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt as _;
    use std::os::unix::fs::PermissionsExt as _;

    let parent = path.parent().ok_or_else(|| state_error("parent-missing"))?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|_| state_error("parent-inspection-failed"))?;
    if !parent_metadata.file_type().is_dir() || parent_metadata.uid() != unsafe { libc::geteuid() } {
        return Err(state_error("migration-parent-unsafe"));
    }
    let file_metadata = fs::symlink_metadata(path).map_err(|_| state_error("target-inspection-failed"))?;
    if !file_metadata.file_type().is_file() || file_metadata.uid() != unsafe { libc::geteuid() } {
        return Err(state_error("migration-target-unsafe"));
    }
    fs::set_permissions(parent, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE))
        .map_err(|_| state_error("migration-parent-permissions-failed"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_FILE_MODE))
        .map_err(|_| state_error("migration-target-permissions-failed"))?;
    Ok(())
}

#[cfg(not(unix))]
fn harden_explicit_migration_permissions(_path: &Path) -> Result<(), RunError> {
    Ok(())
}

struct ZeroizingJsonValue {
    value: serde_json::Value,
}

impl ZeroizingJsonValue {
    fn parse(bytes: &[u8]) -> Result<Self, RunError> {
        let value = serde_json::from_slice(bytes).map_err(|_| state_error("malformed"))?;
        Ok(Self { value })
    }

    fn as_value(&self) -> &serde_json::Value {
        &self.value
    }

    fn take(&mut self) -> serde_json::Value {
        std::mem::take(&mut self.value)
    }

    fn zeroize_strings(&mut self) {
        zeroize_json_value(&mut self.value);
    }
}

impl Drop for ZeroizingJsonValue {
    fn drop(&mut self) {
        self.zeroize_strings();
    }
}

fn zeroize_json_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(string) => string.zeroize(),
        serde_json::Value::Array(values) => {
            for nested in values {
                zeroize_json_value(nested);
            }
        }
        serde_json::Value::Object(fields) => {
            let owned_fields = std::mem::take(fields);
            for (mut key, mut nested) in owned_fields {
                key.zeroize();
                zeroize_json_value(&mut nested);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn json_contains_field(value: &serde_json::Value, field: &str) -> bool {
    match value {
        serde_json::Value::Array(values) => values.iter().any(|nested| json_contains_field(nested, field)),
        serde_json::Value::Object(fields) => {
            fields.contains_key(field) || fields.values().any(|nested| json_contains_field(nested, field))
        }
        _ => false,
    }
}

fn state_schema_version(value: &serde_json::Value) -> Result<u32, RunError> {
    let object = value.as_object().ok_or_else(|| state_error("malformed"))?;
    let Some(version) = object.get("schema_version") else {
        if object.get("tickets").is_some_and(serde_json::Value::is_object) {
            return Ok(LEGACY_TICKET_STATE_SCHEMA_VERSION);
        }
        return Err(state_error("schema-version-missing"));
    };
    let version = version.as_u64().ok_or_else(|| state_error("schema-version-malformed"))?;
    u32::try_from(version).map_err(|_| state_error("schema-version-unsupported"))
}

fn legacy_ticket_ids(value: &serde_json::Value) -> Result<Vec<String>, RunError> {
    let tickets = value
        .get("tickets")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| state_error("legacy-tickets-malformed"))?;
    if tickets.len() > MAX_INVALIDATED_LEGACY_TICKET_IDS {
        return Err(state_error("legacy-ticket-count-exceeded"));
    }
    let mut ids = BTreeSet::new();
    for (map_id, record) in tickets {
        let record = record.as_object().ok_or_else(|| state_error("legacy-ticket-malformed"))?;
        let record_id = record
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| state_error("legacy-ticket-id-malformed"))?;
        if record_id != map_id {
            return Err(state_error("legacy-ticket-id-mismatch"));
        }
        if !record.get("secret").is_some_and(serde_json::Value::is_string) {
            return Err(state_error("legacy-ticket-secret-missing"));
        }
        ids.insert(record_id.to_string());
    }
    Ok(ids.into_iter().collect())
}

fn read_private_state_file(path: &Path) -> Result<Option<Zeroizing<Vec<u8>>>, RunError> {
    let Some(parent) = path.parent() else {
        return Err(state_error("parent-missing"));
    };
    match fs::symlink_metadata(parent) {
        Ok(metadata) => validate_private_state_directory_metadata(&metadata)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(state_error("parent-inspection-failed")),
    }
    let mut file = match open_state_read_no_follow(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(state_error("open-failed")),
    };
    validate_private_state_file_metadata(&file.metadata().map_err(|_| state_error("metadata-failed"))?)?;
    let limit_with_probe =
        REMOTE_TICKET_STATE_BYTES_MAX.checked_add(1).ok_or_else(|| state_error("size-limit-overflow"))?;
    let mut bytes = Zeroizing::new(Vec::new());
    std::io::Read::by_ref(&mut file)
        .take(limit_with_probe)
        .read_to_end(&mut bytes)
        .map_err(|_| state_error("read-failed"))?;
    let observed_bytes = u64::try_from(bytes.len()).map_err(|_| state_error("size-conversion-failed"))?;
    if observed_bytes > REMOTE_TICKET_STATE_BYTES_MAX {
        return Err(state_error("size-limit-exceeded"));
    }
    Ok(Some(bytes))
}

fn ensure_private_state_directory(parent: &Path) -> Result<(), RunError> {
    match fs::symlink_metadata(parent) {
        Ok(metadata) => validate_private_state_directory_metadata(&metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => create_private_state_directory(parent),
        Err(_) => Err(state_error("parent-inspection-failed")),
    }
}

#[cfg(unix)]
fn create_private_state_directory(parent: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::DirBuilderExt as _;

    let ancestor = parent.parent().ok_or_else(|| state_error("parent-ancestor-missing"))?;
    fs::create_dir_all(ancestor).map_err(|_| state_error("parent-ancestor-create-failed"))?;
    let mut builder = fs::DirBuilder::new();
    builder.mode(PRIVATE_DIRECTORY_MODE);
    match builder.create(parent) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(state_error("parent-create-failed")),
    }
    let metadata = fs::symlink_metadata(parent).map_err(|_| state_error("parent-inspection-failed"))?;
    validate_private_state_directory_metadata(&metadata)
}

#[cfg(not(unix))]
fn create_private_state_directory(parent: &Path) -> Result<(), RunError> {
    fs::create_dir_all(parent).map_err(|_| state_error("parent-create-failed"))?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| state_error("parent-inspection-failed"))?;
    validate_private_state_directory_metadata(&metadata)
}

fn validate_existing_state_target(path: &Path) -> Result<(), RunError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_private_state_file_metadata(&metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(state_error("target-inspection-failed")),
    }
}

#[cfg(unix)]
fn validate_private_state_directory_metadata(metadata: &fs::Metadata) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt as _;

    if !metadata.file_type().is_dir() {
        return Err(state_error("parent-not-directory"));
    }
    let mode = metadata.mode();
    if mode & NON_OWNER_PERMISSION_BITS != 0
        || mode & PRIVATE_DIRECTORY_REQUIRED_OWNER_BITS != PRIVATE_DIRECTORY_REQUIRED_OWNER_BITS
    {
        return Err(state_error("parent-permissions-unsafe"));
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(state_error("parent-owner-unsafe"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_state_directory_metadata(metadata: &fs::Metadata) -> Result<(), RunError> {
    if !metadata.file_type().is_dir() {
        return Err(state_error("parent-not-directory"));
    }
    Ok(())
}

#[cfg(unix)]
fn validate_private_state_file_metadata(metadata: &fs::Metadata) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt as _;

    if !metadata.file_type().is_file() {
        return Err(state_error("target-not-regular"));
    }
    let mode = metadata.mode();
    if mode & NON_OWNER_PERMISSION_BITS != 0
        || mode & PRIVATE_FILE_REQUIRED_OWNER_BITS != PRIVATE_FILE_REQUIRED_OWNER_BITS
    {
        return Err(state_error("target-permissions-unsafe"));
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(state_error("target-owner-unsafe"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_state_file_metadata(metadata: &fs::Metadata) -> Result<(), RunError> {
    if !metadata.file_type().is_file() {
        return Err(state_error("target-not-regular"));
    }
    Ok(())
}

#[cfg(unix)]
fn open_state_read_no_follow(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)
}

#[cfg(not(unix))]
fn open_state_read_no_follow(path: &Path) -> std::io::Result<File> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "state symlink rejected"));
    }
    OpenOptions::new().read(true).open(path)
}

#[cfg(unix)]
fn open_private_state_lock(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .mode(PRIVATE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
}

#[cfg(not(unix))]
fn open_private_state_lock(path: &Path) -> std::io::Result<File> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "state lock symlink rejected"));
    }
    OpenOptions::new().create(true).read(true).write(true).open(path)
}

fn create_private_state_temp(parent: &Path) -> Result<(PathBuf, File), RunError> {
    for _attempt in 0..STATE_TEMP_OPEN_ATTEMPTS_MAX {
        let suffix = random_temp_suffix()?;
        let path = parent.join(format!("{STATE_TEMP_PREFIX}{suffix}"));
        match open_state_temp_no_follow(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(state_error("temp-open-failed")),
        }
    }
    Err(state_error("temp-open-attempts-exhausted"))
}

#[cfg(unix)]
fn open_state_temp_no_follow(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(PRIVATE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
}

#[cfg(not(unix))]
fn open_state_temp_no_follow(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().create_new(true).write(true).open(path)
}

fn random_temp_suffix() -> Result<String, RunError> {
    let mut random = [0_u8; STATE_TEMP_RANDOM_BYTES];
    OsRng.try_fill_bytes(&mut random).map_err(|_| state_error("temp-random-failed"))?;
    let capacity = STATE_TEMP_RANDOM_BYTES
        .checked_mul(HEX_CHARS_PER_BYTE)
        .ok_or_else(|| state_error("temp-name-capacity-overflow"))?;
    let mut suffix = String::with_capacity(capacity);
    for byte in random {
        use std::fmt::Write as _;
        write!(&mut suffix, "{byte:02x}").map_err(|_| state_error("temp-name-format-failed"))?;
    }
    Ok(suffix)
}

fn write_and_commit_state(
    path: &Path,
    parent: &Path,
    tmp_path: &Path,
    file: &mut File,
    rendered: &[u8],
) -> Result<(), RunError> {
    file.write_all(rendered).map_err(|_| state_error("temp-write-failed"))?;
    file.sync_all().map_err(|_| state_error("temp-sync-failed"))?;
    validate_private_state_file_metadata(&file.metadata().map_err(|_| state_error("temp-metadata-failed"))?)?;
    fs::rename(tmp_path, path).map_err(|_| state_error("atomic-replace-failed"))?;
    sync_state_parent(parent)?;
    Ok(())
}

#[cfg(unix)]
fn sync_state_parent(parent: &Path) -> Result<(), RunError> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| state_error("parent-sync-failed"))
}

#[cfg(not(unix))]
fn sync_state_parent(_parent: &Path) -> Result<(), RunError> {
    Ok(())
}

fn state_error(category: &str) -> RunError {
    RunError::Internal(format!("remote-ticket-state-{category}"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    use super::*;
    use crate::remote_credentials::TICKET_ENTROPY_BYTES;
    use crate::remote_credentials::TICKET_VERIFIER_KEY_BYTES;
    use crate::remote_credentials::TicketIssueInput;
    use crate::remote_credentials::TicketVerifierKey;
    use crate::remote_credentials::apply_ticket_issue;
    use crate::remote_credentials::plan_ticket_issue;

    const TEST_KEY_BYTE: u8 = 0x61;
    const TEST_ENTROPY_BYTE: u8 = 0x42;
    const TEST_NOW_UNIX_S: u64 = 100;
    const TEST_TTL_SECS: u64 = 300;
    const TEST_MAX_BUILD_TIME_SECS: u64 = 60;
    const TEST_MAX_UPLOAD_BYTES: u64 = 1_024;
    const TEST_LOCK_WAIT_TIMEOUT_MS: u64 = 30;
    const TEST_LOCK_POLL_INTERVAL_MS: u64 = 1;

    fn populated_state() -> RemoteTicketState {
        let mut state = RemoteTicketState::default();
        let encoded = URL_SAFE_NO_PAD.encode([TEST_KEY_BYTE; TICKET_VERIFIER_KEY_BYTES]);
        let key = Arc::new(TicketVerifierKey::parse(&format!("ticket-key-1:{encoded}")).unwrap());
        let plan = plan_ticket_issue(
            &state,
            TicketIssueInput {
                display_name: "state test".to_string(),
                now_unix_s: TEST_NOW_UNIX_S,
                ttl_secs: TEST_TTL_SECS,
                uses: 1,
                max_build_time_secs: TEST_MAX_BUILD_TIME_SECS,
                max_upload_bytes: TEST_MAX_UPLOAD_BYTES,
                bound_client_endpoint: None,
            },
            &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES],
            &key,
        )
        .unwrap();
        apply_ticket_issue(&mut state, &plan).unwrap();
        state
    }

    #[test]
    fn parsed_legacy_json_cleanup_wipes_nested_strings_and_keys() {
        let source = br#"{"tickets":{"legacy-a":{"secret":"plaintext-a"}},"list":["plaintext-b"]}"#;
        let mut value = ZeroizingJsonValue::parse(source).unwrap();
        assert!(value.as_value().to_string().contains("plaintext-a"));
        assert!(value.as_value().to_string().contains("plaintext-b"));

        value.zeroize_strings();

        assert_eq!(value.as_value(), &serde_json::json!({}));
        assert!(!value.as_value().to_string().contains("plaintext-a"));
        assert!(!value.as_value().to_string().contains("plaintext-b"));
    }

    #[test]
    fn private_state_round_trip_uses_versioned_verifier_schema() {
        let temp = tempfile::tempdir().unwrap();
        let state = populated_state();
        save_ticket_state(temp.path(), &state).unwrap();
        let loaded = load_ticket_state(temp.path()).unwrap();
        let rendered = fs::read_to_string(ticket_state_path(temp.path())).unwrap();

        assert_eq!(loaded, state);
        assert!(rendered.contains("\"schema_version\": 2"));
        assert!(rendered.contains("\"verifier\""));
        assert!(!rendered.contains("\"secret\""));
    }

    #[test]
    fn mutation_lock_has_bounded_busy_timeout() {
        let temp = tempfile::tempdir().unwrap();
        let first = acquire_ticket_state_mutation_guard(temp.path()).unwrap();
        let second = acquire_ticket_state_mutation_guard_with_wait(
            temp.path(),
            Duration::from_millis(TEST_LOCK_WAIT_TIMEOUT_MS),
            Duration::from_millis(TEST_LOCK_POLL_INTERVAL_MS),
        )
        .unwrap_err();

        assert_eq!(second.to_string(), "error: remote-ticket-state-lock-timeout");
        drop(first);
        assert!(acquire_ticket_state_mutation_guard(temp.path()).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn explicit_unlock_releases_lock_with_inherited_like_descriptor_open() {
        use std::os::fd::AsRawFd as _;

        let temp = tempfile::tempdir().unwrap();
        let first = acquire_ticket_state_mutation_guard(temp.path()).unwrap();
        let inherited_fd = unsafe { libc::dup(first.file.as_raw_fd()) };
        assert!(inherited_fd >= 0);
        drop(first);

        let second = acquire_ticket_state_mutation_guard_with_wait(
            temp.path(),
            Duration::from_millis(TEST_LOCK_WAIT_TIMEOUT_MS),
            Duration::from_millis(TEST_LOCK_POLL_INTERVAL_MS),
        );
        assert!(second.is_ok());
        assert_eq!(unsafe { libc::close(inherited_fd) }, 0);
    }

    #[cfg(unix)]
    #[test]
    fn state_rejects_links_and_unsafe_permissions() {
        use std::os::unix::fs::PermissionsExt as _;
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let state = populated_state();
        save_ticket_state(temp.path(), &state).unwrap();
        let path = ticket_state_path(temp.path());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(load_ticket_state(temp.path()).is_err());

        fs::remove_file(&path).unwrap();
        let outside = temp.path().join("outside");
        fs::write(&outside, b"{}").unwrap();
        symlink(&outside, &path).unwrap();
        assert!(load_ticket_state(temp.path()).is_err());
    }

    #[test]
    fn malformed_and_unknown_versions_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        ensure_private_state_directory(ticket_state_path(temp.path()).parent().unwrap()).unwrap();
        let path = ticket_state_path(temp.path());
        write_private_fixture(&path, b"not-json");
        assert!(load_ticket_state(temp.path()).is_err());

        write_private_fixture(&path, br#"{"schema_version":99,"next_ticket_sequence":0,"tickets":{}}"#);
        assert!(load_ticket_state(temp.path()).is_err());
    }

    #[test]
    fn invalidating_legacy_migration_supports_dry_run_and_execution() {
        let temp = tempfile::tempdir().unwrap();
        ensure_private_state_directory(ticket_state_path(temp.path()).parent().unwrap()).unwrap();
        let legacy = br#"{
          "tickets": {
            "legacy-a": {"id":"legacy-a","secret":"plaintext-a"},
            "legacy-b": {"id":"legacy-b","secret":"plaintext-b"}
          }
        }"#;
        write_private_fixture(&ticket_state_path(temp.path()), legacy);
        assert!(load_ticket_state(temp.path()).is_err());

        let path = ticket_state_path(temp.path());
        let mode_before_dry_run = private_fixture_mode(&path);
        let dry_run = migrate_legacy_ticket_state(temp.path(), true, true).unwrap();
        assert_eq!(dry_run.invalidated_ticket_count, 2);
        assert!(!dry_run.executed);
        assert!(fs::read_to_string(&path).unwrap().contains("plaintext-a"));
        assert_eq!(private_fixture_mode(&path), mode_before_dry_run);

        let executed = migrate_legacy_ticket_state(temp.path(), true, false).unwrap();
        let loaded = load_ticket_state(temp.path()).unwrap();
        let rendered = fs::read_to_string(ticket_state_path(temp.path())).unwrap();
        assert!(executed.executed);
        assert_eq!(loaded.invalidated_legacy_ticket_ids.len(), 2);
        assert!(!rendered.contains("plaintext-a"));
        assert!(!rendered.contains("plaintext-b"));
        assert!(!rendered.contains("\"secret\""));
    }

    #[test]
    fn stale_temp_file_does_not_replace_committed_state() {
        let temp = tempfile::tempdir().unwrap();
        let state = populated_state();
        save_ticket_state(temp.path(), &state).unwrap();
        let path = ticket_state_path(temp.path());
        let parent = path.parent().unwrap();
        let stale = parent.join(format!("{STATE_TEMP_PREFIX}stale"));
        write_private_fixture(&stale, b"partial");

        save_ticket_state(temp.path(), &state).unwrap();
        let loaded = load_ticket_state(temp.path()).unwrap();
        assert_eq!(loaded, state);
        assert_eq!(fs::read(&stale).unwrap(), b"partial");
    }

    #[cfg(unix)]
    fn private_fixture_mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt as _;
        fs::metadata(path).unwrap().permissions().mode()
    }

    #[cfg(not(unix))]
    fn private_fixture_mode(_path: &Path) -> u32 {
        0
    }

    fn write_private_fixture(path: &Path, bytes: &[u8]) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .mode(PRIVATE_FILE_MODE)
                .open(path)
                .unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        #[cfg(not(unix))]
        fs::write(path, bytes).unwrap();
    }
}
