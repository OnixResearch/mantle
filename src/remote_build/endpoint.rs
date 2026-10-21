//! Durable endpoint bindings under the existing private remote state directory.
//!
//! Reads and admission hold shared binding leases; create/revoke hold the
//! independent binding lock exclusively and reload the table. No cached snapshot
//! may resurrect a revoked generation. A descriptor is not authentication or
//! payload admission: callers retain all ticket, transfer, signed PathInfo,
//! content and layer checks. Ticket mutation locks remain independent and short.

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

use crunch_remote_core::endpoint::EndpointBinding;
use crunch_remote_core::endpoint::EndpointBindingError;
use crunch_remote_core::endpoint::EndpointBindingRecord;
use crunch_remote_core::endpoint::EndpointBindingSpec;
use crunch_remote_core::endpoint::EndpointBindingTable;
use crunch_remote_core::endpoint::EndpointReceiverRequirements;
use crunch_remote_core::endpoint::MAX_ENDPOINT_BINDINGS;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::remote_credential_state::REMOTE_TICKET_STATE_DIRECTORY;

const ENDPOINT_BINDING_LOCK_FILENAME: &str = "endpoint-bindings.lock";

pub const ENDPOINT_BINDING_STATE_FILENAME: &str = "endpoint-bindings.json";
pub const MAX_ENDPOINT_BINDING_STATE_BYTES: u64 = 8_388_608;
const ENDPOINT_BINDING_SCHEMA_VERSION: u32 = 1;
const ENDPOINT_BINDING_LOCK_WAIT_TIMEOUT_MS: u64 = 5_000;
const ENDPOINT_BINDING_LOCK_POLL_INTERVAL_MS: u64 = 10;

#[derive(Debug)]
pub struct EndpointBindingStore {
    state_dir: PathBuf,
}

/// Receiver-local name admission held stable through an operation's existing
/// authentication/quota checks and effects. This guard is not authentication.
/// Its shared binding lease allows concurrent admission and independent ticket
/// commits, but blocks binding creation/revocation until the operation finishes.
#[derive(Debug)]
pub struct EndpointAdmissionGuard {
    binding: EndpointBinding,
    _binding_lease: EndpointBindingLease,
}

impl EndpointAdmissionGuard {
    pub fn binding(&self) -> &EndpointBinding {
        &self.binding
    }
}

#[derive(Debug)]
struct EndpointBindingLease {
    file: File,
}

impl Drop for EndpointBindingLease {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

#[derive(Clone, Copy)]
enum BindingLockMode {
    Shared,
    Exclusive,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBindings {
    schema_version: u32,
    #[serde(deserialize_with = "deserialize_bounded_records")]
    records: Vec<EndpointBindingRecord>,
}

#[derive(Serialize)]
struct RenderedBindings<'a> {
    schema_version: u32,
    records: &'a [EndpointBindingRecord],
}

impl EndpointBindingStore {
    pub fn open(state_dir: &Path) -> Result<Self, RunError> {
        let _lease = acquire_binding_lease(state_dir, BindingLockMode::Shared)?;
        load_bindings(state_dir)?;
        Ok(Self {
            state_dir: state_dir.to_path_buf(),
        })
    }

    pub fn path(&self) -> PathBuf {
        endpoint_binding_state_path(&self.state_dir)
    }

    // r[impl remote_builds.single_binding_resolution]
    // r[impl remote_builds.binding_revocation]
    pub fn create(
        &self,
        spec: EndpointBindingSpec,
        expected_previous_generation: Option<u64>,
        now_unix_s: u64,
    ) -> Result<EndpointBinding, RunError> {
        let _lease = acquire_binding_lease(&self.state_dir, BindingLockMode::Exclusive)?;
        let mut table = load_bindings(&self.state_dir)?;
        let binding = table.create(spec, expected_previous_generation, now_unix_s).map_err(decision_error)?.clone();
        save_bindings(&self.state_dir, &table)?;
        Ok(binding)
    }

    // r[impl remote_builds.binding_revocation]
    pub fn revoke(&self, exact: &EndpointBinding) -> Result<(), RunError> {
        let _lease = acquire_binding_lease(&self.state_dir, BindingLockMode::Exclusive)?;
        let mut table = load_bindings(&self.state_dir)?;
        table.revoke(exact).map_err(decision_error)?;
        save_bindings(&self.state_dir, &table)
    }

