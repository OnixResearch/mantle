// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::ambient_clock)]

//! Thin durable shell for the combined quota ledger. The stable lock covers
//! read/decision/atomic replacement; no admission is acknowledged before fsync.
//! A publication/fsync error is uncertain, not a release. Retrying the identical
//! attempt operation reads the durable state and cannot double reserve/charge.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_remote_core::resource_quota::Entry;
use crunch_remote_core::resource_quota::Ledger;
use crunch_remote_core::resource_quota::Limits;
use crunch_remote_core::resource_quota::QuotaError;
use crunch_remote_core::resource_quota::Reservation;
use crunch_remote_core::resource_quota::UsageEvent;
use fs2::FileExt;

pub const MAX_LEDGER_BYTES: u64 = 8_388_608;
pub const QUOTA_LOCK_TIMEOUT_MS: u64 = 250;
pub const QUOTA_LOCK_POLL_MS: u64 = 5;
/// Upper bound on exclusive-lock attempts: the timeout window divided by the
/// poll interval, plus the final attempt that observes the elapsed timeout.
const QUOTA_LOCK_MAX_ATTEMPTS: u64 = 51;

#[cfg(unix)]
fn private_options() -> OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK);
    options
}
#[cfg(not(unix))]
fn private_options() -> OpenOptions {
    OpenOptions::new()
}

