//! Closed backend selection for persistent Mantle store state.

use std::str::FromStr;

use crate::Error;

/// Statically declared backend capabilities; optional paths must fail closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct StoreBackendCapabilityProfile {
    pub core: &'static [&'static str],
    pub overlay_composition: bool,
    pub atomic_batch_import: bool,
    pub rust_unit_cache: bool,
    pub unsigned_admission: bool,
    /// Backend-level atomic root-change bound; `None` means no backend bound.
    pub max_root_changes: Option<usize>,
}

/// Selected durable authority for admitted store outputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreBackend {
    Snix,
    Casita,
}

impl StoreBackend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Snix => "snix",
            Self::Casita => "casita",
        }
    }

    #[must_use]
    pub const fn profile(self) -> StoreBackendCapabilityProfile {
        const CASITA_CORE: &[&str] = &[
            "output-admission",
            "lookup",
            "fresh-process-reopen",
            "closure-resolution",
            "export",
            "store-archive-export",
            "store-archive-import",
            "gc-plan",
            "plan-bound-gc",
            "action-result-pathinfo-output-reuse",
            "store-sign",
        ];
        const SNIX_CORE: &[&str] = &[
            "output-admission",
            "lookup",
            "fresh-process-reopen",
            "closure-resolution",
            "export",
            "store-archive-export",
            "store-archive-import",
            "gc-plan",
            "plan-bound-gc",
            "action-result-pathinfo-output-reuse",
            "store-sign",
            "store-repair-final-nar",
        ];
        StoreBackendCapabilityProfile {
            core: match self {
                Self::Snix => SNIX_CORE,
                Self::Casita => CASITA_CORE,
            },
            overlay_composition: matches!(self, Self::Snix),
            atomic_batch_import: true,
            rust_unit_cache: matches!(self, Self::Snix),
            unsigned_admission: matches!(self, Self::Snix),
            max_root_changes: match self {
                Self::Snix => None,
                Self::Casita => Some(1024),
            },
        }
    }
    pub(crate) fn require_overlay_composition(self, profile: StoreBackendCapabilityProfile) -> Result<(), Error> {
        if profile.overlay_composition {
            return Ok(());
        }
        Err(Error::Store(format!(
            "{}-overlay-unsupported: backend cannot compose base stores",
            self.as_str()
        )))
    }

    pub(crate) fn require_verified_source_batch(self, profile: StoreBackendCapabilityProfile, count: usize) -> Result<(), Error> {
        if !profile.atomic_batch_import {
            return Err(Error::Store(format!(
                "verified-source-batch-unsupported: backend {} has no atomic batch admission",
                self.as_str()
            )));
        }
        if let Some(max) = profile.max_root_changes
            && count > max
        {
            return Err(Error::Store(format!(
                "verified-source-batch-limit: {count} roots exceed backend limit {max}"
            )));
        }
        Ok(())
    }
}

impl FromStr for StoreBackend {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "snix" => Ok(Self::Snix),
            "casita" => Ok(Self::Casita),
            _ => Err(Error::Store(format!("store-backend-unknown: {value}"))),
        }
    }
}
