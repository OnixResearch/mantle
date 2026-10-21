//! Owner-private, operation-bound Nix import and attenuated store-read endpoints.
//! Protocol 1.37 remains explicit; socket ownership never substitutes for the
//! canonical Basalt holder invocation required for these capability surfaces.
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crunch_nix_gateway::ParsedRequest;
use crunch_nix_gateway_core::Reject;
use crunch_nix_gateway_core::{self as core};
use mantle_application_contract::gateway_operations::GatewayOperationEvidence;
use serde::Deserialize;
use serde::Serialize;

use super::authority::GatewayAuthorityConfig;
use super::authority::GatewaySession;
use super::authority::GatewayVerifier;
use super::authority::Invocation;
use super::authority::Operation;
use super::authority::OperationRequest;
use crate::RunContext;
use crate::errors::RunError;

const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
const MAX_ENVELOPE_BYTES: u64 = 128 * 1024;
const MAX_NATIVE_OPERATIONS: u32 = 64;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeImportConfig {
    schema: String,
    authority: GatewayAuthorityConfig,
    endpoint_state_directory: PathBuf,
    endpoint_binding: crunch_remote_core::endpoint::EndpointBinding,
    transfer_scope: String,
    transfer_trust_requirement: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NarBinding {
    version: u32,
    logical_store_path: String,
    nar_hash: String,
    nar_size: u64,
    references: Vec<String>,
    deriver: String,
    content_address: String,
    #[serde(default)]
    max_bytes: u64,
    endpoint_generation: u64,
}
impl NarBinding {
    fn from_request(request: &ParsedRequest, max_bytes: u64, endpoint_generation: u64) -> Result<Self, Reject> {
        let ParsedRequest::AddToStoreNar {
            path,
            nar_hash,
            nar_size,
            references,
            deriver,
            content_address,
            ..
        } = request
        else {
            return Err(Reject::Operation);
        };
        let value = Self {
            version: 1,
            logical_store_path: path.clone(),
            nar_hash: nar_hash.clone(),
            nar_size: *nar_size,
            references: references.clone(),
            deriver: deriver.clone(),
            content_address: content_address.clone(),
            max_bytes,
            endpoint_generation,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Reject> {
        if self.version != 1
            || self.endpoint_generation == 0
            || !core::valid_store_path(&self.logical_store_path, false)
            || self.nar_hash.len() != 64
            || !self.nar_hash.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || self.nar_size == 0
            || self.nar_size > self.max_bytes
            || self.max_bytes > core::MAX_TRANSFER_BYTES
            || self.references.len() > core::MAX_STORE_PATHS as usize
            || self.references.iter().any(|path| !core::valid_store_path(path, false))
            || (!self.deriver.is_empty() && !core::valid_store_path(&self.deriver, true))
            || self.content_address.len() > 128
        {
            return Err(Reject::Bound);
        }
        Ok(())
    }
    fn digest(&self) -> Result<[u8; 32], Reject> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| Reject::Identity)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mantle-native-nar-upload-v1\0");
        hasher.update(&bytes);
        Ok(*hasher.finalize().as_bytes())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UploadEnvelope {
    version: u32,
    authority: OperationRequest,
    compact_token: String,
    nonce: [u8; 32],
    holder_signature: Vec<u8>,
    max_bytes: u64,
}
impl Drop for UploadEnvelope {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.compact_token.zeroize();
        self.nonce.zeroize();
        self.holder_signature.zeroize();
    }
}

fn failure(reason: Reject) -> RunError {
    RunError::Internal(reason.code().into())
}
fn bytes(path: &Path, maximum: u64) -> Result<zeroize::Zeroizing<Vec<u8>>, RunError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| failure(Reject::Authority))?;
    let metadata = file.metadata().map_err(|_| failure(Reject::Authority))?;
    let uid = std::fs::metadata("/proc/self").map_err(|_| failure(Reject::Authority))?.uid();
    if !metadata.is_file() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 || metadata.len() > maximum {
        return Err(failure(Reject::Authority));
    }
    let mut data = zeroize::Zeroizing::new(Vec::new());
    file.take(maximum + 1).read_to_end(&mut data).map_err(|_| failure(Reject::Bound))?;
    if data.len() as u64 > maximum {
        return Err(failure(Reject::Bound));
    }
    Ok(data)
}
fn now() -> Result<u64, Reject> {
    crate::remote_build::remote_cli_now_unix_s("gateway-native-upload").map_err(|_| Reject::Authority)
}

