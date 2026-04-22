use alloc::string::String;
use core::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    CollectionTooLarge { limit: u32, actual: u32 },
    EmptyField { field: String },
    DuplicateNodeId { node_id: String },
    MissingNode { node_id: String },
    InvalidArtifactSubject { node_id: String },
    InvalidClosureNode { node_id: String },
    EmptyClosureRoots,
    InvalidClosureRoot { node_id: String },
    MissingClosureRoot { node_id: String },
    InvalidProjectNode { node_id: String },
    InvalidProjectRoot { node_id: String },
    Serialize { message: String },
    InvalidDigestHex { value: String },
    SchemaTagMismatch { expected: String, actual: String },
    FieldTooLong { field: String, limit: u32, actual: u32 },
    UnsupportedPolicyField { field: String, value: String },
    InvalidDetachedSignature { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CollectionTooLarge { limit, actual } => write!(f, "collection exceeds limit {limit}: {actual}"),
            Error::EmptyField { field } => write!(f, "string field must not be empty: {field}"),
            Error::DuplicateNodeId { node_id } => write!(f, "duplicate node id: {node_id}"),
            Error::MissingNode { node_id } => write!(f, "edge endpoint missing from node set: {node_id}"),
            Error::InvalidArtifactSubject { node_id } => {
                write!(f, "artifact subject must exist and be an artifact node: {node_id}")
            }
            Error::InvalidClosureNode { node_id } => {
                write!(f, "closure node must exist and be a closure node: {node_id}")
            }
            Error::EmptyClosureRoots => f.write_str("closure roots must not be empty"),
            Error::InvalidClosureRoot { node_id } => {
                write!(f, "selected closure root must exist and be an artifact node: {node_id}")
            }
            Error::MissingClosureRoot { node_id } => write!(f, "closure root missing from members: {node_id}"),
            Error::InvalidProjectNode { node_id } => {
                write!(f, "project node must exist and be a project node: {node_id}")
            }
            Error::InvalidProjectRoot { node_id } => {
                write!(f, "selected project root must exist and be an artifact node: {node_id}")
            }
            Error::Serialize { message } => write!(f, "canonical serialization failed: {message}"),
            Error::InvalidDigestHex { value } => write!(f, "digest hex must be 64 lowercase characters: {value}"),
            Error::SchemaTagMismatch { expected, actual } => {
                write!(f, "schema tag mismatch: expected {expected}, got {actual}")
            }
            Error::FieldTooLong { field, limit, actual } => {
                write!(f, "field exceeds length limit: {field} ({actual} > {limit})")
            }
            Error::UnsupportedPolicyField { field, value } => {
                write!(f, "unsupported policy field value: {field}={value}")
            }
            Error::InvalidDetachedSignature { message } => write!(f, "detached signature parse error: {message}"),
        }
    }
}
