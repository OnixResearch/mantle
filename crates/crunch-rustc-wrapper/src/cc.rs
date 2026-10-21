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

pub(crate) fn serve(
    stream: &mut UnixStream,
    cache: &crunch_rust_cache::RustCache,
    reads: bool,
    writes: bool,
    request: Request,
) -> Result<(), Error> {
    if request.schema != CC_PROTOCOL_SCHEMA {
        return Err(Error::Protocol("cc-protocol-schema".into()));
    }
    let response = match request.operation {
        Operation::Lookup { base_key } => {
            let record = if reads {
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
                accepted: reads,
            }
        }
        Operation::Publish { record, object } => {
            if writes {
                cache.cc_publish(&record, object.as_deref()).map_err(cache_error)?;
            }
            Response {
                record: None,
                object: None,
                accepted: writes,
            }
        }
    };
    write_frame(stream, &response, MAX_CC_FRAME_BYTES)
}

fn request(socket: &Path, operation: Operation) -> Result<Response, Error> {
    let mut stream = UnixStream::connect(socket).map_err(io)?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(io)?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(io)?;
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

/// Real compiler arguments are never reconstructed when classification fails.
pub fn run_driver(policy: &DriverPolicy, arguments: &[OsString]) -> Result<i32, Error> {
    validate_policy(policy)?;
    let socket = std::env::var_os(SOCKET_ENV).map(PathBuf::from);
    let build_id = std::env::var(BUILD_ID_ENV).unwrap_or_else(|_| format!("driver-{}", std::process::id()));
    let strings = arguments.iter().map(|a| a.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>();
    let invocation = strings.as_deref().and_then(|args| cc::classify(args).ok());
    let plain = |disposition| -> Result<i32, Error> {
        let execution = execute(policy, arguments, socket.as_deref(), &build_id, None)?;
        receipt(policy, disposition, execution.status, execution.slot, None);
        Ok(execution.status)
    };
    if !policy.cache_enabled || invocation.is_none() {
        return plain(if invocation.is_none() {
            CacheDisposition::ForwardUnchanged
        } else {
            CacheDisposition::CacheOff
        });
    }
    let invocation = invocation.expect("classified above");
    let Ok(mut inputs) = prepare(policy, &invocation) else {
        return plain(CacheDisposition::ForwardUnprovenInputs);
    };
    let Ok(base_key) = key(&invocation, &inputs) else {
        return plain(CacheDisposition::ForwardUnprovenPaths);
    };
    let lookup = socket.as_deref().map(|s| {
        request(s, Operation::Lookup {
            base_key: base_key.clone(),
        })
    });
    let mut disposition = CacheDisposition::CacheMiss;
    if let Some(Ok(response)) = &lookup {
        // Invalid/corrupt records are advisory misses, not build errors.
        if response.accepted
            && let Some(record) = &response.record
            && record.validate().is_ok()
            && record.base_key == base_key
            && let Ok(manifest) = current_manifest(policy, &record.dependencies)
        {
            inputs.manifest = manifest;
            let kind = if policy.probe_script.is_some() {
                cc::CachedResultKind::Probe
            } else if record.status == 0 {
                cc::CachedResultKind::Object
            } else {
                cc::CachedResultKind::Failure
            };
            let admitted = normalized_invocation(&invocation).and_then(|normalized| {
                cc::replay_admission(kind, &normalized, &inputs).map_err(|e| Error::Core(format!("cc-replay:{e:?}")))
            });
            if admitted.ok().as_deref() == Some(record.key.as_str()) {
                if record.status == 0 {
                    if let Some(bytes) = &response.object
                        && Some(blake3::hash(bytes).to_hex().as_str()) == record.object_digest.as_deref()
                        && bytes.len() as u64 == record.object_bytes
                        && restore_outputs(&invocation, policy, record, bytes).is_ok()
                    {
                        let _ = std::io::stdout().write_all(&record.stdout);
                        let _ = std::io::stderr().write_all(&record.stderr);
                        receipt(
                            policy,
                            CacheDisposition::ObjectHit,
                            0,
                            SlotDisposition::NotRequestedCacheHit,
                            Some(&record.key),
                        );
                        return Ok(0);
                    }
                } else if invocation.depfile.is_none() && !path_entry_exists(Path::new(&invocation.output)) {
                    // Failed compiles can delete destinations or leave depfiles.
                    // Replay only the declared no-output mode, with no existing
                    // destination whose filesystem effects would be skipped.
                    let _ = std::io::stdout().write_all(&record.stdout);
                    let _ = std::io::stderr().write_all(&record.stderr);
                    receipt(
                        policy,
                        CacheDisposition::FailureHit,
                        record.status,
                        SlotDisposition::NotRequestedCacheHit,
                        Some(&record.key),
                    );
                    return Ok(record.status);
                }
            }
        }
    } else {
        disposition = CacheDisposition::DaemonUnavailable;
    }

    // All optional capture setup happens before the compiler. If it fails,
    // execute the untouched invocation with inherited streams instead.
    let capture = || -> std::io::Result<_> {
        let temporary = tempfile::tempdir()?;
        let stdout = File::create(temporary.path().join("stdout"))?;
        let stderr = File::create(temporary.path().join("stderr"))?;
        let out_reader = File::open(temporary.path().join("stdout"))?;
        let err_reader = File::open(temporary.path().join("stderr"))?;
        Ok((temporary, stdout, stderr, out_reader, err_reader))
    };
    let Ok((temporary, stdout, stderr, mut out_reader, mut err_reader)) = capture() else {
        return plain(CacheDisposition::ForwardUnprovenInputs);
    };
    let depfile = temporary.path().join("dependencies.d");
    let mut actual = arguments.to_vec();
    if invocation.depfile.is_none() {
        actual.extend([
            OsString::from("-MD"),
            OsString::from("-MF"),
            depfile.clone().into_os_string(),
        ]);
    }
    let output_existed = path_entry_exists(Path::new(&invocation.output));
    let execution = execute(policy, &actual, socket.as_deref(), &build_id, Some((stdout, stderr)))?;
    // Open read handles survive pathname loss; neither replay/publication nor
    // receipt failures can replace an actual compiler's streams or exit status.
    let _ = std::io::copy(&mut out_reader, &mut std::io::stdout());
    let _ = std::io::copy(&mut err_reader, &mut std::io::stderr());
    let publish = || -> Result<(), Error> {
        if !execution.cacheable {
            return Ok(());
        }
        let after = prepare(policy, &invocation)?;
        if key(&invocation, &after)? != base_key {
            return Ok(());
        }
        let learned_path = invocation.depfile.as_deref().map(Path::new).unwrap_or(&depfile);
        let dependencies = learn(policy, &invocation, learned_path)?;
        inputs.manifest = current_manifest(policy, &dependencies)?;
        let key = key(&invocation, &inputs)?;
        if execution.status != 0
            && (output_existed || path_entry_exists(Path::new(&invocation.output)) || invocation.depfile.is_some())
        {
            return Ok(());
        }
        let object = if execution.status == 0 {
            Some(bounded_read(Path::new(&invocation.output), MAX_CC_OBJECT_BYTES).map_err(io)?)
        } else {
            None
        };
        let record = CcRecord {
            schema: "mantle-cc-cache-record-v1".into(),
            base_key,
            key,
            dependencies,
            complete: true,
            status: execution.status,
            stdout: read_capture(&mut out_reader)?,
            stderr: read_capture(&mut err_reader)?,
            object_digest: object.as_ref().map(|b| blake3::hash(b).to_hex().to_string()),
            object_bytes: object.as_ref().map_or(0, |b| b.len() as u64),
            object_mode: if execution.status == 0 {
                Some(output_mode(Path::new(&invocation.output))?)
            } else {
                None
            },
            depfile: if invocation.depfile.is_some() {
                Some(depfile_template(policy, &invocation, learned_path)?)
            } else {
                None
            },
            depfile_mode: if invocation.depfile.is_some() {
                Some(output_mode(learned_path)?)
            } else {
                None
            },
        };
        if let Some(socket) = &socket {
            let response = request(socket, Operation::Publish { record, object })?;
            if !response.accepted {
                return Err(Error::Protocol("cc-publication-declined".into()));
            }
        }
        Ok(())
    };
    if publish().is_err() && matches!(disposition, CacheDisposition::CacheMiss) {
        disposition = CacheDisposition::PublishUnavailable;
    }
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
    let started = Instant::now();
    let mut completed = None;
    let mut timed_out = false;
    loop {
        if completed.is_none() {
            completed = child.try_wait().map_err(io)?;
        }
        if completed.is_some() && !crate::slots::process_group_live(group) {
            break;
        }
        if started.elapsed().as_millis() >= u128::from(policy.elapsed_ms_max) {
            timed_out = true;
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
    let completed = completed.expect("child was waited above");
    if let Some(lease) = lease {
        let _ = lease.release();
    }
    Ok(Execution {
        status: if timed_out {
            124
        } else {
            completed.code().unwrap_or_else(|| 128 + completed.signal().unwrap_or(0))
        },
        slot,
        cacheable: !timed_out && completed.code().is_some(),
    })
}

fn path_entry_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).map_or_else(|error| error.kind() != std::io::ErrorKind::NotFound, |_| true)
}

fn read_capture(file: &mut File) -> Result<Vec<u8>, Error> {
    file.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut bytes = Vec::new();
    file.take(MAX_CC_DIAGNOSTIC_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(io)?;
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

fn validate_policy(policy: &DriverPolicy) -> Result<(), Error> {
    if policy.schema != "mantle-cc-driver-policy-v1"
        || !policy.compiler.is_absolute()
        || !policy.receipt_dir.is_absolute()
        || policy.roots.is_empty()
        || policy.roots.len() > 64
        || policy.platform.is_empty()
        || policy.elapsed_ms_max == 0
        || policy.elapsed_ms_max > 3_600_000
        || policy.slot_wait_ms_max == 0
        || policy.slot_wait_ms_max > 3_600_000
    {
        return Err(Error::Policy("cc-driver-policy-invalid".into()));
    }
    let mut aliases = std::collections::BTreeSet::new();
    for root in &policy.roots {
        if !root.path.is_absolute()
            || root.alias.is_empty()
            || root.alias.len() > 128
            || !root.alias.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !aliases.insert(&root.alias)
        {
            return Err(Error::Policy("cc-driver-root-invalid".into()));
        }
    }
    Ok(())
}

fn prepare(policy: &DriverPolicy, invocation: &cc::Invocation) -> Result<cc::KeyInputs, Error> {
    let mut roots = Vec::new();
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
    locator(policy, &policy.compiler)?;
    let mut environment = std::env::vars().collect::<BTreeMap<_, _>>();
    environment.remove(SOCKET_ENV);
    environment.remove(BUILD_ID_ENV);
    if environment != policy.environment {
        return Err(Error::Policy("cc-environment-not-admitted".into()));
    }
    let normalized = environment
        .iter()
        .map(|(k, v)| Ok((k.clone(), normalize_value(policy, v)?)))
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
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

fn normalize_value(policy: &DriverPolicy, value: &str) -> Result<String, Error> {
    let mut result = value.to_string();
    let mut roots = policy.roots.iter().collect::<Vec<_>>();
    roots.sort_by_key(|r| std::cmp::Reverse(r.path.as_os_str().len()));
    for root in roots {
        result = result.replace(
            root.path.to_str().ok_or_else(|| Error::Policy("cc-root-utf8".into()))?,
            &format!("@{}", root.alias),
        );
    }
    // Values containing any remaining absolute path are not classified semantic inputs.
    if result.starts_with('/') || result.contains("=/") || result.contains(":/") {
        return Err(Error::Policy("cc-environment-path-unclassified".into()));
    }
    Ok(result)
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
    for part in parts {
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
    }
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
    let mut result = Vec::new();
    let mut rest = text.as_str();
    while !rest.is_empty() {
        let found = patterns
            .iter()
            .filter_map(|(pattern, part)| {
                rest.match_indices(pattern.as_str())
                    .find(|(offset, _)| {
                        make_boundary(rest, *offset)
                            && (make_boundary(rest, offset + pattern.len())
                                || (matches!(part, CcDepfilePart::Root { .. })
                                    && rest.as_bytes().get(offset + pattern.len()) == Some(&b'/')))
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
        rest = &rest[offset + pattern.len()..];
        if result.len() > 8192 {
            return Err(Error::Bound("cc-depfile-template".into()));
        }
    }
    Ok(result)
}

fn make_boundary(text: &str, offset: usize) -> bool {
    if offset == 0 || offset == text.len() {
        return true;
    }
    let before = text.as_bytes()[offset - 1];
    let at = text.as_bytes()[offset];
    // Callers use this at a token start and end. Escaped spaces and colons
    // belong to a filename, never a substitution boundary.
    let escaped =
        |index: usize| text.as_bytes()[..index].iter().rev().take_while(|byte| **byte == b'\\').count() % 2 != 0;
    (matches!(before, b' ' | b'\t' | b'\n' | b'\r' | b':') && !escaped(offset - 1))
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
    let _ = write();
}
fn bounded_read(path: &Path, max: u64) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > max {
        return Err(std::io::Error::other("cc-file-bound"));
    }
    let mut bytes = Vec::new();
    file.take(max + 1).read_to_end(&mut bytes)?;
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
    loop {
        let count = file.read(&mut buffer[overlap..]).map_err(io)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > 268_435_456 {
            return Err(Error::Bound("cc-file-bound".into()));
        }
        hash.update(&buffer[overlap..overlap + count]);
        let end = overlap + count;
        if check_volatile && cc::volatile_input(&buffer[..end]) {
            return Err(Error::Core(format!("cc-input:{:?}", cc::Bypass::VolatileInput)));
        }
        overlap = end.min(16);
        buffer.copy_within(end - overlap..end, 0);
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
