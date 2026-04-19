use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use data_encoding::HEXLOWER;
use futures::StreamExt;
use nix_compat::store_path::StorePath;
use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::CaMappings;
use crate::Error;
use crate::StoreFallbackMode;

/// Services and paths needed for GC operations.
pub struct GcContext<'a> {
    pub state_dir: &'a Path,
    pub output_dir_str: &'a str,
    pub store_dir: &'a str,
    pub pathinfo: &'a dyn PathInfoService,
    pub directory_service: &'a dyn DirectoryService,
    pub blob_service: &'a dyn BlobService,
}
use crate::artifact_attestation_file_path;
use crate::closure::resolve_closure;
use crate::roots;
use crate::roots::GcRootRecord;

const MAX_GC_BYTES_WALK_ENTRIES: u32 = 100_000;
const MAX_GC_FILE_SCAN_ENTRIES: u32 = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcOperationKind {
    ExportedOutputs,
    PathInfoRewrite,
    ArtifactAttestations,
    ClosureAttestations,
    DirectoryRewrite,
    BlobIndexFiles,
    BlobChunkFiles,
    CaMappings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcReport {
    pub is_dry_run: bool,
    pub retained_root_count: u32,
    pub candidate_path_count: u32,
    pub reclaimable_bytes_total: u64,
    pub candidate_paths: Vec<String>,
    pub candidate_blob_index_count: u32,
    pub candidate_blob_chunk_count: u32,
    pub candidate_artifact_attestation_count: u32,
    pub candidate_closure_attestation_count: u32,
    pub candidate_exported_output_count: u32,
    pub operations: Vec<GcOperationKind>,
}

#[derive(Default)]
struct LiveCastoreState {
    directories: HashMap<B3Digest, Directory>,
    blob_index_digests: HashSet<B3Digest>,
    chunk_digests: HashSet<B3Digest>,
}

struct GcPlan {
    retained_roots: Vec<GcRootRecord>,
    live_pathinfos: Vec<PathInfo>,
    dead_pathinfos: Vec<PathInfo>,
    live_castore: LiveCastoreState,
    orphaned_on_disk: Vec<PathBuf>,
    artifact_attestation_paths: Vec<PathBuf>,
    closure_attestation_paths: Vec<PathBuf>,
    blob_index_paths: Vec<PathBuf>,
    blob_chunk_paths: Vec<PathBuf>,
    candidate_paths: Vec<String>,
    reclaimable_bytes_total: u64,
}

pub async fn run_gc(
    ctx: &GcContext<'_>,
    ca_mappings: &mut CaMappings,
    is_dry_run: bool,
) -> Result<GcReport, Error> {
    assert!(!ctx.store_dir.is_empty(), "store_dir must not be empty");
    assert!(ctx.store_dir.starts_with('/'), "store_dir must be absolute");

    let plan = build_plan(ctx).await?;
    let mut gc_result = GcReport {
        is_dry_run,
        retained_root_count: saturating_u32(plan.retained_roots.len()),
        candidate_path_count: saturating_u32(plan.dead_pathinfos.len()),
        reclaimable_bytes_total: plan.reclaimable_bytes_total,
        candidate_paths: plan.candidate_paths.clone(),
        candidate_blob_index_count: saturating_u32(plan.blob_index_paths.len()),
        candidate_blob_chunk_count: saturating_u32(plan.blob_chunk_paths.len()),
        candidate_artifact_attestation_count: saturating_u32(plan.artifact_attestation_paths.len()),
        candidate_closure_attestation_count: saturating_u32(plan.closure_attestation_paths.len()),
        candidate_exported_output_count: saturating_u32(plan.orphaned_on_disk.len()),
        operations: Vec::new(),
    };
    if is_dry_run {
        return Ok(gc_result);
    }

    remove_exported_outputs(&plan.orphaned_on_disk)?;
    gc_result.operations.push(GcOperationKind::ExportedOutputs);

    rewrite_pathinfo_db(ctx.state_dir, &plan.live_pathinfos).await?;
    gc_result.operations.push(GcOperationKind::PathInfoRewrite);

    remove_files(&plan.artifact_attestation_paths)?;
    gc_result.operations.push(GcOperationKind::ArtifactAttestations);

    remove_files(&plan.closure_attestation_paths)?;
    gc_result.operations.push(GcOperationKind::ClosureAttestations);

    rewrite_directory_db(ctx.state_dir, plan.live_castore.directories.values()).await?;
    gc_result.operations.push(GcOperationKind::DirectoryRewrite);

    remove_files(&plan.blob_index_paths)?;
    gc_result.operations.push(GcOperationKind::BlobIndexFiles);

    remove_files(&plan.blob_chunk_paths)?;
    gc_result.operations.push(GcOperationKind::BlobChunkFiles);

    let live_paths: BTreeSet<String> = plan
        .live_pathinfos
        .iter()
        .map(|path_info| path_info.store_path.to_absolute_path_with_prefix(ctx.store_dir))
        .collect();
    ca_mappings.retain_output_paths(&live_paths);
    ca_mappings.save_checked(ctx.state_dir).map_err(|err| Error::Gc(format!("saving CA mappings: {err}")))?;
    gc_result.operations.push(GcOperationKind::CaMappings);

    Ok(gc_result)
}

async fn build_plan(ctx: &GcContext<'_>) -> Result<GcPlan, Error> {
    assert!(!ctx.store_dir.is_empty(), "build_plan: store_dir must not be empty");
    assert!(ctx.store_dir.starts_with('/'), "build_plan: store_dir must be absolute");

    let retained_roots = roots::list_roots(ctx.state_dir)?;
    let live_paths = mark_live_paths(&retained_roots, ctx.pathinfo, ctx.store_dir).await?;
    let snapshot = snapshot_pathinfos(ctx.pathinfo).await?;
    let (live_pathinfos, dead_pathinfos) = split_pathinfos(snapshot, &live_paths);
    let live_castore = collect_live_castore_state(&live_pathinfos, ctx.directory_service, ctx.blob_service).await?;
    let orphaned_on_disk = collect_existing_exported_outputs(&dead_pathinfos, ctx.output_dir_str)?;
    let artifact_attestation_paths =
        collect_existing_artifact_attestation_paths(ctx.state_dir, ctx.store_dir, &dead_pathinfos)?;
    let closure_attestation_paths = collect_dead_closure_attestations(ctx.state_dir, &retained_roots)?;
    let blob_index_paths = collect_dead_blob_files(ctx.state_dir, &live_castore.blob_index_digests, true)?;
    let blob_chunk_paths = collect_dead_blob_files(ctx.state_dir, &live_castore.chunk_digests, false)?;
    let reclaimable_bytes_total = compute_reclaimable_bytes(
        &orphaned_on_disk,
        &artifact_attestation_paths,
        &closure_attestation_paths,
        &blob_index_paths,
        &blob_chunk_paths,
    )?;
    let candidate_paths = dead_pathinfos
        .iter()
        .map(|path_info| path_info.store_path.to_absolute_path_with_prefix(ctx.store_dir))
        .collect();

    Ok(GcPlan {
        retained_roots,
        live_pathinfos,
        dead_pathinfos,
        live_castore,
        orphaned_on_disk,
        artifact_attestation_paths,
        closure_attestation_paths,
        blob_index_paths,
        blob_chunk_paths,
        candidate_paths,
        reclaimable_bytes_total,
    })
}

async fn mark_live_paths(
    retained_roots: &[GcRootRecord],
    pathinfo: &dyn PathInfoService,
    store_dir: &str,
) -> Result<BTreeSet<StorePath<String>>, Error> {
    let mut live_paths = BTreeSet::new();
    for root in retained_roots {
        let store_path = roots::parse_logical_store_path(roots::LogicalStorePathRef {
            logical_path: &root.logical_path,
            store_dir,
        })?;
        let closure = resolve_closure(&store_path, pathinfo, None, StoreFallbackMode::Strict, store_dir).await?;
        for member in closure.paths {
            live_paths.insert(member);
        }
    }
    Ok(live_paths)
}

pub(crate) async fn snapshot_pathinfos(pathinfo: &dyn PathInfoService) -> Result<Vec<PathInfo>, Error> {
    const MAX_PATHINFO_ENTRIES: usize = 1_000_000;
    let mut snapshot = Vec::with_capacity(256);
    let mut stream = pathinfo.list();
    for _ in 0..MAX_PATHINFO_ENTRIES {
        let Some(item) = stream.next().await else { break };
        let path_info = item.map_err(|err| Error::Gc(format!("listing PathInfo rows: {err}")))?;
        snapshot.push(path_info);
    }
    Ok(snapshot)
}

fn split_pathinfos(
    snapshot: Vec<PathInfo>,
    live_paths: &BTreeSet<StorePath<String>>,
) -> (Vec<PathInfo>, Vec<PathInfo>) {
    let mut live = Vec::with_capacity(snapshot.len());
    let mut dead = Vec::with_capacity(snapshot.len());
    for path_info in snapshot {
        if live_paths.contains(&path_info.store_path) {
            live.push(path_info);
        } else {
            dead.push(path_info);
        }
    }
    live.sort_by_key(|path_info| path_info.store_path.to_string());
    dead.sort_by_key(|path_info| path_info.store_path.to_string());
    (live, dead)
}

async fn collect_live_castore_state(
    live_pathinfos: &[PathInfo],
    directory_service: &dyn DirectoryService,
    blob_service: &dyn BlobService,
) -> Result<LiveCastoreState, Error> {
    let mut state = LiveCastoreState {
        directories: HashMap::new(),
        blob_index_digests: HashSet::new(),
        chunk_digests: HashSet::new(),
    };
    for path_info in live_pathinfos {
        collect_node_state(&path_info.node, directory_service, blob_service, &mut state).await?;
    }
    Ok(state)
}

async fn collect_node_state(
    node: &Node,
    directory_service: &dyn DirectoryService,
    blob_service: &dyn BlobService,
    state: &mut LiveCastoreState,
) -> Result<(), Error> {
    match node {
        Node::File { digest, .. } => collect_blob_state(*digest, blob_service, state).await,
        Node::Symlink { .. } => Ok(()),
        Node::Directory { digest, .. } => {
            collect_directory_state(*digest, directory_service, blob_service, state).await
        }
    }
}

async fn collect_blob_state(
    digest: B3Digest,
    blob_service: &dyn BlobService,
    state: &mut LiveCastoreState,
) -> Result<(), Error> {
    assert!(!digest.as_ref().is_empty(), "blob digest must not be empty");

    if state.blob_index_digests.contains(&digest) || state.chunk_digests.contains(&digest) {
        return Ok(());
    }
    let chunks = blob_service
        .chunks(&digest)
        .await
        .map_err(|err| Error::Gc(format!("reading blob metadata for {}: {err}", encode_digest(&digest))))?;
    let Some(chunks) = chunks else {
        return Err(Error::Gc(format!(
            "missing castore blob metadata for reachable digest {}",
            encode_digest(&digest)
        )));
    };
    if chunks.is_empty() {
        state.chunk_digests.insert(digest);
        return Ok(());
    }
    state.blob_index_digests.insert(digest);
    for chunk in chunks {
        let chunk_digest = B3Digest::try_from(chunk.digest)
            .map_err(|err| Error::Gc(format!("invalid chunk digest in blob {}: {err}", encode_digest(&digest))))?;
        state.chunk_digests.insert(chunk_digest);
    }
    let is_blob_recorded = state.blob_index_digests.contains(&digest) || state.chunk_digests.contains(&digest);
    assert!(is_blob_recorded, "processed blob must be recorded in state");
    Ok(())
}

async fn collect_directory_state(
    root_digest: B3Digest,
    directory_service: &dyn DirectoryService,
    blob_service: &dyn BlobService,
    state: &mut LiveCastoreState,
) -> Result<(), Error> {
    assert!(!root_digest.as_ref().is_empty(), "directory digest must not be empty");

    if state.directories.contains_key(&root_digest) {
        return Ok(());
    }

    const MAX_DIR_ENTRIES: usize = 1_000_000;
    let mut stream = directory_service.get_recursive(&root_digest);
    let mut has_seen_root_digest = false;
    for _ in 0..MAX_DIR_ENTRIES {
        let Some(item) = stream.next().await else { break };
        let directory =
            item.map_err(|err| Error::Gc(format!("reading directory {}: {err}", encode_digest(&root_digest))))?;
        let digest = directory.digest();
        if digest == root_digest {
            has_seen_root_digest = true;
        }
        if state.directories.insert(digest, directory.clone()).is_some() {
            continue;
        }
        for (_name, child) in directory.nodes() {
            match child {
                Node::File { digest, .. } => collect_blob_state(*digest, blob_service, state).await?,
                Node::Symlink { .. } => {}
                Node::Directory { .. } => {}
            }
        }
    }
    if !has_seen_root_digest {
        return Err(Error::Gc(format!(
            "missing directory metadata for reachable digest {}",
            encode_digest(&root_digest)
        )));
    }
    assert!(state.directories.contains_key(&root_digest), "root directory must be in state after walk");
    assert!(has_seen_root_digest, "directory stream must include the root");
    Ok(())
}

fn collect_existing_exported_outputs(dead_pathinfos: &[PathInfo], output_dir_str: &str) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::with_capacity(dead_pathinfos.len());
    for path_info in dead_pathinfos {
        let path = PathBuf::from(path_info.store_path.to_absolute_path_with_prefix(output_dir_str));
        if path.exists() || symlink_exists(&path)? {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn collect_existing_artifact_attestation_paths(
    state_dir: &Path,
    store_dir: &str,
    dead_pathinfos: &[PathInfo],
) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::with_capacity(dead_pathinfos.len());
    for path_info in dead_pathinfos {
        let path = artifact_attestation_file_path(state_dir, store_dir, &path_info.store_path);
        if path.exists() {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn collect_dead_closure_attestations(state_dir: &Path, retained_roots: &[GcRootRecord]) -> Result<Vec<PathBuf>, Error> {
    assert!(
        retained_roots.iter().all(|root| !root.logical_path.is_empty()),
        "gc root logical paths must not be empty"
    );

    let live_roots: BTreeSet<String> = retained_roots.iter().map(|root| root.logical_path.clone()).collect();
    let closure_dir = state_dir.join("attestations").join("closures");
    let mut dead_paths = Vec::with_capacity(64);
    if !closure_dir.exists() {
        return Ok(dead_paths);
    }
    let mut scanned_entries: u32 = 0;
    for entry in
        std::fs::read_dir(&closure_dir).map_err(|err| Error::Gc(format!("reading {}: {err}", closure_dir.display())))?
    {
        let entry = entry.map_err(|err| Error::Gc(format!("reading {} entry: {err}", closure_dir.display())))?;
        scanned_entries = scanned_entries.saturating_add(1);
        if scanned_entries > MAX_GC_FILE_SCAN_ENTRIES {
            return Err(Error::Gc(format!("closure attestation scan exceeded {} entries", MAX_GC_FILE_SCAN_ENTRIES)));
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let closure: crunch_attestation::ClosureAttestation = match serde_json::from_slice(&bytes) {
            Ok(closure) => closure,
            Err(_) => continue,
        };
        let is_closure_live = closure
            .facts
            .root_node_ids
            .iter()
            .filter_map(|node_id| node_id.strip_prefix("artifact:"))
            .all(|logical_path| live_roots.contains(logical_path));
        if !is_closure_live {
            dead_paths.push(path);
        }
    }
    // dead_paths.len() fits in u64 on any platform where usize <= u64.
    let dead_count = u64::try_from(dead_paths.len());
    debug_assert!(dead_count.is_ok(), "dead_paths.len() overflows u64");
    if let Ok(n) = dead_count {
        assert!(n <= u64::from(scanned_entries), "dead paths cannot exceed scanned entries");
    }
    Ok(dead_paths)
}

fn collect_dead_blob_files(
    state_dir: &Path,
    live_digests: &HashSet<B3Digest>,
    metadata_files: bool,
) -> Result<Vec<PathBuf>, Error> {
    let root = if metadata_files {
        state_dir.join("blobs").join("blobs").join("b3")
    } else {
        state_dir.join("blobs").join("chunks").join("b3")
    };
    let mut dead_paths = Vec::with_capacity(256);
    if !root.exists() {
        return Ok(dead_paths);
    }
    let mut scanned_entries: u32 = 0;
    scan_blob_dir(&root, &mut scanned_entries, live_digests, &mut dead_paths)?;
    Ok(dead_paths)
}

fn scan_blob_dir(
    root: &Path,
    scanned_entries: &mut u32,
    live_digests: &HashSet<B3Digest>,
    dead_paths: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    assert!(root.is_absolute(), "scan_blob_dir: root must be absolute");
    let dead_count_before = dead_paths.len();

    const MAX_SCAN_DEPTH: usize = 8;
    let mut worklist = Vec::with_capacity(MAX_SCAN_DEPTH);
    worklist.push(root.to_path_buf());

    while let Some(current_dir) = worklist.pop() {
        assert!(worklist.len() < MAX_SCAN_DEPTH, "blob scan directory nesting exceeded {MAX_SCAN_DEPTH}");
        for entry in std::fs::read_dir(&current_dir)
            .map_err(|err| Error::Gc(format!("reading {}: {err}", current_dir.display())))?
        {
            let entry =
                entry.map_err(|err| Error::Gc(format!("reading {} entry: {err}", current_dir.display())))?;
            *scanned_entries = scanned_entries.saturating_add(1);
            if *scanned_entries > MAX_GC_FILE_SCAN_ENTRIES {
                return Err(Error::Gc(format!("blob scan exceeded {} entries", MAX_GC_FILE_SCAN_ENTRIES)));
            }
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|err| Error::Gc(format!("reading {} file type: {err}", path.display())))?;
            if file_type.is_dir() {
                worklist.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.len() != B3Digest::LENGTH.saturating_mul(2) {
                continue;
            };
            let Ok(decoded) = HEXLOWER.decode(name.as_bytes()) else {
                continue;
            };
            let Ok(digest) = B3Digest::try_from(decoded) else {
                continue;
            };
            if !live_digests.contains(&digest) {
                dead_paths.push(path);
            }
        }
    }
    assert!(dead_paths.len() >= dead_count_before, "scan can only grow dead_paths, never shrink");
    Ok(())
}

fn compute_reclaimable_bytes(
    orphaned_on_disk: &[PathBuf],
    artifact_attestation_paths: &[PathBuf],
    closure_attestation_paths: &[PathBuf],
    blob_index_paths: &[PathBuf],
    blob_chunk_paths: &[PathBuf],
) -> Result<u64, Error> {
    let mut total: u64 = 0;
    for path in orphaned_on_disk
        .iter()
        .chain(artifact_attestation_paths.iter())
        .chain(closure_attestation_paths.iter())
        .chain(blob_index_paths.iter())
        .chain(blob_chunk_paths.iter())
    {
        total = total
            .checked_add(path_size_bytes(path)?)
            .ok_or_else(|| Error::Gc("reclaimable byte total overflowed u64".to_string()))?;
    }
    Ok(total)
}

fn path_size_bytes(root: &Path) -> Result<u64, Error> {
    assert!(root.is_absolute(), "path_size_bytes: root must be absolute");
    const MAX_WORKLIST: usize = 256;
    let mut total: u64 = 0;
    let mut seen_entries: u32 = 0;
    let mut worklist = Vec::with_capacity(16);
    worklist.push(root.to_path_buf());

    while let Some(current) = worklist.pop() {
        assert!(
            worklist.len() < MAX_WORKLIST,
            "path walk directory nesting exceeded {MAX_WORKLIST}"
        );
        seen_entries = seen_entries.saturating_add(1);
        if seen_entries > MAX_GC_BYTES_WALK_ENTRIES {
            return Err(Error::Gc(format!("path walk exceeded {} entries", MAX_GC_BYTES_WALK_ENTRIES)));
        }

        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|err| Error::Gc(format!("reading metadata for {}: {err}", current.display())))?;
        if metadata.file_type().is_symlink() || metadata.is_file() {
            total = total
                .checked_add(metadata.len())
                .ok_or_else(|| Error::Gc(format!("byte total overflowed while walking {}", root.display())))?;
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }

        for entry in std::fs::read_dir(&current)
            .map_err(|err| Error::Gc(format!("reading {}: {err}", current.display())))?
        {
            let entry =
                entry.map_err(|err| Error::Gc(format!("reading {} entry: {err}", current.display())))?;
            worklist.push(entry.path());
        }
    }
    Ok(total)
}

fn remove_exported_outputs(paths: &[PathBuf]) -> Result<(), Error> {
    for path in paths {
        remove_path(path)?;
    }
    Ok(())
}

fn remove_files(paths: &[PathBuf]) -> Result<(), Error> {
    for path in paths {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(Error::Gc(format!("removing {}: {err}", path.display()))),
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<(), Error> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(Error::Gc(format!("reading metadata for {}: {err}", path.display()))),
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        std::fs::remove_file(path).map_err(|err| Error::Gc(format!("removing {}: {err}", path.display())))?;
        return Ok(());
    }
    if metadata.is_dir() {
        std::fs::remove_dir_all(path).map_err(|err| Error::Gc(format!("removing {}: {err}", path.display())))?;
    }
    Ok(())
}

async fn rewrite_pathinfo_db(state_dir: &Path, live_pathinfos: &[PathInfo]) -> Result<(), Error> {
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

    assert!(state_dir.is_dir(), "rewrite_pathinfo_db: state_dir must exist");
    assert!(
        live_pathinfos.iter().all(|pi| !pi.signatures.is_empty()),
        "all retained PathInfos must be signed"
    );

    let final_path = state_dir.join("pathinfo.redb");
    let tmp_path = state_dir.join("pathinfo.redb.gc-tmp");
    if tmp_path.exists() {
        if let Err(err) = std::fs::remove_file(&tmp_path) {
            tracing::debug!(path = %tmp_path.display(), err = %err, "stale gc-tmp pathinfo already cleaned");
        }
    }
    let tmp_service = RedbPathInfoService::new("crunch-gc-pathinfo".to_string(), RedbPathInfoServiceConfig {
        path: Some(tmp_path.clone()),
        read_only: false,
        cache_size: None,
    })
    .await
    .map_err(|err| Error::Gc(format!("opening {}: {err}", tmp_path.display())))?;
    for path_info in live_pathinfos {
        tmp_service
            .put(path_info.clone())
            .await
            .map_err(|err| Error::Gc(format!("writing rewritten PathInfo DB: {err}")))?;
    }
    drop(tmp_service);
    std::fs::rename(&tmp_path, &final_path)
        .map_err(|err| Error::Gc(format!("renaming {} -> {}: {err}", tmp_path.display(), final_path.display())))?;
    Ok(())
}

async fn rewrite_directory_db<'a>(
    state_dir: &Path,
    live_directories: impl Iterator<Item = &'a Directory>,
) -> Result<(), Error> {
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    assert!(state_dir.is_dir(), "rewrite_directory_db: state_dir must exist");

    let final_path = state_dir.join("directories.redb");
    let tmp_path = state_dir.join("directories.redb.gc-tmp");
    if tmp_path.exists() {
        if let Err(err) = std::fs::remove_file(&tmp_path) {
            tracing::debug!(path = %tmp_path.display(), err = %err, "stale gc-tmp directories already cleaned");
        }
    }
    let tmp_service = RedbDirectoryService::new("crunch-gc-directories".to_string(), RedbDirectoryServiceConfig {
        path: Some(tmp_path.clone()),
        read_only: false,
        cache_size: None,
    })
    .await
    .map_err(|err| Error::Gc(format!("opening {}: {err}", tmp_path.display())))?;
    for directory in live_directories {
        tmp_service
            .put(directory.clone())
            .await
            .map_err(|err| Error::Gc(format!("writing rewritten directory DB: {err}")))?;
    }
    drop(tmp_service);
    assert!(
        tmp_path.exists(),
        "rewrite_directory_db: tmp file must exist before rename"
    );
    std::fs::rename(&tmp_path, &final_path)
        .map_err(|err| Error::Gc(format!("renaming {} -> {}: {err}", tmp_path.display(), final_path.display())))?;
    Ok(())
}

fn encode_digest(digest: &B3Digest) -> String {
    HEXLOWER.encode(digest.as_ref())
}

fn symlink_exists(path: &Path) -> Result<bool, Error> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(Error::Gc(format!("reading metadata for {}: {err}", path.display()))),
    }
}

#[allow(tigerstyle::sentinel_fallback)]
pub(crate) fn saturating_u32(len: usize) -> u32 {
    // Intentional saturation: signature/scan counts above u32::MAX are clamped.
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use futures::StreamExt as _;
    use futures::stream::BoxStream;
    use pretty_assertions::assert_eq;
    use snix_castore::SymlinkTarget;
    use snix_store::pathinfoservice;
    use tokio::io::AsyncWriteExt;
    use tokio_stream::wrappers::ReceiverStream;

    use super::*;
    use crate::StoreConfig;
    use crate::StoreHandle;
    use crate::handle::PersistOutputRequest;
    use crate::roots::GcRootSource;

    fn store_path(name: &str, seed: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [seed; 20]).unwrap()
    }

    async fn open_store(state_dir: &Path, output_dir: &Path) -> StoreHandle {
        StoreHandle::open(StoreConfig {
            state_dir: state_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            remote_cache_url: None,
            fallback_mode: crate::StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
        })
        .await
        .unwrap()
    }

    async fn write_blob(store: &StoreHandle, contents: &[u8]) -> Node {
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(contents).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: contents.len() as u64,
            executable: false,
        }
    }

    async fn write_directory_with_file(store: &StoreHandle, name: &str, contents: &[u8]) -> Node {
        let child = write_blob(store, contents).await;
        let directory = Directory::try_from_iter([(name.try_into().unwrap(), child)]).unwrap();
        let digest = store.directory_service().put(directory.clone()).await.unwrap();
        Node::Directory {
            digest,
            size: directory.size(),
        }
    }

    fn test_signature() -> nix_compat::narinfo::Signature<String> {
        let signing_key = nix_compat::narinfo::SigningKey::new(
            "gc-test-1".to_string(),
            ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]),
        );
        signing_key.sign(b"gc-signed").to_owned()
    }

    fn signed_pathinfo(
        store_path: StorePath<String>,
        node: Node,
        refs: Vec<StorePath<String>>,
    ) -> snix_store::path_info::PathInfo {
        snix_store::path_info::PathInfo {
            store_path,
            node,
            references: refs,
            nar_size: 1,
            nar_sha256: [7u8; 32],
            signatures: vec![test_signature()],
            deriver: None,
            ca: None,
        }
    }

    async fn persist_output(
        store: &mut StoreHandle,
        output_name: &str,
        store_path: StorePath<String>,
        node: Node,
        refs: Vec<StorePath<String>>,
        is_root: bool,
        source: Option<GcRootSource>,
    ) {
        let path_info = signed_pathinfo(store_path.clone(), node.clone(), refs);
        store
            .persist_and_export_signed_output(PersistOutputRequest {
                output_name,
                output_path: &store_path,
                path_info,
                final_node: node,
                provenance: None,
                is_root,
                root_source: source,
            })
            .await
            .unwrap();
    }

    fn blob_chunk_path(state_dir: &Path, digest: &B3Digest) -> PathBuf {
        state_dir
            .join("blobs")
            .join("chunks")
            .join("b3")
            .join(HEXLOWER.encode(&digest.as_ref()[..2]))
            .join(HEXLOWER.encode(digest.as_ref()))
    }

    async fn reopen_store(state_dir: &Path, output_dir: &Path) -> StoreHandle {
        open_store(state_dir, output_dir).await
    }

    #[tokio::test]
    async fn dry_run_reports_same_candidates_as_real_run() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let keep_path = store_path("keep", 1);
        let drop_path = store_path("drop", 2);
        let keep_node = write_blob(&store, b"keep").await;
        let drop_node = write_blob(&store, b"drop").await;
        persist_output(&mut store, "out", keep_path.clone(), keep_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;

        let is_dry_run = store.garbage_collect(true).await.unwrap();
        assert_eq!(is_dry_run.candidate_paths, vec![drop_path.to_absolute_path()]);
        assert!(output_dir.path().join(drop_path.to_string()).exists());

        let real = store.garbage_collect(false).await.unwrap();
        assert_eq!(real.candidate_paths, is_dry_run.candidate_paths);

        let reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let listed = crate::store_list(reopened.pathinfo_service().as_ref()).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0, keep_path.to_string());
    }

    #[tokio::test]
    async fn retained_root_keeps_transitive_closure_and_sidecars() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let dep_path = store_path("dep", 3);
        let root_path = store_path("root", 4);
        let dep_node = write_blob(&store, b"dep-bytes").await;
        let root_node = write_blob(&store, b"root-bytes").await;

        persist_output(&mut store, "out", dep_path.clone(), dep_node, vec![], false, None).await;
        persist_output(
            &mut store,
            "out",
            root_path.clone(),
            root_node,
            vec![dep_path.clone()],
            true,
            Some(GcRootSource::Build),
        )
        .await;
        let closure_path = crate::closure_attestation_file_path(
            state_dir.path(),
            "/nix/store",
            std::slice::from_ref(&root_path),
            crunch_attestation::ClosureSemantics::Runtime,
        );
        store.runtime_closure_attestation(std::slice::from_ref(&root_path)).await.unwrap();
        assert!(closure_path.exists());

        let report = store.garbage_collect(false).await.unwrap();
        assert_eq!(report.candidate_path_count, 0);
        assert!(closure_path.exists());
        assert!(crate::artifact_attestation_file_path(state_dir.path(), "/nix/store", &dep_path).exists());
        assert!(crate::artifact_attestation_file_path(state_dir.path(), "/nix/store", &root_path).exists());

        let reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let listed = crate::store_list(reopened.pathinfo_service().as_ref()).await.unwrap();
        assert_eq!(listed.len(), 2);
    }

    #[tokio::test]
    async fn unreachable_output_removes_pathinfo_exports_and_attestations() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("root", 5);
        let drop_path = store_path("drop", 6);
        let root_node = write_blob(&store, b"root").await;
        let drop_node = write_blob(&store, b"drop").await;

        persist_output(&mut store, "out", root_path.clone(), root_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;
        let closure_path = crate::closure_attestation_file_path(
            state_dir.path(),
            "/nix/store",
            std::slice::from_ref(&drop_path),
            crunch_attestation::ClosureSemantics::Runtime,
        );
        store.runtime_closure_attestation(std::slice::from_ref(&drop_path)).await.unwrap();
        assert!(closure_path.exists());
        let export_path = output_dir.path().join(drop_path.to_string());
        assert!(export_path.exists());

        let report = store.garbage_collect(false).await.unwrap();
        assert_eq!(report.candidate_paths, vec![drop_path.to_absolute_path()]);
        assert!(!export_path.exists());
        assert!(!crate::artifact_attestation_file_path(state_dir.path(), "/nix/store", &drop_path).exists());
        assert!(!closure_path.exists());
    }

    #[tokio::test]
    async fn shared_blob_survives_when_reachable_path_still_references_it() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let shared_node = write_blob(&store, b"shared").await;
        let shared_digest = match &shared_node {
            Node::File { digest, .. } => *digest,
            _ => panic!("expected file node"),
        };
        let root_path = store_path("root", 7);
        let drop_path = store_path("drop", 8);

        persist_output(
            &mut store,
            "out",
            root_path.clone(),
            shared_node.clone(),
            vec![],
            true,
            Some(GcRootSource::Build),
        )
        .await;
        persist_output(&mut store, "out", drop_path.clone(), shared_node, vec![], true, None).await;
        let chunk_path = blob_chunk_path(state_dir.path(), &shared_digest);
        assert!(chunk_path.exists());

        store.garbage_collect(false).await.unwrap();

        assert!(chunk_path.exists());
    }

    #[tokio::test]
    async fn gc_aborts_when_retained_root_pathinfo_is_missing() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("root", 9);
        let root_node = write_blob(&store, b"root").await;
        persist_output(&mut store, "out", root_path.clone(), root_node, vec![], true, Some(GcRootSource::Build)).await;
        drop(store);
        std::fs::remove_file(state_dir.path().join("pathinfo.redb")).unwrap();

        let mut reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let err = reopened.garbage_collect(true).await.unwrap_err();
        assert!(matches!(err, Error::MissingClosureFacts { .. }));
    }

    #[tokio::test]
    async fn gc_aborts_when_root_registry_is_corrupt() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        std::fs::write(state_dir.path().join("gc-roots.json"), b"not json").unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;

        let err = store.garbage_collect(true).await.unwrap_err();
        assert!(matches!(err, Error::RootRegistry(_)));
    }

    #[tokio::test]
    async fn gc_operation_order_matches_design() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("root", 10);
        let drop_path = store_path("drop", 11);
        let root_node = write_blob(&store, b"root").await;
        let drop_node = write_blob(&store, b"drop").await;
        persist_output(&mut store, "out", root_path, root_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;

        let report = store.garbage_collect(false).await.unwrap();

        assert_eq!(report.operations, vec![
            GcOperationKind::ExportedOutputs,
            GcOperationKind::PathInfoRewrite,
            GcOperationKind::ArtifactAttestations,
            GcOperationKind::ClosureAttestations,
            GcOperationKind::DirectoryRewrite,
            GcOperationKind::BlobIndexFiles,
            GcOperationKind::BlobChunkFiles,
            GcOperationKind::CaMappings,
        ]);
    }

    #[tokio::test]
    async fn directory_outputs_survive_reopen_and_gc() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let dir_path = store_path("dir-root", 12);
        let dir_node = write_directory_with_file(&store, "hello.txt", b"hello").await;
        persist_output(&mut store, "out", dir_path.clone(), dir_node, vec![], true, Some(GcRootSource::Build)).await;

        drop(store);
        let mut reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let report = reopened.garbage_collect(false).await.unwrap();
        assert_eq!(report.candidate_path_count, 0);
        assert!(output_dir.path().join(dir_path.to_string()).join("hello.txt").exists());
    }

    #[derive(Clone, Default)]
    struct SnapshotGuardPathInfoService {
        entries: Arc<std::sync::Mutex<Vec<PathInfo>>>,
        listing_active: Arc<std::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl pathinfoservice::PathInfoService for SnapshotGuardPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            Ok(self.entries.lock().unwrap().iter().find(|entry| *entry.store_path.digest() == digest).cloned())
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
            if *self.listing_active.lock().unwrap() {
                return Err(std::io::Error::other("mutation while listing").into());
            }
            self.entries.lock().unwrap().push(path_info.clone());
            Ok(path_info)
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
            *self.listing_active.lock().unwrap() = true;
            let entries = self.entries.lock().unwrap().clone();
            let active = self.listing_active.clone();
            let (tx, rx) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                for entry in entries {
                    if tx.send(Ok(entry)).await.is_err() {
                        break;
                    }
                }
                *active.lock().unwrap() = false;
            });
            ReceiverStream::new(rx).boxed()
        }
    }

    #[tokio::test]
    async fn snapshot_pathinfos_finishes_before_later_mutation() {
        let service = SnapshotGuardPathInfoService::default();
        service.entries.lock().unwrap().push(signed_pathinfo(
            store_path("snap", 13),
            Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            vec![],
        ));

        let snapshot = snapshot_pathinfos(&service).await.unwrap();
        assert_eq!(snapshot.len(), 1);
        assert!(!*service.listing_active.lock().unwrap());
        service
            .put(signed_pathinfo(
                store_path("later", 14),
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                vec![],
            ))
            .await
            .unwrap();
    }
}
