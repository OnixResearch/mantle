//! Checked remote protocol identity roles.
//!
//! ```compile_fail
//! use mantle::remote_nominal::{RemoteEndpointId, RemoteRequestId};
//! fn requires_request(_: RemoteRequestId) {}
//! let endpoint = RemoteEndpointId::new("builder-a").unwrap();
//! requires_request(endpoint);
//! ```

// r[impl build_correctness.nominal_boundaries.identities]
// r[impl build_correctness.nominal_boundaries.units_and_paths]

const MAX_REMOTE_PROTOCOL_ID_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteNominalError {
    Empty,
    Oversized,
    ControlCharacter,
}

impl RemoteNominalError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Oversized => "oversized",
            Self::ControlCharacter => "control-character",
        }
    }
}

fn admit_remote_id(value: String) -> Result<String, RemoteNominalError> {
    if value.is_empty() {
        return Err(RemoteNominalError::Empty);
    }
    if value.len() > MAX_REMOTE_PROTOCOL_ID_BYTES {
        return Err(RemoteNominalError::Oversized);
    }
    if value.chars().any(char::is_control) {
        return Err(RemoteNominalError::ControlCharacter);
    }
    Ok(value)
}

macro_rules! remote_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, RemoteNominalError> {
                admit_remote_id(value.into()).map(Self)
            }
        }
    };
}

remote_id!(RemoteRequestId);
remote_id!(RemoteProtocolSessionId);
remote_id!(RemoteEndpointId);
remote_id!(RemoteOutputId);

impl RemoteEndpointId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RemoteOutputId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ID: &str = "remote-role-1";

    #[test]
    fn remote_roles_admit_valid_values_and_reject_malformed_values() {
        let request = RemoteRequestId::new(VALID_ID).unwrap();
        let session = RemoteProtocolSessionId::new(VALID_ID).unwrap();
        let endpoint = RemoteEndpointId::new(VALID_ID).unwrap();
        let oversized = "a".repeat(MAX_REMOTE_PROTOCOL_ID_BYTES.saturating_add(1));

        assert_eq!(request, RemoteRequestId::new(VALID_ID).unwrap());
        assert_eq!(session, RemoteProtocolSessionId::new(VALID_ID).unwrap());
        assert_eq!(endpoint.as_str(), VALID_ID);
        assert_eq!(RemoteRequestId::new(""), Err(RemoteNominalError::Empty));
        assert_eq!(RemoteEndpointId::new("bad\nendpoint"), Err(RemoteNominalError::ControlCharacter));
        assert_eq!(RemoteProtocolSessionId::new(oversized), Err(RemoteNominalError::Oversized));
    }

    /// Remote protocol roles are not interchangeable.
    ///
    /// ```compile_fail
    /// use mantle::remote_nominal::{RemoteEndpointId, RemoteRequestId};
    /// fn requires_request(_: RemoteRequestId) {}
    /// let endpoint = RemoteEndpointId::new("builder-a").unwrap();
    /// requires_request(endpoint);
    /// ```
    #[allow(dead_code)]
    fn role_separation_compile_fixture() {}
}
