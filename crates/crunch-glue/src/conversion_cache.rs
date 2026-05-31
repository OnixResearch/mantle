//! Track derivations and their computed hashes during conversion.
//!
//! `ConversionCache` handles conversion-time concerns: ATerm hash
//! dedup, HDM cache, cycle detection. Build-time state (output
//! resolution, CA tracking) lives in `crunch_build::DerivationRegistry`.

use std::collections::HashMap;

use crunch_attestation::Claims;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;

/// Tracks derivations produced by `convert()`.
///
/// Memoizes already-converted derivations to avoid redundant work
/// and enables diamond dependency dedup via ATerm hash.
/// Maximum pending entries before drain_pending() must be called.
/// Prevents unbounded memory growth if callers forget to drain.
const MAX_PENDING: u32 = 16_384;

pub type PendingEntry = (StorePath<String>, [u8; 32], Derivation, bool, Vec<String>, Option<Claims>);

pub struct InsertCaEntry {
    pub aterm_hash: [u8; 32],
    pub drv_path: StorePath<String>,
    pub hdm: [u8; 32],
    pub derivation: Derivation,
    pub content_addressed: bool,
    pub dynamic_plan_outputs: Vec<String>,
    pub provenance_claims: Option<Claims>,
}

pub struct ConversionCache {
    /// derivation ATerm hash -> (drv store path, hash_derivation_modulo, Derivation)
    by_aterm_hash: HashMap<[u8; 32], ConversionEntry>,
    /// drv store path string -> hash_derivation_modulo
    hdm_by_drv_path: HashMap<String, [u8; 32]>,
    /// drv store path string -> aterm hash (reverse index for O(1) lookup)
    drv_path_to_aterm: HashMap<String, [u8; 32]>,
    /// Track in-progress conversions for cycle detection.
    /// Key is an opaque identity derived from CrunchDerivation name + builder + system.
    in_progress: std::collections::HashSet<String>,
    /// Track completed conversions by the same opaque identity used for cycle detection.
    ///
    /// This is intentionally separate from the ATerm-hash cache: recursive conversion
    /// needs a cheap pre-order memoization key before the ATerm bytes are available.
    completed_by_identity: HashMap<String, CompletedConversion>,
    /// Store directory prefix (e.g. "/nix/store" or "/opt/crunch").
    store_dir: String,
    /// Entries added since the last `drain_pending()`. Enables
    /// incremental bridging to DerivationRegistry while converting.
    pending: Vec<PendingEntry>,
}

/// A derivation entry produced during conversion.
#[derive(Clone)]
pub struct CompletedConversion {
    pub drv_path: StorePath<String>,
    pub derivation: Derivation,
    pub dynamic_plan_outputs: Vec<String>,
}

pub struct ConversionEntry {
    pub drv_path: StorePath<String>,
    pub hash_derivation_modulo: [u8; 32],
    pub derivation: Derivation,
    /// Whether this is a content-addressed derivation (output paths
    /// resolved after build).
    pub content_addressed: bool,
    pub dynamic_plan_outputs: Vec<String>,
    pub provenance_claims: Option<Claims>,
}

impl ConversionCache {
    pub fn new(store_dir: &str) -> Self {
        Self {
            by_aterm_hash: HashMap::new(),
            hdm_by_drv_path: HashMap::new(),
            drv_path_to_aterm: HashMap::new(),
            in_progress: std::collections::HashSet::new(),
            completed_by_identity: HashMap::new(),
            store_dir: store_dir.to_string(),
            pending: Vec::new(),
        }
    }

    /// The store directory prefix used for path serialization.
    pub fn store_dir(&self) -> &str {
        &self.store_dir
    }

