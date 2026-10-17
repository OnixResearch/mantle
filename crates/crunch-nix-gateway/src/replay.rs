//! Private durable replay storage for the pinned UCAN `DurableHolderReplayAdmission`.
//!
//! This is only a `ReplayLedgerPort`, not token, holder, revocation, or
//! application-policy verification. No gateway request currently calls it.

use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{self};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use fs2::FileExt as _;
use replay_ledger_core::CommitObservation;
use replay_ledger_core::CompareAndSwapCommand;
use replay_ledger_core::DurabilityObservation;
use replay_ledger_core::RecordVersion;
use replay_ledger_core::ReplayLedgerPort;
use replay_ledger_core::ReplayRecord;
use replay_ledger_core::ReplaySlotIdentity;
use replay_ledger_core::SlotObservation;
use serde::Deserialize;
use serde::Serialize;

const STATE_NAME: &str = "gateway-ucan-replay.json";
const LOCK_NAME: &str = "gateway-ucan-replay.lock";
const INITIALIZED_MARKER: &[u8; 8] = b"MNTLREP1";
const MAX_STATE_BYTES: u64 = 1_048_576;
const MAX_RECORDS: usize = 8_192;
const STATE_VERSION: u32 = 1;
const LOCK_WAIT: Duration = Duration::from_secs(5);
const LOCK_POLL: Duration = Duration::from_millis(10);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerState {
    version: u32,
    records: BTreeMap<String, ReplayRecord>,
}

impl Default for LedgerState {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            records: BTreeMap::new(),
        }
    }
}

/// Caller must configure an owner-controlled 0700 directory and keep the
/// authority generation and ledger namespace consistent with this storage.
/// Never expose this directory to a Nix worker peer or a public endpoint.
pub struct PrivateReplayLedger {
    dir: PathBuf,
}

impl PrivateReplayLedger {
    pub fn new(dir: &Path) -> io::Result<Self> {
        require_private_dir(dir)?;
        Ok(Self { dir: dir.to_path_buf() })
    }

    fn lock(&self, exclusive: bool) -> io::Result<File> {
        self.lock_with_wait(exclusive, LOCK_WAIT)
    }

