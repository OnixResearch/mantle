// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
//
// The compiler deadline is a wall-clock scheduling decision owned by this driver
// boundary: the deadline is enforced where the compiler is supervised, so the
// clock read stays in the driver.
#![allow(tigerstyle::ambient_clock)]
//! Explicit C/C++ compiler driver and existing-daemon object protocol.
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use crunch_rust_cache::cc::CcDependency;
use crunch_rust_cache::cc::CcDepfilePart;
use crunch_rust_cache::cc::CcRecord;
use crunch_rust_cache::cc::MAX_CC_DIAGNOSTIC_BYTES;
use crunch_rust_cache::cc::MAX_CC_OBJECT_BYTES;
use crunch_rust_cache_core::cc;
use serde::Deserialize;
use serde::Serialize;

use crate::Error;
use crate::read_frame;
use crate::write_frame;

pub const CC_PROTOCOL_SCHEMA: &str = "mantle-cc-cache-request-v1";
pub const MAX_CC_FRAME_BYTES: u64 = 40_000_000;
pub const SOCKET_ENV: &str = "MANTLE_CC_CACHE_SOCKET";
pub const BUILD_ID_ENV: &str = "MANTLE_COMPILE_BUILD_ID";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverRoot {
    pub alias: String,
    pub path: PathBuf,
    /// Admission attests all embedded paths are mapped by compiler options.
    pub path_independent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverPolicy {
    pub schema: String,
    pub compiler: PathBuf,
    pub roots: Vec<DriverRoot>,
    pub platform: String,
    pub environment: BTreeMap<String, String>,
    pub probe_script: Option<PathBuf>,
    pub cache_enabled: bool,
    pub slots_enabled: bool,
    pub output_path_independent: bool,
    pub elapsed_ms_max: u64,
    pub slot_wait_ms_max: u64,
    /// Outside build outputs. Cache and scheduling receipts are not proof.
    pub receipt_dir: PathBuf,
}

// Keep the wire envelope unboxed; boxing adds allocation without changing serialized protocol
// bytes.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Operation {
    Lookup { base_key: String },
    Publish { record: CcRecord, object: Option<Vec<u8>> },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub operation: Operation,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub record: Option<CcRecord>,
    pub object: Option<Vec<u8>>,
    pub accepted: bool,
}

/// Cache access is granted per direction so the two flags cannot be swapped.
pub(crate) struct CacheAccess {
    pub reads: bool,
    pub writes: bool,
}

pub(crate) fn serve(
    stream: &mut UnixStream,
    cache: &crunch_rust_cache::RustCache,
    access: CacheAccess,
    request: Request,
) -> Result<(), Error> {
    let is_schema_known = request.schema == CC_PROTOCOL_SCHEMA;
    if !is_schema_known {
        return Err(Error::Protocol("cc-protocol-schema".into()));
    }
    let response = match request.operation {
        Operation::Lookup { base_key } => {
            let record = if access.reads {
                cache.cc_lookup(&base_key).map_err(cache_error)?
            } else {
                None
            };
            let object = match &record {
                Some(r) => cache.cc_object(r).map_err(cache_error)?,
                None => None,
            };
            let record = record.filter(|r| r.status != 0 || object.is_some());
            Response {
                record,
                object,
                accepted: access.reads,
            }
        }
        Operation::Publish { record, object } => {
            if access.writes {
                cache.cc_publish(&record, object.as_deref()).map_err(cache_error)?;
            }
            Response {
                record: None,
                object: None,
                accepted: access.writes,
            }
        }
    };
    debug_assert!(is_schema_known, "a served request carries the protocol schema");
    debug_assert!(
        response.record.is_none() || response.accepted,
        "a cache record is only served when reads are enabled"
    );
    write_frame(stream, &response, MAX_CC_FRAME_BYTES)
}

fn request(socket: &Path, operation: Operation) -> Result<Response, Error> {
    let budget = Duration::from_secs(5);
    let deadline = Instant::now() + budget;
    let mut connection = crate::slots::connect_bounded(socket, budget).map_err(io)?;
    let mut stream = crate::slots::DeadlineIo::new(&mut connection, deadline);
    write_frame(
        &mut stream,
        &Request {
            schema: CC_PROTOCOL_SCHEMA.into(),
            operation,
        },
        MAX_CC_FRAME_BYTES,
    )?;
    read_frame(&mut stream, MAX_CC_FRAME_BYTES)
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CacheDisposition {
    ForwardUnchanged,
    CacheOff,
    ForwardUnprovenInputs,
    ForwardUnprovenPaths,
    CacheMiss,
    DaemonUnavailable,
    PublishUnavailable,
    ObjectHit,
    FailureHit,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SlotDisposition {
    Granted,
    Disabled,
    DegradedNoGrant,
    DegradedHandshakeFailed,
    NotRequestedCacheHit,
}

struct Execution {
    status: i32,
    slot: SlotDisposition,
    /// Signals, timeout and lingering descendants are not deterministic failures.
    cacheable: bool,
}

/// One driver invocation: the policy, the daemon socket and the build identity.
struct Driver<'a> {
    policy: &'a DriverPolicy,
    socket: Option<PathBuf>,
    build_id: String,
}

impl Driver<'_> {
    /// Runs the untouched invocation and reports the compiler's exit status.
    fn forward(&self, arguments: &[OsString], disposition: CacheDisposition) -> Result<i32, Error> {
        let execution = execute(self.policy, arguments, self.socket.as_deref(), &self.build_id, None)?;
        receipt(self.policy, disposition, execution.status, execution.slot, None);
        Ok(execution.status)
    }

    /// Asks the daemon for a cached record; an unavailable daemon is a miss.
    fn look_up(&self, base_key: &str) -> Option<Response> {
        let socket = self.socket.as_deref()?;
        let response = request(socket, Operation::Lookup {
            base_key: base_key.to_owned(),
        })
        .ok()?;
        Some(response)
    }

    /// Runs the compiler with its own streams captured.
    fn compile(&self, arguments: &[OsString], stdout: File, stderr: File) -> Result<Execution, Error> {
        execute(self.policy, arguments, self.socket.as_deref(), &self.build_id, Some((stdout, stderr)))
    }
}

