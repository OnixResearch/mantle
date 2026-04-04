use bstr::ByteSlice;

use crate::{
    SymlinkTargetError,
    path::{PathComponent, PathComponentError},
};

/// Errors that occur during construction of [crate::Node]
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ValidateNodeError {
    /// Invalid digest length encountered
    #[error("invalid digest length: {0}")]
    InvalidDigestLen(usize),
    /// Invalid symlink target
    #[error("Invalid symlink target: {0}")]
    InvalidSymlinkTarget(SymlinkTargetError),
    /// Invalid hash type encountered
    #[error("invalid hash type: expected a 'blake3-' prefixed digest")]
    InvalidHashType,
}

impl From<crate::digests::Error> for ValidateNodeError {
    fn from(e: crate::digests::Error) -> Self {
        match e {
            crate::digests::Error::InvalidDigestLen(n) => ValidateNodeError::InvalidDigestLen(n),
            crate::digests::Error::InvalidHashType => ValidateNodeError::InvalidHashType,
        }
    }
}

/// Errors that can occur when populating [crate::Directory] messages,
/// or parsing [crate::proto::Directory]
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DirectoryError {
    /// Multiple elements with the same name encountered
    #[error("{:?} is a duplicate name", .0)]
    DuplicateName(PathComponent),
    /// Node failed validation
    #[error("invalid node with name {}: {:?}", .0.as_bstr(), .1.to_string())]
    InvalidNode(bytes::Bytes, ValidateNodeError),
    #[error("Total size exceeds u64::MAX")]
    SizeOverflow,
    /// Invalid name encountered
    #[error("Invalid name: {0}")]
    InvalidName(PathComponentError),
    /// This can occur if a protobuf node with a name is passed where we expect
    /// it to be anonymous.
    #[error("Name is set when it shouldn't")]
    NameInAnonymousNode,
    /// Elements are not in sorted order. Can only happen on protos
    #[error("{:?} is not sorted", .0.as_bstr())]
    WrongSorting(bytes::Bytes),
    /// This can only happen if there's an unknown entry type (on protos)
    #[error("No entry set")]
    NoEntrySet,
}
