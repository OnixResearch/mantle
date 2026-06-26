use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use async_trait::async_trait;
use bstr::BStr;
use snix_castore::Node;
use snix_castore::PathComponent;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::fs::fuse::FuseDaemon;
use snix_castore::import::fs::ingest_path;
use snix_castore::refscan::ReferencePattern;
use snix_castore::refscan::ReferenceScanner;
use tokio::io::AsyncReadExt;
use tracing::Span;
use tracing::debug;
use tracing::info;
use tracing::instrument;
use tracing::warn;
use uuid::Uuid;

use super::BuildService;
use super::ephemeral_dir::create_ephemeral_dir;
use crate::buildservice::BuildConstraints;
use crate::buildservice::BuildOutput;
use crate::buildservice::BuildRequest;
use crate::buildservice::BuildResult;
use crate::bwrap::Bwrap;
use crate::sandbox::SandboxSpec;
/// Compile-time default for the sandbox shell.
const SANDBOX_SHELL_PLACEHOLDER: &str = "/bin/sh";
const SANDBOX_SHELL_DEFAULT: &str = match option_env!("SNIX_BUILD_SANDBOX_SHELL") {
    Some(path) => path,
    None => SANDBOX_SHELL_PLACEHOLDER,
};
const MAX_INPUT_EXPORT_DEPTH: u32 = 128;
const MAX_INPUT_ROOTS: usize = 65_536;
const MAX_NIX_STORE_SCAN_ENTRIES: u32 = 200_000;

enum ProvidedInputs {
    Fuse { _daemon: FuseDaemon },
    Materialized,
}

/// Resolve the sandbox shell path at runtime.
/// Checks the `SNIX_BUILD_SANDBOX_SHELL` environment variable first,
/// then falls back to the compile-time default. When both are the
/// placeholder [`SANDBOX_SHELL_PLACEHOLDER`], try to discover a static busybox in common
/// NixOS locations so sandboxed builds do not depend on the host glibc.
fn sandbox_shell() -> String {
    let env_shell = std::env::var("SNIX_BUILD_SANDBOX_SHELL").ok();
    let discovered_static = find_static_sandbox_shell();
    choose_sandbox_shell(env_shell.as_deref(), SANDBOX_SHELL_DEFAULT, discovered_static.as_deref())
}

fn choose_sandbox_shell(env_shell: Option<&str>, compile_default: &str, discovered_static: Option<&Path>) -> String {
    if let Some(shell_path) = env_shell {
        if shell_path != SANDBOX_SHELL_PLACEHOLDER {
            return shell_path.to_string();
        }
    }
    if compile_default != SANDBOX_SHELL_PLACEHOLDER {
        if compile_default_shell_exists(compile_default) {
            return compile_default.to_string();
        }
    }
    if let Some(shell_path) = discovered_static {
        return shell_path.display().to_string();
    }
    SANDBOX_SHELL_PLACEHOLDER.to_string()
}

fn compile_default_shell_exists(compile_default: &str) -> bool {
    if compile_default.is_empty() {
        return false;
    }
    let path = Path::new(compile_default);
    path.is_file()
}

fn find_static_sandbox_shell() -> Option<PathBuf> {
    for candidate in ["/run/current-system/sw/bin/busybox-static", "/bin/busybox.static"] {
        let path = Path::new(candidate);
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }
    find_busybox_static_in_dir(Path::new("/nix/store"))
}

fn find_busybox_static_in_dir(store_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store_dir).ok()?;
    let mut scanned_entries: u32 = 0;
    for entry in entries.flatten() {
        scanned_entries = scanned_entries.saturating_add(1);
        if scanned_entries > MAX_NIX_STORE_SCAN_ENTRIES {
            break;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.contains("busybox-static") {
            continue;
        }
        let candidate = entry.path().join("bin").join("busybox");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn should_materialize_inputs_fallback(error: &std::io::Error) -> bool {
    if error.kind() != ErrorKind::Other {
        return false;
    }
    let message = error.to_string();
    if !message.contains("fusermount") {
        return false;
    }
    message.contains("Unexpected exit code when running fusermount")
}

async fn materialize_inputs_to_disk<BS, DS>(
    inputs_root: &Path,
    root_nodes: &BTreeMap<PathComponent, Node>,
    blob_service: &BS,
    directory_service: &DS,
) -> std::io::Result<()>
where
    BS: BlobService + Clone,
    DS: DirectoryService + Clone,
{
    assert!(root_nodes.len() <= MAX_INPUT_ROOTS, "input root count exceeded {}", MAX_INPUT_ROOTS,);
    std::fs::create_dir_all(inputs_root)?;
    for (name, node) in root_nodes {
        let name_str = std::str::from_utf8(name.as_ref()).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid input path component: {e}"))
        })?;
        let dest = inputs_root.join(name_str);
        materialize_node_to_disk(&dest, node, blob_service, directory_service, 0).await?;
    }
    Ok(())
}

