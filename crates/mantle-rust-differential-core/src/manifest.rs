use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Digest;
use crate::Error;
use crate::Identity;
use crate::identity;
use crate::safe_key;

pub const SCHEMA: &str = "mantle-rust-differential-manifest-v1";
pub const POLICY: &str = "ordered-rustc-semantics-v1";
pub const MAX_KEY_BYTES: usize = 256;
pub const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_CELLS: u32 = 32;
pub const MAX_STEPS: u32 = 64;
pub const MAX_MEMBERS: usize = 65_536;
pub const MAX_DIAGNOSTICS: usize = 4096;
pub const MAX_FIELD_BYTES: usize = 65_536;
pub const MAX_OBSERVATION_BYTES: usize = 16 * 1024 * 1024;

/// Hard ceilings as well as a predeclared per-invocation resource policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub cells: u32,
    pub steps: u32,
    pub processes: u32,
    pub source_files: u32,
    pub source_bytes: u64,
    pub artifact_bytes: u64,
    pub scratch_bytes: u64,
    pub stream_bytes: u64,
    pub diagnostics: u32,
    pub edit_bytes: u64,
    pub probe_ms: u64,
    pub build_ms: u64,
    pub program_ms: u64,
    pub total_ms: u64,
    pub teardown_ms: u64,
    pub concurrent_cells: u32,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            cells: MAX_CELLS,
            steps: MAX_STEPS,
            processes: 64,
            source_files: 256,
            source_bytes: 4 * 1024 * 1024,
            artifact_bytes: 1024 * 1024 * 1024,
            scratch_bytes: 2 * 1024 * 1024 * 1024,
            stream_bytes: 8 * 1024 * 1024,
            diagnostics: MAX_DIAGNOSTICS as u32,
            edit_bytes: 1024 * 1024,
            probe_ms: 10_000,
            build_ms: 120_000,
            program_ms: 10_000,
            total_ms: 86_400_000,
            teardown_ms: 5_000,
            concurrent_cells: 1,
        }
    }
}

