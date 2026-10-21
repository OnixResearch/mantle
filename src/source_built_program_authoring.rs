//! Preparation-only source-program authoring. These products are inert source;
//! admission and actual execution remain the existing cold-route authorities.
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_recipe::ArtifactRef;

pub(crate) struct AuthoringRequest<'a> {
    pub proof: &'a SourceBuiltFixedPointPlan,
    pub source_roots: &'a BTreeMap<SourceAuthorityRole, PathBuf>,
    /// Exact predecessor artifact references, keyed `<stage-id>:<export-name>`.
    pub predecessor_exports: &'a BTreeMap<String, ArtifactRef>,
    pub scratch_root: &'a Path,
    pub targets: &'a [String],
}

/// UTF-8 source files only. No prepared headers, objects, binaries, transcripts,
/// execution envelopes or measured future output identities belong here.
pub(crate) struct AuthoredPrograms {
    pub files: BTreeMap<String, String>,
    /// Exact exports for subsequent authoring, with the same key convention.
    pub exports: BTreeMap<String, ArtifactRef>,
}

pub(crate) const CONTINUATION_POLICY_PATH: &str = "bootstrap/source-built-continuations.ncl";
const CONTINUATION_POLICY_SCHEMA: &str = "mantle-source-continuation-reservations-v1";
const POLICY_BYTES_MAX: u64 = 1024 * 1024;

/// Resource reservations, not a proof of macro expansion or native command counts.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContinuationReservationPolicy {
    schema: String,
    macro_base_slots: u32,
    macro_slots_per_source_site: u32,
    native_base_slots: u32,
    native_slots_per_source_file: u32,
    overrides: BTreeMap<String, u32>,
}

impl ContinuationReservationPolicy {
    pub(crate) fn macro_slots(&self, phase_id: &str, source_sites: u32, slots_max: u32) -> Result<usize, RunError> {
        self.slots(phase_id, source_sites, self.macro_base_slots, self.macro_slots_per_source_site, slots_max)
    }

    pub(crate) fn native_slots(&self, phase_id: &str, source_files: u32, slots_max: u32) -> Result<usize, RunError> {
        self.slots(phase_id, source_files, self.native_base_slots, self.native_slots_per_source_file, slots_max)
    }

    fn slots(
        &self,
        phase_id: &str,
        source_hint: u32,
        base: u32,
        multiplier: u32,
        slots_max: u32,
    ) -> Result<usize, RunError> {
        if self.schema != CONTINUATION_POLICY_SCHEMA || phase_id.is_empty() || phase_id.contains('\0') || slots_max == 0
        {
            return Err(author_error("invalid source continuation reservation policy or phase"));
        }
        let slots = match self.overrides.get(phase_id) {
            Some(slots) => *slots,
            None => multiplier
                .checked_mul(source_hint)
                .and_then(|weighted| base.checked_add(weighted))
                .ok_or_else(|| author_error(format!("continuation reservation overflow for {phase_id}")))?,
        };
        if slots == 0 || slots > slots_max {
            return Err(author_error(format!(
                "phase {phase_id} requires {slots} reserved slots, available {slots_max}"
            )));
        }
        assert!(slots > 0);
        assert!(slots <= slots_max);
        usize::try_from(slots).map_err(|_| author_error("continuation reservation is not addressable"))
    }
}

pub(crate) fn read_continuation_policy(source_root: &Path) -> Result<ContinuationReservationPolicy, RunError> {
    let path = crate::source_built_recipe::source_member(source_root, CONTINUATION_POLICY_PATH)?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| author_error(error.to_string()))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > POLICY_BYTES_MAX {
        return Err(author_error("continuation policy must be bounded regular source text"));
    }
    crate::source_built_recipe::validate_source_imports(source_root, &path, &[])?;
    let policy: ContinuationReservationPolicy = crunch_eval::evaluate_and_deserialize(&path, &[])
        .map_err(|error| author_error(format!("continuation policy: {error}")))?;
    if policy.schema != CONTINUATION_POLICY_SCHEMA
        || policy.macro_base_slots == 0
        || policy.native_base_slots == 0
        || policy
            .overrides
            .iter()
            .any(|(phase, slots)| phase.is_empty() || phase.contains('\0') || *slots == 0)
    {
        return Err(author_error("invalid source continuation reservation policy"));
    }
    assert!(!policy.schema.is_empty());
    assert!(policy.macro_base_slots > 0 && policy.native_base_slots > 0);
    Ok(policy)
}

fn author_error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("source program authoring: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ContinuationReservationPolicy {
        serde_json::from_value(serde_json::json!({
            "schema": CONTINUATION_POLICY_SCHEMA,
            "macro_base_slots": 2, "macro_slots_per_source_site": 3,
            "native_base_slots": 4, "native_slots_per_source_file": 2,
            "overrides": { "selected.macro": 7 }
        }))
        .unwrap()
    }

    #[test]
    fn exact_override_precedes_source_hint_without_spending_above_budget() {
        let policy = policy();
        assert_eq!(policy.macro_slots("selected.macro", u32::MAX, 7).unwrap(), 7);
        assert!(policy.macro_slots("selected.macro", 0, 6).is_err());
        assert_eq!(policy.macro_slots("other.macro", 2, 8).unwrap(), 8);
        assert!(policy.macro_slots("other.macro", 2, 7).is_err());
    }

    #[test]
    fn reservations_reject_overflow_zero_and_invalid_phase_instead_of_clamping() {
        let mut policy = policy();
        assert!(policy.macro_slots("other.macro", u32::MAX, u32::MAX).is_err());
        assert_eq!(policy.native_slots("native", 2, 8).unwrap(), 8);
        assert!(policy.native_slots("native", 2, 7).is_err());
        assert!(policy.native_slots("", 0, 8).is_err());
        policy.overrides.insert("zero".into(), 0);
        assert!(policy.macro_slots("zero", 0, 8).is_err());
    }
}
