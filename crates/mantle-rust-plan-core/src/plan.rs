//! Deterministic unit, effect, and observation planning over admitted facts.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::digest::Blake3Digest;
use crate::digest::DigestError;
use crate::digest::domain_digest;
use crate::model::DependencyKind;
use crate::model::MAX_ARGS_PER_EFFECT;
use crate::model::MAX_ENVIRONMENT_ENTRIES_PER_EFFECT;
use crate::model::PackageFacts;
use crate::model::PlanBlocker;
use crate::model::TargetKind;
use crate::model::admit_package_facts;
use crate::model::count_exceeds;

/// Effect identity prefix.
const EFFECT_ID_PREFIX: &str = "effect:";

/// Domain tag for unit identities.
const UNIT_DOMAIN: &[u8] = b"mantle-rust-unit-v1\0";

/// Domain tag for plan identities.
const PLAN_DOMAIN: &[u8] = b"mantle-rust-plan-v1\0";

/// Maximum admitted units in one plan.
pub const MAX_UNITS: u32 = 16_384;

/// Maximum admitted effects in one plan (one per unit).
pub const MAX_EFFECTS: u32 = MAX_UNITS;

/// Build profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildProfile {
    Dev,
    Release,
}

/// Explicit planning limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitLimitFacts {
    pub max_units: u32,
}

impl Default for UnitLimitFacts {
    fn default() -> Self {
        Self { max_units: MAX_UNITS }
    }
}

/// One plan request over adapter-supplied facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRequest {
    /// Package names whose non-dependency targets are planned.
    pub roots: Vec<String>,
    pub packages: Vec<PackageFacts>,
    /// Explicit feature requests per package.
    pub feature_requests: Vec<crate::features::PackageFeatureRequest>,
    pub profile: BuildProfile,
    pub limits: UnitLimitFacts,
}

/// Stable unit identity: package key plus target name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnitId(pub String);

/// Stable effect identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EffectId(pub String);

/// One planned unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedUnit {
    pub unit_id: UnitId,
    pub package_key: String,
    pub target_name: String,
    pub target_kind: TargetKind,
    pub profile: BuildProfile,
    /// Activated package features in canonical order.
    pub activated_features: Vec<String>,
    /// Dependency unit identities in canonical order.
    pub dependency_unit_ids: Vec<UnitId>,
    pub unit_blake3: Blake3Digest,
}

/// One ordered unit effect with declared inputs, arguments, and outputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitEffect {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub arguments: Vec<String>,
    /// Environment entries in canonical order.
    pub environment: Vec<(String, String)>,
    pub input_identities: Vec<Blake3Digest>,
    pub expected_outputs: Vec<String>,
}

/// Plan outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanOutcome {
    Completed,
    Blocked,
}

/// One admitted plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustPlan {
    pub schema: String,
    pub outcome: PlanOutcome,
    pub units: Vec<PlannedUnit>,
    pub effects: Vec<UnitEffect>,
    pub blockers: Vec<PlanBlocker>,
    pub plan_blake3: Blake3Digest,
}

/// Observation status reported by an adapter after executing one effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitObservationStatus {
    Succeeded,
    Failed,
    Skipped,
}

/// One typed observation for a planned effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitObservation {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub status: UnitObservationStatus,
    pub exit_code: Option<i32>,
    pub diagnostics_code: Option<String>,
}

/// Result of classifying observations against one plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanExecutionOutcome {
    /// Every planned effect observed `Succeeded`.
    Completed,
    /// Some effects failed; the count is exact and bounded.
    Failed { failed_effect_count: u32 },
    /// Observed effects do not match the plan: unknown or missing identities.
    Rejected {
        unknown_effect_count: u32,
        missing_effect_count: u32,
    },
}

#[derive(Serialize)]
struct UnitIdentityInput<'a> {
    package_key: &'a str,
    target_name: &'a str,
    target_kind: TargetKind,
    profile: BuildProfile,
    activated_features: &'a [String],
    dependency_unit_ids: &'a [UnitId],
}

