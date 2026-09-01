//! Thin castore shell for frontend-neutral root composition.

use std::collections::HashMap;
use std::sync::Arc;

use crunch_composition_core::CastoreRootRef;
use crunch_composition_core::CompositionRequest;
use crunch_composition_core::PreparedComposition;
use crunch_composition_core::RealizationPolicy;
use crunch_composition_core::RealizationReceipt;
use crunch_composition_core::RootSnapshot;
use crunch_composition_core::SnapshotEntry;
use crunch_composition_core::SnapshotNode;
use data_encoding::HEXLOWER;
use futures::future::BoxFuture;
use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::PathComponent;
use snix_castore::SymlinkTarget;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;

use crate::Error;
use crate::StoreHandle;
use crate::recursive_castore_completeness;

const INITIAL_DIRECTORY_CAPACITY: usize = 64;
const SNAPSHOT_TASKS_PER_ENTRY_MAX: usize = 2;
const SNAPSHOT_ROOT_TASKS_MAX: usize = 2;
const ROOT_DEPTH: u32 = 0;

fn castore_root_node(root: &CastoreRootRef) -> Result<Node, Error> {
    assert!(!root.digest_blake3.is_empty());
    let root_node = Node::Directory {
        digest: parse_digest(&root.digest_blake3)?,
        size: root.size,
    };
    assert!(matches!(root_node, Node::Directory { .. }));
    Ok(root_node)
}

async fn load_checked_directory(
    directory_service: &Arc<dyn DirectoryService>,
    digest: &B3Digest,
    expected_size_bytes: u64,
) -> Result<Directory, Error> {
    let directory = directory_service
        .get(digest)
        .await
        .map_err(|error| Error::DirectoryService(format!("composition load: {error}")))?
        .ok_or_else(|| Error::Composition(format!("composition-missing-directory:{digest}")))?;
    if directory.digest() != *digest || directory.size() != expected_size_bytes {
        return Err(Error::Composition(format!("composition-inconsistent-directory:{digest}")));
    }
    assert_eq!(directory.digest(), *digest);
    assert_eq!(directory.size(), expected_size_bytes);
    Ok(directory)
}

pub fn plan_composition_request(request: &CompositionRequest) -> Result<PreparedComposition, Error> {
    crunch_composition_core::prepare_composition(request).map_err(composition_error)
}

pub async fn realize_composition(
    store: &StoreHandle,
    request: &CompositionRequest,
) -> Result<RealizationReceipt, Error> {
    let prepared = plan_composition_request(request)?;
    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    realize_with_services(blob_service, directory_service, &prepared).await
}

async fn realize_with_services(
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
    prepared: &PreparedComposition,
) -> Result<RealizationReceipt, Error> {
    let snapshots = load_input_snapshots(&blob_service, &directory_service, prepared).await?;
    let outcome = crunch_composition_core::plan_composition(prepared, snapshots).map_err(composition_error)?;
    let resulting_node = persist_snapshot_node(&directory_service, &outcome.root).await?;
    let resulting_root = root_ref_from_node(&resulting_node)?;
    recheck_result(&blob_service, &directory_service, &resulting_node).await?;
    crunch_composition_core::finalize_receipt(prepared, &outcome, resulting_root).map_err(composition_error)
}

async fn load_input_snapshots(
    blob_service: &Arc<dyn BlobService>,
    directory_service: &Arc<dyn DirectoryService>,
    prepared: &PreparedComposition,
) -> Result<Vec<RootSnapshot>, Error> {
    let mut roots = prepared.bindings.iter().map(|binding| binding.root.clone()).collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    let mut snapshots = Vec::with_capacity(roots.len());
    for root in roots {
        snapshots.push(load_root_snapshot(blob_service, directory_service, &root, &prepared.realization_policy).await?);
    }
    Ok(snapshots)
}

