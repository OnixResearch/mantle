//! Owner-only local worker-protocol listener. No network/public transport.

use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crunch_nix_gateway_core::{self as core, Reject};
use rustix::net::sockopt::socket_peercred;
use rustix::process::geteuid;
use zeroize::Zeroizing;

use crate::store::VerifiedStore;
use crate::{handshake, read_request, write_rejection};

fn denied() -> io::Error {
    io::Error::new(ErrorKind::PermissionDenied, Reject::Authority.code())
}

/// Refuse a socket under a shared, symlinked or other-owner directory even
/// when the socket itself would later be changed to mode 0600.
fn bind_private(socket: &Path) -> io::Result<(UnixListener, BoundSocket)> {
    let parent = socket.parent().ok_or_else(denied)?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.file_type().is_dir() || metadata.uid() != geteuid().as_raw()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(denied());
    }
    let listener = UnixListener::bind(socket)?;
    let metadata = match fs::symlink_metadata(socket) {
        Ok(metadata) => metadata,
        Err(error) => {
            let _ = fs::remove_file(socket);
            return Err(error);
        }
    };
    let guard = BoundSocket { path: socket.to_path_buf(), dev: metadata.dev(), ino: metadata.ino() };
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    Ok((listener, guard))
}

struct BoundSocket {
    path: PathBuf,
    dev: u64,
    ino: u64,
}

impl Drop for BoundSocket {
    fn drop(&mut self) {
        if let Ok(metadata) = fs::symlink_metadata(&self.path)
            && metadata.file_type().is_socket()
            && metadata.dev() == self.dev && metadata.ino() == self.ino
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Called only with a listener bound inside a private owner-controlled state
/// directory. `stop` is set by the host's existing signal-handling boundary.
pub fn serve_private(socket: &Path, store: VerifiedStore, stop: &AtomicBool) -> io::Result<()> {
    let (listener, _socket_guard) = bind_private(socket)?;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                // Only the listener's own accept/bind errors are fatal.
                // Wrong magic, a partial frame or disconnect terminates this
                // connection without stopping the private service.
                let _ = serve_connection(stream, &store, &runtime);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(25)),
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

const CREDENTIAL_MAGIC: [u8; 8] = *b"MNTLAUT1";
const CHALLENGE_BYTES: usize = 32;
const MAX_CREDENTIAL_ENVELOPE_BYTES: u64 = 512;

/// Challenge-bound ticket presentation on a separate credentialed socket.
/// The native Nix worker stream begins only after verification on this same
/// connection. The operator-side bridge reads its bearer from an owned FD;
/// native Nix clients cannot send this private preface themselves.
/// Echoing a fresh challenge rejects an old frame, not reuse of a stolen
/// bearer with a new challenge. Transport confidentiality and the existing
/// atomic ticket redemption at the remote build boundary remain required.
///
/// The verifier must use the current persisted ticket state and the
/// service-observed endpoint/time, not any field supplied by the client.
/// A successful envelope never grants BuildPaths in this read-only adapter.
pub fn authenticate_ticket_envelope(
    stream: &mut (impl Read + Write),
    challenge: &[u8; CHALLENGE_BYTES],
    expected_audience: &str,
    verify: impl FnOnce(&str, &str) -> Result<(), Reject>,
) -> Result<(), Reject> {
    if expected_audience.is_empty() || expected_audience.len() > core::MAX_IDENTITY_BYTES
        || !expected_audience.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Reject::Audience);
    }
    stream.write_all(&CREDENTIAL_MAGIC).map_err(|_| Reject::Bound)?;
    stream.write_all(challenge).map_err(|_| Reject::Bound)?;
    stream.flush().map_err(|_| Reject::Bound)?;
    let len = crate::read_number(stream).map_err(|_| Reject::Bound)?;
    if len < (CHALLENGE_BYTES + 6) as u64 || len > MAX_CREDENTIAL_ENVELOPE_BYTES {
        return Err(Reject::Bound);
    }
    let mut frame = Zeroizing::new(vec![0_u8; len as usize]);
    stream.read_exact(&mut frame).map_err(|_| Reject::Bound)?;
    if frame[..CHALLENGE_BYTES] != *challenge {
        return Err(Reject::Authority);
    }
    let mut cursor = CHALLENGE_BYTES;
    let ticket_id = credential_field(&frame, &mut cursor)?;
    let bearer = credential_field(&frame, &mut cursor)?;
    let audience = credential_field(&frame, &mut cursor)?;
    if cursor != frame.len() || ticket_id.is_empty() || bearer.is_empty()
        || ticket_id.len() > core::MAX_IDENTITY_BYTES || bearer.len() > core::MAX_IDENTITY_BYTES
    {
        return Err(Reject::Bound);
    }
    if audience != expected_audience {
        return Err(Reject::Audience);
    }
    verify(ticket_id, bearer)
}

