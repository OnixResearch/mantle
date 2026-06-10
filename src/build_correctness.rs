use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

const ACTION_SPEC_SCHEMA: &str = "mantle-action-spec-v1";
const ACTION_REF_PREFIX: &str = "mantle-action://blake3/";
const OBJECT_MANIFEST_SCHEMA: &str = "mantle-object-manifest-v1";
const OBJECT_REF_PREFIX: &str = "mantle-object://blake3/";
const NICKEL_EVAL_RECEIPT_SCHEMA: &str = "mantle-nickel-eval-receipt-v1";
const NICKEL_EVAL_REF_PREFIX: &str = "mantle-nickel-eval://blake3/";
const REFERENCE_SCAN_SCHEMA: &str = "mantle-reference-scan-v1";
const REFERENCE_SCAN_REF_PREFIX: &str = "mantle-reference-scan://blake3/";
const SANDBOX_REPORT_SCHEMA: &str = "mantle-sandbox-report-v1";
const SANDBOX_REPORT_REF_PREFIX: &str = "mantle-sandbox-report://blake3/";
const ACTION_RECEIPT_SCHEMA: &str = "mantle-action-receipt-v1";
const ACTION_RECEIPT_REF_PREFIX: &str = "mantle-action-receipt://blake3/";
const BLAKE3_HEX_CHARS: usize = 64;
const REF_PREFIX_SEPARATOR: &str = "://blake3/";
const STRONG_CLAIM: &str = "produced objects match the declared action and receipt policy";
const NON_CLAIM_COMPILER_CORRECTNESS: &str = "compiler-correctness";
const NON_CLAIM_SOURCE_REPRODUCIBILITY: &str = "source-to-binary-reproducibility";
const NON_CLAIM_FRONTEND_MODULE_CORRECTNESS: &str = "frontend-module-correctness";
const NON_CLAIM_DEPLOY_SUCCESS: &str = "deploy-success";
const NON_CLAIM_PHYSICAL_TARGET_DETERMINISM: &str = "physical-target-determinism";
const SECRET_DESCRIPTOR_KIND: &str = "redacted-secret-descriptor";
const FILE_OBJECT_KIND: &str = "file";
const DIRECTORY_OBJECT_KIND: &str = "directory";
const SYMLINK_OBJECT_KIND: &str = "symlink";
const GENERATED_PAYLOAD_OBJECT_KIND: &str = "generated-payload";
const PATH_TRAVERSAL_SEGMENT: &str = "..";
const PLAINTEXT_SECRET_MARKER: &str = "plaintext-secret:";
const ENFORCED_STATUS: &str = "enforced";
const UNSUPPORTED_STATUS: &str = "unsupported";
const NETWORK_RESULT_DENIED: &str = "denied";
const NETWORK_POLICY_OFFLINE: &str = "offline";
const EMPTY_DIGEST_INPUT: &str = "";
const ZERO_COUNT: u32 = 0;
const MAX_DECLARED_REFS: usize = 4096;
const MAX_REFERENCE_OBSERVATIONS: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OutputDeclaration {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) object_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SandboxPolicy {
    pub(crate) mode: String,
    pub(crate) writable_paths: Vec<String>,
    pub(crate) env_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NetworkPolicy {
    pub(crate) mode: String,
    pub(crate) allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ExpectedReferencePolicy {
    pub(crate) allowed_refs: Vec<String>,
    pub(crate) forbidden_refs: Vec<String>,
    pub(crate) allow_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleActionSpecInput {
    pub(crate) action_kind: String,
    pub(crate) platform: String,
    pub(crate) toolchain_refs: Vec<String>,
    pub(crate) input_object_refs: Vec<String>,
    pub(crate) args_digest_blake3: String,
    pub(crate) env_digest_blake3: String,
    pub(crate) output_declarations: Vec<OutputDeclaration>,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) network_policy: NetworkPolicy,
    pub(crate) expected_reference_policy: ExpectedReferencePolicy,
    pub(crate) frontend_spec_refs: Vec<String>,
    pub(crate) nickel_eval_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleActionSpec {
    pub(crate) schema: String,
    pub(crate) action_ref: String,
    pub(crate) action_kind: String,
    pub(crate) platform: String,
    pub(crate) toolchain_refs: Vec<String>,
    pub(crate) input_object_refs: Vec<String>,
    pub(crate) args_digest_blake3: String,
    pub(crate) env_digest_blake3: String,
    pub(crate) output_declarations: Vec<OutputDeclaration>,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) network_policy: NetworkPolicy,
    pub(crate) expected_reference_policy: ExpectedReferencePolicy,
    pub(crate) frontend_spec_refs: Vec<String>,
    pub(crate) nickel_eval_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NickelEvalReceiptInput {
    pub(crate) root_src_ref: String,
    pub(crate) transitive_dep_refs: Vec<String>,
    pub(crate) import_path_policy: Vec<String>,
    pub(crate) evaluator_ref: String,
    pub(crate) export_format: String,
    pub(crate) output_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NickelEvalReceipt {
    pub(crate) schema: String,
    pub(crate) eval_ref: String,
    pub(crate) root_src_ref: String,
    pub(crate) transitive_dep_refs: Vec<String>,
    pub(crate) import_path_policy: Vec<String>,
    pub(crate) evaluator_ref: String,
    pub(crate) export_format: String,
    pub(crate) output_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DirectoryChild {
    pub(crate) name: String,
    pub(crate) object_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RedactedSecretDescriptor {
    pub(crate) descriptor_ref: String,
    pub(crate) purpose: String,
    pub(crate) redaction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CasObjectManifestInput {
    pub(crate) kind: String,
    pub(crate) byte_count: Option<u64>,
    pub(crate) content_digest_blake3: Option<String>,
    pub(crate) executable: Option<bool>,
    pub(crate) mode: Option<String>,
    pub(crate) symlink_target: Option<String>,
    pub(crate) children: Vec<DirectoryChild>,
    pub(crate) secret_descriptor: Option<RedactedSecretDescriptor>,
    pub(crate) path_views: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CasObjectManifest {
    pub(crate) schema: String,
    pub(crate) object_ref: String,
    pub(crate) kind: String,
    pub(crate) byte_count: Option<u64>,
    pub(crate) content_digest_blake3: Option<String>,
    pub(crate) executable: Option<bool>,
    pub(crate) mode: Option<String>,
    pub(crate) symlink_target: Option<String>,
    pub(crate) children: Vec<DirectoryChild>,
    pub(crate) secret_descriptor: Option<RedactedSecretDescriptor>,
    pub(crate) path_views: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ReferenceObservation {
    pub(crate) ref_value: String,
    pub(crate) view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReferenceScanInput {
    pub(crate) output_object_ref: String,
    pub(crate) scan_root_ref: String,
    pub(crate) scanner_kind: String,
    pub(crate) declared_refs: Vec<String>,
    pub(crate) forbidden_refs: Vec<String>,
    pub(crate) observations: Vec<ReferenceObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OutputReferenceScanReport {
    pub(crate) schema: String,
    pub(crate) scan_ref: String,
    pub(crate) output_object_ref: String,
    pub(crate) scan_root_ref: String,
    pub(crate) scanner_kind: String,
    pub(crate) status: String,
    pub(crate) accepted_refs: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ExecutorCapabilities {
    pub(crate) sandbox_modes: Vec<String>,
    pub(crate) network_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxReport {
    pub(crate) schema: String,
    pub(crate) sandbox_report_ref: String,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) network_policy: NetworkPolicy,
    pub(crate) enforcement_status: String,
    pub(crate) diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReuseBasis {
    pub(crate) prior_receipt_ref: String,
    pub(crate) matched_signature_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleActionReceiptInput {
    pub(crate) action_ref: String,
    pub(crate) nickel_eval_ref: Option<String>,
    pub(crate) input_object_refs: Vec<String>,
    pub(crate) toolchain_refs: Vec<String>,
    pub(crate) produced_object_refs: Vec<String>,
    pub(crate) reference_scan_ref: String,
    pub(crate) sandbox_report_ref: String,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) network_policy: NetworkPolicy,
    pub(crate) network_policy_result: String,
    pub(crate) producer_identity: String,
    pub(crate) signature_refs: Vec<String>,
    pub(crate) execution_status: String,
    pub(crate) build_or_reuse_reason: String,
    pub(crate) reuse_basis: Option<ReuseBasis>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleActionReceipt {
    pub(crate) schema: String,
    pub(crate) receipt_ref: String,
    pub(crate) action_ref: String,
    pub(crate) nickel_eval_ref: Option<String>,
    pub(crate) input_object_refs: Vec<String>,
    pub(crate) toolchain_refs: Vec<String>,
    pub(crate) produced_object_refs: Vec<String>,
    pub(crate) reference_scan_ref: String,
    pub(crate) sandbox_report_ref: String,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) network_policy: NetworkPolicy,
    pub(crate) network_policy_result: String,
    pub(crate) producer_identity: String,
    pub(crate) signature_refs: Vec<String>,
    pub(crate) execution_status: String,
    pub(crate) build_or_reuse_reason: String,
    pub(crate) reuse_basis: Option<ReuseBasis>,
    pub(crate) claim: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReuseAdmissionRequest {
    pub(crate) requested_action_ref: String,
    pub(crate) requested_output_refs: Vec<String>,
    pub(crate) requested_sandbox_policy: SandboxPolicy,
    pub(crate) requested_network_policy: NetworkPolicy,
    pub(crate) required_signatures: bool,
    pub(crate) trusted_producers: Vec<String>,
    pub(crate) candidate: MantleActionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReuseAdmissionReport {
    pub(crate) admitted: bool,
    pub(crate) diagnostics: Vec<String>,
}

pub(crate) fn canonical_action_spec(input: MantleActionSpecInput) -> Result<MantleActionSpec, String> {
    validate_required("action_kind", &input.action_kind)?;
    validate_required("platform", &input.platform)?;
    validate_ref_list("toolchain_refs", &input.toolchain_refs)?;
    validate_ref_list("input_object_refs", &input.input_object_refs)?;
    validate_digest("args_digest_blake3", &input.args_digest_blake3)?;
    validate_digest("env_digest_blake3", &input.env_digest_blake3)?;
    let output_declarations = sorted_unique_structs(input.output_declarations.clone());
    let hashable = action_hashable(&input, &output_declarations);
    let action_ref = prefixed_digest(ACTION_REF_PREFIX, &hashable)?;
    Ok(MantleActionSpec {
        schema: ACTION_SPEC_SCHEMA.to_string(),
        action_ref,
        action_kind: input.action_kind,
        platform: input.platform,
        toolchain_refs: sorted_unique_strings(input.toolchain_refs),
        input_object_refs: sorted_unique_strings(input.input_object_refs),
        args_digest_blake3: input.args_digest_blake3,
        env_digest_blake3: input.env_digest_blake3,
        output_declarations,
        sandbox_policy: normalize_sandbox_policy(input.sandbox_policy),
        network_policy: normalize_network_policy(input.network_policy),
        expected_reference_policy: normalize_reference_policy(input.expected_reference_policy),
        frontend_spec_refs: sorted_unique_strings(input.frontend_spec_refs),
        nickel_eval_ref: input.nickel_eval_ref,
    })
}

fn action_hashable(input: &MantleActionSpecInput, output_declarations: &[OutputDeclaration]) -> serde_json::Value {
    serde_json::json!({
        "schema": ACTION_SPEC_SCHEMA,
        "action_kind": input.action_kind,
        "platform": input.platform,
        "toolchain_refs": sorted_unique_strings(input.toolchain_refs.clone()),
        "input_object_refs": sorted_unique_strings(input.input_object_refs.clone()),
        "args_digest_blake3": input.args_digest_blake3,
        "env_digest_blake3": input.env_digest_blake3,
        "output_declarations": output_declarations,
        "sandbox_policy": normalize_sandbox_policy(input.sandbox_policy.clone()),
        "network_policy": normalize_network_policy(input.network_policy.clone()),
        "expected_reference_policy": normalize_reference_policy(input.expected_reference_policy.clone()),
        "frontend_spec_refs": sorted_unique_strings(input.frontend_spec_refs.clone()),
        "nickel_eval_ref": input.nickel_eval_ref,
    })
}

pub(crate) fn canonical_nickel_eval_receipt(input: NickelEvalReceiptInput) -> Result<NickelEvalReceipt, String> {
    validate_ref("root_src_ref", &input.root_src_ref)?;
    validate_ref_list("transitive_dep_refs", &input.transitive_dep_refs)?;
    validate_ref("evaluator_ref", &input.evaluator_ref)?;
    validate_required("export_format", &input.export_format)?;
    validate_digest("output_digest_blake3", &input.output_digest_blake3)?;
    let hashable = serde_json::json!({
        "schema": NICKEL_EVAL_RECEIPT_SCHEMA,
        "root_src_ref": input.root_src_ref,
        "transitive_dep_refs": sorted_unique_strings(input.transitive_dep_refs.clone()),
        "import_path_policy": sorted_unique_strings(input.import_path_policy.clone()),
        "evaluator_ref": input.evaluator_ref,
        "export_format": input.export_format,
        "output_digest_blake3": input.output_digest_blake3,
    });
    let eval_ref = prefixed_digest(NICKEL_EVAL_REF_PREFIX, &hashable)?;
    Ok(NickelEvalReceipt {
        schema: NICKEL_EVAL_RECEIPT_SCHEMA.to_string(),
        eval_ref,
        root_src_ref: input.root_src_ref,
        transitive_dep_refs: sorted_unique_strings(input.transitive_dep_refs),
        import_path_policy: sorted_unique_strings(input.import_path_policy),
        evaluator_ref: input.evaluator_ref,
        export_format: input.export_format,
        output_digest_blake3: input.output_digest_blake3,
    })
}

pub(crate) fn validate_nickel_imports(declared_refs: &[String], observed_refs: &[String]) -> Result<(), Vec<String>> {
    let declared = declared_refs.iter().collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    for observed in observed_refs {
        if !declared.contains(observed) {
            diagnostics.push(format!("undeclared-import:{observed}"));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn admit_cas_object(input: CasObjectManifestInput) -> Result<CasObjectManifest, String> {
    validate_object_identity(&input)?;
    validate_path_views(&input.path_views)?;
    let children = sorted_unique_structs(input.children.clone());
    let hashable = object_hashable(&input, &children);
    let object_ref = prefixed_digest(OBJECT_REF_PREFIX, &hashable)?;
    Ok(CasObjectManifest {
        schema: OBJECT_MANIFEST_SCHEMA.to_string(),
        object_ref,
        kind: input.kind,
        byte_count: input.byte_count,
        content_digest_blake3: input.content_digest_blake3,
        executable: input.executable,
        mode: input.mode,
        symlink_target: input.symlink_target,
        children,
        secret_descriptor: input.secret_descriptor,
        path_views: sorted_unique_strings(input.path_views),
    })
}

fn object_hashable(input: &CasObjectManifestInput, children: &[DirectoryChild]) -> serde_json::Value {
    serde_json::json!({
        "schema": OBJECT_MANIFEST_SCHEMA,
        "kind": input.kind,
        "byte_count": input.byte_count,
        "content_digest_blake3": input.content_digest_blake3,
        "executable": input.executable,
        "mode": input.mode,
        "symlink_target": input.symlink_target,
        "children": children,
        "secret_descriptor": input.secret_descriptor,
    })
}

fn validate_object_identity(input: &CasObjectManifestInput) -> Result<(), String> {
    validate_required("kind", &input.kind)?;
    match input.kind.as_str() {
        FILE_OBJECT_KIND | GENERATED_PAYLOAD_OBJECT_KIND => {
            let digest = input.content_digest_blake3.as_deref().ok_or("path-only identity is not accepted")?;
            validate_digest("content_digest_blake3", digest)?;
            if input.byte_count.is_none() {
                return Err("byte_count is required for byte objects".to_string());
            }
            Ok(())
        }
        DIRECTORY_OBJECT_KIND => {
            if input.children.is_empty() {
                return Err("directory object requires at least one child object ref".to_string());
            }
            validate_ref_list(
                "directory_children",
                &input.children.iter().map(|child| child.object_ref.clone()).collect(),
            )
        }
        SYMLINK_OBJECT_KIND => {
            validate_required("symlink_target", input.symlink_target.as_deref().unwrap_or(EMPTY_DIGEST_INPUT))
        }
        SECRET_DESCRIPTOR_KIND => input
            .secret_descriptor
            .as_ref()
            .map(validate_secret_descriptor)
            .unwrap_or_else(|| Err("redacted secret object requires descriptor metadata".to_string())),
        _ => Err(format!("unsupported object kind `{}`", input.kind)),
    }
}

fn validate_secret_descriptor(descriptor: &RedactedSecretDescriptor) -> Result<(), String> {
    validate_ref("secret_descriptor.descriptor_ref", &descriptor.descriptor_ref)?;
    validate_required("secret_descriptor.purpose", &descriptor.purpose)?;
    validate_required("secret_descriptor.redaction", &descriptor.redaction)
}

pub(crate) fn validate_hermetic_policy(
    sandbox_policy: SandboxPolicy,
    network_policy: NetworkPolicy,
    capabilities: &ExecutorCapabilities,
) -> Result<SandboxReport, Vec<String>> {
    let sandbox_policy = normalize_sandbox_policy(sandbox_policy);
    let network_policy = normalize_network_policy(network_policy);
    let mut diagnostics = Vec::new();
    if !capabilities.sandbox_modes.contains(&sandbox_policy.mode) {
        diagnostics.push(format!("unsupported-sandbox-policy:{}", sandbox_policy.mode));
    }
    if !capabilities.network_modes.contains(&network_policy.mode) {
        diagnostics.push(format!("unsupported-network-policy:{}", network_policy.mode));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(sandbox_report(sandbox_policy, network_policy, ENFORCED_STATUS, Vec::new()))
}

fn sandbox_report(
    sandbox_policy: SandboxPolicy,
    network_policy: NetworkPolicy,
    enforcement_status: &str,
    diagnostics: Vec<String>,
) -> SandboxReport {
    let hashable = serde_json::json!({
        "schema": SANDBOX_REPORT_SCHEMA,
        "sandbox_policy": sandbox_policy,
        "network_policy": network_policy,
        "enforcement_status": enforcement_status,
        "diagnostics": sorted_unique_strings(diagnostics.clone()),
    });
    SandboxReport {
        schema: SANDBOX_REPORT_SCHEMA.to_string(),
        sandbox_report_ref: prefixed_digest(SANDBOX_REPORT_REF_PREFIX, &hashable)
            .expect("sandbox report is serializable"),
        sandbox_policy,
        network_policy,
        enforcement_status: enforcement_status.to_string(),
        diagnostics: sorted_unique_strings(diagnostics),
    }
}

pub(crate) fn validate_reference_scan(
    input: ReferenceScanInput,
) -> Result<OutputReferenceScanReport, OutputReferenceScanReport> {
    let diagnostics = reference_scan_diagnostics(&input);
    let accepted_refs = accepted_reference_values(&input.observations);
    let status = if diagnostics.is_empty() { "accepted" } else { "rejected" };
    let report = reference_scan_report(input, status, accepted_refs, diagnostics);
    if report.diagnostics.is_empty() {
        Ok(report)
    } else {
        Err(report)
    }
}

fn reference_scan_diagnostics(input: &ReferenceScanInput) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if input.declared_refs.len() > MAX_DECLARED_REFS {
        diagnostics.push("too-many-declared-refs".to_string());
    }
    if input.observations.len() > MAX_REFERENCE_OBSERVATIONS {
        diagnostics.push("too-many-reference-observations".to_string());
    }
    let declared = input.declared_refs.iter().collect::<BTreeSet<_>>();
    let forbidden = input.forbidden_refs.iter().collect::<BTreeSet<_>>();
    let mut view_to_ref = BTreeMap::new();
    for observation in &input.observations {
        scan_one_reference(observation, &declared, &forbidden, &mut view_to_ref, &mut diagnostics);
    }
    sorted_unique_strings(diagnostics)
}

fn scan_one_reference<'a>(
    observation: &'a ReferenceObservation,
    declared: &BTreeSet<&'a String>,
    forbidden: &BTreeSet<&'a String>,
    view_to_ref: &mut BTreeMap<&'a String, &'a String>,
    diagnostics: &mut Vec<String>,
) {
    if !declared.contains(&observation.ref_value) {
        diagnostics.push(format!("undeclared-ref:{}", observation.ref_value));
    }
    if forbidden.contains(&observation.ref_value) {
        diagnostics.push(format!("forbidden-ref:{}", observation.ref_value));
    }
    if contains_path_traversal(&observation.view) || contains_path_traversal(&observation.ref_value) {
        diagnostics.push(format!("path-traversal:{}", observation.view));
    }
    if observation.ref_value.contains(PLAINTEXT_SECRET_MARKER) || observation.view.contains(PLAINTEXT_SECRET_MARKER) {
        diagnostics.push("plaintext-secret-bytes".to_string());
    }
    if let Some(previous_ref) = view_to_ref.insert(&observation.view, &observation.ref_value) {
        if previous_ref != &observation.ref_value {
            diagnostics.push(format!("duplicate-conflicting-view:{}", observation.view));
        }
    }
}

fn reference_scan_report(
    input: ReferenceScanInput,
    status: &str,
    accepted_refs: Vec<String>,
    diagnostics: Vec<String>,
) -> OutputReferenceScanReport {
    let hashable = serde_json::json!({
        "schema": REFERENCE_SCAN_SCHEMA,
        "output_object_ref": input.output_object_ref,
        "scan_root_ref": input.scan_root_ref,
        "scanner_kind": input.scanner_kind,
        "status": status,
        "accepted_refs": sorted_unique_strings(accepted_refs.clone()),
        "diagnostics": sorted_unique_strings(diagnostics.clone()),
    });
    OutputReferenceScanReport {
        schema: REFERENCE_SCAN_SCHEMA.to_string(),
        scan_ref: prefixed_digest(REFERENCE_SCAN_REF_PREFIX, &hashable).expect("reference scan is serializable"),
        output_object_ref: input.output_object_ref,
        scan_root_ref: input.scan_root_ref,
        scanner_kind: input.scanner_kind,
        status: status.to_string(),
        accepted_refs: sorted_unique_strings(accepted_refs),
        diagnostics: sorted_unique_strings(diagnostics),
    }
}

pub(crate) fn action_receipt(input: MantleActionReceiptInput) -> Result<MantleActionReceipt, String> {
    validate_ref("action_ref", &input.action_ref)?;
    validate_ref_list("input_object_refs", &input.input_object_refs)?;
    validate_ref_list("toolchain_refs", &input.toolchain_refs)?;
    validate_ref_list("produced_object_refs", &input.produced_object_refs)?;
    validate_required("producer_identity", &input.producer_identity)?;
    let hashable = action_receipt_hashable(&input);
    let receipt_ref = prefixed_digest(ACTION_RECEIPT_REF_PREFIX, &hashable)?;
    Ok(MantleActionReceipt {
        schema: ACTION_RECEIPT_SCHEMA.to_string(),
        receipt_ref,
        action_ref: input.action_ref,
        nickel_eval_ref: input.nickel_eval_ref,
        input_object_refs: sorted_unique_strings(input.input_object_refs),
        toolchain_refs: sorted_unique_strings(input.toolchain_refs),
        produced_object_refs: sorted_unique_strings(input.produced_object_refs),
        reference_scan_ref: input.reference_scan_ref,
        sandbox_report_ref: input.sandbox_report_ref,
        sandbox_policy: normalize_sandbox_policy(input.sandbox_policy),
        network_policy: normalize_network_policy(input.network_policy),
        network_policy_result: input.network_policy_result,
        producer_identity: input.producer_identity,
        signature_refs: sorted_unique_strings(input.signature_refs),
        execution_status: input.execution_status,
        build_or_reuse_reason: input.build_or_reuse_reason,
        reuse_basis: input.reuse_basis,
        claim: STRONG_CLAIM.to_string(),
        non_claims: receipt_non_claims(),
    })
}

fn action_receipt_hashable(input: &MantleActionReceiptInput) -> serde_json::Value {
    serde_json::json!({
        "schema": ACTION_RECEIPT_SCHEMA,
        "action_ref": input.action_ref,
        "nickel_eval_ref": input.nickel_eval_ref,
        "input_object_refs": sorted_unique_strings(input.input_object_refs.clone()),
        "toolchain_refs": sorted_unique_strings(input.toolchain_refs.clone()),
        "produced_object_refs": sorted_unique_strings(input.produced_object_refs.clone()),
        "reference_scan_ref": input.reference_scan_ref,
        "sandbox_report_ref": input.sandbox_report_ref,
        "sandbox_policy": normalize_sandbox_policy(input.sandbox_policy.clone()),
        "network_policy": normalize_network_policy(input.network_policy.clone()),
        "network_policy_result": input.network_policy_result,
        "producer_identity": input.producer_identity,
        "signature_refs": sorted_unique_strings(input.signature_refs.clone()),
        "execution_status": input.execution_status,
        "build_or_reuse_reason": input.build_or_reuse_reason,
        "reuse_basis": input.reuse_basis,
    })
}

pub(crate) fn admit_reuse(request: ReuseAdmissionRequest) -> ReuseAdmissionReport {
    let mut diagnostics = Vec::new();
    if request.candidate.action_ref != request.requested_action_ref {
        diagnostics.push("stale-action-ref".to_string());
    }
    if sorted_unique_strings(request.candidate.produced_object_refs.clone())
        != sorted_unique_strings(request.requested_output_refs)
    {
        diagnostics.push("stale-object-ref".to_string());
    }
    if request.candidate.sandbox_policy != normalize_sandbox_policy(request.requested_sandbox_policy) {
        diagnostics.push("sandbox-policy-mismatch".to_string());
    }
    if request.candidate.network_policy != normalize_network_policy(request.requested_network_policy) {
        diagnostics.push("network-policy-mismatch".to_string());
    }
    if request.required_signatures && request.candidate.signature_refs.is_empty() {
        diagnostics.push("missing-required-signature".to_string());
    }
    if !request.trusted_producers.is_empty()
        && !request.trusted_producers.contains(&request.candidate.producer_identity)
    {
        diagnostics.push("unsupported-producer-identity".to_string());
    }
    ReuseAdmissionReport {
        admitted: diagnostics.is_empty(),
        diagnostics: sorted_unique_strings(diagnostics),
    }
}

pub(crate) fn receipt_json(receipt: &MantleActionReceipt) -> Result<String, String> {
    serde_json::to_string_pretty(receipt).map_err(|error| format!("render action receipt JSON: {error}"))
}

fn receipt_non_claims() -> Vec<String> {
    vec![
        NON_CLAIM_COMPILER_CORRECTNESS.to_string(),
        NON_CLAIM_SOURCE_REPRODUCIBILITY.to_string(),
        NON_CLAIM_FRONTEND_MODULE_CORRECTNESS.to_string(),
        NON_CLAIM_DEPLOY_SUCCESS.to_string(),
        NON_CLAIM_PHYSICAL_TARGET_DETERMINISM.to_string(),
    ]
}

fn normalize_sandbox_policy(mut policy: SandboxPolicy) -> SandboxPolicy {
    policy.writable_paths = sorted_unique_strings(policy.writable_paths);
    policy
}

fn normalize_network_policy(mut policy: NetworkPolicy) -> NetworkPolicy {
    policy.allowed_hosts = sorted_unique_strings(policy.allowed_hosts);
    policy
}

fn normalize_reference_policy(mut policy: ExpectedReferencePolicy) -> ExpectedReferencePolicy {
    policy.allowed_refs = sorted_unique_strings(policy.allowed_refs);
    policy.forbidden_refs = sorted_unique_strings(policy.forbidden_refs);
    policy
}

fn sorted_unique_strings(values: Vec<String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn sorted_unique_structs<T>(values: Vec<T>) -> Vec<T>
where T: Ord {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn accepted_reference_values(observations: &[ReferenceObservation]) -> Vec<String> {
    sorted_unique_strings(observations.iter().map(|observation| observation.ref_value.clone()).collect())
}

fn validate_required(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

fn validate_ref_list(field: &str, refs: &Vec<String>) -> Result<(), String> {
    if refs.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    for item in refs {
        validate_ref(field, item)?;
    }
    Ok(())
}

fn validate_ref(field: &str, value: &str) -> Result<(), String> {
    validate_required(field, value)?;
    if !value.contains(REF_PREFIX_SEPARATOR) {
        return Err(format!("{field} must be a content-addressed ref, got `{value}`"));
    }
    Ok(())
}

fn validate_digest(field: &str, value: &str) -> Result<(), String> {
    if value.len() != BLAKE3_HEX_CHARS {
        return Err(format!("{field} must be {BLAKE3_HEX_CHARS} lowercase hex chars"));
    }
    if !value.chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase()) {
        return Err(format!("{field} must be lowercase hex"));
    }
    Ok(())
}

fn validate_path_views(path_views: &[String]) -> Result<(), String> {
    for view in path_views {
        if contains_path_traversal(view) {
            return Err(format!("path view contains traversal: {view}"));
        }
    }
    Ok(())
}

fn contains_path_traversal(value: &str) -> bool {
    value.split('/').any(|segment| segment == PATH_TRAVERSAL_SEGMENT)
}

fn prefixed_digest(prefix: &str, value: &serde_json::Value) -> Result<String, String> {
    let canonical = serde_json::to_vec(value).map_err(|error| format!("canonicalize value: {error}"))?;
    Ok(format!("{prefix}{}", blake3::hash(&canonical).to_hex()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const PRODUCER_LOCAL: &str = "local-mantle-test-producer";

    fn object_ref(seed: &str) -> String {
        format!("{OBJECT_REF_PREFIX}{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn action_ref(seed: &str) -> String {
        format!("{ACTION_REF_PREFIX}{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn scan_ref(seed: &str) -> String {
        format!("{REFERENCE_SCAN_REF_PREFIX}{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn sandbox_ref(seed: &str) -> String {
        format!("{SANDBOX_REPORT_REF_PREFIX}{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn sandbox_policy() -> SandboxPolicy {
        SandboxPolicy {
            mode: "bwrap-readonly-inputs".to_string(),
            writable_paths: vec!["/build/out".to_string()],
            env_policy: "declared-only".to_string(),
        }
    }

    fn network_policy() -> NetworkPolicy {
        NetworkPolicy {
            mode: NETWORK_POLICY_OFFLINE.to_string(),
            allowed_hosts: Vec::new(),
        }
    }

    fn reference_policy() -> ExpectedReferencePolicy {
        ExpectedReferencePolicy {
            allowed_refs: vec![object_ref("runtime")],
            forbidden_refs: vec!["/tmp".to_string()],
            allow_generated: true,
        }
    }

    fn action_input() -> MantleActionSpecInput {
        MantleActionSpecInput {
            action_kind: "compile-rust".to_string(),
            platform: "x86_64-linux".to_string(),
            toolchain_refs: vec![object_ref("toolchain-b"), object_ref("toolchain-a")],
            input_object_refs: vec![object_ref("input-b"), object_ref("input-a")],
            args_digest_blake3: DIGEST_A.to_string(),
            env_digest_blake3: DIGEST_B.to_string(),
            output_declarations: vec![OutputDeclaration {
                name: "out".to_string(),
                kind: FILE_OBJECT_KIND.to_string(),
                object_ref: None,
            }],
            sandbox_policy: sandbox_policy(),
            network_policy: network_policy(),
            expected_reference_policy: reference_policy(),
            frontend_spec_refs: vec!["onix://opaque/module-role/system".to_string()],
            nickel_eval_ref: None,
        }
    }

    fn file_manifest_input(path_views: Vec<String>) -> CasObjectManifestInput {
        CasObjectManifestInput {
            kind: FILE_OBJECT_KIND.to_string(),
            byte_count: Some(3),
            content_digest_blake3: Some(DIGEST_A.to_string()),
            executable: Some(false),
            mode: Some("0444".to_string()),
            symlink_target: None,
            children: Vec::new(),
            secret_descriptor: None,
            path_views,
        }
    }

    fn receipt_input(action_ref: String, produced_object_ref: String) -> MantleActionReceiptInput {
        MantleActionReceiptInput {
            action_ref,
            nickel_eval_ref: None,
            input_object_refs: vec![object_ref("input")],
            toolchain_refs: vec![object_ref("toolchain")],
            produced_object_refs: vec![produced_object_ref],
            reference_scan_ref: scan_ref("scan"),
            sandbox_report_ref: sandbox_ref("sandbox"),
            sandbox_policy: sandbox_policy(),
            network_policy: network_policy(),
            network_policy_result: NETWORK_RESULT_DENIED.to_string(),
            producer_identity: PRODUCER_LOCAL.to_string(),
            signature_refs: vec![
                "mantle-signature://blake3/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
            ],
            execution_status: "success".to_string(),
            build_or_reuse_reason: "built".to_string(),
            reuse_basis: None,
        }
    }

    #[test]
    fn build_correctness_action_spec_ref_is_deterministic_for_equivalent_inputs() {
        let left = canonical_action_spec(action_input()).unwrap();
        let mut shuffled = action_input();
        shuffled.toolchain_refs.reverse();
        shuffled.input_object_refs.reverse();
        shuffled.frontend_spec_refs.push(shuffled.frontend_spec_refs[0].clone());
        shuffled.sandbox_policy.writable_paths.push("/build/out".to_string());
        let right = canonical_action_spec(shuffled).unwrap();

        assert_eq!(left.action_ref, right.action_ref);
        assert_eq!(left.toolchain_refs.len(), 2);
        assert!(left.action_ref.starts_with(ACTION_REF_PREFIX));
        assert!(!left.action_ref.contains("/tmp"));
    }

    #[test]
    fn build_correctness_action_spec_ref_changes_when_declared_input_changes() {
        let baseline = canonical_action_spec(action_input()).unwrap();
        let mut changed = action_input();
        changed.input_object_refs.push(object_ref("new-input"));
        let changed = canonical_action_spec(changed).unwrap();

        assert_ne!(baseline.action_ref, changed.action_ref);
        assert!(changed.input_object_refs.iter().any(|item| item == &object_ref("new-input")));
    }

    #[test]
    fn build_correctness_nickel_eval_receipt_binds_source_closure_and_blocks_undeclared_imports() {
        let declared = vec![object_ref("root"), object_ref("dep")];
        let receipt = canonical_nickel_eval_receipt(NickelEvalReceiptInput {
            root_src_ref: declared[0].clone(),
            transitive_dep_refs: vec![declared[1].clone()],
            import_path_policy: vec!["repo-relative".to_string()],
            evaluator_ref: object_ref("nickel-evaluator"),
            export_format: "build-ir-json".to_string(),
            output_digest_blake3: DIGEST_C.to_string(),
        })
        .unwrap();
        let observed = vec![declared[0].clone(), object_ref("undeclared")];
        let diagnostics = validate_nickel_imports(&declared, &observed).unwrap_err();

        assert!(receipt.eval_ref.starts_with(NICKEL_EVAL_REF_PREFIX));
        assert_eq!(receipt.transitive_dep_refs, vec![declared[1].clone()]);
        assert_eq!(diagnostics, vec![format!("undeclared-import:{}", object_ref("undeclared"))]);
    }

    #[test]
    fn build_correctness_cas_admits_file_directory_symlink_generated_and_secret_objects() {
        let file = admit_cas_object(file_manifest_input(vec!["/export/out".to_string()])).unwrap();
        let directory = admit_cas_object(CasObjectManifestInput {
            kind: DIRECTORY_OBJECT_KIND.to_string(),
            byte_count: None,
            content_digest_blake3: None,
            executable: None,
            mode: None,
            symlink_target: None,
            children: vec![DirectoryChild {
                name: "file".to_string(),
                object_ref: file.object_ref.clone(),
            }],
            secret_descriptor: None,
            path_views: vec!["/export/dir".to_string()],
        })
        .unwrap();
        let symlink = admit_cas_object(CasObjectManifestInput {
            kind: SYMLINK_OBJECT_KIND.to_string(),
            byte_count: None,
            content_digest_blake3: None,
            executable: None,
            mode: None,
            symlink_target: Some("file".to_string()),
            children: Vec::new(),
            secret_descriptor: None,
            path_views: Vec::new(),
        })
        .unwrap();
        let generated = admit_cas_object(CasObjectManifestInput {
            kind: GENERATED_PAYLOAD_OBJECT_KIND.to_string(),
            byte_count: Some(1),
            content_digest_blake3: Some(DIGEST_B.to_string()),
            executable: None,
            mode: Some("0444".to_string()),
            symlink_target: None,
            children: Vec::new(),
            secret_descriptor: None,
            path_views: Vec::new(),
        })
        .unwrap();
        let secret = admit_cas_object(CasObjectManifestInput {
            kind: SECRET_DESCRIPTOR_KIND.to_string(),
            byte_count: None,
            content_digest_blake3: None,
            executable: None,
            mode: None,
            symlink_target: None,
            children: Vec::new(),
            secret_descriptor: Some(RedactedSecretDescriptor {
                descriptor_ref: object_ref("secret-descriptor"),
                purpose: "signing-token".to_string(),
                redaction: "descriptor-only".to_string(),
            }),
            path_views: Vec::new(),
        })
        .unwrap();

        assert!(file.object_ref.starts_with(OBJECT_REF_PREFIX));
        assert!(directory.object_ref.starts_with(OBJECT_REF_PREFIX));
        assert_eq!(symlink.symlink_target.as_deref(), Some("file"));
        assert_eq!(generated.kind, GENERATED_PAYLOAD_OBJECT_KIND);
        assert_eq!(secret.kind, SECRET_DESCRIPTOR_KIND);
    }

    #[test]
    fn build_correctness_cas_rejects_digest_mismatch_shape_path_only_identity_and_path_traversal() {
        let path_only = CasObjectManifestInput {
            content_digest_blake3: None,
            ..file_manifest_input(vec!["/tmp/out".to_string()])
        };
        let bad_digest = CasObjectManifestInput {
            content_digest_blake3: Some("not-a-digest".to_string()),
            ..file_manifest_input(Vec::new())
        };
        let traversal = file_manifest_input(vec!["../secret".to_string()]);

        assert!(admit_cas_object(path_only).unwrap_err().contains("path-only"));
        assert!(admit_cas_object(bad_digest).unwrap_err().contains("lowercase hex"));
        assert!(admit_cas_object(traversal).unwrap_err().contains("traversal"));
    }

    #[test]
    fn build_correctness_path_views_do_not_change_object_identity() {
        let left = admit_cas_object(file_manifest_input(vec!["/tmp/left".to_string()])).unwrap();
        let right = admit_cas_object(file_manifest_input(vec!["/different/view".to_string()])).unwrap();

        assert_eq!(left.object_ref, right.object_ref);
        assert_ne!(left.path_views, right.path_views);
    }

    #[test]
    fn build_correctness_hermetic_policy_reports_enforced_or_blocks_unsupported_modes() {
        let capabilities = ExecutorCapabilities {
            sandbox_modes: vec![sandbox_policy().mode],
            network_modes: vec![NETWORK_POLICY_OFFLINE.to_string()],
        };
        let report = validate_hermetic_policy(sandbox_policy(), network_policy(), &capabilities).unwrap();
        let unsupported = validate_hermetic_policy(
            SandboxPolicy {
                mode: "kernel-vm".to_string(),
                ..sandbox_policy()
            },
            network_policy(),
            &capabilities,
        )
        .unwrap_err();

        assert_eq!(report.enforcement_status, ENFORCED_STATUS);
        assert!(report.sandbox_report_ref.starts_with(SANDBOX_REPORT_REF_PREFIX));
        assert_eq!(unsupported, vec!["unsupported-sandbox-policy:kernel-vm".to_string()]);
    }

    #[test]
    fn build_correctness_reference_scan_accepts_declared_refs_and_rejects_forbidden_cases() {
        let declared = object_ref("runtime");
        let accepted = validate_reference_scan(ReferenceScanInput {
            output_object_ref: object_ref("out"),
            scan_root_ref: object_ref("scan-root"),
            scanner_kind: "byte-scan".to_string(),
            declared_refs: vec![declared.clone()],
            forbidden_refs: Vec::new(),
            observations: vec![ReferenceObservation {
                ref_value: declared.clone(),
                view: "bin/app".to_string(),
            }],
        })
        .unwrap();
        let rejected = validate_reference_scan(ReferenceScanInput {
            output_object_ref: object_ref("out"),
            scan_root_ref: object_ref("scan-root"),
            scanner_kind: "byte-scan".to_string(),
            declared_refs: vec![declared.clone()],
            forbidden_refs: vec!["/tmp/host".to_string()],
            observations: vec![
                ReferenceObservation {
                    ref_value: "/tmp/host".to_string(),
                    view: "bin/app".to_string(),
                },
                ReferenceObservation {
                    ref_value: declared,
                    view: "bin/../secret".to_string(),
                },
                ReferenceObservation {
                    ref_value: format!("{PLAINTEXT_SECRET_MARKER}token"),
                    view: "secret-bytes".to_string(),
                },
                ReferenceObservation {
                    ref_value: object_ref("other"),
                    view: "bin/app".to_string(),
                },
            ],
        })
        .unwrap_err();

        assert_eq!(accepted.status, "accepted");
        assert!(accepted.scan_ref.starts_with(REFERENCE_SCAN_REF_PREFIX));
        assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic.starts_with("forbidden-ref:")));
        assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic.starts_with("path-traversal:")));
        assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic == "plaintext-secret-bytes"));
        assert!(rejected.diagnostics.iter().any(|diagnostic| diagnostic.starts_with("duplicate-conflicting-view:")));
    }

    #[test]
    fn build_correctness_action_receipt_is_deterministic_and_bounded() {
        let action = action_ref("action");
        let produced = object_ref("produced");
        let receipt = action_receipt(receipt_input(action.clone(), produced.clone())).unwrap();
        let repeated = action_receipt(receipt_input(action, produced)).unwrap();
        let rendered = receipt_json(&receipt).unwrap();

        assert_eq!(receipt.receipt_ref, repeated.receipt_ref);
        assert!(receipt.receipt_ref.starts_with(ACTION_RECEIPT_REF_PREFIX));
        assert!(rendered.contains(STRONG_CLAIM));
        assert!(receipt.non_claims.iter().any(|claim| claim == NON_CLAIM_COMPILER_CORRECTNESS));
        assert!(receipt.non_claims.iter().any(|claim| claim == NON_CLAIM_FRONTEND_MODULE_CORRECTNESS));
    }

    #[test]
    fn build_correctness_reuse_admission_accepts_matching_receipts_and_rejects_stale_or_unsigned() {
        let action = action_ref("action");
        let produced = object_ref("produced");
        let candidate = action_receipt(receipt_input(action.clone(), produced.clone())).unwrap();
        let accepted = admit_reuse(ReuseAdmissionRequest {
            requested_action_ref: action.clone(),
            requested_output_refs: vec![produced.clone()],
            requested_sandbox_policy: sandbox_policy(),
            requested_network_policy: network_policy(),
            required_signatures: true,
            trusted_producers: vec![PRODUCER_LOCAL.to_string()],
            candidate: candidate.clone(),
        });
        let mut stale_candidate = candidate;
        stale_candidate.action_ref = action_ref("old-action");
        stale_candidate.signature_refs.clear();
        stale_candidate.producer_identity = "unknown".to_string();
        let rejected = admit_reuse(ReuseAdmissionRequest {
            requested_action_ref: action,
            requested_output_refs: vec![produced],
            requested_sandbox_policy: sandbox_policy(),
            requested_network_policy: network_policy(),
            required_signatures: true,
            trusted_producers: vec![PRODUCER_LOCAL.to_string()],
            candidate: stale_candidate,
        });

        assert!(accepted.admitted);
        assert!(accepted.diagnostics.is_empty());
        assert!(!rejected.admitted);
        assert!(rejected.diagnostics.contains(&"stale-action-ref".to_string()));
        assert!(rejected.diagnostics.contains(&"missing-required-signature".to_string()));
        assert!(rejected.diagnostics.contains(&"unsupported-producer-identity".to_string()));
    }

    #[test]
    fn build_correctness_frontend_refs_remain_opaque_boundary_data() {
        let mut input = action_input();
        input.frontend_spec_refs = vec![
            "onix://module/roles/system".to_string(),
            "nixos://option/services.sshd.enable".to_string(),
        ];
        let spec = canonical_action_spec(input).unwrap();

        assert_eq!(spec.frontend_spec_refs.len(), 2);
        assert!(spec.frontend_spec_refs[0].starts_with("nixos://"));
        assert!(spec.frontend_spec_refs[1].starts_with("onix://"));
    }
}