fn private_file(path: &Path) -> Result<File, QuotaError> {
    let file = private_options().read(true).open(path).map_err(|_| QuotaError::StoreUnavailable)?;
    validate_private_file(&file)?;
    Ok(file)
}
fn validate_private_file(file: &File) -> Result<(), QuotaError> {
    let metadata = file.metadata().map_err(|_| QuotaError::StoreUnavailable)?;
    debug_assert!(metadata.is_file(), "quota store file must be a regular file");
    debug_assert!(!metadata.is_symlink(), "quota store file must not be a symlink");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let is_owned_by_caller = metadata.uid() == unsafe { libc::geteuid() };
        let is_private_mode = metadata.mode() & 0o077 == 0;
        let is_single_link = metadata.nlink() == 1;
        let is_private_file = metadata.is_file() && is_owned_by_caller && is_private_mode && is_single_link;
        if !is_private_file {
            return Err(QuotaError::StoreUnavailable);
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Err(QuotaError::StoreUnavailable)
    }
}
fn private_directory(path: &Path) -> Result<File, QuotaError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::OpenOptionsExt;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(path)
            .map_err(|_| QuotaError::StoreUnavailable)?;
        let metadata = file.metadata().map_err(|_| QuotaError::StoreUnavailable)?;
        if !metadata.is_dir() || metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(QuotaError::StoreUnavailable);
        }
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(QuotaError::StoreUnavailable)
    }
}
/// The explicit empty ledger written when the store is first provisioned.
fn empty_ledger() -> Ledger {
    Ledger {
        schema: crunch_remote_core::resource_quota::LEDGER_SCHEMA.to_string(),
        revision: 0,
        entries: BTreeMap::new(),
        observations: BTreeMap::new(),
        resource_contexts: BTreeMap::new(),
    }
}
#[derive(Debug, Clone)]
pub struct ResourceQuotaStore {
    directory: PathBuf,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaStoreFault {
    None,
    BeforePublish,
    AfterPublish,
}
impl ResourceQuotaStore {
    /// Explicit first-time provisioning, not a fallback for an unreadable store.
    /// Existing state is validated and never reset.
    pub fn provision(directory: &Path) -> Result<Self, QuotaError> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(directory).map_err(|_| QuotaError::StoreUnavailable)?;
        private_directory(directory)?;
        let store = Self {
            directory: directory.to_path_buf(),
        };
        let _lock = store.lock()?;
        match fs::symlink_metadata(store.state_path()) {
            Ok(_) => {
                store.read_locked()?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // A retained sentinel prevents accidental reset after lost state.
                // Interrupted initial provisioning fails closed for operator recovery.
                let marker = private_options()
                    .write(true)
                    .create_new(true)
                    .open(directory.join("resource-quota.initialized"))
                    .map_err(|_| QuotaError::StoreUnavailable)?;
                marker.sync_all().map_err(|_| QuotaError::StoreUnavailable)?;
                private_directory(directory)?.sync_all().map_err(|_| QuotaError::StoreUnavailable)?;
                store.publish(&empty_ledger(), QuotaStoreFault::None)?;
            }
            Err(_) => return Err(QuotaError::StoreUnavailable),
        }
        Ok(store)
    }
    pub fn open(directory: &Path) -> Result<Self, QuotaError> {
        let store = Self {
            directory: directory.to_path_buf(),
        };
        let _lock = store.lock()?;
        store.read_locked()?;
        Ok(store)
    }
    fn state_path(&self) -> PathBuf {
        self.directory.join("resource-quota.json")
    }
    fn lock(&self) -> Result<File, QuotaError> {
        private_directory(&self.directory)?;
        let path = self.directory.join("resource-quota.lock");
        let file = private_options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|_| QuotaError::StoreUnavailable)?;
        validate_private_file(&file)?;
        let started = std::time::Instant::now();
        let timeout_ms = QUOTA_LOCK_TIMEOUT_MS;
        let poll_ms = QUOTA_LOCK_POLL_MS;
        for _attempt in 0..QUOTA_LOCK_MAX_ATTEMPTS {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(file),
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && started.elapsed() < std::time::Duration::from_millis(timeout_ms) =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(poll_ms));
                }
                Err(_) => return Err(QuotaError::StoreUnavailable),
            }
        }
        Err(QuotaError::StoreUnavailable)
    }
    fn read_locked(&self) -> Result<Ledger, QuotaError> {
        let marker = private_file(&self.directory.join("resource-quota.initialized"))?;
        if marker.metadata().map_err(|_| QuotaError::StoreUnavailable)?.len() != 0 {
            return Err(QuotaError::StoreUnavailable);
        }
        let file = private_file(&self.state_path())?;
        if file.metadata().map_err(|_| QuotaError::StoreUnavailable)?.len() > MAX_LEDGER_BYTES {
            return Err(QuotaError::StoreUnavailable);
        }
        let mut bytes = Vec::new();
        file.take(MAX_LEDGER_BYTES.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| QuotaError::StoreUnavailable)?;
        if bytes.len() as u64 > MAX_LEDGER_BYTES {
            return Err(QuotaError::StoreUnavailable);
        }
        let ledger: Ledger = serde_json::from_slice(&bytes).map_err(|_| QuotaError::StoreUnavailable)?;
        ledger.validate().map_err(|_| QuotaError::StoreUnavailable)?;
        if serde_json::to_vec(&ledger).map_err(|_| QuotaError::StoreUnavailable)? != bytes {
            return Err(QuotaError::StoreUnavailable);
        }
        Ok(ledger)
    }
    fn publish(&self, ledger: &Ledger, fault: QuotaStoreFault) -> Result<(), QuotaError> {
        ledger.validate()?;
        let bytes = serde_json::to_vec(ledger).map_err(|_| QuotaError::StoreUnavailable)?;
        if bytes.len() as u64 > MAX_LEDGER_BYTES {
            return Err(QuotaError::Full);
        }
        let mut temporary =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|_| QuotaError::StoreUnavailable)?;
        validate_private_file(temporary.as_file())?;
        temporary
            .write_all(&bytes)
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|_| QuotaError::StoreUnavailable)?;
        if fault == QuotaStoreFault::BeforePublish {
            return Err(QuotaError::StoreUnavailable);
        }
        temporary.persist(self.state_path()).map_err(|_| QuotaError::StoreUncertain)?;
        if fault == QuotaStoreFault::AfterPublish {
            return Err(QuotaError::StoreUncertain);
        }
        private_directory(&self.directory)?.sync_all().map_err(|_| QuotaError::StoreUncertain)
    }
    pub fn snapshot(&self) -> Result<Ledger, QuotaError> {
        let _lock = self.lock()?;
        self.read_locked()
    }
    pub fn record_observation(
        &self,
        reservation: &Reservation,
        record: crunch_remote_core::resource_quota::ObservationRecord,
    ) -> Result<(), QuotaError> {
        let _lock = self.lock()?;
        let mut ledger = self.read_locked()?;
        let revision = ledger.revision;
        ledger.record_observation(reservation, record)?;
        if ledger.revision != revision {
            self.publish(&ledger, QuotaStoreFault::None)?;
        }
        private_directory(&self.directory)?.sync_all().map_err(|_| QuotaError::StoreUncertain)?;
        Ok(())
    }
    /// Quota and the authenticated host's selected-resource provenance become
    /// durable together, before a worker can execute this fenced attempt.
    pub fn reserve_with_resource_context(
        &self,
        reservation: Reservation,
        limits: Limits,
        context: crunch_remote_core::resource_quota::ResourceContext,
    ) -> Result<Entry, QuotaError> {
        let _lock = self.lock()?;
        let mut ledger = self.read_locked()?;
        let revision = ledger.revision;
        let entry = ledger.reserve(reservation, limits)?;
        ledger.record_resource_context(&entry.reservation, context)?;
        if ledger.revision != revision {
            self.publish(&ledger, QuotaStoreFault::None)?;
        }
        private_directory(&self.directory)?.sync_all().map_err(|_| QuotaError::StoreUncertain)?;
        Ok(entry)
    }
    pub fn reserve(&self, reservation: Reservation, limits: Limits) -> Result<Entry, QuotaError> {
        self.reserve_with_fault(reservation, limits, QuotaStoreFault::None)
    }
    fn reserve_with_fault(
        &self,
        reservation: Reservation,
        limits: Limits,
        fault: QuotaStoreFault,
    ) -> Result<Entry, QuotaError> {
        let _lock = self.lock()?;
        let mut ledger = self.read_locked()?;
        let revision = ledger.revision;
        let entry = ledger.reserve(reservation, limits)?;
        if ledger.revision != revision {
            self.publish(&ledger, fault)?;
        }
        private_directory(&self.directory)?.sync_all().map_err(|_| QuotaError::StoreUncertain)?;
        Ok(entry)
    }
    pub fn reconcile(&self, reservation: &Reservation, event: UsageEvent) -> Result<Entry, QuotaError> {
        self.reconcile_with_fault(reservation, event, QuotaStoreFault::None)
    }
    fn reconcile_with_fault(
        &self,
        reservation: &Reservation,
        event: UsageEvent,
        fault: QuotaStoreFault,
    ) -> Result<Entry, QuotaError> {
        let _lock = self.lock()?;
        let mut ledger = self.read_locked()?;
        let revision = ledger.revision;
        let entry = ledger.reconcile(reservation, event)?;
        if ledger.revision != revision {
            self.publish(&ledger, fault)?;
        }
        private_directory(&self.directory)?.sync_all().map_err(|_| QuotaError::StoreUncertain)?;
        Ok(entry)
    }
}

