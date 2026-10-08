// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in the store-capability-migration change evidence and scheduled for the
// standalone hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::too_many_parameters)]

//! Build-time derivation registry.
//!
//! `DerivationRegistry` holds all derivations the build engine needs
//! to schedule and execute. It is populated by the pipeline layer
//! (from `ConversionCache` output) before builds start, and updated
//! at runtime for dynamic derivations and CA output resolution.
//!
//! This module has no dependency on crunch-glue. The build engine
//! only needs `nix_compat::Derivation` and associated store paths.

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

use crunch_attestation::Claims;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;

use crate::ExecutionProfile;
use crate::dynamic::DynamicRegistrationDecision;
use crate::dynamic::RegistryReadyDynamicDerivation;
use crate::validate_execution_profile;
use crate::verify_execution_profile_binding;

/// Maximum registered derivation metadata entries before watch admission.
pub(crate) const MAX_ENTRIES: u32 = 16_384;

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
    /// Explicit execution policy used when the worker creates a build request.
    pub execution_profile: ExecutionProfile,
    /// Full staged identity for dynamically admitted registry entries.
    pub dynamic_admission_identity: Option<[u8; 32]>,
    /// Resolved output paths for CA derivations. Populated by
    /// `resolve_output()` after the build completes.
    /// Key: output name, Value: final store path.
    pub resolved_outputs: HashMap<String, StorePath<String>>,
    /// Exact action form admitted for this goal after CA prerequisites complete.
    /// The original entry and key remain the scheduler's identity.
    pub resolved_derivation: Option<Arc<Derivation>>,
    pub resolved_drv_path: Option<StorePath<String>>,
}

