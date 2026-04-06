//! Native closure resolution: walk the reference graph in PathInfo
//! (local redb + optional remote binary cache) instead of shelling
//! out to `nix-store -qR`.
use async_trait::async_trait;

use std::collections::BTreeSet;

use nix_compat::store_path::StorePath;
use snix_store::pathinfoservice::PathInfoService;
use tracing::warn;

use crate::Error;

/// Maximum transitive depth for closure walks. Prevents runaway
/// recursion from cycles the visited-set misses or absurdly deep
/// dependency chains.
pub const MAX_CLOSURE_DEPTH: u32 = 1024;

/// Resolve the full runtime closure of `root` by walking PathInfo
/// references.
///
/// Tries `local` first, then `remote` (if provided). Returns the
/// transitive set of all referenced store paths, including `root`
/// itself.
///
/// Paths with no PathInfo in either source are skipped with a warning
/// (graceful degradation — static binaries have no refs, dynamic ones
/// will fail at load time with a clear error).
pub async fn resolve_closure(
    root: &StorePath<String>,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
) -> Result<Vec<StorePath<String>>, Error> {
    let mut visited: BTreeSet<[u8; 20]> = BTreeSet::new();
    let mut result: Vec<StorePath<String>> = Vec::new();
    let mut stack: Vec<(StorePath<String>, u32)> = vec![(root.clone(), 0)];

    while let Some((path, depth)) = stack.pop() {
        let digest = *path.digest();

        if !visited.insert(digest) {
            // Already visited — cycle or diamond dep. Skip.
            continue;
        }

        result.push(path.clone());

        if depth >= MAX_CLOSURE_DEPTH {
            warn!(
                path = %path,
                depth = depth,
                "closure depth limit ({MAX_CLOSURE_DEPTH}) reached, skipping deeper refs"
            );
            continue;
        }

        // Try local PathInfo, then remote.
        let refs = match local.get(digest).await {
            Ok(Some(pi)) => pi.references,
            Ok(None) => {
                if let Some(r) = remote {
                    match r.get(digest).await {
                        Ok(Some(pi)) => pi.references,
                        Ok(None) => {
                            if depth > 0 {
                                warn!(
                                    path = %path,
                                    "no closure data (local or remote), mounting without further refs"
                                );
                            }
                            continue;
                        }
                        Err(e) => {
                            warn!(
                                path = %path,
                                err = %e,
                                "remote PathInfo query failed, skipping refs"
                            );
                            continue;
                        }
                    }
                } else {
                    if depth > 0 {
                        warn!(
                            path = %path,
                            "no local PathInfo and no remote configured, mounting without further refs"
                        );
                    }
                    continue;
                }
            }
            Err(e) => {
                warn!(
                    path = %path,
                    err = %e,
                    "local PathInfo query failed, skipping refs"
                );
                continue;
            }
        };

        for r in refs {
            if !visited.contains(r.digest()) {
                stack.push((r, depth.saturating_add(1)));
            }
        }
    }

    debug_assert!(!result.is_empty(), "closure must contain at least the root");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::{self, PathInfoService};

    // -- Mock PathInfoService --------------------------------------------------

    /// In-memory PathInfoService for testing closure walks.
    #[derive(Default, Clone)]
    struct MockPathInfoService {
        entries: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], PathInfo>>>,
    }

    impl MockPathInfoService {
        fn insert(&self, pi: PathInfo) {
            self.entries.lock().unwrap().insert(*pi.store_path.digest(), pi);
        }
    }

    
    #[async_trait]
    impl PathInfoService for MockPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            Ok(self.entries.lock().unwrap().get(&digest).cloned())
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

    use futures::stream::StreamExt;

    // -- Helpers ---------------------------------------------------------------

    /// Create a StorePath with a unique digest derived from `name_bytes`.
    fn make_sp(name: &str, seed: u8) -> StorePath<String> {
        let mut digest = [0u8; 20];
        digest[0] = seed;
        for (i, b) in name.as_bytes().iter().enumerate() {
            digest[(i + 1) % 20] = *b;
        }
        StorePath::from_name_and_digest_fixed(name, digest)
            .expect("valid store path")
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

    // -- Tests -----------------------------------------------------------------

    #[tokio::test]
    async fn single_path_no_refs() {
        let local = MockPathInfoService::default();
        let sp = make_sp("hello", 1);
        local.insert(make_pi(&sp, vec![]));

        let closure = resolve_closure(&sp, &local, None).await.unwrap();
        assert_eq!(closure.len(), 1);
        assert_eq!(closure[0], sp);
    }

    #[tokio::test]
    async fn linear_chain() {
        // A -> B -> C (no refs)
        let local = MockPathInfoService::default();
        let c = make_sp("c", 3);
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&c, vec![]));
        local.insert(make_pi(&b, vec![c.clone()]));
        local.insert(make_pi(&a, vec![b.clone()]));

        let closure = resolve_closure(&a, &local, None).await.unwrap();
        assert_eq!(closure.len(), 3);
        // All three must be present (order depends on stack traversal).
        let digests: BTreeSet<_> = closure.iter().map(|p| *p.digest()).collect();
        assert!(digests.contains(a.digest()));
        assert!(digests.contains(b.digest()));
        assert!(digests.contains(c.digest()));
    }

    #[tokio::test]
    async fn diamond_dedup() {
        // A -> B, A -> C, B -> D, C -> D
        let local = MockPathInfoService::default();
        let d = make_sp("d", 4);
        let c = make_sp("c", 3);
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&d, vec![]));
        local.insert(make_pi(&c, vec![d.clone()]));
        local.insert(make_pi(&b, vec![d.clone()]));
        local.insert(make_pi(&a, vec![b.clone(), c.clone()]));

        let closure = resolve_closure(&a, &local, None).await.unwrap();
        assert_eq!(closure.len(), 4, "diamond should dedup D");
    }

    #[tokio::test]
    async fn cycle_terminates() {
        // A -> B -> A
        let local = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        local.insert(make_pi(&b, vec![a.clone()]));

        let closure = resolve_closure(&a, &local, None).await.unwrap();
        assert_eq!(closure.len(), 2);
    }

    #[tokio::test]
    async fn missing_path_warns_continues() {
        // A -> B, B not in any service.
        let local = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        // B intentionally missing.

        let closure = resolve_closure(&a, &local, None).await.unwrap();
        // Both A and B in the closure (B was listed as a ref of A).
        assert_eq!(closure.len(), 2);
    }

    #[tokio::test]
    async fn remote_fallback() {
        // A is local, B is remote-only.
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        remote.insert(make_pi(&b, vec![]));

        let closure = resolve_closure(&a, &local, Some(&remote)).await.unwrap();
        assert_eq!(closure.len(), 2);
    }

    #[tokio::test]
    async fn root_with_no_pathinfo_returns_just_root() {
        // Root has no PathInfo anywhere.
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);

        let closure = resolve_closure(&a, &local, None).await.unwrap();
        assert_eq!(closure.len(), 1);
        assert_eq!(closure[0], a);
    }

    #[tokio::test]
    async fn self_reference() {
        // A references itself.
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);
        local.insert(make_pi(&a, vec![a.clone()]));

        let closure = resolve_closure(&a, &local, None).await.unwrap();
        assert_eq!(closure.len(), 1);
    }

    #[tokio::test]
    async fn depth_limit_enforced() {
        // Build a chain deeper than MAX_CLOSURE_DEPTH.
        let local = MockPathInfoService::default();
        let depth = MAX_CLOSURE_DEPTH + 10;
        let mut paths: Vec<StorePath<String>> = Vec::new();

        for i in 0..=depth {
            let sp = make_sp(&format!("n{i}"), (i % 255) as u8);
            paths.push(sp);
        }

        // Wire refs: n0 -> n1 -> n2 -> ... -> n_{depth}
        for i in 0..depth as usize {
            local.insert(make_pi(&paths[i], vec![paths[i + 1].clone()]));
        }
        local.insert(make_pi(&paths[depth as usize], vec![]));

        let closure = resolve_closure(&paths[0], &local, None).await.unwrap();
        // Should have at most MAX_CLOSURE_DEPTH + 1 entries (root + 1024 levels).
        assert!(
            closure.len() <= (MAX_CLOSURE_DEPTH as usize) + 2,
            "closure too large: {} (limit {})",
            closure.len(),
            MAX_CLOSURE_DEPTH + 2
        );
    }
}