/// Classifies the compiler arguments; unclassifiable arguments are forwarded.
fn classify_invocation(arguments: &[OsString]) -> Option<cc::Invocation> {
    let strings = arguments.iter().map(|argument| argument.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>()?;
    cc::classify(&strings).ok()
}

/// Adds the dependency capture flags when the invocation does not request them.
fn depfile_arguments(arguments: &[OsString], invocation: &cc::Invocation, depfile: &Path) -> Vec<OsString> {
    let mut actual = arguments.to_vec();
    if invocation.depfile.is_none() {
        actual.extend([
            OsString::from("-MD"),
            OsString::from("-MF"),
            depfile.to_path_buf().into_os_string(),
        ]);
    }
    actual
}

/// Replayed compiler output is advisory: a failed replay write never replaces
/// the compiler's own streams or its exit status.
fn replay_output(stdout: &[u8], stderr: &[u8]) {
    let _stdout_write = std::io::stdout().write_all(stdout);
    let _stderr_write = std::io::stderr().write_all(stderr);
}

/// Replays a cached record when it is admissible; otherwise the cache misses.
fn replay_hit(
    policy: &DriverPolicy,
    invocation: &cc::Invocation,
    inputs: &mut cc::KeyInputs,
    base_key: &str,
    response: &Response,
) -> Option<i32> {
    // Invalid/corrupt records are advisory misses, not build errors.
    if !response.accepted {
        return None;
    }
    let Some(record) = &response.record else {
        return None;
    };
    if record.validate().is_err() {
        return None;
    }
    if record.base_key != base_key {
        return None;
    }
    let Ok(manifest) = current_manifest(policy, &record.dependencies) else {
        return None;
    };
    inputs.manifest = manifest;
    debug_assert!(inputs.manifest.complete, "an admitted record restores a complete manifest");
    let kind = if policy.probe_script.is_some() {
        cc::CachedResultKind::Probe
    } else if record.status == 0 {
        cc::CachedResultKind::Object
    } else {
        cc::CachedResultKind::Failure
    };
    let admitted = normalized_invocation(invocation).and_then(|normalized| {
        cc::replay_admission(kind, &normalized, inputs).map_err(|e| Error::Core(format!("cc-replay:{e:?}")))
    });
    if admitted.ok().as_deref() != Some(record.key.as_str()) {
        return None;
    }
    if record.status == 0 {
        return replay_object(policy, invocation, record, response);
    }
    if invocation.depfile.is_some() {
        return None;
    }
    if path_entry_exists(Path::new(&invocation.output)) {
        return None;
    }
    // Failed compiles can delete destinations or leave depfiles.
    // Replay only the declared no-output mode, with no existing
    // destination whose filesystem effects would be skipped.
    debug_assert!(record.status != 0, "a failure replay only replays a failing record");
    replay_output(&record.stdout, &record.stderr);
    receipt(
        policy,
        CacheDisposition::FailureHit,
        record.status,
        SlotDisposition::NotRequestedCacheHit,
        Some(&record.key),
    );
    Some(record.status)
}

/// Restores a cached object; an unrestorable or mismatched object is a miss.
fn replay_object(
    policy: &DriverPolicy,
    invocation: &cc::Invocation,
    record: &CcRecord,
    response: &Response,
) -> Option<i32> {
    debug_assert!(record.status == 0, "an object replay only replays a successful record");
    let bytes = response.object.as_ref()?;
    let is_digest_matched = Some(blake3::hash(bytes).to_hex().as_str()) == record.object_digest.as_deref();
    if !is_digest_matched {
        return None;
    }
    if bytes.len() as u64 != record.object_bytes {
        return None;
    }
    if restore_outputs(invocation, policy, record, bytes).is_err() {
        return None;
    }
    debug_assert!(record.object_mode.is_some(), "a restored object carries its recorded mode");
    replay_output(&record.stdout, &record.stderr);
    receipt(policy, CacheDisposition::ObjectHit, 0, SlotDisposition::NotRequestedCacheHit, Some(&record.key));
    Some(0)
}

/// The compiler's captured streams and the temporary directory that owns them.
struct Capture {
    stdout: File,
    stderr: File,
    out_reader: File,
    err_reader: File,
    depfile: PathBuf,
}

/// Captures the compiler's streams; a capture that cannot be set up is not fatal.
fn capture_streams(temporary: &tempfile::TempDir) -> Option<Capture> {
    let stdout = File::create(temporary.path().join("stdout")).ok()?;
    let stderr = File::create(temporary.path().join("stderr")).ok()?;
    let out_reader = File::open(temporary.path().join("stdout")).ok()?;
    let err_reader = File::open(temporary.path().join("stderr")).ok()?;
    Some(Capture {
        stdout,
        stderr,
        out_reader,
        err_reader,
        depfile: temporary.path().join("dependencies.d"),
    })
}

/// The finished compilation, ready to become a cache record.
struct Publication<'a> {
    policy: &'a DriverPolicy,
    invocation: &'a cc::Invocation,
    inputs: &'a mut cc::KeyInputs,
    base_key: String,
    socket: Option<&'a Path>,
    depfile: PathBuf,
    out_reader: &'a mut File,
    err_reader: &'a mut File,
    execution: &'a Execution,
    is_output_existed: bool,
}

impl Publication<'_> {
    /// Publishes the finished compilation as a cache record.
    fn publish(&mut self) -> Result<(), Error> {
        if !self.execution.cacheable {
            return Ok(());
        }
        let after = prepare(self.policy, self.invocation)?;
        if key(self.invocation, &after)? != self.base_key {
            return Ok(());
        }
        let learned_path = self.invocation.depfile.as_deref().map(Path::new).unwrap_or(self.depfile.as_path());
        let dependencies = learn(self.policy, self.invocation, learned_path)?;
        self.inputs.manifest = current_manifest(self.policy, &dependencies)?;
        let key = key(self.invocation, self.inputs)?;
        if self.execution.status != 0 {
            let is_output_present = self.is_output_existed || path_entry_exists(Path::new(&self.invocation.output));
            if is_output_present || self.invocation.depfile.is_some() {
                return Ok(());
            }
        }
        let object = if self.execution.status == 0 {
            Some(bounded_read(Path::new(&self.invocation.output), MAX_CC_OBJECT_BYTES).map_err(io)?)
        } else {
            None
        };
        let record = CcRecord {
            schema: "mantle-cc-cache-record-v1".into(),
            base_key: self.base_key.clone(),
            key,
            dependencies,
            complete: true,
            status: self.execution.status,
            stdout: read_capture(self.out_reader)?,
            stderr: read_capture(self.err_reader)?,
            object_digest: object.as_ref().map(|b| blake3::hash(b).to_hex().to_string()),
            object_bytes: object.as_ref().map_or(0, |b| b.len() as u64),
            object_mode: if self.execution.status == 0 {
                Some(output_mode(Path::new(&self.invocation.output))?)
            } else {
                None
            },
            depfile: if self.invocation.depfile.is_some() {
                Some(depfile_template(self.policy, self.invocation, learned_path)?)
            } else {
                None
            },
            depfile_mode: if self.invocation.depfile.is_some() {
                Some(output_mode(learned_path)?)
            } else {
                None
            },
        };
        debug_assert!(record.complete, "a published record is complete");
        debug_assert_eq!(record.status, self.execution.status, "the published record carries the compiler status");
        debug_assert!(
            record.object_bytes == 0 || record.status == 0,
            "only a successful compilation publishes object bytes"
        );
        if let Some(socket) = self.socket {
            let response = request(socket, Operation::Publish { record, object })?;
            if !response.accepted {
                return Err(Error::Protocol("cc-publication-declined".into()));
            }
        }
        Ok(())
    }
}

