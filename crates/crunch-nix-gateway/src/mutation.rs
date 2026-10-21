//! Bounded staging for an operation already admitted by the existing host.
//! No bearer, build ticket, socket ownership or NAR signature grants upload
//! authority here. The host must check its exact current input admission
//! before calling, including revocation, endpoint generation and byte budget.

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_nix_gateway_core::Reject;
use crunch_nix_gateway_core::{self as core};

use crate::ParsedRequest;
use crate::read_transfer;
use crate::store::VerifiedStore;

struct StagedNar {
    path: PathBuf,
    file: File,
}
impl Drop for StagedNar {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn stage(directory: &Path) -> Result<StagedNar, Reject> {
    let metadata = std::fs::symlink_metadata(directory).map_err(|_| Reject::Authority)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(Reject::Authority);
    }
    let mut nonce = [0_u8; 16];
    getrandom::fill(&mut nonce).map_err(|_| Reject::Authority)?;
    let mut suffix = String::with_capacity(32);
    for byte in nonce {
        use std::fmt::Write as _;
        write!(&mut suffix, "{byte:02x}").map_err(|_| Reject::Authority)?;
    }
    let path = directory.join(format!(".gateway-nar-{suffix}"));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .map_err(|_| Reject::Authority)?;
    Ok(StagedNar { path, file })
}

/// A partial/disconnected transfer is discarded and never publishes PathInfo.
/// Content ingest remains bounded; a signature/hash conflict cannot publish
/// metadata. A successful return is the number of actually verified bytes,
/// which the host must durably reconcile before acknowledging the operation.
// r[impl remote_builds.gateway_bounds_and_recovery]
// r[impl remote_builds.gateway_store_integrity]
pub fn import_admitted_transfer(
    wire: &mut impl Read,
    store: &mut VerifiedStore,
    request: ParsedRequest,
    admitted_inputs: &[String],
    remaining_upload_bytes: u64,
    directory: &Path,
    runtime: &tokio::runtime::Runtime,
    mut reauthorize: impl FnMut() -> Result<(), Reject>,
) -> Result<u64, Reject> {
    let ParsedRequest::AddToStoreNar { ref path, nar_size, .. } = request else {
        return Err(Reject::Operation);
    };
    if admitted_inputs.len() > core::MAX_STORE_PATHS as usize
        || !admitted_inputs.contains(path)
        || nar_size == 0
        || nar_size > remaining_upload_bytes
        || nar_size > core::MAX_TRANSFER_BYTES
    {
        return Err(Reject::Authority);
    }
    reauthorize()?;
    let mut staged = stage(directory)?;
    read_transfer(wire, nar_size, |bytes| staged.file.write_all(bytes), |_| Ok(())).map_err(|_| Reject::Bound)?;
    staged.file.sync_all().map_err(|_| Reject::Authority)?;
    staged.file.seek(SeekFrom::Start(0)).map_err(|_| Reject::Authority)?;
    reauthorize()?;
    let file = staged.file.try_clone().map_err(|_| Reject::Authority)?;
    runtime.block_on(async {
        let mut reader = tokio::fs::File::from_std(file);
        store
            .import_admitted_nar(request, &mut reader, admitted_inputs, remaining_upload_bytes, &mut reauthorize)
            .await
    })
}