/// Header is the serialized NarBinding except max_bytes, which this utility
/// replaces with the operator-selected limit before computing the signed tuple.
pub(crate) fn upload_binding_header(header: &Path, max_bytes: u64) -> Result<(serde_json::Value, usize), RunError> {
    let input = bytes(header, MAX_ENVELOPE_BYTES)?;
    let mut value: NarBinding = serde_json::from_slice(&input).map_err(|_| failure(Reject::Identity))?;
    value.max_bytes = max_bytes;
    let digest = value.digest().map_err(failure)?;
    Ok((
        serde_json::json!({"operation":Operation::InputUpload,"target":value.logical_store_path,"request_digest":digest}),
        input.len(),
    ))
}

struct SocketGuard {
    path: PathBuf,
    dev: u64,
    ino: u64,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        if let Ok(metadata) = std::fs::symlink_metadata(&self.path)
            && metadata.dev() == self.dev
            && metadata.ino() == self.ino
        {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
fn bind(path: &Path) -> Result<(UnixListener, SocketGuard), RunError> {
    let parent = path.parent().ok_or_else(|| failure(Reject::Identity))?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|_| failure(Reject::Authority))?;
    let uid = std::fs::metadata("/proc/self").map_err(|_| failure(Reject::Authority))?.uid();
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
        return Err(failure(Reject::Authority));
    }
    let listener = UnixListener::bind(path).map_err(|_| failure(Reject::Authority))?;
    let metadata = std::fs::symlink_metadata(path).map_err(|_| failure(Reject::Authority))?;
    let guard = SocketGuard {
        path: path.into(),
        dev: metadata.dev(),
        ino: metadata.ino(),
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|_| failure(Reject::Authority))?;
    listener.set_nonblocking(true).map_err(|_| failure(Reject::Authority))?;
    Ok((listener, guard))
}
fn peer(stream: &UnixStream) -> Result<(), Reject> {
    use std::os::fd::AsRawFd;
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let status = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut size,
        )
    };
    let uid = std::fs::metadata("/proc/self").map_err(|_| Reject::Authority)?.uid();
    if status != 0 || size as usize != std::mem::size_of::<libc::ucred>() || credentials.uid != uid {
        return Err(Reject::Authority);
    }
    Ok(())
}

