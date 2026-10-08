//! One validated, content-addressed file per declared retention interest.
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crunch_gc_core::interest::INTEREST_SCHEMA_VERSION;
use crunch_gc_core::interest::InterestFact;
use crunch_gc_core::interest::MAX_INTEREST_BYTES;
use crunch_gc_core::interest::MAX_INTEREST_RECORDS;
use data_encoding::HEXLOWER;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;

use crate::Error;
use crate::roots::GcRootRecord;

const RECORD_DIRECTORY: &str = "retention-interests";
const RECORD_EXTENSION: &str = "json";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

// r[impl mantle.store_lifecycle.retention_interest_records]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InterestRecord {
    pub schema_version: u32,
    pub owner: String,
    pub logical_path: String,
    pub reason: String,
    pub declaration: GcRootRecord,
}

impl InterestRecord {
    pub(crate) fn new(declaration: GcRootRecord) -> Self {
        Self {
            schema_version: INTEREST_SCHEMA_VERSION,
            owner: declaration.owner_scope.clone(),
            logical_path: declaration.logical_path.clone(),
            reason: declaration.last_transition_reason.clone(),
            declaration,
        }
    }

    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != INTEREST_SCHEMA_VERSION {
            return Err(invalid("unsupported retention interest version"));
        }
        if self.owner.is_empty()
            || self.owner.len() > 256
            || self.logical_path.len() > 4_096
            || self.reason.is_empty()
            || self.reason.len() > 256
        {
            return Err(invalid("retention interest field exceeds bound or is empty"));
        }
        if self.owner != self.declaration.owner_scope
            || self.logical_path != self.declaration.logical_path
            || self.reason != self.declaration.last_transition_reason
        {
            return Err(invalid("retention interest declaration mismatch"));
        }
        let Some((prefix, basename)) = self.logical_path.rsplit_once('/') else {
            return Err(invalid("retention interest path is not an absolute store path"));
        };
        if !prefix.starts_with('/')
            || prefix
                .split('/')
                .skip(1)
                .any(|component| component.is_empty() || component == "." || component == "..")
            || nix_compat::store_path::StorePath::<String>::from_bytes(basename.as_bytes()).is_err()
        {
            return Err(invalid("retention interest path is not a canonical store path"));
        }
        crate::roots::validate_record(&self.declaration)
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let bytes =
            serde_json::to_vec(self).map_err(|error| invalid(format!("encoding retention interest: {error}")))?;
        if bytes.len() > MAX_INTEREST_BYTES {
            return Err(invalid("retention interest exceeds size limit"));
        }
        Ok(bytes)
    }
}

fn invalid(message: impl std::fmt::Display) -> Error {
    Error::RootRegistry(message.to_string())
}

fn identity(bytes: &[u8]) -> String {
    HEXLOWER.encode(&crunch_gc_core::interest::identity(bytes))
}

fn directory(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join(RECORD_DIRECTORY)
}

fn read_file(path: &Path) -> Result<(String, InterestRecord), Error> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|error| invalid(format!("observing {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_INTEREST_BYTES as u64 {
        return Err(invalid(format!("invalid or oversized retention interest {}", path.display())));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| invalid(format!("opening {}: {error}", path.display())))?
        .take(MAX_INTEREST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| invalid(format!("reading {}: {error}", path.display())))?;
    if bytes.len() > MAX_INTEREST_BYTES {
        return Err(invalid("retention interest exceeds size limit"));
    }
    let record: InterestRecord =
        serde_json::from_slice(&bytes).map_err(|error| invalid(format!("parsing {}: {error}", path.display())))?;
    let expected = record.canonical_bytes()?;
    if bytes != expected {
        return Err(invalid(format!("noncanonical retention interest {}", path.display())));
    }
    let digest = identity(&bytes);
    if path.file_name().and_then(|name| name.to_str()) != Some(format!("{digest}.{RECORD_EXTENSION}").as_str()) {
        return Err(invalid(format!("retention interest identity mismatch {}", path.display())));
    }
    Ok((digest, record))
}

pub(crate) fn load(state_dir: &Path) -> Result<Vec<(String, InterestRecord)>, Error> {
    let dir = directory(state_dir);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(invalid(format!("reading {}: {error}", dir.display()))),
    };
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| invalid(format!("enumerating {}: {error}", dir.display())))?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(".pending-") {
            continue;
        }
        if records.len() >= MAX_INTEREST_RECORDS {
            return Err(invalid("retention interest record count exceeds limit"));
        }
        records.push(read_file(&entry.path())?);
    }
    records.sort_by(|left, right| left.0.cmp(&right.0));
    let mut keys = BTreeMap::new();
    for (_, record) in &records {
        let key = (&record.owner, &record.logical_path, &record.reason);
        if keys.insert(key, ()).is_some() {
            return Err(invalid("duplicate retention interest for owner/path/reason"));
        }
    }
    let facts = facts(&records);
    crunch_gc_core::interest::merge(&facts).map_err(invalid)?;
    Ok(records)
}

fn facts(records: &[(String, InterestRecord)]) -> Vec<InterestFact> {
    records
        .iter()
        .map(|(id, record)| InterestFact {
            identity: id.clone(),
            owner: record.owner.clone(),
            path: record.logical_path.clone(),
            reason: record.reason.clone(),
        })
        .collect()
}