    /// Register a fully-converted derivation (input-addressed).
    pub fn insert(&mut self, aterm_hash: [u8; 32], drv_path: StorePath<String>, hdm: [u8; 32], derivation: Derivation) {
        self.insert_ca(InsertCaEntry {
            aterm_hash,
            drv_path,
            hdm,
            derivation,
            content_addressed: false,
            dynamic_plan_outputs: Vec::new(),
            provenance_claims: None,
        });
    }

    /// Register a derivation, optionally marking it as content-addressed.
    pub fn insert_ca(&mut self, entry: InsertCaEntry) {
        debug_assert!(!entry.derivation.outputs.is_empty(), "derivation must have at least one output");
        debug_assert!(entry.aterm_hash != [0u8; 32], "aterm_hash must not be all zeros");
        debug_assert!(
            entry.dynamic_plan_outputs.iter().all(|output| entry.derivation.outputs.contains_key(output)),
            "dynamic plan outputs must be declared derivation outputs"
        );

        let drv_path_str = entry.drv_path.to_absolute_path_with_prefix(&self.store_dir);
        self.hdm_by_drv_path.insert(drv_path_str.clone(), entry.hdm);
        self.drv_path_to_aterm.insert(drv_path_str.clone(), entry.aterm_hash);
        self.pending.push((
            entry.drv_path.clone(),
            entry.hdm,
            entry.derivation.clone(),
            entry.content_addressed,
            entry.dynamic_plan_outputs.clone(),
            entry.provenance_claims.clone(),
        ));
        debug_assert!(
            (self.pending.len() as u32) <= MAX_PENDING,
            "pending entries exceeded limit ({}); call drain_pending()",
            MAX_PENDING,
        );

        self.by_aterm_hash.insert(entry.aterm_hash, ConversionEntry {
            drv_path: entry.drv_path,
            hash_derivation_modulo: entry.hdm,
            derivation: entry.derivation,
            content_addressed: entry.content_addressed,
            dynamic_plan_outputs: entry.dynamic_plan_outputs,
            provenance_claims: entry.provenance_claims,
        });

        debug_assert!(self.hdm_by_drv_path.contains_key(&drv_path_str));
        debug_assert!(self.drv_path_to_aterm.contains_key(&drv_path_str));
        debug_assert!(self.by_aterm_hash.contains_key(&entry.aterm_hash));
    }

    /// Look up a derivation by its ATerm hash. Used for deduplication.
    pub fn get_by_aterm_hash(&self, hash: &[u8; 32]) -> Option<&ConversionEntry> {
        self.by_aterm_hash.get(hash)
    }

    /// Look up hash_derivation_modulo by drv store path.
    pub fn get_hdm_by_drv_path(&self, drv_path: &str) -> Option<[u8; 32]> {
        self.hdm_by_drv_path.get(drv_path).copied()
    }

    /// Look up a ConversionEntry by drv store path string.
    pub fn get_by_drv_path(&self, drv_path: &str) -> Option<&ConversionEntry> {
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

    /// Look up a completed conversion by the pre-order identity key.
    pub fn get_completed_identity(&self, identity: &str) -> Option<CompletedConversion> {
        self.completed_by_identity.get(identity).cloned()
    }

    /// Remember a completed conversion by the pre-order identity key.
    pub fn insert_completed_identity(
        &mut self,
        identity: String,
        drv_path: StorePath<String>,
        derivation: Derivation,
        dynamic_plan_outputs: Vec<String>,
    ) {
        self.completed_by_identity.insert(identity, CompletedConversion {
            drv_path,
            derivation,
            dynamic_plan_outputs,
        });
    }

    /// Iterate all entries for populating a `DerivationRegistry`.
    ///
    /// Yields `(drv_path, hdm, derivation, content_addressed, dynamic_plan_outputs,
    /// provenance_claims)` for each registered derivation.
    pub fn iter_entries(&self) -> impl Iterator<Item = PendingEntry> + '_ {
        self.by_aterm_hash.values().map(|e| {
            (
                e.drv_path.clone(),
                e.hash_derivation_modulo,
                e.derivation.clone(),
                e.content_addressed,
                e.dynamic_plan_outputs.clone(),
                e.provenance_claims.clone(),
            )
        })
    }

