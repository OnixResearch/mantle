//! Standard-library file verifier over one explicit capability root.
//!
//! The shell receives one opened root directory and the declared member
//! logical paths of a bundle. It rejects absolute escapes, parent traversal,
//! symlinks, non-regular files, oversized members, and drifted bytes. It
//! performs no registry, store, network, or ambient path search.

use std::fs;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::StoreObject;

use crate::contract::ConsumerLayer;
use crate::contract::ConsumerMemberObservation;
use crate::contract::ConsumerMemberStatus;
use crate::contract::ConsumerVerificationReport;
use crate::contract::ConsumerVerificationStatus;
use crate::facade::declared_member_objects;
use crate::facade::identity_from_bytes;
use crate::facade::verify_consumer_bundle;

/// Open flags: refuse to follow a symlink at the final path component.
/// Linux `O_NOFOLLOW`; the consumer shell is a bounded Linux surface.
const OPEN_FLAGS_NO_FOLLOW_READ_ONLY: i32 = 0o400000;

/// Read bytes beyond the declared length of one member.
const MEMBER_READ_HEADROOM_BYTES: u64 = 1;

/// Maximum bytes remeasured for one member.
pub const MAX_MEMBER_BYTES: u64 = 1_073_741_824;

/// Maximum combined bytes remeasured for one bundle.
pub const MAX_TOTAL_REMEASURED_BYTES: u64 = 4_294_967_296;

/// Blocker code when remeasured bytes drift from declared identities.
pub const BLOCKER_MEMBER_DRIFT: &str = "consumer-member-bytes-drift";

/// Blocker code when the combined remeasured size exceeds the bound.
pub const BLOCKER_TOTAL_BOUND: &str = "consumer-total-byte-bound";

/// Errors surfaced by the capability-root shell.
#[derive(Debug)]
pub enum ShellError {
    /// Root is not a directory.
    RootNotDirectory,
    /// A member path is empty, escapes the root, or traverses parents.
    InvalidMemberPath,
    /// One logical path is declared with conflicting member identities.
    ConflictingMemberIdentity,
}

/// Capability root for consumer member resolution.
pub struct ConsumerRoot {
    root: std::path::PathBuf,
}

/// One required member and its declared facts.
struct RequiredMember {
    role: String,
    logical_path: String,
    digest_blake3: String,
    size_bytes: u64,
}

/// One measured logical path.
enum Measurement {
    /// The path could not be resolved as a readable regular file.
    Missing,
    /// A readable file existed but its length differed from the declaration.
    LengthMismatch,
    /// Exactly `size_bytes` bytes were read and hashed to this identity.
    Identity(String),
}