async fn load_root_snapshot(
    blob_service: &Arc<dyn BlobService>,
    directory_service: &Arc<dyn DirectoryService>,
    root: &CastoreRootRef,
    policy: &RealizationPolicy,
) -> Result<RootSnapshot, Error> {
    assert!(!root.digest_blake3.is_empty());
    assert!(policy.max_entries > 0);
    let root_node = castore_root_node(root)?;
    let mut directories = HashMap::with_capacity(INITIAL_DIRECTORY_CAPACITY);
    let mut stack = Vec::with_capacity(INITIAL_DIRECTORY_CAPACITY);
    stack.push((root_node.clone(), ROOT_DEPTH));
    let mut entry_count = 0_u32;
    let mut total_file_bytes = 0_u64;

    while let Some((node, depth)) = stack.pop() {
        let Node::Directory { digest, size } = node else {
            return Err(Error::Composition("composition-loader-expected-directory".to_string()));
        };
        if depth > policy.max_depth {
            return Err(Error::Composition("composition-limit-exceeded:depth".to_string()));
        }
        if directories.contains_key(&digest) {
            continue;
        }
        let directory = load_checked_directory(directory_service, &digest, size).await?;
        for (_, child) in directory.nodes() {
            entry_count = entry_count
                .checked_add(1)
                .ok_or_else(|| Error::Composition("composition-integer-overflow:entries".to_string()))?;
            if entry_count > policy.max_entries {
                return Err(Error::Composition("composition-limit-exceeded:entries".to_string()));
            }
            match child {
                Node::Directory { .. } => {
                    let child_depth = depth
                        .checked_add(1)
                        .ok_or_else(|| Error::Composition("composition-integer-overflow:depth".to_string()))?;
                    stack.push((child.clone(), child_depth));
                }
                Node::File { size, .. } => {
                    if *size > policy.max_file_bytes {
                        return Err(Error::Composition("composition-limit-exceeded:file-bytes".to_string()));
                    }
                    total_file_bytes = total_file_bytes.checked_add(*size).ok_or_else(|| {
                        Error::Composition("composition-integer-overflow:total-file-bytes".to_string())
                    })?;
                    if total_file_bytes > policy.max_total_file_bytes {
                        return Err(Error::Composition("composition-limit-exceeded:total-file-bytes".to_string()));
                    }
                    if !recursive_castore_completeness(blob_service.as_ref(), directory_service.as_ref(), child).await?
                    {
                        return Err(Error::Composition("composition-missing-blob".to_string()));
                    }
                }
                Node::Symlink { .. } if !policy.allow_symlinks => {
                    return Err(Error::Composition("composition-symlink-rejected".to_string()));
                }
                Node::Symlink { .. } => {}
            }
        }
        directories.insert(digest, directory);
    }

    let node = snapshot_from_castore(&root_node, &directories, SnapshotLimits {
        max_depth: policy.max_depth,
        max_entries: policy.max_entries,
    })?;
    Ok(RootSnapshot {
        root: root.clone(),
        node,
    })
}

#[derive(Clone, Copy)]
struct SnapshotLimits {
    max_depth: u32,
    max_entries: u32,
}

enum SnapshotTask<'a> {
    Visit {
        name: Option<String>,
        node: &'a Node,
        depth: u32,
    },
    FinishDirectory {
        name: Option<String>,
        child_count: usize,
    },
}

struct BuiltSnapshot {
    name: Option<String>,
    node: SnapshotNode,
}

struct SnapshotBuilder<'a> {
    directories: &'a HashMap<B3Digest, Directory>,
    limits: SnapshotLimits,
    maximum_task_count: usize,
    tasks: Vec<SnapshotTask<'a>>,
    built: Vec<BuiltSnapshot>,
}