#[derive(Serialize)]
struct PlanIdentityInput<'a> {
    schema: &'a str,
    profile: BuildProfile,
    units: &'a [PlannedUnit],
    effects: &'a [UnitEffect],
}

/// Plan units, ordered effects, and blockers from admitted facts.
pub fn plan_rust_units(request: &PlanRequest) -> RustPlan {
    let mut blockers = Vec::new();
    if let Err(admission_blockers) = admit_package_facts(&request.packages) {
        blockers.extend(admission_blockers);
    }
    if request.roots.is_empty() {
        blockers.push(PlanBlocker::new("missing-roots", "roots", "a plan request must name at least one root package"));
    }
    let by_name = packages_by_name(&request.packages, &mut blockers);
    if !blockers.is_empty() {
        return blocked_plan(blockers);
    }
    let units = match build_units(request, &by_name) {
        Ok(units) => units,
        Err(unit_blockers) => return blocked_plan(unit_blockers),
    };
    let effects = match order_effects(&units) {
        Ok(effects) => effects,
        Err(effect_blockers) => return blocked_plan(effect_blockers),
    };
    let identity = plan_identity(request.profile, &units, &effects);
    let identity = match identity {
        Ok(identity) => identity,
        Err(_) => {
            return blocked_plan(vec![PlanBlocker::new(
                "plan-identity-failed",
                "plan",
                "plan could not be canonically identified",
            )]);
        }
    };
    debug_assert_eq!(effects.len(), units.len());
    RustPlan {
        schema: String::from(crate::model::PLAN_SCHEMA),
        outcome: PlanOutcome::Completed,
        units,
        effects,
        blockers: Vec::new(),
        plan_blake3: identity,
    }
}

fn packages_by_name<'a>(
    packages: &'a [PackageFacts],
    blockers: &mut Vec<PlanBlocker>,
) -> BTreeMap<&'a str, &'a PackageFacts> {
    let mut seen_names: Vec<&str> = Vec::with_capacity(packages.len());
    let mut entries: Vec<(&str, &PackageFacts)> = Vec::with_capacity(packages.len());
    for package in packages {
        if seen_names.contains(&package.name.as_str()) {
            blockers.push(PlanBlocker::new(
                "ambiguous-package-name",
                &package.name,
                "two admitted versions of one package name cannot share a plan",
            ));
            continue;
        }
        seen_names.push(package.name.as_str());
        entries.push((package.name.as_str(), package));
    }
    debug_assert!(entries.len() <= packages.len());
    entries.into_iter().collect()
}

fn build_units(
    request: &PlanRequest,
    by_name: &BTreeMap<&str, &PackageFacts>,
) -> Result<Vec<PlannedUnit>, Vec<PlanBlocker>> {
    let root_names: BTreeSet<&str> = request.roots.iter().map(String::as_str).collect();
    let mut blockers: Vec<PlanBlocker> = Vec::with_capacity(request.packages.len());
    let mut resolved: Vec<(&str, crate::features::FeatureResolution)> = Vec::with_capacity(request.packages.len());
    for package in &request.packages {
        let request_for = feature_request_for(package, &request.feature_requests);
        let outcome = crate::features::resolve_package_features(package, &request_for);
        match outcome {
            Ok(resolution) => resolved.push((package.name.as_str(), resolution)),
            Err(mut found) => blockers.append(&mut found),
        }
    }
    let resolutions: BTreeMap<&str, crate::features::FeatureResolution> = resolved.into_iter().collect();
    if !blockers.is_empty() {
        blockers.sort();
        return Err(blockers);
    }
    let mut units = collect_units(request, &root_names, &resolutions, &mut blockers);
    blockers.extend(missing_dependency_blockers(request, by_name, &root_names, &resolutions));
    if count_exceeds(units.len(), request.limits.max_units) {
        blockers.push(PlanBlocker::new("unit-limit", "units", "planned units exceed the declared limit"));
    }
    if !blockers.is_empty() {
        blockers.sort();
        return Err(blockers);
    }
    for unit in &mut units {
        unit.unit_blake3 = unit_identity(unit).unwrap_or_else(|_| Blake3Digest::from_slice(b"unidentifiable"));
    }
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    debug_assert!(units.windows(2).all(|pair| pair[0].unit_id != pair[1].unit_id));
    Ok(units)
}

