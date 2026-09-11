//! Structural facts and nominal admission for Rust-plan inputs.
//!
//! Every value here is supplied by an adapter. Admission validates bounds,
//! uniqueness, and cross-fact consistency, and returns typed blockers; the
//! core never reparses bytes or reads host state.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

/// Plan schema identifier.
pub const PLAN_SCHEMA: &str = "mantle-rust-plan-v1";

/// Maximum admitted packages in one plan request.
pub const MAX_PACKAGES: u32 = 4_096;

/// Maximum admitted targets per package.
pub const MAX_TARGETS_PER_PACKAGE: u32 = 64;

/// Maximum admitted dependencies per package.
pub const MAX_DEPENDENCIES_PER_PACKAGE: u32 = 512;

/// Maximum admitted features per package.
pub const MAX_FEATURES_PER_PACKAGE: u32 = 512;

/// Maximum admitted effect arguments.
pub const MAX_ARGS_PER_EFFECT: u32 = 512;

/// Maximum admitted environment entries per effect.
pub const MAX_ENVIRONMENT_ENTRIES_PER_EFFECT: u32 = 256;

/// One bounded typed blocker.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlanBlocker {
    pub code: String,
    pub subject: String,
    pub message: String,
}

impl PlanBlocker {
    pub(crate) fn new(code: &str, subject: &str, message: &str) -> Self {
        debug_assert!(!code.is_empty() && !subject.is_empty() && !message.is_empty());
        Self {
            code: String::from(code),
            subject: String::from(subject),
            message: String::from(message),
        }
    }
}

/// Package source class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageSourceKind {
    Registry,
    Git,
    Path,
    WorkspaceMember,
}

/// Structural package source facts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageSource {
    pub kind: PackageSourceKind,
    /// Adapter-supplied source identity (registry name, git url+rev, or path).
    pub identity: String,
}

/// Target class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Lib,
    Bin,
    Example,
    Test,
    Bench,
    CustomBuild,
    ProcMacro,
}

/// Structural target facts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TargetFacts {
    pub name: String,
    pub kind: TargetKind,
    pub crate_types: Vec<String>,
    pub required_features: Vec<String>,
}

/// Dependency edge class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyKind {
    Normal,
    Dev,
    Build,
}

/// Structural dependency facts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DependencyFacts {
    /// Dependency key as written in the manifest.
    pub key: String,
    /// Package name the key resolves to.
    pub package: String,
    pub version_requirement: String,
    pub kind: DependencyKind,
    pub optional: bool,
    /// Target cfg predicate, when the dependency is conditional.
    pub target_predicate: Option<String>,
    pub features: Vec<String>,
    pub default_features: bool,
    pub renamed: bool,
}

/// One feature reference: `Feature(name)` or `Dependency(name)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "reference", content = "value")]
pub enum FeatureReference {
    Feature(String),
    Dependency(String),
}

/// Structural feature facts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FeatureFacts {
    pub name: String,
    pub enables: Vec<FeatureReference>,
}

/// Structural package facts supplied by an adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFacts {
    pub name: String,
    pub version: String,
    pub source: PackageSource,
    pub targets: Vec<TargetFacts>,
    pub dependencies: Vec<DependencyFacts>,
    pub features: Vec<FeatureFacts>,
    /// `links` metadata, when the package runs a native build script.
    pub links: Option<String>,
}

impl PackageFacts {
    /// Structural package key used for dependency resolution.
    pub fn key(&self) -> String {
        let key_capacity_bytes = self.name.len().saturating_add(self.version.len()).saturating_add(1);
        let mut key = String::with_capacity(key_capacity_bytes);
        key.push_str(&self.name);
        key.push('@');
        key.push_str(&self.version);
        debug_assert!(!key.is_empty());
        key
    }
}

/// Admit bounded structural facts, returning typed blockers instead of panics.
pub(crate) fn admit_package_facts(facts: &[PackageFacts]) -> Result<(), Vec<PlanBlocker>> {
    let mut blockers: Vec<PlanBlocker> = Vec::with_capacity(facts.len());
    if count_exceeds(facts.len(), MAX_PACKAGES) {
        blockers.push(PlanBlocker::new("package-limit", "packages", "plan request exceeds the admitted package bound"));
        return Err(blockers);
    }
    let mut seen_keys: Vec<String> = Vec::with_capacity(facts.len());
    debug_assert!(seen_keys.capacity() >= facts.len());
    for package in facts {
        check_package_bounds(package, &mut blockers);
        let key = package.key();
        if seen_keys.iter().any(|seen| seen == &key) {
            blockers.push(PlanBlocker::new("duplicate-package", &key, "one package name and version may appear once"));
            continue;
        }
        seen_keys.push(key.clone());
        check_package_targets(package, &key, &mut blockers);
        check_dependency_keys(package, &key, &mut blockers);
        check_feature_references(package, &key, &mut blockers);
    }
    if blockers.is_empty() {
        debug_assert!(seen_keys.len() <= facts.len());
        Ok(())
    } else {
        blockers.sort();
        Err(blockers)
    }
}

