//! Build-time derivation registry.
//!
//! `DerivationRegistry` holds all derivations the build engine needs
//! to schedule and execute. It is populated by the pipeline layer
//! (from `ConversionCache` output) before builds start, and updated
//! at runtime for dynamic derivations and CA output resolution.
//!
//! This module has no dependency on crunch-glue. The build engine
//! only needs `nix_compat::Derivation` and associated store paths.

use std::collections::HashMap;
use std::sync::Arc;

use crunch_attestation::Claims;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;

/// Maximum registry entries. Matches `goal::MAX_GOALS`.
const MAX_ENTRIES: u32 = 16_384;

/// A derivation registered for building.
pub struct RegistryEntry {
    pub drv_path: StorePath<String>,
    pub hash_derivation_modulo: [u8; 32],
    pub derivation: Arc<Derivation>,
    /// Whether this is a content-addressed derivation (output paths
    /// resolved after build).
    pub content_addressed: bool,
    pub dynamic_plan_outputs: Vec<String>,
    pub provenance_claims: Option<Claims>,
    /// Resolved output paths for CA derivations. Populated by
    /// `resolve_output()` after the build completes.
    /// Key: output name, Value: final store path.
    pub resolved_outputs: HashMap<String, StorePath<String>>,
}

/// Build-time derivation lookup for the Worker and Builder.
///
/// Indexed by absolute derivation store path (e.g.,
/// "/nix/store/abc...-foo.drv"). No conversion-time state lives
/// here -- no ATerm dedup, no cycle detection.
pub struct DerivationRegistry {
    /// drv absolute path -> entry.
    entries: HashMap<String, RegistryEntry>,
    /// Reverse: drv absolute path -> HDM (fast lookup for
    /// `hash_derivation_modulo` callbacks during dynamic drv
    /// registration).
    hdm_by_drv_path: HashMap<String, [u8; 32]>,
    /// Store directory prefix (e.g., "/nix/store").
    store_dir: String,
}

impl DerivationRegistry {
    pub fn new(store_dir: &str) -> Self {
        debug_assert!(!store_dir.is_empty(), "store_dir must not be empty");
        debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute");

        Self {
            entries: HashMap::new(),
            hdm_by_drv_path: HashMap::new(),
            store_dir: store_dir.to_string(),
        }
    }

    /// The store directory prefix.
    pub fn store_dir(&self) -> &str {
        &self.store_dir
    }