fn credential_field<'a>(frame: &'a [u8], cursor: &mut usize) -> Result<&'a str, Reject> {
    let len = *frame.get(*cursor).ok_or(Reject::Bound)? as usize;
    let start = cursor.checked_add(1).ok_or(Reject::Bound)?;
    let end = start.checked_add(len).ok_or(Reject::Bound)?;
    let field = frame.get(start..end).ok_or(Reject::Bound)?;
    *cursor = end;
    std::str::from_utf8(field).map_err(|_| Reject::Authority)
}

/// Credentialed socket entrypoint: a verifier supplied by the host must
/// authenticate the compatibility ticket against live state before the
/// ordinary Nix 1.37 handshake. The worker stays read-only after admission.
pub fn serve_credential_connection(
    mut stream: UnixStream,
    store: &VerifiedStore,
    runtime: &tokio::runtime::Runtime,
    expected_audience: &str,
    verify: impl FnOnce(&str, &str) -> Result<(), Reject>,
) -> io::Result<()> {
    let peer = socket_peercred(&stream).map_err(|_| denied())?;
    if peer.uid != geteuid() {
        return Err(denied());
    }
    stream.set_read_timeout(Some(Duration::from_secs(core::MAX_NEGOTIATION_SECS)))?;
    stream.set_write_timeout(Some(Duration::from_secs(core::MAX_NEGOTIATION_SECS)))?;
    let mut challenge = [0_u8; CHALLENGE_BYTES];
    getrandom::fill(&mut challenge).map_err(|_| denied())?;
    authenticate_ticket_envelope(&mut stream, &challenge, expected_audience, verify)
        .map_err(|_| denied())?;
    serve_connection(stream, store, runtime)
}

/// One independently OS-authenticated private peer. Worker operations are
/// serial per connection, so the actual concurrency is at most one and never
/// exceeds the named 32-connection/64-operation admission ceilings.
pub fn serve_connection(mut stream: UnixStream, store: &VerifiedStore, runtime: &tokio::runtime::Runtime) -> io::Result<()> {
    let credentials = socket_peercred(&stream).map_err(|_| denied())?;
    if credentials.uid != geteuid() {
        return Err(denied());
    }
    stream.set_read_timeout(Some(Duration::from_secs(core::MAX_NEGOTIATION_SECS)))?;
    stream.set_write_timeout(Some(Duration::from_secs(core::MAX_NEGOTIATION_SECS)))?;
    handshake(&mut stream)?;
    stream.set_read_timeout(Some(Duration::from_secs(core::MAX_IDLE_SECS)))?;
    stream.set_write_timeout(Some(Duration::from_secs(core::MAX_IDLE_SECS)))?;
    loop {
        let request = match read_request(&mut stream) {
            Ok(request) => request,
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => {
                let reason = if error.to_string() == Reject::Operation.code() { Reject::Operation }
                    else { Reject::Bound };
                write_rejection(&mut stream, reason)?;
                return Ok(());
            }
        };
        if let Err(reason) = runtime.block_on(store.answer(request, &mut stream)) {
            write_rejection(&mut stream, reason)?;
            return Ok(());
        }
    }
}
