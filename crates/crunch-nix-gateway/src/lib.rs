#![forbid(unsafe_code)]

//! Streaming Nix worker-protocol 1.37 decoder. This intentionally does not
//! call the vendored daemon handler: that handler allocates untrusted vectors
//! before checking counts and advertises an unconditional trusted connection.
//! A production caller supplies independently verified authorization and must
//! dispatch every parsed request to the existing verified Mantle services.
#[cfg(target_os = "linux")]
pub mod replay;
#[cfg(target_os = "linux")]
pub mod server;
pub mod store;

use std::io::Read;
use std::io::Write;
use std::io::{self};

use crunch_nix_gateway_core as core;

const WORKER_MAGIC_1: u64 = 0x6e697863;
const WORKER_MAGIC_2: u64 = 0x6478696f;
const STDERR_LAST: u64 = 0x616c7473;
const STDERR_ERROR: u64 = 0x63787470;
const PROTOCOL_VERSION: u64 = (core::NIX_PROTOCOL_MAJOR as u64) << 8 | core::NIX_PROTOCOL_MINOR as u64;
const MAX_SETTING_BYTES: u64 = 1024;
const MAX_OVERRIDES: u64 = 32;
const MAX_SIGNATURES: u64 = 64;
const MAX_PATH_BYTES: u64 = 255;
const MAX_NAR_HASH_BYTES: u64 = 128;
const MAX_BUILD_PATH_BYTES: u64 = 384;

