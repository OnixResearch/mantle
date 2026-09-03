use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::DependencyRole;
use crate::FeatureActivation;
use crate::FeatureRequest;
use crate::PackageFact;
use crate::RustPlanCoreError;
use crate::RustPlanRequest;
use crate::RustPlanningInput;

const DEFAULT_FEATURE: &str = "default";
const TEST_PROFILE: &str = "test";

struct ResolvedSelection {
    package_ids: Vec<String>,
    features: BTreeMap<String, Vec<String>>,
}

struct ValueValidation<'a> {
    code: &'static str,
    value: &'a str,
}

pub fn admit_workspace(
    mut input: RustPlanningInput,
    mut request: RustPlanRequest,
) -> Result<crate::AdmittedWorkspace, RustPlanCoreError> {
    normalize_input(&mut input, &mut request);
    validate_input(&input, &request)?;
    let package_index = package_index(&input.workspace.packages);
    let selection = resolve_selection(&package_index, &request)?;
    debug_assert!(!selection.package_ids.is_empty());
    debug_assert!(selection.package_ids.len() <= input.workspace.packages.len());
    Ok(crate::AdmittedWorkspace {
        workspace_identity: input.workspace.workspace_identity,
        packages: input.workspace.packages,
        selected_package_ids: selection.package_ids,
        selected_features: selection.features,
        toolchain: input.toolchain,
        oracle: input.oracle,
        profile: request.profile,
    })
}

fn normalize_input(input: &mut RustPlanningInput, request: &mut RustPlanRequest) {
    input.workspace.packages.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    for package in &mut input.workspace.packages {
        package.targets.sort();
        package.dependencies.sort();
        package.features.sort();
        for dependency in &mut package.dependencies {
            dependency.activates_on.sort();
            dependency.activates_on.dedup();
        }
        for feature in &mut package.features {
            feature.activations.sort();
            feature.activations.dedup();
        }
    }
    request.workspace_members.sort();
    request.workspace_members.dedup();
    request.features.sort();
    request.features.dedup();
    if let Some(oracle) = &mut input.oracle {
        oracle.package_ids.sort();
        oracle.package_ids.dedup();
        oracle.unit_ids.sort();
        oracle.unit_ids.dedup();
    }
    debug_assert!(input.workspace.packages.windows(2).all(|pair| pair[0].package_id <= pair[1].package_id));
    debug_assert!(request.workspace_members.windows(2).all(|pair| pair[0] < pair[1]));
}

fn validate_input(input: &RustPlanningInput, request: &RustPlanRequest) -> Result<(), RustPlanCoreError> {
    validate_workspace_header(input)?;
    validate_toolchain(&input.toolchain)?;
    validate_packages(&input.workspace.packages)?;
    validate_package_references(&input.workspace.packages)?;
    validate_request(&input.workspace.packages, request)?;
    if let Some(oracle) = &input.oracle {
        validate_identity(ValueValidation {
            code: "oracle-metadata-identity",
            value: &oracle.metadata_identity,
        })?;
        validate_identity(ValueValidation {
            code: "oracle-unit-graph-identity",
            value: &oracle.unit_graph_identity,
        })?;
    }
    Ok(())
}

fn validate_workspace_header(input: &RustPlanningInput) -> Result<(), RustPlanCoreError> {
    if input.workspace.schema != crate::WORKSPACE_FACTS_SCHEMA {
        return Err(RustPlanCoreError::invalid("workspace-schema", input.workspace.schema.clone()));
    }
    validate_identity(ValueValidation {
        code: "workspace-identity",
        value: &input.workspace.workspace_identity,
    })?;
    check_count("package-count", input.workspace.packages.len(), crate::MAX_PACKAGES)?;
    if input.workspace.packages.is_empty() {
        return Err(RustPlanCoreError::invalid("packages-empty", "workspace"));
    }
    Ok(())
}

fn validate_toolchain(toolchain: &crate::ToolchainFact) -> Result<(), RustPlanCoreError> {
    validate_identity(ValueValidation {
        code: "compiler-identity",
        value: &toolchain.compiler_identity,
    })?;
    validate_identity(ValueValidation {
        code: "compiler-version-identity",
        value: &toolchain.compiler_version_identity,
    })?;
    validate_text(ValueValidation {
        code: "host-triple",
        value: &toolchain.host_triple,
    })?;
    validate_text(ValueValidation {
        code: "target-triple",
        value: &toolchain.target_triple,
    })?;
    if toolchain.host_triple == toolchain.target_triple {
        debug_assert!(!toolchain.host_triple.is_empty());
    }
    Ok(())
}