/// Real compiler arguments are never reconstructed when classification fails.
pub fn run_driver(policy: &DriverPolicy, arguments: &[OsString]) -> Result<i32, Error> {
    validate_policy(policy)?;
    let driver = Driver {
        policy,
        socket: std::env::var_os(SOCKET_ENV).map(PathBuf::from),
        build_id: std::env::var(BUILD_ID_ENV).unwrap_or_else(|_| format!("driver-{}", std::process::id())),
    };
    let Some(invocation) = classify_invocation(arguments) else {
        return driver.forward(arguments, CacheDisposition::ForwardUnchanged);
    };
    if !policy.cache_enabled {
        return driver.forward(arguments, CacheDisposition::CacheOff);
    }
    let Ok(mut inputs) = prepare(policy, &invocation) else {
        return driver.forward(arguments, CacheDisposition::ForwardUnprovenInputs);
    };
    let Ok(base_key) = key(&invocation, &inputs) else {
        return driver.forward(arguments, CacheDisposition::ForwardUnprovenPaths);
    };
    let mut disposition = CacheDisposition::CacheMiss;
    if let Some(response) = driver.look_up(&base_key) {
        if let Some(status) = replay_hit(policy, &invocation, &mut inputs, &base_key, &response) {
            return Ok(status);
        }
    } else {
        disposition = CacheDisposition::DaemonUnavailable;
    }
    // All optional capture setup happens before the compiler. If it fails,
    // execute the untouched invocation with inherited streams instead.
    let Some(temporary) = tempfile::tempdir().ok() else {
        return driver.forward(arguments, CacheDisposition::ForwardUnprovenInputs);
    };
    let Some(mut capture) = capture_streams(&temporary) else {
        return driver.forward(arguments, CacheDisposition::ForwardUnprovenInputs);
    };
    let actual = depfile_arguments(arguments, &invocation, &capture.depfile);
    let is_output_existed = path_entry_exists(Path::new(&invocation.output));
    let execution = driver.compile(&actual, capture.stdout, capture.stderr)?;
    // Open read handles survive pathname loss; neither replay/publication nor
    // receipt failures can replace an actual compiler's streams or exit status.
    let _stdout_copy = std::io::copy(&mut capture.out_reader, &mut std::io::stdout());
    let _stderr_copy = std::io::copy(&mut capture.err_reader, &mut std::io::stderr());
    let mut publication = Publication {
        policy,
        invocation: &invocation,
        inputs: &mut inputs,
        base_key,
        socket: driver.socket.as_deref(),
        depfile: capture.depfile.clone(),
        out_reader: &mut capture.out_reader,
        err_reader: &mut capture.err_reader,
        execution: &execution,
        is_output_existed,
    };
    if publication.publish().is_err() && matches!(disposition, CacheDisposition::CacheMiss) {
        disposition = CacheDisposition::PublishUnavailable;
    }
    let is_compiled_disposition = matches!(
        disposition,
        CacheDisposition::CacheMiss | CacheDisposition::DaemonUnavailable | CacheDisposition::PublishUnavailable
    );
    debug_assert!(is_compiled_disposition, "a compiled run reports how the cache was used");
    let is_cache_hit_slot = matches!(execution.slot, SlotDisposition::NotRequestedCacheHit);
    debug_assert!(!is_cache_hit_slot, "a compiled run never reports a cache-hit slot");
    receipt(policy, disposition, execution.status, execution.slot, None);
    Ok(execution.status)
}

fn execute(
    policy: &DriverPolicy,
    arguments: &[OsString],
    socket: Option<&Path>,
    build_id: &str,
    capture: Option<(File, File)>,
) -> Result<Execution, Error> {
    let make_command = || -> Result<Command, Error> {
        let mut command = Command::new(&policy.compiler);
        command.args(arguments).process_group(0);
        if let Some((stdout, stderr)) = &capture {
            command
                .stdout(Stdio::from(stdout.try_clone().map_err(io)?))
                .stderr(Stdio::from(stderr.try_clone().map_err(io)?));
        }
        Ok(command)
    };
    let mut command = make_command()?;
    let outcome = if policy.slots_enabled {
        socket.map(|s| crate::slots::acquire_slot(s, build_id, Duration::from_millis(policy.slot_wait_ms_max)))
    } else {
        None
    };
    let (mut child, lease, slot) = match outcome {
        Some(crate::slots::SlotClientOutcome::Granted(mut lease)) => match lease.spawn(&mut command) {
            Ok(child) => (child, Some(lease), SlotDisposition::Granted),
            Err(_) => {
                drop(lease);
                (make_command()?.spawn().map_err(io)?, None, SlotDisposition::DegradedHandshakeFailed)
            }
        },
        _ => (
            command.spawn().map_err(io)?,
            None,
            if policy.slots_enabled {
                SlotDisposition::DegradedNoGrant
            } else {
                SlotDisposition::Disabled
            },
        ),
    };
    let group = child.id() as i32;
    let is_slot_leased = lease.is_some();
    let (completed, is_timed_out) = await_completion(&mut child, group, policy.elapsed_ms_max)?;
    debug_assert!(group != 0, "the compiler runs in its own process group");
    let is_lease_granted = matches!(slot, SlotDisposition::Granted);
    debug_assert!(is_slot_leased == is_lease_granted, "a held slot lease is only reported as granted");
    if let Some(lease) = lease {
        // Releasing the slot lease is advisory: the compiler already finished.
        let _released = lease.release();
    }
    Ok(Execution {
        status: if is_timed_out {
            124
        } else {
            completed.code().unwrap_or_else(|| 128 + completed.signal().unwrap_or(0))
        },
        slot,
        cacheable: !is_timed_out && completed.code().is_some(),
    })
}

