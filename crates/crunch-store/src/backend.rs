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
            rust_unit_cache: true,
            unsigned_admission: matches!(self, Self::Snix),
            max_root_changes: match self {
                Self::Snix => None,
                Self::Casita => Some(1024),
            },
        }
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

#[cfg(test)]
mod tests {
    use super::StoreBackend;
    use crate::StoreConfig;

    #[test]
    fn undeclared_overlay_rejects_before_accessing_writable_or_base_state() {
        let root = tempfile::tempdir().unwrap();
        let writable = root.path().join("not-a-state-directory");
        let base = root.path().join("unopened-base");
        std::fs::write(&writable, b"original").unwrap();
        let mut profile = StoreBackend::Snix.profile();
        profile.overlay_composition = false;

        let error = StoreConfig::preflight_backend_identity_with_profile(
            StoreBackend::Snix,
            profile,
            &writable,
            "/nix/store",
            std::slice::from_ref(&base),
        )
        .unwrap_err();
        assert!(error.to_string().contains("snix-overlay-unsupported"), "{error}");
        assert_eq!(std::fs::read(&writable).unwrap(), b"original");
        assert!(!base.exists());

        // This unusable state would fail the identity read if the capability
        // gate were moved behind even the first layer's preflight.
        profile.overlay_composition = true;
        let error = StoreConfig::preflight_backend_identity_with_profile(
            StoreBackend::Snix,
            profile,
            &writable,
            "/nix/store",
            std::slice::from_ref(&base),
        )
        .unwrap_err();
        assert!(error.to_string().contains("reading state directory"), "{error}");
        assert!(!base.exists());
    }
}