async fn materialize_node_to_disk<BS, DS>(
    dest: &Path,
    node: &Node,
    blob_service: &BS,
    directory_service: &DS,
    depth: u32,
) -> std::io::Result<()>
where
    BS: BlobService + Clone,
    DS: DirectoryService + Clone,
{
    if depth >= MAX_INPUT_EXPORT_DEPTH {
        return Err(std::io::Error::other(format!(
            "input export depth limit ({MAX_INPUT_EXPORT_DEPTH}) exceeded at {}",
            dest.display(),
        )));
    }
    match node {
        Node::File { digest, executable, .. } => {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut reader = blob_service
                .open_read(digest)
                .await?
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("blob {digest} not found")))?;
            let mut file = std::fs::File::create(dest)?;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let read_bytes = reader.read(&mut buf).await?;
                if read_bytes == 0 {
                    break;
                }
                std::io::Write::write_all(&mut file, &buf[..read_bytes])?;
            }
            #[cfg(unix)]
            if *executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o555))?;
            }
        }
        Node::Directory { digest, .. } => {
            std::fs::create_dir_all(dest)?;
            let directory = directory_service.get(digest).await.map_err(std::io::Error::other)?.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, format!("directory {digest} not found"))
            })?;
            for (name, child_node) in directory.nodes() {
                let name_str = std::str::from_utf8(name.as_ref()).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid directory entry name: {e}"))
                })?;
                let child_dest = dest.join(name_str);
                Box::pin(materialize_node_to_disk(
                    &child_dest,
                    child_node,
                    blob_service,
                    directory_service,
                    depth.saturating_add(1),
                ))
                .await?;
            }
        }
        Node::Symlink { target, .. } => {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let target_os = std::ffi::OsStr::from_bytes(target.as_ref());
                std::os::unix::fs::symlink(target_os, dest)?;
            }
        }
    }
    Ok(())
}

fn materialize_inputs_blocking<BS, DS>(
    inputs_root: &Path,
    root_nodes: &BTreeMap<PathComponent, Node>,
    blob_service: &BS,
    directory_service: &DS,
) -> std::io::Result<()>
where
    BS: BlobService + Clone,
    DS: DirectoryService + Clone,
{
    let runtime_handle = tokio::runtime::Handle::current();
    tokio::task::block_in_place(|| {
        runtime_handle.block_on(materialize_inputs_to_disk(inputs_root, root_nodes, blob_service, directory_service))
    })
}

pub struct BubblewrapBuildService<BS, DS> {
    /// Root path in which all builds run
    workdir: PathBuf,

    /// Handle to a [BlobService], used by filesystems spawned during builds.
    blob_service: BS,
    /// Handle to a [DirectoryService], used by filesystems spawned during builds.
    directory_service: DS,

    // semaphore to track number of concurrently running builds.
    // this is necessary, as otherwise we very quickly run out of open file handles.
    concurrent_builds: tokio::sync::Semaphore,
}
impl<BS, DS> BubblewrapBuildService<BS, DS> {
    pub fn new(workdir: PathBuf, blob_service: BS, directory_service: DS) -> Self {
        // We map root inside the container to the uid/gid this is running at,
        // and allocate one for uid 1000 into the container from the range we
        // got in /etc/sub{u,g}id.
        // FUTUREWORK: use different uids?
        Self {
            workdir,
            blob_service,
            directory_service,
            concurrent_builds: tokio::sync::Semaphore::new(2),
        }
    }
}

