use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::OutputClass;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;
use crate::digest::is_count_within_bound;

pub const GENERATED_INPUT_PLAN_SCHEMA: &str = "mantle-wasm-component-generated-input-plan-v1";
pub const GENERATED_INPUT_RECEIPT_SCHEMA: &str = "mantle-wasm-component-generated-input-receipt-v1";
pub const GENERATED_INPUT_OWNER: &str = "mantle-wasm-component-export";

const MAX_GENERATED_INPUTS: u32 = 64;
const MAX_GENERATED_CONTENT_BYTES: u32 = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedInputOwner {
    pub schema: String,
    pub generator: String,
    pub export_name: String,
    pub source_identity_blake3: Blake3Identity,
    pub dependency_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedInputCandidate {
    pub name: String,
    pub target: String,
    pub output_class: OutputClass,
    pub owner: GeneratedInputOwner,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedInputReceipt {
    pub name: String,
    pub target: String,
    pub output_class: OutputClass,
    pub owner: GeneratedInputOwner,
    pub content: String,
    pub content_identity_blake3: Blake3Identity,
    pub receipt_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedInputPlan {
    pub schema: String,
    pub inputs: Vec<GeneratedInputReceipt>,
    pub plan_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedInputPlanResult {
    pub plan: Option<GeneratedInputPlan>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedGeneratedInput {
    pub target: String,
    pub content: Option<Vec<u8>>,
    pub receipt_identity_blake3: Option<Blake3Identity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedFreshnessValidation {
    pub fresh: bool,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Serialize)]
struct ReceiptIdentityInput {
    name: String,
    target: String,
    output_class: OutputClass,
    owner: GeneratedInputOwner,
    content_identity_blake3: Blake3Identity,
}

#[derive(Serialize)]
struct PlanIdentityInput {
    schema: String,
    inputs: Vec<GeneratedInputReceipt>,
}

struct OwnedInputFields<'a> {
    name: &'a str,
    target: &'a str,
    owner: &'a GeneratedInputOwner,
    content_byte_count: usize,
}

pub fn finalize_generated_inputs(mut candidates: Vec<GeneratedInputCandidate>) -> GeneratedInputPlanResult {
    let is_candidate_count_within_bound = is_count_within_bound(candidates.len(), MAX_GENERATED_INPUTS);
    // Vec capacity is measured in blocker items, which this lint does not recognize as a unit.
    #[allow(tigerstyle::numeric_units)]
    let blocker_capacity_items_count = if is_candidate_count_within_bound {
        candidates.len()
    } else {
        1
    };
    let mut blockers = Vec::with_capacity(blocker_capacity_items_count);
    validate_candidates(&candidates, &mut blockers);
    if !blockers.is_empty() {
        return GeneratedInputPlanResult { plan: None, blockers };
    }
    candidates.sort_by(|left, right| left.target.cmp(&right.target));
    let mut receipts = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        match finalize_candidate(candidate) {
            Ok(receipt) => receipts.push(receipt),
            Err(blocker) => blockers.push(blocker),
        }
    }
    if !blockers.is_empty() {
        return GeneratedInputPlanResult { plan: None, blockers };
    }
    let identity_input = PlanIdentityInput {
        schema: String::from(GENERATED_INPUT_PLAN_SCHEMA),
        inputs: receipts.clone(),
    };
    let plan_identity = match canonical_identity(identity_input) {
        Ok(identity) => identity,
        Err(_) => {
            return GeneratedInputPlanResult {
                plan: None,
                blockers: vec![blocker(
                    "generated-plan-identity-failed",
                    "generated-inputs",
                    "generated input plan could not be canonically identified",
                )],
            };
        }
    };
    debug_assert!(!receipts.is_empty());
    debug_assert!(receipts.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0].target <= pair[1].target));
    GeneratedInputPlanResult {
        plan: Some(GeneratedInputPlan {
            schema: String::from(GENERATED_INPUT_PLAN_SCHEMA),
            inputs: receipts,
            plan_identity_blake3: plan_identity,
        }),
        blockers: Vec::new(),
    }
}

pub fn verify_generated_freshness(
    plan: GeneratedInputPlan,
    observed: Vec<ObservedGeneratedInput>,
) -> GeneratedFreshnessValidation {
    let mut blockers = Vec::new();
    validate_generated_plan(&plan, &mut blockers);
    if !blockers.is_empty() {
        return freshness_result(blockers);
    }
    validate_observed_inputs(&plan, &observed, &mut blockers);
    if !blockers.is_empty() {
        return freshness_result(blockers);
    }
    compare_observed_inputs(&plan, &observed, &mut blockers);
    freshness_result(blockers)
}

fn validate_generated_plan(plan: &GeneratedInputPlan, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if plan.schema != GENERATED_INPUT_PLAN_SCHEMA {
        blockers.push(blocker(
            "invalid-generated-plan-schema",
            "generated-inputs",
            "generated input plan schema is unsupported",
        ));
    }
    if plan.inputs.is_empty() || is_count_above_bound(plan.inputs.len(), MAX_GENERATED_INPUTS) {
        blockers.push(blocker(
            "generated-plan-input-limit",
            "generated-inputs",
            "generated input plan must be non-empty and within its fixed bound",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    validate_receipt_set(&plan.inputs, blockers);
    if plan.inputs.windows(crate::ADJACENT_WINDOW_LENGTH).any(|pair| pair[0].target >= pair[1].target) {
        blockers.push(blocker(
            "non-canonical-generated-input-order",
            "generated-inputs",
            "generated input receipts must be strictly ordered by target",
        ));
    }
    validate_plan_identity(plan, blockers);
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_receipt_set(receipts: &[GeneratedInputReceipt], blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    let mut names = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut source_identities = BTreeSet::new();
    let mut dependency_identities = BTreeSet::new();
    for receipt in receipts {
        validate_receipt(receipt, blockers);
        if !names.insert(receipt.name.clone()) {
            blockers.push(blocker(
                "duplicate-generated-input-name",
                &receipt.name,
                "generated input name is duplicated",
            ));
        }
        if !targets.insert(receipt.target.clone()) {
            blockers.push(blocker(
                "duplicate-generated-input-target",
                &receipt.target,
                "generated input target is duplicated",
            ));
        }
        source_identities.insert(receipt.owner.source_identity_blake3.clone());
        dependency_identities.insert(receipt.owner.dependency_identity_blake3.clone());
    }
    if source_identities.len() > 1 || dependency_identities.len() > 1 {
        blockers.push(blocker(
            "mixed-generated-input-ownership",
            "generated-inputs",
            "one generated input plan cannot mix source or dependency identities",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_receipt(receipt: &GeneratedInputReceipt, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    validate_owned_input_fields(
        OwnedInputFields {
            name: &receipt.name,
            target: &receipt.target,
            owner: &receipt.owner,
            content_byte_count: receipt.content.len(),
        },
        blockers,
    );
    if is_count_above_bound(receipt.content.len(), MAX_GENERATED_CONTENT_BYTES) {
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    let content_identity = Blake3Identity::from_slice(receipt.content.as_bytes());
    if content_identity != receipt.content_identity_blake3 {
        blockers.push(blocker(
            "stale-generated-receipt-content",
            &receipt.target,
            "generated input receipt content identity does not match its bytes",
        ));
    }
    let identity_input = ReceiptIdentityInput {
        name: receipt.name.clone(),
        target: receipt.target.clone(),
        output_class: receipt.output_class,
        owner: receipt.owner.clone(),
        content_identity_blake3: receipt.content_identity_blake3.clone(),
    };
    if canonical_identity(identity_input).ok().as_ref() != Some(&receipt.receipt_identity_blake3) {
        blockers.push(blocker(
            "stale-generated-receipt-identity",
            &receipt.target,
            "generated input receipt identity does not match its canonical fields",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_plan_identity(plan: &GeneratedInputPlan, blockers: &mut Vec<ComponentBlocker>) {
    let identity_input = PlanIdentityInput {
        schema: plan.schema.clone(),
        inputs: plan.inputs.clone(),
    };
    if canonical_identity(identity_input).ok().as_ref() != Some(&plan.plan_identity_blake3) {
        blockers.push(blocker(
            "stale-generated-plan-identity",
            "generated-inputs",
            "generated input plan identity does not match its canonical receipts",
        ));
    }
}

fn validate_observed_inputs(
    plan: &GeneratedInputPlan,
    observed: &[ObservedGeneratedInput],
    blockers: &mut Vec<ComponentBlocker>,
) {
    let blocker_count_before = blockers.len();
    if is_count_above_bound(observed.len(), MAX_GENERATED_INPUTS) {
        blockers.push(blocker(
            "observed-generated-input-limit",
            "generated-inputs",
            "observed generated input set exceeds its fixed bound",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    if observed.len() != plan.inputs.len() {
        blockers.push(blocker(
            "observed-generated-input-count",
            "generated-inputs",
            "observed generated input set does not exactly match the owned plan",
        ));
    }
    let mut targets = BTreeSet::new();
    for fact in observed {
        validate_observed_input(plan, fact, &mut targets, blockers);
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_observed_input(
    plan: &GeneratedInputPlan,
    fact: &ObservedGeneratedInput,
    targets: &mut BTreeSet<String>,
    blockers: &mut Vec<ComponentBlocker>,
) {
    let blocker_count_before = blockers.len();
    if !valid_relative_path(&fact.target) {
        blockers.push(blocker("invalid-observed-input-target", &fact.target, "observed input target is not safe"));
    }
    if !targets.insert(fact.target.clone()) {
        blockers.push(blocker("duplicate-observed-input", &fact.target, "observed input target is duplicated"));
    }
    if !plan.inputs.iter().any(|receipt| receipt.target == fact.target) {
        blockers.push(blocker(
            "unexpected-generated-input",
            &fact.target,
            "observed generated input is not owned by the plan",
        ));
    }
    match &fact.content {
        None => blockers.push(blocker(
            "unreadable-generated-input",
            &fact.target,
            "owned generated input could not be read",
        )),
        Some(content) if is_count_above_bound(content.len(), MAX_GENERATED_CONTENT_BYTES) => blockers.push(blocker(
            "observed-generated-input-content-limit",
            &fact.target,
            "observed generated input content exceeds its fixed byte bound",
        )),
        Some(_) => {}
    }
    if fact.receipt_identity_blake3.is_none() {
        blockers.push(blocker(
            "missing-generated-input-receipt",
            &fact.target,
            "observed generated input is missing its owner receipt identity",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn compare_observed_inputs(
    plan: &GeneratedInputPlan,
    observed: &[ObservedGeneratedInput],
    blockers: &mut Vec<ComponentBlocker>,
) {
    for receipt in &plan.inputs {
        let Some(fact) = observed.iter().find(|fact| fact.target == receipt.target) else {
            blockers.push(blocker("missing-generated-input", &receipt.target, "owned generated input is missing"));
            continue;
        };
        let Some(content) = &fact.content else {
            continue;
        };
        if Blake3Identity::from_slice(content) != receipt.content_identity_blake3 {
            blockers.push(blocker(
                "stale-generated-input",
                &receipt.target,
                "generated input bytes differ from the source/export receipt",
            ));
        }
        if fact.receipt_identity_blake3.as_ref() != Some(&receipt.receipt_identity_blake3) {
            blockers.push(blocker(
                "stale-generated-input-receipt",
                &receipt.target,
                "generated input owner receipt differs from the current source/dependency identity",
            ));
        }
    }
}

fn freshness_result(blockers: Vec<ComponentBlocker>) -> GeneratedFreshnessValidation {
    let is_fresh = blockers.is_empty();
    debug_assert_eq!(is_fresh, blockers.is_empty());
    debug_assert!(is_fresh || !blockers.is_empty());
    GeneratedFreshnessValidation {
        fresh: is_fresh,
        blockers,
    }
}

fn validate_candidates(candidates: &[GeneratedInputCandidate], blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if candidates.is_empty() || is_count_above_bound(candidates.len(), MAX_GENERATED_INPUTS) {
        blockers.push(blocker(
            "generated-input-limit",
            "generated-inputs",
            "generated input set must be non-empty and within its fixed bound",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    let mut names = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut source_identities = BTreeSet::new();
    let mut dependency_identities = BTreeSet::new();
    for candidate in candidates {
        validate_candidate(candidate, blockers);
        if !names.insert(candidate.name.clone()) {
            blockers.push(blocker(
                "duplicate-generated-input-name",
                &candidate.name,
                "generated input name is duplicated",
            ));
        }
        if !targets.insert(candidate.target.clone()) {
            blockers.push(blocker(
                "duplicate-generated-input-target",
                &candidate.target,
                "generated input target is duplicated",
            ));
        }
        source_identities.insert(candidate.owner.source_identity_blake3.clone());
        dependency_identities.insert(candidate.owner.dependency_identity_blake3.clone());
    }
    if source_identities.len() > 1 || dependency_identities.len() > 1 {
        blockers.push(blocker(
            "mixed-generated-input-ownership",
            "generated-inputs",
            "one generated input plan cannot mix source or dependency identities",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_candidate(candidate: &GeneratedInputCandidate, blockers: &mut Vec<ComponentBlocker>) {
    validate_owned_input_fields(
        OwnedInputFields {
            name: &candidate.name,
            target: &candidate.target,
            owner: &candidate.owner,
            content_byte_count: candidate.content.len(),
        },
        blockers,
    );
}

fn validate_owned_input_fields(fields: OwnedInputFields<'_>, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if fields.name.is_empty() || fields.owner.export_name != fields.name {
        blockers.push(blocker(
            "invalid-generated-input-name",
            fields.name,
            "generated input name must be non-empty and match its owner receipt",
        ));
    }
    if !valid_relative_path(fields.target) {
        blockers.push(blocker(
            "invalid-generated-input-target",
            fields.target,
            "generated input target must be a safe relative path",
        ));
    }
    if fields.owner.schema != GENERATED_INPUT_RECEIPT_SCHEMA || fields.owner.generator != GENERATED_INPUT_OWNER {
        blockers.push(blocker(
            "invalid-generated-input-owner",
            fields.target,
            "generated input owner schema or generator is unsupported",
        ));
    }
    if is_count_above_bound(fields.content_byte_count, MAX_GENERATED_CONTENT_BYTES) {
        blockers.push(blocker(
            "generated-input-content-limit",
            fields.target,
            "generated input content exceeds its fixed byte bound",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn finalize_candidate(candidate: GeneratedInputCandidate) -> Result<GeneratedInputReceipt, ComponentBlocker> {
    let content_identity = Blake3Identity::from_bytes(candidate.content.as_bytes().to_vec());
    let identity_input = ReceiptIdentityInput {
        name: candidate.name.clone(),
        target: candidate.target.clone(),
        output_class: candidate.output_class,
        owner: candidate.owner.clone(),
        content_identity_blake3: content_identity.clone(),
    };
    let receipt_identity = canonical_identity(identity_input).map_err(|_| {
        blocker(
            "generated-receipt-identity-failed",
            &candidate.target,
            "generated input receipt could not be canonically identified",
        )
    })?;
    debug_assert_eq!(candidate.owner.export_name, candidate.name);
    debug_assert!(is_count_within_bound(candidate.content.len(), MAX_GENERATED_CONTENT_BYTES));
    Ok(GeneratedInputReceipt {
        name: candidate.name,
        target: candidate.target,
        output_class: candidate.output_class,
        owner: candidate.owner,
        content: candidate.content,
        content_identity_blake3: content_identity,
        receipt_identity_blake3: receipt_identity,
    })
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
}
