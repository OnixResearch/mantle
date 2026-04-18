#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum Error {
    #[error("collection exceeds limit {limit}: {actual}")]
    CollectionTooLarge { limit: u32, actual: u32 },

    #[error("string field must not be empty: {field}")]
    EmptyField { field: &'static str },

    #[error("duplicate node id: {node_id}")]
    DuplicateNodeId { node_id: String },

    #[error("edge endpoint missing from node set: {node_id}")]
    MissingNode { node_id: String },

    #[error("artifact subject must exist and be an artifact node: {node_id}")]
    InvalidArtifactSubject { node_id: String },

    #[error("closure node must exist and be a closure node: {node_id}")]
    InvalidClosureNode { node_id: String },

    #[error("closure roots must not be empty")]
    EmptyClosureRoots,

    #[error("selected closure root must exist and be an artifact node: {node_id}")]
    InvalidClosureRoot { node_id: String },

    #[error("closure root missing from members: {node_id}")]
    MissingClosureRoot { node_id: String },

    #[error("project node must exist and be a project node: {node_id}")]
    InvalidProjectNode { node_id: String },

    #[error("selected project root must exist and be an artifact node: {node_id}")]
    InvalidProjectRoot { node_id: String },

    #[error("canonical serialization failed: {message}")]
    Serialize { message: String },

    #[error("digest hex must be 64 lowercase characters: {value}")]
    InvalidDigestHex { value: String },

    #[error("schema tag mismatch: expected {expected}, got {actual}")]
    SchemaTagMismatch { expected: &'static str, actual: String },

    #[error("field exceeds length limit: {field} ({actual} > {limit})")]
    FieldTooLong {
        field: &'static str,
        limit: u32,
        actual: u32,
    },

    #[error("detached signature parse error: {message}")]
    InvalidDetachedSignature { message: String },
}
