use std::ffi::CString;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use cap_fs_ext::DirExt;
use crunch_release_core::PublicationDestinationObservation;
use crunch_release_core::PublicationPlan;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;

pub(crate) const RELEASE_STAGE_MARKER_FILENAME: &str = ".mantle-release-stage-owner.json";
const RELEASE_STAGE_MARKER_SCHEMA: &str = "mantle-release-stage-owner-v1";
const RELEASE_STAGE_PREFIX: &str = ".mantle-release-stage-";
const RELEASE_QUARANTINE_PREFIX: &str = ".mantle-release-quarantine-";
const RELEASE_STAGE_RANDOM_BYTES: usize = 16;
const HEX_CHARS_PER_BYTE: usize = 2;
const RELEASE_STAGE_CREATE_ATTEMPTS_MAX: u32 = 16;
const RELEASE_PARENT_ENTRIES_MAX: u32 = 4_096;
const RELEASE_STAGE_MARKER_BYTES_MAX: u64 = 4_096;
const RELEASE_PLAN_ID_STAGE_PREFIX_CHARS: usize = 16;
#[cfg(unix)]
const PRIVATE_STAGE_DIRECTORY_MODE: libc::mode_t = 0o700;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseStageOwnershipMarker {
    schema: String,
    plan_identity_blake3: String,
    final_name: String,
    stage_name: String,
}

#[derive(Debug)]
pub(crate) struct ReleasePublicationStage {
    parent_root: ReleaseCapabilityRoot,
    final_path: PathBuf,
    final_name: String,
    stage_path: PathBuf,
    stage_name: String,
    stage_root: ReleaseCapabilityRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PublicationCommitError {
    DestinationExists(String),
    Other(String),
}

impl ReleasePublicationStage {
    pub(crate) fn stage_path(&self) -> &Path {
        &self.stage_path
    }

    pub(crate) fn final_path(&self) -> &Path {
        &self.final_path
    }

    pub(crate) fn root(&self) -> &ReleaseCapabilityRoot {
        &self.stage_root
    }

    pub(crate) fn remove_ownership_marker(&self) -> Result<(), RunError> {
        let marker = ValidatedReleasePath::new(RELEASE_STAGE_MARKER_FILENAME)
            .map_err(|error| RunError::Internal(format!("invalid release stage marker path: {error:?}")))?;
        self.stage_root
            .remove_file_nofollow(&marker)
            .map_err(|error| RunError::Internal(format!("removing release stage ownership marker: {error}")))
    }

    // r[impl mantle.release_provenance.bundle_publication.atomic_commit]
    pub(crate) fn commit_no_replace(&self) -> Result<(), PublicationCommitError> {
        if self.stage_path == self.final_path {
            return Err(PublicationCommitError::Other(
                "release stage path must differ from final destination".to_string(),
            ));
        }
        rename_child_no_replace(self.parent_root.dir(), &self.stage_name, &self.final_name).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                return PublicationCommitError::DestinationExists(format!(
                    "release publication destination appeared before commit: {}",
                    self.final_path.display()
                ));
            }
            PublicationCommitError::Other(format!(
                "atomically publishing release bundle without replacement to {}: {error}",
                self.final_path.display()
            ))
        })
    }

    pub(crate) fn cleanup_current_stage(&self) -> Result<(), RunError> {
        self.parent_root.dir().remove_dir_all(&self.stage_name).map_err(|error| {
            RunError::Internal(format!("removing current release stage {}: {error}", self.stage_path.display()))
        })
    }
}

pub(crate) fn observe_publication_destination(path: &Path) -> Result<PublicationDestinationObservation, RunError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PublicationDestinationObservation::Absent);
        }
        Err(error) => {
            return Err(RunError::Internal(format!(
                "observing release publication destination {}: {error}",
                path.display()
            )));
        }
    };
    if metadata.file_type().is_symlink() {
        return Ok(PublicationDestinationObservation::Symlink);
    }
    if metadata.is_file() {
        return Ok(PublicationDestinationObservation::File);
    }
    if !metadata.is_dir() {
        return Ok(PublicationDestinationObservation::Other);
    }
    let empty = std::fs::read_dir(path)
        .map_err(|error| {
            RunError::Internal(format!("reading release publication destination {}: {error}", path.display()))
        })?
        .next()
        .is_none();
    Ok(if empty {
        PublicationDestinationObservation::EmptyDirectory
    } else {
        PublicationDestinationObservation::NonEmptyDirectory
    })
}

