use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use fs2::FileExt as _;
use serde::Deserialize;
use serde::Serialize;
use zeroize::Zeroizing;

use super::Authority;
use super::Completion;
use super::Error;
use super::Event;
use super::MAX_ATTEMPTS;
use super::MAX_EVENTS_PER_ATTEMPT;
use super::MAX_JOURNAL_BYTES;
use super::Observation;
use super::Phase;
use super::Status;
use super::crypto;

const NAME: &str = "gateway-api-v1.journal";
const LOCK: &str = "gateway-api-v1.lock";
const MARKER: &[u8; 8] = b"MNTLAPI1";
const LOCK_WAIT: Duration = Duration::from_secs(5);
const LOCK_POLL: Duration = Duration::from_millis(10);

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    pub id: String,
    pub subject: String,
    pub scope: String,
    pub digest: [u8; 32],
    pub policy: [u8; 32],
    pub admitted: bool,
    pub cancel_requested: bool,
    pub observation: Observation,
    pub events: Vec<Event>,
}

impl Record {
    pub fn pending(id: String, authority: Authority<'_>, digest: [u8; 32], policy: [u8; 32]) -> Self {
        Self {
            id,
            subject: authority.subject.into(),
            scope: authority.scope.into(),
            digest,
            policy,
            admitted: false,
            cancel_requested: false,
            observation: Observation {
                phase: Phase::Queued,
                fence: 0,
                result_evidence: None,
            },
            events: Vec::new(),
        }
    }

    pub fn observe(&mut self, observation: Observation, signer: &SigningKey) -> Result<(), Error> {
        if observation.fence == 0
            || observation.fence < self.observation.fence
            || (!observation.phase.terminal() && observation.result_evidence.is_some())
        {
            return Err(Error::Integrity);
        }
        if self.admitted && self.observation == observation {
            return Ok(());
        }
        if self.admitted && self.observation.phase.terminal() {
            return Err(Error::Conflict);
        }
        if self.admitted && self.observation.phase == Phase::Running && observation.phase == Phase::Queued {
            return Err(Error::Conflict);
        }
        // Reserve one event slot for completion; unbounded fence churn cannot
        // make a terminal event impossible to retain.
        let limit = if observation.phase.terminal() {
            MAX_EVENTS_PER_ATTEMPT
        } else {
            MAX_EVENTS_PER_ATTEMPT - 1
        };
        if self.events.len() >= limit {
            return Err(super::core::Reject::Bound.into());
        }
        self.observation = observation;
        self.admitted = true;
        let sequence = self.events.len() as u64 + 1;
        let completion = if self.observation.phase.terminal() {
            Some(Completion::create(self, sequence, signer)?)
        } else {
            None
        };
        self.events.push(Event {
            sequence,
            phase: self.observation.phase,
            completion,
        });
        Ok(())
    }