fn invalid(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

/// Fixed-width reads do not allocate even when a peer announces u64::MAX.
pub fn read_number(input: &mut impl Read) -> io::Result<u64> {
    let mut bytes = [0_u8; 8];
    input.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

pub fn write_number(output: &mut impl Write, value: u64) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

pub fn read_count(input: &mut impl Read, maximum: u64) -> io::Result<u32> {
    let count = read_number(input)?;
    if count > maximum || count > u32::MAX as u64 {
        return Err(invalid("gateway-collection-bound"));
    }
    Ok(count as u32)
}

/// Wire strings use 64-bit byte length and 8-byte padding. Reject the length
/// before creating a Vec, then reject invalid padding to avoid desync.
pub fn read_bytes(input: &mut impl Read, maximum: u64) -> io::Result<Vec<u8>> {
    let length = read_number(input)?;
    if length > maximum || length > core::MAX_FRAME_BYTES {
        return Err(invalid("gateway-frame-bound"));
    }
    let aligned = length.checked_add(7).ok_or_else(|| invalid("gateway-frame-bound"))? & !7;
    let mut data = vec![0; length as usize];
    input.read_exact(&mut data)?;
    let mut padding = [0_u8; 7];
    let extra = (aligned - length) as usize;
    input.read_exact(&mut padding[..extra])?;
    if padding[..extra].iter().any(|&byte| byte != 0) {
        return Err(invalid("gateway-padding-invalid"));
    }
    Ok(data)
}

pub fn read_string(input: &mut impl Read, maximum: u64) -> io::Result<String> {
    String::from_utf8(read_bytes(input, maximum)?).map_err(|_| invalid("gateway-string-invalid"))
}

pub fn write_bytes(output: &mut impl Write, data: &[u8]) -> io::Result<()> {
    write_number(output, data.len() as u64)?;
    output.write_all(data)?;
    let padding = [0_u8; 7];
    output.write_all(&padding[..(8 - data.len() % 8) % 8])
}

/// Nix 1.37 structured error envelope, built entirely from a closed reason
/// enum. Never serialize an I/O error, bearer, raw request or provider text.
// r[impl remote_builds.gateway_non_claims]
pub fn write_rejection(output: &mut impl Write, reason: core::Reject) -> io::Result<()> {
    write_number(output, STDERR_ERROR)?;
    write_bytes(output, b"Error")?;
    write_number(output, 0)?;
    write_bytes(output, b"Error")?;
    write_bytes(output, reason.code().as_bytes())?;
    write_number(output, 0)?;
    write_number(output, 0)?;
    output.flush()
}

/// Settings overrides are bounded and consumed but never applied to the
/// gateway's policy or host services. Nix clients commonly send defaults.
fn read_options(input: &mut impl Read) -> io::Result<()> {
    for _ in 0..12 {
        read_number(input)?;
    }
    let override_count = read_count(input, MAX_OVERRIDES)?;
    for _ in 0..override_count {
        read_string(input, MAX_SETTING_BYTES)?;
        read_string(input, MAX_SETTING_BYTES)?;
    }
    Ok(())
}

/// The reported trust status is **not trusted** (Nix status 2); authenticating
/// a peer remains the caller's responsibility. Handshake is deadline-limited
/// by the caller and accepts only negotiated version 1.37.
// r[impl remote_builds.nix_compatibility_gateway]
pub fn handshake(stream: &mut (impl Read + Write)) -> io::Result<()> {
    if read_number(stream)? != WORKER_MAGIC_1 {
        return Err(invalid("gateway-magic-invalid"));
    }
    write_number(stream, WORKER_MAGIC_2)?;
    write_number(stream, PROTOCOL_VERSION)?;
    stream.flush()?;
    let client_version = read_number(stream)?;
    if client_version >> 8 != core::NIX_PROTOCOL_MAJOR as u64
        || (client_version & 0xff) < core::NIX_PROTOCOL_MINOR as u64
    {
        return Err(invalid("gateway-version-unsupported"));
    }
    if read_number(stream)? != 0 || read_number(stream)? != 0 {
        return Err(invalid("gateway-legacy-setting-denied"));
    }
    write_bytes(stream, b"2.20.0")?;
    write_number(stream, 2)?;
    write_number(stream, STDERR_LAST)?;
    stream.flush()?;
    if read_number(stream)? != 19 {
        return Err(invalid("gateway-set-options-required"));
    }
    read_options(stream)?;
    write_number(stream, STDERR_LAST)?;
    stream.flush()
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedRequest {
    IsValidPath {
        path: String,
    },
    QueryPathInfo {
        path: String,
    },
    QueryValidPaths {
        paths: Vec<String>,
        substitute: bool,
    },
    BuildPaths {
        paths: Vec<String>,
    },
    AddToStoreNar {
        path: String,
        deriver: String,
        nar_hash: String,
        references: Vec<String>,
        nar_size: u64,
        registration_time: u64,
        signatures: Vec<String>,
        content_address: String,
    },
    QueryMissing {
        targets: Vec<String>,
    },
}

fn read_path(input: &mut impl Read) -> io::Result<String> {
    let path = read_string(input, MAX_PATH_BYTES)?;
    if !core::valid_store_path(&path, false) {
        return Err(invalid("gateway-store-path-invalid"));
    }
    Ok(path)
}

fn read_paths(input: &mut impl Read) -> io::Result<Vec<String>> {
    let count = read_count(input, core::MAX_STORE_PATHS as u64)?;
    let mut paths = Vec::with_capacity(count as usize);
    for _ in 0..count {
        paths.push(read_path(input)?);
    }
    Ok(paths)
}

fn valid_output_spec(spec: &str) -> bool {
    if spec == "*" {
        return true;
    }
    spec.split(',').all(|name| {
        let mut bytes = name.bytes();
        matches!(bytes.next(), Some(b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'+' | b'-' | b'_' | b'?' | b'='))
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'_' | b'?' | b'=' | b'.'))
    })
}

fn read_missing_targets(input: &mut impl Read) -> io::Result<Vec<String>> {
    let count = read_count(input, core::MAX_STORE_PATHS as u64)?;
    let mut targets = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let target = read_string(input, MAX_BUILD_PATH_BYTES)?;
        let valid = if let Some((drv, outputs)) = target.split_once('!') {
            core::valid_store_path(drv, true) && valid_output_spec(outputs)
        } else {
            core::valid_store_path(&target, false)
        };
        if !valid {
            return Err(invalid("gateway-query-target-invalid"));
        }
        targets.push(target);
    }
    Ok(targets)
}