fn check_package_bounds(package: &PackageFacts, blockers: &mut Vec<PlanBlocker>) {
    let blocker_count_before = blockers.len();
    if package.name.is_empty() || package.version.is_empty() || package.source.identity.is_empty() {
        blockers.push(PlanBlocker::new(
            "package-identity",
            &package.name,
            "package name, version, and source identity must be non-empty",
        ));
    }
    if package.targets.is_empty() {
        blockers.push(PlanBlocker::new("package-targets", &package.name, "package must declare at least one target"));
    }
    if count_exceeds(package.targets.len(), MAX_TARGETS_PER_PACKAGE)
        || count_exceeds(package.dependencies.len(), MAX_DEPENDENCIES_PER_PACKAGE)
        || count_exceeds(package.features.len(), MAX_FEATURES_PER_PACKAGE)
    {
        blockers.push(PlanBlocker::new(
            "package-bound",
            &package.name,
            "package facts exceed an admitted collection bound",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    let is_target_count_admissible =
        u32::try_from(package.targets.len()).is_ok_and(|count| count <= MAX_TARGETS_PER_PACKAGE);
    debug_assert!(is_target_count_admissible || !blockers.is_empty());
}

fn check_package_targets(package: &PackageFacts, key: &str, blockers: &mut Vec<PlanBlocker>) {
    let blocker_count_before = blockers.len();
    // Cargo permits one lib and one bin sharing a name, so uniqueness is per
    // name and target kind rather than per name alone.
    let mut seen: Vec<(&str, TargetKind)> = Vec::with_capacity(package.targets.len());
    for target in &package.targets {
        if target.name.is_empty() {
            blockers.push(PlanBlocker::new("target-name", key, "target names must be non-empty"));
            continue;
        }
        let is_duplicate = seen.iter().any(|(name, kind)| *name == target.name.as_str() && *kind == target.kind);
        if is_duplicate {
            blockers.push(PlanBlocker::new(
                "duplicate-target",
                &target.name,
                "target names must be unique per package and target kind",
            ));
            continue;
        }
        seen.push((target.name.as_str(), target.kind));
        let is_proc_macro_flag = target.crate_types.iter().any(|crate_type| crate_type == "proc-macro");
        if is_proc_macro_flag != (target.kind == TargetKind::ProcMacro) {
            blockers.push(PlanBlocker::new(
                "proc-macro-mismatch",
                &target.name,
                "proc-macro crate types and target kinds must agree",
            ));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(seen.len() <= package.targets.len());
}

fn check_dependency_keys(package: &PackageFacts, key: &str, blockers: &mut Vec<PlanBlocker>) {
    let blocker_count_before = blockers.len();
    let mut seen_keys: Vec<&str> = Vec::with_capacity(package.dependencies.len());
    for dependency in &package.dependencies {
        if dependency.key.is_empty() || dependency.package.is_empty() || dependency.version_requirement.is_empty() {
            blockers.push(PlanBlocker::new(
                "dependency-identity",
                key,
                "dependency key, package, and version requirement must be non-empty",
            ));
            continue;
        }
        if seen_keys.contains(&dependency.key.as_str()) {
            blockers.push(PlanBlocker::new(
                "duplicate-dependency",
                &dependency.key,
                "dependency keys must be unique per package",
            ));
            continue;
        }
        seen_keys.push(dependency.key.as_str());
        let is_renamed = dependency.renamed || dependency.key != dependency.package;
        if is_renamed && !dependency.renamed {
            blockers.push(PlanBlocker::new(
                "renamed-dependency-flag",
                &dependency.key,
                "a renamed dependency must be declared as renamed",
            ));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(seen_keys.len() <= package.dependencies.len());
}

fn check_feature_references(package: &PackageFacts, key: &str, blockers: &mut Vec<PlanBlocker>) {
    let blocker_count_before = blockers.len();
    let mut seen_names: Vec<&str> = Vec::with_capacity(package.features.len());
    for feature in &package.features {
        if feature.name.is_empty() {
            blockers.push(PlanBlocker::new("feature-name", key, "feature names must be non-empty"));
            continue;
        }
        if seen_names.contains(&feature.name.as_str()) {
            blockers.push(PlanBlocker::new(
                "duplicate-feature",
                &feature.name,
                "feature names must be unique per package",
            ));
            continue;
        }
        seen_names.push(feature.name.as_str());
        for reference in &feature.enables {
            match reference {
                FeatureReference::Feature(name) => {
                    let is_known = package.features.iter().any(|declared| &declared.name == name);
                    if !is_known {
                        blockers.push(PlanBlocker::new(
                            "unknown-feature-reference",
                            &feature.name,
                            "feature references must name a declared feature",
                        ));
                    }
                }
                FeatureReference::Dependency(name) => {
                    let is_known =
                        package.dependencies.iter().any(|dependency| &dependency.key == name && dependency.optional);
                    if !is_known {
                        blockers.push(PlanBlocker::new(
                            "unknown-optional-dependency-reference",
                            &feature.name,
                            "dependency feature entries must name a declared optional dependency",
                        ));
                    }
                }
            }
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(seen_names.len() <= package.features.len());
}

pub(crate) fn count_exceeds(count_items: usize, maximum_items: u32) -> bool {
    match u32::try_from(count_items) {
        Ok(count_items) => count_items > maximum_items,
        // A collection wider than the admitted bound is over the bound.
        Err(_) => true,
    }
}
