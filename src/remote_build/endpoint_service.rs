//! Owner-local endpoint name resolution on the coordination socket. This is not
//! credential, content, or execution authority. Rejection returns no frame.

use std::io::BufRead as _;
use std::io::Write as _;
use std::io::{self};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crunch_remote_core::endpoint::EndpointBinding;
use crunch_remote_core::endpoint::EndpointBindingSpec;
use serde::Deserialize;
use serde::Serialize;

use super::endpoint::EndpointBindingStore;

pub const ENDPOINT_PROTOCOL_SCHEMA: &str = "mantle-endpoint-binding-v1";
pub const MAX_ENDPOINT_FRAME_BYTES: usize = 16_384;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Request {
    Create {
        schema: String,
        spec: EndpointBindingSpec,
        expected_previous_generation: Option<u64>,
    },
    Resolve {
        schema: String,
        binding: EndpointBinding,
    },
    Revoke {
        schema: String,
        binding: EndpointBinding,
    },
}

impl Request {
    fn schema(&self) -> &str {
        match self {
            Self::Create { schema, .. } | Self::Resolve { schema, .. } | Self::Revoke { schema, .. } => schema,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Reply {
    Resolved { schema: String, binding: EndpointBinding },
    Revoked { schema: String },
}

/// Called only after the socket adapter has observed the same operating-system
/// owner. Time is always receiver-observed; no client timestamp is accepted.
pub(super) fn handle_request(state_dir: &Path, request: Request) -> Option<Vec<u8>> {
    if request.schema() != ENDPOINT_PROTOCOL_SCHEMA {
        return None;
    }
    let now = crate::unix_time_now_s().ok()?;
    let store = EndpointBindingStore::open(state_dir).ok()?;
    let reply = match request {
        Request::Create {
            spec,
            expected_previous_generation,
            ..
        } => Reply::Resolved {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            binding: store.create(spec, expected_previous_generation, now).ok()?,
        },
        Request::Resolve { binding, .. } => Reply::Resolved {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            binding: store.resolve(&binding, now).ok()?,
        },
        Request::Revoke { binding, .. } => {
            store.revoke(&binding).ok()?;
            Reply::Revoked {
                schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            }
        }
    };
    let mut encoded = serde_json::to_vec(&reply).ok()?;
    if encoded.len() >= MAX_ENDPOINT_FRAME_BYTES {
        return None;
    }
    encoded.push(b'\n');
    Some(encoded)
}

#[cfg(target_os = "linux")]
pub(super) fn same_owner(stream: &UnixStream) -> bool {
    use std::os::fd::AsRawFd as _;
    let mut credential = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut credential).cast(),
            &raw mut length,
        )
    };
    result == 0 && length as usize == std::mem::size_of::<libc::ucred>() && credential.uid == unsafe { libc::geteuid() }
}

#[cfg(not(target_os = "linux"))]
pub(super) fn same_owner(_stream: &UnixStream) -> bool {
    // Native peer-identity observation must be supplied before enabling mutation.
    false
}

/// A bounded actual coordination-socket call. A denied name is EOF, not an
/// explanatory rejection response or a success-shaped fallback descriptor.
pub fn exchange(socket: &Path, request: &Request) -> io::Result<Reply> {
    let mut bytes = serde_json::to_vec(request).map_err(io::Error::other)?;
    if bytes.len() >= MAX_ENDPOINT_FRAME_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "endpoint-frame-bound"));
    }
    bytes.push(b'\n');
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(&bytes)?;
    let mut reader = io::BufReader::new(stream);
    let mut response = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "endpoint-admission-unavailable"));
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(available.len(), |index| index + 1);
        if response.len().saturating_add(count) > MAX_ENDPOINT_FRAME_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "endpoint-frame-bound"));
        }
        response.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            break;
        }
    }
    let reply: Reply = serde_json::from_slice(&response).map_err(io::Error::other)?;
    let schema = match &reply {
        Reply::Resolved { schema, .. } | Reply::Revoked { schema } => schema,
    };
    if schema != ENDPOINT_PROTOCOL_SCHEMA {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "endpoint-schema"));
    }
    Ok(reply)
}