impl<'a> SnapshotBuilder<'a> {
    fn new(directories: &'a HashMap<B3Digest, Directory>, limits: SnapshotLimits) -> Result<Self, Error> {
        assert!(!directories.is_empty());
        assert!(limits.max_entries > 0);
        let maximum_entry_count = usize::try_from(limits.max_entries)
            .map_err(|_| Error::Composition("composition-entry-limit-exceeds-usize".to_string()))?;
        let maximum_task_count = maximum_entry_count
            .checked_mul(SNAPSHOT_TASKS_PER_ENTRY_MAX)
            .and_then(|count| count.checked_add(SNAPSHOT_ROOT_TASKS_MAX))
            .ok_or_else(|| Error::Composition("composition-integer-overflow:snapshot-tasks".to_string()))?;
        Ok(Self {
            directories,
            limits,
            maximum_task_count,
            tasks: Vec::with_capacity(INITIAL_DIRECTORY_CAPACITY),
            built: Vec::with_capacity(INITIAL_DIRECTORY_CAPACITY),
        })
    }

    fn build(mut self, root: &'a Node) -> Result<SnapshotNode, Error> {
        assert!(self.tasks.is_empty());
        assert!(self.built.is_empty());
        self.tasks.push(SnapshotTask::Visit {
            name: None,
            node: root,
            depth: ROOT_DEPTH,
        });
        for _task_index in 0..self.maximum_task_count {
            let Some(task) = self.tasks.pop() else {
                break;
            };
            match task {
                SnapshotTask::Visit { name, node, depth } => self.visit(name, node, depth)?,
                SnapshotTask::FinishDirectory { name, child_count } => self.finish_directory(name, child_count)?,
            }
        }
        if !self.tasks.is_empty() {
            return Err(Error::Composition("composition-limit-exceeded:snapshot-tasks".to_string()));
        }
        if self.built.len() != 1 {
            return Err(Error::Composition("composition-invalid-snapshot-stack".to_string()));
        }
        assert!(self.tasks.is_empty());
        assert_eq!(self.built.len(), 1);
        let result = self.built.pop().ok_or_else(|| Error::Composition("composition-snapshot-missing".to_string()))?;
        if result.name.is_some() {
            return Err(Error::Composition("composition-root-name-unexpected".to_string()));
        }
        Ok(result.node)
    }

    fn visit(&mut self, name: Option<String>, node: &'a Node, depth: u32) -> Result<(), Error> {
        if depth > self.limits.max_depth {
            return Err(Error::Composition("composition-limit-exceeded:depth".to_string()));
        }
        assert!(depth <= self.limits.max_depth);
        assert!(self.built.len() <= self.maximum_task_count);
        match node {
            Node::File {
                digest,
                size,
                executable,
            } => self.built.push(BuiltSnapshot {
                name,
                node: SnapshotNode::File {
                    digest_blake3: digest_hex(digest),
                    size_bytes: *size,
                    executable: *executable,
                },
            }),
            Node::Symlink { target } => self.built.push(BuiltSnapshot {
                name,
                node: SnapshotNode::Symlink {
                    target: target.as_ref().to_vec(),
                },
            }),
            Node::Directory { digest, .. } => self.schedule_directory(name, digest, depth)?,
        }
        Ok(())
    }

    fn schedule_directory(&mut self, name: Option<String>, digest: &B3Digest, depth: u32) -> Result<(), Error> {
        assert!(depth <= self.limits.max_depth);
        assert!(self.tasks.len() <= self.maximum_task_count);
        let directory = self
            .directories
            .get(digest)
            .ok_or_else(|| Error::Composition(format!("composition-missing-directory:{digest}")))?;
        let child_count = directory.nodes().count();
        let additional_task_count = child_count
            .checked_add(1)
            .ok_or_else(|| Error::Composition("composition-integer-overflow:snapshot-tasks".to_string()))?;
        let scheduled_task_count = self
            .tasks
            .len()
            .checked_add(additional_task_count)
            .ok_or_else(|| Error::Composition("composition-integer-overflow:snapshot-tasks".to_string()))?;
        if scheduled_task_count > self.maximum_task_count {
            return Err(Error::Composition("composition-limit-exceeded:snapshot-tasks".to_string()));
        }
        let mut children = Vec::with_capacity(child_count);
        for (child_name, child_node) in directory.nodes() {
            let child_name = std::str::from_utf8(child_name.as_ref())
                .map_err(|_| Error::Composition("composition-non-utf8-entry-name".to_string()))?
                .to_string();
            let child_depth = depth
                .checked_add(1)
                .ok_or_else(|| Error::Composition("composition-integer-overflow:depth".to_string()))?;
            children.push((child_name, child_node, child_depth));
        }
        assert_eq!(children.len(), child_count);
        assert!(scheduled_task_count <= self.maximum_task_count);
        self.tasks.push(SnapshotTask::FinishDirectory { name, child_count });
        for (child_name, child_node, child_depth) in children.into_iter().rev() {
            self.tasks.push(SnapshotTask::Visit {
                name: Some(child_name),
                node: child_node,
                depth: child_depth,
            });
        }
        Ok(())
    }