// r[impl mantle.release_provenance.bundle_publication.stale_stage]
// r[impl mantle.release_provenance.bundle_publication.atomic_commit]
pub(crate) fn create_release_publication_stage(
    final_path: &Path,
    plan: &PublicationPlan,
) -> Result<ReleasePublicationStage, RunError> {
    let absolute = std::path::absolute(final_path).map_err(|error| {
        RunError::Internal(format!("resolving release destination {}: {error}", final_path.display()))
    })?;
    let parent_path = absolute
        .parent()
        .ok_or_else(|| RunError::Internal(format!("release destination has no parent: {}", absolute.display())))?
        .to_path_buf();
    let final_name = absolute
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| {
            RunError::Internal(format!("release destination name is not valid UTF-8: {}", absolute.display()))
        })?
        .to_string();
    let parent_root =
        ReleaseCapabilityRoot::create_ambient_dir_all_nofollow(ReleaseRootKind::ReleaseEvidence, &parent_path)
            .map_err(|error| {
                RunError::Internal(format!("opening release destination parent {}: {error}", parent_path.display()))
            })?;
    quarantine_matching_stale_stages(&parent_root, &parent_path, &final_name, plan)?;
    let (stage_name, stage_root) = create_private_stage_directory(&parent_root, plan)?;
    let stage_path = parent_path.join(&stage_name);
    if let Err(marker_error) = write_stage_marker(&stage_root, plan, &final_name, &stage_name) {
        let cleanup = parent_root.dir().remove_dir_all(&stage_name);
        if let Err(cleanup_error) = cleanup {
            return Err(RunError::Internal(format!(
                "{marker_error}; cleaning markerless release stage {} failed: {cleanup_error}",
                stage_path.display()
            )));
        }
        return Err(marker_error);
    }
    assert_ne!(stage_path, absolute, "release stage must be a sibling of the final destination");
    assert_eq!(stage_root.kind(), ReleaseRootKind::ReleaseEvidence);
    Ok(ReleasePublicationStage {
        parent_root,
        final_path: absolute,
        final_name,
        stage_path,
        stage_name,
        stage_root,
    })
}

fn create_private_stage_directory(
    parent_root: &ReleaseCapabilityRoot,
    plan: &PublicationPlan,
) -> Result<(String, ReleaseCapabilityRoot), RunError> {
    let prefix = stage_prefix(plan)?;
    for _attempt in 0..RELEASE_STAGE_CREATE_ATTEMPTS_MAX {
        let stage_name = format!("{prefix}{}", random_hex_suffix());
        match parent_root.dir().create_dir(&stage_name) {
            Ok(()) => {
                let dir = match parent_root.dir().open_dir_nofollow(&stage_name) {
                    Ok(dir) => dir,
                    Err(error) => {
                        let cleanup = parent_root.dir().remove_dir_all(&stage_name);
                        if let Err(cleanup_error) = cleanup {
                            return Err(RunError::Internal(format!(
                                "opening created release stage {stage_name}: {error}; cleanup failed: {cleanup_error}"
                            )));
                        }
                        return Err(RunError::Internal(format!("opening created release stage {stage_name}: {error}")));
                    }
                };
                if let Err(error) = set_private_stage_mode(&dir) {
                    let cleanup = parent_root.dir().remove_dir_all(&stage_name);
                    if let Err(cleanup_error) = cleanup {
                        return Err(RunError::Internal(format!(
                            "{error}; cleaning unprotected release stage {stage_name} failed: {cleanup_error}"
                        )));
                    }
                    return Err(error);
                }
                return Ok((stage_name, ReleaseCapabilityRoot::from_open_dir(ReleaseRootKind::ReleaseEvidence, dir)));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(RunError::Internal(format!("creating private release stage {stage_name}: {error}")));
            }
        }
    }
    Err(RunError::Internal(format!(
        "creating private release stage exceeded {RELEASE_STAGE_CREATE_ATTEMPTS_MAX} attempts"
    )))
}

