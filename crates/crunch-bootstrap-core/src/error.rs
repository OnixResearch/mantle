use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageError {
    MissingSeedField(String),
    UnsupportedSeedClass(String),
    OversizedSeed {
        actual_bytes: u32,
        budget_bytes: u32,
    },
    MissingDigest(String),
    MalformedBlake3Hex(String),
    NonBlake3WithoutReason(String),
    UndeclaredGeneratedArtifact(String),
    ForbiddenRoot(String),
    UnreachableProviderOutput(String),
    EmptyField(String),
    DuplicateNodeId(String),
    InvalidNominal {
        field: String,
        reason: String,
    },
    MissingLineageNode(String),
    WrongLineageNodeRole {
        reference: String,
        expected: String,
        actual: String,
    },
    MultipleErrors(Vec<LineageError>),
}

impl fmt::Display for LineageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeedField(field) => write!(f, "seed missing required audit-bound field: {field}"),
            Self::UnsupportedSeedClass(class) => write!(
                f,
                "unsupported seed class '{class}': new seed classes require separate OpenSpec and ADR approval"
            ),
            Self::OversizedSeed {
                actual_bytes,
                budget_bytes,
            } => write!(
                f,
                "seed byte length {actual_bytes} exceeds audit_seed_max_bytes budget {budget_bytes}: \
                 a larger seed requires separate OpenSpec and ADR approval"
            ),
            Self::MissingDigest(context) => write!(f, "missing digest: {context}"),
            Self::MalformedBlake3Hex(value) => write!(f, "malformed BLAKE3 hex digest: {value}"),
            Self::NonBlake3WithoutReason(context) => {
                write!(f, "non-BLAKE3 digest without interoperability_reason: {context}")
            }
            Self::UndeclaredGeneratedArtifact(name) => {
                write!(f, "generated artifact '{name}' not bound to a transition rule")
            }
            Self::ForbiddenRoot(root) => write!(f, "forbidden prebuilt root: {root}"),
            Self::UnreachableProviderOutput(role) => {
                write!(f, "provider output '{role}' not reachable from declared lineage")
            }
            Self::EmptyField(field) => write!(f, "required field is empty: {field}"),
            Self::DuplicateNodeId(id) => write!(f, "duplicate lineage node id: {id}"),
            Self::InvalidNominal { field, reason } => write!(f, "invalid nominal lineage value for {field}: {reason}"),
            Self::MissingLineageNode(reference) => write!(f, "lineage reference names no declared node: {reference}"),
            Self::WrongLineageNodeRole {
                reference,
                expected,
                actual,
            } => write!(f, "lineage reference {reference} has role {actual}; expected {expected}"),
            Self::MultipleErrors(errors) => {
                write!(f, "{} lineage validation errors:", errors.len())?;
                for e in errors {
                    write!(f, "\n  - {e}")?;
                }
                Ok(())
            }
        }
    }
}
