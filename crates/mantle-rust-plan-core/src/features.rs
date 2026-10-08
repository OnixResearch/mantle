//! Bounded native Cargo feature selection over adapter-decoded definitions.
//! The shell owns manifest parsing; the core owns the activation fixed point.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::model::PlanBlocker;

/// The implicit default feature name.
pub const DEFAULT_FEATURE: &str = "default";

/// Whether a native optional dependency is selected by any activated feature.
/// Supports direct, `dep:` and weak `?/` references in the accepted fragment.
pub fn native_optional_dependency_selected(
    dependency_name: &str,
    feature_defs: &BTreeMap<String, Vec<String>>,
    selected_features: &[String],
) -> bool {
    selected_features.iter().any(|feature| {
        feature == dependency_name
            || feature_defs.get(feature).is_some_and(|entries| {
                entries.iter().any(|entry| {
                    if entry == dependency_name || entry.strip_prefix("dep:") == Some(dependency_name) {
                        return true;
                    }
                    entry
                        .split_once('/')
                        .is_some_and(|(key, _)| key.trim_start_matches("dep:").trim_end_matches('?') == dependency_name)
                })
            })
    })
}

/// Preserve the accepted three-way cfg dependency decision.
pub fn classify_target_cfg_dependency(is_dependency_selected: bool, is_cfg_selected: bool) -> &'static str {
    if is_dependency_selected {
        "selected"
    } else if is_cfg_selected {
        "not-selected-optional"
    } else {
        "not-selected"
    }
}

/// The accepted native feature fixed-point limit (one dequeue per selected key).
pub const MAX_NATIVE_FEATURE_STEPS: usize = 4_096;

/// One adapter-decoded Cargo feature definition with semantic feature entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFeatureDefinition {
    pub name: String,
    pub entries: Vec<String>,
}

/// Structural native feature request, independent of manifest bytes or host state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFeatureSelectionRequest {
    pub feature_definitions: Vec<NativeFeatureDefinition>,
    pub optional_dependencies: Vec<String>,
    pub explicit_features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
}

/// A dependency feature requested by an activated parent feature.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NativeDependencyFeatureEdge {
    pub dependency: String,
    pub feature: String,
    pub weak: bool,
}

/// Accepted native feature selection, including forwarded feature requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFeatureSelection {
    pub selected_features: Vec<String>,
    pub activated_optional_dependencies: Vec<String>,
    pub dependency_feature_edges: Vec<NativeDependencyFeatureEdge>,
    pub blockers: Vec<PlanBlocker>,
}