#[async_trait]
impl<BS, DS> BuildService for BubblewrapBuildService<BS, DS>
where
    BS: BlobService + Clone + 'static,
    DS: DirectoryService + Clone + 'static,
{
    #[instrument(skip_all, err)]
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        let _permit = self.concurrent_builds.acquire().await.unwrap();

        let build_name = Uuid::new_v4();
        let build_name_str = build_name.to_string();
        let sandbox_dir = create_ephemeral_dir(&self.workdir, &format!("{build_name_str}-"))?;
        info!(build_name = %build_name_str, sandbox_path = %sandbox_dir.path().display(), "Starting bwrap build");

        let span = Span::current();
        span.record("build_name", build_name_str.clone());

        let blob_service = self.blob_service.clone();
        let directory_service = self.directory_service.clone();

        let spec = SandboxSpec::builder()
            .host_workdir(sandbox_dir.path().to_path_buf())
            .sandbox_workdir(request.working_dir)
            .scratches(request.scratch_paths)
            .command(request.command_args)
            .env_vars(request.environment_vars)
            .additional_files(request.additional_files)
            .with_inputs(request.inputs_dir, move |path| {
                let root_nodes = request.inputs.clone();
                let fs = snix_castore::fs::SnixStoreFs::new(
                    blob_service.clone(),
                    directory_service.clone(),
                    Box::new(root_nodes.clone()),
                    snix_castore::fs::FSSettings {
                        list_root: true,
                        uid_gid_override: None,
                        show_xattr: false,
                    },
                    tokio::runtime::Handle::current(),
                );
                // FUTUREWORK: make fuse daemon threads configurable?
                if std::env::var_os("CRUNCH_NO_FUSE").is_some() {
                    info!(?path, "CRUNCH_NO_FUSE set, materializing inputs to disk");
                    materialize_inputs_blocking(path, &root_nodes, &blob_service, &directory_service)?;
                    Ok(ProvidedInputs::Materialized)
                } else {
                    match FuseDaemon::new(fs, path, 4, false) {
                        Ok(daemon) => Ok(ProvidedInputs::Fuse { _daemon: daemon }),
                        Err(error) => {
                            if !should_materialize_inputs_fallback(&error) {
                                return Err(error);
                            }
                            warn!(?error, ?path, "fusermount mount failed, materializing inputs to disk");
                            materialize_inputs_blocking(path, &root_nodes, &blob_service, &directory_service)?;
                            Ok(ProvidedInputs::Materialized)
                        }
                    }
                }
            })
            .allow_network(request.constraints.contains(&BuildConstraints::NetworkAccess))
            .provide_shell(
                request.constraints.contains(&BuildConstraints::ProvideBinSh).then_some(sandbox_shell().into()),
            )
            .build();

        let outcome = Bwrap::initialize(spec)?.run().await?;

        if !outcome.output().status.success() {
            let stdout = BStr::new(&outcome.output().stdout);
            let stderr = BStr::new(&outcome.output().stderr);

            warn!(stdout=%stdout, stderr=%stderr, exit_code=%outcome.output().status, "build failed");

            // Include stdout+stderr in the error so callers can display the
            // build log. Previously this was just "nonzero exit code".
            let mut log = String::new();
            if !outcome.output().stdout.is_empty() {
                log.push_str(&String::from_utf8_lossy(&outcome.output().stdout));
            }
            if !outcome.output().stderr.is_empty() {
                if !log.is_empty() {
                    log.push('\n');
                }
                log.push_str(&String::from_utf8_lossy(&outcome.output().stderr));
            }
            let msg = if log.is_empty() {
                format!("nonzero exit code: {}", outcome.output().status)
            } else {
                format!("nonzero exit code: {}\n{}", outcome.output().status, log)
            };
            return Err(std::io::Error::other(msg));
        }

        let outputs: Vec<_> = request.outputs.iter().filter_map(|o| outcome.find_path(o)).collect();
        if outputs.len() != request.outputs.len() {
            warn!("Not all outputs produced");
            return Err(std::io::Error::other("Not all outputs produced".to_string()));
        }
        let patterns = ReferencePattern::new(request.refscan_needles);
        let outputs = futures::future::try_join_all(outputs.into_iter().enumerate().map(|(i, host_output_path)| {
            let output_path = &request.outputs[i];
            debug!(host.path=?host_output_path, output.path=?output_path, "ingesting path");
            let patterns = patterns.clone();
            async move {
                let scanner = ReferenceScanner::new(patterns);
                Ok::<_, std::io::Error>(BuildOutput {
                    node: ingest_path(&self.blob_service, &self.directory_service, host_output_path, Some(&scanner))
                        .await
                        .map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unable to ingest output: {e}"),
                            )
                        })?,

                    output_needles: scanner
                        .matches()
                        .into_iter()
                        .enumerate()
                        .filter(|(_, val)| *val)
                        .map(|(idx, _)| idx as u64)
                        .collect(),
                })
            }
        }))
        .await?;
        // Capture stdout+stderr for the log (even on success)
        let mut log = String::new();
        if !outcome.output().stdout.is_empty() {
            log.push_str(&String::from_utf8_lossy(&outcome.output().stdout));
        }
        if !outcome.output().stderr.is_empty() {
            if !log.is_empty() {
                log.push('\n');
            }
            log.push_str(&String::from_utf8_lossy(&outcome.output().stderr));
        }

        Ok(BuildResult {
            outputs,
            log: if log.is_empty() { None } else { Some(log) },
        })
    }
}

#[cfg(test)]
mod tests {
    use snix_castore::Directory;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use tokio::io::AsyncWriteExt;