/// Each supported operation consumes only its declared body. Unknown opcodes
/// are rejected without allocation and the caller closes the connection.
// r[impl remote_builds.gateway_bounds_and_recovery]
pub fn read_request(input: &mut impl Read) -> io::Result<ParsedRequest> {
    let code = read_number(input)?;
    let operation = core::nix_operation(1, 37, code).map_err(|error| invalid(error.code()))?;
    match operation {
        core::NixOperation::IsValidPath => Ok(ParsedRequest::IsValidPath {
            path: read_path(input)?,
        }),
        core::NixOperation::QueryPathInfo => Ok(ParsedRequest::QueryPathInfo {
            path: read_path(input)?,
        }),
        core::NixOperation::QueryValidPaths => {
            let paths = read_paths(input)?;
            let substitute = read_number(input)?;
            if substitute != 0 {
                return Err(invalid("gateway-substitution-unsupported"));
            }
            Ok(ParsedRequest::QueryValidPaths {
                paths,
                substitute: false,
            })
        }
        core::NixOperation::BuildPaths => {
            let count = read_count(input, core::MAX_STORE_PATHS as u64)?;
            if count == 0 {
                return Err(invalid("gateway-build-empty"));
            }
            let mut paths = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let path = read_string(input, MAX_BUILD_PATH_BYTES)?;
                let (drv, outputs) = path.split_once('!').ok_or_else(|| invalid("gateway-build-not-concrete"))?;
                if !core::valid_store_path(drv, true)
                    || !valid_output_spec(outputs)
                {
                    return Err(invalid("gateway-build-not-concrete"));
                }
                paths.push(path);
            }
            if read_number(input)? != 0 {
                return Err(invalid("gateway-build-mode-unsupported"));
            }
            Ok(ParsedRequest::BuildPaths { paths })
        }
        core::NixOperation::AddToStoreNar => {
            let path = read_path(input)?;
            let deriver = read_string(input, MAX_PATH_BYTES)?;
            if !deriver.is_empty() && !core::valid_store_path(&deriver, true) {
                return Err(invalid("gateway-deriver-invalid"));
            }
            let nar_hash = read_string(input, MAX_NAR_HASH_BYTES)?;
            if nar_hash.len() != 64
                || !nar_hash.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err(invalid("gateway-nar-hash-invalid"));
            }
            let references = read_paths(input)?;
            let registration_time = read_number(input)?;
            let nar_size = read_number(input)?;
            if nar_size > core::MAX_TRANSFER_BYTES {
                return Err(invalid("gateway-transfer-bound"));
            }
            if read_number(input)? != 0 {
                return Err(invalid("gateway-ultimate-denied"));
            }
            let count = read_count(input, MAX_SIGNATURES)?;
            let mut signatures = Vec::with_capacity(count as usize);
            for _ in 0..count {
                signatures.push(read_string(input, MAX_SETTING_BYTES)?);
            }
            let content_address = read_string(input, MAX_NAR_HASH_BYTES)?;
            if read_number(input)? != 0 || read_number(input)? != 0 {
                return Err(invalid("gateway-import-flags-denied"));
            }
            Ok(ParsedRequest::AddToStoreNar {
                path,
                deriver,
                nar_hash,
                references,
                nar_size,
                registration_time,
                signatures,
                content_address,
            })
        }
        core::NixOperation::QueryMissing => Ok(ParsedRequest::QueryMissing {
            targets: read_missing_targets(input)?,
        }),
        core::NixOperation::SetOptions => Err(invalid("gateway-set-options-repeat-denied")),
    }
}