    pub fn status(&self) -> Status {
        Status {
            attempt_id: self.id.clone(),
            request_digest: self.digest,
            observation: self.observation.clone(),
            cancel_requested: self.cancel_requested,
            sequence: self.events.len() as u64,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct State {
    version: u32,
    producer: [u8; 32],
    pub records: BTreeMap<String, Record>,
}

impl State {
    pub fn by_id_mut(&mut self, id: &str) -> Result<&mut Record, Error> {
        self.records.values_mut().find(|record| record.id == id).ok_or(Error::Unknown)
    }

    fn validate(&self, verifier: &VerifyingKey) -> Result<(), Error> {
        if self.version != 1 || self.producer != verifier.to_bytes() || self.records.len() > MAX_ATTEMPTS {
            return Err(Error::Integrity);
        }
        let mut ids = std::collections::BTreeSet::new();
        for (slot, record) in &self.records {
            if slot.len() != 64
                || !super::identity(&record.id)
                || !super::identity(&record.subject)
                || !super::identity(&record.scope)
                || !ids.insert(&record.id)
                || record.events.len() > MAX_EVENTS_PER_ATTEMPT
                || record.admitted == record.events.is_empty()
                || (record.admitted && record.observation.fence == 0)
            {
                return Err(Error::Integrity);
            }
            for (index, event) in record.events.iter().enumerate() {
                if event.sequence != index as u64 + 1
                    || event.phase.terminal() != event.completion.is_some()
                    || (event.phase.terminal() && index + 1 != record.events.len())
                {
                    return Err(Error::Integrity);
                }
                if let Some(completion) = &event.completion {
                    super::verify_completion(completion, verifier)?;
                    if completion.attempt_id != record.id
                        || completion.request_digest != record.digest
                        || completion.policy_version != record.policy
                        || completion.sequence != event.sequence
                        || completion.terminal != event.phase
                        || completion.result_evidence != record.observation.result_evidence
                    {
                        return Err(Error::Integrity);
                    }
                }
            }
            if let Some(last) = record.events.last()
                && last.phase != record.observation.phase
            {
                return Err(Error::Integrity);
            }
        }
        Ok(())
    }
}

pub(super) struct Journal {
    directory: PathBuf,
    secret: Zeroizing<[u8; 32]>,
    verifier: VerifyingKey,
}

impl Journal {
    pub fn open(directory: &Path, secret: [u8; 32], verifier: VerifyingKey) -> Result<Self, Error> {
        private_directory(directory)?;
        let journal = Self {
            directory: directory.into(),
            secret: Zeroizing::new(secret),
            verifier,
        };
        journal.update(|_| Ok(()))?;
        Ok(journal)
    }

    pub fn submission_slot(&self, subject: &str, scope: &str, key: &str) -> String {
        super::hex(&crypto::slot(&self.secret, &["gateway-submit-v1", subject, scope, key]))
    }

    pub fn cursor_read(
        &self,
        cursor: Option<&str>,
        authority: Authority<'_>,
        id: &str,
        kind: u8,
        now: u64,
    ) -> Result<u64, Error> {
        crypto::cursor_read(&self.secret, cursor, authority, id, kind, now)
    }

    pub fn cursor_write(
        &self,
        authority: Authority<'_>,
        id: &str,
        kind: u8,
        offset: u64,
        now: u64,
    ) -> Result<String, Error> {
        crypto::cursor_write(&self.secret, authority, id, kind, offset, now)
    }

    pub fn find(&self, id: &str) -> Result<Record, Error> {
        let lock = self.lock()?;
        let state = self.load(&lock)?;
        state.records.values().find(|record| record.id == id).cloned().ok_or(Error::Unknown)
    }

    pub fn update<T>(&self, change: impl FnOnce(&mut State) -> Result<T, Error>) -> Result<T, Error> {
        let lock = self.lock()?;
        let mut state = self.load(&lock)?;
        let result = change(&mut state)?;
        state.validate(&self.verifier)?;
        self.save(&lock, &state)?;
        Ok(result)
    }

    fn lock(&self) -> Result<File, Error> {
        private_directory(&self.directory)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.directory.join(LOCK))
            .map_err(|_| Error::Storage)?;
        private_file(&lock)?;
        let started = Instant::now();
        loop {
            match lock.try_lock_exclusive() {
                Ok(()) => return Ok(lock),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && started.elapsed() < LOCK_WAIT => {
                    std::thread::sleep(LOCK_POLL)
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Err(Error::Busy),
                Err(_) => return Err(Error::Storage),
            }
        }
    }

    fn load(&self, lock: &File) -> Result<State, Error> {
        let initialized = initialized(lock)?;
        let file = match OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(self.directory.join(NAME)) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !initialized => {
                return Ok(State {
                    version: 1,
                    producer: self.verifier.to_bytes(),
                    records: BTreeMap::new(),
                });
            }
            Err(_) => return Err(Error::Storage),
        };
        private_file(&file)?;
        if file.metadata().map_err(|_| Error::Storage)?.len() > MAX_JOURNAL_BYTES {
            return Err(Error::Integrity);
        }
        let mut bytes = Vec::new();
        file.take(MAX_JOURNAL_BYTES + 1).read_to_end(&mut bytes).map_err(|_| Error::Storage)?;
        if bytes.len() < 32 || bytes.len() as u64 > MAX_JOURNAL_BYTES {
            return Err(Error::Integrity);
        }
        let tag: [u8; 32] = bytes[..32].try_into().map_err(|_| Error::Integrity)?;
        if blake3::keyed_hash(&self.secret, &bytes[32..]) != blake3::Hash::from(tag) {
            return Err(Error::Integrity);
        }
        let state: State = serde_json::from_slice(&bytes[32..]).map_err(|_| Error::Integrity)?;
        state.validate(&self.verifier)?;
        if serde_json::to_vec(&state).map_err(|_| Error::Integrity)? != bytes[32..] {
            return Err(Error::Integrity);
        }
        Ok(state)
    }

    fn save(&self, lock: &File, state: &State) -> Result<(), Error> {
        let bytes = serde_json::to_vec(state).map_err(|_| Error::Storage)?;
        if bytes.len() as u64 + 32 > MAX_JOURNAL_BYTES {
            return Err(super::core::Reject::Bound.into());
        }
        if !initialized(lock)? {
            let mut writer = lock.try_clone().map_err(|_| Error::Storage)?;
            writer.seek(SeekFrom::Start(0)).map_err(|_| Error::Storage)?;
            writer.write_all(MARKER).map_err(|_| Error::Storage)?;
            writer.sync_all().map_err(|_| Error::Storage)?;
        }
        let mut random = [0_u8; 32];
        getrandom::fill(&mut random).map_err(|_| Error::Storage)?;
        let temporary = self.directory.join(format!(".gateway-api-{}.tmp", super::hex(&random)));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&temporary)
                .map_err(|_| Error::Storage)?;
            private_file(&file)?;
            file.write_all(blake3::keyed_hash(&self.secret, &bytes).as_bytes()).map_err(|_| Error::Storage)?;
            file.write_all(&bytes).map_err(|_| Error::Storage)?;
            file.sync_all().map_err(|_| Error::Storage)?;
            fs::rename(&temporary, self.directory.join(NAME)).map_err(|_| Error::Storage)?;
            File::open(&self.directory).and_then(|file| file.sync_all()).map_err(|_| Error::Storage)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
}

fn initialized(lock: &File) -> Result<bool, Error> {
    match lock.metadata().map_err(|_| Error::Storage)?.len() {
        0 => Ok(false),
        8 => {
            let mut marker = [0_u8; 8];
            let mut reader = lock.try_clone().map_err(|_| Error::Storage)?;
            reader.seek(SeekFrom::Start(0)).map_err(|_| Error::Storage)?;
            reader.read_exact(&mut marker).map_err(|_| Error::Storage)?;
            if &marker != MARKER {
                return Err(Error::Integrity);
            }
            Ok(true)
        }
        _ => Err(Error::Integrity),
    }
}

fn private_directory(path: &Path) -> Result<(), Error> {
    let meta = fs::symlink_metadata(path).map_err(|_| Error::Storage)?;
    if !meta.is_dir() || meta.uid() != rustix::process::geteuid().as_raw() || meta.permissions().mode() & 0o077 != 0 {
        return Err(Error::Storage);
    }
    Ok(())
}

fn private_file(file: &File) -> Result<(), Error> {
    let meta = file.metadata().map_err(|_| Error::Storage)?;
    if !meta.is_file()
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.permissions().mode() & 0o077 != 0
        || meta.nlink() != 1
    {
        return Err(Error::Storage);
    }
    Ok(())
}
