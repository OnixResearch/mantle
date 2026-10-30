//! Store reads restricted to a receiver-owned logical-path view.
//!
//! A restricted view gates one exact normalized logical store path BEFORE any
//! PathInfo or remote lookup. It exposes no enumeration: a broad listing under
//! a restricted view is denied by construction, because filtering an
//! unrestricted scan afterwards would already have read paths outside the view.
//! A read outside the view returns one constant error that names no path and
//! does not depend on whether the path exists, so it is no existence oracle.

use std::sync::Arc;

use nix_compat::store_path::StorePath;
use snix_store::path_info::PathInfo;

use crate::Error;
use crate::OutputLookup;

/// The only text a refused read carries: no path, caveat or reason.
pub const RESTRICTED_VIEW_DENIED: &str = "restricted-store-view-denied";

/// Upper bound on a logical path presented to a view, matching the caveat
/// filter language's canonical value bound.
pub const MAX_LOGICAL_PATH_BYTES: usize = 512;

/// Pure decision over one exact absolute logical store path. Implementations
/// must use receiver-owned facts only (their own clock, revocation and site
/// policy) and must be safe to consult before every read.
pub trait LogicalPathView: Send + Sync {
    fn admits(&self, logical_path: &str) -> bool;
}

/// Read-only output lookup narrowed to a view. Cloning shares the view.
#[derive(Clone)]
pub struct RestrictedOutputLookup {
    inner: OutputLookup,
    view: Arc<dyn LogicalPathView>,
}

impl RestrictedOutputLookup {
    #[must_use]
    pub fn new(inner: OutputLookup, view: Arc<dyn LogicalPathView>) -> Self {
        Self { inner, view }
    }

    /// Gate the normalized logical path of `path` against the view.
    fn admit(&self, path: &StorePath<String>) -> Result<(), Error> {
        let logical = path.to_absolute_path_with_prefix(self.inner.store_dir());
        let is_bounded = logical.len() <= MAX_LOGICAL_PATH_BYTES;
        debug_assert!(logical.starts_with('/'));
        debug_assert!(!logical.is_empty());
        if is_bounded && self.view.admits(&logical) {
            return Ok(());
        }
        Err(Error::Store(RESTRICTED_VIEW_DENIED.to_string()))
    }

    // r[impl mantle.authority_attenuation.attenuated_store_view]
    pub async fn find(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, Error> {
        self.admit(path)?;
        self.inner.find(path).await
    }

    // r[impl mantle.authority_attenuation.attenuated_store_view]
    pub async fn find_with_layer(
        &self,
        path: &StorePath<String>,
    ) -> Result<Option<crate::layer::Layered<PathInfo>>, Error> {
        self.admit(path)?;
        self.inner.find_with_layer(path).await
    }

    // r[impl mantle.authority_attenuation.attenuated_store_view]
    pub async fn find_remote(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, Error> {
        self.admit(path)?;
        self.inner.find_remote(path).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    use nix_compat::store_path::DIGEST_SIZE;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;

    use super::*;
    use crate::StoreBackend;
    use crate::StoreConfig;
    use crate::StoreHandle;

    const STORE_DIR: &str = "/mantle/store";
    const NAR_DIGEST_BYTES: usize = 32;

    /// Admits exactly the listed logical paths and counts every consultation.
    struct ExactSet {
        admitted: Vec<String>,
        consulted: AtomicU32,
    }

    impl LogicalPathView for ExactSet {
        fn admits(&self, logical_path: &str) -> bool {
            self.consulted.fetch_add(1, Ordering::SeqCst);
            self.admitted.iter().any(|path| path == logical_path)
        }
    }

    fn store_path(name: &str, fill: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [fill; DIGEST_SIZE]).expect("valid test store path")
    }

    fn path_info(store_path: StorePath<String>) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").expect("valid symlink target"),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [7; NAR_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        }
    }

    struct Fixture {
        _state: tempfile::TempDir,
        _output: tempfile::TempDir,
        inside: StorePath<String>,
        outside: StorePath<String>,
        view: Arc<ExactSet>,
        lookup: RestrictedOutputLookup,
    }

    async fn fixture() -> Fixture {
        let state = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let handle = StoreHandle::open(StoreConfig::new(
            StoreBackend::Snix,
            state.path().to_path_buf(),
            output.path().to_path_buf(),
            STORE_DIR.to_string(),
        ))
        .await
        .unwrap();
        let inside = store_path("inside-view", 3);
        let outside = store_path("outside-view", 4);
        for path in [&inside, &outside] {
            handle.pathinfo_service().put(path_info(path.clone())).await.unwrap();
        }
        let view = Arc::new(ExactSet {
            admitted: vec![inside.to_absolute_path_with_prefix(STORE_DIR)],
            consulted: AtomicU32::new(0),
        });
        let lookup = RestrictedOutputLookup::new(handle.output_lookup(), view.clone());
        Fixture {
            _state: state,
            _output: output,
            inside,
            outside,
            view,
            lookup,
        }
    }

    // A view serves only its pattern, and a read outside it fails before the
    // service is consulted, identically for existing and absent paths.
    #[tokio::test]
    async fn view_serves_only_its_path_and_refuses_every_lookup_outside_it() {
        let fixture = fixture().await;
        let served = fixture.lookup.find(&fixture.inside).await.unwrap().expect("inside path served");
        assert_eq!(served.store_path, fixture.inside);
        let layered = fixture.lookup.find_with_layer(&fixture.inside).await.unwrap().expect("inside layered");
        assert_eq!(layered.value.store_path, fixture.inside);
        // The configured store has no remote cache: an admitted lookup reaches
        // the remote branch and finds nothing, which a refusal must not mimic.
        assert!(fixture.lookup.find_remote(&fixture.inside).await.unwrap().is_none());

        let absent = store_path("absent-outside", 9);
        for path in [&fixture.outside, &absent] {
            for error in [
                fixture.lookup.find(path).await.unwrap_err(),
                fixture.lookup.find_with_layer(path).await.map(|_| ()).unwrap_err(),
                fixture.lookup.find_remote(path).await.map(|_| ()).unwrap_err(),
            ] {
                let message = error.to_string();
                assert_eq!(message, format!("store: {RESTRICTED_VIEW_DENIED}"));
                assert!(!message.contains(&path.to_string()), "refusal disclosed a path");
            }
        }
        assert_eq!(fixture.view.consulted.load(Ordering::SeqCst), 3 + 6);
    }

    // The same view admits the exact path only: a sibling with the same name
    // under another digest is a different logical path and stays refused.
    #[tokio::test]
    async fn digest_lookalike_of_an_admitted_name_is_refused() {
        let fixture = fixture().await;
        let lookalike = store_path("inside-view", 5);
        assert_ne!(lookalike, fixture.inside);
        assert!(fixture.lookup.find(&lookalike).await.is_err());
        assert!(fixture.lookup.find(&fixture.inside).await.is_ok());
    }
}