/// Waits for the compiler process group, enforcing the elapsed-time deadline.
///
/// The deadline is a wall-clock scheduling decision owned by this driver
/// boundary, so the clock read and the deadline decision stay together.
fn await_completion(
    child: &mut std::process::Child,
    group: i32,
    elapsed_ms_max: u64,
) -> Result<(std::process::ExitStatus, bool), Error> {
    let started = Instant::now();
    let mut completed: Option<std::process::ExitStatus> = None;
    let mut is_timed_out = false;
    while completed.is_none() || crate::slots::process_group_live(group) {
        if completed.is_none() {
            completed = child.try_wait().map_err(io)?;
        }
        if completed.is_some() && !crate::slots::process_group_live(group) {
            break;
        }
        if started.elapsed().as_millis() >= u128::from(elapsed_ms_max) {
            is_timed_out = true;
            crate::slots::terminate_process_group(group);
            if completed.is_none() {
                completed = Some(child.wait().map_err(io)?);
            }
            while crate::slots::process_group_live(group) {
                std::thread::sleep(Duration::from_millis(5));
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let Some(completed) = completed else {
        return Err(Error::Process("cc-child-status-unavailable".into()));
    };
    let elapsed = started.elapsed();
    let is_deadline_passed = elapsed.as_millis() >= u128::from(elapsed_ms_max);
    debug_assert!(is_deadline_passed || !is_timed_out, "a timeout is only declared after the deadline passed");
    if is_timed_out {
        let is_group_live = crate::slots::process_group_live(group);
        let is_group_finished = !is_group_live;
        debug_assert!(is_group_finished, "a timed-out compiler leaves no live descendants");
    }
    Ok((completed, is_timed_out))
}

fn path_entry_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).map_or_else(|error| error.kind() != std::io::ErrorKind::NotFound, |_| true)
}

fn read_capture(file: &mut File) -> Result<Vec<u8>, Error> {
    file.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut bytes = Vec::new();
    let bound = MAX_CC_DIAGNOSTIC_BYTES.saturating_add(1) as u64;
    file.take(bound).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len() > MAX_CC_DIAGNOSTIC_BYTES {
        return Err(Error::Bound("cc-diagnostics-bound".into()));
    }
    Ok(bytes)
}

fn restore_outputs(
    invocation: &cc::Invocation,
    policy: &DriverPolicy,
    record: &CcRecord,
    bytes: &[u8],
) -> Result<(), Error> {
    // Render before replacing the object so a malformed template is only a miss.
    if invocation.depfile.is_some() && record.depfile.is_none() {
        return Err(Error::Protocol("cc-depfile-template-missing".into()));
    }
    write_depfile(invocation, policy, record.depfile.as_deref(), record.depfile_mode)?;
    atomic_output(
        Path::new(&invocation.output),
        bytes,
        record.object_mode.ok_or_else(|| Error::Protocol("cc-object-mode-missing".into()))?,
    )
}

/// Every declared check must hold for a policy to be admitted.
fn all_hold(checks: &[bool]) -> bool {
    checks.iter().all(|check| *check)
}

fn validate_policy(policy: &DriverPolicy) -> Result<(), Error> {
    let is_schema_known = policy.schema == "mantle-cc-driver-policy-v1";
    let is_compiler_absolute = policy.compiler.is_absolute();
    let is_receipt_dir_absolute = policy.receipt_dir.is_absolute();
    let is_root_count_admitted = !policy.roots.is_empty() && policy.roots.len() <= 64;
    let is_platform_named = !policy.platform.is_empty();
    let is_elapsed_admitted = policy.elapsed_ms_max > 0 && policy.elapsed_ms_max <= 3_600_000;
    let is_slot_wait_admitted = policy.slot_wait_ms_max > 0 && policy.slot_wait_ms_max <= 3_600_000;
    let is_policy_admitted = all_hold(&[
        is_schema_known,
        is_compiler_absolute,
        is_receipt_dir_absolute,
        is_root_count_admitted,
        is_platform_named,
        is_elapsed_admitted,
        is_slot_wait_admitted,
    ]);
    if !is_policy_admitted {
        return Err(Error::Policy("cc-driver-policy-invalid".into()));
    }
    let mut aliases = std::collections::BTreeSet::new();
    for root in &policy.roots {
        let is_path_absolute = root.path.is_absolute();
        let is_alias_named = !root.alias.is_empty();
        let is_alias_short = root.alias.len() <= 128;
        let is_alias_charset = root.alias.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
        let is_root_admitted = all_hold(&[is_path_absolute, is_alias_named, is_alias_short, is_alias_charset]);
        let is_alias_unique = is_root_admitted && aliases.insert(&root.alias);
        if !all_hold(&[is_root_admitted, is_alias_unique]) {
            return Err(Error::Policy("cc-driver-root-invalid".into()));
        }
    }
    debug_assert!(is_policy_admitted, "an admitted policy passes every check");
    debug_assert_eq!(aliases.len(), policy.roots.len(), "every admitted root contributes a distinct alias");
    Ok(())
}

fn prepare(policy: &DriverPolicy, invocation: &cc::Invocation) -> Result<cc::KeyInputs, Error> {
    let mut roots = Vec::with_capacity(policy.roots.len());
    let mut toolchain = blake3::Hasher::new();
    for root in &policy.roots {
        let digest = crate::declared_input_digest_blake3(
            &root.path,
            if root.path.is_dir() {
                crunch_rust_cache_core::wrapper::DeclaredInputKind::Directory
            } else {
                crunch_rust_cache_core::wrapper::DeclaredInputKind::File
            },
            268_435_456,
        )?;
        toolchain.update(root.alias.as_bytes());
        toolchain.update(digest.as_bytes());
        roots.push(cc::IdentityRoot {
            path: root.path.to_string_lossy().into_owned(),
            digest_blake3: digest,
            path_independent: root.path_independent,
        });
    }
    debug_assert_eq!(roots.len(), policy.roots.len(), "each declared root contributes one identity");
    locator(policy, &policy.compiler)?;
    let mut environment = std::env::vars().collect::<BTreeMap<_, _>>();
    environment.remove(SOCKET_ENV);
    environment.remove(BUILD_ID_ENV);
    if environment != policy.environment {
        return Err(Error::Policy("cc-environment-not-admitted".into()));
    }
    let normalized = environment
        .iter()
        .map(|(k, v)| Ok((k.clone(), normalize_value(policy, EnvironmentValue { name: k, value: v })?)))
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
    debug_assert_eq!(normalized.len(), environment.len(), "each environment value is normalized in place");
    let source = Path::new(&invocation.source);
    let source_digest = input_digest(source)?;
    Ok(cc::KeyInputs {
        compiler_blake3: file_digest(&policy.compiler)?,
        source_blake3: source_digest.clone(),
        platform_blake3: blake3::hash(policy.platform.as_bytes()).to_hex().to_string(),
        environment_blake3: blake3::hash(&serde_json::to_vec(&(&normalized, process_umask()?)).map_err(json)?)
            .to_hex()
            .to_string(),
        toolchain_blake3: toolchain.finalize().to_hex().to_string(),
        probe_blake3: policy.probe_script.as_deref().map(file_digest).transpose()?,
        roots,
        manifest: cc::DependencyManifest {
            complete: true,
            dependencies: vec![cc::DependencyIdentity {
                name: locator(policy, source)?,
                digest_blake3: source_digest,
            }],
        },
        output_path_independent: policy.output_path_independent,
    })
}

/// Colon-separated environment entries are either literal scalars or declared
/// paths. Tagged fields prevent a literal alias from colliding with a relocated
/// path; component-aware matching must never mask an undeclared sibling root.
#[derive(Serialize)]
enum EnvironmentEntry<'a> {
    Literal(&'a str),
    Path { alias: &'a str, relative: &'a str },
}

struct EnvironmentValue<'a> {
    name: &'a str,
    value: &'a str,
}