    /// Drain entries added since the last drain. Returns new entries
    /// for incremental bridging to `DerivationRegistry` while the
    /// convert loop is still running.
    ///
    /// Call after each `convert()` to get derivations produced by
    /// that root and its transitive deps.
    pub fn drain_pending(&mut self) -> Vec<PendingEntry> {
        std::mem::take(&mut self.pending)
    }

    /// Number of pending entries not yet drained.
    pub fn pending_count(&self) -> u32 {
        self.pending.len() as u32
    }
}

impl Default for ConversionCache {
    fn default() -> Self {
        Self::new(nix_compat::store_path::STORE_DIR)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use nix_compat::derivation::Output;

    use super::*;

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

    fn fake_store_path(name: &str) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [0xaa; 20]).unwrap()
    }

    #[test]
    fn insert_and_get_by_aterm_hash() {
        let mut cc = ConversionCache::default();
        let aterm = [1u8; 32];
        let hdm = [2u8; 32];
        let path = fake_store_path("foo.drv");
        let drv = dummy_derivation("foo");

        cc.insert(aterm, path.clone(), hdm, drv);

        let entry = cc.get_by_aterm_hash(&aterm).unwrap();
        assert_eq!(entry.drv_path, path);
        assert_eq!(entry.hash_derivation_modulo, hdm);
    }

    #[test]
    fn get_hdm_by_drv_path() {
        let mut cc = ConversionCache::default();
        let aterm = [1u8; 32];
        let hdm = [2u8; 32];
        let path = fake_store_path("bar.drv");

        cc.insert(aterm, path.clone(), hdm, dummy_derivation("bar"));

        let got = cc.get_hdm_by_drv_path(&path.to_absolute_path()).unwrap();
        assert_eq!(got, hdm);
    }

    #[test]
    fn get_by_drv_path_indexed() {
        let mut cc = ConversionCache::default();
        let aterm = [3u8; 32];
        let hdm = [4u8; 32];
        let path = fake_store_path("baz.drv");
        let drv = dummy_derivation("baz");

        cc.insert(aterm, path.clone(), hdm, drv);

        let entry = cc.get_by_drv_path(&path.to_absolute_path()).unwrap();
        assert_eq!(entry.drv_path, path);
        assert_eq!(entry.hash_derivation_modulo, hdm);
    }

    #[test]
    fn get_by_drv_path_unknown_returns_none() {
        let cc = ConversionCache::default();
        assert!(cc.get_by_drv_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nope.drv").is_none());
    }

    #[test]
    fn multiple_inserts_resolve_independently() {
        let mut cc = ConversionCache::default();

        let aterm_a = [10u8; 32];
        let hdm_a = [11u8; 32];
        let path_a = fake_store_path("alpha.drv");

        let aterm_b = [20u8; 32];
        let hdm_b = [21u8; 32];
        let path_b = fake_store_path("beta.drv");

        cc.insert(aterm_a, path_a.clone(), hdm_a, dummy_derivation("alpha"));
        cc.insert(aterm_b, path_b.clone(), hdm_b, dummy_derivation("beta"));

        let ea = cc.get_by_drv_path(&path_a.to_absolute_path()).unwrap();
        assert_eq!(ea.hash_derivation_modulo, hdm_a);

        let eb = cc.get_by_drv_path(&path_b.to_absolute_path()).unwrap();
        assert_eq!(eb.hash_derivation_modulo, hdm_b);
    }

    #[test]
    fn overwrite_same_drv_path_updates_index() {
        let mut cc = ConversionCache::default();
        let path = fake_store_path("reuse.drv");

        let aterm_1 = [30u8; 32];
        let hdm_1 = [31u8; 32];
        cc.insert(aterm_1, path.clone(), hdm_1, dummy_derivation("reuse"));

        let aterm_2 = [40u8; 32];
        let hdm_2 = [41u8; 32];
        cc.insert(aterm_2, path.clone(), hdm_2, dummy_derivation("reuse"));

        let entry = cc.get_by_drv_path(&path.to_absolute_path()).unwrap();
        assert_eq!(entry.hash_derivation_modulo, hdm_2);
    }

    #[test]
    fn cycle_detection_begin_returns_false_on_reentry() {
        let mut cc = ConversionCache::default();
        assert!(cc.begin_conversion("foo:bar:baz"));
        assert!(!cc.begin_conversion("foo:bar:baz"));
    }

    #[test]
    fn cycle_detection_end_allows_reentry() {
        let mut cc = ConversionCache::default();
        assert!(cc.begin_conversion("foo:bar:baz"));
        cc.end_conversion("foo:bar:baz");
        assert!(cc.begin_conversion("foo:bar:baz"));
    }

    #[test]
    fn iter_entries_yields_all() {
        let mut cc = ConversionCache::default();

        cc.insert([1u8; 32], fake_store_path("a.drv"), [10u8; 32], dummy_derivation("a"));
        cc.insert_ca(InsertCaEntry {
            aterm_hash: [2u8; 32],
            drv_path: fake_store_path("b.drv"),
            hdm: [20u8; 32],
            derivation: dummy_derivation("b"),
            content_addressed: true,
            dynamic_plan_outputs: vec!["out".to_string()],
            provenance_claims: None,
        });

        let entries: Vec<_> = cc.iter_entries().collect();
        assert_eq!(entries.len(), 2);

        // One should be CA, one not.
        let ca_count = entries.iter().filter(|(_, _, _, ca, _, _)| *ca).count();
        assert_eq!(ca_count, 1);
    }

    #[test]
    fn drain_pending_returns_new_entries() {
        let mut cc = ConversionCache::default();
        assert_eq!(cc.pending_count(), 0);

        cc.insert([1u8; 32], fake_store_path("a.drv"), [10u8; 32], dummy_derivation("a"));
        cc.insert([2u8; 32], fake_store_path("b.drv"), [20u8; 32], dummy_derivation("b"));
        assert_eq!(cc.pending_count(), 2);

        let batch = cc.drain_pending();
        assert_eq!(batch.len(), 2);
        assert_eq!(cc.pending_count(), 0);

        // Second drain returns nothing.
        let batch2 = cc.drain_pending();
        assert!(batch2.is_empty());
    }

    #[test]
    fn drain_pending_incremental() {
        let mut cc = ConversionCache::default();

        // First insert + drain.
        cc.insert([1u8; 32], fake_store_path("a.drv"), [10u8; 32], dummy_derivation("a"));
        let batch1 = cc.drain_pending();
        assert_eq!(batch1.len(), 1);

        // Second insert + drain only gets the new entry.
        cc.insert([2u8; 32], fake_store_path("b.drv"), [20u8; 32], dummy_derivation("b"));
        let batch2 = cc.drain_pending();
        assert_eq!(batch2.len(), 1);

        // iter_entries still yields all 2.
        let all: Vec<_> = cc.iter_entries().collect();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn drain_pending_ca_entries_marked() {
        let mut cc = ConversionCache::default();

        cc.insert_ca(InsertCaEntry {
            aterm_hash: [1u8; 32],
            drv_path: fake_store_path("ca.drv"),
            hdm: [10u8; 32],
            derivation: dummy_derivation("ca"),
            content_addressed: true,
            dynamic_plan_outputs: vec!["out".to_string()],
            provenance_claims: None,
        });
        let batch = cc.drain_pending();
        assert_eq!(batch.len(), 1);
        assert!(batch[0].3, "CA flag should be true");
        assert_eq!(batch[0].4, vec!["out".to_string()]);
    }
}