pub(crate) fn serve_import_once(
    ctx: &RunContext,
    authority_config: &Path,
    socket: &Path,
    envelope: &Path,
    staging_directory: &Path,
) -> Result<GatewayOperationEvidence, RunError> {
    let config: NativeImportConfig =
        serde_json::from_slice(&bytes(authority_config, MAX_CONFIG_BYTES)?).map_err(|_| failure(Reject::Identity))?;
    if config.schema != "mantle-native-import-config-v1" || config.endpoint_state_directory.as_os_str().is_empty() {
        return Err(failure(Reject::Identity));
    }
    let verifier = GatewayVerifier::open(config.authority).map_err(|_| failure(Reject::Authority))?;
    let mut envelope: UploadEnvelope =
        serde_json::from_slice(&bytes(envelope, MAX_ENVELOPE_BYTES)?).map_err(|_| failure(Reject::Identity))?;
    if envelope.version != 1
        || envelope.authority.operation != Operation::InputUpload
        || envelope.max_bytes == 0
        || envelope.max_bytes > core::MAX_TRANSFER_BYTES
    {
        return Err(failure(Reject::Authority));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| failure(Reject::Authority))?;
    let mut store = runtime.block_on(super::open_read_authority_async(ctx))?;
    let (listener, _socket_guard) = bind(socket)?;
    let deadline = Instant::now() + Duration::from_secs(core::MAX_NEGOTIATION_SECS);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(value) => break value,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(_) => return Err(failure(Reject::Bound)),
        }
    };
    peer(&stream).map_err(failure)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(core::MAX_IDLE_SECS)))
        .map_err(|_| failure(Reject::Bound))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(core::MAX_IDLE_SECS)))
        .map_err(|_| failure(Reject::Bound))?;
    crunch_nix_gateway::handshake(&mut stream).map_err(|_| failure(Reject::Version))?;
    for _ in 0..MAX_NATIVE_OPERATIONS {
        let request = crunch_nix_gateway::read_request(&mut stream).map_err(|_| failure(Reject::Operation))?;
        if let ParsedRequest::AddToStoreNar { path, nar_size, .. } = &request {
            let import_path = path.clone();
            let declared_bytes = *nar_size;
            use crunch_remote_core::endpoint::EndpointKind;
            use crunch_remote_core::endpoint::EndpointReceiverRequirements;
            let endpoint_store =
                crate::remote_build::endpoint::EndpointBindingStore::open(&config.endpoint_state_directory)?;
            let _endpoint_guard = endpoint_store.admit_received(
                &config.endpoint_binding,
                EndpointReceiverRequirements {
                    kind: EndpointKind::Transfer,
                    scope: &config.transfer_scope,
                    trust_requirement: &config.transfer_trust_requirement,
                    store_prefix: &ctx.store_prefix,
                },
                now().map_err(failure)?,
            )?;
            let result = import(
                &mut stream,
                &mut store,
                request,
                &mut envelope,
                &verifier,
                &config.endpoint_binding,
                staging_directory,
                &runtime,
            );
            let verified_bytes = match result {
                Ok(bytes) => bytes,
                Err(reason) => {
                    let _ = crunch_nix_gateway::write_rejection(&mut stream, reason);
                    return Err(failure(reason));
                }
            };
            let missing_paths = runtime.block_on(store.query_missing(&[import_path])).map_err(failure)?.len();
            if missing_paths != 0 || verified_bytes != declared_bytes {
                return Err(failure(Reject::Authority));
            }
            crunch_nix_gateway::write_number(&mut stream, 0x616c7473).map_err(|_| failure(Reject::Bound))?;
            stream.flush().map_err(|_| failure(Reject::Bound))?;
            return Ok(GatewayOperationEvidence::Import {
                declared_bytes,
                verified_bytes,
                admitted_bytes: envelope.max_bytes,
                missing_paths,
            });
        }
        runtime.block_on(store.answer(request, &mut stream)).map_err(failure)?;
    }
    Err(failure(Reject::Bound))
}