    fn finish_directory(&mut self, name: Option<String>, child_count: usize) -> Result<(), Error> {
        if self.built.len() < child_count {
            return Err(Error::Composition("composition-invalid-snapshot-stack".to_string()));
        }
        assert!(self.built.len() >= child_count);
        assert!(child_count <= self.maximum_task_count);
        let first_child_index = self
            .built
            .len()
            .checked_sub(child_count)
            .ok_or_else(|| Error::Composition("composition-invalid-snapshot-stack".to_string()))?;
        let children = self.built.split_off(first_child_index);
        let mut entries = Vec::with_capacity(child_count);
        for child in children {
            let child_name =
                child.name.ok_or_else(|| Error::Composition("composition-child-name-missing".to_string()))?;
            entries.push(SnapshotEntry {
                name: child_name,
                node: child.node,
            });
        }
        assert_eq!(entries.len(), child_count);
        self.built.push(BuiltSnapshot {
            name,
            node: SnapshotNode::Directory { entries },
        });
        Ok(())
    }
}

fn snapshot_from_castore(
    node: &Node,
    directories: &HashMap<B3Digest, Directory>,
    limits: SnapshotLimits,
) -> Result<SnapshotNode, Error> {
    SnapshotBuilder::new(directories, limits)?.build(node)
}

fn persist_snapshot_node<'a>(
    directory_service: &'a Arc<dyn DirectoryService>,
    node: &'a SnapshotNode,
) -> BoxFuture<'a, Result<Node, Error>> {
    Box::pin(async move {
        match node {
            SnapshotNode::File {
                digest_blake3,
                size_bytes,
                executable,
            } => Ok(Node::File {
                digest: parse_digest(digest_blake3)?,
                size: *size_bytes,
                executable: *executable,
            }),
            SnapshotNode::Symlink { target } => Ok(Node::Symlink {
                target: SymlinkTarget::try_from(tokio_util::bytes::Bytes::copy_from_slice(target))
                    .map_err(|error| Error::Composition(format!("composition-invalid-symlink-target:{error}")))?,
            }),
            SnapshotNode::Directory { entries } => {
                let mut castore_entries = Vec::with_capacity(entries.len());
                for entry in entries {
                    let name = PathComponent::try_from(entry.name.as_str())
                        .map_err(|error| Error::Composition(format!("composition-invalid-entry-name:{error}")))?;
                    let child = persist_snapshot_node(directory_service, &entry.node).await?;
                    castore_entries.push((name, child));
                }
                let directory = Directory::try_from_iter(castore_entries)
                    .map_err(|error| Error::Composition(format!("composition-invalid-directory:{error}")))?;
                let expected_digest = directory.digest();
                let size_bytes = directory.size();
                let observed_digest = directory_service
                    .put(directory)
                    .await
                    .map_err(|error| Error::DirectoryService(format!("composition persist: {error}")))?;
                if observed_digest != expected_digest {
                    return Err(Error::Composition("composition-persist-digest-mismatch".to_string()));
                }
                Ok(Node::Directory {
                    digest: expected_digest,
                    size: size_bytes,
                })
            }
        }
    })
}

