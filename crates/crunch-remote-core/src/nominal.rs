//! Bounded, role-specific remote IDs. The same wire text is not interchangeable
//! between request, session, endpoint, and output identities.
//!
//! ```compile_fail
//! use crunch_remote_core::nominal::{EndpointId, RequestId};
//! fn needs_request(_: RequestId) {}
//! needs_request(EndpointId::new("builder-a").unwrap());
//! ```

use alloc::string::String;

pub const MAX_REMOTE_ID_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteIdError {
    Empty,
    Oversized,
    ControlCharacter,
}

impl RemoteIdError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Oversized => "oversized",
            Self::ControlCharacter => "control-character",
        }
    }
}

// r[impl remote_builds.hexagonal_core]
fn admit_remote_id(value: String) -> Result<String, RemoteIdError> {
    if value.is_empty() {
        return Err(RemoteIdError::Empty);
    }
    if value.len() > MAX_REMOTE_ID_BYTES {
        return Err(RemoteIdError::Oversized);
    }
    if value.chars().any(char::is_control) {
        return Err(RemoteIdError::ControlCharacter);
    }
    Ok(value)
}

macro_rules! nominal_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, RemoteIdError> {
                admit_remote_id(value.into()).map(Self)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

nominal_id!(RequestId);
nominal_id!(SessionId);
nominal_id!(EndpointId);
nominal_id!(OutputId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_identity_role_admits_only_bounded_control_free_text() {
        let request = RequestId::new("request-a").unwrap();
        let session = SessionId::new("session-a").unwrap();
        let endpoint = EndpointId::new("builder-a").unwrap();
        let output = OutputId::new("out").unwrap();
        assert_eq!(request.as_str(), "request-a");
        assert_eq!(session.as_str(), "session-a");
        assert_eq!(endpoint.as_str(), "builder-a");
        assert_eq!(output.as_str(), "out");
        assert_eq!(RequestId::new(""), Err(RemoteIdError::Empty));
        assert_eq!(EndpointId::new("bad\nendpoint"), Err(RemoteIdError::ControlCharacter));
        assert_eq!(SessionId::new("a".repeat(MAX_REMOTE_ID_BYTES + 1)), Err(RemoteIdError::Oversized));
    }
}