impl ConsumerRoot {
    /// Open an existing directory as the only member-resolution root.
    pub fn open(root: &Path) -> Result<Self, ShellError> {
        let metadata = fs::metadata(root).map_err(|_| ShellError::RootNotDirectory)?;
        if !metadata.is_dir() {
            return Err(ShellError::RootNotDirectory);
        }
        debug_assert!(metadata.file_type().is_dir());
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    /// Verify a bundle structurally, then remeasure every required member.
    ///
    /// Structural rejection short-circuits before any filesystem access. A
    /// byte layer that cannot resolve required bytes reports `blocked`;
    /// drifted bytes report the drift blocker and `rejected`.
    pub fn verify_bundle(&self, bundle: MaterializationBundle) -> Result<ConsumerVerificationReport, ShellError> {
        let mut structural_outcome = verify_consumer_bundle(bundle.clone());
        if structural_outcome.status != ConsumerVerificationStatus::Verified {
            return Ok(structural_outcome);
        }
        let required = required_members(&bundle)?;
        self.run_byte_layer(&required, &mut structural_outcome);
        Ok(structural_outcome)
    }

    fn run_byte_layer(&self, required: &[RequiredMember], structural_outcome: &mut ConsumerVerificationReport) {
        let mut members = Vec::with_capacity(required.len());
        let mut missing = 0_u32;
        let mut mismatched = 0_u32;
        let mut total_bytes = 0_u64;
        // Measure each unique logical path once, then judge every member
        // that declares it against its own expected identity.
        let mut measured: Vec<(&str, Measurement)> = Vec::with_capacity(required.len());
        for member in required {
            if measured.iter().any(|(path, _)| *path == member.logical_path.as_str()) {
                continue;
            }
            let measurement = self.measure_path(&member.logical_path, member.size_bytes);
            total_bytes += member.size_bytes;
            measured.push((member.logical_path.as_str(), measurement));
        }
        for member in required {
            let shared_measurement = match measured.iter().find(|(path, _)| *path == member.logical_path.as_str()) {
                Some((_, measurement)) => measurement,
                None => &Measurement::Missing,
            };
            let status = match shared_measurement {
                Measurement::Missing => ConsumerMemberStatus::Missing,
                Measurement::LengthMismatch => ConsumerMemberStatus::Mismatched,
                Measurement::Identity(identity) => {
                    if *identity == member.digest_blake3 {
                        debug_assert!(member.size_bytes <= MAX_MEMBER_BYTES);
                        ConsumerMemberStatus::Matched
                    } else {
                        ConsumerMemberStatus::Mismatched
                    }
                }
            };
            match status {
                ConsumerMemberStatus::Matched => {}
                ConsumerMemberStatus::Missing => missing += 1,
                ConsumerMemberStatus::Mismatched => mismatched += 1,
            }
            debug_assert!(matches!(
                status,
                ConsumerMemberStatus::Matched | ConsumerMemberStatus::Missing | ConsumerMemberStatus::Mismatched
            ));
            let doubled = match required.len().checked_mul(2) {
                Some(bound) => bound,
                None => required.len(),
            };
            assert!(members.len() < doubled, "member observation bound exceeded");
            members.push(ConsumerMemberObservation {
                role: member.role.clone(),
                digest_blake3: member.digest_blake3.clone(),
                size_bytes: member.size_bytes,
                status: Some(status),
            });
        }
        structural_outcome.members = members;
        structural_outcome.layers_completed.push(ConsumerLayer::Bytes);
        structural_outcome.status = if total_bytes > MAX_TOTAL_REMEASURED_BYTES {
            structural_outcome.blockers.push(String::from(BLOCKER_TOTAL_BOUND));
            ConsumerVerificationStatus::Rejected
        } else if missing > 0 {
            ConsumerVerificationStatus::Blocked
        } else if mismatched > 0 {
            structural_outcome.blockers.push(String::from(BLOCKER_MEMBER_DRIFT));
            ConsumerVerificationStatus::Rejected
        } else {
            debug_assert!(structural_outcome.blockers.is_empty());
            debug_assert!(missing == 0 && mismatched == 0);
            ConsumerVerificationStatus::Verified
        };
    }

    fn measure_path(&self, logical_path: &str, size_bytes: u64) -> Measurement {
        let relative = match relative_member_path(logical_path) {
            Ok(relative) => relative,
            Err(_) => return Measurement::Missing,
        };
        let resolved = self.root.join(relative);
        let metadata = match fs::symlink_metadata(&resolved) {
            Ok(metadata) => metadata,
            Err(_) => return Measurement::Missing,
        };
        if !metadata.is_file() {
            return Measurement::Missing;
        }
        let opened = fs::OpenOptions::new().read(true).custom_flags(OPEN_FLAGS_NO_FOLLOW_READ_ONLY).open(&resolved);
        let mut file = match opened {
            Ok(file) => file,
            Err(_) => return Measurement::Missing,
        };
        if size_bytes > MAX_MEMBER_BYTES {
            return Measurement::LengthMismatch;
        }
        let mut bytes = Vec::new();
        let read_limit_bytes = size_bytes.saturating_add(MEMBER_READ_HEADROOM_BYTES);
        let mut handle = file.by_ref().take(read_limit_bytes);
        if handle.read_to_end(&mut bytes).is_err() {
            return Measurement::Missing;
        }
        if bytes.len() as u64 != size_bytes {
            return Measurement::LengthMismatch;
        }
        Measurement::Identity(identity_from_bytes(&bytes).into_hex())
    }
}

/// Collect every declared store object as a required byte member.
///
/// Roles may share one logical path; the byte layer measures each unique
/// path once and judges every declaring member against its own identity.
/// Collect every declared store object as a required byte member.
///
/// Roles may share one logical path when they declare the same identity;
/// a path declared with conflicting digests is member substitution and is
/// rejected before any filesystem access.
fn required_members(bundle: &MaterializationBundle) -> Result<Vec<RequiredMember>, ShellError> {
    let declared = declared_member_objects(bundle);
    let mut members = Vec::with_capacity(declared.len());
    for (role, object) in &declared {
        let StoreObject {
            logical_path,
            digest_blake3,
            size_bytes,
        } = object;
        relative_member_path(logical_path)?;
        debug_assert!(!(*role).is_empty());
        let digest_hex = digest_blake3.to_string();
        let conflicting = members
            .iter()
            .any(|member| member.logical_path == *logical_path && member.digest_blake3 != digest_hex);
        if conflicting {
            return Err(ShellError::ConflictingMemberIdentity);
        }
        members.push(RequiredMember {
            role: String::from(*role),
            logical_path: logical_path.clone(),
            digest_blake3: digest_hex,
            size_bytes: *size_bytes,
        });
    }
    debug_assert_eq!(members.len(), declared.len());
    Ok(members)
}

/// Map a member logical store path to its root-relative consumer path.
///
/// The consumer root mirrors the declared member tree: the leading `/` of
/// the logical path becomes the root-relative layout. Parent, current, and
/// empty components are rejected before any filesystem access.
fn relative_member_path(logical_path: &str) -> Result<&str, ShellError> {
    let Some(relative) = logical_path.strip_prefix('/') else {
        return Err(ShellError::InvalidMemberPath);
    };
    if relative.is_empty() {
        return Err(ShellError::InvalidMemberPath);
    }
    for component in relative.split('/') {
        if component.is_empty() || component == ".." || component == "." {
            return Err(ShellError::InvalidMemberPath);
        }
    }
    debug_assert!(!relative.starts_with('/') && !relative.ends_with('/'));
    Ok(relative)
}
