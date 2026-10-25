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
        Reply::Resolved { schema, binding } => {
            binding.validate().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "endpoint-descriptor"))?;
            schema
        }
        Reply::Revoked { schema } => schema,
    };
    if schema != ENDPOINT_PROTOCOL_SCHEMA {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "endpoint-schema"));
    }
    let matches_request = match (request, &reply) {
        (Request::Resolve { binding: expected, .. }, Reply::Resolved { binding, .. }) => binding == expected,
        (
            Request::Create {
                spec,
                expected_previous_generation,
                ..
            },
            Reply::Resolved { binding, .. },
        ) => {
            let expected_generation =
                expected_previous_generation.map_or(Some(1), |generation| generation.checked_add(1));
            binding.spec == *spec && Some(binding.generation) == expected_generation
        }
        (Request::Revoke { .. }, Reply::Revoked { .. }) => true,
        _ => false,
    };
    if !matches_request {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "endpoint-reply-mismatch"));
    }
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixListener;

    use crunch_remote_core::endpoint::EndpointKind;

    use super::*;

    fn binding() -> EndpointBinding {
        EndpointBinding {
            spec: EndpointBindingSpec {
                kind: EndpointKind::BuilderTicket,
                name: "builder-a".into(),
                scope: "project-a".into(),
                trust_requirement: "full-reviewed-key".into(),
                store_prefix: "/mantle/store".into(),
                expires_unix_s: u64::MAX,
            },
            generation: 1,
        }
    }

    fn receive(request: &Request, reply: Reply) -> io::Result<Reply> {
        let root = tempfile::tempdir().unwrap();
        let socket = root.path().join("endpoint.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut frame = Vec::new();
            io::BufReader::new(&mut stream).read_until(b'\n', &mut frame).unwrap();
            let mut encoded = serde_json::to_vec(&reply).unwrap();
            encoded.push(b'\n');
            stream.write_all(&encoded).unwrap();
        });
        let result = exchange(&socket, request);
        server.join().unwrap();
        result
    }

    #[test]
    fn socket_resolution_rejects_substituted_descriptor_and_operation() {
        let original = binding();
        let request = Request::Resolve {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            binding: original.clone(),
        };
        for mutation in 0..7 {
            let mut candidate = original.clone();
            match mutation {
                0 => candidate.generation += 1,
                1 => candidate.spec.name = "builder-b".into(),
                2 => candidate.spec.scope = "project-b".into(),
                3 => candidate.spec.trust_requirement = "other-reviewed-key".into(),
                4 => candidate.spec.store_prefix = "/other/store".into(),
                5 => candidate.spec.kind = EndpointKind::Transfer,
                _ => candidate.generation = 0,
            }
            assert_eq!(
                receive(&request, Reply::Resolved {
                    schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
                    binding: candidate,
                })
                .unwrap_err()
                .kind(),
                io::ErrorKind::InvalidData
            );
        }
        assert_eq!(
            receive(&request, Reply::Revoked {
                schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidData
        );
        let Reply::Resolved { binding: resolved, .. } = receive(&request, Reply::Resolved {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            binding: original.clone(),
        })
        .unwrap() else {
            panic!("expected exact descriptor")
        };
        assert_eq!(resolved, original);
    }

    #[test]
    fn socket_creation_requires_exact_successor_and_specification() {
        let original = binding();
        let request = Request::Create {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            spec: original.spec.clone(),
            expected_previous_generation: Some(original.generation),
        };
        for generation in [0, 1, 3, u64::MAX] {
            let mut candidate = original.clone();
            candidate.generation = generation;
            assert_eq!(
                receive(&request, Reply::Resolved {
                    schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
                    binding: candidate,
                })
                .unwrap_err()
                .kind(),
                io::ErrorKind::InvalidData
            );
        }
        let mut successor = original;
        successor.generation += 1;
        let mut substituted = successor.clone();
        substituted.spec.scope = "another-project".into();
        assert_eq!(
            receive(&request, Reply::Resolved {
                schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
                binding: substituted,
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            receive(&request, Reply::Revoked {
                schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidData
        );
        let Reply::Resolved { binding: created, .. } = receive(&request, Reply::Resolved {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            binding: successor.clone(),
        })
        .unwrap() else {
            panic!("expected exact successor")
        };
        assert_eq!(created, successor);
        let revoke = Request::Revoke {
            schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
            binding: successor.clone(),
        };
        assert_eq!(
            receive(&revoke, Reply::Resolved {
                schema: ENDPOINT_PROTOCOL_SCHEMA.into(),
                binding: successor,
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