fn normalize_value<'a>(
    policy: &'a DriverPolicy,
    environment: EnvironmentValue<'a>,
) -> Result<Vec<EnvironmentEntry<'a>>, Error> {
    let EnvironmentValue { name, value } = environment;
    // Only these compiler variables define colon-separated search paths. An
    // arbitrary variable could instead name a single file containing a colon.
    if !matches!(
        name,
        "PATH"
            | "CPATH"
            | "C_INCLUDE_PATH"
            | "CPLUS_INCLUDE_PATH"
            | "OBJC_INCLUDE_PATH"
            | "LIBRARY_PATH"
            | "COMPILER_PATH"
    ) {
        if value.contains('/') {
            return Err(Error::Policy("cc-environment-path-unclassified".into()));
        }
        return Ok(vec![EnvironmentEntry::Literal(value)]);
    }
    value
        .split(':')
        .map(|entry| {
            let path = Path::new(entry);
            if !path.is_absolute() {
                // Empty and relative search entries depend on ambient cwd,
                // whose negative lookup tree has not been declared here.
                return Err(Error::Policy("cc-environment-path-unclassified".into()));
            }
            if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                return Err(Error::Policy("cc-environment-path-unclassified".into()));
            }
            let root = policy
                .roots
                .iter()
                .filter(|root| path.starts_with(&root.path))
                .max_by_key(|root| root.path.as_os_str().len())
                .ok_or_else(|| Error::Policy("cc-environment-path-unclassified".into()))?;
            if !root.path_independent {
                return Err(Error::Policy("cc-environment-path-dependent".into()));
            }
            // These are the admission invariants for relocation: the selected
            // root is component-matched, and only independent roots lose paths.
            debug_assert!(path.starts_with(&root.path));
            debug_assert!(root.path_independent);
            let root_path = root.path.to_str().ok_or_else(|| Error::Policy("cc-environment-path-utf8".into()))?;
            let suffix = entry
                .strip_prefix(root_path)
                .ok_or_else(|| Error::Policy("cc-environment-path-unclassified".into()))?;
            let relative = suffix.strip_prefix('/').unwrap_or(suffix);
            Ok(EnvironmentEntry::Path {
                alias: &root.alias,
                relative,
            })
        })
        .collect()
}

