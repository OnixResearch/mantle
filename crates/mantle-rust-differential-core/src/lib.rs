#![no_std]

//! Pure, bounded ordinary Rust differential evidence policy. No execution or
//! output-admission authority is granted by an equivalence verdict.
extern crate alloc;

mod edits;
mod manifest;
mod population;
mod recovery;
mod semantics;
pub mod waits;

use alloc::string::String;

pub use edits::*;
pub use manifest::*;
pub use mantle_rust_plan_core::Blake3Digest as Digest;
pub use population::*;
pub use recovery::*;
pub use semantics::*;
use serde::Deserialize;
use serde::Serialize;

/// Stable nonpassing policy failures, deliberately independent of human output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Error {
    Schema,
    Bound,
    Identity,
    Duplicate,
    Missing,
    Malformed,
    UnknownSemanticField,
    IncompleteCapture,
    IncompleteTeardown,
    Ice,
    Oracle,
    BaselineRewrite,
    CoverageDeficit,
    Incomparable,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for Error {}

pub(crate) fn safe_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_KEY_BYTES
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.:/".contains(&b))
}

pub(crate) fn identity<T: Serialize>(role: &[u8], value: &T) -> Result<Digest, Error> {
    mantle_rust_plan_core::domain_digest(role, value).map_err(|_| Error::Identity)
}

/// Every digest carries its domain role, rather than relying on vector position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub role: String,
    pub digest: Digest,
}