impl Bounds {
    pub fn validate(&self) -> Result<(), Error> {
        let ceiling = Self::default();
        let values = [
            (u64::from(self.cells), u64::from(ceiling.cells)),
            (u64::from(self.steps), u64::from(ceiling.steps)),
            (u64::from(self.processes), u64::from(ceiling.processes)),
            (u64::from(self.source_files), u64::from(ceiling.source_files)),
            (self.source_bytes, ceiling.source_bytes),
            (self.artifact_bytes, ceiling.artifact_bytes),
            (self.scratch_bytes, ceiling.scratch_bytes),
            (self.stream_bytes, ceiling.stream_bytes),
            (u64::from(self.diagnostics), u64::from(ceiling.diagnostics)),
            (self.edit_bytes, ceiling.edit_bytes),
            (self.probe_ms, ceiling.probe_ms),
            (self.build_ms, ceiling.build_ms),
            (self.program_ms, ceiling.program_ms),
            (self.total_ms, ceiling.total_ms),
            (self.teardown_ms, ceiling.teardown_ms),
        ];
        if self.concurrent_cells != 1 || values.iter().any(|(v, max)| *v == 0 || v > max) {
            return Err(Error::Bound);
        }
        if self.total_ms < self.build_ms || self.scratch_bytes < self.source_bytes {
            return Err(Error::Bound);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Build,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub id: String,
    pub fixture: String,
    pub operation: Operation,
    pub profile: String,
    pub lto: Option<String>,
    pub features: Vec<String>,
    pub required: bool,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub key: String,
    pub role: String,
    pub required: bool,
    pub selected: bool,
    pub eligible: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Denominator {
    pub role: String,
    pub members: Vec<String>,
}

/// Freeze this object before observing runs; reports never mutate its population.
// r[impl rust_package_planning.differential_execution.population]
// r[impl rust_package_planning.differential_execution.identity]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub policy: String,
    pub bounds: Bounds,
    pub identities: Vec<Identity>,
    pub population: Vec<Member>,
    pub denominators: Vec<Denominator>,
    pub cells: Vec<Cell>,
}

impl Manifest {
    /// Bound input bytes before serde allocates any containers.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(Error::Bound);
        }
        let manifest: Self = serde_json::from_slice(bytes).map_err(|_| Error::Malformed)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn identity(&self) -> Result<Digest, Error> {
        self.validate()?;
        identity(b"mantle-rust-differential-manifest-v1\0", self)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.schema != SCHEMA || self.policy != POLICY {
            return Err(Error::Schema);
        }
        self.bounds.validate()?;
        if self.cells.is_empty()
            || self.cells.len() > self.bounds.cells as usize
            || self.population.is_empty()
            || self.population.len() > MAX_MEMBERS
            || self.identities.len() > 64
            || self.denominators.len() > 16
        {
            return Err(Error::Bound);
        }
        validate_identities(&self.identities)?;
        for role in [
            "source",
            "config",
            "lock",
            "dependencies",
            "rustc",
            "cargo",
            "sysroot",
            "tools",
            "argv",
            "environment",
            "profile",
            "features",
            "cache",
            "mode",
            "comparison",
            "edits",
            "harness",
        ] {
            if !self.identities.iter().any(|i| i.role == role) {
                return Err(Error::Missing);
            }
        }
        self.validate_members()?;
        let mut cells = BTreeSet::new();
        for cell in &self.cells {
            if !safe_key(&cell.id)
                || !safe_key(&cell.fixture)
                || !safe_key(&cell.profile)
                || !safe_key(&cell.mode)
                || !cells.insert(&cell.id)
            {
                return Err(Error::Identity);
            }
            if cell.features.len() > 64 || cell.features.iter().any(|s| !safe_key(s)) {
                return Err(Error::Bound);
            }
            if cell.features.windows(2).any(|w| w[0] >= w[1]) {
                return Err(Error::Duplicate);
            }
            if let Some(lto) = &cell.lto
                && (cell.operation != Operation::Build || !matches!(lto.as_str(), "thin" | "fat"))
            {
                return Err(Error::Schema);
            }
            if !self.population.iter().any(|m| m.role == "cell" && m.key == cell.id && m.required == cell.required) {
                return Err(Error::Missing);
            }
        }
        Ok(())
    }

    fn validate_members(&self) -> Result<(), Error> {
        let mut members = BTreeSet::new();
        for member in &self.population {
            if !safe_key(&member.key) || !safe_key(&member.role) || !members.insert(&member.key) {
                return Err(Error::Duplicate);
            }
            if (!member.selected || !member.eligible) && member.reason.as_ref().is_none_or(|r| !safe_key(r)) {
                return Err(Error::Missing);
            }
            if member.required && !member.selected {
                return Err(Error::CoverageDeficit);
            }
        }
        let mut roles = BTreeSet::new();
        for denominator in &self.denominators {
            if !safe_key(&denominator.role)
                || !roles.insert(&denominator.role)
                || denominator.members.len() > MAX_MEMBERS
            {
                return Err(Error::Duplicate);
            }
            let mut keys = BTreeSet::new();
            for key in &denominator.members {
                if !keys.insert(key) {
                    return Err(Error::Duplicate);
                }
                let member = self.population.iter().find(|m| &m.key == key).ok_or(Error::Missing)?;
                if member.role != denominator.role || !member.selected || !member.eligible {
                    return Err(Error::CoverageDeficit);
                }
            }
            let expected: BTreeSet<_> = self
                .population
                .iter()
                .filter(|m| m.role == denominator.role && m.selected && m.eligible)
                .map(|m| &m.key)
                .collect();
            if keys != expected {
                return Err(Error::CoverageDeficit);
            }
        }
        for role in self.population.iter().filter(|m| m.selected && m.eligible).map(|m| &m.role) {
            if !roles.contains(role) {
                return Err(Error::Missing);
            }
        }
        Ok(())
    }
}

pub(crate) fn validate_identities(values: &[Identity]) -> Result<(), Error> {
    let mut roles = BTreeSet::new();
    for value in values {
        if !safe_key(&value.role) || !roles.insert(&value.role) {
            return Err(Error::Duplicate);
        }
    }
    Ok(())
}

/// Future treatment admission cannot exchange binaries or silently change inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeRegistration {
    pub id: String,
    pub version: u32,
    pub off: Vec<Identity>,
    pub on: Vec<Identity>,
    pub toggles: Vec<String>,
    pub capability: Option<Digest>,
    pub proof_obligations: Vec<String>,
}

impl ModeRegistration {
    pub fn validate(&self) -> Result<(), Error> {
        if !safe_key(&self.id)
            || self.version != 1
            || self.off.len() > 64
            || self.on.len() > 64
            || self.toggles.len() > 16
        {
            return Err(Error::Bound);
        }
        validate_identities(&self.off)?;
        validate_identities(&self.on)?;
        for role in ["cargo", "rustc", "sysroot", "tools"] {
            let off = self.off.iter().find(|v| v.role == role).ok_or(Error::Missing)?;
            let on = self.on.iter().find(|v| v.role == role).ok_or(Error::Missing)?;
            if off != on {
                return Err(Error::Incomparable);
            }
        }
        for role in [
            "after-publication-body-error",
            "swap-success",
            "crash",
            "cancel",
            "stale-output",
            "incremental-clean",
        ] {
            if !self.proof_obligations.iter().any(|v| v == role) {
                return Err(Error::Missing);
            }
        }
        if self.capability.is_none() || self.toggles.is_empty() {
            return Err(Error::Missing);
        }
        let mut toggles = BTreeSet::new();
        for toggle in &self.toggles {
            if !safe_key(toggle) || !toggles.insert(toggle) {
                return Err(Error::Duplicate);
            }
        }
        if self.off.iter().map(|v| &v.role).collect::<BTreeSet<_>>()
            != self.on.iter().map(|v| &v.role).collect::<BTreeSet<_>>()
        {
            return Err(Error::Incomparable);
        }
        for off in &self.off {
            let on = self.on.iter().find(|v| v.role == off.role).ok_or(Error::Missing)?;
            if off != on && !toggles.contains(&off.role) {
                return Err(Error::Incomparable);
            }
        }
        Ok(())
    }
}