/// Deterministic local fixture/fault entrypoint. This is a filesystem model of
/// interruption boundaries, NOT ChaosControl VM, replay or service evidence.
#[derive(Debug, Clone)]
pub enum QuotaFixtureStep {
    Reserve {
        reservation: Reservation,
        limits: Limits,
        fault: QuotaStoreFault,
    },
    Reconcile {
        reservation: Reservation,
        event: UsageEvent,
        fault: QuotaStoreFault,
    },
    Reopen,
}
#[derive(Debug, Clone)]
pub struct QuotaFixtureReport {
    pub results: Vec<Result<Option<Entry>, QuotaError>>,
    pub ledger: Ledger,
    pub evidence_class: &'static str,
    pub non_claims: &'static [&'static str],
}
pub fn run_quota_fault_fixture(directory: &Path, steps: &[QuotaFixtureStep]) -> Result<QuotaFixtureReport, QuotaError> {
    if steps.len() > crunch_remote_core::resource_quota::MAX_RESERVATIONS {
        return Err(QuotaError::Full);
    }
    let store = ResourceQuotaStore::open(directory)?;
    let mut results = Vec::with_capacity(steps.len());
    for step in steps {
        let result = match step {
            QuotaFixtureStep::Reserve {
                reservation,
                limits,
                fault,
            } => store.reserve_with_fault(reservation.clone(), *limits, *fault).map(Some),
            QuotaFixtureStep::Reconcile {
                reservation,
                event,
                fault,
            } => store.reconcile_with_fault(reservation, event.clone(), *fault).map(Some),
            QuotaFixtureStep::Reopen => ResourceQuotaStore::open(directory).map(|_| None),
        };
        results.push(result);
    }
    debug_assert_eq!(results.len(), steps.len(), "every fixture step yields exactly one result");
    debug_assert!(results.len() <= crunch_remote_core::resource_quota::MAX_RESERVATIONS);
    Ok(QuotaFixtureReport {
        results,
        ledger: store.snapshot()?,
        evidence_class: "local-deterministic-fixture-only",
        non_claims: &crunch_remote_core::resource_policy::NON_CLAIMS,
    })
}

#[cfg(test)]
#[path = "resource_quota_store_tests.rs"]
mod tests;