    fn lock_with_wait(&self, exclusive: bool, max_wait: Duration) -> io::Result<File> {
        require_private_dir(&self.dir)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.dir.join(LOCK_NAME))?;
        require_private_file(&lock)?;
        let start = Instant::now();
        loop {
            let result = if exclusive {
                lock.try_lock_exclusive()
            } else {
                fs2::FileExt::try_lock_shared(&lock)
            };
            match result {
                Ok(()) => break,
                Err(error) if error.kind() == ErrorKind::WouldBlock && start.elapsed() < max_wait => {
                    std::thread::sleep(LOCK_POLL);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    return Err(io::Error::new(ErrorKind::TimedOut, "gateway-replay-lock-timeout"));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(lock)
    }

    fn load_locked(&self, lock: &File) -> io::Result<LedgerState> {
        let initialized = lock_initialized(lock)?;
        let file = match OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(self.dir.join(STATE_NAME)) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound && !initialized => return Ok(LedgerState::default()),
            Err(error) if error.kind() == ErrorKind::NotFound => return Err(invalid_state()),
            Err(error) => return Err(error),
        };
        require_private_file(&file)?;
        if file.metadata()?.len() > MAX_STATE_BYTES {
            return Err(invalid_state());
        }
        let mut bytes = Vec::new();
        file.take(MAX_STATE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(invalid_state());
        }
        let state: LedgerState = serde_json::from_slice(&bytes).map_err(|_| invalid_state())?;
        validate_state(&state)?;
        // All writes are canonical. Reject duplicate JSON fields, duplicate
        // map keys and partial/reformatted state rather than silently dropping
        // a previously committed replay slot during serde deserialization.
        if serde_json::to_vec(&state).map_err(|_| invalid_state())? != bytes {
            return Err(invalid_state());
        }
        Ok(state)
    }

    fn save_locked(&self, lock: &File, state: &LedgerState) -> io::Result<()> {
        validate_state(state)?;
        let bytes = serde_json::to_vec(state).map_err(|_| invalid_state())?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(invalid_state());
        }
        let mut entropy = [0_u8; 16];
        getrandom::fill(&mut entropy).map_err(|_| invalid_state())?;
        if !lock_initialized(lock)? {
            // Mark a previously empty ledger before replacing state. A crash
            // between these sync points blocks admission instead of treating
            // a lost state file as an unused nonce namespace.
            let mut writer = lock.try_clone()?;
            writer.seek(SeekFrom::Start(0))?;
            writer.write_all(INITIALIZED_MARKER)?;
            writer.sync_all()?;
        }
        let mut suffix = String::with_capacity(32);
        for byte in entropy {
            use std::fmt::Write as _;
            write!(&mut suffix, "{byte:02x}").expect("String write");
        }
        let temp = self.dir.join(format!(".gateway-ucan-replay-{suffix}.tmp"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp)?;
        let result = (|| {
            require_private_file(&file)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temp, self.dir.join(STATE_NAME))?;
            File::open(&self.dir)?.sync_all()
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
}

impl ReplayLedgerPort for PrivateReplayLedger {
    type Error = io::Error;

    fn load(&self, slot: ReplaySlotIdentity) -> io::Result<SlotObservation> {
        let _lock = self.lock(false)?;
        let state = self.load_locked(&_lock)?;
        Ok(state
            .records
            .get(&hex(slot.as_bytes()))
            .copied()
            .map_or(SlotObservation::Empty, SlotObservation::Present))
    }

    fn compare_and_swap(&self, command: CompareAndSwapCommand) -> io::Result<CommitObservation> {
        let _lock = self.lock(true)?;
        let mut state = self.load_locked(&_lock)?;
        let key = hex(command.slot().as_bytes());
        let previous = state.records.get(&key).copied();
        let observed_version = previous.map(|record| record.version());
        if observed_version != command.expected_version() {
            return Ok(CommitObservation::Stale { observed_version });
        }
        let replacement = command.replacement();
        let expected_version = match observed_version {
            Some(version) => {
                if previous.is_some_and(|record| record.identity() != replacement.identity()) {
                    return Err(invalid_state());
                }
                version.checked_next().ok_or_else(invalid_state)?
            }
            None => RecordVersion::initial(),
        };
        if replacement.slot() != command.slot() || replacement.version() != expected_version {
            return Err(invalid_state());
        }
        if previous.is_none() && state.records.len() >= MAX_RECORDS {
            return Err(invalid_state());
        }
        state.records.insert(key, replacement);
        self.save_locked(&_lock, &state)?;
        Ok(CommitObservation::Applied {
            version: replacement.version(),
            durability: DurabilityObservation::Synchronized,
        })
    }
}

fn lock_initialized(lock: &File) -> io::Result<bool> {
    match lock.metadata()?.len() {
        0 => Ok(false),
        8 => {
            let mut marker = [0_u8; 8];
            let mut reader = lock.try_clone()?;
            reader.seek(SeekFrom::Start(0))?;
            reader.read_exact(&mut marker)?;
            if marker == *INITIALIZED_MARKER {
                Ok(true)
            } else {
                Err(invalid_state())
            }
        }
        _ => Err(invalid_state()),
    }
}

fn invalid_state() -> io::Error {
    io::Error::new(ErrorKind::InvalidData, "gateway-replay-state-invalid")
}

fn require_private_dir(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.file_type().is_dir()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.permissions().mode() & 0o077 != 0
    {
        return Err(invalid_state());
    }
    Ok(())
}

fn require_private_file(file: &File) -> io::Result<()> {
    let meta = file.metadata()?;
    if !meta.is_file()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.permissions().mode() & 0o077 != 0
        || meta.nlink() != 1
    {
        return Err(invalid_state());
    }
    Ok(())
}

fn validate_state(state: &LedgerState) -> io::Result<()> {
    if state.version != STATE_VERSION
        || state.records.len() > MAX_RECORDS
        || state.records.iter().any(|(key, record)| key != &hex(record.slot().as_bytes()))
    {
        return Err(invalid_state());
    }
    Ok(())
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut result, "{byte:02x}").expect("String write");
    }
    result
}

#[cfg(test)]
mod tests {
    use replay_ledger_core::ActorDigest;
    use replay_ledger_core::AttemptIdentity;
    use replay_ledger_core::AuthorityGeneration;
    use replay_ledger_core::ConsumptionDecision;
    use replay_ledger_core::LedgerNamespaceDigest;
    use replay_ledger_core::NonceDigest;
    use replay_ledger_core::OperationProfileDigest;
    use replay_ledger_core::ReplayScope;
    use replay_ledger_core::ReplayScopeInput;
    use replay_ledger_core::RequestDigest;
    use replay_ledger_core::TenantDigest;
    use replay_ledger_core::plan_consumption;

    use super::*;

    fn scope(request: u8) -> ReplayScope {
        ReplayScope::new(ReplayScopeInput {
            tenant: TenantDigest::new([1; 32]).unwrap(),
            actor: ActorDigest::new([2; 32]).unwrap(),
            operation_profile: OperationProfileDigest::new([3; 32]).unwrap(),
            request: RequestDigest::new([request; 32]).unwrap(),
            nonce: NonceDigest::new([4; 32]).unwrap(),
            generation: AuthorityGeneration::new(1).unwrap(),
            namespace: LedgerNamespaceDigest::new([5; 32]).unwrap(),
        })
    }