fn import(
    stream: &mut UnixStream,
    store: &mut crunch_nix_gateway::store::VerifiedStore,
    request: ParsedRequest,
    envelope: &mut UploadEnvelope,
    verifier: &GatewayVerifier,
    endpoint: &crunch_remote_core::endpoint::EndpointBinding,
    staging: &Path,
    runtime: &tokio::runtime::Runtime,
) -> Result<u64, Reject> {
    let binding = NarBinding::from_request(&request, envelope.max_bytes, endpoint.generation)?;
    let digest = binding.digest()?;
    if envelope.authority.target != binding.logical_store_path
        || envelope.authority.request_digest != digest
        || envelope.authority.endpoint != endpoint.spec.name
        || envelope.authority.project != endpoint.spec.scope
    {
        return Err(Reject::Authority);
    }
    let signature =
        basalt::ucan::SignerSignature::try_from_slice(&envelope.holder_signature).map_err(|_| Reject::Authority)?;
    let nonce = basalt::ucan::HolderInvocationNonce::from_bytes(envelope.nonce);
    let authority = std::mem::replace(&mut envelope.authority, OperationRequest {
        operation: Operation::InputUpload,
        subject: String::new(),
        account: String::new(),
        project: String::new(),
        endpoint: String::new(),
        audience: String::new(),
        generation: 0,
        request_digest: [0; 32],
        target: String::new(),
    });
    let session = verifier
        .verify(
            authority,
            Invocation {
                compact_token: &envelope.compact_token,
                nonce,
                holder_signature: &signature,
            },
            now()?,
        )
        .map_err(|_| Reject::Authority)?;
    let reauthorize = || {
        if now()? >= endpoint.spec.expires_unix_s {
            return Err(Reject::Expired);
        }
        session
            .require(Operation::InputUpload, &binding.logical_store_path, digest)
            .map_err(|_| Reject::Authority)?;
        let facts = verifier.revalidate(&session, now()?).map_err(|_| Reject::Authority)?;
        if !facts.permits(core::Permission::Upload) {
            return Err(Reject::Authority);
        }
        Ok(())
    };
    crunch_nix_gateway::mutation::import_admitted_transfer(
        stream,
        store,
        request,
        std::slice::from_ref(&binding.logical_store_path),
        envelope.max_bytes,
        staging,
        runtime,
        reauthorize,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum StoreReadOperation {
    IsValidPath,
    QueryPathInfo,
}

pub(crate) fn store_read_binding(
    path: &str,
    operation: StoreReadOperation,
    endpoint_generation: u64,
) -> Result<serde_json::Value, RunError> {
    let digest = store_read_digest(path, operation, endpoint_generation).map_err(failure)?;
    Ok(serde_json::json!({"operation":Operation::StoreRead,"target":path,"request_digest":digest}))
}

fn store_read_digest(path: &str, operation: StoreReadOperation, generation: u64) -> Result<[u8; 32], Reject> {
    if !core::valid_store_path(path, false) || generation == 0 {
        return Err(Reject::Identity);
    }
    let mut hash = blake3::Hasher::new();
    hash.update(b"mantle-native-store-read-v1\0");
    hash.update(&[match operation {
        StoreReadOperation::IsValidPath => 1,
        StoreReadOperation::QueryPathInfo => 2,
    }]);
    hash.update(&(path.len() as u64).to_le_bytes());
    hash.update(path.as_bytes());
    hash.update(&generation.to_le_bytes());
    Ok(*hash.finalize().as_bytes())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreReadConfig {
    schema: String,
    authority: GatewayAuthorityConfig,
    endpoint_state_directory: PathBuf,
    endpoint_binding: crunch_remote_core::endpoint::EndpointBinding,
    read_scope: String,
    read_trust_requirement: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreReadEnvelope {
    version: u32,
    authority: OperationRequest,
    compact_token: String,
    nonce: [u8; 32],
    holder_signature: Vec<u8>,
}
impl Drop for StoreReadEnvelope {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.compact_token.zeroize();
        self.nonce.zeroize();
        self.holder_signature.zeroize();
    }
}

/// Nothing returns this store or its OutputLookup authority. The only accepted
/// request names one exact signed path and operation; even response writes are
/// revalidated after asynchronous signed PathInfo/CAS verification completes.
pub(crate) fn serve_store_read_once(
    ctx: &RunContext,
    authority_config: &Path,
    socket: &Path,
    envelope: &Path,
) -> Result<GatewayOperationEvidence, RunError> {
    use crunch_remote_core::endpoint::EndpointKind;
    use crunch_remote_core::endpoint::EndpointReceiverRequirements;
    let config: StoreReadConfig =
        serde_json::from_slice(&bytes(authority_config, MAX_CONFIG_BYTES)?).map_err(|_| failure(Reject::Identity))?;
    if config.schema != "mantle-native-store-read-config-v1" || config.endpoint_state_directory.as_os_str().is_empty() {
        return Err(failure(Reject::Identity));
    }
    let verifier = GatewayVerifier::open(config.authority).map_err(|_| failure(Reject::Authority))?;
    let mut envelope: StoreReadEnvelope =
        serde_json::from_slice(&bytes(envelope, MAX_ENVELOPE_BYTES)?).map_err(|_| failure(Reject::Identity))?;
    if envelope.version != 1
        || envelope.authority.operation != Operation::StoreRead
        || envelope.authority.endpoint != config.endpoint_binding.spec.name
        || envelope.authority.project != config.endpoint_binding.spec.scope
    {
        return Err(failure(Reject::Authority));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| failure(Reject::Authority))?;
    let store = runtime.block_on(super::open_read_authority_async(ctx))?;
    let (listener, _socket_guard) = bind(socket)?;
    let deadline = Instant::now() + Duration::from_secs(core::MAX_NEGOTIATION_SECS);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(value) => break value,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(_) => return Err(failure(Reject::Bound)),
        }
    };
    peer(&stream).map_err(failure)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(core::MAX_IDLE_SECS)))
        .map_err(|_| failure(Reject::Bound))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(core::MAX_IDLE_SECS)))
        .map_err(|_| failure(Reject::Bound))?;
    crunch_nix_gateway::handshake(&mut stream).map_err(|_| failure(Reject::Version))?;
    let request = crunch_nix_gateway::read_request(&mut stream).map_err(|_| failure(Reject::Operation))?;
    let (operation, path) = match &request {
        ParsedRequest::IsValidPath { path } => (StoreReadOperation::IsValidPath, path.as_str()),
        ParsedRequest::QueryPathInfo { path } => (StoreReadOperation::QueryPathInfo, path.as_str()),
        _ => {
            let _ = crunch_nix_gateway::write_rejection(&mut stream, Reject::Operation);
            return Err(failure(Reject::Operation));
        }
    };
    let digest = store_read_digest(path, operation, config.endpoint_binding.generation).map_err(failure)?;
    if envelope.authority.target != path || envelope.authority.request_digest != digest {
        let _ = crunch_nix_gateway::write_rejection(&mut stream, Reject::Authority);
        return Err(failure(Reject::Authority));
    }
    let endpoint_store = crate::remote_build::endpoint::EndpointBindingStore::open(&config.endpoint_state_directory)?;
    let _endpoint_guard = endpoint_store.admit_received(
        &config.endpoint_binding,
        EndpointReceiverRequirements {
            kind: EndpointKind::Substituter,
            scope: &config.read_scope,
            trust_requirement: &config.read_trust_requirement,
            store_prefix: &ctx.store_prefix,
        },
        now().map_err(failure)?,
    )?;
    let signature = basalt::ucan::SignerSignature::try_from_slice(&envelope.holder_signature)
        .map_err(|_| failure(Reject::Authority))?;
    let nonce = basalt::ucan::HolderInvocationNonce::from_bytes(envelope.nonce);
    let authority = std::mem::replace(&mut envelope.authority, OperationRequest {
        operation: Operation::StoreRead,
        subject: String::new(),
        account: String::new(),
        project: String::new(),
        endpoint: String::new(),
        audience: String::new(),
        generation: 0,
        request_digest: [0; 32],
        target: String::new(),
    });
    let session = verifier
        .verify(
            authority,
            Invocation {
                compact_token: &envelope.compact_token,
                nonce,
                holder_signature: &signature,
            },
            now().map_err(failure)?,
        )
        .map_err(|_| failure(Reject::Authority))?;
    let mut output = StoreReadWriter {
        stream: &mut stream,
        verifier: &verifier,
        session: &session,
        path: session.target(),
        digest,
        expires: config.endpoint_binding.spec.expires_unix_s,
        authorized_writes: 0,
        response_bytes: 0,
    };
    output.check().map_err(failure)?;
    runtime.block_on(store.answer(request, &mut output)).map_err(failure)?;
    Ok(GatewayOperationEvidence::StoreRead {
        authorized_writes: output.authorized_writes,
        response_bytes: output.response_bytes,
    })
}