async fn recheck_result(
    blob_service: &Arc<dyn BlobService>,
    directory_service: &Arc<dyn DirectoryService>,
    root: &Node,
) -> Result<(), Error> {
    let Node::Directory { digest, size } = root else {
        return Err(Error::Composition("composition-result-not-directory".to_string()));
    };
    let directory = directory_service
        .get(digest)
        .await
        .map_err(|error| Error::DirectoryService(format!("composition recheck: {error}")))?
        .ok_or_else(|| Error::Composition("composition-root-recheck-missing".to_string()))?;
    if directory.digest() != *digest || directory.size() != *size {
        return Err(Error::Composition("composition-root-recheck-mismatch".to_string()));
    }
    if !recursive_castore_completeness(blob_service.as_ref(), directory_service.as_ref(), root).await? {
        return Err(Error::Composition("composition-root-recheck-incomplete".to_string()));
    }
    Ok(())
}

fn root_ref_from_node(node: &Node) -> Result<CastoreRootRef, Error> {
    let Node::Directory { digest, size } = node else {
        return Err(Error::Composition("composition-result-not-directory".to_string()));
    };
    Ok(CastoreRootRef {
        digest_blake3: digest_hex(digest),
        size: *size,
    })
}

fn digest_hex(digest: &B3Digest) -> String {
    HEXLOWER.encode(digest.as_slice())
}

fn parse_digest(value: &str) -> Result<B3Digest, Error> {
    let bytes = HEXLOWER
        .decode(value.as_bytes())
        .map_err(|_| Error::Composition("composition-invalid-digest".to_string()))?;
    let bytes: [u8; B3Digest::LENGTH] =
        bytes.try_into().map_err(|_| Error::Composition("composition-invalid-digest".to_string()))?;
    Ok(B3Digest::from(&bytes))
}