fn validate_packages(packages: &[PackageFact]) -> Result<(), RustPlanCoreError> {
    let mut package_ids = BTreeSet::new();
    for package in packages {
        validate_package(package)?;
        if !package_ids.insert(package.package_id.clone()) {
            return Err(RustPlanCoreError::AmbiguousReference {
                code: "duplicate-package-id",
                subject: package.package_id.clone(),
            });
        }
    }
    debug_assert_eq!(package_ids.len(), packages.len());
    debug_assert!(!package_ids.is_empty());
    Ok(())
}

fn validate_package(package: &PackageFact) -> Result<(), RustPlanCoreError> {
    validate_text(ValueValidation {
        code: "package-id",
        value: &package.package_id,
    })?;
    validate_text(ValueValidation {
        code: "package-name",
        value: &package.name,
    })?;
    validate_text(ValueValidation {
        code: "package-version",
        value: &package.version,
    })?;
    validate_identity(ValueValidation {
        code: "manifest-identity",
        value: &package.manifest_identity,
    })?;
    validate_identity(ValueValidation {
        code: "source-identity",
        value: &package.source_identity,
    })?;
    check_count("target-count", package.targets.len(), crate::MAX_TARGETS_PER_PACKAGE)?;
    check_count("dependency-count", package.dependencies.len(), crate::MAX_DEPENDENCIES_PER_PACKAGE)?;
    check_count("feature-count", package.features.len(), crate::MAX_FEATURES_PER_PACKAGE)?;
    if package.targets.is_empty() {
        return Err(RustPlanCoreError::invalid("targets-empty", package.package_id.clone()));
    }
    validate_targets(package)?;
    validate_features(package)?;
    validate_dependencies(package)?;
    debug_assert!(!package.package_id.is_empty());
    debug_assert!(!package.targets.is_empty());
    Ok(())
}

fn validate_targets(package: &PackageFact) -> Result<(), RustPlanCoreError> {
    let mut target_keys = BTreeSet::new();
    for target in &package.targets {
        validate_text(ValueValidation {
            code: "target-name",
            value: &target.name,
        })?;
        validate_text(ValueValidation {
            code: "crate-name",
            value: &target.crate_name,
        })?;
        validate_text(ValueValidation {
            code: "target-edition",
            value: &target.edition,
        })?;
        validate_identity(ValueValidation {
            code: "target-source-identity",
            value: &target.source_identity,
        })?;
        let key = (target.name.clone(), target.kind);
        if !target_keys.insert(key) {
            return Err(RustPlanCoreError::AmbiguousReference {
                code: "duplicate-target",
                subject: package.package_id.clone(),
            });
        }
    }
    debug_assert_eq!(target_keys.len(), package.targets.len());
    debug_assert!(!target_keys.is_empty());
    Ok(())
}

fn validate_features(package: &PackageFact) -> Result<(), RustPlanCoreError> {
    let mut names = BTreeSet::new();
    for feature in &package.features {
        validate_text(ValueValidation {
            code: "feature-name",
            value: &feature.name,
        })?;
        check_count("feature-activation-count", feature.activations.len(), crate::MAX_FEATURE_ACTIVATIONS)?;
        if !names.insert(feature.name.clone()) {
            return Err(RustPlanCoreError::AmbiguousReference {
                code: "duplicate-feature",
                subject: package.package_id.clone(),
            });
        }
    }
    for feature in &package.features {
        for activation in &feature.activations {
            if let FeatureActivation::Feature { name } = activation
                && !names.contains(name)
            {
                return Err(RustPlanCoreError::missing("feature-activation", name.clone()));
            }
        }
    }
    debug_assert_eq!(names.len(), package.features.len());
    debug_assert!(names.iter().all(|name| !name.is_empty()));
    Ok(())
}

fn validate_dependencies(package: &PackageFact) -> Result<(), RustPlanCoreError> {
    let mut keys = BTreeSet::new();
    for dependency in &package.dependencies {
        validate_text(ValueValidation {
            code: "dependency-package-id",
            value: &dependency.package_id,
        })?;
        check_count("dependency-activation-count", dependency.activates_on.len(), crate::MAX_FEATURES_PER_PACKAGE)?;
        let key = (dependency.package_id.clone(), dependency.role);
        if !keys.insert(key) {
            return Err(RustPlanCoreError::AmbiguousReference {
                code: "duplicate-dependency",
                subject: package.package_id.clone(),
            });
        }
    }
    debug_assert_eq!(keys.len(), package.dependencies.len());
    debug_assert!(u32::try_from(keys.len()).is_ok_and(|count| count <= crate::MAX_DEPENDENCIES_PER_PACKAGE));
    Ok(())
}