struct RegistryInsert {
    drv_path: StorePath<String>,
    hash_derivation_modulo: [u8; 32],
    derivation: Arc<Derivation>,
    content_addressed: bool,
    dynamic_plan_outputs: Vec<String>,
    provenance_claims: Option<Claims>,
    execution_profile: ExecutionProfile,
    dynamic_admission_identity: Option<[u8; 32]>,
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
        self.insert_registration(RegistryInsert {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation: derivation.into(),
            content_addressed,
            dynamic_plan_outputs: Vec::new(),
            provenance_claims,
            execution_profile: ExecutionProfile::native_compatibility(),
            dynamic_admission_identity: None,
        });
    }

    /// The native plan owns this fully admitted derivation; move it into the
    /// registry rather than copying every argument, environment and input.
    pub(crate) fn insert_registry_ready_dynamic(
        &mut self,
        ready: RegistryReadyDynamicDerivation,
        dynamic_plan_outputs: Vec<String>,
    ) -> Result<(), crate::Error> {
        if !self.registry_ready_is_new(&ready, &dynamic_plan_outputs)? {
            return Ok(());
        }
        let (drv_path, hash_derivation_modulo, derivation, content_addressed, full_identity) =
            ready.into_registration();
        self.insert_registration(RegistryInsert {
            drv_path,
            hash_derivation_modulo,
            derivation: Arc::new(derivation),
            content_addressed,
            dynamic_plan_outputs,
            provenance_claims: None,
            execution_profile: ExecutionProfile::native_compatibility(),
            dynamic_admission_identity: Some(full_identity),
        });
        Ok(())
    }

    /// Traditional `.drv` discovery also retains the owned derivation for the
    /// scheduler, so only this path must copy it into the registry.
    pub(crate) fn insert_registry_ready_discovered(
        &mut self,
        ready: &RegistryReadyDynamicDerivation,
    ) -> Result<(), crate::Error> {
        if !self.registry_ready_is_new(ready, &[])? {
            return Ok(());
        }
        self.insert_registration(RegistryInsert {
            drv_path: ready.drv_path().clone(),
            hash_derivation_modulo: ready.hash_derivation_modulo(),
            derivation: Arc::new(ready.derivation().clone()),
            content_addressed: ready.content_addressed(),
            dynamic_plan_outputs: Vec::new(),
            provenance_claims: None,
            execution_profile: ExecutionProfile::native_compatibility(),
            dynamic_admission_identity: Some(ready.full_identity()),
        });
        Ok(())
    }

    fn registry_ready_is_new(
        &self,
        ready: &RegistryReadyDynamicDerivation,
        dynamic_plan_outputs: &[String],
    ) -> Result<bool, crate::Error> {
        let absolute = ready.drv_path().to_absolute_path_with_prefix(&self.store_dir);
        if ready.decision() == DynamicRegistrationDecision::AlreadyPresent {
            let existing = self.entries.get(&absolute).ok_or_else(|| {
                crate::Error::Store(format!("dynamic duplicate `{absolute}` is absent from the registry"))
            })?;
            if existing.dynamic_admission_identity != Some(ready.full_identity()) {
                return Err(crate::Error::Store(format!("dynamic duplicate `{absolute}` changed admitted identity")));
            }
            if existing.dynamic_plan_outputs.as_slice() != dynamic_plan_outputs {
                return Err(crate::Error::Store(format!("dynamic duplicate `{absolute}` changed plan outputs")));
            }
            return Ok(false);
        }
        if self.entries.contains_key(&absolute) {
            return Err(crate::Error::Store(format!(
                "dynamic insertion `{absolute}` collides with an existing registry entry"
            )));
        }
        Ok(true)
    }

    /// Register an already admitted static/evaluation derivation carrying plan outputs.
    /// Native dynamic-plan units instead enter through `insert_registry_ready_dynamic`,
    /// which retains their complete staged identity and refuses collisions.
    #[allow(tigerstyle::too_many_parameters)] // Stable evaluation registration boundary.
    pub fn insert_with_dynamic_plan_outputs(
        &mut self,
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: impl Into<Arc<Derivation>>,
        content_addressed: bool,
        dynamic_plan_outputs: Vec<String>,
        provenance_claims: Option<Claims>,
    ) {
        self.insert_registration(RegistryInsert {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation: derivation.into(),
            content_addressed,
            dynamic_plan_outputs,
            provenance_claims,
            execution_profile: ExecutionProfile::native_compatibility(),
            dynamic_admission_identity: None,
        });
    }

    /// Register a derivation with an explicit identity-checked execution profile.
    pub fn insert_with_execution_profile(
        &mut self,
        drv_path: StorePath<String>,
        hdm: [u8; 32],
        derivation: impl Into<Arc<Derivation>>,
        content_addressed: bool,
        provenance_claims: Option<Claims>,
        execution_profile: ExecutionProfile,
    ) -> Result<(), crate::Error> {
        let derivation = derivation.into();
        validate_execution_profile(&execution_profile)
            .map_err(|error| crate::Error::Store(format!("invalid execution profile: {error}")))?;
        verify_execution_profile_binding(derivation.as_ref(), &execution_profile)
            .map_err(|error| crate::Error::Store(format!("invalid execution profile binding: {error}")))?;
        self.insert_registration(RegistryInsert {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation,
            content_addressed,
            dynamic_plan_outputs: Vec::new(),
            provenance_claims,
            execution_profile,
            dynamic_admission_identity: None,
        });
        Ok(())
    }

    fn insert_registration(&mut self, registration: RegistryInsert) {
        let RegistryInsert {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation,
            content_addressed,
            dynamic_plan_outputs,
            provenance_claims,
            execution_profile,
            dynamic_admission_identity,
        } = registration;
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
            execution_profile,
            dynamic_admission_identity,
            resolved_outputs: HashMap::new(),
            resolved_derivation: None,
            resolved_drv_path: None,
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

    /// Prune a settled watch generation without orphaning a retained
    /// derivation or its resolved CA identity. The caller includes every
    /// admitted root dependency, pending teardown fence, and dynamic unit.
    pub fn retain_watch_derivations(&mut self, live: &BTreeSet<String>) -> Result<(), crate::Error> {
        for path in live {
            let entry = self
                .entries
                .get(path)
                .ok_or_else(|| crate::Error::Store(format!("watch retained derivation not registered: {path}")))?;
            for derivation in std::iter::once(entry.derivation.as_ref()).chain(entry.resolved_derivation.as_deref()) {
                for input in derivation.input_derivations.keys() {
                    let dependency = input.to_absolute_path_with_prefix(&self.store_dir);
                    if self.entries.contains_key(&dependency) && !live.contains(&dependency) {
                        return Err(crate::Error::Store(format!(
                            "watch cannot prune retained derivation input {dependency} referenced by {path}",
                        )));
                    }
                }
            }
        }
        self.entries.retain(|path, _| live.contains(path));
        self.hdm_by_drv_path.retain(|path, _| live.contains(path));
        Ok(())
    }

    pub fn record_resolved_derivation(
        &mut self,
        drv_abs: &str,
        resolved: Arc<Derivation>,
        resolved_path: StorePath<String>,
    ) -> Result<(), crate::Error> {
        let entry = self
            .entries
            .get_mut(drv_abs)
            .ok_or_else(|| crate::Error::Store(format!("ca-input-unrealized: derivation not registered: {drv_abs}")))?;
        entry.resolved_derivation = Some(resolved);
        entry.resolved_drv_path = Some(resolved_path);
        Ok(())
    }

    /// Get the resolved output path for a derivation.
    ///
    /// For input-addressed derivations, returns the prepared resolved output
    /// when CA prerequisites changed the action identity, otherwise the
    /// precomputed output. CA derivations use `resolve_output()`.
    pub fn get_output_path(&self, drv_abs: &str, output_name: &str) -> Option<StorePath<String>> {
        let entry = self.entries.get(drv_abs)?;
        if entry.content_addressed {
            entry.resolved_outputs.get(output_name).cloned()
        } else {
            entry
                .resolved_derivation
                .as_deref()
                .unwrap_or(entry.derivation.as_ref())
                .outputs
                .get(output_name)
                .and_then(|output| output.path.clone())
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
        registry.insert_registration(RegistryInsert {
            drv_path,
            hash_derivation_modulo: hdm,
            derivation: Arc::new(derivation),
            content_addressed,
            dynamic_plan_outputs,
            provenance_claims,
            execution_profile: ExecutionProfile::native_compatibility(),
            dynamic_admission_identity: None,
        });
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
    fn explicit_execution_profile_is_verified_and_stored() {
        let mut registry = DerivationRegistry::default();
        let drv_path = fake_sp("foreign-profile.drv");
        let profile = ExecutionProfile::foreign_nix();
        let mut derivation = dummy_derivation();
        crate::bind_execution_profile(&mut derivation, &profile).unwrap();

        registry
            .insert_with_execution_profile(drv_path.clone(), [8u8; 32], derivation, false, None, profile.clone())
            .unwrap();

        let entry = registry.get_by_drv_path(&drv_path.to_absolute_path()).unwrap();
        assert_eq!(entry.execution_profile, profile);
    }

    #[test]
    fn explicit_execution_profile_rejects_stale_binding() {
        let mut registry = DerivationRegistry::default();
        let drv_path = fake_sp("stale-profile.drv");
        let guix = ExecutionProfile::foreign_guix();
        let nix = ExecutionProfile::foreign_nix();
        let mut derivation = dummy_derivation();
        derivation.builder = "/mantle/store/00000000000000000000000000000000-builder".to_string();
        crate::bind_execution_profile(&mut derivation, &guix).unwrap();

        let error = registry
            .insert_with_execution_profile(drv_path, [9u8; 32], derivation, false, None, nix)
            .unwrap_err();

        assert!(error.to_string().contains("binding digest does not match"));
        assert!(registry.is_empty());
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

    // r[verify mantle.ca_input_resolution.resolved_identity]
    #[test]
    fn ca_resolved_input_addressed_output_path_uses_completed_derivation() {
        let mut registry = DerivationRegistry::default();
        let original_path = fake_sp("ca-dependent-ia.drv");
        let original_output = fake_sp("provisional-ia-output");
        let resolved_output = fake_sp("realized-ia-output");
        let mut original = dummy_derivation();
        original.outputs.get_mut("out").unwrap().path = Some(original_output.clone());
        registry.insert(original_path.clone(), [5u8; 32], original, false, None);

        let original_abs = original_path.to_absolute_path();
        assert_eq!(registry.get_output_path(&original_abs, "out"), Some(original_output.clone()));
        let mut resolved = dummy_derivation();
        resolved.outputs.get_mut("out").unwrap().path = Some(resolved_output.clone());
        registry
            .record_resolved_derivation(&original_abs, Arc::new(resolved), fake_sp("ca-dependent-ia-resolved.drv"))
            .unwrap();

        assert_eq!(
            registry.get_output_path(&original_abs, "out"),
            Some(resolved_output),
            "plan placeholders and dependents must see the CA-resolved IA output",
        );
        assert_eq!(
            registry.get_by_drv_path(&original_abs).unwrap().derivation.outputs["out"].path,
            Some(original_output),
            "the scheduler retains the unresolved derivation separately",
        );
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

    #[test]
    fn watch_prune_preserves_referenced_parent_ca_output_and_rejects_orphaning() {
        let mut registry = DerivationRegistry::default();
        let parent = fake_sp("watch-parent.drv");
        let child = fake_sp("watch-child.drv");
        let stale = fake_sp("watch-stale.drv");
        registry.insert(parent.clone(), [1; 32], dummy_derivation(), true, None);
        let resolved = fake_sp("watch-resolved");
        registry.resolve_output(&parent.to_absolute_path(), "out", resolved.clone()).unwrap();
        let mut dependent = dummy_derivation();
        dependent.input_derivations.insert(parent.clone(), BTreeSet::from(["out".to_string()]));
        registry.insert(child.clone(), [2; 32], dependent, false, None);
        registry.insert(stale.clone(), [3; 32], dummy_derivation(), false, None);

        let child_only = BTreeSet::from([child.to_absolute_path()]);
        assert!(registry.retain_watch_derivations(&child_only).is_err());
        assert_eq!(registry.len(), 3, "rejected prune must not mutate either index");
        let live = BTreeSet::from([parent.to_absolute_path(), child.to_absolute_path()]);
        registry.retain_watch_derivations(&live).unwrap();
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get_output_path(&parent.to_absolute_path(), "out"), Some(resolved));
        assert_eq!(registry.get_hdm_by_drv_path(&parent.to_absolute_path()), Some([1; 32]));
        assert_eq!(registry.get_hdm_by_drv_path(&stale.to_absolute_path()), None);
        registry.retain_watch_derivations(&BTreeSet::new()).unwrap();
        assert!(registry.is_empty());
        assert_eq!(registry.get_hdm_by_drv_path(&child.to_absolute_path()), None);
    }

    #[test]
    fn watch_prune_bounds_registry_through_more_than_max_entries_of_edits() {
        let mut registry = DerivationRegistry::default();
        let mut previous = None;
        for generation in 0..=MAX_ENTRIES {
            let path = fake_sp(&format!("watch-{generation:05}.drv"));
            registry.insert(path.clone(), [generation as u8; 32], dummy_derivation(), false, None);
            registry.retain_watch_derivations(&BTreeSet::from([path.to_absolute_path()])).unwrap();
            assert_eq!(registry.len(), 1);
            if let Some(previous) = previous.replace(path) {
                assert!(registry.get_by_drv_path(&previous.to_absolute_path()).is_none());
                assert!(registry.get_hdm_by_drv_path(&previous.to_absolute_path()).is_none());
            }
        }
    }
}
