use std::fs;
use std::io::Read as _;
use std::path::Path;

use super::*;

pub(super) const DEV_RESUME_MANIFEST_FILE: &str = "manifest.json";
const DEV_RESUME_MANIFEST_BYTES_MAX: u64 = 65_536;

pub(super) fn read_manifest(root: &Path) -> Result<crunch_dev_resume_core::ResumeBundleManifest, RunError> {
    require_manifest_root(root)?;
    let path = root.join(DEV_RESUME_MANIFEST_FILE);
    let file = open_manifest_no_follow(&path)
        .map_err(|error| proof_error(format!("opening dev resume manifest {}: {error}", path.display())))?;
    let metadata = file
        .metadata()
        .map_err(|error| proof_error(format!("reading dev resume manifest metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > DEV_RESUME_MANIFEST_BYTES_MAX {
        return Err(proof_error("dev resume manifest file shape is invalid".to_string()));
    }
    let bytes_max = DEV_RESUME_MANIFEST_BYTES_MAX
        .checked_add(1)
        .ok_or_else(|| proof_error("dev resume manifest read bound overflowed".to_string()))?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len())
            .map_err(|_| proof_error("dev resume manifest length does not fit usize".to_string()))?,
    );
    file.take(bytes_max)
        .read_to_end(&mut bytes)
        .map_err(|error| proof_error(format!("reading dev resume manifest {}: {error}", path.display())))?;
    let observed_bytes = u64::try_from(bytes.len())
        .map_err(|_| proof_error("dev resume manifest observed length does not fit u64".to_string()))?;
    if observed_bytes != metadata.len() {
        return Err(proof_error("dev resume manifest changed during its bounded read".to_string()));
    }
    let manifest = serde_json::from_slice(&bytes)
        .map_err(|error| proof_error(format!("parsing dev resume manifest {}: {error}", path.display())))?;
    debug_assert!(!bytes.is_empty());
    Ok(manifest)
}

fn require_manifest_root(root: &Path) -> Result<(), RunError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| proof_error(format!("reading dev resume manifest root {}: {error}", root.display())))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(proof_error("dev resume manifest root must be a non-symlink directory".to_string()));
    }
    Ok(())
}

#[cfg(unix)]
fn open_manifest_no_follow(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)
}

#[cfg(not(unix))]
fn open_manifest_no_follow(path: &Path) -> std::io::Result<fs::File> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "dev resume manifest symlink rejected"));
    }
    fs::OpenOptions::new().read(true).open(path)
}
