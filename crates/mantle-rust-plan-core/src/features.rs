//! Deterministic feature resolution over admitted package facts.
//!
//! Activation is a bounded fixed point over declared features: seeds are the
//! requested names plus the implicit `default` feature, and each pass
//! activates declared features whose references are already activated. The
//! pass count is bounded by the declared feature bound, so a malformed cycle
//! ends in a typed blocker instead of unbounded work.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::model::FeatureReference;
use crate::model::MAX_FEATURES_PER_PACKAGE;
use crate::model::PackageFacts;
use crate::model::PlanBlocker;
use crate::model::count_exceeds;

/// The implicit default feature name.
pub const DEFAULT_FEATURE: &str = "default";

/// Maximum activation passes over the declared feature list.
pub const MAX_FEATURE_PASSES: u32 = 64;

/// One feature request for a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFeatureRequest {
    /// Package name the request applies to.
    pub package: String,
    /// Requested feature names in caller order.
    pub features: Vec<String>,
    /// Whether default features are requested.
    pub include_default_features: bool,
}

/// Resolved feature activation for one package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureResolution {
    pub package: String,
    /// Activated features in canonical order.
    pub activated_features: Vec<String>,
    /// Activated optional-dependency keys in canonical order.
    pub activated_optional_dependencies: Vec<String>,
}

impl FeatureResolution {
    /// Whether one feature is activated.
    pub fn has_feature(&self, name: &str) -> bool {
        self.activated_features.iter().any(|feature| feature == name)
    }

    /// Whether one optional dependency key is activated.
    pub fn has_optional_dependency(&self, key: &str) -> bool {
        self.activated_optional_dependencies.iter().any(|dependency| dependency == key)
    }
}

/// Resolve feature activation for one package.
pub fn resolve_package_features(
    package: &PackageFacts,
    request: &PackageFeatureRequest,
) -> Result<FeatureResolution, Vec<PlanBlocker>> {
    let mut blockers: Vec<PlanBlocker> = Vec::with_capacity(2);
    if request.package != package.name {
        blockers.push(PlanBlocker::new(
            "feature-request-package-mismatch",
            &request.package,
            "feature requests must name the package they apply to",
        ));
        return Err(blockers);
    }
    if count_exceeds(request.features.len(), MAX_FEATURES_PER_PACKAGE) {
        blockers.push(PlanBlocker::new(
            "feature-request-limit",
            &package.name,
            "feature request exceeds the admitted bound",
        ));
        return Err(blockers);
    }
    let requested = requested_names(request);
    let unknown = requested.iter().find(|name| !package.features.iter().any(|declared| &declared.name == *name));
    if let Some(name) = unknown {
        blockers.push(PlanBlocker::new(
            "unknown-feature-request",
            name,
            "requested features must be declared by the package",
        ));
        return Err(blockers);
    }
    let (is_activated, is_stable) = close_activation(package, &requested);
    if !is_stable {
        blockers.push(PlanBlocker::new(
            "feature-propagation-limit",
            &package.name,
            "feature activation exceeded the bounded pass limit",
        ));
        return Err(blockers);
    }
    let activated_features: Vec<String> = package
        .features
        .iter()
        .enumerate()
        .filter(|(index, _)| is_activated[*index])
        .map(|(_, feature)| feature.name.clone())
        .collect();
    let activated_optional_dependencies = activated_dependency_keys(package, &is_activated);
    debug_assert!(activated_features.len() <= package.features.len());
    debug_assert!(activated_optional_dependencies.len() <= package.dependencies.len());
    Ok(FeatureResolution {
        package: package.name.clone(),
        activated_features,
        activated_optional_dependencies,
    })
}

fn requested_names(request: &PackageFeatureRequest) -> Vec<String> {
    let mut requested: Vec<String> = Vec::with_capacity(request.features.len().saturating_add(1));
    if request.include_default_features {
        requested.push(String::from(DEFAULT_FEATURE));
    }
    requested.extend(request.features.iter().cloned());
    debug_assert!(requested.len() <= request.features.len().saturating_add(1));
    requested
}

/// Compute the activation fixed point, returning activation flags and whether
/// the closure stabilized within the bounded pass count.
fn close_activation(package: &PackageFacts, requested: &[String]) -> (Vec<bool>, bool) {
    let mut is_activated: Vec<bool> = vec![false; package.features.len()];
    for (index, feature) in package.features.iter().enumerate() {
        if requested.iter().any(|name| name == &feature.name) {
            is_activated[index] = true;
        }
    }
    let mut is_stable = false;
    for _pass in 0..MAX_FEATURE_PASSES {
        let is_progressing = advance_activation(package, &mut is_activated);
        if !is_progressing {
            is_stable = true;
            break;
        }
    }
    debug_assert!(is_activated.len() == package.features.len());
    (is_activated, is_stable)
}

/// Propagate activation forward: enabling a feature enables its references.
fn advance_activation(package: &PackageFacts, is_activated: &mut [bool]) -> bool {
    let referenced: Vec<usize> = package
        .features
        .iter()
        .enumerate()
        .filter(|(index, _)| is_activated[*index])
        .flat_map(|(_, feature)| feature.enables.iter())
        .filter_map(|reference| match reference {
            FeatureReference::Feature(name) => package.features.iter().position(|declared| &declared.name == name),
            FeatureReference::Dependency(_) => None,
        })
        .collect();
    let referenced_len = referenced.len();
    let mut is_advanced = false;
    for index in referenced {
        if !is_activated[index] {
            is_activated[index] = true;
            is_advanced = true;
        }
    }
    debug_assert!(referenced_len <= package.features.len());
    is_advanced
}

/// Optional dependency keys activated by any activated feature.
fn activated_dependency_keys(package: &PackageFacts, is_activated: &[bool]) -> Vec<String> {
    let mut keys: Vec<String> = package
        .features
        .iter()
        .enumerate()
        .filter(|(index, _)| is_activated[*index])
        .flat_map(|(_, feature)| feature.enables.iter())
        .filter_map(|reference| match reference {
            FeatureReference::Dependency(key) => Some(key.clone()),
            FeatureReference::Feature(_) => None,
        })
        .filter(|key| package.dependencies.iter().any(|dependency| &dependency.key == key && dependency.optional))
        .collect();
    keys.sort();
    keys.dedup();
    debug_assert!(keys.len() <= package.dependencies.len());
    keys
}
