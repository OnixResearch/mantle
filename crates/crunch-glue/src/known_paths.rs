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
    /// Track in-progress conversions for cycle detection.
    /// Key is an opaque identity derived from the CrunchDerivation name + builder + system.
    in_progress: std::collections::HashSet<String>,
}

pub struct KnownEntry {
    pub drv_path: StorePath<String>,
    pub hash_derivation_modulo: [u8; 32],
    pub derivation: Derivation,
}

impl KnownPaths {
    pub fn new() -> Self {
        Self {
            by_aterm_hash: HashMap::new(),
            hdm_by_drv_path: HashMap::new(),
            in_progress: std::collections::HashSet::new(),
        }
    }

    /// Register a fully-converted derivation.
    pub fn insert(
        &mut self,
        aterm_hash: [u8; 32],
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: Derivation,
    ) {
        let drv_path_str = drv_path.to_absolute_path();
        self.hdm_by_drv_path.insert(drv_path_str, hdm);
        self.by_aterm_hash.insert(aterm_hash, KnownEntry {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation,
        });
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
        self.by_aterm_hash
            .values()
            .find(|e| e.drv_path.to_absolute_path() == drv_path)
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
        Self::new()
    }
}