    /// The owned descriptor crosses the short-lived durable-table read. Its
    /// existence does not substitute for receiver-local verification or auth.
    // r[impl remote_builds.single_binding_resolution]
    pub fn resolve(&self, presented: &EndpointBinding, now_unix_s: u64) -> Result<EndpointBinding, RunError> {
        let _lease = acquire_binding_lease(&self.state_dir, BindingLockMode::Shared)?;
        let table = load_bindings(&self.state_dir)?;
        Ok(table.resolve(presented, now_unix_s).map_err(decision_error)?.binding().clone())
    }

    // r[impl remote_builds.single_binding_resolution]
    pub fn verify_received(
        &self,
        presented: &EndpointBinding,
        expected: EndpointReceiverRequirements<'_>,
        now_unix_s: u64,
    ) -> Result<EndpointBinding, RunError> {
        let _lease = acquire_binding_lease(&self.state_dir, BindingLockMode::Shared)?;
        let table = load_bindings(&self.state_dir)?;
        Ok(table.verify_received(presented, expected, now_unix_s).map_err(decision_error)?.binding().clone())
    }

    /// Admit the receiver's exact typed purpose and requirements under a shared
    /// binding lease retained through its operation. No purpose is inferred from
    /// a name, and no endpoint kind has a fallback.
    // r[impl remote_builds.single_binding_resolution]
    pub fn admit_received(
        &self,
        presented: &EndpointBinding,
        expected: EndpointReceiverRequirements<'_>,
        now_unix_s: u64,
    ) -> Result<EndpointAdmissionGuard, RunError> {
        let binding_lease = acquire_binding_lease(&self.state_dir, BindingLockMode::Shared)?;
        let table = load_bindings(&self.state_dir)?;
        let binding = table.verify_received(presented, expected, now_unix_s).map_err(decision_error)?.binding().clone();
        Ok(EndpointAdmissionGuard {
            binding,
            _binding_lease: binding_lease,
        })
    }

    /// The callback must retain its existing operation-specific authentication
    /// and payload checks. Rejection never calls it; successful name admission
    /// retains the shared binding lease until the callback returns, including errors.
    pub fn with_admitted_received<R>(
        &self,
        presented: &EndpointBinding,
        expected: EndpointReceiverRequirements<'_>,
        now_unix_s: u64,
        operation: impl FnOnce(&EndpointAdmissionGuard) -> Result<R, RunError>,
    ) -> Result<R, RunError> {
        let guard = self.admit_received(presented, expected, now_unix_s)?;
        operation(&guard)
    }