    /// Number of entries.
    pub fn len(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Register a derivation. The `drv_path` is converted to an
    /// absolute path using the registry's store_dir for the key.
    pub fn insert(
        &mut self,
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: impl Into<Arc<Derivation>>,
        content_addressed: bool,
        provenance_claims: Option<Claims>,
    ) {
        self.insert_with_dynamic_plan_outputs(
            drv_path,
            hdm,
            derivation,
            content_addressed,
            Vec::new(),
            provenance_claims,
        );
    }

    /// Register a derivation with native dynamic-plan output metadata.
    pub fn insert_with_dynamic_plan_outputs(
        &mut self,
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: impl Into<Arc<Derivation>>,
        content_addressed: bool,
        dynamic_plan_outputs: Vec<String>,
        provenance_claims: Option<Claims>,
    ) {
        let derivation = derivation.into();
        debug_assert!(!derivation.outputs.is_empty(), "derivation must have at least one output");
        debug_assert!(
            dynamic_plan_outputs.iter().all(|output| derivation.outputs.contains_key(output)),
            "dynamic plan outputs must be declared derivation outputs"
        );
        debug_assert!(
            u32::try_from(self.entries.len()).is_ok_and(|n| n < MAX_ENTRIES),
            "registry exceeds MAX_ENTRIES ({MAX_ENTRIES})"
        );

        let drv_abs = drv_path.to_absolute_path_with_prefix(&self.store_dir);
        self.hdm_by_drv_path.insert(drv_abs.clone(), hdm);
        self.entries.insert(drv_abs, RegistryEntry {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation,
            content_addressed,
            dynamic_plan_outputs,
            provenance_claims,
            resolved_outputs: HashMap::new(),
        });
    }

    /// Look up a `RegistryEntry` by absolute drv store path.
    pub fn get_by_drv_path(&self, drv_abs: &str) -> Option<&RegistryEntry> {
        self.entries.get(drv_abs)
    }

    /// Mutable lookup by absolute drv store path.
    pub fn get_by_drv_path_mut(&mut self, drv_abs: &str) -> Option<&mut RegistryEntry> {
        self.entries.get_mut(drv_abs)
    }

    /// Look up `hash_derivation_modulo` by absolute drv store path.
    ///
    /// Used as the callback for `Derivation::hash_derivation_modulo()`
    /// when registering dynamic derivations.
    pub fn get_hdm_by_drv_path(&self, drv_abs: &str) -> Option<[u8; 32]> {
        self.hdm_by_drv_path.get(drv_abs).copied()
    }

    /// Get the resolved output path for a derivation.
    ///
    /// For input-addressed derivations, returns the pre-computed path
    /// from `derivation.outputs`. For CA derivations, returns the
    /// resolved path set by `resolve_output()`.
    pub fn get_output_path(&self, drv_abs: &str, output_name: &str) -> Option<StorePath<String>> {
        let entry = self.entries.get(drv_abs)?;
        if entry.content_addressed {
            entry.resolved_outputs.get(output_name).cloned()
        } else {
            entry.derivation.outputs.get(output_name).and_then(|o| o.path.clone())
        }
    }

    /// Register a CA derivation's resolved output path after build.
    ///
    /// Returns an error if `drv_abs` is not registered or is not a CA
    /// derivation.
    pub fn resolve_output(
        &mut self,
        drv_abs: &str,
        output_name: &str,
        final_path: StorePath<String>,
    ) -> Result<(), crate::Error> {
        let entry = self
            .entries
            .get_mut(drv_abs)
            .ok_or_else(|| crate::Error::Store(format!("resolve_output: derivation not registered: {drv_abs}")))?;
        if !entry.content_addressed {
            return Err(crate::Error::Store(format!("resolve_output called on non-CA derivation: {drv_abs}")));
        }
        entry.resolved_outputs.insert(output_name.to_string(), final_path);
        Ok(())
    }
}

impl Default for DerivationRegistry {
    fn default() -> Self {
        Self::new(nix_compat::store_path::STORE_DIR)
    }
}

/// Populate a `DerivationRegistry` from an iterator of conversion
/// output tuples. This is the bridge between `ConversionCache`
/// (crunch-glue) and the build engine.
///
/// The caller iterates the cache and feeds entries here. The registry
/// and cache never see each other directly.
pub fn populate_registry<I>(registry: &mut DerivationRegistry, entries: I)
where I: IntoIterator<Item = (StorePath<String>, [u8; 32], Derivation, bool, Vec<String>, Option<Claims>)> {
    for (drv_path, hdm, derivation, content_addressed, dynamic_plan_outputs, provenance_claims) in entries {
        registry.insert_with_dynamic_plan_outputs(
            drv_path,
            hdm,
            derivation,
            content_addressed,
            dynamic_plan_outputs,
            provenance_claims,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use nix_compat::derivation::Output;

    use super::*;

    fn dummy_derivation() -> Derivation {
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

    fn fake_sp(name: &str) -> StorePath<String> {
        let mut digest = [0u8; 20];
        for (i, b) in name.bytes().enumerate() {
            digest[i % 20] ^= b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    #[test]
    fn insert_and_get_by_drv_path() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("foo.drv");
        let hdm = [1u8; 32];

        reg.insert(sp.clone(), hdm, dummy_derivation(), false, None);

        let abs = sp.to_absolute_path();
        let entry = reg.get_by_drv_path(&abs).unwrap();
        assert_eq!(entry.drv_path, sp);
        assert_eq!(entry.hash_derivation_modulo, hdm);
        assert!(!entry.content_addressed);
        assert!(entry.dynamic_plan_outputs.is_empty());
    }

    #[test]
    fn insert_with_dynamic_plan_outputs_preserves_metadata() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("plan.drv");
        let hdm = [7u8; 32];
        let mut drv = dummy_derivation();
        drv.outputs.insert("plan".to_string(), Output {
            path: None,
            ca_hash: None,
        });

        reg.insert_with_dynamic_plan_outputs(sp.clone(), hdm, drv, false, vec!["plan".to_string()], None);

        let entry = reg.get_by_drv_path(&sp.to_absolute_path()).unwrap();
        assert_eq!(entry.dynamic_plan_outputs, vec!["plan".to_string()]);
        assert!(entry.derivation.outputs.contains_key("plan"));
    }

    #[test]
    fn get_hdm_by_drv_path() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("bar.drv");
        let hdm = [2u8; 32];

        reg.insert(sp.clone(), hdm, dummy_derivation(), false, None);

        let abs = sp.to_absolute_path();
        assert_eq!(reg.get_hdm_by_drv_path(&abs), Some(hdm));
    }

    #[test]
    fn unknown_drv_returns_none() {
        let reg = DerivationRegistry::default();
        assert!(reg.get_by_drv_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nope.drv").is_none());
        assert!(reg.get_hdm_by_drv_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nope.drv").is_none());
    }

    #[test]
    fn get_output_path_input_addressed() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("ia.drv");
        let out_sp = fake_sp("ia-out");

        let mut drv = dummy_derivation();
        drv.outputs.insert("out".to_string(), Output {
            path: Some(out_sp.clone()),
            ca_hash: None,
        });

        reg.insert(sp.clone(), [3u8; 32], drv, false, None);

        let abs = sp.to_absolute_path();
        assert_eq!(reg.get_output_path(&abs, "out"), Some(out_sp));
        assert_eq!(reg.get_output_path(&abs, "dev"), None);
    }

    #[test]
    fn resolve_output_ca() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("ca.drv");
        let final_sp = fake_sp("ca-resolved");

        reg.insert(sp.clone(), [4u8; 32], dummy_derivation(), true, None);

        let abs = sp.to_absolute_path();
        // Before resolve: no output path (outputs have path: None).
        assert_eq!(reg.get_output_path(&abs, "out"), None);

        reg.resolve_output(&abs, "out", final_sp.clone()).unwrap();

        assert_eq!(reg.get_output_path(&abs, "out"), Some(final_sp));
    }

    #[test]
    fn multiple_entries() {
        let mut reg = DerivationRegistry::default();

        let sp_a = fake_sp("alpha.drv");
        let sp_b = fake_sp("beta.drv");
        let hdm_a = [10u8; 32];
        let hdm_b = [20u8; 32];

        reg.insert(sp_a.clone(), hdm_a, dummy_derivation(), false, None);
        reg.insert(sp_b.clone(), hdm_b, dummy_derivation(), true, None);

        assert_eq!(reg.len(), 2);

        let ea = reg.get_by_drv_path(&sp_a.to_absolute_path()).unwrap();
        assert_eq!(ea.hash_derivation_modulo, hdm_a);
        assert!(!ea.content_addressed);

        let eb = reg.get_by_drv_path(&sp_b.to_absolute_path()).unwrap();
        assert_eq!(eb.hash_derivation_modulo, hdm_b);
        assert!(eb.content_addressed);
    }

    #[test]
    fn store_dir_custom() {
        let reg = DerivationRegistry::new("/opt/crunch");
        assert_eq!(reg.store_dir(), "/opt/crunch");
    }

    #[test]
    fn overwrite_updates_entry() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("reuse.drv");

        reg.insert(sp.clone(), [30u8; 32], dummy_derivation(), false, None);
        reg.insert(sp.clone(), [40u8; 32], dummy_derivation(), true, None);

        let entry = reg.get_by_drv_path(&sp.to_absolute_path()).unwrap();
        assert_eq!(entry.hash_derivation_modulo, [40u8; 32]);
        assert!(entry.content_addressed);
    }

    #[test]
    fn populate_registry_from_iterator() {
        let mut reg = DerivationRegistry::default();

        let entries = vec![
            (fake_sp("a.drv"), [1u8; 32], dummy_derivation(), false, Vec::new(), None),
            (fake_sp("b.drv"), [2u8; 32], dummy_derivation(), true, vec!["out".to_string()], None),
        ];

        populate_registry(&mut reg, entries);

        assert_eq!(reg.len(), 2);
        assert!(reg.get_by_drv_path(&fake_sp("a.drv").to_absolute_path()).is_some());
        assert!(reg.get_by_drv_path(&fake_sp("b.drv").to_absolute_path()).is_some());
    }

    #[test]
    fn default_store_dir() {
        let reg = DerivationRegistry::default();
        assert_eq!(reg.store_dir(), "/nix/store");
    }

    #[test]
    fn is_empty_and_len() {
        let mut reg = DerivationRegistry::default();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);

        reg.insert(fake_sp("x.drv"), [0u8; 32], dummy_derivation(), false, None);
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn get_by_drv_path_mut_modifies() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("mut.drv");

        reg.insert(sp.clone(), [5u8; 32], dummy_derivation(), false, None);

        let abs = sp.to_absolute_path();
        let entry = reg.get_by_drv_path_mut(&abs).unwrap();
        entry.content_addressed = true;

        assert!(reg.get_by_drv_path(&abs).unwrap().content_addressed);
    }

    #[test]
    fn resolve_output_unknown_drv_returns_error() {
        let mut reg = DerivationRegistry::default();
        let bogus = "/nix/store/00000000000000000000000000000000-ghost.drv";
        let sp = fake_sp("resolved");

        let result = reg.resolve_output(bogus, "out", sp);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not registered"), "error should say 'not registered': {msg}");
    }

    #[test]
    fn resolve_output_non_ca_returns_error() {
        let mut reg = DerivationRegistry::default();
        let sp = fake_sp("ia.drv");
        reg.insert(sp.clone(), [6u8; 32], dummy_derivation(), false, None);

        let abs = sp.to_absolute_path();
        let result = reg.resolve_output(&abs, "out", fake_sp("resolved"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("non-CA"), "error should say 'non-CA': {msg}");
    }
}