/// Collect one unit per planable target, skipping targets whose required
/// features are not activated.
fn collect_units(
    request: &PlanRequest,
    root_names: &BTreeSet<&str>,
    resolutions: &BTreeMap<&str, crate::features::FeatureResolution>,
    blockers: &mut Vec<PlanBlocker>,
) -> Vec<PlannedUnit> {
    let mut units: Vec<PlannedUnit> = Vec::with_capacity(request.packages.len());
    for package in &request.packages {
        let is_root = root_names.contains(package.name.as_str());
        let activated_features = resolutions
            .get(package.name.as_str())
            .map(|resolution| resolution.activated_features.clone())
            .unwrap_or_default();
        for target in planable_targets(package, is_root) {
            let missing_required_feature = target
                .required_features
                .iter()
                .find(|required| !activated_features.iter().any(|feature| feature == *required));
            if let Some(required) = missing_required_feature {
                blockers.extend(core::iter::once(PlanBlocker::new(
                    "target-required-feature-missing",
                    &target.name,
                    required,
                )));
                continue;
            }
            units.push(PlannedUnit {
                unit_id: unit_id_of(package, &target.name, target.kind),
                package_key: package.key(),
                target_name: target.name.clone(),
                target_kind: target.kind,
                profile: request.profile,
                activated_features: activated_features.clone(),
                dependency_unit_ids: Vec::new(),
                unit_blake3: Blake3Digest::from_slice(b"pending"),
            });
        }
    }
    debug_assert!(!units.is_empty() || request.packages.is_empty());
    units
}

/// Blocker for every required dependency whose facts are absent.
///
/// Optional dependencies participate only once a feature activates them, and
/// dev dependencies belong to root packages only.
fn missing_dependency_blockers(
    request: &PlanRequest,
    by_name: &BTreeMap<&str, &PackageFacts>,
    root_names: &BTreeSet<&str>,
    resolutions: &BTreeMap<&str, crate::features::FeatureResolution>,
) -> Vec<PlanBlocker> {
    let mut missing: Vec<String> = Vec::with_capacity(request.packages.len());
    for package in &request.packages {
        let resolution = resolutions.get(package.name.as_str());
        for dependency in &package.dependencies {
            if dependency.kind == DependencyKind::Dev && !root_names.contains(package.name.as_str()) {
                continue;
            }
            let is_inactive_optional = dependency.optional
                && !resolution.is_some_and(|resolution| resolution.has_optional_dependency(&dependency.key));
            if is_inactive_optional {
                continue;
            }
            if !by_name.contains_key(dependency.package.as_str()) && !missing.contains(&dependency.package) {
                missing.push(dependency.package.clone());
            }
        }
    }
    debug_assert!(missing.len() <= request.packages.len());
    missing
        .into_iter()
        .map(|name| PlanBlocker::new("missing-dependency-package", &name, "dependency package facts were not supplied"))
        .collect()
}

/// Feature request for one package: explicit when supplied, otherwise only
/// the implicit default feature when the package declares one.
fn feature_request_for(
    package: &PackageFacts,
    requests: &[crate::features::PackageFeatureRequest],
) -> crate::features::PackageFeatureRequest {
    if let Some(explicit) = requests.iter().find(|request| request.package == package.name) {
        debug_assert_eq!(explicit.package, package.name);
        return explicit.clone();
    }
    let has_default_feature = package.features.iter().any(|feature| feature.name == crate::features::DEFAULT_FEATURE);
    debug_assert!(!has_default_feature || !package.features.is_empty());
    crate::features::PackageFeatureRequest {
        package: package.name.clone(),
        features: Vec::new(),
        include_default_features: has_default_feature,
    }
}