fn write_stage_marker(
    stage_root: &ReleaseCapabilityRoot,
    plan: &PublicationPlan,
    final_name: &str,
    stage_name: &str,
) -> Result<(), RunError> {
    let marker = ReleaseStageOwnershipMarker {
        schema: RELEASE_STAGE_MARKER_SCHEMA.to_string(),
        plan_identity_blake3: plan.plan_identity_blake3.clone(),
        final_name: final_name.to_string(),
        stage_name: stage_name.to_string(),
    };
    let bytes = serde_json::to_vec(&marker)
        .map_err(|error| RunError::Internal(format!("serializing release stage ownership marker: {error}")))?;
    let path = ValidatedReleasePath::new(RELEASE_STAGE_MARKER_FILENAME)
        .map_err(|error| RunError::Internal(format!("invalid release stage marker path: {error:?}")))?;
    stage_root
        .write_new_file_nofollow(&path, &bytes)
        .map_err(|error| RunError::Internal(format!("writing release stage ownership marker: {error}")))?;
    assert!(!bytes.is_empty(), "release stage marker must not be empty");
    assert_eq!(stage_root.kind(), ReleaseRootKind::ReleaseEvidence);
    Ok(())
}

fn quarantine_matching_stale_stages(
    parent_root: &ReleaseCapabilityRoot,
    parent_path: &Path,
    final_name: &str,
    plan: &PublicationPlan,
) -> Result<(), RunError> {
    let prefix = stage_prefix(plan)?;
    let mut candidates = collect_parent_candidate_names(parent_root, &prefix)?;
    candidates.sort();
    for stage_name in candidates {
        if !valid_stage_marker(parent_root, &stage_name, final_name, plan)? {
            continue;
        }
        let quarantine_name = format!("{RELEASE_QUARANTINE_PREFIX}{}-{}", plan_id_prefix(plan)?, random_hex_suffix());
        rename_child_no_replace(parent_root.dir(), &stage_name, &quarantine_name).map_err(|error| {
            RunError::Internal(format!(
                "quarantining Mantle-owned stale release stage {}: {error}",
                parent_path.join(&stage_name).display()
            ))
        })?;
    }
    Ok(())
}

fn collect_parent_candidate_names(parent_root: &ReleaseCapabilityRoot, prefix: &str) -> Result<Vec<String>, RunError> {
    let mut names = Vec::new();
    let mut visited_count = 0_u32;
    let entries = parent_root
        .dir()
        .entries()
        .map_err(|error| RunError::Internal(format!("reading release publication parent: {error}")))?;
    for entry in entries {
        visited_count = visited_count
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("release publication parent entry count overflowed u32".to_string()))?;
        if visited_count > RELEASE_PARENT_ENTRIES_MAX {
            return Err(RunError::Internal(format!(
                "release publication parent exceeds {RELEASE_PARENT_ENTRIES_MAX} entries"
            )));
        }
        let entry =
            entry.map_err(|error| RunError::Internal(format!("reading release publication parent entry: {error}")))?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with(prefix) {
            names.push(name);
        }
    }
    assert!(visited_count <= RELEASE_PARENT_ENTRIES_MAX);
    let entries_max = usize::try_from(RELEASE_PARENT_ENTRIES_MAX).expect("release parent entry bound must fit usize");
    assert!(names.len() <= entries_max);
    Ok(names)
}

