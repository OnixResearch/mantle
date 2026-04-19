//! Native closure resolution: walk the reference graph in PathInfo
//! (local redb + optional remote binary cache) instead of shelling
//! out to `nix-store -qR`.
use std::collections::BTreeSet;

use nix_compat::store_path::StorePath;
use snix_store::pathinfoservice::PathInfoService;
use tracing::warn;

use crate::Error;
use crate::StoreAuditEvent;
use crate::StoreAuditKind;
use crate::StoreFallbackMode;

/// Maximum transitive depth for closure walks. Prevents runaway
/// recursion from cycles the visited-set misses or absurdly deep
/// dependency chains.
pub const MAX_CLOSURE_DEPTH: u32 = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureResolution {
    pub paths: Vec<StorePath<String>>,
    pub audit_events: Vec<StoreAuditEvent>,
}

enum LookupStatus {
    Found(Vec<StorePath<String>>),
    Missing(String),
    Failed(String),
}

enum ReferenceLookup {
    Resolved(Vec<StorePath<String>>),
    Degraded(String),
}

/// Resolve the full runtime closure of `root` by walking PathInfo
/// references.
///
/// Tries `local` first, then `remote` (if provided). Returns the
/// transitive set of all referenced store paths, including `root`
/// itself, plus any practical-mode degraded audit events.
pub async fn resolve_closure(
    root: &StorePath<String>,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    fallback_mode: StoreFallbackMode,
    store_dir: &str,
) -> Result<ClosureResolution, Error> {
    const MAX_CLOSURE_MEMBERS: usize = 100_000;
    let mut visited: BTreeSet<[u8; 20]> = BTreeSet::new();
    let mut result: Vec<StorePath<String>> = Vec::with_capacity(64);
    let mut audit_events: Vec<StoreAuditEvent> = Vec::with_capacity(4);
    let mut stack: Vec<(StorePath<String>, u32)> = Vec::with_capacity(64);
    stack.push((root.clone(), 0));

    while let Some((path, depth)) = stack.pop() {
        let digest = *path.digest();
        if !visited.insert(digest) {
            continue;
        }
        assert!(result.len() < MAX_CLOSURE_MEMBERS, "closure exceeded {MAX_CLOSURE_MEMBERS} members");
        result.push(path.clone());
        if depth >= MAX_CLOSURE_DEPTH {
            warn!(path = %path, depth = depth, "closure depth limit ({MAX_CLOSURE_DEPTH}) reached, skipping deeper refs");
            continue;
        }
        match lookup_references(local, remote, digest).await {
            ReferenceLookup::Resolved(refs) => push_unvisited_refs(&mut stack, &visited, refs, depth),
            ReferenceLookup::Degraded(reason) => {
                handle_degraded_lookup(fallback_mode, &path, reason, store_dir, &mut audit_events)?;
            }
        }
    }

    debug_assert!(!result.is_empty(), "closure must contain at least the root");
    Ok(ClosureResolution {
        paths: result,
        audit_events,
    })
}

async fn lookup_references(
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    digest: [u8; 20],
) -> ReferenceLookup {
    match lookup_local_references(local, digest).await {
        LookupStatus::Found(refs) => ReferenceLookup::Resolved(refs),
        LookupStatus::Missing(local_reason) => lookup_remote_or_degrade(remote, digest, local_reason).await,
        LookupStatus::Failed(local_reason) => lookup_remote_or_degrade(remote, digest, local_reason).await,
    }
}

async fn lookup_local_references(local: &dyn PathInfoService, digest: [u8; 20]) -> LookupStatus {
    match local.get_references(digest).await {
        Ok(Some(refs)) => LookupStatus::Found(refs),
        Ok(None) => LookupStatus::Missing("no local PathInfo".to_string()),
        Err(e) => LookupStatus::Failed(format!("local PathInfo query failed: {e}")),
    }
}

async fn lookup_remote_or_degrade(
    remote: Option<&dyn PathInfoService>,
    digest: [u8; 20],
    local_reason: String,
) -> ReferenceLookup {
    match lookup_remote_references(remote, digest).await {
        LookupStatus::Found(refs) => ReferenceLookup::Resolved(refs),
        LookupStatus::Missing(remote_reason) | LookupStatus::Failed(remote_reason) => {
            ReferenceLookup::Degraded(format!("{local_reason}; {remote_reason}"))
        }
    }
}

async fn lookup_remote_references(remote: Option<&dyn PathInfoService>, digest: [u8; 20]) -> LookupStatus {
    let Some(remote) = remote else {
        return LookupStatus::Missing("no remote narinfo configured".to_string());
    };
    match remote.get_references(digest).await {
        Ok(Some(refs)) => LookupStatus::Found(refs),
        Ok(None) => LookupStatus::Missing("no remote narinfo".to_string()),
        Err(e) => LookupStatus::Failed(format!("remote narinfo query failed: {e}")),
    }
}

