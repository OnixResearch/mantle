//! Track derivations and their computed hashes during conversion.

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use std::collections::HashMap;

/// Tracks known derivation paths and their `hash_derivation_modulo` values.
///
/// During recursive conversion, we memoize already-converted derivations
/// here to avoid redundant work and enable diamond dependency dedup.
pub struct KnownPaths {
    /// derivation ATerm hash → (drv store path, hash_derivation_modulo, Derivation)
    by_aterm_hash: HashMap<[u8; 32], KnownEntry>,
    /// drv store path string → hash_derivation_modulo
    hdm_by_drv_path: HashMap<String, [u8; 32]>,
    /// drv store path string → aterm hash (reverse index for O(1) lookup)
    drv_path_to_aterm: HashMap<String, [u8; 32]>,
    /// Track in-progress conversions for cycle detection.
    /// Key is an opaque identity derived from the CrunchDerivation name + builder + system.
    in_progress: std::collections::HashSet<String>,
    /// Store directory prefix (e.g. "/nix/store" or "/opt/crunch").
    store_dir: String,
}

pub struct KnownEntry {
    pub drv_path: StorePath<String>,
    pub hash_derivation_modulo: [u8; 32],
    pub derivation: Derivation,
    /// Whether this is a content-addressed derivation (output paths
    /// resolved after build).
    pub content_addressed: bool,
    /// Resolved output paths for CA derivations. Populated by
    /// `resolve_output()` after the build completes.
    /// Key: output name, Value: final store path.
    pub resolved_outputs: HashMap<String, StorePath<String>>,
}

impl KnownPaths {
    pub fn new(store_dir: &str) -> Self {
        Self {
            by_aterm_hash: HashMap::new(),
            hdm_by_drv_path: HashMap::new(),
            drv_path_to_aterm: HashMap::new(),
            in_progress: std::collections::HashSet::new(),
            store_dir: store_dir.to_string(),
        }
    }

    /// The store directory prefix used for path serialization.
    pub fn store_dir(&self) -> &str {
        &self.store_dir
    }

    /// Register a fully-converted derivation.
    pub fn insert(
        &mut self,
        aterm_hash: [u8; 32],
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: Derivation,
    ) {
        self.insert_ca(aterm_hash, drv_path, hdm, derivation, false);
    }

    /// Register a derivation, optionally marking it as content-addressed.
    pub fn insert_ca(
        &mut self,
        aterm_hash: [u8; 32],
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: Derivation,
        content_addressed: bool,
    ) {
        let drv_path_str = drv_path.to_absolute_path_with_prefix(&self.store_dir);
        self.hdm_by_drv_path.insert(drv_path_str.clone(), hdm);
        self.drv_path_to_aterm.insert(drv_path_str, aterm_hash);
        self.by_aterm_hash.insert(aterm_hash, KnownEntry {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation,
            content_addressed,
            resolved_outputs: HashMap::new(),
        });
    }

    /// Resolve a CA derivation's output path after build.
    pub fn resolve_output(
        &mut self,
        drv_path_abs: &str,
        output_name: &str,
        final_path: StorePath<String>,
    ) {
        let aterm_hash = self.drv_path_to_aterm.get(drv_path_abs)
            .copied()
            .expect("BUG: resolve_output called for unknown drv path");
        let entry = self.by_aterm_hash.get_mut(&aterm_hash)
            .expect("BUG: aterm hash not found");
        entry.resolved_outputs.insert(output_name.to_string(), final_path);
    }

    /// Get the resolved output path for a CA derivation, or the
    /// pre-computed path for an input-addressed derivation.
    pub fn get_output_path(
        &self,
        drv_path_abs: &str,
        output_name: &str,
    ) -> Option<StorePath<String>> {
        let aterm_hash = self.drv_path_to_aterm.get(drv_path_abs)?;
        let entry = self.by_aterm_hash.get(aterm_hash)?;
        if entry.content_addressed {
            entry.resolved_outputs.get(output_name).cloned()
        } else {
            entry.derivation.outputs.get(output_name)
                .and_then(|o| o.path.clone())
        }
    }

    /// Look up a derivation by its ATerm hash. Used for deduplication.
    pub fn get_by_aterm_hash(&self, hash: &[u8; 32]) -> Option<&KnownEntry> {
        self.by_aterm_hash.get(hash)
    }

    /// Look up hash_derivation_modulo by drv store path.
    /// This is the callback for `Derivation::hash_derivation_modulo()`.
    pub fn get_hdm_by_drv_path(&self, drv_path: &str) -> Option<[u8; 32]> {
        self.hdm_by_drv_path.get(drv_path).copied()
    }

    /// Look up a KnownEntry by drv store path string.
    /// Used by the build orchestrator to find derivations and their
    /// outputs when resolving input_derivations.
    pub fn get_by_drv_path(&self, drv_path: &str) -> Option<&KnownEntry> {
        let aterm_hash = self.drv_path_to_aterm.get(drv_path)?;
        self.by_aterm_hash.get(aterm_hash)
    }