struct StoreReadWriter<'a> {
    stream: &'a mut UnixStream,
    verifier: &'a GatewayVerifier,
    session: &'a GatewaySession,
    path: &'a str,
    digest: [u8; 32],
    expires: u64,
    authorized_writes: u64,
    response_bytes: u64,
}
impl StoreReadWriter<'_> {
    fn check(&self) -> Result<(), Reject> {
        let time = now()?;
        if time >= self.expires {
            return Err(Reject::Expired);
        }
        self.session.require(Operation::StoreRead, self.path, self.digest).map_err(|_| Reject::Authority)?;
        let facts = self.verifier.revalidate(self.session, time).map_err(|_| Reject::Authority)?;
        if !facts.permits(core::Permission::StoreRead) {
            return Err(Reject::Authority);
        }
        Ok(())
    }
}
impl Write for StoreReadWriter<'_> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.check()
            .map_err(|reason| std::io::Error::new(std::io::ErrorKind::PermissionDenied, reason.code()))?;
        let written = self.stream.write(data)?;
        self.authorized_writes = self.authorized_writes.saturating_add(1);
        self.response_bytes = self.response_bytes.saturating_add(written as u64);
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.check()
            .map_err(|reason| std::io::Error::new(std::io::ErrorKind::PermissionDenied, reason.code()))?;
        self.stream.flush()
    }
}