fn validate_package_references(packages: &[PackageFact]) -> Result<(), RustPlanCoreError> {
    let package_ids = packages.iter().map(|package| package.package_id.as_str()).collect::<BTreeSet<_>>();
    for package in packages {
        for dependency in &package.dependencies {
            if !package_ids.contains(dependency.package_id.as_str()) {
                return Err(RustPlanCoreError::missing("dependency-package", dependency.package_id.clone()));
            }
        }
        for feature in &package.features {
            for activation in &feature.activations {
                validate_activation_reference(activation, &package_ids)?;
            }
        }
    }
    debug_assert_eq!(package_ids.len(), packages.len());
    debug_assert!(!package_ids.is_empty());
    Ok(())
}

fn validate_activation_reference(
    activation: &FeatureActivation,
    package_ids: &BTreeSet<&str>,
) -> Result<(), RustPlanCoreError> {
    if let FeatureActivation::Dependency { package_id } = activation
        && !package_ids.contains(package_id.as_str())
    {
        return Err(RustPlanCoreError::missing("feature-dependency", package_id.clone()));
    }
    Ok(())
}

fn validate_request(packages: &[PackageFact], request: &RustPlanRequest) -> Result<(), RustPlanCoreError> {
    validate_text(ValueValidation {
        code: "profile",
        value: &request.profile,
    })?;
    let package_index = package_index(packages);
    for member in &request.workspace_members {
        let package = package_index
            .get(member)
            .ok_or_else(|| RustPlanCoreError::missing("workspace-member", member.clone()))?;
        if !package.workspace_member {
            return Err(RustPlanCoreError::invalid("selected-package-not-workspace-member", member.clone()));
        }
    }
    for feature in &request.features {
        validate_feature_request(&package_index, feature)?;
    }
    Ok(())
}

fn validate_feature_request(
    package_index: &BTreeMap<String, &PackageFact>,
    request: &FeatureRequest,
) -> Result<(), RustPlanCoreError> {
    let package = package_index
        .get(&request.package_id)
        .ok_or_else(|| RustPlanCoreError::missing("feature-package", request.package_id.clone()))?;
    if !package.features.iter().any(|feature| feature.name == request.feature) {
        return Err(RustPlanCoreError::missing("requested-feature", request.feature.clone()));
    }
    Ok(())
}

fn resolve_selection(
    package_index: &BTreeMap<String, &PackageFact>,
    request: &RustPlanRequest,
) -> Result<ResolvedSelection, RustPlanCoreError> {
    let mut selected = initial_members(package_index, request);
    let mut features = initial_features(package_index, request, &selected);
    debug_assert!(!selected.is_empty());
    debug_assert!(selected.len() <= package_index.len());
    for _step in 0..crate::MAX_FEATURE_RESOLUTION_STEPS {
        let before = (selected.len(), selected_feature_count(&features));
        expand_dependencies(package_index, request, &mut selected, &features);
        seed_selected_features(package_index, request, &selected, &mut features);
        expand_feature_activations(package_index, &mut selected, &mut features)?;
        let after = (selected.len(), selected_feature_count(&features));
        if before == after {
            let selected = selected.into_iter().collect();
            let features =
                features.into_iter().map(|(package, values)| (package, values.into_iter().collect())).collect();
            return Ok(ResolvedSelection {
                package_ids: selected,
                features,
            });
        }
    }
    Err(RustPlanCoreError::LimitExceeded {
        code: "feature-resolution-steps",
        observed: crate::MAX_FEATURE_RESOLUTION_STEPS,
        maximum: crate::MAX_FEATURE_RESOLUTION_STEPS,
    })
}

fn initial_members(package_index: &BTreeMap<String, &PackageFact>, request: &RustPlanRequest) -> BTreeSet<String> {
    let selected: BTreeSet<String> = if request.workspace_members.is_empty() {
        package_index
            .values()
            .filter(|package| package.workspace_member)
            .map(|package| package.package_id.clone())
            .collect()
    } else {
        request.workspace_members.iter().cloned().collect()
    };
    debug_assert!(!selected.is_empty());
    debug_assert!(selected.len() <= package_index.len());
    selected
}

fn initial_features(
    package_index: &BTreeMap<String, &PackageFact>,
    request: &RustPlanRequest,
    selected: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut features = BTreeMap::new();
    seed_selected_features(package_index, request, selected, &mut features);
    for requested in &request.features {
        features.entry(requested.package_id.clone()).or_default().insert(requested.feature.clone());
    }
    debug_assert!(features.len() <= selected.len());
    debug_assert!(request.features.iter().all(|feature| selected.contains(&feature.package_id)));
    features
}