    /// Mark a derivation as in-progress (for cycle detection).
    /// Returns false if already in progress (cycle detected).
    pub fn begin_conversion(&mut self, identity: &str) -> bool {
        self.in_progress.insert(identity.to_string())
    }

    /// Unmark a derivation as in-progress.
    pub fn end_conversion(&mut self, identity: &str) {
        self.in_progress.remove(identity);
    }
}

impl Default for KnownPaths {
    fn default() -> Self {
        Self::new(nix_compat::store_path::STORE_DIR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use bstr::BString;
    use nix_compat::derivation::Output;

    /// Build a minimal Derivation for testing. The content doesn't need to
    /// be valid — we just need a struct to store.
    fn dummy_derivation(_name: &str) -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        Derivation {
            arguments: vec![],
            builder: "/bin/sh".to_string(),
            environment: BTreeMap::new(),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    /// Build a fake StorePath. Uses a fixed 20-byte hash with the name appended.
    fn fake_store_path(name: &str) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [0xaa; 20]).unwrap()
    }

    #[test]
    fn insert_and_get_by_aterm_hash() {
        let mut kp = KnownPaths::default();
        let aterm = [1u8; 32];
        let hdm = [2u8; 32];
        let path = fake_store_path("foo.drv");
        let drv = dummy_derivation("foo");

        kp.insert(aterm, path.clone(), hdm, drv);

        let entry = kp.get_by_aterm_hash(&aterm).unwrap();
        assert_eq!(entry.drv_path, path);
        assert_eq!(entry.hash_derivation_modulo, hdm);
    }

    #[test]
    fn get_hdm_by_drv_path() {
        let mut kp = KnownPaths::default();
        let aterm = [1u8; 32];
        let hdm = [2u8; 32];
        let path = fake_store_path("bar.drv");

        kp.insert(aterm, path.clone(), hdm, dummy_derivation("bar"));

        let got = kp.get_hdm_by_drv_path(&path.to_absolute_path()).unwrap();
        assert_eq!(got, hdm);
    }

    #[test]
    fn get_by_drv_path_indexed() {
        let mut kp = KnownPaths::default();
        let aterm = [3u8; 32];
        let hdm = [4u8; 32];
        let path = fake_store_path("baz.drv");
        let drv = dummy_derivation("baz");

        kp.insert(aterm, path.clone(), hdm, drv);

        let entry = kp.get_by_drv_path(&path.to_absolute_path()).unwrap();
        assert_eq!(entry.drv_path, path);
        assert_eq!(entry.hash_derivation_modulo, hdm);
    }

    #[test]
    fn get_by_drv_path_unknown_returns_none() {
        let kp = KnownPaths::default();
        assert!(kp.get_by_drv_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nope.drv").is_none());
    }

    #[test]
    fn multiple_inserts_resolve_independently() {
        let mut kp = KnownPaths::default();

        let aterm_a = [10u8; 32];
        let hdm_a = [11u8; 32];
        let path_a = fake_store_path("alpha.drv");

        let aterm_b = [20u8; 32];
        let hdm_b = [21u8; 32];
        let path_b = fake_store_path("beta.drv");

        kp.insert(aterm_a, path_a.clone(), hdm_a, dummy_derivation("alpha"));
        kp.insert(aterm_b, path_b.clone(), hdm_b, dummy_derivation("beta"));

        let ea = kp.get_by_drv_path(&path_a.to_absolute_path()).unwrap();
        assert_eq!(ea.hash_derivation_modulo, hdm_a);

        let eb = kp.get_by_drv_path(&path_b.to_absolute_path()).unwrap();
        assert_eq!(eb.hash_derivation_modulo, hdm_b);
    }

    #[test]
    fn overwrite_same_drv_path_updates_index() {
        let mut kp = KnownPaths::default();
        let path = fake_store_path("reuse.drv");

        let aterm_1 = [30u8; 32];
        let hdm_1 = [31u8; 32];
        kp.insert(aterm_1, path.clone(), hdm_1, dummy_derivation("reuse"));

        // Re-insert same drv path with different aterm hash
        let aterm_2 = [40u8; 32];
        let hdm_2 = [41u8; 32];
        kp.insert(aterm_2, path.clone(), hdm_2, dummy_derivation("reuse"));

        // Index should point to the latest entry
        let entry = kp.get_by_drv_path(&path.to_absolute_path()).unwrap();
        assert_eq!(entry.hash_derivation_modulo, hdm_2);
    }

    #[test]
    fn cycle_detection_begin_returns_false_on_reentry() {
        let mut kp = KnownPaths::default();
        assert!(kp.begin_conversion("foo:bar:baz"));
        assert!(!kp.begin_conversion("foo:bar:baz"));
    }

    #[test]
    fn cycle_detection_end_allows_reentry() {
        let mut kp = KnownPaths::default();
        assert!(kp.begin_conversion("foo:bar:baz"));
        kp.end_conversion("foo:bar:baz");
        assert!(kp.begin_conversion("foo:bar:baz"));
    }
}