/// Resolve the native planner's bounded feature grammar over supplied definitions.
/// Explicit features replace, rather than augment, the implicit default seed.
/// This intentionally leaves unknown explicitly requested keys in `selected_features`
/// as the accepted native planner does; only malformed definition references block.
pub fn resolve_native_feature_selection(request: &NativeFeatureSelectionRequest) -> NativeFeatureSelection {
    let mut definitions = BTreeMap::<&str, &[String]>::new();
    let mut optional = BTreeSet::new();
    let mut blockers = BTreeSet::new();
    if request.feature_definitions.len() > MAX_NATIVE_FEATURE_STEPS
        || request.optional_dependencies.len() > MAX_NATIVE_FEATURE_STEPS
        || request.explicit_features.len() > MAX_NATIVE_FEATURE_STEPS
    {
        blockers.insert(PlanBlocker::new(
            "feature-resolution-step-limit",
            "features",
            "native feature resolution exceeded bounded fixed-point step limit",
        ));
        return NativeFeatureSelection {
            selected_features: Vec::new(),
            activated_optional_dependencies: Vec::new(),
            dependency_feature_edges: Vec::new(),
            blockers: blockers.into_iter().collect(),
        };
    }
    for feature in &request.feature_definitions {
        if feature.name.is_empty() || definitions.insert(&feature.name, &feature.entries).is_some() {
            blockers.insert(PlanBlocker::new(
                "duplicate-feature",
                "features",
                "native feature names must be non-empty and unique",
            ));
        }
    }
    for key in &request.optional_dependencies {
        if key.is_empty() || !optional.insert(key.as_str()) {
            blockers.insert(PlanBlocker::new(
                "duplicate-optional-dependency",
                "features",
                "optional dependency keys must be non-empty and unique",
            ));
        }
    }
    if !blockers.is_empty() {
        return NativeFeatureSelection {
            selected_features: Vec::new(),
            activated_optional_dependencies: Vec::new(),
            dependency_feature_edges: Vec::new(),
            blockers: blockers.into_iter().collect(),
        };
    }
    let mut selected = BTreeSet::<String>::new();
    let mut queue = Vec::<String>::new();
    if request.all_features {
        for name in definitions.keys().chain(optional.iter()) {
            push_native_feature_key(name, &mut selected, &mut queue);
        }
    } else if !request.explicit_features.is_empty() {
        let mut explicit = request.explicit_features.clone();
        explicit.sort();
        explicit.dedup();
        for name in &explicit {
            push_native_feature_key(name, &mut selected, &mut queue);
        }
    } else if !request.no_default_features && definitions.contains_key(DEFAULT_FEATURE) {
        push_native_feature_key(DEFAULT_FEATURE, &mut selected, &mut queue);
    }
    let mut activated_optional = BTreeSet::<String>::new();
    let mut edges = BTreeSet::<NativeDependencyFeatureEdge>::new();
    let mut cursor = 0;
    while cursor < queue.len() {
        if cursor >= MAX_NATIVE_FEATURE_STEPS {
            blockers.insert(PlanBlocker::new(
                "feature-resolution-step-limit",
                "features",
                "native feature resolution exceeded bounded fixed-point step limit",
            ));
            break;
        }
        let feature = queue[cursor].clone();
        cursor += 1;
        if let Some(entries) = definitions.get(feature.as_str()) {
            for entry in *entries {
                if let Some(dependency) = entry.strip_prefix("dep:") {
                    if dependency.is_empty() {
                        blockers.insert(PlanBlocker::new(
                            "unsupported-feature-entry",
                            "features",
                            "empty dep: feature entry",
                        ));
                    } else {
                        activated_optional.insert(String::from(dependency));
                    }
                } else if let Some((dependency, enabled_feature)) = entry.split_once('/') {
                    let (dependency, weak) =
                        dependency.strip_suffix('?').map_or((dependency, false), |name| (name, true));
                    if dependency.is_empty() || enabled_feature.is_empty() {
                        blockers.insert(PlanBlocker::new(
                            "unsupported-feature-entry",
                            "features",
                            &alloc::format!("dependency feature entry `{entry}` is malformed"),
                        ));
                    } else {
                        edges.insert(NativeDependencyFeatureEdge {
                            dependency: String::from(dependency),
                            feature: String::from(enabled_feature),
                            weak,
                        });
                    }
                } else if definitions.contains_key(entry.as_str()) {
                    push_native_feature_key(entry, &mut selected, &mut queue);
                } else if optional.contains(entry.as_str()) {
                    activated_optional.insert(entry.clone());
                    push_native_feature_key(entry, &mut selected, &mut queue);
                } else {
                    blockers.insert(PlanBlocker::new(
                        "unknown-feature-entry",
                        "features",
                        &alloc::format!("feature entry `{entry}` references no known feature or optional dependency"),
                    ));
                }
            }
        } else if optional.contains(feature.as_str()) {
            activated_optional.insert(feature.clone());
        }
    }
    NativeFeatureSelection {
        selected_features: selected.into_iter().collect(),
        activated_optional_dependencies: activated_optional.into_iter().collect(),
        dependency_feature_edges: edges.into_iter().collect(),
        blockers: blockers.into_iter().collect(),
    }
}

fn push_native_feature_key(key: &str, selected: &mut BTreeSet<String>, queue: &mut Vec<String>) {
    if selected.insert(String::from(key)) {
        queue.push(String::from(key));
    }
}
