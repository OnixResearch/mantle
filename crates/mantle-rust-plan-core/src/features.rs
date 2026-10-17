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

/// Whether the target cfg expression selected a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCfgSelection {
    Selected,
    NotSelected,
}

/// Preserve the accepted three-way cfg dependency decision.
pub fn classify_target_cfg_dependency(is_dependency_selected: bool, cfg_selection: TargetCfgSelection) -> &'static str {
    if is_dependency_selected {
        "selected"
    } else if cfg_selection == TargetCfgSelection::Selected {
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
        return blocked_native_features(blockers);
    }
    validate_native_feature_request(request, &mut definitions, &mut optional, &mut blockers);
    if !blockers.is_empty() {
        return blocked_native_features(blockers);
    }

    let mut activation = NativeFeatureActivation::default();
    seed_native_features(request, &definitions, &optional, &mut activation);
    activation.resolve(&definitions, &optional);
    assert_eq!(activation.queue.len(), activation.selected.len());
    assert!(
        activation.queue.len()
            <= request
                .feature_definitions
                .len()
                .saturating_add(request.optional_dependencies.len())
                .saturating_add(request.explicit_features.len())
    );
    activation.finish()
}

fn blocked_native_features(blockers: BTreeSet<PlanBlocker>) -> NativeFeatureSelection {
    NativeFeatureSelection {
        selected_features: Vec::new(),
        activated_optional_dependencies: Vec::new(),
        dependency_feature_edges: Vec::new(),
        blockers: blockers.into_iter().collect(),
    }
}

fn validate_native_feature_request<'a>(
    request: &'a NativeFeatureSelectionRequest,
    definitions: &mut BTreeMap<&'a str, &'a [String]>,
    optional: &mut BTreeSet<&'a str>,
    blockers: &mut BTreeSet<PlanBlocker>,
) {
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
    debug_assert!(definitions.keys().all(|name| !name.is_empty()));
    debug_assert!(optional.iter().all(|name| !name.is_empty()));
}

#[derive(Default)]
struct NativeFeatureActivation {
    selected: BTreeSet<String>,
    queue: Vec<String>,
    activated_optional: BTreeSet<String>,
    edges: BTreeSet<NativeDependencyFeatureEdge>,
    blockers: BTreeSet<PlanBlocker>,
}

fn seed_native_features(
    request: &NativeFeatureSelectionRequest,
    definitions: &BTreeMap<&str, &[String]>,
    optional: &BTreeSet<&str>,
    activation: &mut NativeFeatureActivation,
) {
    if request.all_features {
        for name in definitions.keys().chain(optional.iter()) {
            push_native_feature_key(name, &mut activation.selected, &mut activation.queue);
        }
    } else if !request.explicit_features.is_empty() {
        let mut explicit = request.explicit_features.clone();
        explicit.sort();
        explicit.dedup();
        for name in &explicit {
            push_native_feature_key(name, &mut activation.selected, &mut activation.queue);
        }
    } else if !request.no_default_features && definitions.contains_key(DEFAULT_FEATURE) {
        push_native_feature_key(DEFAULT_FEATURE, &mut activation.selected, &mut activation.queue);
    }
}

impl NativeFeatureActivation {
    fn resolve(&mut self, definitions: &BTreeMap<&str, &[String]>, optional: &BTreeSet<&str>) {
        let mut cursor = 0;
        while cursor < self.queue.len() {
            if cursor >= MAX_NATIVE_FEATURE_STEPS {
                self.blockers.insert(PlanBlocker::new(
                    "feature-resolution-step-limit",
                    "features",
                    "native feature resolution exceeded bounded fixed-point step limit",
                ));
                break;
            }
            let feature = self.queue[cursor].clone();
            cursor += 1;
            if let Some(entries) = definitions.get(feature.as_str()) {
                for entry in *entries {
                    self.resolve_entry(entry, definitions, optional);
                }
            } else if optional.contains(feature.as_str()) {
                self.activated_optional.insert(feature);
            }
        }
    }

    fn resolve_entry(&mut self, entry: &str, definitions: &BTreeMap<&str, &[String]>, optional: &BTreeSet<&str>) {
        if let Some(dependency) = entry.strip_prefix("dep:") {
            if dependency.is_empty() {
                self.blockers.insert(PlanBlocker::new(
                    "unsupported-feature-entry",
                    "features",
                    "empty dep: feature entry",
                ));
            } else {
                self.activated_optional.insert(String::from(dependency));
            }
        } else if let Some((dependency, enabled_feature)) = entry.split_once('/') {
            let (dependency, weak) = dependency.strip_suffix('?').map_or((dependency, false), |name| (name, true));
            if dependency.is_empty() || enabled_feature.is_empty() {
                self.blockers.insert(PlanBlocker::new(
                    "unsupported-feature-entry",
                    "features",
                    &alloc::format!("dependency feature entry `{entry}` is malformed"),
                ));
            } else {
                self.edges.insert(NativeDependencyFeatureEdge {
                    dependency: String::from(dependency),
                    feature: String::from(enabled_feature),
                    weak,
                });
            }
        } else if definitions.contains_key(entry) {
            push_native_feature_key(entry, &mut self.selected, &mut self.queue);
        } else if optional.contains(entry) {
            self.activated_optional.insert(String::from(entry));
            push_native_feature_key(entry, &mut self.selected, &mut self.queue);
        } else {
            self.blockers.insert(PlanBlocker::new(
                "unknown-feature-entry",
                "features",
                &alloc::format!("feature entry `{entry}` references no known feature or optional dependency"),
            ));
        }
    }

    fn finish(self) -> NativeFeatureSelection {
        NativeFeatureSelection {
            selected_features: self.selected.into_iter().collect(),
            activated_optional_dependencies: self.activated_optional.into_iter().collect(),
            dependency_feature_edges: self.edges.into_iter().collect(),
            blockers: self.blockers.into_iter().collect(),
        }
    }
}
fn push_native_feature_key(key: &str, selected: &mut BTreeSet<String>, queue: &mut Vec<String>) {
    if selected.insert(String::from(key)) {
        queue.push(String::from(key));
    }
}
