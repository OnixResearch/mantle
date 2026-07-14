use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AotMode;
use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::DigestError;
use crate::LockFacts;
use crate::PackageMaterialization;
use crate::PackageResolution;
use crate::RegistryBackend;
use crate::StoreObject;
use crate::ToolCohort;
use crate::ToolIdentity;
use crate::WizerMode;
use crate::WkgLockPackage;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;
use crate::digest::is_count_within_bound;
use crate::model::COMPONENT_MANIFEST_SCHEMA;
use crate::model::ComponentManifest;

pub const WKG_LOCK_VERSION: u32 = 1;
pub const SOURCE_ACQUISITION_PLAN_SCHEMA: &str = "mantle-wasm-component-source-acquisition-plan-v1";
pub const REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM: &str = "not-runtime-authority";
pub const REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM: &str = "not-release-eligibility";

const MAX_REGISTRY_MAPPINGS: u32 = 64;
const MAX_PACKAGE_REQUIREMENTS: u32 = 256;
const MAX_LOCAL_PACKAGE_OVERRIDES: u32 = 256;
const MAX_LOCK_PACKAGES: u32 = 4096;
const MAX_LOCK_VERSIONS_PER_PACKAGE: u32 = 64;
const MAX_PACKAGE_MATERIALIZATIONS: u32 = 4096;
const MAX_OUTPUT_DECLARATIONS: u32 = 128;
const MAX_NON_CLAIMS: u32 = 64;
const MAX_OPTIONAL_STAGE_INPUTS: u32 = 256;
const MAX_COMPONENT_STRING_BYTES: u32 = 4096;
const CREDENTIAL_HANDLE_PREFIX: &str = "secret://";
const EXACT_REQUIREMENT_PREFIX: char = '=';
const WIZER_LINKER: &str = "wasm-component-ld";
const WIZER_COMPONENTIZER: &str = "wasm-tools";
const WIZER_LINKER_ARG: &str = "--skip-wit-component";
const WIZER_RUSTFLAGS_KEY: &str = "RUSTFLAGS";
const WIZER_RUSTFLAGS_VALUE: &str = "-C link-arg=--skip-wit-component";
const WIZER_LINKER_ENV_KEY: &str = "CARGO_TARGET_WASM32_WASIP2_LINKER";
const WIZER_COMPILE_ENVIRONMENT_ENTRY_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestValidation {
    pub manifest_identity_blake3: Option<Blake3Identity>,
    pub cohort_identity_blake3: Option<Blake3Identity>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockValidationRequest {
    pub manifest: ComponentManifest,
    pub facts: LockFacts,
    pub lock_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockValidation {
    pub lock_identity_blake3: Blake3Identity,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFetchPlan {
    pub package: String,
    pub version: String,
    pub registry: String,
    pub protocol_digest: crate::OciSha256Digest,
    pub expected_object: StoreObject,
    pub credential_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAcquisitionPlan {
    pub schema: String,
    pub lock_identity_blake3: Blake3Identity,
    pub registry_config_identity_blake3: Blake3Identity,
    pub requests: Vec<PackageFetchPlan>,
    pub plan_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePlanResult {
    pub plan: Option<SourceAcquisitionPlan>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Serialize)]
struct SourcePlanIdentityInput {
    schema: String,
    lock_identity_blake3: Blake3Identity,
    registry_config_identity_blake3: Blake3Identity,
    requests: Vec<PackageFetchPlan>,
}

pub fn validate_manifest(manifest: ComponentManifest) -> ManifestValidation {
    let mut blockers = Vec::new();
    validate_manifest_header(&manifest, &mut blockers);
    validate_package_resolution(&manifest.package_resolution, &mut blockers);
    validate_store_object(&manifest.wit.source, "wit.source", &mut blockers);
    validate_store_object(&manifest.implementation.source, "implementation.source", &mut blockers);
    validate_tool_cohort(&manifest.cohort, &mut blockers);
    validate_outputs(&manifest, &mut blockers);
    validate_non_claims(&manifest, &mut blockers);
    validate_optional_stage_config(&manifest, &mut blockers);
    let mut composition_blockers = crate::validate_composition(manifest.composition.clone()).blockers;
    let mut virtualization_blockers = crate::plan_virtualization(manifest.virtualization.clone()).blockers;
    blockers.append(&mut composition_blockers);
    blockers.append(&mut virtualization_blockers);

    let manifest_identity_blake3 = identity_if_clean(manifest.clone(), &mut blockers, "manifest-identity");
    let cohort_identity_blake3 = identity_if_clean(manifest.cohort, &mut blockers, "cohort-identity");
    debug_assert!(blockers.is_empty() || manifest_identity_blake3.is_none());
    debug_assert!(blockers.is_empty() || cohort_identity_blake3.is_none());
    ManifestValidation {
        manifest_identity_blake3,
        cohort_identity_blake3,
        blockers,
    }
}

pub fn cohort_identity(cohort: ToolCohort) -> Result<Blake3Identity, Vec<ComponentBlocker>> {
    let mut blockers = Vec::new();
    validate_tool_cohort(&cohort, &mut blockers);
    if !blockers.is_empty() {
        return Err(blockers);
    }
    let identity = canonical_identity(cohort).map_err(|error| vec![digest_blocker("cohort-identity", error)])?;
    debug_assert_eq!(identity.clone().into_hex().len(), crate::BLAKE3_HEX_LENGTH);
    debug_assert!(identity.clone().into_hex().bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(identity)
}

pub fn registry_config_identity(config: PackageResolution) -> Result<Blake3Identity, DigestError> {
    canonical_identity((config.default_registry, config.registries))
}

pub fn validate_lock(request: LockValidationRequest) -> LockValidation {
    let lock_identity = Blake3Identity::from_bytes(request.lock_bytes.clone());
    let mut blockers = validate_manifest(request.manifest.clone()).blockers;
    validate_lock_shape(&request.facts, &mut blockers);
    if lock_request_exceeds_bounds(&request) {
        return LockValidation {
            lock_identity_blake3: lock_identity,
            blockers,
        };
    }
    validate_lock_bytes(&request.facts, &lock_identity, &mut blockers);
    validate_registry_identity(&request, &mut blockers);
    validate_direct_requirements(&request, &mut blockers);
    validate_materializations(&request.facts, &mut blockers);
    debug_assert!(blockers.is_empty() || !blockers[0].code.is_empty());
    debug_assert_eq!(lock_identity.clone().into_hex().len(), crate::BLAKE3_HEX_LENGTH);
    LockValidation {
        lock_identity_blake3: lock_identity,
        blockers,
    }
}

pub fn plan_source_acquisition(request: LockValidationRequest) -> SourcePlanResult {
    let validation = validate_lock(request.clone());
    if !validation.blockers.is_empty() {
        return SourcePlanResult {
            plan: None,
            blockers: validation.blockers,
        };
    }
    let mut requests = package_fetch_requests(&request);
    requests.sort_by(|left, right| left.package.cmp(&right.package).then(left.version.cmp(&right.version)));
    let identity_input = SourcePlanIdentityInput {
        schema: String::from(SOURCE_ACQUISITION_PLAN_SCHEMA),
        lock_identity_blake3: validation.lock_identity_blake3.clone(),
        registry_config_identity_blake3: request.facts.registry_config_blake3.clone(),
        requests: requests.clone(),
    };
    let plan_identity = match canonical_identity(identity_input) {
        Ok(identity) => identity,
        Err(error) => {
            return SourcePlanResult {
                plan: None,
                blockers: vec![digest_blocker("source-plan", error)],
            };
        }
    };
    debug_assert!(!requests.is_empty());
    debug_assert!(is_count_within_bound(requests.len(), MAX_LOCK_PACKAGES));
    SourcePlanResult {
        plan: Some(SourceAcquisitionPlan {
            schema: String::from(SOURCE_ACQUISITION_PLAN_SCHEMA),
            lock_identity_blake3: validation.lock_identity_blake3,
            registry_config_identity_blake3: request.facts.registry_config_blake3,
            requests,
            plan_identity_blake3: plan_identity,
        }),
        blockers: Vec::new(),
    }
}

fn validate_manifest_header(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    if manifest.schema != COMPONENT_MANIFEST_SCHEMA {
        blockers.push(blocker(
            "unsupported-manifest-schema",
            "manifest.schema",
            "component manifest schema is unsupported",
        ));
    }
    validate_non_empty((&manifest.name, "manifest.name"), blockers);
    validate_non_empty((&manifest.wit.package, "wit.package"), blockers);
    validate_non_empty((&manifest.wit.world, "wit.world"), blockers);
    validate_non_empty((&manifest.implementation.package, "implementation.package"), blockers);
    validate_non_empty((&manifest.implementation.crate_name, "implementation.crate_name"), blockers);
}

fn validate_package_resolution(config: &PackageResolution, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    validate_count(config.registries.len(), MAX_REGISTRY_MAPPINGS, "registries", blockers);
    validate_count(config.requirements.len(), MAX_PACKAGE_REQUIREMENTS, "requirements", blockers);
    validate_count(config.local_overrides.len(), MAX_LOCAL_PACKAGE_OVERRIDES, "local_overrides", blockers);
    validate_non_empty((&config.default_registry, "package_resolution.default_registry"), blockers);
    validate_non_empty((&config.lock_path, "package_resolution.lock_path"), blockers);
    validate_requirements(config, blockers);
    validate_requirement_registries(config, blockers);
    validate_local_overrides(config, blockers);
    if !is_count_above_bound(config.registries.len(), MAX_REGISTRY_MAPPINGS) {
        blockers.reserve(config.registries.len());
        let mut namespaces = BTreeSet::new();
        let mut registry_names = BTreeSet::new();
        for registry in &config.registries {
            validate_non_empty((&registry.namespace, "registry.namespace"), blockers);
            validate_non_empty((&registry.registry, "registry.registry"), blockers);
            validate_registry_backend(registry, blockers);
            if !namespaces.insert(registry.namespace.clone()) {
                blockers.push(blocker(
                    "duplicate-registry-namespace",
                    &registry.namespace,
                    "registry namespace mapping is duplicated",
                ));
            }
            if !registry_names.insert(registry.registry.clone()) {
                blockers.push(blocker(
                    "duplicate-registry-transport",
                    &registry.registry,
                    "registry transport mapping is duplicated and would emit conflicting wkg config tables",
                ));
            }
            if let Some(handle) = &registry.credential_handle
                && !handle.starts_with(CREDENTIAL_HANDLE_PREFIX)
            {
                blockers.push(blocker(
                    "invalid-credential-handle",
                    &registry.registry,
                    "registry credentials must be referenced by a secret:// handle",
                ));
            }
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_registry_backend(registry: &crate::RegistryMapping, blockers: &mut Vec<ComponentBlocker>) {
    match registry.backend {
        RegistryBackend::Oci => {
            if registry.oci_registry.as_ref().is_none_or(|value| value.is_empty()) || registry.local_root.is_some() {
                blockers.push(blocker(
                    "invalid-oci-registry-config",
                    &registry.registry,
                    "OCI registry mapping requires an OCI registry and forbids a local root",
                ));
            }
        }
        RegistryBackend::Local => {
            if registry.local_root.as_ref().is_none_or(|value| !value.starts_with('/'))
                || registry.oci_registry.is_some()
                || registry.credential_handle.is_some()
            {
                blockers.push(blocker(
                    "invalid-local-registry-config",
                    &registry.registry,
                    "local registry mapping requires an absolute root and forbids OCI or credential fields",
                ));
            }
        }
    }
}

fn validate_requirements(config: &PackageResolution, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(config.requirements.len(), MAX_PACKAGE_REQUIREMENTS) {
        debug_assert!(blockers.len() >= blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    blockers.reserve(config.requirements.len());
    let mut packages = BTreeSet::new();
    for requirement in &config.requirements {
        validate_non_empty((&requirement.package, "requirement.package"), blockers);
        validate_non_empty((&requirement.registry, "requirement.registry"), blockers);
        if exact_version(&requirement.requirement).is_none() {
            blockers.push(blocker(
                "non-exact-package-requirement",
                &requirement.package,
                "component package requirements must use =<version>",
            ));
        }
        if !packages.insert(requirement.package.clone()) {
            blockers.push(blocker(
                "duplicate-package-requirement",
                &requirement.package,
                "component package requirement is duplicated",
            ));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_requirement_registries(config: &PackageResolution, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(config.requirements.len(), MAX_PACKAGE_REQUIREMENTS)
        || is_count_above_bound(config.registries.len(), MAX_REGISTRY_MAPPINGS)
    {
        debug_assert!(blockers.len() >= blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    blockers.reserve(config.requirements.len());
    for requirement in &config.requirements {
        let Some(namespace) = requirement.package.split(':').next() else {
            continue;
        };
        let is_mapped = config
            .registries
            .iter()
            .any(|mapping| mapping.namespace == namespace && mapping.registry == requirement.registry);
        if !is_mapped && requirement.registry != config.default_registry {
            blockers.push(blocker(
                "unmapped-package-registry",
                &requirement.package,
                "package requirement registry is not admitted by its namespace mapping or default registry",
            ));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_local_overrides(config: &PackageResolution, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(config.local_overrides.len(), MAX_LOCAL_PACKAGE_OVERRIDES) {
        debug_assert!(blockers.len() >= blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    let mut packages = BTreeSet::new();
    for local_override in &config.local_overrides {
        if local_override.package.is_empty() || !local_override.path.starts_with('/') {
            blockers.push(blocker(
                "invalid-local-package-override",
                &local_override.package,
                "local package override must bind a package to an absolute immutable path",
            ));
        }
        if !packages.insert(local_override.package.clone()) {
            blockers.push(blocker(
                "duplicate-local-package-override",
                &local_override.package,
                "local package override is duplicated",
            ));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_tool_cohort(cohort: &ToolCohort, blockers: &mut Vec<ComponentBlocker>) {
    validate_non_empty((&cohort.rust_target, "cohort.rust_target"), blockers);
    let tools = [
        ("rust-toolchain", &cohort.rust_toolchain),
        ("wkg", &cohort.wkg),
        ("wit-bindgen", &cohort.wit_bindgen),
        ("wasm-component-ld", &cohort.wasm_component_ld),
        ("wasm-tools", &cohort.wasm_tools),
        ("wac", &cohort.wac),
        ("wasi-virt", &cohort.wasi_virt),
        ("wizer", &cohort.wizer),
        ("wasmtime", &cohort.wasmtime),
    ];
    for (name, tool) in tools {
        validate_tool(name, tool, blockers);
    }
}

fn validate_tool(name: &str, tool: &ToolIdentity, blockers: &mut Vec<ComponentBlocker>) {
    validate_non_empty((&tool.version, name), blockers);
    validate_store_object(&tool.executable, name, blockers);
}

fn validate_outputs(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    validate_count(manifest.outputs.len(), MAX_OUTPUT_DECLARATIONS, "outputs", blockers);
    if manifest.outputs.is_empty() {
        blockers.push(blocker(
            "missing-component-output",
            "outputs",
            "component manifest must declare at least one typed output",
        ));
    }
    if is_count_above_bound(manifest.outputs.len(), MAX_OUTPUT_DECLARATIONS) {
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    blockers.reserve(manifest.outputs.len());
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for output in &manifest.outputs {
        validate_non_empty((&output.name, "output.name"), blockers);
        validate_relative_path((&output.path, &output.name), blockers);
        if !names.insert(output.name.clone()) {
            blockers.push(blocker("duplicate-output-name", &output.name, "component output name is duplicated"));
        }
        if !paths.insert(output.path.clone()) {
            blockers.push(blocker("duplicate-output-path", &output.path, "component output path is duplicated"));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_optional_stage_config(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    validate_wizer_stage_config(manifest, blockers);
    validate_optional_stage_sets(manifest, blockers);
    if manifest.aot.mode == AotMode::TrustedNative
        && (manifest.aot.target.as_ref().is_none_or(|target| target.is_empty())
            || manifest.aot.wasmtime_configuration_identity_blake3.is_none())
    {
        blockers.push(blocker(
            "incomplete-aot-configuration",
            "aot",
            "trusted-native AOT configuration requires target and Wasmtime configuration identity",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_wizer_stage_config(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    let is_disabled = manifest.wizer.mode == WizerMode::Disabled;
    if is_disabled {
        if manifest.wizer.linker_split.is_some() || manifest.wizer.componentization.is_some() {
            blockers.push(blocker(
                "disabled-wizer-stage-config",
                "wizer",
                "disabled Wizer configuration must not declare linker or componentization stages",
            ));
        }
        return;
    }
    if manifest.wizer.initialization_entrypoint.as_ref().is_none_or(|entrypoint| entrypoint.is_empty()) {
        blockers.push(blocker(
            "missing-wizer-entrypoint",
            "wizer",
            "enabled Wizer configuration requires an initialization entrypoint",
        ));
    }
    validate_wizer_linker_split(manifest, blockers);
    validate_wizer_componentization(manifest, blockers);
    debug_assert!(!is_disabled);
    debug_assert!(manifest.wizer.mode != WizerMode::Disabled);
}

fn validate_wizer_linker_split(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    let Some(linker) = &manifest.wizer.linker_split else {
        blockers.push(blocker(
            "missing-wizer-linker-split",
            "wizer.linker-split",
            "enabled Wizer requires an explicit wasm-component-ld core-module split",
        ));
        return;
    };
    let has_linker = linker.linker == WIZER_LINKER;
    let has_split_arg = linker.linker_args.as_slice() == [WIZER_LINKER_ARG];
    let has_rustflags = linker
        .compile_environment
        .get(WIZER_RUSTFLAGS_KEY)
        .is_some_and(|value| value == WIZER_RUSTFLAGS_VALUE);
    let has_linker_env =
        linker.compile_environment.get(WIZER_LINKER_ENV_KEY).is_some_and(|value| value == WIZER_LINKER);
    let has_exact_environment =
        linker.compile_environment.len() == WIZER_COMPILE_ENVIRONMENT_ENTRY_COUNT && has_rustflags && has_linker_env;
    if !has_linker || !has_split_arg || !has_exact_environment {
        blockers.push(blocker(
            "invalid-wizer-linker-split",
            "wizer.linker-split",
            "Wizer linker split must bind the pinned linker, skip-wit-component argument, and exact compile environment",
        ));
    }
    debug_assert!(!linker.linker.is_empty());
    debug_assert!(!linker.linker_args.is_empty());
}

fn validate_wizer_componentization(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    let Some(componentization) = &manifest.wizer.componentization else {
        blockers.push(blocker(
            "missing-wizer-componentization",
            "wizer.componentization",
            "enabled Wizer requires an explicit re-componentization stage",
        ));
        return;
    };
    if componentization.tool != WIZER_COMPONENTIZER || !componentization.args.is_empty() {
        blockers.push(blocker(
            "invalid-wizer-componentization",
            "wizer.componentization",
            "bounded Wizer re-componentization requires pinned wasm-tools with no implicit options",
        ));
    }
    debug_assert!(!componentization.tool.is_empty());
    debug_assert!(componentization.args.is_empty() || componentization.tool == WIZER_COMPONENTIZER);
}

fn validate_optional_stage_sets(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(manifest.wizer.deterministic_virtual_imports.len(), MAX_OPTIONAL_STAGE_INPUTS)
        || is_count_above_bound(manifest.aot.cpu_features.len(), MAX_OPTIONAL_STAGE_INPUTS)
    {
        blockers.push(blocker(
            "optional-stage-input-limit",
            "wizer-or-aot",
            "Wizer virtual imports or AOT CPU features exceed the fixed bound",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    let wizer_inputs: BTreeSet<String> = manifest.wizer.deterministic_virtual_imports.iter().cloned().collect();
    let cpu_features: BTreeSet<String> = manifest.aot.cpu_features.iter().cloned().collect();
    if wizer_inputs.len() != manifest.wizer.deterministic_virtual_imports.len()
        || cpu_features.len() != manifest.aot.cpu_features.len()
    {
        blockers.push(blocker(
            "duplicate-optional-stage-input",
            "wizer-or-aot",
            "Wizer virtual imports and AOT CPU features must be unique",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_non_claims(manifest: &ComponentManifest, blockers: &mut Vec<ComponentBlocker>) {
    validate_count(manifest.non_claims.len(), MAX_NON_CLAIMS, "non_claims", blockers);
    if is_count_above_bound(manifest.non_claims.len(), MAX_NON_CLAIMS) {
        return;
    }
    for required in [
        REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM,
        REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM,
    ] {
        if !manifest.non_claims.iter().any(|value| value == required) {
            blockers.push(blocker(
                "missing-required-non-claim",
                required,
                "component manifest omits a required build-only non-claim",
            ));
        }
    }
}

fn validate_lock_shape(facts: &LockFacts, blockers: &mut Vec<ComponentBlocker>) {
    if facts.lock.version != WKG_LOCK_VERSION {
        blockers.push(blocker("unsupported-wkg-lock-version", "wkg.lock", "wkg.lock version is unsupported"));
    }
    validate_count(facts.lock.packages.len(), MAX_LOCK_PACKAGES, "lock.packages", blockers);
    validate_count(facts.materializations.len(), MAX_PACKAGE_MATERIALIZATIONS, "materializations", blockers);
    if is_count_above_bound(facts.lock.packages.len(), MAX_LOCK_PACKAGES)
        || is_count_above_bound(facts.materializations.len(), MAX_PACKAGE_MATERIALIZATIONS)
    {
        return;
    }
    blockers.reserve(facts.lock.packages.len());
    let mut package_keys = BTreeSet::new();
    for package in &facts.lock.packages {
        validate_lock_package(package, &mut package_keys, blockers);
    }
}

fn validate_lock_package(
    package: &WkgLockPackage,
    package_keys: &mut BTreeSet<(String, String)>,
    blockers: &mut Vec<ComponentBlocker>,
) {
    let blocker_count_before = blockers.len();
    validate_non_empty((&package.name, "lock.package.name"), blockers);
    validate_non_empty((&package.registry, "lock.package.registry"), blockers);
    validate_count(package.versions.len(), MAX_LOCK_VERSIONS_PER_PACKAGE, &package.name, blockers);
    if !package_keys.insert((package.name.clone(), package.registry.clone())) {
        blockers.push(blocker(
            "duplicate-lock-package",
            &package.name,
            "wkg.lock package/registry entry is duplicated",
        ));
    }
    if is_count_above_bound(package.versions.len(), MAX_LOCK_VERSIONS_PER_PACKAGE) {
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    blockers.reserve(package.versions.len());
    let mut versions = BTreeSet::new();
    for version in &package.versions {
        if !versions.insert((version.requirement.clone(), version.version.clone())) {
            blockers.push(blocker(
                "duplicate-lock-version",
                &package.name,
                "wkg.lock requirement/version entry is duplicated",
            ));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_lock_bytes(facts: &LockFacts, measured: &Blake3Identity, blockers: &mut Vec<ComponentBlocker>) {
    if measured != &facts.lock_bytes_blake3 {
        blockers.push(blocker(
            "stale-lock-bytes",
            "wkg.lock",
            "wkg.lock bytes do not match the checked BLAKE3 identity",
        ));
    }
}

fn validate_registry_identity(request: &LockValidationRequest, blockers: &mut Vec<ComponentBlocker>) {
    match registry_config_identity(request.manifest.package_resolution.clone()) {
        Ok(identity) if identity == request.facts.registry_config_blake3 => {}
        Ok(_) => blockers.push(blocker(
            "registry-config-drift",
            "wkg-config.toml",
            "registry configuration identity differs from checked lock facts",
        )),
        Err(error) => blockers.push(digest_blocker("registry-config", error)),
    }
}

fn validate_direct_requirements(request: &LockValidationRequest, blockers: &mut Vec<ComponentBlocker>) {
    for requirement in &request.manifest.package_resolution.requirements {
        let Some(exact) = exact_version(&requirement.requirement) else {
            continue;
        };
        let matching = request
            .facts
            .lock
            .packages
            .iter()
            .find(|package| package.name == requirement.package && package.registry == requirement.registry);
        let Some(package) = matching else {
            blockers.push(blocker(
                "missing-lock-package",
                &requirement.package,
                "required package is absent from wkg.lock",
            ));
            continue;
        };
        let is_selected = package
            .versions
            .iter()
            .any(|version| version.requirement == requirement.requirement && version.version == exact);
        if !is_selected {
            blockers.push(blocker(
                "lock-version-drift",
                &requirement.package,
                "wkg.lock does not select the exact declared package version",
            ));
        }
    }
}

type MaterializationKey = (String, String, String, crate::OciSha256Digest);

// BTreeSet has no reserve API; the explicit materialization/lock guards below
// bound each insertion before either index is built.
#[allow(tigerstyle::unbounded_collection_growth)]
fn validate_materializations(facts: &LockFacts, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(facts.materializations.len(), MAX_PACKAGE_MATERIALIZATIONS)
        || is_count_above_bound(facts.lock.packages.len(), MAX_LOCK_PACKAGES)
        || facts
            .lock
            .packages
            .iter()
            .any(|package| is_count_above_bound(package.versions.len(), MAX_LOCK_VERSIONS_PER_PACKAGE))
    {
        blockers.push(blocker(
            "materialization-validation-limit",
            "materializations",
            "package materialization validation input exceeds fixed lock bounds",
        ));
        return;
    }
    let locked_keys = locked_materialization_keys(facts);
    let mut materialization_keys = BTreeSet::new();
    blockers.reserve(facts.materializations.len());
    for materialization in &facts.materializations {
        validate_store_object(&materialization.object, &materialization.package, blockers);
        let key = materialization_key(materialization);
        if !materialization_keys.insert(key.clone()) {
            blockers.push(blocker(
                "duplicate-package-materialization",
                &materialization.package,
                "package materialization identity is duplicated",
            ));
        }
        if !locked_keys.contains(&key) {
            blockers.push(blocker(
                "unexpected-package-materialization",
                &materialization.package,
                "package materialization is not bound by the checked wkg.lock",
            ));
        }
    }
    validate_required_materializations(&locked_keys, &materialization_keys, blockers);
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_required_materializations(
    locked_keys: &BTreeSet<MaterializationKey>,
    materialization_keys: &BTreeSet<MaterializationKey>,
    blockers: &mut Vec<ComponentBlocker>,
) {
    for locked_key in locked_keys {
        if !materialization_keys.contains(locked_key) {
            blockers.push(blocker(
                "missing-package-materialization",
                &locked_key.0,
                "locked package version has no exact immutable store input",
            ));
        }
    }
}

// BTreeSet has no reserve API; callers reject package/version counts above the
// fixed lock bounds before constructing this index.
#[allow(tigerstyle::unbounded_collection_growth)]
fn locked_materialization_keys(facts: &LockFacts) -> BTreeSet<MaterializationKey> {
    let mut keys = BTreeSet::new();
    for package in &facts.lock.packages {
        for version in &package.versions {
            keys.insert((
                package.name.clone(),
                package.registry.clone(),
                version.version.clone(),
                version.digest.clone(),
            ));
        }
    }
    keys
}

fn materialization_key(materialization: &PackageMaterialization) -> MaterializationKey {
    (
        materialization.package.clone(),
        materialization.registry.clone(),
        materialization.version.clone(),
        materialization.protocol_digest.clone(),
    )
}

fn lock_request_exceeds_bounds(request: &LockValidationRequest) -> bool {
    is_count_above_bound(request.manifest.package_resolution.requirements.len(), MAX_PACKAGE_REQUIREMENTS)
        || is_count_above_bound(request.facts.lock.packages.len(), MAX_LOCK_PACKAGES)
        || is_count_above_bound(request.facts.materializations.len(), MAX_PACKAGE_MATERIALIZATIONS)
        || request
            .facts
            .lock
            .packages
            .iter()
            .any(|package| is_count_above_bound(package.versions.len(), MAX_LOCK_VERSIONS_PER_PACKAGE))
}

fn package_fetch_requests(request: &LockValidationRequest) -> Vec<PackageFetchPlan> {
    request
        .facts
        .materializations
        .iter()
        .map(|materialization| package_fetch_request(materialization, &request.manifest.package_resolution))
        .collect()
}

fn package_fetch_request(materialization: &PackageMaterialization, config: &PackageResolution) -> PackageFetchPlan {
    let namespace = materialization.package.split(':').next().unwrap_or_default();
    let credential_handle = config
        .registries
        .iter()
        .find(|mapping| mapping.namespace == namespace && mapping.registry == materialization.registry)
        .and_then(|mapping| mapping.credential_handle.clone());
    PackageFetchPlan {
        package: materialization.package.clone(),
        version: materialization.version.clone(),
        registry: materialization.registry.clone(),
        protocol_digest: materialization.protocol_digest.clone(),
        expected_object: materialization.object.clone(),
        credential_handle,
    }
}

fn validate_store_object(object: &StoreObject, subject: &str, blockers: &mut Vec<ComponentBlocker>) {
    if !object.logical_path.starts_with('/') {
        blockers.push(blocker("non-absolute-store-object", subject, "store object locator must be absolute"));
    }
    if object.size_bytes == 0 {
        blockers.push(blocker("empty-store-object", subject, "identified store object must have a positive byte size"));
    }
}

fn validate_relative_path(field: (&str, &str), blockers: &mut Vec<ComponentBlocker>) {
    let (path, subject) = field;
    if path.is_empty() || path.starts_with('/') {
        blockers.push(blocker("invalid-output-path", subject, "component output path must be non-empty and relative"));
        return;
    }
    if path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        blockers.push(blocker(
            "invalid-output-path",
            subject,
            "component output path must not contain empty, dot, or parent segments",
        ));
    }
}

fn validate_non_empty(field: (&str, &str), blockers: &mut Vec<ComponentBlocker>) {
    let (value, subject) = field;
    if value.is_empty() {
        blockers.push(blocker("empty-component-field", subject, "component field must not be empty"));
    } else if is_count_above_bound(value.len(), MAX_COMPONENT_STRING_BYTES) {
        blockers.push(blocker("oversized-component-field", subject, "component field exceeds the bounded byte length"));
    }
}

fn validate_count(count: usize, maximum: u32, subject: &str, blockers: &mut Vec<ComponentBlocker>) {
    if is_count_above_bound(count, maximum) {
        blockers.push(blocker("component-collection-limit", subject, "component collection exceeds its fixed bound"));
    }
}

fn exact_version(requirement: &str) -> Option<String> {
    let version = requirement.strip_prefix(EXACT_REQUIREMENT_PREFIX)?;
    if version.is_empty() {
        return None;
    }
    Some(String::from(version))
}

fn identity_if_clean<T>(value: T, blockers: &mut Vec<ComponentBlocker>, subject: &str) -> Option<Blake3Identity>
where T: Serialize {
    if !blockers.is_empty() {
        return None;
    }
    match canonical_identity(value) {
        Ok(identity) => Some(identity),
        Err(error) => {
            blockers.push(digest_blocker(subject, error));
            None
        }
    }
}

fn digest_blocker(subject: &str, error: DigestError) -> ComponentBlocker {
    ComponentBlocker {
        code: String::from("canonical-identity-failed"),
        subject: String::from(subject),
        message: error.to_string(),
    }
}