    pub fn records(&self) -> Result<Vec<EndpointBindingRecord>, RunError> {
        let _lease = acquire_binding_lease(&self.state_dir, BindingLockMode::Shared)?;
        let table = load_bindings(&self.state_dir)?;
        Ok(table.into_records())
    }
}

pub fn endpoint_binding_state_path(state_dir: &Path) -> PathBuf {
    state_dir.join(REMOTE_TICKET_STATE_DIRECTORY).join(ENDPOINT_BINDING_STATE_FILENAME)
}

fn acquire_binding_lease(state_dir: &Path, mode: BindingLockMode) -> Result<EndpointBindingLease, RunError> {
    acquire_binding_lease_with_wait(
        state_dir,
        mode,
        Duration::from_millis(ENDPOINT_BINDING_LOCK_WAIT_TIMEOUT_MS),
        Duration::from_millis(ENDPOINT_BINDING_LOCK_POLL_INTERVAL_MS),
    )
}

fn acquire_binding_lease_with_wait(
    state_dir: &Path,
    mode: BindingLockMode,
    wait_timeout: Duration,
    poll_interval: Duration,
) -> Result<EndpointBindingLease, RunError> {
    let parent = state_dir.join(REMOTE_TICKET_STATE_DIRECTORY);
    ensure_private_binding_directory(&parent)?;
    let file =
        open_binding_lock(&parent.join(ENDPOINT_BINDING_LOCK_FILENAME)).map_err(|_| state_error("lock-open-failed"))?;
    validate_private_file(&file.metadata().map_err(|_| state_error("lock-metadata-failed"))?)?;
    let started = Instant::now();
    loop {
        let result = match mode {
            BindingLockMode::Shared => fs2::FileExt::try_lock_shared(&file),
            BindingLockMode::Exclusive => fs2::FileExt::try_lock_exclusive(&file),
        };
        match result {
            Ok(()) => return Ok(EndpointBindingLease { file }),
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

#[cfg(unix)]
fn ensure_private_binding_directory(parent: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::DirBuilderExt as _;
    use std::os::unix::fs::MetadataExt as _;

    let metadata = match fs::symlink_metadata(parent) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let ancestor = parent.parent().ok_or_else(|| state_error("parent-ancestor-missing"))?;
            fs::create_dir_all(ancestor).map_err(|_| state_error("parent-ancestor-create-failed"))?;
            match fs::DirBuilder::new().mode(0o700).create(parent) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(state_error("parent-create-failed")),
            }
            fs::symlink_metadata(parent).map_err(|_| state_error("parent-inspection-failed"))?
        }
        Err(_) => return Err(state_error("parent-inspection-failed")),
    };
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
        || metadata.mode() & 0o700 != 0o700
    {
        return Err(state_error("parent-unsafe"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_private_binding_directory(_parent: &Path) -> Result<(), RunError> {
    Err(state_error("private-state-requires-unix"))
}

#[cfg(unix)]
fn open_binding_lock(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
}

#[cfg(not(unix))]
fn open_binding_lock(_path: &Path) -> std::io::Result<File> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "endpoint-private-state-requires-unix"))
}

fn load_bindings(state_dir: &Path) -> Result<EndpointBindingTable, RunError> {
    let path = endpoint_binding_state_path(state_dir);
    let mut file = match open_no_follow(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(EndpointBindingTable::default()),
        Err(_) => return Err(state_error("open-failed")),
    };
    let metadata = file.metadata().map_err(|_| state_error("metadata-failed"))?;
    validate_private_file(&metadata)?;
    if metadata.len() > MAX_ENDPOINT_BINDING_STATE_BYTES {
        return Err(state_error("size-limit-exceeded"));
    }
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(MAX_ENDPOINT_BINDING_STATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| state_error("read-failed"))?;
    if bytes.len() as u64 > MAX_ENDPOINT_BINDING_STATE_BYTES {
        return Err(state_error("size-limit-exceeded"));
    }
    let stored: StoredBindings = serde_json::from_slice(&bytes).map_err(|_| state_error("malformed"))?;
    if stored.schema_version != ENDPOINT_BINDING_SCHEMA_VERSION {
        return Err(state_error("schema-version-unsupported"));
    }
    EndpointBindingTable::from_records(stored.records).map_err(decision_error)
}

fn save_bindings(state_dir: &Path, table: &EndpointBindingTable) -> Result<(), RunError> {
    let path = endpoint_binding_state_path(state_dir);
    let parent = path.parent().ok_or_else(|| state_error("parent-missing"))?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) => validate_private_file(&metadata)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(state_error("target-inspection-failed")),
    }
    let rendered = RenderedBindings {
        schema_version: ENDPOINT_BINDING_SCHEMA_VERSION,
        records: table.records(),
    };
    let mut bytes = serde_json::to_vec(&rendered).map_err(|_| state_error("serialization-failed"))?;
    bytes.push(b'\n');
    if bytes.len() as u64 > MAX_ENDPOINT_BINDING_STATE_BYTES {
        return Err(state_error("size-limit-exceeded"));
    }
    // NamedTempFile uses exclusive creation and owner-only permissions, matching
    // the ticket state's private temporary/rename/file+directory fsync protocol.
    let mut temporary = tempfile::Builder::new()
        .prefix(".endpoint-bindings.json.tmp-")
        .tempfile_in(parent)
        .map_err(|_| state_error("temp-open-failed"))?;
    validate_private_file(&temporary.as_file().metadata().map_err(|_| state_error("temp-metadata-failed"))?)?;
    temporary.write_all(&bytes).map_err(|_| state_error("temp-write-failed"))?;
    temporary.as_file().sync_all().map_err(|_| state_error("temp-sync-failed"))?;
    temporary.persist(&path).map_err(|_| state_error("atomic-replace-failed"))?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| state_error("parent-sync-failed"))
}