fn planable_targets(package: &PackageFacts, is_root: bool) -> Vec<crate::model::TargetFacts> {
    let mut targets: Vec<crate::model::TargetFacts> = package
        .targets
        .iter()
        .filter(|target| match target.kind {
            TargetKind::Lib | TargetKind::ProcMacro | TargetKind::CustomBuild => true,
            TargetKind::Bin | TargetKind::Example | TargetKind::Test | TargetKind::Bench => is_root,
        })
        .cloned()
        .collect();
    targets.sort_by(|left, right| (left.name.as_str(), left.kind).cmp(&(right.name.as_str(), right.kind)));
    targets
}

fn unit_id_of(package: &PackageFacts, target_name: &str, target_kind: TargetKind) -> UnitId {
    let mut id = package.key();
    id.push(':');
    id.push_str(target_name);
    id.push(':');
    id.push_str(target_label(target_kind));
    UnitId(id)
}

fn target_label(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Lib => "lib",
        TargetKind::Bin => "bin",
        TargetKind::Example => "example",
        TargetKind::Test => "test",
        TargetKind::Bench => "bench",
        TargetKind::CustomBuild => "custom-build",
        TargetKind::ProcMacro => "proc-macro",
    }
}

fn unit_identity(unit: &PlannedUnit) -> Result<Blake3Digest, DigestError> {
    domain_digest(UNIT_DOMAIN, &UnitIdentityInput {
        package_key: &unit.package_key,
        target_name: &unit.target_name,
        target_kind: unit.target_kind,
        profile: unit.profile,
        activated_features: &unit.activated_features,
        dependency_unit_ids: &unit.dependency_unit_ids,
    })
}

fn are_effect_bounds_exceeded(environment: &[(String, String)], arguments: &[String]) -> bool {
    count_exceeds(environment.len(), MAX_ENVIRONMENT_ENTRIES_PER_EFFECT)
        || count_exceeds(arguments.len(), MAX_ARGS_PER_EFFECT)
}

fn effect_bound_blockers(units: &[PlannedUnit]) -> Vec<PlanBlocker> {
    units
        .iter()
        .filter(|unit| count_exceeds(effect_arguments(unit).len(), MAX_ARGS_PER_EFFECT))
        .map(|unit| {
            PlanBlocker::new("effect-argument-limit", &unit.unit_id.0, "effect arguments exceed the declared bound")
        })
        .collect()
}

fn order_effects(units: &[PlannedUnit]) -> Result<Vec<UnitEffect>, Vec<PlanBlocker>> {
    let mut effects: Vec<UnitEffect> = Vec::with_capacity(units.len());
    let mut blockers: Vec<PlanBlocker> = effect_bound_blockers(units);
    for unit in units {
        let environment: Vec<(String, String)> =
            vec![(String::from("PROFILE"), String::from(profile_label(unit.profile)))];
        let arguments = effect_arguments(unit);
        if are_effect_bounds_exceeded(&environment, &arguments) {
            continue;
        }
        effects.push(UnitEffect {
            effect_id: EffectId(effect_id_of(unit)),
            unit_id: unit.unit_id.clone(),
            arguments,
            environment,
            input_identities: vec![unit.unit_blake3.clone()],
            expected_outputs: vec![expected_output_of(unit)],
        });
    }
    if !blockers.is_empty() {
        blockers.sort();
        return Err(blockers);
    }
    effects.sort_by(|left, right| left.effect_id.cmp(&right.effect_id));
    debug_assert_eq!(effects.len(), units.len());
    Ok(effects)
}

fn effect_arguments(unit: &PlannedUnit) -> Vec<String> {
    vec![
        String::from("rustc"),
        String::from("--crate-type"),
        String::from(crate_type_label(unit.target_kind)),
        String::from("--crate-name"),
        crate_name(unit),
    ]
}

