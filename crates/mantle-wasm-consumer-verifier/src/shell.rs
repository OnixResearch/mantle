//! Standard-library verifier over one opened directory capability.
//!
//! Only source-closure may be a bounded directory tree. Descendant resolution
//! never follows symlinks; no registry, store, network, or ambient search runs.

use std::path::Path;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use cap_std::fs::Dir;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::StoreObject;

use crate::contract::ConsumerLayer;
use crate::contract::ConsumerMemberObservation;
use crate::contract::ConsumerMemberStatus;
use crate::contract::ConsumerVerificationReport;
use crate::contract::ConsumerVerificationStatus;
use crate::facade::declared_member_objects;
use crate::facade::verify_consumer_bundle;
use crate::source_tree;

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
    /// Required bytes could not be opened through the capability.
    MemberUnavailable,
    /// A member was a symlink or unsupported filesystem node.
    UnsupportedMember,
    /// Observed identity, length, or source facts changed.
    MemberMismatch,
    /// A named count, depth, or byte bound was exceeded.
    BoundExceeded,
    /// Destination creation failed, including an existing target.
    DestinationUnavailable,
    /// Shared bounded-tree observation, planning, or copying failed.
    Tree(bounded_tree_cap::ShellError),
}

/// Capability root for consumer member resolution.
pub struct ConsumerRoot {
    root: Dir,
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
    /// Open the caller-supplied root once, including `/proc/self/fd` bridges.
    /// Retargeting its ambient path later does not retarget this capability.
    // r[impl mantle.wasm_source_closure_trees.capabilities]
    pub fn open(root: &Path) -> Result<Self, ShellError> {
        let root =
            Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|_| ShellError::RootNotDirectory)?;
        Ok(Self { root })
    }

    /// Verify a bundle structurally, then remeasure every required member.
    ///
    /// Structural rejection short-circuits before any filesystem access. A
    /// byte layer that cannot resolve required bytes reports `blocked`;
    /// drifted bytes report the drift blocker and `rejected`.
    // r[impl mantle.wasm_source_closure_trees.pipeline]
    pub fn verify_bundle(&self, bundle: MaterializationBundle) -> Result<ConsumerVerificationReport, ShellError> {
        let mut structural_outcome = verify_consumer_bundle(bundle.clone());
        if structural_outcome.status != ConsumerVerificationStatus::Verified {
            return Ok(structural_outcome);
        }
        let required = required_members(&bundle)?;
        self.run_byte_layer(&required, &mut structural_outcome);
        Ok(structural_outcome)
    }

    /// Copy only the declared source closure into a new matching logical path.
    /// The caller owns private destination cleanup on failure. No durability,
    /// publication authority, or atomic source snapshot is claimed.
    /// This observation is not whole-bundle admission; use `verify_bundle`
    /// separately for structural and complete member-report agreement.
    // r[impl mantle.wasm_source_closure_trees.copy]
    pub fn copy_source_closure(
        &self,
        bundle: &MaterializationBundle,
        destination: &ConsumerRoot,
    ) -> Result<ConsumerMemberObservation, ShellError> {
        let required = required_members(bundle)?;
        let object = &bundle.source_closure;
        let relative = Path::new(relative_member_path(&object.logical_path)?);
        let (parent, name) = source_tree::open_parent(&self.root, relative, false)?;
        let metadata = parent.symlink_metadata(&name).map_err(|_| ShellError::MemberUnavailable)?;
        if object.size_bytes > MAX_MEMBER_BYTES {
            return Err(ShellError::BoundExceeded);
        }
        if metadata.is_dir() {
            if required
                .iter()
                .any(|member| member.logical_path == object.logical_path && member.role != "source-closure")
            {
                return Err(ShellError::UnsupportedMember);
            }
            let source =
                cap_fs_ext::DirExt::open_dir_nofollow(&parent, &name).map_err(|_| ShellError::MemberUnavailable)?;
            if object.size_bytes > source_tree::MAX_TREE_BYTES {
                return Err(ShellError::BoundExceeded);
            }
            let prepared = source_tree::prepare(&source)?;
            ensure_identity(source_tree::measure(&source, &prepared)?, object)?;
            let (target_parent, target_name) = source_tree::open_parent(&destination.root, relative, true)?;
            target_parent.create_dir(&target_name).map_err(|_| ShellError::DestinationUnavailable)?;
            let target = cap_fs_ext::DirExt::open_dir_nofollow(&target_parent, &target_name)
                .map_err(|_| ShellError::DestinationUnavailable)?;
            bounded_tree_cap::execute(&prepared, &target).map_err(ShellError::Tree)?;
            if source_tree::prepare(&source)?.plan() != prepared.plan() {
                return Err(ShellError::MemberMismatch);
            }
            let copied = source_tree::prepare(&target)?;
            ensure_identity(source_tree::measure(&target, &copied)?, object)?;
            source_tree::check_directory_binding(&parent, Path::new(&name), &source)?;
        } else {
            let mut source = source_tree::open_regular(&parent, Path::new(&name), object.size_bytes)?;
            let (target_parent, target_name) = source_tree::open_parent(&destination.root, relative, true)?;
            let mut target = source_tree::create_file(&target_parent, Path::new(&target_name))?;
            let digest = source_tree::stream_file(&mut source, Some(&mut target), object.size_bytes, None)?;
            ensure_identity((digest.to_hex().to_string(), object.size_bytes), object)?;
            ensure_identity(self.measure_member(&object.logical_path, object.size_bytes, false)?, object)?;
        }
        Ok(ConsumerMemberObservation {
            role: String::from("source-closure"),
            digest_blake3: object.digest_blake3.to_string(),
            size_bytes: object.size_bytes,
            status: Some(ConsumerMemberStatus::Matched),
        })
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
            let is_tree_allowed = required
                .iter()
                .filter(|candidate| candidate.logical_path == member.logical_path)
                .all(|candidate| candidate.role == "source-closure");
            let next_total = total_bytes.saturating_add(member.size_bytes);
            let measurement = if next_total > MAX_TOTAL_REMEASURED_BYTES {
                Measurement::LengthMismatch
            } else {
                self.measure_path(&member.logical_path, member.size_bytes, is_tree_allowed)
            };
            total_bytes = next_total;
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

    fn measure_path(&self, logical_path: &str, size_bytes: u64, is_tree_allowed: bool) -> Measurement {
        match self.measure_member(logical_path, size_bytes, is_tree_allowed) {
            Ok((identity, observed_bytes)) if observed_bytes == size_bytes => Measurement::Identity(identity),
            Ok(_) | Err(ShellError::MemberMismatch | ShellError::BoundExceeded) => Measurement::LengthMismatch,
            Err(ShellError::Tree(error)) => match error.kind() {
                bounded_tree_cap::ShellErrorKind::LimitExceeded(_)
                | bounded_tree_cap::ShellErrorKind::SourceChanged(_) => Measurement::LengthMismatch,
                _ => Measurement::Missing,
            },
            Err(_) => Measurement::Missing,
        }
    }

    fn measure_member(
        &self,
        logical_path: &str,
        size_bytes: u64,
        is_tree_allowed: bool,
    ) -> Result<(String, u64), ShellError> {
        if size_bytes > MAX_MEMBER_BYTES {
            return Err(ShellError::BoundExceeded);
        }
        let relative = Path::new(relative_member_path(logical_path)?);
        let (parent, name) = source_tree::open_parent(&self.root, relative, false)?;
        let metadata = parent.symlink_metadata(&name).map_err(|_| ShellError::MemberUnavailable)?;
        if metadata.is_dir() && is_tree_allowed {
            if size_bytes > source_tree::MAX_TREE_BYTES {
                return Err(ShellError::BoundExceeded);
            }
            let source =
                cap_fs_ext::DirExt::open_dir_nofollow(&parent, &name).map_err(|_| ShellError::MemberUnavailable)?;
            let prepared = source_tree::prepare(&source)?;
            let measured = source_tree::measure(&source, &prepared)?;
            source_tree::check_directory_binding(&parent, Path::new(&name), &source)?;
            return Ok(measured);
        }
        let mut file = source_tree::open_regular(&parent, Path::new(&name), size_bytes)?;
        let digest = source_tree::stream_file(&mut file, None, size_bytes, None)?;
        Ok((digest.to_hex().to_string(), size_bytes))
    }
}

/// Roles may share a logical path only with identical digest AND length.
// r[impl mantle.wasm_source_closure_trees.compatibility]
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
        let has_conflicting_identity = members.iter().any(|member: &RequiredMember| {
            member.logical_path == logical_path.as_str()
                && (member.digest_blake3 != digest_hex || member.size_bytes != *size_bytes)
        });
        if has_conflicting_identity {
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

fn ensure_identity((digest, size_bytes): (String, u64), object: &StoreObject) -> Result<(), ShellError> {
    if digest != object.digest_blake3.to_string() || size_bytes != object.size_bytes {
        return Err(ShellError::MemberMismatch);
    }
    Ok(())
}
