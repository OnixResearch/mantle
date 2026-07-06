//! StoreLayer: tracking which store layer produced or served a path, blob,
//! directory, or PathInfo during overlay store composition.

use std::fmt;

/// Identifies which store layer produced or served a given artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreLayer {
    /// Served from the writable overlay (local store).
    Overlay,
    /// Served from a read-only base store.
    Base,
}

impl StoreLayer {
    /// Return `true` if this is the Base layer.
    pub fn is_base(self) -> bool {
        matches!(self, StoreLayer::Base)
    }

    /// Return `true` if this is the Overlay layer.
    pub fn is_overlay(self) -> bool {
        matches!(self, StoreLayer::Overlay)
    }
}

impl fmt::Display for StoreLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreLayer::Overlay => write!(f, "overlay"),
            StoreLayer::Base => write!(f, "base"),
        }
    }
}

/// A value that carries a store-layer provenance tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layered<T> {
    pub value: T,
    pub layer: StoreLayer,
}

impl<T> Layered<T> {
    pub fn overlay(value: T) -> Self {
        Self { value, layer: StoreLayer::Overlay }
    }

    pub fn base(value: T) -> Self {
        Self { value, layer: StoreLayer::Base }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Layered<U> {
        Layered { value: f(self.value), layer: self.layer }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_layer_display() {
        assert_eq!(StoreLayer::Overlay.to_string(), "overlay");
        assert_eq!(StoreLayer::Base.to_string(), "base");
    }

    #[test]
    fn store_layer_booleans() {
        assert!(StoreLayer::Overlay.is_overlay());
        assert!(!StoreLayer::Overlay.is_base());
        assert!(StoreLayer::Base.is_base());
        assert!(!StoreLayer::Base.is_overlay());
    }

    #[test]
    fn layered_overlay_constructor() {
        let l = Layered::overlay(42usize);
        assert_eq!(l.value, 42);
        assert_eq!(l.layer, StoreLayer::Overlay);
    }

    #[test]
    fn layered_base_constructor() {
        let l = Layered::base("hello");
        assert_eq!(l.value, "hello");
        assert_eq!(l.layer, StoreLayer::Base);
    }

    #[test]
    fn layered_map() {
        let l = Layered::overlay(5u64);
        let mapped = l.map(|v| v * 2);
        assert_eq!(mapped.value, 10);
        assert_eq!(mapped.layer, StoreLayer::Overlay);
    }
}