fn crate_name(unit: &PlannedUnit) -> String {
    let mut name = String::with_capacity(unit.target_name.len());
    for character in unit.target_name.chars() {
        name.push(if character == '-' { '_' } else { character });
    }
    debug_assert!(!name.is_empty());
    name
}

fn crate_type_label(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Lib => "rlib",
        TargetKind::Bin | TargetKind::CustomBuild => "bin",
        TargetKind::Example => "example",
        TargetKind::Test => "test",
        TargetKind::Bench => "bench",
        TargetKind::ProcMacro => "proc-macro",
    }
}

fn expected_output_of(unit: &PlannedUnit) -> String {
    let mut name = String::new();
    name.push_str(&crate_name(unit));
    match unit.target_kind {
        TargetKind::Lib => name.push_str(".rlib"),
        TargetKind::ProcMacro => name.push_str(".so"),
        _ => name.push_str(".out"),
    }
    name
}

fn effect_id_of(unit: &PlannedUnit) -> String {
    let id_capacity_bytes = unit.unit_id.0.len().saturating_add(EFFECT_ID_PREFIX.len());
    let mut id = String::with_capacity(id_capacity_bytes);
    id.push_str(EFFECT_ID_PREFIX);
    id.push_str(&unit.unit_id.0);
    debug_assert!(!id.is_empty());
    id
}

fn profile_label(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Dev => "dev",
        BuildProfile::Release => "release",
    }
}

fn plan_identity(
    profile: BuildProfile,
    units: &[PlannedUnit],
    effects: &[UnitEffect],
) -> Result<Blake3Digest, DigestError> {
    domain_digest(PLAN_DOMAIN, &PlanIdentityInput {
        schema: crate::model::PLAN_SCHEMA,
        profile,
        units,
        effects,
    })
}

fn blocked_plan(mut blockers: Vec<PlanBlocker>) -> RustPlan {
    blockers.sort();
    blockers.dedup();
    debug_assert!(!blockers.is_empty());
    RustPlan {
        schema: String::from(crate::model::PLAN_SCHEMA),
        outcome: PlanOutcome::Blocked,
        units: Vec::new(),
        effects: Vec::new(),
        blockers,
        plan_blake3: Blake3Digest::from_slice(b"blocked-plan"),
    }
}

/// Classify observations against a completed plan.
///
/// Unknown effect identities, duplicate observations, and missing effects are
/// rejected; failures are counted exactly; skips count as failures because a
/// skipped planned unit is not a completed build.
pub fn classify_plan_observations(plan: &RustPlan, observations: &[UnitObservation]) -> PlanExecutionOutcome {
    let planned: BTreeSet<&str> = plan.effects.iter().map(|effect| effect.effect_id.0.as_str()).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut unknown: u32 = 0;
    let mut failed: u32 = 0;
    for observation in observations {
        if !planned.contains(observation.effect_id.0.as_str()) || !seen.insert(observation.effect_id.0.as_str()) {
            unknown = unknown.saturating_add(1);
            continue;
        }
        if observation.status != UnitObservationStatus::Succeeded {
            failed = failed.saturating_add(1);
        }
    }
    let missing = match u32::try_from(planned.len()) {
        Ok(planned_count) => planned_count.saturating_sub(u32::try_from(seen.len()).unwrap_or(0)),
        // A plan wider than the observation domain rejects every observation.
        Err(_) => 0,
    };
    if unknown > 0 || missing > 0 {
        return PlanExecutionOutcome::Rejected {
            unknown_effect_count: unknown,
            missing_effect_count: missing,
        };
    }
    if failed > 0 {
        return PlanExecutionOutcome::Failed {
            failed_effect_count: failed,
        };
    }
    debug_assert_eq!(seen.len(), planned.len());
    PlanExecutionOutcome::Completed
}