#[cfg(unix)]
fn open_no_follow(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
}

#[cfg(not(unix))]
fn open_no_follow(_path: &Path) -> std::io::Result<File> {
    // Do not silently downgrade private state-path enforcement.
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "endpoint-private-state-requires-unix"))
}

#[cfg(unix)]
fn validate_private_file(metadata: &fs::Metadata) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt as _;
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
        || metadata.mode() & 0o600 != 0o600
    {
        return Err(state_error("target-unsafe"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_file(_metadata: &fs::Metadata) -> Result<(), RunError> {
    Err(state_error("private-state-requires-unix"))
}

fn deserialize_bounded_records<'de, D>(deserializer: D) -> Result<Vec<EndpointBindingRecord>, D::Error>
where D: serde::Deserializer<'de> {
    struct BoundedRecords;
    impl<'de> serde::de::Visitor<'de> for BoundedRecords {
        type Value = Vec<EndpointBindingRecord>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded endpoint binding table")
        }
        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de> {
            let mut records = Vec::new();
            while let Some(record) = sequence.next_element()? {
                if records.len() >= MAX_ENDPOINT_BINDINGS {
                    return Err(serde::de::Error::custom("endpoint-binding-table-full"));
                }
                records.push(record);
            }
            Ok(records)
        }
    }
    deserializer.deserialize_seq(BoundedRecords)
}

fn decision_error(error: EndpointBindingError) -> RunError {
    RunError::Internal(format!("remote-endpoint-binding-{error:?}"))
}

fn state_error(category: &str) -> RunError {
    RunError::Internal(format!("remote-endpoint-state-{category}"))
}

#[cfg(all(test, unix))]
mod tests {
    use crunch_remote_core::endpoint::EndpointKind;

    use super::*;

    fn spec(name: &str) -> EndpointBindingSpec {
        EndpointBindingSpec {
            kind: EndpointKind::BuilderTicket,
            name: name.into(),
            scope: "farm-a".into(),
            trust_requirement: "existing-ticket-policy".into(),
            store_prefix: "/mantle/store".into(),
            expires_unix_s: 100,
        }
    }

    #[test]
    fn durable_revoke_recreate_and_receiver_checks_survive_reopen_and_stale_store_objects() {
        let temporary = tempfile::tempdir().unwrap();
        let first = EndpointBindingStore::open(temporary.path()).unwrap();
        let stale_reader = EndpointBindingStore::open(temporary.path()).unwrap();
        let old = first.create(spec("builder-a"), None, 1).unwrap();
        let other = first.create(spec("builder-b"), None, 1).unwrap();
        assert_eq!(stale_reader.resolve(&old, 2).unwrap(), old);
        first.revoke(&old).unwrap();
        assert!(stale_reader.resolve(&old, 2).is_err());
        let reopened = EndpointBindingStore::open(temporary.path()).unwrap();
        assert!(reopened.records().unwrap().iter().any(|record| record.binding == old && record.revoked));
        assert_eq!(reopened.resolve(&other, 2).unwrap(), other);
        let new = reopened.create(old.spec.clone(), Some(old.generation), 2).unwrap();
        assert_eq!(new.generation, old.generation + 1);
        assert!(stale_reader.resolve(&old, 2).is_err());
        assert!(stale_reader.revoke(&old).is_err());
        let requirements = EndpointReceiverRequirements {
            kind: EndpointKind::BuilderTicket,
            scope: "farm-a",
            trust_requirement: "existing-ticket-policy",
            store_prefix: "/mantle/store",
        };
        assert_eq!(stale_reader.verify_received(&new, requirements, 2).unwrap(), new);
        assert!(
            stale_reader
                .verify_received(
                    &new,
                    EndpointReceiverRequirements {
                        store_prefix: "/nix/store",
                        ..requirements
                    },
                    2
                )
                .is_err()
        );
        assert!(stale_reader.resolve(&new, 100).is_err());
    }

    #[test]
    fn durable_state_rejects_symlinks_unsafe_permissions_and_malformed_records() {
        use std::os::unix::fs::PermissionsExt as _;
        use std::os::unix::fs::symlink;
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        fs::set_permissions(store.path(), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(store.resolve(&binding, 2).is_err());
        fs::set_permissions(store.path(), fs::Permissions::from_mode(0o600)).unwrap();
        let mut forged = binding.clone();
        forged.generation = 0;
        let bytes = serde_json::to_vec(&RenderedBindings {
            schema_version: 1,
            records: &[EndpointBindingRecord {
                binding: forged,
                revoked: false,
            }],
        })
        .unwrap();
        fs::write(store.path(), bytes).unwrap();
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
        fs::remove_file(store.path()).unwrap();
        let outside = temporary.path().join("outside");
        fs::write(&outside, b"{}").unwrap();
        symlink(&outside, store.path()).unwrap();
        assert!(store.resolve(&binding, 2).is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"{}");
    }

    #[test]
    fn durable_state_enforces_file_and_table_bounds() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        let file = OpenOptions::new().write(true).open(store.path()).unwrap();
        file.set_len(MAX_ENDPOINT_BINDING_STATE_BYTES + 1).unwrap();
        assert!(store.resolve(&binding, 2).is_err());
        let record = EndpointBindingRecord { binding, revoked: true };
        let records = vec![record; MAX_ENDPOINT_BINDINGS + 1];
        let bytes = serde_json::to_vec(&RenderedBindings {
            schema_version: 1,
            records: &records,
        })
        .unwrap();
        assert!(bytes.len() as u64 <= MAX_ENDPOINT_BINDING_STATE_BYTES);
        fs::write(store.path(), bytes).unwrap();
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
    }

    #[test]
    fn operation_gate_holds_shared_binding_lease_and_rejects_before_payload_effects_for_every_kind() {
        for kind in [
            EndpointKind::BuilderTicket,
            EndpointKind::Transfer,
            EndpointKind::Substituter,
            EndpointKind::BaseLayer,
        ] {
            let temporary = tempfile::tempdir().unwrap();
            let store = EndpointBindingStore::open(temporary.path()).unwrap();
            let binding = store
                .create(
                    EndpointBindingSpec {
                        kind,
                        ..spec("entry-a")
                    },
                    None,
                    1,
                )
                .unwrap();
            let expected = EndpointReceiverRequirements {
                kind,
                scope: "farm-a",
                trust_requirement: "existing-ticket-policy",
                store_prefix: "/mantle/store",
            };
            let payload = temporary.path().join("admitted-content");
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .open(temporary.path().join(REMOTE_TICKET_STATE_DIRECTORY).join(ENDPOINT_BINDING_LOCK_FILENAME))
                .unwrap();
            store
                .with_admitted_received(&binding, expected, 2, |guard| {
                    assert_eq!(guard.binding(), &binding);
                    let error = fs2::FileExt::try_lock_exclusive(&lock).unwrap_err();
                    assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
                    fs::write(&payload, b"already-admitted-content").unwrap();
                    Ok(())
                })
                .unwrap();
            fs2::FileExt::try_lock_exclusive(&lock).unwrap();
            fs2::FileExt::unlock(&lock).unwrap();
            store.revoke(&binding).unwrap();
            let mut called = false;
            assert!(
                store
                    .with_admitted_received(&binding, expected, 2, |_| {
                        called = true;
                        fs::write(&payload, b"incorrect-overwrite").unwrap();
                        Ok(())
                    })
                    .is_err()
            );
            assert!(!called);
            assert_eq!(fs::read(&payload).unwrap(), b"already-admitted-content");
            let new = store.create(binding.spec.clone(), Some(binding.generation), 2).unwrap();
            let rejected_payload = temporary.path().join("rejected-content");
            assert!(
                store
                    .with_admitted_received(&binding, expected, 2, |_| {
                        fs::write(&rejected_payload, b"stale").unwrap();
                        Ok(())
                    })
                    .is_err()
            );
            let wrong_kind = match kind {
                EndpointKind::BuilderTicket => EndpointKind::Transfer,
                EndpointKind::Transfer => EndpointKind::Substituter,
                EndpointKind::Substituter => EndpointKind::BaseLayer,
                EndpointKind::BaseLayer => EndpointKind::BuilderTicket,
            };
            assert!(
                store
                    .with_admitted_received(
                        &new,
                        EndpointReceiverRequirements {
                            kind: wrong_kind,
                            ..expected
                        },
                        2,
                        |_| {
                            fs::write(&rejected_payload, b"wrong-purpose").unwrap();
                            Ok(())
                        }
                    )
                    .is_err()
            );
            let mut forged = new.clone();
            forged.spec.trust_requirement = "forged-policy".into();
            assert!(
                store
                    .with_admitted_received(&forged, expected, 2, |_| {
                        fs::write(&rejected_payload, b"forged").unwrap();
                        Ok(())
                    })
                    .is_err()
            );
            assert!(!rejected_payload.exists());
            assert_eq!(fs::read(&payload).unwrap(), b"already-admitted-content");
        }
    }

    #[test]
    fn client_receiver_and_distinct_sources_share_leases_while_ticket_commit_succeeds() {
        use crate::remote_credential_state::acquire_ticket_state_mutation_guard;
        use crate::remote_credential_state::load_ticket_state;
        use crate::remote_credential_state::save_ticket_state;

        let temporary = tempfile::tempdir().unwrap();
        let client = EndpointBindingStore::open(temporary.path()).unwrap();
        let builder = client.create(spec("builder-a"), None, 1).unwrap();
        let substituter = client
            .create(
                EndpointBindingSpec {
                    kind: EndpointKind::Substituter,
                    ..spec("cache-a")
                },
                None,
                1,
            )
            .unwrap();
        let base = client
            .create(
                EndpointBindingSpec {
                    kind: EndpointKind::BaseLayer,
                    ..spec("layer-a")
                },
                None,
                1,
            )
            .unwrap();
        let expected = EndpointReceiverRequirements {
            kind: EndpointKind::BuilderTicket,
            scope: "farm-a",
            trust_requirement: "existing-ticket-policy",
            store_prefix: "/mantle/store",
        };
        let client_guard = client.admit_received(&builder, expected, 2).unwrap();
        let ticket_commit = acquire_ticket_state_mutation_guard(temporary.path()).unwrap();
        let receiver = EndpointBindingStore::open(temporary.path()).unwrap();
        let receiver_guard = receiver.admit_received(&builder, expected, 2).unwrap();
        let cache_guard = receiver
            .admit_received(
                &substituter,
                EndpointReceiverRequirements {
                    kind: EndpointKind::Substituter,
                    ..expected
                },
                2,
            )
            .unwrap();
        let base_guard = receiver
            .admit_received(
                &base,
                EndpointReceiverRequirements {
                    kind: EndpointKind::BaseLayer,
                    ..expected
                },
                2,
            )
            .unwrap();
        let mut tickets = load_ticket_state(temporary.path()).unwrap();
        tickets.next_ticket_sequence = 7;
        save_ticket_state(temporary.path(), &tickets).unwrap();
        assert_eq!(load_ticket_state(temporary.path()).unwrap().next_ticket_sequence, 7);
        drop(ticket_commit);
        assert_eq!(client_guard.binding(), receiver_guard.binding());
        assert_eq!(cache_guard.binding(), &substituter);
        assert_eq!(base_guard.binding(), &base);
        assert_eq!(receiver.resolve(&builder, 2).unwrap(), builder);
        assert_eq!(receiver.records().unwrap().len(), 3);
    }

    #[test]
    fn create_and_revoke_wait_for_every_shared_admission_to_drop() {
        use std::sync::mpsc;

        for revoke in [false, true] {
            let temporary = tempfile::tempdir().unwrap();
            let store = EndpointBindingStore::open(temporary.path()).unwrap();
            let binding = store.create(spec("builder-a"), None, 1).unwrap();
            let expected = EndpointReceiverRequirements {
                kind: EndpointKind::BuilderTicket,
                scope: "farm-a",
                trust_requirement: "existing-ticket-policy",
                store_prefix: "/mantle/store",
            };
            let client = store.admit_received(&binding, expected, 2).unwrap();
            let receiver = store.admit_received(&binding, expected, 2).unwrap();
            let (started_tx, started_rx) = mpsc::channel();
            let (done_tx, done_rx) = mpsc::channel();
            let exact = binding.clone();
            let mutation = thread::spawn(move || {
                started_tx.send(()).unwrap();
                let result = if revoke {
                    store.revoke(&exact)
                } else {
                    store.create(spec("builder-b"), None, 2).map(drop)
                };
                done_tx.send(result).unwrap();
            });
            started_rx.recv().unwrap();
            assert!(matches!(done_rx.recv_timeout(Duration::from_millis(30)), Err(mpsc::RecvTimeoutError::Timeout)));
            drop(client);
            assert!(matches!(done_rx.recv_timeout(Duration::from_millis(30)), Err(mpsc::RecvTimeoutError::Timeout)));
            drop(receiver);
            done_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
            mutation.join().unwrap();
            let reopened = EndpointBindingStore::open(temporary.path()).unwrap();
            if revoke {
                assert!(reopened.admit_received(&binding, expected, 2).is_err());
                assert!(reopened.create(binding.spec.clone(), None, 2).is_err());
                let next = reopened.create(binding.spec.clone(), Some(binding.generation), 2).unwrap();
                assert_eq!(next.generation, binding.generation + 1);
                assert!(reopened.admit_received(&binding, expected, 2).is_err());
            } else {
                assert_eq!(reopened.resolve(&binding, 2).unwrap(), binding);
                assert!(reopened.records().unwrap().iter().any(|record| record.binding.spec.name == "builder-b"));
            }
        }
    }

    #[test]
    fn binding_lock_has_bounded_wait_and_exclusive_mutation_blocks_readers() {
        let temporary = tempfile::tempdir().unwrap();
        let writer = acquire_binding_lease(temporary.path(), BindingLockMode::Exclusive).unwrap();
        for mode in [BindingLockMode::Shared, BindingLockMode::Exclusive] {
            let error = acquire_binding_lease_with_wait(
                temporary.path(),
                mode,
                Duration::from_millis(30),
                Duration::from_millis(1),
            )
            .unwrap_err();
            assert_eq!(error.to_string(), "error: remote-endpoint-state-lock-timeout");
        }
        drop(writer);
        let reader = acquire_binding_lease(temporary.path(), BindingLockMode::Shared).unwrap();
        let error = acquire_binding_lease_with_wait(
            temporary.path(),
            BindingLockMode::Exclusive,
            Duration::from_millis(30),
            Duration::from_millis(1),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "error: remote-endpoint-state-lock-timeout");
        drop(reader);
        assert!(acquire_binding_lease(temporary.path(), BindingLockMode::Exclusive).is_ok());
    }

    #[test]
    fn binding_lock_rejects_unsafe_files_and_private_directory_without_fallback() {
        use std::os::unix::fs::PermissionsExt as _;
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        let parent = temporary.path().join(REMOTE_TICKET_STATE_DIRECTORY);
        let lock = parent.join(ENDPOINT_BINDING_LOCK_FILENAME);
        fs::set_permissions(&lock, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(store.resolve(&binding, 2).is_err());
        fs::remove_file(&lock).unwrap();
        let outside = temporary.path().join("outside");
        fs::write(&outside, b"unchanged").unwrap();
        symlink(&outside, &lock).unwrap();
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
        assert!(store.revoke(&binding).is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"unchanged");
        fs::remove_file(&lock).unwrap();
        fs::create_dir(&lock).unwrap();
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
        fs::remove_dir(&lock).unwrap();
        let fifo = std::ffi::CString::new(lock.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
        fs::remove_file(&lock).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
        assert!(store.create(spec("builder-b"), None, 2).is_err());
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let moved = temporary.path().join("moved-private-state");
        fs::rename(&parent, &moved).unwrap();
        symlink(&moved, &parent).unwrap();
        assert!(EndpointBindingStore::open(temporary.path()).is_err());
        fs::remove_file(&parent).unwrap();
        fs::rename(&moved, &parent).unwrap();
        assert_eq!(store.resolve(&binding, 2).unwrap(), binding);
    }
}
