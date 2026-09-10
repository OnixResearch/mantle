// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/changes/complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::usize_in_public_api)]

//! Exact store-layer provenance for composed reads.

use std::fmt;

/// Identifies the selected precedence layer for one store read.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoreLayer {
    /// Served from the writable overlay or the only local store.
    #[default]
    Overlay,
    /// Served from a read-only base. Index one is the first declared base.
    Base { index: usize },
}

impl StoreLayer {
    pub fn from_service_index(index: usize) -> Result<Self, LayerIndexError> {
        if index == 0 {
            return Ok(Self::Overlay);
        }
        Ok(Self::Base { index })
    }

    #[must_use]
    pub const fn service_index(self) -> usize {
        match self {
            Self::Overlay => 0,
            Self::Base { index } => index,
        }
    }

    #[must_use]
    pub const fn is_base(self) -> bool {
        matches!(self, Self::Base { .. })
    }

    #[must_use]
    pub const fn is_overlay(self) -> bool {
        matches!(self, Self::Overlay)
    }
}

impl fmt::Display for StoreLayer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overlay => formatter.write_str("overlay"),
            Self::Base { index } => write!(formatter, "base[{index}]"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("store layer index is invalid")]
pub struct LayerIndexError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerShadowStatus {
    Matching,
    Conflicting,
    DigestCollision,
    ReadFailure,
}

impl LayerShadowStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Matching => "matching",
            Self::Conflicting => "conflicting",
            Self::DigestCollision => "digest-collision",
            Self::ReadFailure => "read-failure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerShadowObservation {
    pub layer: StoreLayer,
    pub status: LayerShadowStatus,
}

/// A value with exact store-layer provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layered<T> {
    pub value: T,
    pub layer: StoreLayer,
    pub shadows: Vec<LayerShadowObservation>,
}

/// Bounded evidence for one logical store path selected during an operation.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct StoreLayerSelection {
    pub store_path: String,
    pub selected_layer: StoreLayer,
    pub shadows: Vec<LayerShadowObservation>,
}

impl StoreLayerSelection {
    #[must_use]
    pub fn from_layered<T>(store_path: String, layered: &Layered<T>) -> Self {
        Self {
            store_path,
            selected_layer: layered.layer,
            shadows: layered.shadows.clone(),
        }
    }
}

impl<T> Layered<T> {
    #[must_use]
    pub const fn overlay(value: T) -> Self {
        Self {
            value,
            layer: StoreLayer::Overlay,
            shadows: Vec::new(),
        }
    }

    pub fn from_service_index(value: T, index: usize) -> Result<Self, LayerIndexError> {
        Ok(Self {
            value,
            layer: StoreLayer::from_service_index(index)?,
            shadows: Vec::new(),
        })
    }

    #[must_use]
    pub fn map<U, F>(self, map_value: F) -> Layered<U>
    where F: FnOnce(T) -> U {
        Layered {
            value: map_value(self.value),
            layer: self.layer,
            shadows: self.shadows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_BASE_INDEX: usize = 1;
    const SECOND_BASE_INDEX: usize = 2;

    #[test]
    fn store_layer_display_includes_exact_base_index() {
        assert_eq!(StoreLayer::Overlay.to_string(), "overlay");
        assert_eq!(
            StoreLayer::Base {
                index: SECOND_BASE_INDEX
            }
            .to_string(),
            "base[2]"
        );
    }

    #[test]
    fn store_layer_booleans_are_disjoint() {
        assert!(StoreLayer::Overlay.is_overlay());
        assert!(!StoreLayer::Overlay.is_base());
        assert!(
            StoreLayer::Base {
                index: FIRST_BASE_INDEX
            }
            .is_base()
        );
        assert!(
            !StoreLayer::Base {
                index: FIRST_BASE_INDEX
            }
            .is_overlay()
        );
    }

    #[test]
    fn layered_value_preserves_exact_index_through_map() {
        let layered = Layered::from_service_index(5_u64, SECOND_BASE_INDEX).unwrap();
        let mapped = layered.map(|value| value * 2);
        assert_eq!(mapped.value, 10);
        assert_eq!(mapped.layer, StoreLayer::Base {
            index: SECOND_BASE_INDEX
        });
    }

    #[test]
    fn zero_service_index_is_overlay() {
        let layered = Layered::from_service_index("value", 0).unwrap();
        assert_eq!(layered.layer, StoreLayer::Overlay);
    }
}