fn valid_stage_marker(
    parent_root: &ReleaseCapabilityRoot,
    stage_name: &str,
    final_name: &str,
    plan: &PublicationPlan,
) -> Result<bool, RunError> {
    let metadata = match parent_root.dir().symlink_metadata(stage_name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(RunError::Internal(format!("reading stale release stage metadata: {error}"))),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Ok(false);
    }
    let stage_dir = parent_root
        .dir()
        .open_dir_nofollow(stage_name)
        .map_err(|error| RunError::Internal(format!("opening stale release stage {stage_name}: {error}")))?;
    let stage_root = ReleaseCapabilityRoot::from_open_dir(ReleaseRootKind::ReleaseEvidence, stage_dir);
    let Some(marker) = read_stage_marker(&stage_root)? else {
        return Ok(false);
    };
    Ok(marker.schema == RELEASE_STAGE_MARKER_SCHEMA
        && marker.plan_identity_blake3 == plan.plan_identity_blake3
        && marker.final_name == final_name
        && marker.stage_name == stage_name)
}

fn read_stage_marker(stage_root: &ReleaseCapabilityRoot) -> Result<Option<ReleaseStageOwnershipMarker>, RunError> {
    let path = ValidatedReleasePath::new(RELEASE_STAGE_MARKER_FILENAME)
        .map_err(|error| RunError::Internal(format!("invalid release stage marker path: {error:?}")))?;
    let mut file = match stage_root.open_file_read_nofollow(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Ok(None),
    };
    let metadata = file
        .metadata()
        .map_err(|error| RunError::Internal(format!("reading release stage marker metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() > RELEASE_STAGE_MARKER_BYTES_MAX {
        return Ok(None);
    }
    let capacity = usize::try_from(metadata.len())
        .map_err(|_| RunError::Internal("release stage marker length overflowed usize".to_string()))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading release stage marker: {error}")))?;
    Ok(serde_json::from_slice(&bytes).ok())
}

fn stage_prefix(plan: &PublicationPlan) -> Result<String, RunError> {
    Ok(format!("{RELEASE_STAGE_PREFIX}{}-", plan_id_prefix(plan)?))
}

fn plan_id_prefix(plan: &PublicationPlan) -> Result<&str, RunError> {
    plan.plan_identity_blake3
        .get(..RELEASE_PLAN_ID_STAGE_PREFIX_CHARS)
        .ok_or_else(|| RunError::Internal("release publication plan identity is too short".to_string()))
}

fn random_hex_suffix() -> String {
    let mut bytes = [0_u8; RELEASE_STAGE_RANDOM_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let rendered_capacity = RELEASE_STAGE_RANDOM_BYTES
        .checked_mul(HEX_CHARS_PER_BYTE)
        .expect("bounded release stage suffix capacity must fit usize");
    let mut rendered = String::with_capacity(rendered_capacity);
    for byte in bytes {
        rendered.push_str(&format!("{byte:02x}"));
    }
    assert_eq!(rendered.len(), rendered_capacity);
    rendered
}

#[cfg(unix)]
fn set_private_stage_mode(dir: &cap_std::fs::Dir) -> Result<(), RunError> {
    use cap_std::fs::Permissions;
    use cap_std::fs::PermissionsExt;

    dir.set_permissions(".", Permissions::from_mode(PRIVATE_STAGE_DIRECTORY_MODE))
        .map_err(|error| RunError::Internal(format!("setting private release stage mode: {error}")))
}

#[cfg(not(unix))]
fn set_private_stage_mode(_dir: &cap_std::fs::Dir) -> Result<(), RunError> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn rename_child_no_replace(
    parent: &cap_std::fs::Dir,
    source_name: &str,
    destination_name: &str,
) -> std::io::Result<()> {
    use std::os::fd::AsRawFd;

    let source = CString::new(source_name)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "release source name contains NUL"))?;
    let destination = CString::new(destination_name)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "release destination name contains NUL"))?;
    // SAFETY: both names are relative NUL-terminated strings and both directory fds refer to the same
    // open parent.
    let result = unsafe {
        libc::renameat2(
            parent.as_raw_fd(),
            source.as_ptr(),
            parent.as_raw_fd(),
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        return Ok(());
    }
    Err(std::io::Error::last_os_error())
}

#[cfg(not(target_os = "linux"))]
fn rename_child_no_replace(
    _parent: &cap_std::fs::Dir,
    _source_name: &str,
    _destination_name: &str,
) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic no-replace release publication is unsupported on this platform",
    ))
}