/// Nix 1.37 framed upload: a zero-length frame terminates the stream. A
/// truncated frame never reaches `finish`; the caller must discard staging.
/// The sink receives only bounded chunks; it must stage and verify before
/// publishing CAS/PathInfo, and must abort on any error or client disconnect.
// r[impl remote_builds.gateway_store_integrity]
pub fn read_transfer(
    input: &mut impl Read,
    declared_bytes: u64,
    mut chunk: impl FnMut(&[u8]) -> io::Result<()>,
    mut finish: impl FnMut(u64) -> io::Result<()>,
) -> io::Result<()> {
    if declared_bytes > core::MAX_TRANSFER_BYTES {
        return Err(invalid("gateway-transfer-bound"));
    }
    let mut total = 0_u64;
    let mut buffer = [0_u8; 16_384];
    loop {
        let frame = read_number(input)?;
        if frame == 0 {
            if total != declared_bytes {
                return Err(invalid("gateway-transfer-incomplete"));
            }
            return finish(total);
        }
        if frame > core::MAX_FRAME_BYTES || total.checked_add(frame).is_none_or(|value| value > declared_bytes) {
            return Err(invalid("gateway-transfer-bound"));
        }
        let mut remaining = frame;
        while remaining != 0 {
            let size = remaining.min(buffer.len() as u64) as usize;
            input.read_exact(&mut buffer[..size])?;
            chunk(&buffer[..size])?;
            remaining -= size as u64;
        }
        total += frame;
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const PATH: &str = "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-hello.drv";

    fn request(code: u64, body: impl FnOnce(&mut Vec<u8>)) -> Cursor<Vec<u8>> {
        let mut wire = Vec::new();
        write_number(&mut wire, code).unwrap();
        body(&mut wire);
        Cursor::new(wire)
    }

    #[test]
    fn bounded_counts_and_bad_padding_reject_before_allocating() {
        let mut huge = Cursor::new(u64::MAX.to_le_bytes());
        assert_eq!(read_count(&mut huge, core::MAX_STORE_PATHS as u64).unwrap_err().kind(), io::ErrorKind::InvalidData);
        let mut huge = Cursor::new(u64::MAX.to_le_bytes());
        assert_eq!(read_bytes(&mut huge, core::MAX_FRAME_BYTES).unwrap_err().kind(), io::ErrorKind::InvalidData);
        let mut bad = request(31, |wire| write_number(wire, u64::MAX).unwrap());
        assert!(read_request(&mut bad).is_err());
        let mut bad = request(9, |wire| write_number(wire, u64::MAX).unwrap());
        assert!(read_request(&mut bad).is_err());
        let mut bad = request(40, |wire| write_number(wire, u64::MAX).unwrap());
        assert_eq!(read_request(&mut bad).unwrap_err().to_string(), "gateway-collection-bound");
        let mut bad = request(40, |wire| {
            write_number(wire, 1).unwrap();
            write_number(wire, u64::MAX).unwrap();
        });
        assert_eq!(read_request(&mut bad).unwrap_err().to_string(), "gateway-frame-bound");
        let mut bad = request(1, |wire| {
            write_number(wire, 1).unwrap();
            wire.extend_from_slice(b"x");
            wire.extend_from_slice(&[1; 7]);
        });
        assert!(read_request(&mut bad).is_err());
    }

    #[test]
    fn concrete_requests_and_denied_operations() {
        let mut valid = request(1, |wire| write_bytes(wire, PATH.as_bytes()).unwrap());
        assert_eq!(read_request(&mut valid).unwrap(), ParsedRequest::IsValidPath { path: PATH.into() });
        let mut valid = request(31, |wire| {
            write_number(wire, 1).unwrap();
            write_bytes(wire, PATH.as_bytes()).unwrap();
            write_number(wire, 0).unwrap();
        });
        assert_eq!(read_request(&mut valid).unwrap(), ParsedRequest::QueryValidPaths {
            paths: vec![PATH.into()],
            substitute: false
        });
        let mut build = request(9, |wire| {
            write_number(wire, 1).unwrap();
            write_bytes(wire, format!("{PATH}!out").as_bytes()).unwrap();
            write_number(wire, 0).unwrap();
        });
        assert_eq!(read_request(&mut build).unwrap(), ParsedRequest::BuildPaths {
            paths: vec![format!("{PATH}!out")]
        });
        let mut repair = request(9, |wire| {
            write_number(wire, 1).unwrap();
            write_bytes(wire, format!("{PATH}!out").as_bytes()).unwrap();
            write_number(wire, 1).unwrap();
        });
        assert!(read_request(&mut repair).is_err());
        for code in [7_u64, 20, 36, 43, 46] {
            assert_eq!(read_request(&mut request(code, |_| {})).unwrap_err().kind(), io::ErrorKind::InvalidData);
        }
        let mut query = request(40, |wire| {
            write_number(wire, 1).unwrap();
            write_bytes(wire, PATH.as_bytes()).unwrap();
        });
        assert_eq!(read_request(&mut query).unwrap(), ParsedRequest::QueryMissing { targets: vec![PATH.into()] });
        for spec in ["*,out", "out,,dev", ".hidden", "out!dev"] {
            let mut invalid = request(40, |wire| {
                write_number(wire, 1).unwrap();
                write_bytes(wire, format!("{PATH}!{spec}").as_bytes()).unwrap();
            });
            assert_eq!(read_request(&mut invalid).unwrap_err().to_string(), "gateway-query-target-invalid");
        }
        let mut selected = request(40, |wire| {
            write_number(wire, 1).unwrap();
            write_bytes(wire, format!("{PATH}!out,dev").as_bytes()).unwrap();
        });
        assert_eq!(read_request(&mut selected).unwrap(), ParsedRequest::QueryMissing {
            targets: vec![format!("{PATH}!out,dev")],
        });
    }

    #[test]
    fn import_header_requires_bounded_truthful_metadata() {
        let header = |wire: &mut Vec<u8>, ultimate: u64, sig_count: u64| {
            write_bytes(wire, PATH.as_bytes()).unwrap();
            write_bytes(wire, PATH.as_bytes()).unwrap();
            write_bytes(wire, "a".repeat(64).as_bytes()).unwrap();
            write_number(wire, 1).unwrap();
            write_bytes(wire, PATH.as_bytes()).unwrap();
            write_number(wire, 20).unwrap();
            write_number(wire, 4).unwrap();
            write_number(wire, ultimate).unwrap();
            write_number(wire, sig_count).unwrap();
            if sig_count == 0 {
                write_bytes(wire, b"").unwrap();
                write_number(wire, 0).unwrap();
                write_number(wire, 0).unwrap();
            }
        };
        let mut valid = request(39, |wire| header(wire, 0, 0));
        assert_eq!(read_request(&mut valid).unwrap(), ParsedRequest::AddToStoreNar {
            path: PATH.into(),
            deriver: PATH.into(),
            nar_hash: "a".repeat(64),
            references: vec![PATH.into()],
            nar_size: 4,
            registration_time: 20,
            signatures: Vec::new(),
            content_address: String::new(),
        });
        let mut forged = request(39, |wire| header(wire, 1, 0));
        assert_eq!(read_request(&mut forged).unwrap_err().to_string(), "gateway-ultimate-denied");
        let mut forged = request(39, |wire| header(wire, 0, u64::MAX));
        assert_eq!(read_request(&mut forged).unwrap_err().to_string(), "gateway-collection-bound");
    }

    #[cfg(unix)]
    #[test]
    fn unsupported_version_and_partial_handshake_fail_before_options() {
        use std::os::unix::net::UnixStream;
        use std::time::Duration;
        for version in [0x123_u64, 0x235, 0x237] {
            let (mut client, mut server) = UnixStream::pair().unwrap();
            server.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let worker = std::thread::spawn(move || handshake(&mut server));
            write_number(&mut client, WORKER_MAGIC_1).unwrap();
            assert_eq!(read_number(&mut client).unwrap(), WORKER_MAGIC_2);
            assert_eq!(read_number(&mut client).unwrap(), PROTOCOL_VERSION);
            write_number(&mut client, version).unwrap();
            assert_eq!(worker.join().unwrap().unwrap_err().kind(), io::ErrorKind::InvalidData);
        }
        let (mut client, mut server) = UnixStream::pair().unwrap();
        let worker = std::thread::spawn(move || handshake(&mut server));
        client.write_all(&WORKER_MAGIC_1.to_le_bytes()[..3]).unwrap();
        drop(client);
        assert_eq!(worker.join().unwrap().unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn incomplete_transfer_never_commits_and_exact_transfer_does() {
        let mut wire = Vec::new();
        write_number(&mut wire, 4).unwrap();
        wire.extend_from_slice(b"data");
        let mut observed = Vec::new();
        let mut commits = 0;
        assert!(
            read_transfer(
                &mut Cursor::new(&wire),
                4,
                |chunk| {
                    observed.extend_from_slice(chunk);
                    Ok(())
                },
                |_| {
                    commits += 1;
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(observed, b"data");
        assert_eq!(commits, 0);
        write_number(&mut wire, 0).unwrap();
        read_transfer(
            &mut Cursor::new(&wire),
            4,
            |_| Ok(()),
            |_| {
                commits += 1;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(commits, 1);
        let mut forged = Cursor::new(u64::MAX.to_le_bytes());
        assert!(read_transfer(&mut forged, 4, |_| Ok(()), |_| panic!("oversized transfer committed")).is_err());
    }
    #[cfg(unix)]
    #[test]
    #[ignore = "requires an installed Nix client and a private writable socket directory"]
    fn installed_nix_client_sends_bounded_query_missing() {
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::net::UnixListener;
        use std::process::Command;
        use std::process::Stdio;
        use std::time::Duration;
        use std::time::Instant;

        let socket = std::env::temp_dir().join(format!("mantle-gateway-{}-query.sock", std::process::id()));
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let worker = std::thread::spawn(move || -> io::Result<ParsedRequest> {
            let (mut stream, _) = listener.accept()?;
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            handshake(&mut stream)?;
            // The handler deliberately does not answer without an actual
            // verified Mantle store lookup for these targets.
            read_request(&mut stream)
        });
        let mut child = Command::new("nix")
            .args(["path-info", "--store"])
            .arg(format!("unix://{}", socket.display()))
            .arg(PATH)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if child.try_wait().unwrap().is_none() {
            child.kill().unwrap();
        }
        let output = child.wait_with_output().unwrap();
        let request = worker.join().unwrap();
        std::fs::remove_file(&socket).unwrap();
        assert_eq!(request.unwrap(), ParsedRequest::QueryMissing { targets: vec![PATH.into()] },
            "client stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(!output.status.success(), "unverified store lookup must not succeed");
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires an installed Nix client and a private writable socket directory"]
    fn installed_nix_store_realise_queries_missing_before_build() {
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::net::UnixListener;
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};

        let socket = std::env::temp_dir().join(format!("mantle-gateway-{}-build.sock", std::process::id()));
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let worker = std::thread::spawn(move || -> io::Result<ParsedRequest> {
            let (mut stream, _) = listener.accept()?;
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            handshake(&mut stream)?;
            read_request(&mut stream)
        });
        let mut child = Command::new("nix-store").args(["--realise", "--store"])
            .arg(format!("unix://{}", socket.display())).arg(PATH)
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if child.try_wait().unwrap().is_none() { child.kill().unwrap(); }
        let output = child.wait_with_output().unwrap();
        let request = worker.join().unwrap();
        std::fs::remove_file(&socket).unwrap();
        assert_eq!(request.unwrap(), ParsedRequest::QueryMissing { targets: vec![format!("{PATH}!*")] },
            "client stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(!output.status.success(), "unverified build prerequisites must not succeed");
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires installed Nix and a private socket; no real store is modified"]
    fn installed_nix_client_receives_redacted_admin_denial() {
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::net::UnixListener;
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};

        let socket = std::env::temp_dir().join(format!("mantle-gateway-{}-deny.sock", std::process::id()));
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let server = std::thread::spawn(move || -> io::Result<u64> {
            let (mut stream, _) = listener.accept()?;
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            handshake(&mut stream)?;
            let code = read_number(&mut stream)?;
            if core::nix_operation(1, 37, code) != Err(core::Reject::Operation) {
                return Err(invalid("gateway-admin-operation-not-rejected"));
            }
            write_rejection(&mut stream, core::Reject::Operation)?;
            Ok(code)
        });
        let mut client = Command::new("nix").args(["store", "gc", "--store"])
            .arg(format!("unix://{}", socket.display()))
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while client.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if client.try_wait().unwrap().is_none() { client.kill().unwrap(); }
        let output = client.wait_with_output().unwrap();
        let opcode = server.join().unwrap();
        std::fs::remove_file(&socket).unwrap();
        assert_eq!(opcode.unwrap(), 20);
        assert!(!output.status.success(), "unauthorized GC must not succeed");
        assert!(String::from_utf8_lossy(&output.stderr).contains("gateway-operation-unsupported"),
            "client stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires an installed Nix client and a private writable socket directory"]
    fn installed_nix_client_negotiates_private_socket() {
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::net::UnixListener;
        use std::process::Command;
        use std::process::Stdio;
        use std::time::Duration;
        use std::time::Instant;

        let socket = std::env::temp_dir().join(format!("mantle-gateway-{}-{}.sock", std::process::id(), 1));
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            handshake(&mut stream)
        });
        let mut child = Command::new("nix")
            .args(["store", "info", "--json", "--store"])
            .arg(format!("unix://{}", socket.display()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if child.try_wait().unwrap().is_none() {
            child.kill().unwrap();
        }
        let output = child.wait_with_output().unwrap();
        let handshake_result = server.join().unwrap();
        std::fs::remove_file(&socket).unwrap();
        assert!(
            handshake_result.is_ok(),
            "handshake: {handshake_result:?}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "client stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("\"trusted\":false"),
            "client stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