fn handle_degraded_lookup(
    fallback_mode: StoreFallbackMode,
    path: &StorePath<String>,
    reason: String,
    store_dir: &str,
    audit_events: &mut Vec<StoreAuditEvent>,
) -> Result<(), Error> {
    assert!(!store_dir.is_empty(), "store_dir must not be empty");
    assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    if fallback_mode.is_strict() {
        return Err(Error::MissingClosureFacts {
            path: path.clone(),
            store_dir: store_dir.to_string(),
            detail: reason,
        });
    }
    let detail = format!("missing closure facts for {}: {reason}", path.to_absolute_path_with_prefix(store_dir));
    warn!(path = %path, detail = %detail, "closure resolution degraded, mounting declared path only");
    audit_events.push(StoreAuditEvent::new(StoreAuditKind::ClosureResolutionDegraded, detail));
    Ok(())
}

fn push_unvisited_refs(
    stack: &mut Vec<(StorePath<String>, u32)>,
    visited: &BTreeSet<[u8; 20]>,
    refs: Vec<StorePath<String>>,
    depth: u32,
) {
    for r in refs {
        if !visited.contains(r.digest()) {
            stack.push((r, depth.saturating_add(1)));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use async_trait::async_trait;
    use futures::stream::StreamExt;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::PathInfoService;
    use snix_store::pathinfoservice::{self};

    use super::*;

    #[derive(Default, Clone)]
    struct MockPathInfoService {
        entries: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], PathInfo>>>,
        failed_gets: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], String>>>,
        get_counts: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], u32>>>,
        reference_counts: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], u32>>>,
    }

    impl MockPathInfoService {
        fn insert(&self, pi: PathInfo) {
            self.entries.lock().unwrap().insert(*pi.store_path.digest(), pi);
        }

        fn fail_get(&self, sp: &StorePath<String>, message: &str) {
            self.failed_gets.lock().unwrap().insert(*sp.digest(), message.to_string());
        }

        fn get_count(&self, sp: &StorePath<String>) -> u32 {
            self.get_counts.lock().unwrap().get(sp.digest()).copied().unwrap_or(0)
        }

        fn reference_count(&self, sp: &StorePath<String>) -> u32 {
            self.reference_counts.lock().unwrap().get(sp.digest()).copied().unwrap_or(0)
        }

        fn bump_count(counts: &std::sync::Mutex<BTreeMap<[u8; 20], u32>>, digest: [u8; 20]) {
            let mut counts = counts.lock().unwrap();
            let next = counts.get(&digest).copied().unwrap_or(0).saturating_add(1);
            counts.insert(digest, next);
        }
    }

    #[async_trait]
    impl PathInfoService for MockPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            Self::bump_count(&self.get_counts, digest);
            if let Some(message) = self.failed_gets.lock().unwrap().get(&digest).cloned() {
                return Err(std::io::Error::other(message).into());
            }
            Ok(self.entries.lock().unwrap().get(&digest).cloned())
        }

        async fn get_references(
            &self,
            digest: [u8; 20],
        ) -> Result<Option<Vec<StorePath<String>>>, pathinfoservice::Error> {
            Self::bump_count(&self.reference_counts, digest);
            if let Some(message) = self.failed_gets.lock().unwrap().get(&digest).cloned() {
                return Err(std::io::Error::other(message).into());
            }
            Ok(self.entries.lock().unwrap().get(&digest).map(|pi| pi.references.clone()))
        }

        async fn put(&self, pi: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
            self.entries.lock().unwrap().insert(*pi.store_path.digest(), pi.clone());
            Ok(pi)
        }

        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
            let entries: Vec<_> = self.entries.lock().unwrap().values().cloned().collect();
            futures::stream::iter(entries.into_iter().map(Ok)).boxed()
        }
    }

    fn make_sp(name: &str, seed: u8) -> StorePath<String> {
        let mut digest = [0u8; 20];
        digest[0] = seed;
        for (i, b) in name.as_bytes().iter().enumerate() {
            digest[(i + 1) % 20] = *b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).expect("valid store path")
    }

    fn make_pi(sp: &StorePath<String>, refs: Vec<StorePath<String>>) -> PathInfo {
        PathInfo {
            store_path: sp.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("x").unwrap(),
            },
            references: refs,
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn single_path_no_refs() {
        let local = MockPathInfoService::default();
        let sp = make_sp("hello", 1);
        local.insert(make_pi(&sp, vec![]));

        let closure = resolve_closure(&sp, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 1);
        assert_eq!(closure.paths[0], sp);
    }

    #[tokio::test]
    async fn linear_chain() {
        let local = MockPathInfoService::default();
        let c = make_sp("c", 3);
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&c, vec![]));
        local.insert(make_pi(&b, vec![c.clone()]));
        local.insert(make_pi(&a, vec![b.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 3);
        let digests: BTreeSet<_> = closure.paths.iter().map(|p| *p.digest()).collect();
        assert!(digests.contains(a.digest()));
        assert!(digests.contains(b.digest()));
        assert!(digests.contains(c.digest()));
    }

    #[tokio::test]
    async fn diamond_dedup() {
        let local = MockPathInfoService::default();
        let d = make_sp("d", 4);
        let c = make_sp("c", 3);
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&d, vec![]));
        local.insert(make_pi(&c, vec![d.clone()]));
        local.insert(make_pi(&b, vec![d.clone()]));
        local.insert(make_pi(&a, vec![b.clone(), c.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 4, "diamond should dedup D");
    }

    #[tokio::test]
    async fn cycle_terminates() {
        let local = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        local.insert(make_pi(&b, vec![a.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 2);
    }

    #[tokio::test]
    async fn practical_mode_records_degraded_child_lookup() {
        let local = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/crunch/store").await.unwrap();
        assert_eq!(closure.paths.len(), 2);
        assert_eq!(closure.audit_events.len(), 1);
        assert_eq!(closure.audit_events[0].kind, StoreAuditKind::ClosureResolutionDegraded);
        assert!(closure.audit_events[0].detail.contains(&b.to_absolute_path_with_prefix("/crunch/store")));
        assert!(!closure.audit_events[0].detail.contains("/nix/store"));
    }

    #[tokio::test]
    async fn remote_fallback() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        remote.insert(make_pi(&b, vec![]));

        let closure = resolve_closure(&a, &local, Some(&remote), StoreFallbackMode::Practical, "/nix/store")
            .await
            .unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 2);
        assert_eq!(remote.reference_count(&b), 1);
        assert_eq!(remote.get_count(&b), 0);
    }

    #[tokio::test]
    async fn practical_mode_records_degraded_root_lookup() {
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/crunch/store").await.unwrap();
        assert_eq!(closure.paths.len(), 1);
        assert_eq!(closure.paths[0], a);
        assert_eq!(closure.audit_events.len(), 1);
        assert_eq!(closure.audit_events[0].kind, StoreAuditKind::ClosureResolutionDegraded);
        assert!(closure.audit_events[0].detail.contains("/crunch/store"));
    }

    #[tokio::test]
    async fn strict_mode_rejects_missing_root_closure_facts() {
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);

        let err = resolve_closure(&a, &local, None, StoreFallbackMode::Strict, "/crunch/store").await.unwrap_err();
        assert!(
            matches!(err, Error::MissingClosureFacts { ref path, ref store_dir, .. } if *path == a && store_dir == "/crunch/store")
        );
        assert!(err.to_string().contains("/crunch/store"));
        assert!(!err.to_string().contains("/nix/store"));
    }

    #[tokio::test]
    async fn self_reference() {
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);
        local.insert(make_pi(&a, vec![a.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 1);
    }

    #[tokio::test]
    async fn depth_limit_enforced() {
        let local = MockPathInfoService::default();
        let depth = MAX_CLOSURE_DEPTH + 10;
        let mut paths: Vec<StorePath<String>> = Vec::new();

        for i in 0..=depth {
            let sp = make_sp(&format!("n{i}"), (i % 255) as u8);
            paths.push(sp);
        }

        for i in 0..depth as usize {
            local.insert(make_pi(&paths[i], vec![paths[i + 1].clone()]));
        }
        local.insert(make_pi(&paths[depth as usize], vec![]));

        let closure =
            resolve_closure(&paths[0], &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert!(
            closure.paths.len() <= (MAX_CLOSURE_DEPTH as usize) + 2,
            "closure too large: {} (limit {})",
            closure.paths.len(),
            MAX_CLOSURE_DEPTH + 2
        );
    }

    #[tokio::test]
    async fn local_query_error_can_still_use_remote_refs() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let a = make_sp("a", 1);

        local.fail_get(&a, "disk offline");
        remote.insert(make_pi(&a, vec![]));

        let closure = resolve_closure(&a, &local, Some(&remote), StoreFallbackMode::Practical, "/nix/store")
            .await
            .unwrap();
        assert_eq!(closure.paths.len(), 1);
        assert!(closure.audit_events.is_empty());
        assert_eq!(remote.reference_count(&a), 1);
        assert_eq!(remote.get_count(&a), 0);
    }

    #[tokio::test]
    async fn remote_metadata_lookup_failure_degrades_practical_mode() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        remote.fail_get(&b, "narinfo offline");

        let closure = resolve_closure(&a, &local, Some(&remote), StoreFallbackMode::Practical, "/crunch/store")
            .await
            .unwrap();
        assert_eq!(closure.paths.len(), 2);
        assert_eq!(closure.audit_events.len(), 1);
        assert_eq!(closure.audit_events[0].kind, StoreAuditKind::ClosureResolutionDegraded);
        assert!(closure.audit_events[0].detail.contains("narinfo offline"));
        assert_eq!(remote.reference_count(&b), 1);
        assert_eq!(remote.get_count(&b), 0);
    }
}
