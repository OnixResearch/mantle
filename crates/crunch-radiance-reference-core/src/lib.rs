#![no_std]
//! Pure policy for the optional external Radiance bootstrap reference.

// r[impl mantle.bootstrap.radiance_reference.source_cohort]
// r[impl mantle.bootstrap.radiance_reference.build_graph]
// r[impl mantle.bootstrap.radiance_reference.lineage]
// r[impl mantle.bootstrap.radiance_reference.convergence]
// r[impl mantle.bootstrap.radiance_reference.receipt]
// r[impl mantle.bootstrap.radiance_reference.publication]
// r[impl mantle.bootstrap.radiance_reference.claim_boundary]

extern crate alloc;

#[cfg(test)]
extern crate std;

mod admission;
mod constants;
mod graph;
mod model;
mod policy;

use alloc::string::ToString;

pub use admission::*;
pub use constants::*;
pub use graph::*;
pub use model::*;
pub use policy::*;

const DOMAIN_SEPARATOR: u8 = 0;

pub(crate) fn canonical_blake3<T: serde::Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<alloc::string::String, RadianceReferenceError> {
    let bytes = serde_json::to_vec(value).map_err(|_| RadianceReferenceError::Canonicalization)?;
    let domain_bytes = u64::try_from(domain.len()).map_err(|_| RadianceReferenceError::Canonicalization)?;
    let value_bytes = u64::try_from(bytes.len()).map_err(|_| RadianceReferenceError::Canonicalization)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&domain_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(domain);
    hasher.update(&value_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(valid_blake3(&digest));
    debug_assert!(!domain.is_empty());
    Ok(digest)
}

pub(crate) fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests;