fn locator(policy: &DriverPolicy, path: &Path) -> Result<String, Error> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io)?.join(path)
    };
    if absolute.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(Error::Policy("cc-parent-path".into()));
    }
    let root = policy
        .roots
        .iter()
        .filter(|r| absolute.starts_with(&r.path))
        .max_by_key(|r| r.path.as_os_str().len())
        .ok_or_else(|| Error::Policy("cc-dependency-outside-roots".into()))?;
    let relative = absolute.strip_prefix(&root.path).map_err(|_| Error::Policy("cc-path-prefix".into()))?;
    Ok(format!("{}/{}", root.alias, relative.to_str().ok_or_else(|| Error::Policy("cc-path-utf8".into()))?))
}
fn resolve(policy: &DriverPolicy, locator: &str) -> Result<PathBuf, Error> {
    let (alias, relative) = locator.split_once('/').ok_or_else(|| Error::Policy("cc-locator".into()))?;
    if Path::new(relative).is_absolute()
        || Path::new(relative).components().any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(Error::Policy("cc-locator-escape".into()));
    }
    let root = policy
        .roots
        .iter()
        .find(|r| r.alias == alias)
        .ok_or_else(|| Error::Policy("cc-locator-root".into()))?;
    Ok(root.path.join(relative))
}
fn current_manifest(policy: &DriverPolicy, deps: &[CcDependency]) -> Result<cc::DependencyManifest, Error> {
    let dependencies = deps
        .iter()
        .map(|d| {
            Ok(cc::DependencyIdentity {
                name: d.locator.clone(),
                digest_blake3: input_digest(&resolve(policy, &d.locator)?)?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(cc::DependencyManifest {
        complete: true,
        dependencies,
    })
}
fn learn(policy: &DriverPolicy, invocation: &cc::Invocation, depfile: &Path) -> Result<Vec<CcDependency>, Error> {
    // -MMD omits system headers; it is never a complete replay manifest.
    if invocation.dependency_mode == Some(cc::DependencyMode::Mmd) {
        return Err(Error::Policy("cc-incomplete-system-manifest".into()));
    }
    let bytes = bounded_read(depfile, 1_048_576).map_err(io)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| Error::Protocol("cc-depfile-utf8".into()))?;
    let paths = cc::parse_depfile(text, &invocation.output, &invocation.source)
        .map_err(|e| Error::Core(format!("cc-depfile:{e:?}")))?;
    paths
        .into_iter()
        .map(|p| {
            Ok(CcDependency {
                locator: locator(policy, Path::new(&p))?,
                digest_blake3: input_digest(Path::new(&p))?,
            })
        })
        .collect()
}
fn write_depfile(
    invocation: &cc::Invocation,
    policy: &DriverPolicy,
    parts: Option<&[CcDepfilePart]>,
    mode: Option<u32>,
) -> Result<(), Error> {
    let Some(path) = &invocation.depfile else {
        return Ok(());
    };
    let parts = parts.ok_or_else(|| Error::Protocol("cc-depfile-template-missing".into()))?;
    let mut text = String::new();
    let mut rendered_parts = 0_usize;
    let mut rendered_bytes = 0_usize;
    for part in parts {
        let before = text.len();
        match part {
            CcDepfilePart::Literal { text: value } => text.push_str(value),
            CcDepfilePart::Output => text.push_str(&escape_make(&invocation.output)),
            CcDepfilePart::Root { alias } => {
                let root = policy
                    .roots
                    .iter()
                    .find(|r| r.alias == *alias)
                    .ok_or_else(|| Error::Policy("cc-depfile-root".into()))?;
                text.push_str(&escape_make(&root.path.to_string_lossy()));
            }
        }
        rendered_parts = rendered_parts.saturating_add(1);
        rendered_bytes = rendered_bytes.saturating_add(text.len().saturating_sub(before));
    }
    debug_assert_eq!(rendered_parts, parts.len(), "every template part is rendered exactly once");
    debug_assert_eq!(rendered_bytes, text.len(), "the rendered text is exactly the rendered parts");
    atomic_output(
        Path::new(path),
        text.as_bytes(),
        mode.ok_or_else(|| Error::Protocol("cc-depfile-mode-missing".into()))?,
    )
}

fn depfile_template(
    policy: &DriverPolicy,
    invocation: &cc::Invocation,
    path: &Path,
) -> Result<Vec<CcDepfilePart>, Error> {
    let text = String::from_utf8(bounded_read(path, 1_048_576).map_err(io)?)
        .map_err(|_| Error::Protocol("cc-depfile-utf8".into()))?;
    let mut patterns = policy
        .roots
        .iter()
        .map(|r| (escape_make(&r.path.to_string_lossy()), CcDepfilePart::Root { alias: r.alias.clone() }))
        .collect::<Vec<_>>();
    patterns.push((escape_make(&invocation.output), CcDepfilePart::Output));
    patterns.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
    // The rendered template is bounded by the part limit below; one part per
    // declared pattern is reserved before the template is rendered.
    let mut result = Vec::with_capacity(patterns.len().saturating_add(1));
    let mut rest = text.as_str();
    while !rest.is_empty() {
        let found = patterns
            .iter()
            .filter_map(|(pattern, part)| {
                rest.match_indices(pattern.as_str())
                    .find(|(offset, _)| {
                        let end = offset.saturating_add(pattern.len());
                        make_boundary(rest, *offset)
                            && (make_boundary(rest, end)
                                || (matches!(part, CcDepfilePart::Root { .. })
                                    && rest.as_bytes().get(end) == Some(&b'/')))
                    })
                    .map(|(offset, _)| (offset, pattern, part))
            })
            .min_by_key(|(offset, _, _)| *offset);
        let Some((offset, pattern, part)) = found else {
            result.push(CcDepfilePart::Literal { text: rest.into() });
            break;
        };
        if offset > 0 {
            result.push(CcDepfilePart::Literal {
                text: rest[..offset].into(),
            });
        }
        result.push(part.clone());
        let Some(end) = offset.checked_add(pattern.len()) else {
            return Err(Error::Bound("cc-depfile-template".into()));
        };
        rest = &rest[end..];
        if result.len() > 8192 {
            return Err(Error::Bound("cc-depfile-template".into()));
        }
    }
    debug_assert!(result.len() <= 8193, "a rendered template stays within the part bound");
    debug_assert_eq!(result.is_empty(), text.is_empty(), "an empty depfile renders no parts");
    Ok(result)
}

fn make_boundary(text: &str, offset: usize) -> bool {
    if offset == 0 || offset == text.len() {
        return true;
    }
    let Some(before_index) = offset.checked_sub(1) else {
        return true;
    };
    let before = text.as_bytes()[before_index];
    let at = text.as_bytes()[offset];
    // Callers use this at a token start and end. Escaped spaces and colons
    // belong to a filename, never a substitution boundary.
    let escaped =
        |index: usize| text.as_bytes()[..index].iter().rev().take_while(|byte| **byte == b'\\').count() % 2 != 0;
    (matches!(before, b' ' | b'\t' | b'\n' | b'\r' | b':') && !escaped(before_index))
        || (matches!(at, b' ' | b'\t' | b'\n' | b'\r' | b':') && !escaped(offset))
}

fn normalized_invocation(invocation: &cc::Invocation) -> Result<cc::Invocation, Error> {
    let cwd = std::env::current_dir().map_err(io)?;
    let absolute = |p: &str| {
        if Path::new(p).is_absolute() {
            p.to_owned()
        } else {
            cwd.join(p).to_string_lossy().into_owned()
        }
    };
    let mut normalized = invocation.clone();
    normalized.source = absolute(&normalized.source);
    for argument in &mut normalized.semantic_args {
        match argument {
            cc::SemanticArgument::Path { path, .. } => *path = absolute(path),
            cc::SemanticArgument::PrefixMap { from, .. } => *from = absolute(from),
            _ => {}
        }
    }
    debug_assert!(normalized.source.starts_with('/'), "the normalized source is an absolute path");
    debug_assert!(
        normalized.semantic_args.iter().all(|argument| match argument {
            cc::SemanticArgument::Path { path, .. } => path.starts_with('/'),
            cc::SemanticArgument::PrefixMap { from, .. } => from.starts_with('/'),
            _ => true,
        }),
        "every normalized semantic path is absolute"
    );
    Ok(normalized)
}
fn key(invocation: &cc::Invocation, inputs: &cc::KeyInputs) -> Result<String, Error> {
    cc::semantic_key(&normalized_invocation(invocation)?, inputs).map_err(|e| Error::Core(format!("cc-key:{e:?}")))
}
fn escape_make(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(' ', "\\ ")
        .replace('#', "\\#")
        .replace('$', "$$")
        .replace(':', "\\:")
}
fn output_mode(path: &Path) -> Result<u32, Error> {
    let metadata = fs::symlink_metadata(path).map_err(io)?;
    if !metadata.file_type().is_file() || metadata.nlink() != 1 {
        return Err(Error::Policy("cc-output-not-single-regular-file".into()));
    }
    Ok(metadata.permissions().mode() & 0o777)
}

fn process_umask() -> Result<u32, Error> {
    let status = String::from_utf8(bounded_read(Path::new("/proc/self/status"), 65_536).map_err(io)?)
        .map_err(|_| Error::Policy("cc-umask-utf8".into()))?;
    let value = status
        .lines()
        .find_map(|line| line.strip_prefix("Umask:"))
        .ok_or_else(|| Error::Policy("cc-umask-unavailable".into()))?;
    u32::from_str_radix(value.trim(), 8).map_err(|_| Error::Policy("cc-umask-invalid".into()))
}

fn atomic_output(path: &Path, bytes: &[u8], mode: u32) -> Result<(), Error> {
    // Do not replace symlinks/devices with cache bytes: the real compiler has
    // different effects there. Existing regular-file permissions are preserved.
    let mode = match fs::symlink_metadata(path) {
        Ok(_) => {
            let mode = output_mode(path)?;
            let _writable = fs::OpenOptions::new().write(true).custom_flags(libc::O_NOFOLLOW).open(path).map_err(io)?;
            mode
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => mode,
        Err(error) => return Err(io(error)),
    };
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(io)?;
    temporary.write_all(bytes).map_err(io)?;
    temporary.as_file().set_permissions(fs::Permissions::from_mode(mode)).map_err(io)?;
    temporary.persist(path).map_err(|e| io(e.error))?;
    Ok(())
}
/// Receipts are optional scheduling/cache observations, never compiler effects.
fn receipt(
    policy: &DriverPolicy,
    disposition: CacheDisposition,
    status: i32,
    slot: SlotDisposition,
    key: Option<&str>,
) {
    let core_disposition = match disposition {
        CacheDisposition::ObjectHit => cc::Disposition::Hit,
        CacheDisposition::FailureHit => cc::Disposition::FailureHit,
        CacheDisposition::ForwardUnchanged
        | CacheDisposition::CacheOff
        | CacheDisposition::ForwardUnprovenInputs
        | CacheDisposition::ForwardUnprovenPaths => cc::Disposition::Forwarded,
        CacheDisposition::CacheMiss => cc::Disposition::Compiled,
        CacheDisposition::DaemonUnavailable | CacheDisposition::PublishUnavailable => cc::Disposition::Degraded,
    };
    let observation = cc::Receipt {
        schema: cc::RECEIPT_SCHEMA.into(),
        disposition: core_disposition,
        key: key.map(str::to_owned),
        bypass: None,
        probe: policy.probe_script.is_some(),
    };
    debug_assert_eq!(observation.key.is_some(), key.is_some(), "the receipt carries exactly the key it was given");
    debug_assert!(
        matches!(observation.disposition, cc::Disposition::Hit) == matches!(disposition, CacheDisposition::ObjectHit),
        "an object hit is the only disposition that maps to a core hit"
    );
    let write = || -> Result<(), Error> {
        fs::create_dir_all(&policy.receipt_dir).map_err(io)?;
        let mut file = tempfile::Builder::new()
            .prefix("cc-")
            .suffix(".json")
            .tempfile_in(&policy.receipt_dir)
            .map_err(io)?;
        serde_json::to_writer(
            &mut file,
            &serde_json::json!({"schema":"mantle-cc-driver-receipt-v1",
            "claim_class":"machine-local-performance-only", "cache_disposition":disposition,
            "slot_disposition":slot,"compiler_status":status,"key":key, "cache":observation,
            "proof_evidence":false}),
        )
        .map_err(json)?;
        file.keep().map_err(|e| io(e.error))?;
        Ok(())
    };
    // Receipts are advisory: a failed receipt never replaces the compiler's
    // exit status, so the write result is deliberately not propagated.
    let _receipt_write = write();
}
fn bounded_read(path: &Path, max: u64) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > max {
        return Err(std::io::Error::other("cc-file-bound"));
    }
    let mut bytes = Vec::new();
    file.take(max.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(std::io::Error::other("cc-file-bound"));
    }
    Ok(bytes)
}
fn file_digest(path: &Path) -> Result<String, Error> {
    digest_file(path, false)
}
fn input_digest(path: &Path) -> Result<String, Error> {
    digest_file(path, true)
}

fn digest_file(path: &Path, check_volatile: bool) -> Result<String, Error> {
    let mut file = File::open(path).map_err(io)?;
    let metadata = file.metadata().map_err(io)?;
    if !metadata.is_file() || metadata.len() > 268_435_456 {
        return Err(Error::Bound("cc-file-bound".into()));
    }
    let mut buffer = [0_u8; 65_552];
    let mut overlap = 0;
    let mut total = 0_u64;
    let mut hash = blake3::Hasher::new();
    let mut count = file.read(&mut buffer[overlap..]).map_err(io)?;
    while count != 0 {
        total += count as u64;
        if total > 268_435_456 {
            return Err(Error::Bound("cc-file-bound".into()));
        }
        let Some(end) = overlap.checked_add(count) else {
            return Err(Error::Bound("cc-file-bound".into()));
        };
        hash.update(&buffer[overlap..end]);
        if check_volatile && cc::volatile_input(&buffer[..end]) {
            return Err(Error::Core(format!("cc-input:{:?}", cc::Bypass::VolatileInput)));
        }
        overlap = end.min(16);
        buffer.copy_within(end.saturating_sub(overlap)..end, 0);
        debug_assert!(overlap <= 16, "the overlap window stays within the boundary size");
        debug_assert!(end <= buffer.len(), "the read window stays within the buffer");
        count = file.read(&mut buffer[overlap..]).map_err(io)?;
    }
    Ok(hash.finalize().to_hex().to_string())
}
fn cache_error(e: crunch_rust_cache::Error) -> Error {
    Error::Process(e.to_string())
}
fn io(source: std::io::Error) -> Error {
    Error::Io {
        context: "cc-driver".into(),
        source,
    }
}
fn json(source: serde_json::Error) -> Error {
    Error::Json {
        context: "cc-driver".into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(root: &Path) -> DriverPolicy {
        DriverPolicy {
            schema: "mantle-cc-driver-policy-v1".into(),
            compiler: "/bin/sh".into(),
            roots: vec![DriverRoot {
                alias: "source".into(),
                path: root.to_path_buf(),
                path_independent: true,
            }],
            platform: "test-linux".into(),
            environment: BTreeMap::new(),
            probe_script: None,
            cache_enabled: false,
            slots_enabled: false,
            output_path_independent: true,
            elapsed_ms_max: 2_000,
            slot_wait_ms_max: 1_000,
            receipt_dir: root.join("receipts"),
        }
    }

    #[test]
    fn environment_relocation_preserves_identity_without_masking_other_inputs() {
        let mut admitted = policy(Path::new("/old/source"));
        let encoded = |policy: &DriverPolicy, value: &str| {
            serde_json::to_vec(&normalize_value(policy, EnvironmentValue { name: "PATH", value }).unwrap()).unwrap()
        };
        let original = encoded(&admitted, "/old/source/bin:/old/source/include");
        for ambient in ["", ".", "relative", "@source", "/old/source:"] {
            assert!(
                normalize_value(&admitted, EnvironmentValue {
                    name: "PATH",
                    value: ambient
                })
                .is_err()
            );
        }
        assert!(
            normalize_value(&admitted, EnvironmentValue {
                name: "PATH",
                value: "/old/source-other/bin"
            })
            .is_err()
        );
        assert!(
            normalize_value(&admitted, EnvironmentValue {
                name: "PATH",
                value: "-I/old/source/include"
            })
            .is_err()
        );
        assert!(
            normalize_value(&admitted, EnvironmentValue {
                name: "PATH",
                value: "/old/source/../outside"
            })
            .is_err()
        );
        assert!(
            normalize_value(&admitted, EnvironmentValue {
                name: "CUSTOM",
                value: "/old/source:other-file"
            })
            .is_err()
        );
        admitted.roots[0].path = "/new/source".into();
        assert_eq!(original, encoded(&admitted, "/new/source/bin:/new/source/include"));
        assert_ne!(original, encoded(&admitted, "/new/source/other:/new/source/include"));
        assert_ne!(original, encoded(&admitted, "/new/source/include:/new/source/bin"));
        admitted.roots[0].path_independent = false;
        assert!(
            normalize_value(&admitted, EnvironmentValue {
                name: "PATH",
                value: "/new/source/bin:/new/source/include"
            },)
            .is_err()
        );
    }

    #[test]
    fn advisory_receipt_failure_preserves_real_exit_status() {
        let root = tempfile::tempdir().unwrap();
        let mut policy = policy(root.path());
        fs::write(&policy.receipt_dir, b"not a directory").unwrap();
        assert_eq!(run_driver(&policy, &["-c".into(), "exit 7".into()]).unwrap(), 7);
        // Signal termination must retain the conventional signal status too.
        policy.receipt_dir = root.path().join("other-receipts");
        assert_eq!(run_driver(&policy, &["-c".into(), "kill -TERM $$".into()]).unwrap(), 128 + libc::SIGTERM);
    }

    #[test]
    fn timeout_terminates_descendants_even_without_slot_authority() {
        let root = tempfile::tempdir().unwrap();
        let mut policy = policy(root.path());
        policy.elapsed_ms_max = 100;
        let pidfile = root.path().join("group");
        let script = format!("printf '%s' $$ > '{}'; /bin/sleep 30 & wait", pidfile.display());
        let execution = execute(&policy, &["-c".into(), script.into()], None, "timeout", None).unwrap();
        let group = fs::read_to_string(pidfile).unwrap().parse::<i32>().unwrap();
        assert_eq!(execution.status, 124);
        assert!(!execution.cacheable);
        assert!(!crate::slots::process_group_live(group), "compiler descendants survived timeout");
    }

    #[test]
    fn real_command_streams_and_nonzero_status_are_preserved() {
        let root = tempfile::tempdir().unwrap();
        let mut out = tempfile::tempfile().unwrap();
        let mut err = tempfile::tempfile().unwrap();
        let execution = execute(
            &policy(root.path()),
            &["-c".into(), "printf output; printf diagnostic >&2; exit 9".into()],
            None,
            "streams",
            Some((out.try_clone().unwrap(), err.try_clone().unwrap())),
        )
        .unwrap();
        assert_eq!(execution.status, 9);
        assert_eq!(read_capture(&mut out).unwrap(), b"output");
        assert_eq!(read_capture(&mut err).unwrap(), b"diagnostic");
    }

    #[test]
    fn volatile_input_crossing_digest_buffer_boundary_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("a.h");
        let mut bytes = vec![b' '; 65_547];
        bytes.extend_from_slice(b"__TIMESTAMP__");
        fs::write(&path, &bytes).unwrap();
        assert!(input_digest(&path).is_err());
        fs::write(&path, b"#define VALUE 17\n").unwrap();
        assert_eq!(input_digest(&path).unwrap(), blake3::hash(b"#define VALUE 17\n").to_hex().as_str());
    }

    #[test]
    fn depfile_relocation_never_rewrites_filename_substrings() {
        let root = tempfile::tempdir().unwrap();
        let mut policy = policy(root.path());
        let depfile = root.path().join("a.d");
        let invocation = cc::classify(&[
            "-c".into(),
            "/old/a.c".into(),
            "-MD".into(),
            "-MF".into(),
            depfile.to_str().unwrap().into(),
            "-o".into(),
            "a.o".into(),
        ])
        .unwrap();
        policy.roots[0].path = "/old".into();
        fs::write(&depfile, b"a.o: /old/a.c /old/a.o.h /oldish/unchanged.h\n").unwrap();
        let parts = depfile_template(&policy, &invocation, &depfile).unwrap();
        policy.roots[0].path = "/new".into();
        let mut moved = invocation;
        moved.output = "b.o".into();
        write_depfile(&moved, &policy, Some(&parts), Some(0o640)).unwrap();
        assert_eq!(fs::read_to_string(&depfile).unwrap(), "b.o: /new/a.c /new/a.o.h /oldish/unchanged.h\n");
    }

    #[test]
    fn replay_preserves_modes_and_does_not_replace_symlinks() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("a.o");
        atomic_output(&output, b"object", 0o640).unwrap();
        assert_eq!(output_mode(&output).unwrap(), 0o640);
        atomic_output(&output, b"new object", 0o600).unwrap();
        assert_eq!(output_mode(&output).unwrap(), 0o640);
        let link = root.path().join("link.o");
        std::os::unix::fs::symlink(&output, &link).unwrap();
        assert!(atomic_output(&link, b"wrong", 0o600).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"new object");
        let hardlink = root.path().join("hardlink.o");
        fs::hard_link(&output, &hardlink).unwrap();
        assert!(atomic_output(&output, b"wrong", 0o600).is_err());
        assert_eq!(fs::read(&hardlink).unwrap(), b"new object");
    }
}
