use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Digest;
use crate::Error;
use crate::MAX_MEMBERS;
use crate::Manifest;
use crate::identity;

/// Dimensions overlap; retries belong to the shell event ledger, not new members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub key: String,
    pub attempted: bool,
    pub executed: bool,
    pub reused: bool,
    pub fallback: Option<String>,
    pub unsupported: Option<String>,
    pub failed: bool,
    pub unrun: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleCoverage {
    pub role: String,
    pub candidates: u32,
    pub selected: u32,
    pub eligible: u32,
    pub attempted: u32,
    pub executed: u32,
    pub reused: u32,
    pub fallback: u32,
    pub unsupported: u32,
    pub excluded: u32,
    pub failed: u32,
    pub unrun: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageSummary {
    pub baseline: Digest,
    pub manifest: Digest,
    pub roles: Vec<RoleCoverage>,
    pub complete: bool,
    pub deficits: Vec<String>,
}

/// Reject any revised baseline, even if surviving members have identical totals.
/// Both manifest identities remain available from the independently retained files.
// r[impl rust_package_planning.differential_execution.population]
pub fn reconcile(
    baseline: &Manifest,
    manifest: &Manifest,
    observations: &[Coverage],
) -> Result<CoverageSummary, Error> {
    let baseline_id = baseline.identity()?;
    let manifest_id = manifest.identity()?;
    if baseline != manifest {
        return Err(Error::BaselineRewrite);
    }
    if observations.len() > MAX_MEMBERS {
        return Err(Error::Bound);
    }
    let mut seen = BTreeSet::new();
    for observed in observations {
        if !seen.insert(&observed.key) {
            return Err(Error::Duplicate);
        }
        if !manifest.population.iter().any(|m| m.key == observed.key) {
            return Err(Error::Missing);
        }
        validate_coverage(observed)?;
    }
    if seen.len() != manifest.population.len() {
        return Err(Error::Missing);
    }
    let mut roles = BTreeMap::<String, RoleCoverage>::new();
    let mut deficits = Vec::new();
    for member in &manifest.population {
        let observed = observations.iter().find(|v| v.key == member.key).ok_or(Error::Missing)?;
        let role = roles.entry(member.role.clone()).or_insert_with(|| RoleCoverage {
            role: member.role.clone(),
            candidates: 0,
            selected: 0,
            eligible: 0,
            attempted: 0,
            executed: 0,
            reused: 0,
            fallback: 0,
            unsupported: 0,
            excluded: 0,
            failed: 0,
            unrun: 0,
        });
        role.candidates += 1;
        role.selected += u32::from(member.selected);
        role.eligible += u32::from(member.selected && member.eligible);
        role.excluded += u32::from(!member.selected || !member.eligible);
        role.attempted += u32::from(observed.attempted);
        role.executed += u32::from(observed.executed);
        role.reused += u32::from(observed.reused);
        role.fallback += u32::from(observed.fallback.is_some());
        role.unsupported += u32::from(observed.unsupported.is_some());
        role.failed += u32::from(observed.failed);
        role.unrun += u32::from(observed.unrun);
        if !member.selected && (observed.attempted || observed.executed || observed.reused) {
            return Err(Error::CoverageDeficit);
        }
        if member.required
            && (!member.eligible
                || !observed.attempted
                || (!observed.executed && !observed.reused)
                || observed.unrun
                || observed.failed
                || observed.unsupported.is_some()
                || observed.fallback.is_some())
        {
            deficits.push(member.key.clone());
        }
    }
    Ok(CoverageSummary {
        baseline: baseline_id,
        manifest: manifest_id,
        roles: roles.into_values().collect(),
        complete: deficits.is_empty(),
        deficits,
    })
}

fn validate_coverage(value: &Coverage) -> Result<(), Error> {
    if (value.executed || value.reused) && !value.attempted {
        return Err(Error::Malformed);
    }
    if value.unrun && (value.executed || value.reused) {
        return Err(Error::Malformed);
    }
    if value.unsupported.is_some() && (value.executed || value.reused) && value.fallback.is_none() {
        return Err(Error::Malformed);
    }
    for reason in [&value.fallback, &value.unsupported].into_iter().flatten() {
        if !crate::safe_key(reason) {
            return Err(Error::Malformed);
        }
    }
    Ok(())
}

/// A separate reviewed revision records differences; it never modifies a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationRevision {
    pub old: Digest,
    pub new: Digest,
    pub removed: Vec<String>,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub review: Digest,
    pub reason: String,
}

pub fn propose_revision(
    old: &Manifest,
    new: &Manifest,
    review: Digest,
    reason: String,
) -> Result<PopulationRevision, Error> {
    if !crate::safe_key(&reason) {
        return Err(Error::Malformed);
    }
    let old_id = old.identity()?;
    let new_id = new.identity()?;
    if old_id == new_id {
        return Err(Error::BaselineRewrite);
    }
    let old_keys: BTreeSet<_> = old.population.iter().map(|m| &m.key).collect();
    let new_keys: BTreeSet<_> = new.population.iter().map(|m| &m.key).collect();
    let removed = old_keys.difference(&new_keys).map(|s| (*s).clone()).collect();
    let added = new_keys.difference(&old_keys).map(|s| (*s).clone()).collect();
    let changed = old
        .population
        .iter()
        .filter(|m| new.population.iter().find(|n| n.key == m.key).is_some_and(|n| n != *m))
        .map(|m| m.key.clone())
        .collect();
    Ok(PopulationRevision {
        old: old_id,
        new: new_id,
        removed,
        added,
        changed,
        review,
        reason,
    })
}

impl CoverageSummary {
    pub fn identity(&self) -> Result<Digest, Error> {
        identity(b"mantle-rust-differential-coverage-v1\0", self)
    }
}