    #[test]
    fn synchronized_replay_survives_reopen_and_conflicting_request_cannot_reuse_nonce() {
        let temp = tempfile::tempdir().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let ledger = PrivateReplayLedger::new(temp.path()).unwrap();
        let first = scope(6);
        let attempt = AttemptIdentity::new([7; 32]).unwrap();
        let command = match plan_consumption(first, attempt, ledger.load(first.slot_identity()).unwrap()) {
            ConsumptionDecision::First { command } => command,
            other => panic!("expected first use: {other:?}"),
        };
        assert_eq!(ledger.compare_and_swap(command).unwrap(), CommitObservation::Applied {
            version: RecordVersion::initial(),
            durability: DurabilityObservation::Synchronized,
        });
        let restarted = PrivateReplayLedger::new(temp.path()).unwrap();
        assert!(matches!(
            plan_consumption(first, attempt, restarted.load(first.slot_identity()).unwrap()),
            ConsumptionDecision::Reserved { .. }
        ));
        let changed_request = scope(8);
        assert_eq!(changed_request.slot_identity(), first.slot_identity());
        assert_eq!(
            plan_consumption(changed_request, attempt, restarted.load(first.slot_identity()).unwrap()),
            ConsumptionDecision::Conflict
        );
        assert_eq!(restarted.compare_and_swap(command).unwrap(), CommitObservation::Stale {
            observed_version: Some(RecordVersion::initial())
        });
        fs::remove_file(temp.path().join(STATE_NAME)).unwrap();
        assert!(restarted.load(first.slot_identity()).is_err());
        assert!(restarted.compare_and_swap(command).is_err());
    }

    #[test]
    fn corrupt_or_nonprivate_ledger_fails_closed_without_erasing_committed_nonce() {
        let temp = tempfile::tempdir().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let ledger = PrivateReplayLedger::new(temp.path()).unwrap();
        let first = scope(6);
        let attempt = AttemptIdentity::new([7; 32]).unwrap();
        let command = match plan_consumption(first, attempt, ledger.load(first.slot_identity()).unwrap()) {
            ConsumptionDecision::First { command } => command,
            _ => panic!("expected first use"),
        };
        ledger.compare_and_swap(command).unwrap();
        let path = temp.path().join(STATE_NAME);
        let mut content = fs::read(&path).unwrap();
        content.extend_from_slice(b"{}");
        fs::write(&path, content).unwrap();
        assert!(ledger.load(first.slot_identity()).is_err());
        assert!(ledger.compare_and_swap(command).is_err());
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(PrivateReplayLedger::new(temp.path()).is_err());
    }
    #[test]
    fn competing_replay_reservations_commit_only_once_across_independent_connections() {
        use std::sync::Arc;
        use std::sync::Barrier;
        let temp = tempfile::tempdir().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let ledger = Arc::new(PrivateReplayLedger::new(temp.path()).unwrap());
        let scope = scope(6);
        let attempt = AttemptIdentity::new([7; 32]).unwrap();
        let command = match plan_consumption(scope, attempt, ledger.load(scope.slot_identity()).unwrap()) {
            ConsumptionDecision::First { command } => command,
            _ => panic!("expected first use"),
        };
        let gate = Arc::new(Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let ledger = Arc::clone(&ledger);
                let gate = Arc::clone(&gate);
                std::thread::spawn(move || {
                    gate.wait();
                    ledger.compare_and_swap(command).unwrap()
                })
            })
            .collect();
        let outcomes: Vec<_> = handles.into_iter().map(|handle| handle.join().unwrap()).collect();
        assert_eq!(outcomes.iter().filter(|outcome| matches!(outcome, CommitObservation::Applied { .. })).count(), 1);
        assert_eq!(outcomes.iter().filter(|outcome| matches!(outcome, CommitObservation::Stale { .. })).count(), 7);
        assert!(matches!(ledger.load(scope.slot_identity()).unwrap(), SlotObservation::Present(_)));
    }
    #[test]
    fn replay_lock_contention_is_bounded_before_any_nonce_commit() {
        let temp = tempfile::tempdir().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let ledger = PrivateReplayLedger::new(temp.path()).unwrap();
        let held = ledger.lock(true).unwrap();
        let denied = ledger.lock_with_wait(true, Duration::ZERO).err().unwrap();
        assert_eq!(denied.kind(), ErrorKind::TimedOut);
        drop(held);
        assert_eq!(ledger.load(scope(6).slot_identity()).unwrap(), SlotObservation::Empty);
    }
}