fn lock(state_dir: &Path) -> Result<std::fs::File, Error> {
    std::fs::create_dir_all(state_dir)
        .map_err(|error| invalid(format!("creating {}: {error}", state_dir.display())))?;
    let path = state_dir.join("retention-interests.lock");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|error| invalid(format!("opening {}: {error}", path.display())))?;
    file.lock_exclusive().map_err(|error| invalid(format!("locking {}: {error}", path.display())))?;
    Ok(file)
}

// r[impl mantle.store_lifecycle.retention_owner_scope]
pub(crate) fn publish(state_dir: &Path, record: &InterestRecord) -> Result<(), Error> {
    publish_with_replacement(state_dir, record, false)
}

/// Automatic root re-registration changes the transition reason on renewal.
/// Replace that owner's prior declaration without affecting other owners or
/// separately reasoned operator pins.
pub(crate) fn publish_replacing_owner(state_dir: &Path, record: &InterestRecord) -> Result<(), Error> {
    publish_with_replacement(state_dir, record, true)
}

fn publish_with_replacement(state_dir: &Path, record: &InterestRecord, replace_owner: bool) -> Result<(), Error> {
    let bytes = record.canonical_bytes()?;
    crate::roots::ensure_root_mutation_allowed(state_dir)?;
    let _lock = lock(state_dir)?;
    let prior = load(state_dir)?;
    let digest = identity(&bytes);
    let already_published = prior.iter().any(|(id, _)| id == &digest);
    let replaces = |existing: &InterestRecord| {
        existing.owner == record.owner
            && existing.logical_path == record.logical_path
            && (replace_owner || existing.reason == record.reason)
    };
    let needs_cleanup = prior.iter().any(|(id, existing)| id != &digest && replaces(existing));
    if already_published && !needs_cleanup {
        return Ok(());
    }
    if prior.len() >= MAX_INTEREST_RECORDS && !already_published && !needs_cleanup {
        return Err(invalid("retention interest record count exceeds limit"));
    }
    let dir = directory(state_dir);
    if !already_published {
        std::fs::create_dir_all(&dir).map_err(|error| invalid(format!("creating {}: {error}", dir.display())))?;
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let pending = dir.join(format!(".pending-{}-{sequence}", std::process::id()));
        let target = dir.join(format!("{digest}.{RECORD_EXTENSION}"));
        let result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&pending)
                .map_err(|error| invalid(format!("creating {}: {error}", pending.display())))?;
            file.write_all(&bytes).map_err(|error| invalid(format!("writing {}: {error}", pending.display())))?;
            file.sync_all().map_err(|error| invalid(format!("syncing {}: {error}", pending.display())))?;
            std::fs::hard_link(&pending, &target)
                .map_err(|error| invalid(format!("publishing {}: {error}", target.display())))?;
            std::fs::File::open(&dir)
                .and_then(|file| file.sync_all())
                .map_err(|error| invalid(format!("syncing {}: {error}", dir.display())))?;
            Ok(())
        })();
        if pending.exists() {
            std::fs::remove_file(&pending)
                .map_err(|error| invalid(format!("cleaning {}: {error}", pending.display())))?;
        }
        result?;
    }
    if needs_cleanup {
        for (old_id, existing) in &prior {
            if old_id != &digest && replaces(existing) {
                std::fs::remove_file(dir.join(format!("{old_id}.{RECORD_EXTENSION}")))
                    .map_err(|error| invalid(format!("replacing retention interest {old_id}: {error}")))?;
            }
        }
        std::fs::File::open(&dir)
            .and_then(|file| file.sync_all())
            .map_err(|error| invalid(format!("syncing {}: {error}", dir.display())))?;
    }
    Ok(())
}

pub(crate) fn release(
    state_dir: &Path,
    owner: &str,
    logical_path: &str,
    reason: Option<&str>,
) -> Result<Option<GcRootRecord>, Error> {
    crate::roots::ensure_root_mutation_allowed(state_dir)?;
    let _lock = lock(state_dir)?;
    let records = load(state_dir)?;
    let ids =
        crunch_gc_core::interest::release_identities(&facts(&records), owner, logical_path, reason).map_err(invalid)?;
    let mut removed = None;
    for id in ids {
        let record = records
            .iter()
            .find(|(identity, _)| identity == &id)
            .ok_or_else(|| invalid("retention interest release identity disappeared"))?;
        std::fs::remove_file(directory(state_dir).join(format!("{id}.{RECORD_EXTENSION}")))
            .map_err(|error| invalid(format!("releasing interest {id}: {error}")))?;
        removed = Some(record.1.declaration.clone());
    }
    if removed.is_some() {
        let dir = directory(state_dir);
        std::fs::File::open(&dir)
            .and_then(|file| file.sync_all())
            .map_err(|error| invalid(format!("syncing {} after release: {error}", dir.display())))?;
    }
    Ok(removed)
}

#[cfg(test)]
pub(crate) fn per_path(state_dir: &Path) -> Result<BTreeMap<String, Vec<InterestRecord>>, Error> {
    let mut grouped = BTreeMap::<String, Vec<InterestRecord>>::new();
    for (_, record) in load(state_dir)? {
        grouped.entry(record.logical_path.clone()).or_default().push(record);
    }
    Ok(grouped)
}