fn composition_error(error: crunch_composition_core::CompositionError) -> Error {
    Error::Composition(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use crunch_composition_core::CompositionBinding;
    use crunch_composition_core::CompositionPlan;
    use crunch_composition_core::MERGE_POLICY_VERSION;
    use crunch_composition_core::PLAN_SCHEMA;
    use crunch_composition_core::POLICY_SCHEMA;
    use futures::stream::BoxStream;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryPutter;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use tokio::io::AsyncWriteExt;

    use super::*;

    const TEST_LIMIT: u32 = 64;
    const TEST_BYTES_LIMIT: u64 = 1_024;

    fn policy() -> RealizationPolicy {
        RealizationPolicy {
            schema: POLICY_SCHEMA.to_string(),
            max_bindings: TEST_LIMIT,
            max_collision_decisions: TEST_LIMIT,
            max_entries: TEST_LIMIT,
            max_depth: TEST_LIMIT,
            max_path_bytes: TEST_LIMIT,
            max_file_bytes: TEST_BYTES_LIMIT,
            max_total_file_bytes: TEST_BYTES_LIMIT,
            allow_symlinks: true,
        }
    }

    fn request(roots: &[CastoreRootRef]) -> CompositionRequest {
        CompositionRequest {
            plan: CompositionPlan {
                schema: PLAN_SCHEMA.to_string(),
                merge_policy_version: MERGE_POLICY_VERSION,
                bindings: roots
                    .iter()
                    .enumerate()
                    .map(|(index, root)| CompositionBinding {
                        root: root.clone(),
                        mount: if index == 0 { String::new() } else { "usr".to_string() },
                        label: Some(format!("root-{index}")),
                    })
                    .collect(),
                collision_decisions: Vec::new(),
            },
            realization_policy: policy(),
        }
    }

    #[derive(Clone, Copy)]
    enum DirectoryFault {
        RejectPut,
        HidePut,
        CorruptRead,
    }

    struct FaultDirectoryService {
        inner: Arc<dyn DirectoryService>,
        fault: DirectoryFault,
        hidden_digest: Mutex<Option<B3Digest>>,
    }

    impl FaultDirectoryService {
        fn new(inner: Arc<dyn DirectoryService>, fault: DirectoryFault) -> Self {
            Self {
                inner,
                fault,
                hidden_digest: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl DirectoryService for FaultDirectoryService {
        async fn get(&self, digest: &B3Digest) -> Result<Option<Directory>, snix_castore::directoryservice::Error> {
            if matches!(self.fault, DirectoryFault::HidePut)
                && self.hidden_digest.lock().unwrap().as_ref() == Some(digest)
            {
                return Ok(None);
            }
            if matches!(self.fault, DirectoryFault::CorruptRead) {
                return Ok(Some(Directory::new()));
            }
            self.inner.get(digest).await
        }

        async fn put(&self, directory: Directory) -> Result<B3Digest, snix_castore::directoryservice::Error> {
            if matches!(self.fault, DirectoryFault::RejectPut) {
                return Err(std::io::Error::other("composition-test-persist-failure").into());
            }
            let digest = self.inner.put(directory).await?;
            if matches!(self.fault, DirectoryFault::HidePut) {
                *self.hidden_digest.lock().unwrap() = Some(digest);
            }
            Ok(digest)
        }

        fn get_recursive(
            &self,
            root_directory_digest: &B3Digest,
        ) -> BoxStream<'_, Result<Directory, snix_castore::directoryservice::Error>> {
            self.inner.get_recursive(root_directory_digest)
        }

        fn put_multiple_start(&self) -> Box<dyn DirectoryPutter + '_> {
            self.inner.put_multiple_start()
        }
    }

    async fn test_store() -> (tempfile::TempDir, StoreHandle) {
        let root = tempfile::tempdir().unwrap();
        let store = StoreHandle::open(crate::StoreConfig {
            state_dir: root.path().join("state"),
            output_dir: root.path().join("store"),
            remote_cache_urls: Vec::new(),
            base_state_dirs: Vec::new(),
            fallback_mode: crate::StoreFallbackMode::Strict,
            store_dir: "/mantle/store".to_string(),
        })
        .await
        .unwrap();
        (root, store)
    }

    async fn input_root(store: &StoreHandle, name: &str, bytes: &[u8]) -> CastoreRootRef {
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(bytes).await.unwrap();
        let digest = writer.close().await.unwrap();
        let directory = Directory::try_from_iter([(PathComponent::try_from(name).unwrap(), Node::File {
            digest,
            size: u64::try_from(bytes.len()).unwrap(),
            executable: false,
        })])
        .unwrap();
        let size = directory.size();
        let digest = store.directory_service().put(directory).await.unwrap();
        CastoreRootRef {
            digest_blake3: digest_hex(&digest),
            size,
        }
    }

    async fn service_root(
        blob_service: &MemoryBlobService,
        directory_service: &Arc<dyn DirectoryService>,
    ) -> CastoreRootRef {
        let bytes = b"fault-test";
        let mut writer = blob_service.open_write().await;
        writer.write_all(bytes).await.unwrap();
        let blob_digest = writer.close().await.unwrap();
        let directory = Directory::try_from_iter([(PathComponent::try_from("file").unwrap(), Node::File {
            digest: blob_digest,
            size: u64::try_from(bytes.len()).unwrap(),
            executable: false,
        })])
        .unwrap();
        let size = directory.size();
        let digest = directory_service.put(directory).await.unwrap();
        CastoreRootRef {
            digest_blake3: digest_hex(&digest),
            size,
        }
    }

    fn temporary_directory_service(label: &str) -> Arc<dyn DirectoryService> {
        Arc::new(RedbDirectoryService::new_temporary(label.to_string(), RedbDirectoryServiceConfig::default()).unwrap())
    }

    #[tokio::test]
    async fn complete_multi_root_realization_is_repeatable_and_order_independent() {
        let (_root, store) = test_store().await;
        let first_root = input_root(&store, "etc", b"one").await;
        let second_root = input_root(&store, "bin", b"two").await;
        let first_request = request(&[first_root.clone(), second_root.clone()]);
        let mut second_request = first_request.clone();
        second_request.plan.bindings.reverse();

        let first = realize_composition(&store, &first_request).await.unwrap();
        let repeated = realize_composition(&store, &first_request).await.unwrap();
        let reordered = realize_composition(&store, &second_request).await.unwrap();

        assert_eq!(first.receipt_ref, repeated.receipt_ref);
        assert_eq!(first.resulting_root, repeated.resulting_root);
        assert_eq!(first.resulting_root, reordered.resulting_root);
    }

    #[tokio::test]
    async fn missing_root_and_blob_fail_without_receipt() {
        let (_root, store) = test_store().await;
        let missing_root = CastoreRootRef {
            digest_blake3: "f".repeat(crunch_composition_core::BLAKE3_HEX_CHARS),
            size: 1,
        };
        let error = realize_composition(&store, &request(&[missing_root])).await.unwrap_err();
        assert!(error.to_string().contains("composition-missing-directory"));

        let fake_blob = B3Digest::from(&[7_u8; B3Digest::LENGTH]);
        let directory = Directory::try_from_iter([(PathComponent::try_from("missing").unwrap(), Node::File {
            digest: fake_blob,
            size: 1,
            executable: false,
        })])
        .unwrap();
        let size = directory.size();
        let digest = store.directory_service().put(directory).await.unwrap();
        let error = realize_composition(
            &store,
            &request(&[CastoreRootRef {
                digest_blake3: digest_hex(&digest),
                size,
            }]),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("composition-missing-blob"));
    }

    #[tokio::test]
    async fn missing_child_and_corrupt_directory_fail_before_persistence() {
        let blobs = Arc::new(MemoryBlobService::default());
        let directories = temporary_directory_service("composition-missing-child");
        let child_digest = B3Digest::from(&[9_u8; B3Digest::LENGTH]);
        let parent = Directory::try_from_iter([(PathComponent::try_from("child").unwrap(), Node::Directory {
            digest: child_digest,
            size: 0,
        })])
        .unwrap();
        let parent_size = parent.size();
        let parent_digest = directories.put(parent).await.unwrap();
        let root = CastoreRootRef {
            digest_blake3: digest_hex(&parent_digest),
            size: parent_size,
        };
        let prepared = plan_composition_request(&request(&[root])).unwrap();
        let error = realize_with_services(blobs.clone(), directories.clone(), &prepared).await.unwrap_err();
        assert!(error.to_string().contains("composition-missing-directory"));

        let complete_root = service_root(&blobs, &directories).await;
        let prepared = plan_composition_request(&request(&[complete_root])).unwrap();
        let corrupt = Arc::new(FaultDirectoryService::new(directories, DirectoryFault::CorruptRead));
        let error = realize_with_services(blobs, corrupt, &prepared).await.unwrap_err();
        assert!(error.to_string().contains("composition-inconsistent-directory"));
    }

    #[tokio::test]
    async fn persistence_and_root_recheck_failures_emit_no_receipt() {
        let blobs = Arc::new(MemoryBlobService::default());
        let directories = temporary_directory_service("composition-persist-failure");
        let root = service_root(&blobs, &directories).await;
        let prepared = plan_composition_request(&request(&[root])).unwrap();
        let reject_put = Arc::new(FaultDirectoryService::new(directories.clone(), DirectoryFault::RejectPut));
        let error = realize_with_services(blobs.clone(), reject_put, &prepared).await.unwrap_err();
        assert!(error.to_string().contains("composition-test-persist-failure"));

        let hide_put = Arc::new(FaultDirectoryService::new(directories, DirectoryFault::HidePut));
        let error = realize_with_services(blobs, hide_put, &prepared).await.unwrap_err();
        assert!(error.to_string().contains("composition-root-recheck-missing"));
    }
}