fn seed_selected_features(
    package_index: &BTreeMap<String, &PackageFact>,
    request: &RustPlanRequest,
    selected: &BTreeSet<String>,
    features: &mut BTreeMap<String, BTreeSet<String>>,
) {
    for package_id in selected {
        let package = package_index[package_id];
        let entry = features.entry(package_id.clone()).or_default();
        if request.all_features {
            entry.extend(package.features.iter().map(|feature| feature.name.clone()));
        } else if !request.no_default_features && package.features.iter().any(|feature| feature.name == DEFAULT_FEATURE)
        {
            entry.insert(DEFAULT_FEATURE.into());
        }
    }
    debug_assert!(features.len() <= selected.len());
    debug_assert!(features.keys().all(|package_id| selected.contains(package_id)));
}

fn expand_dependencies(
    package_index: &BTreeMap<String, &PackageFact>,
    request: &RustPlanRequest,
    selected: &mut BTreeSet<String>,
    features: &BTreeMap<String, BTreeSet<String>>,
) {
    let selected_snapshot = selected.iter().cloned().collect::<Vec<_>>();
    for package_id in selected_snapshot {
        let package = package_index[&package_id];
        for dependency in &package.dependencies {
            if dependency_selected(dependency, request, features.get(&package_id)) {
                selected.insert(dependency.package_id.clone());
            }
        }
    }
    debug_assert!(selected.len() <= package_index.len());
    debug_assert!(selected.iter().all(|package_id| package_index.contains_key(package_id)));
}

fn dependency_selected(
    dependency: &crate::DependencyFact,
    request: &RustPlanRequest,
    selected_features: Option<&BTreeSet<String>>,
) -> bool {
    if dependency.role == DependencyRole::Development && request.profile != TEST_PROFILE {
        return false;
    }
    if !dependency.optional {
        return true;
    }
    let selected_features = selected_features.cloned().unwrap_or_default();
    dependency.activates_on.iter().any(|feature| selected_features.contains(feature))
}

fn expand_feature_activations(
    package_index: &BTreeMap<String, &PackageFact>,
    selected: &mut BTreeSet<String>,
    features: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), RustPlanCoreError> {
    let snapshot = features.clone();
    for (package_id, selected_features) in snapshot {
        let package = package_index[&package_id];
        for feature_name in selected_features {
            let feature = package
                .features
                .iter()
                .find(|candidate| candidate.name == feature_name)
                .ok_or_else(|| RustPlanCoreError::missing("selected-feature", feature_name.clone()))?;
            apply_activations(&package_id, &feature.activations, selected, features);
        }
    }
    Ok(())
}

fn apply_activations(
    package_id: &str,
    activations: &[FeatureActivation],
    selected: &mut BTreeSet<String>,
    features: &mut BTreeMap<String, BTreeSet<String>>,
) {
    for activation in activations {
        match activation {
            FeatureActivation::Feature { name } => {
                features.entry(package_id.into()).or_default().insert(name.clone());
            }
            FeatureActivation::Dependency { package_id } => {
                selected.insert(package_id.clone());
            }
        }
    }
    debug_assert!(!package_id.is_empty());
    debug_assert!(features.len() <= selected.len());
}

fn package_index(packages: &[PackageFact]) -> BTreeMap<String, &PackageFact> {
    let index = packages.iter().map(|package| (package.package_id.clone(), package)).collect::<BTreeMap<_, _>>();
    debug_assert_eq!(index.len(), packages.len());
    debug_assert!(!index.is_empty());
    index
}

fn selected_feature_count(features: &BTreeMap<String, BTreeSet<String>>) -> usize {
    features.values().map(BTreeSet::len).sum()
}

fn validate_text(input: ValueValidation<'_>) -> Result<(), RustPlanCoreError> {
    let value_bytes = u32::try_from(input.value.len())
        .map_err(|_| RustPlanCoreError::invalid(input.code, String::from(input.value)))?;
    if input.value.is_empty() || value_bytes > crate::MAX_TEXT_BYTES {
        return Err(RustPlanCoreError::invalid(input.code, String::from(input.value)));
    }
    Ok(())
}

fn validate_identity(input: ValueValidation<'_>) -> Result<(), RustPlanCoreError> {
    if !crate::identity::lowercase_blake3(input.value) {
        return Err(RustPlanCoreError::invalid(input.code, String::from(input.value)));
    }
    Ok(())
}

fn check_count(code: &'static str, observed: usize, maximum: u32) -> Result<(), RustPlanCoreError> {
    let observed = u32::try_from(observed).map_err(|_| RustPlanCoreError::invalid("count-width", code))?;
    if observed > maximum {
        return Err(RustPlanCoreError::LimitExceeded {
            code,
            observed,
            maximum,
        });
    }
    Ok(())
}
