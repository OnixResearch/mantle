use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AotAdmission;
use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::ComponentStageKind;
use crate::PackageMaterialization;
use crate::StoreObject;
use crate::TransformAdmission;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;
use crate::manifest::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
use crate::manifest::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;

pub const MATERIALIZATION_BUNDLE_SCHEMA: &str = "mantle-wasm-component-materialization-bundle-v1";

const MAX_BUNDLE_WIT_INPUTS: u32 = 256;
const MAX_BUNDLE_PACKAGE_INPUTS: u32 = 4096;
const MAX_BUNDLE_STAGE_RECEIPTS: u32 = 64;
const MAX_BUNDLE_NON_CLAIMS: u32 = 64;

const REQUIRED_STAGE_KINDS: [ComponentStageKind; 8] = [
    ComponentStageKind::PackageResolution,
    ComponentStageKind::Lock,
    ComponentStageKind::BindingGeneration,
    ComponentStageKind::Compilation,
    ComponentStageKind::Composition,
    ComponentStageKind::Virtualization,
    ComponentStageKind::BuildValidation,
    ComponentStageKind::OctetValidation,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageReceiptReference {
    pub stage_key: String,
    pub kind: ComponentStageKind,
    pub receipt_blake3: Blake3Identity,
    pub artifact: Option<StoreObject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializationBundleRequest {
    pub name: String,
    pub manifest_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub wit_inputs: Vec<StoreObject>,
    pub package_inputs: Vec<PackageMaterialization>,
    pub source_closure: StoreObject,
    pub lock: StoreObject,
    pub final_portable: StoreObject,
    pub expected_octet_profile_blake3: Blake3Identity,
    pub expected_runtime_profile_blake3: Blake3Identity,
    pub stage_receipts: Vec<StageReceiptReference>,
    pub wizer: Option<TransformAdmission>,
    pub aot: Option<AotAdmission>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializationBundle {
    pub schema: String,
    pub name: String,
    pub manifest_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub wit_inputs: Vec<StoreObject>,
    pub package_inputs: Vec<PackageMaterialization>,
    pub source_closure: StoreObject,
    pub lock: StoreObject,
    pub final_portable: StoreObject,
    pub expected_octet_profile_blake3: Blake3Identity,
    pub expected_runtime_profile_blake3: Blake3Identity,
    pub stage_receipts: Vec<StageReceiptReference>,
    pub wizer: Option<TransformAdmission>,
    pub aot: Option<AotAdmission>,
    pub non_claims: Vec<String>,
    pub bundle_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializationBundleResult {
    pub bundle: Option<MaterializationBundle>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Serialize)]
struct MaterializationBundleIdentityInput {
    schema: String,
    name: String,
    manifest_blake3: Blake3Identity,
    cohort_blake3: Blake3Identity,
    wit_inputs: Vec<StoreObject>,
    package_inputs: Vec<PackageMaterialization>,
    source_closure: StoreObject,
    lock: StoreObject,
    final_portable: StoreObject,
    expected_octet_profile_blake3: Blake3Identity,
    expected_runtime_profile_blake3: Blake3Identity,
    stage_receipts: Vec<StageReceiptReference>,
    wizer: Option<TransformAdmission>,
    aot: Option<AotAdmission>,
    non_claims: Vec<String>,
}

pub fn build_materialization_bundle(mut request: MaterializationBundleRequest) -> MaterializationBundleResult {
    normalize_request(&mut request);
    let mut blockers = Vec::new();
    validate_request(&request, &mut blockers);
    if !blockers.is_empty() {
        return MaterializationBundleResult { bundle: None, blockers };
    }
    let identity_input = identity_input(&request);
    let identity = match canonical_identity(identity_input) {
        Ok(identity) => identity,
        Err(_) => {
            return MaterializationBundleResult {
                bundle: None,
                blockers: vec![blocker(
                    "materialization-bundle-identity-failed",
                    "materialization-bundle",
                    "materialization bundle could not be canonically identified",
                )],
            };
        }
    };
    debug_assert!(!request.stage_receipts.is_empty());
    debug_assert!(!request.non_claims.is_empty());
    MaterializationBundleResult {
        bundle: Some(bundle_from_request(request, identity)),
        blockers: Vec::new(),
    }
}

pub fn verify_materialization_bundle(bundle: MaterializationBundle) -> MaterializationBundleResult {
    if bundle.schema != MATERIALIZATION_BUNDLE_SCHEMA {
        return MaterializationBundleResult {
            bundle: None,
            blockers: vec![blocker(
                "unsupported-materialization-bundle-schema",
                "materialization-bundle",
                "materialization bundle schema is unsupported",
            )],
        };
    }
    let expected_identity = bundle.bundle_identity_blake3.clone();
    let request = request_from_bundle(bundle);
    let result = build_materialization_bundle(request);
    let Some(rebuilt) = result.bundle else {
        return result;
    };
    if rebuilt.bundle_identity_blake3 != expected_identity {
        return MaterializationBundleResult {
            bundle: None,
            blockers: vec![blocker(
                "materialization-bundle-identity-mismatch",
                "materialization-bundle",
                "materialization bundle identity does not match its canonical fields",
            )],
        };
    }
    debug_assert_eq!(rebuilt.bundle_identity_blake3, expected_identity);
    debug_assert_eq!(rebuilt.schema, MATERIALIZATION_BUNDLE_SCHEMA);
    MaterializationBundleResult {
        bundle: Some(rebuilt),
        blockers: Vec::new(),
    }
}

fn normalize_request(request: &mut MaterializationBundleRequest) {
    request.wit_inputs.sort_by(|left, right| left.digest_blake3.cmp(&right.digest_blake3));
    request.package_inputs.sort_by(|left, right| {
        left.package
            .cmp(&right.package)
            .then(left.version.cmp(&right.version))
            .then(left.object.digest_blake3.cmp(&right.object.digest_blake3))
    });
    request.stage_receipts.sort_by(|left, right| left.stage_key.cmp(&right.stage_key));
    request.non_claims.sort();
    debug_assert!(
        request
            .wit_inputs
            .windows(crate::ADJACENT_WINDOW_LENGTH)
            .all(|pair| { pair[0].digest_blake3 <= pair[1].digest_blake3 })
    );
    debug_assert!(
        request
            .stage_receipts
            .windows(crate::ADJACENT_WINDOW_LENGTH)
            .all(|pair| { pair[0].stage_key <= pair[1].stage_key })
    );
}

fn validate_request(request: &MaterializationBundleRequest, blockers: &mut Vec<ComponentBlocker>) {
    validate_bounds(request, blockers);
    if request.name.is_empty() {
        blockers.push(blocker(
            "empty-materialization-bundle-name",
            "materialization-bundle.name",
            "materialization bundle name must not be empty",
        ));
    }
    validate_store_object(&request.source_closure, "source-closure", blockers);
    validate_store_object(&request.lock, "wkg.lock", blockers);
    validate_store_object(&request.final_portable, "final-portable", blockers);
    validate_input_objects(request, blockers);
    validate_stage_receipts(request, blockers);
    validate_non_claims(&request.non_claims, blockers);
    validate_optional_outputs(request, blockers);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    debug_assert!(blockers.iter().all(|item| !item.message.is_empty()));
}

fn validate_bounds(request: &MaterializationBundleRequest, blockers: &mut Vec<ComponentBlocker>) {
    let exceeds_bound = is_count_above_bound(request.wit_inputs.len(), MAX_BUNDLE_WIT_INPUTS)
        || is_count_above_bound(request.package_inputs.len(), MAX_BUNDLE_PACKAGE_INPUTS)
        || is_count_above_bound(request.stage_receipts.len(), MAX_BUNDLE_STAGE_RECEIPTS)
        || is_count_above_bound(request.non_claims.len(), MAX_BUNDLE_NON_CLAIMS);
    if exceeds_bound {
        blockers.push(blocker(
            "materialization-bundle-limit",
            "materialization-bundle",
            "materialization bundle collection exceeds a fixed bound",
        ));
    }
    if request.wit_inputs.is_empty() || request.stage_receipts.is_empty() {
        blockers.push(blocker(
            "incomplete-materialization-bundle",
            "materialization-bundle",
            "materialization bundle requires WIT inputs and stage receipts",
        ));
    }
}

fn validate_input_objects(request: &MaterializationBundleRequest, blockers: &mut Vec<ComponentBlocker>) {
    if is_count_above_bound(request.wit_inputs.len(), MAX_BUNDLE_WIT_INPUTS)
        || is_count_above_bound(request.package_inputs.len(), MAX_BUNDLE_PACKAGE_INPUTS)
    {
        return;
    }
    for input in &request.wit_inputs {
        validate_store_object(input, "wit-input", blockers);
    }
    for input in &request.package_inputs {
        validate_store_object(&input.object, &input.package, blockers);
    }
}

fn validate_stage_receipts(request: &MaterializationBundleRequest, blockers: &mut Vec<ComponentBlocker>) {
    if is_count_above_bound(request.stage_receipts.len(), MAX_BUNDLE_STAGE_RECEIPTS) {
        return;
    }
    let mut keys = BTreeSet::new();
    let mut identities = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for receipt in &request.stage_receipts {
        if receipt.stage_key.is_empty() || !keys.insert(receipt.stage_key.clone()) {
            blockers.push(blocker(
                "duplicate-or-empty-bundle-stage",
                &receipt.stage_key,
                "bundle stage keys must be unique and non-empty",
            ));
        }
        if !identities.insert(receipt.receipt_blake3.clone()) {
            blockers.push(blocker(
                "duplicate-bundle-stage-receipt",
                &receipt.stage_key,
                "bundle stage receipt identities must be unique",
            ));
        }
        kinds.insert(receipt.kind);
        if let Some(artifact) = &receipt.artifact {
            validate_store_object(artifact, &receipt.stage_key, blockers);
        }
    }
    for required in REQUIRED_STAGE_KINDS {
        if !kinds.contains(&required) {
            blockers.push(blocker(
                "missing-bundle-stage-receipt",
                &stage_label(required),
                "materialization bundle omits a required stage receipt",
            ));
        }
    }
}

fn validate_non_claims(non_claims: &[String], blockers: &mut Vec<ComponentBlocker>) {
    if is_count_above_bound(non_claims.len(), MAX_BUNDLE_NON_CLAIMS) {
        return;
    }
    let unique: BTreeSet<String> = non_claims.iter().cloned().collect();
    if unique.len() != non_claims.len() {
        blockers.push(blocker(
            "duplicate-materialization-non-claim",
            "materialization-bundle.non-claims",
            "materialization bundle non-claims must be unique",
        ));
    }
    for required in [
        REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM,
        REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM,
    ] {
        if !unique.contains(required) {
            blockers.push(blocker(
                "missing-materialization-non-claim",
                required,
                "materialization bundle omits a required non-claim",
            ));
        }
    }
}

fn validate_optional_outputs(request: &MaterializationBundleRequest, blockers: &mut Vec<ComponentBlocker>) {
    if let Some(wizer) = &request.wizer {
        if !wizer.eligible_for_bundle || wizer.output.digest_blake3 != request.final_portable.digest_blake3 {
            blockers.push(blocker(
                "ineligible-wizer-bundle-output",
                "wizer",
                "bundle Wizer output must be deterministic-eligible and match final portable bytes",
            ));
        }
        if !request.stage_receipts.iter().any(|receipt| receipt.kind == ComponentStageKind::Wizer) {
            blockers.push(blocker(
                "missing-wizer-stage-receipt",
                "wizer",
                "bundle carrying Wizer output must include its stage receipt",
            ));
        }
    }
    if let Some(aot) = &request.aot {
        if aot.source_component_blake3 != request.final_portable.digest_blake3 {
            blockers.push(blocker(
                "aot-bundle-source-mismatch",
                "aot",
                "bundle AOT source identity differs from final portable bytes",
            ));
        }
        if !request.stage_receipts.iter().any(|receipt| receipt.kind == ComponentStageKind::Aot) {
            blockers.push(blocker(
                "missing-aot-stage-receipt",
                "aot",
                "bundle carrying AOT output must include its stage receipt",
            ));
        }
    }
}

fn validate_store_object(object: &StoreObject, subject: &str, blockers: &mut Vec<ComponentBlocker>) {
    if object.size_bytes == 0 || !object.logical_path.starts_with('/') {
        blockers.push(blocker(
            "invalid-materialization-object",
            subject,
            "materialization object must have an absolute locator and positive byte size",
        ));
    }
}

fn identity_input(request: &MaterializationBundleRequest) -> MaterializationBundleIdentityInput {
    MaterializationBundleIdentityInput {
        schema: String::from(MATERIALIZATION_BUNDLE_SCHEMA),
        name: request.name.clone(),
        manifest_blake3: request.manifest_blake3.clone(),
        cohort_blake3: request.cohort_blake3.clone(),
        wit_inputs: request.wit_inputs.clone(),
        package_inputs: request.package_inputs.clone(),
        source_closure: request.source_closure.clone(),
        lock: request.lock.clone(),
        final_portable: request.final_portable.clone(),
        expected_octet_profile_blake3: request.expected_octet_profile_blake3.clone(),
        expected_runtime_profile_blake3: request.expected_runtime_profile_blake3.clone(),
        stage_receipts: request.stage_receipts.clone(),
        wizer: request.wizer.clone(),
        aot: request.aot.clone(),
        non_claims: request.non_claims.clone(),
    }
}

fn bundle_from_request(request: MaterializationBundleRequest, identity: Blake3Identity) -> MaterializationBundle {
    MaterializationBundle {
        schema: String::from(MATERIALIZATION_BUNDLE_SCHEMA),
        name: request.name,
        manifest_blake3: request.manifest_blake3,
        cohort_blake3: request.cohort_blake3,
        wit_inputs: request.wit_inputs,
        package_inputs: request.package_inputs,
        source_closure: request.source_closure,
        lock: request.lock,
        final_portable: request.final_portable,
        expected_octet_profile_blake3: request.expected_octet_profile_blake3,
        expected_runtime_profile_blake3: request.expected_runtime_profile_blake3,
        stage_receipts: request.stage_receipts,
        wizer: request.wizer,
        aot: request.aot,
        non_claims: request.non_claims,
        bundle_identity_blake3: identity,
    }
}

fn request_from_bundle(bundle: MaterializationBundle) -> MaterializationBundleRequest {
    MaterializationBundleRequest {
        name: bundle.name,
        manifest_blake3: bundle.manifest_blake3,
        cohort_blake3: bundle.cohort_blake3,
        wit_inputs: bundle.wit_inputs,
        package_inputs: bundle.package_inputs,
        source_closure: bundle.source_closure,
        lock: bundle.lock,
        final_portable: bundle.final_portable,
        expected_octet_profile_blake3: bundle.expected_octet_profile_blake3,
        expected_runtime_profile_blake3: bundle.expected_runtime_profile_blake3,
        stage_receipts: bundle.stage_receipts,
        wizer: bundle.wizer,
        aot: bundle.aot,
        non_claims: bundle.non_claims,
    }
}

fn stage_label(kind: ComponentStageKind) -> String {
    let label = match kind {
        ComponentStageKind::PackageResolution => "package-resolution",
        ComponentStageKind::Lock => "lock",
        ComponentStageKind::BindingGeneration => "binding-generation",
        ComponentStageKind::Compilation => "compilation",
        ComponentStageKind::Composition => "composition",
        ComponentStageKind::Virtualization => "virtualization",
        ComponentStageKind::BuildValidation => "build-validation",
        ComponentStageKind::OctetValidation => "octet-validation",
        ComponentStageKind::Wizer => "wizer",
        ComponentStageKind::Aot => "aot",
        ComponentStageKind::MaterializationBundle => "materialization-bundle",
    };
    String::from(label)
}