    use super::*;

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("snix-build-bwrap-tests".to_string(), RedbDirectoryServiceConfig::default())
            .unwrap()
    }

    async fn insert_blob(bs: &MemoryBlobService, data: &[u8], executable: bool) -> Node {
        let mut writer = bs.open_write().await;
        writer.write_all(data).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: data.len() as u64,
            executable,
        }
    }

    #[test]
    fn choose_sandbox_shell_prefers_explicit_env() {
        let chosen = choose_sandbox_shell(
            Some("/nix/store/abc-static-bash/bin/bash"),
            "/bin/sh",
            Some(Path::new("/nix/store/def-busybox-static/bin/busybox")),
        );
        assert_eq!(chosen, "/nix/store/abc-static-bash/bin/bash");
    }

    #[test]
    fn choose_sandbox_shell_uses_discovered_static_for_placeholder() {
        let chosen = choose_sandbox_shell(
            Some("/bin/sh"),
            "/bin/sh",
            Some(Path::new("/nix/store/def-busybox-static/bin/busybox")),
        );
        assert_eq!(chosen, "/nix/store/def-busybox-static/bin/busybox");
    }

    #[test]
    fn choose_sandbox_shell_ignores_missing_compile_default() {
        let chosen = choose_sandbox_shell(
            Some("/bin/sh"),
            "/nix/store/missing-busybox/bin/busybox",
            Some(Path::new("/nix/store/def-busybox-static/bin/busybox")),
        );
        assert_eq!(chosen, "/nix/store/def-busybox-static/bin/busybox");
    }

    #[test]
    fn choose_sandbox_shell_keeps_existing_compile_default() {
        let dir = tempfile::tempdir().unwrap();
        let shell_path = dir.path().join("busybox");
        std::fs::write(&shell_path, "#!/bin/sh\n").unwrap();
        let chosen = choose_sandbox_shell(Some("/bin/sh"), shell_path.to_str().unwrap(), None);
        assert_eq!(chosen, shell_path.display().to_string());
    }

    #[test]
    fn choose_sandbox_shell_keeps_placeholder_without_static_candidate() {
        let chosen = choose_sandbox_shell(None, "/bin/sh", None);
        assert_eq!(chosen, "/bin/sh");
    }

    #[test]
    fn find_busybox_static_in_dir_finds_candidate() {
        let dir = tempfile::tempdir().unwrap();
        let candidate = dir.path().join("abc-busybox-static").join("bin");
        std::fs::create_dir_all(&candidate).unwrap();
        let busybox = candidate.join("busybox");
        std::fs::write(&busybox, "#!/bin/sh\n").unwrap();

        let found = find_busybox_static_in_dir(dir.path()).unwrap();
        assert_eq!(found, busybox);
    }

    #[test]
    fn should_materialize_inputs_fallback_matches_fusermount_exit() {
        let error =
            std::io::Error::other("fuse session failure: Unexpected exit code when running fusermount: Some(1)");
        assert!(should_materialize_inputs_fallback(&error));

        let unrelated = std::io::Error::other("failed to create build dir");
        assert!(!should_materialize_inputs_fallback(&unrelated));

        let wrong_kind =
            std::io::Error::new(ErrorKind::PermissionDenied, "Unexpected exit code when running fusermount: Some(1)");
        assert!(!should_materialize_inputs_fallback(&wrong_kind));
    }

    #[tokio::test]
    async fn materialize_inputs_to_disk_writes_nested_tree() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let file_node = insert_blob(&bs, b"hello", true).await;
        let mut inner = Directory::new();
        inner.add("tool".try_into().unwrap(), file_node).unwrap();
        inner
            .add("tool-link".try_into().unwrap(), Node::Symlink {
                target: SymlinkTarget::try_from("tool").unwrap(),
            })
            .unwrap();
        let inner_digest = inner.digest();
        let inner_size = inner.size();
        ds.put(inner).await.unwrap();

        let mut root_nodes = BTreeMap::new();
        root_nodes.insert(PathComponent::try_from("pkg").unwrap(), Node::Directory {
            digest: inner_digest,
            size: inner_size,
        });

        let dest = tempfile::tempdir().unwrap();
        materialize_inputs_to_disk(dest.path(), &root_nodes, &bs, &ds).await.unwrap();

        let file_path = dest.path().join("pkg").join("tool");
        let link_path = dest.path().join("pkg").join("tool-link");
        assert_eq!(std::fs::read(&file_path).unwrap(), b"hello");
        let link_target = std::fs::read_link(&link_path).unwrap();
        assert_eq!(link_target, PathBuf::from("tool"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file_path).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0, "materialized executable should keep x bit");
        }
    }
}
