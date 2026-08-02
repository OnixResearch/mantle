// r[impl foreign_derivation_import.executable_plan]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crunch_build::EXECUTION_PROFILE_BINDING_ENV;
use crunch_build::ExecutionProfile;
use crunch_build::foreign_profile_for_producer;
use data_encoding::HEXLOWER;
use nix_compat::derivation::Derivation;
use serde::Deserialize;
use serde::Serialize;

use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::ForeignDerivationNode;
use crate::foreign_derivation_import::ImportDiagnostic;
use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_derivation_import::PackageIndex;
use crate::foreign_derivation_import::PackageIndexEntry;
use crate::foreign_derivation_import::SandboxAuditEvent;
use crate::foreign_derivation_import::SubstitutionAuditEvent;
use crate::foreign_derivation_import::TranslationPolicy;
use crate::foreign_derivation_import::foreign_import_non_claims;
use crate::foreign_derivation_import::translate_foreign_graph;
use crate::foreign_graph_compiler::CompiledDigestFact;
use crate::foreign_graph_compiler::CompiledForeignBuiltin;
use crate::foreign_graph_compiler::CompiledForeignGraph;
use crate::foreign_graph_compiler::CompiledForeignUnit;
use crate::foreign_graph_compiler::CompiledSourceRequirement;
use crate::foreign_graph_compiler::ExactForeignPathMaps;
use crate::foreign_graph_compiler::compile_foreign_graph_with_profile;
use crate::foreign_graph_compiler::validate_digest_facts;

const EXECUTABLE_PLAN_SCHEMA: &str = "mantle-foreign-executable-plan-v1";
const PLAN_DIGEST_DOMAIN: &str = "mantle-plan";
const BLAKE3_ALGORITHM: &str = "blake3";
const PLAN_IDENTITY_ROLE: &str = "foreign-executable-plan";
const TARGET_HDM_ROLE: &str = "target-hash-derivation-modulo";
const TARGET_ATERM_ROLE: &str = "target-derivation-aterm";
const FIXED_OUTPUT_CONTENT_ROLE: &str = "fixed-output-content";
const FOREIGN_DIGEST_DOMAIN: &str = "foreign-compatible";
const SCHEDULER_PROFILE: &str = "mantle-native-scheduler-v1";
const FETCH_PROFILE: &str = "mantle-bounded-fetch-v1";
const STORE_PROFILE: &str = "mantle-store-admission-v1";
const SANDBOX_CLASSIFICATION: &str = "approved-compatibility-capability";
const SUBSTITUTION_CLASSIFICATION: &str = "cache-hint-policy-data-store-admission-required";
const BLAKE3_HEX_CHARS: usize = 64;
const MAX_PLAN_DIAGNOSTICS: usize = 2_048;
const MAX_PLAN_NON_CLAIMS: usize = 32;
const MAX_PLAN_FETCH_CANDIDATES: usize = 16;
const BLAKE3_BYTES: usize = 32;
const NIX_STORE_PREFIX: &str = "/nix/store";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ForeignExecutablePlan {
    pub(crate) schema: String,
    pub(crate) plan_identity: CompiledDigestFact,
    pub(crate) accepted_import: AcceptedImportIdentity,
    #[serde(rename = "roots")]
    pub(crate) selected_roots: Vec<ExecutableSelectedRoot>,
    pub(crate) target_store_prefix: String,
    pub(crate) native_units: Vec<ExecutableNativeUnit>,
    pub(crate) exact_path_maps: ExactForeignPathMaps,
    pub(crate) source_requirements: Vec<CompiledSourceRequirement>,
    pub(crate) execution_profiles: ExecutionProfileReferences,
    pub(crate) diagnostics: Vec<ImportDiagnostic>,
    pub(crate) sandbox_audit: Vec<SandboxAuditEvent>,
    pub(crate) substitution_audit: Vec<SubstitutionAuditEvent>,
    pub(crate) forbidden_process_invocations: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AcceptedImportIdentity {
    pub(crate) receipt_schema: String,
    pub(crate) receipt_digest_blake3: String,
    pub(crate) producer_identity: String,
    pub(crate) raw_graph_digest_blake3: String,
    pub(crate) translation_policy_digest_blake3: String,
    pub(crate) translated_graph_digest_blake3: String,
    pub(crate) package_index_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExecutableSelectedRoot {
    pub(crate) package_name: String,
    pub(crate) system: String,
    pub(crate) node_id: String,
    pub(crate) foreign_derivation: String,
    pub(crate) target_derivation: String,
    pub(crate) target_outputs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExecutableNativeUnit {
    pub(crate) sequence: usize,
    pub(crate) node_id: String,
    pub(crate) foreign_derivation: String,
    pub(crate) target_derivation: String,
    pub(crate) hdm_blake3: String,
    pub(crate) derivation_aterm: String,
    pub(crate) builder: String,
    pub(crate) system: String,
    pub(crate) arguments: Vec<String>,
    pub(crate) environment: BTreeMap<String, String>,
    pub(crate) foreign_outputs: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, String>,
    pub(crate) input_derivations: BTreeMap<String, Vec<String>>,
    pub(crate) input_sources: Vec<String>,
    pub(crate) execution_profile_id: String,
    pub(crate) execution_profile_digest_blake3: String,
    pub(crate) builtin: CompiledForeignBuiltin,
    pub(crate) digest_facts: Vec<CompiledDigestFact>,
    pub(crate) declared_references: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExecutionProfileReferences {
    pub(crate) scheduler: String,
    pub(crate) fetch: String,
    pub(crate) store_admission: String,
    pub(crate) fetch_cache_policy_digest_blake3: String,
    pub(crate) sandbox_policy_digest_blake3: String,
}

#[derive(Serialize)]
struct PlanIdentityMaterial<'a> {
    schema: &'a str,
    accepted_import: &'a AcceptedImportIdentity,
    selected_roots: &'a [ExecutableSelectedRoot],
    target_store_prefix: &'a str,
    native_units: &'a [ExecutableNativeUnit],
    exact_path_maps: &'a ExactForeignPathMaps,
    source_requirements: &'a [CompiledSourceRequirement],
    execution_profiles: &'a ExecutionProfileReferences,
    diagnostics: &'a [ImportDiagnostic],
    sandbox_audit: &'a [SandboxAuditEvent],
    substitution_audit: &'a [SubstitutionAuditEvent],
    forbidden_process_invocations: &'a [String],
    non_claims: &'a [String],
}

pub(crate) fn compile_foreign_executable_plan(
    graph: &ForeignDerivationGraph,
    package_index: &PackageIndex,
    policy: &TranslationPolicy,
    package: &str,
    system: &str,
) -> Result<(ForeignExecutablePlan, ImportReceipt), ImportDiagnostic> {
    let execution_profile = foreign_profile_for_producer(&graph.producer.kind);
    compile_foreign_executable_plan_with_profile(graph, package_index, policy, package, system, &execution_profile)
}

pub(crate) fn compile_foreign_executable_plan_with_profile(
    graph: &ForeignDerivationGraph,
    package_index: &PackageIndex,
    policy: &TranslationPolicy,
    package: &str,
    system: &str,
    execution_profile: &ExecutionProfile,
) -> Result<(ForeignExecutablePlan, ImportReceipt), ImportDiagnostic> {
    let (_translated, receipt) = translate_foreign_graph(graph, Some(package_index), policy)?;
    let selected_entry = select_package_entry(package_index, package, system)?;
    if !graph.root_derivation_ids.contains(&selected_entry.root_derivation_id) {
        return Err(plan_diagnostic(
            "foreign-plan-selection-not-root",
            None,
            "selected package index entry is not a declared graph root",
        ));
    }
    let compiled = compile_foreign_graph_with_profile(graph, &policy.target_prefix, execution_profile)?;
    let accepted_import = accepted_import_identity(&receipt)?;
    let selected_roots = executable_roots(graph, package_index, selected_entry, &compiled)?;
    let nodes = graph.nodes.iter().map(|node| (node.node_id.as_str(), node)).collect::<BTreeMap<_, _>>();
    let native_units = compiled
        .units
        .iter()
        .enumerate()
        .map(|(sequence, unit)| {
            let node = nodes.get(unit.node_id.as_str()).copied().ok_or_else(|| {
                plan_diagnostic(
                    "foreign-plan-unit-node-missing",
                    Some(&unit.node_id),
                    "compiled unit has no accepted graph node",
                )
            })?;
            executable_unit(sequence, unit, node, &compiled.target_store_prefix)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let execution_profiles = ExecutionProfileReferences {
        scheduler: SCHEDULER_PROFILE.to_string(),
        fetch: FETCH_PROFILE.to_string(),
        store_admission: STORE_PROFILE.to_string(),
        fetch_cache_policy_digest_blake3: receipt.fetch_cache_policy_digest.clone(),
        sandbox_policy_digest_blake3: receipt.sandbox_policy_digest.clone(),
    };
    let diagnostics = receipt.diagnostics.clone();
    let sandbox_audit = sandbox_audit(graph, &compiled.dependency_order);
    let substitution_audit = substitution_audit(graph, &compiled.dependency_order);
    let non_claims = executable_plan_non_claims();
    let mut plan = ForeignExecutablePlan {
        schema: EXECUTABLE_PLAN_SCHEMA.to_string(),
        plan_identity: CompiledDigestFact {
            domain: PLAN_DIGEST_DOMAIN.to_string(),
            algorithm: BLAKE3_ALGORITHM.to_string(),
            role: PLAN_IDENTITY_ROLE.to_string(),
            value: String::new(),
        },
        accepted_import,
        selected_roots,
        target_store_prefix: compiled.target_store_prefix,
        native_units,
        exact_path_maps: compiled.path_maps,
        source_requirements: compiled.source_requirements,
        execution_profiles,
        diagnostics,
        sandbox_audit,
        substitution_audit,
        forbidden_process_invocations: Vec::new(),
        non_claims,
    };
    plan.plan_identity.value = plan_identity_digest(&plan)?;
    validate_foreign_executable_plan(&plan)?;
    debug_assert!(!plan.selected_roots.is_empty());
    debug_assert!(!plan.native_units.is_empty());
    Ok((plan, receipt))
}

pub(crate) fn validate_foreign_executable_plan(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    validate_plan_header(plan)?;
    validate_plan_identity(plan)?;
    validate_plan_profiles(plan)?;
    validate_plan_units(plan)?;
    validate_plan_roots(plan)?;
    validate_plan_sources(plan)?;
    validate_plan_claim_boundary(plan)?;
    debug_assert_eq!(plan.schema, EXECUTABLE_PLAN_SCHEMA);
    debug_assert!(!plan.selected_roots.is_empty());
    Ok(())
}

fn accepted_import_identity(receipt: &ImportReceipt) -> Result<AcceptedImportIdentity, ImportDiagnostic> {
    let package_index_digest = receipt.package_index_digest.clone().ok_or_else(|| {
        plan_diagnostic(
            "foreign-plan-package-index-identity-missing",
            None,
            "accepted import receipt has no package index digest",
        )
    })?;
    Ok(AcceptedImportIdentity {
        receipt_schema: receipt.schema.clone(),
        receipt_digest_blake3: canonical_blake3(receipt, "receipt")?,
        producer_identity: receipt.producer_identity.clone(),
        raw_graph_digest_blake3: receipt.raw_graph_digest.clone(),
        translation_policy_digest_blake3: receipt.translation_policy_digest.clone(),
        translated_graph_digest_blake3: receipt.translated_graph_digest.clone(),
        package_index_digest_blake3: package_index_digest,
    })
}

fn select_package_entry<'a>(
    package_index: &'a PackageIndex,
    package: &str,
    system: &str,
) -> Result<&'a PackageIndexEntry, ImportDiagnostic> {
    let matches = package_index
        .entries
        .iter()
        .filter(|entry| entry.system == system)
        .filter(|entry| entry.name == package || entry.aliases.iter().any(|alias| alias == package))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [entry] => Ok(entry),
        [] => Err(plan_diagnostic(
            "foreign-plan-package-selection-missing",
            None,
            "package index has no matching package and system",
        )),
        _ => Err(plan_diagnostic(
            "foreign-plan-package-selection-ambiguous",
            None,
            "package index has more than one matching package and system",
        )),
    }
}

fn executable_roots(
    graph: &ForeignDerivationGraph,
    package_index: &PackageIndex,
    selected_entry: &PackageIndexEntry,
    compiled: &CompiledForeignGraph,
) -> Result<Vec<ExecutableSelectedRoot>, ImportDiagnostic> {
    let nodes = graph.nodes.iter().map(|node| (node.node_id.as_str(), node)).collect::<BTreeMap<_, _>>();
    let mut roots = Vec::with_capacity(graph.root_derivation_ids.len());
    for node_id in &graph.root_derivation_ids {
        let node = nodes.get(node_id.as_str()).copied().ok_or_else(|| {
            plan_diagnostic("foreign-plan-root-node-missing", Some(node_id), "selected root node is absent")
        })?;
        let target_derivation = compiled.roots.get(node_id).cloned().ok_or_else(|| {
            plan_diagnostic("foreign-plan-partial-root", Some(node_id), "compiled graph omitted a selected root")
        })?;
        let target_outputs = node
            .outputs
            .iter()
            .map(|(output_name, output)| {
                compiled
                    .path_maps
                    .outputs
                    .get(&output.path)
                    .map(|target| (output_name.clone(), target.clone()))
                    .ok_or_else(|| {
                        plan_diagnostic(
                            "foreign-plan-root-output-missing",
                            Some(node_id),
                            "selected root output has no exact target mapping",
                        )
                    })
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let indexed = package_index
            .entries
            .iter()
            .filter(|entry| entry.root_derivation_id == *node_id)
            .min_by(|left, right| left.name.cmp(&right.name));
        let package_name = if selected_entry.root_derivation_id == *node_id {
            selected_entry.name.clone()
        } else {
            indexed.map(|entry| entry.name.clone()).unwrap_or_else(|| node.name.clone())
        };
        roots.push(ExecutableSelectedRoot {
            package_name,
            system: node.system.clone(),
            node_id: node_id.clone(),
            foreign_derivation: node.original_derivation.clone(),
            target_derivation,
            target_outputs,
        });
    }
    roots.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    debug_assert_eq!(roots.len(), graph.root_derivation_ids.len());
    Ok(roots)
}

fn executable_unit(
    sequence: usize,
    unit: &CompiledForeignUnit,
    node: &ForeignDerivationNode,
    target_store_prefix: &str,
) -> Result<ExecutableNativeUnit, ImportDiagnostic> {
    let derivation_aterm = String::from_utf8(unit.derivation.to_aterm_bytes_with_store_dir(target_store_prefix))
        .map_err(|error| {
            plan_diagnostic(
                "foreign-plan-aterm-non-utf8",
                Some(&unit.node_id),
                &format!("native derivation ATerm is not UTF-8: {error}"),
            )
        })?;
    let environment = unit
        .derivation
        .environment
        .iter()
        .map(|(key, value)| {
            std::str::from_utf8(value).map(|text| (key.clone(), text.to_string())).map_err(|error| {
                plan_diagnostic(
                    "foreign-plan-environment-non-utf8",
                    Some(&unit.node_id),
                    &format!("native derivation environment is not UTF-8: {error}"),
                )
            })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let outputs = unit
        .derivation
        .outputs
        .iter()
        .map(|(name, output)| {
            output
                .path
                .as_ref()
                .map(|path| (name.clone(), path.to_absolute_path_with_prefix(target_store_prefix)))
                .ok_or_else(|| {
                    plan_diagnostic(
                        "foreign-plan-output-unresolved",
                        Some(&unit.node_id),
                        "native derivation output has no exact target path",
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let input_derivations = unit
        .derivation
        .input_derivations
        .iter()
        .map(|(path, outputs)| {
            (path.to_absolute_path_with_prefix(target_store_prefix), outputs.iter().cloned().collect::<Vec<_>>())
        })
        .collect::<BTreeMap<_, _>>();
    let input_sources = unit
        .derivation
        .input_sources
        .iter()
        .map(|path| path.to_absolute_path_with_prefix(target_store_prefix))
        .collect::<Vec<_>>();
    Ok(ExecutableNativeUnit {
        sequence,
        node_id: unit.node_id.clone(),
        foreign_derivation: unit.foreign_derivation.clone(),
        target_derivation: unit.target_derivation.clone(),
        hdm_blake3: HEXLOWER.encode(&unit.hdm),
        derivation_aterm,
        builder: unit.derivation.builder.clone(),
        system: unit.derivation.system.clone(),
        arguments: unit.derivation.arguments.clone(),
        environment,
        foreign_outputs: node.outputs.iter().map(|(name, output)| (name.clone(), output.path.clone())).collect(),
        outputs,
        input_derivations,
        input_sources,
        execution_profile_id: unit.execution_profile_id.clone(),
        execution_profile_digest_blake3: unit.execution_profile_digest_blake3.clone(),
        builtin: unit.builtin.clone(),
        digest_facts: unit.digest_facts.clone(),
        declared_references: unit.declared_references.clone(),
    })
}

fn sandbox_audit(graph: &ForeignDerivationGraph, dependency_order: &[String]) -> Vec<SandboxAuditEvent> {
    let reachable = dependency_order.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut events = graph
        .nodes
        .iter()
        .filter(|node| reachable.contains(node.node_id.as_str()))
        .flat_map(|node| {
            node.sandbox_capabilities.iter().map(|capability| SandboxAuditEvent {
                node_id: node.node_id.clone(),
                capability: capability.clone(),
                classification: SANDBOX_CLASSIFICATION.to_string(),
            })
        })
        .collect::<Vec<_>>();
    events.sort_by(|left, right| left.node_id.cmp(&right.node_id).then_with(|| left.capability.cmp(&right.capability)));
    events
}

fn substitution_audit(graph: &ForeignDerivationGraph, dependency_order: &[String]) -> Vec<SubstitutionAuditEvent> {
    let reachable = dependency_order.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut events = graph
        .nodes
        .iter()
        .filter(|node| reachable.contains(node.node_id.as_str()))
        .flat_map(|node| {
            node.cache_hints.iter().map(|hint| SubstitutionAuditEvent {
                node_id: node.node_id.clone(),
                cache_url: hint.cache_url.clone(),
                trust_scope: hint.trust_scope.clone(),
                classification: SUBSTITUTION_CLASSIFICATION.to_string(),
                store_admission_required: true,
            })
        })
        .collect::<Vec<_>>();
    events.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.cache_url.cmp(&right.cache_url))
            .then_with(|| left.trust_scope.cmp(&right.trust_scope))
    });
    events
}

fn executable_plan_non_claims() -> Vec<String> {
    let mut non_claims = foreign_import_non_claims();
    non_claims.extend([
        "not-source-availability".to_string(),
        "not-scheduler-execution".to_string(),
        "not-store-admission".to_string(),
        "not-realization".to_string(),
    ]);
    non_claims.sort();
    non_claims.dedup();
    non_claims
}

fn validate_plan_header(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    if plan.schema != EXECUTABLE_PLAN_SCHEMA {
        return Err(plan_diagnostic(
            "foreign-plan-schema-unsupported",
            None,
            "foreign executable plan schema is unsupported",
        ));
    }
    if plan.selected_roots.is_empty() || plan.native_units.is_empty() {
        return Err(plan_diagnostic(
            "foreign-plan-empty",
            None,
            "foreign executable plan requires roots and native units",
        ));
    }
    if plan.diagnostics.len() > MAX_PLAN_DIAGNOSTICS || plan.non_claims.len() > MAX_PLAN_NON_CLAIMS {
        return Err(plan_diagnostic(
            "foreign-plan-collection-limit-exceeded",
            None,
            "foreign executable plan collection exceeds its bound",
        ));
    }
    Ok(())
}

fn validate_plan_identity(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    if plan.plan_identity.domain != PLAN_DIGEST_DOMAIN
        || plan.plan_identity.algorithm != BLAKE3_ALGORITHM
        || plan.plan_identity.role != PLAN_IDENTITY_ROLE
        || !is_blake3_hex(&plan.plan_identity.value)
    {
        return Err(plan_diagnostic(
            "foreign-plan-identity-domain-mismatch",
            None,
            "foreign executable plan identity has the wrong domain, algorithm, role, or shape",
        ));
    }
    let expected = plan_identity_digest(plan)?;
    if expected != plan.plan_identity.value {
        return Err(plan_diagnostic(
            "foreign-plan-identity-stale",
            None,
            "foreign executable plan identity does not match its canonical payload",
        ));
    }
    for value in [
        &plan.accepted_import.receipt_digest_blake3,
        &plan.accepted_import.raw_graph_digest_blake3,
        &plan.accepted_import.translation_policy_digest_blake3,
        &plan.accepted_import.translated_graph_digest_blake3,
        &plan.accepted_import.package_index_digest_blake3,
    ] {
        if !is_blake3_hex(value) {
            return Err(plan_diagnostic(
                "foreign-plan-import-identity-invalid",
                None,
                "accepted import identity is not a BLAKE3 digest",
            ));
        }
    }
    Ok(())
}

fn validate_plan_profiles(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    if plan.execution_profiles.scheduler != SCHEDULER_PROFILE
        || plan.execution_profiles.fetch != FETCH_PROFILE
        || plan.execution_profiles.store_admission != STORE_PROFILE
        || !is_blake3_hex(&plan.execution_profiles.fetch_cache_policy_digest_blake3)
        || !is_blake3_hex(&plan.execution_profiles.sandbox_policy_digest_blake3)
    {
        return Err(plan_diagnostic(
            "foreign-plan-profile-invalid",
            None,
            "foreign executable plan execution profile reference is invalid",
        ));
    }
    Ok(())
}

fn validate_plan_units(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    let mut seen_nodes = BTreeSet::new();
    let mut seen_derivations = BTreeSet::new();
    let mut known_hdms = BTreeMap::new();
    for (expected_sequence, unit) in plan.native_units.iter().enumerate() {
        if unit.sequence != expected_sequence || !seen_nodes.insert(unit.node_id.as_str()) {
            return Err(plan_diagnostic(
                "foreign-plan-unit-order-invalid",
                Some(&unit.node_id),
                "native unit sequence or node identity is duplicated or out of order",
            ));
        }
        for parent in unit.input_derivations.keys() {
            if !seen_derivations.contains(parent.as_str()) {
                return Err(plan_diagnostic(
                    "foreign-plan-dependency-order-invalid",
                    Some(&unit.node_id),
                    "native unit references a derivation that does not precede it",
                ));
            }
        }
        if seen_derivations.contains(unit.target_derivation.as_str()) {
            return Err(plan_diagnostic(
                "foreign-plan-unit-order-invalid",
                Some(&unit.node_id),
                "native unit target derivation identity is duplicated",
            ));
        }
        let derivation = validate_unit_aterm_projection(plan, unit)?;
        let mut hdm_derivation = derivation.clone();
        for (output_name, output) in &mut hdm_derivation.outputs {
            output.path = None;
            hdm_derivation.environment.insert(output_name.clone(), bstr::BString::from(""));
        }
        let declared_hdm = decode_blake3(&unit.hdm_blake3, &unit.node_id)?;
        let recomputed_hdm = hdm_derivation.hash_derivation_modulo_with_store_dir(
            |parent| {
                let absolute = parent.to_absolute_path_with_prefix(&plan.target_store_prefix);
                let hdm = known_hdms.get(&absolute).copied();
                assert!(hdm.is_some(), "pre-validated executable parent HDM must exist");
                hdm.unwrap_or([0u8; BLAKE3_BYTES])
            },
            &plan.target_store_prefix,
        );
        if recomputed_hdm != declared_hdm {
            return Err(plan_diagnostic(
                "foreign-plan-unit-hdm-mismatch",
                Some(&unit.node_id),
                "native unit HDM does not match its derivation and known parents",
            ));
        }
        seen_derivations.insert(unit.target_derivation.as_str());
        known_hdms.insert(unit.target_derivation.clone(), declared_hdm);
        validate_unit_identity(plan, unit)?;
        validate_unit_builtin(unit)?;
        validate_no_foreign_references(plan, unit)?;
    }
    Ok(())
}

pub(crate) fn validate_unit_aterm_projection(
    plan: &ForeignExecutablePlan,
    unit: &ExecutableNativeUnit,
) -> Result<Derivation, ImportDiagnostic> {
    let parser_aterm = unit.derivation_aterm.replace(&plan.target_store_prefix, NIX_STORE_PREFIX);
    let derivation = Derivation::from_aterm_bytes(parser_aterm.as_bytes()).map_err(|error| {
        plan_diagnostic(
            "foreign-plan-unit-aterm-invalid",
            Some(&unit.node_id),
            &format!("native unit ATerm cannot be parsed: {error:?}"),
        )
    })?;
    let restore = |value: &str| value.replace(NIX_STORE_PREFIX, &plan.target_store_prefix);
    let arguments = derivation.arguments.iter().map(|value| restore(value)).collect::<Vec<_>>();
    let environment = derivation
        .environment
        .iter()
        .map(|(key, value)| {
            std::str::from_utf8(value).map(|text| (key.clone(), restore(text))).map_err(|error| {
                plan_diagnostic(
                    "foreign-plan-environment-non-utf8",
                    Some(&unit.node_id),
                    &format!("parsed native environment is not UTF-8: {error}"),
                )
            })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let outputs = derivation
        .outputs
        .iter()
        .map(|(name, output)| {
            output
                .path
                .as_ref()
                .map(|path| (name.clone(), path.to_absolute_path_with_prefix(&plan.target_store_prefix)))
                .ok_or_else(|| {
                    plan_diagnostic(
                        "foreign-plan-output-unresolved",
                        Some(&unit.node_id),
                        "parsed native output has no path",
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let input_derivations = derivation
        .input_derivations
        .iter()
        .map(|(path, outputs)| {
            (
                path.to_absolute_path_with_prefix(&plan.target_store_prefix),
                outputs.iter().cloned().collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let input_sources = derivation
        .input_sources
        .iter()
        .map(|path| path.to_absolute_path_with_prefix(&plan.target_store_prefix))
        .collect::<Vec<_>>();
    let builder = restore(&derivation.builder);
    let system = restore(&derivation.system);
    if builder != unit.builder
        || system != unit.system
        || arguments != unit.arguments
        || environment != unit.environment
        || outputs != unit.outputs
        || input_derivations != unit.input_derivations
        || input_sources != unit.input_sources
    {
        return Err(plan_diagnostic(
            "foreign-plan-unit-projection-mismatch",
            Some(&unit.node_id),
            "native unit fields differ from its canonical ATerm",
        ));
    }
    let mut target_derivation = derivation;
    target_derivation.builder = builder;
    target_derivation.system = system;
    target_derivation.arguments = arguments;
    target_derivation.environment =
        environment.into_iter().map(|(key, value)| (key, value.into_bytes().into())).collect();
    Ok(target_derivation)
}

fn decode_blake3(value: &str, node_id: &str) -> Result<[u8; BLAKE3_BYTES], ImportDiagnostic> {
    let bytes = HEXLOWER.decode(value.as_bytes()).map_err(|error| {
        plan_diagnostic(
            "foreign-plan-unit-hdm-invalid",
            Some(node_id),
            &format!("native unit HDM is not lowercase hexadecimal: {error}"),
        )
    })?;
    bytes.try_into().map_err(|_| {
        plan_diagnostic("foreign-plan-unit-hdm-invalid", Some(node_id), "native unit HDM has the wrong byte length")
    })
}

fn validate_unit_identity(plan: &ForeignExecutablePlan, unit: &ExecutableNativeUnit) -> Result<(), ImportDiagnostic> {
    if unit.execution_profile_id.is_empty()
        || !is_blake3_hex(&unit.execution_profile_digest_blake3)
        || unit.environment.get(EXECUTION_PROFILE_BINDING_ENV) != Some(&unit.execution_profile_digest_blake3)
    {
        return Err(plan_diagnostic(
            "foreign-plan-execution-profile-binding-invalid",
            Some(&unit.node_id),
            "native unit execution profile binding is absent, malformed, or stale",
        ));
    }
    if plan.exact_path_maps.derivations.get(&unit.foreign_derivation) != Some(&unit.target_derivation) {
        return Err(plan_diagnostic(
            "foreign-plan-derivation-map-mismatch",
            Some(&unit.node_id),
            "native unit derivation identity differs from its exact path map",
        ));
    }
    validate_digest_facts(&unit.digest_facts)?;
    let expected_aterm = blake3::hash(unit.derivation_aterm.as_bytes()).to_hex().to_string();
    let aterm_fact = unit.digest_facts.iter().find(|fact| fact.role == TARGET_ATERM_ROLE);
    let hdm_fact = unit.digest_facts.iter().find(|fact| fact.role == TARGET_HDM_ROLE);
    if aterm_fact.map(|fact| fact.value.as_str()) != Some(expected_aterm.as_str())
        || hdm_fact.map(|fact| fact.value.as_str()) != Some(unit.hdm_blake3.as_str())
    {
        return Err(plan_diagnostic(
            "foreign-plan-unit-digest-stale",
            Some(&unit.node_id),
            "native unit target digest does not match its resolved data",
        ));
    }
    if !unit.target_derivation.starts_with(&format!("{}/", plan.target_store_prefix)) {
        return Err(plan_diagnostic(
            "foreign-plan-target-prefix-mismatch",
            Some(&unit.node_id),
            "native unit derivation is outside the target store prefix",
        ));
    }
    if unit.foreign_outputs.len() != unit.outputs.len() {
        return Err(plan_diagnostic(
            "foreign-plan-output-map-mismatch",
            Some(&unit.node_id),
            "native unit foreign and target output sets differ",
        ));
    }
    for (output_name, output_path) in &unit.outputs {
        if !output_path.starts_with(&format!("{}/", plan.target_store_prefix)) {
            return Err(plan_diagnostic(
                "foreign-plan-output-prefix-mismatch",
                Some(&unit.node_id),
                &format!("native unit output {output_name} is outside the target store prefix"),
            ));
        }
        let foreign_output = unit.foreign_outputs.get(output_name).ok_or_else(|| {
            plan_diagnostic(
                "foreign-plan-output-map-mismatch",
                Some(&unit.node_id),
                "native unit target output has no foreign identity",
            )
        })?;
        if plan.exact_path_maps.outputs.get(foreign_output) != Some(output_path) {
            return Err(plan_diagnostic(
                "foreign-plan-output-map-mismatch",
                Some(&unit.node_id),
                "native unit output differs from the exact output map",
            ));
        }
    }
    Ok(())
}

fn validate_unit_builtin(unit: &ExecutableNativeUnit) -> Result<(), ImportDiagnostic> {
    match &unit.builtin {
        CompiledForeignBuiltin::NativeDerivation | CompiledForeignBuiltin::FixedOutput => Ok(()),
        CompiledForeignBuiltin::Download {
            candidates,
            content_digest,
            ..
        }
        | CompiledForeignBuiltin::GitDownload {
            candidates,
            content_digest,
            ..
        } => {
            if candidates.is_empty() || candidates.len() > MAX_PLAN_FETCH_CANDIDATES {
                return Err(plan_diagnostic(
                    "foreign-plan-fetch-candidates-invalid",
                    Some(&unit.node_id),
                    "native fetch unit candidate count is outside supported limits",
                ));
            }
            if content_digest.role != FIXED_OUTPUT_CONTENT_ROLE
                || content_digest.domain != FOREIGN_DIGEST_DOMAIN
                || content_digest.algorithm == BLAKE3_ALGORITHM
            {
                return Err(plan_diagnostic(
                    "foreign-plan-fixed-output-domain-mismatch",
                    Some(&unit.node_id),
                    "native fetch content digest uses the wrong identity domain",
                ));
            }
            Ok(())
        }
    }
}

fn validate_no_foreign_references(
    plan: &ForeignExecutablePlan,
    unit: &ExecutableNativeUnit,
) -> Result<(), ImportDiagnostic> {
    let mut values = vec![
        unit.derivation_aterm.as_str(),
        unit.builder.as_str(),
        unit.system.as_str(),
        unit.target_derivation.as_str(),
    ];
    values.extend(unit.arguments.iter().map(String::as_str));
    values.extend(unit.environment.values().map(String::as_str));
    values.extend(unit.outputs.values().map(String::as_str));
    values.extend(unit.input_derivations.keys().map(String::as_str));
    values.extend(unit.input_sources.iter().map(String::as_str));
    values.extend(unit.declared_references.iter().map(String::as_str));
    if values
        .iter()
        .any(|value| plan.accepted_source_prefixes().iter().any(|prefix| value.contains(&format!("{prefix}/"))))
    {
        return Err(plan_diagnostic(
            "foreign-plan-leftover-reference",
            Some(&unit.node_id),
            "native unit retains a foreign store reference",
        ));
    }
    Ok(())
}

fn validate_plan_roots(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    let units = plan.native_units.iter().map(|unit| (unit.node_id.as_str(), unit)).collect::<BTreeMap<_, _>>();
    let mut roots = BTreeSet::new();
    for root in &plan.selected_roots {
        if !roots.insert(root.node_id.as_str()) {
            return Err(plan_diagnostic(
                "foreign-plan-root-duplicate",
                Some(&root.node_id),
                "selected root is duplicated",
            ));
        }
        let unit = units.get(root.node_id.as_str()).copied().ok_or_else(|| {
            plan_diagnostic("foreign-plan-partial-root", Some(&root.node_id), "selected root has no native unit")
        })?;
        if unit.target_derivation != root.target_derivation || unit.outputs != root.target_outputs {
            return Err(plan_diagnostic(
                "foreign-plan-root-identity-mismatch",
                Some(&root.node_id),
                "selected root identity differs from its native unit",
            ));
        }
    }
    Ok(())
}

fn validate_plan_sources(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    let mut payload_ids = BTreeSet::new();
    for source in &plan.source_requirements {
        if !payload_ids.insert(source.payload_id.as_str())
            || plan.exact_path_maps.sources.get(&source.foreign_path) != Some(&source.target_path)
            || !is_blake3_hex(&source.descriptor_digest)
        {
            return Err(plan_diagnostic(
                "foreign-plan-source-requirement-invalid",
                None,
                "source requirement is duplicated, stale, or malformed",
            ));
        }
    }
    Ok(())
}

fn validate_plan_claim_boundary(plan: &ForeignExecutablePlan) -> Result<(), ImportDiagnostic> {
    let required = [
        "not-source-availability",
        "not-scheduler-execution",
        "not-store-admission",
        "not-output-trust",
        "not-package-correctness",
        "not-reproducibility",
        "not-realization",
    ];
    if !plan.forbidden_process_invocations.is_empty()
        || required.iter().any(|claim| !plan.non_claims.iter().any(|actual| actual == claim))
    {
        return Err(plan_diagnostic(
            "foreign-plan-claim-boundary-invalid",
            None,
            "foreign executable plan promotes a realization claim or permits a foreign process",
        ));
    }
    Ok(())
}

impl ForeignExecutablePlan {
    fn accepted_source_prefixes(&self) -> Vec<&str> {
        let mut prefixes = BTreeSet::new();
        for path in self
            .exact_path_maps
            .derivations
            .keys()
            .chain(self.exact_path_maps.outputs.keys())
            .chain(self.exact_path_maps.sources.keys())
        {
            if let Some((prefix, _)) = path.rsplit_once('/') {
                prefixes.insert(prefix);
            }
        }
        prefixes.into_iter().collect()
    }
}

fn plan_identity_digest(plan: &ForeignExecutablePlan) -> Result<String, ImportDiagnostic> {
    canonical_blake3(
        &PlanIdentityMaterial {
            schema: &plan.schema,
            accepted_import: &plan.accepted_import,
            selected_roots: &plan.selected_roots,
            target_store_prefix: &plan.target_store_prefix,
            native_units: &plan.native_units,
            exact_path_maps: &plan.exact_path_maps,
            source_requirements: &plan.source_requirements,
            execution_profiles: &plan.execution_profiles,
            diagnostics: &plan.diagnostics,
            sandbox_audit: &plan.sandbox_audit,
            substitution_audit: &plan.substitution_audit,
            forbidden_process_invocations: &plan.forbidden_process_invocations,
            non_claims: &plan.non_claims,
        },
        "executable plan",
    )
}

pub(crate) fn import_receipt_digest(receipt: &ImportReceipt) -> Result<String, ImportDiagnostic> {
    canonical_blake3(receipt, "receipt")
}

fn canonical_blake3<T: Serialize>(value: &T, artifact: &str) -> Result<String, ImportDiagnostic> {
    serde_json::to_vec(value).map(|bytes| blake3::hash(&bytes).to_hex().to_string()).map_err(|error| {
        plan_diagnostic(
            "foreign-plan-canonical-serialization-failed",
            None,
            &format!("canonical {artifact} serialization failed: {error}"),
        )
    })
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn plan_diagnostic(class: &str, node_id: Option<&str>, message: &str) -> ImportDiagnostic {
    ImportDiagnostic {
        class: class.to_string(),
        node_id: node_id.map(str::to_string),
        message: message.to_string(),
    }
}

// r[verify foreign_derivation_import.executable_plan]
#[cfg(test)]
mod tests {
    use super::*;

    const NIX_GRAPH: &str = include_str!("../tests/fixtures/foreign-import/nix-hello.graph.json");
    const NIX_INDEX: &str = include_str!("../tests/fixtures/foreign-import/nix-hello.index.json");
    const POLICY: &str = include_str!("../tests/fixtures/foreign-import/policy.json");

    #[test]
    fn executable_plan_is_deterministic_receipt_bound_and_self_validating() {
        let graph: ForeignDerivationGraph = serde_json::from_str(NIX_GRAPH).expect("graph fixture");
        let index: PackageIndex = serde_json::from_str(NIX_INDEX).expect("index fixture");
        let policy: TranslationPolicy = serde_json::from_str(POLICY).expect("policy fixture");
        let (first, first_receipt) = compile_foreign_executable_plan(&graph, &index, &policy, "hello", "x86_64-linux")
            .expect("executable plan must compile");
        let (second, second_receipt) =
            compile_foreign_executable_plan(&graph, &index, &policy, "hello", "x86_64-linux")
                .expect("repeat executable plan must compile");

        assert_eq!(first, second);
        assert_eq!(first_receipt, second_receipt);
        assert_eq!(first.schema, EXECUTABLE_PLAN_SCHEMA);
        assert!(is_blake3_hex(&first.plan_identity.value));
        assert_eq!(first.selected_roots.len(), graph.root_derivation_ids.len());
        assert_eq!(first.native_units.len(), first.exact_path_maps.derivations.len());
        assert!(first.native_units.iter().all(|unit| !unit.execution_profile_id.is_empty()));
        assert!(first.native_units.iter().all(|unit| is_blake3_hex(&unit.execution_profile_digest_blake3)));
        assert!(first.forbidden_process_invocations.is_empty());
        validate_foreign_executable_plan(&first).expect("emitted plan must validate");
    }

    #[test]
    fn executable_plan_validator_rejects_identity_domain_root_and_reference_tampering() {
        let mut plan = fixture_plan();
        plan.plan_identity.algorithm = "sha256".to_string();
        assert_class(validate_foreign_executable_plan(&plan), "foreign-plan-identity-domain-mismatch");

        let mut target_domain = fixture_plan();
        let target_fact = target_domain.native_units[0]
            .digest_facts
            .iter_mut()
            .find(|fact| fact.role == TARGET_ATERM_ROLE)
            .expect("target ATerm fact");
        target_fact.domain = FOREIGN_DIGEST_DOMAIN.to_string();
        target_fact.algorithm = "sha256".to_string();
        refresh_plan_identity(&mut target_domain);
        assert_class(validate_foreign_executable_plan(&target_domain), "foreign-compiler-digest-domain-mismatch");

        let mut stale_profile = fixture_plan();
        stale_profile.native_units[0].execution_profile_digest_blake3 = "0".repeat(BLAKE3_HEX_CHARS);
        refresh_plan_identity(&mut stale_profile);
        assert_class(
            validate_foreign_executable_plan(&stale_profile),
            "foreign-plan-execution-profile-binding-invalid",
        );

        let mut missing_root = fixture_plan();
        missing_root.native_units.clear();
        refresh_plan_identity(&mut missing_root);
        assert_class(validate_foreign_executable_plan(&missing_root), "foreign-plan-empty");

        let mut projection = fixture_plan();
        projection.native_units[0].builder = "/bin/false".to_string();
        refresh_plan_identity(&mut projection);
        assert_class(validate_foreign_executable_plan(&projection), "foreign-plan-unit-projection-mismatch");

        let mut leftover = fixture_plan();
        leftover.native_units[0]
            .declared_references
            .push("/nix/store/99999999999999999999999999999999-leftover/bin/tool".to_string());
        refresh_plan_identity(&mut leftover);
        assert_class(validate_foreign_executable_plan(&leftover), "foreign-plan-leftover-reference");
    }

    #[test]
    fn executable_plan_compilation_rejects_a_failed_sibling_root_without_partial_success() {
        let graph: ForeignDerivationGraph = serde_json::from_str(NIX_GRAPH).expect("graph fixture");
        let index: PackageIndex = serde_json::from_str(NIX_INDEX).expect("index fixture");
        let policy: TranslationPolicy = serde_json::from_str(POLICY).expect("policy fixture");
        let mut failed = graph.clone();
        let mut sibling = failed.nodes[0].clone();
        sibling.node_id = "nix:failed-sibling".to_string();
        sibling.original_derivation = "/nix/store/77777777777777777777777777777777-failed.drv".to_string();
        sibling.outputs.get_mut("out").expect("out output").path =
            "/nix/store/88888888888888888888888888888888-failed".to_string();
        sibling.builtin = "foreign:unsupported".to_string();
        failed.root_derivation_ids.push(sibling.node_id.clone());
        failed.nodes.push(sibling);

        assert_class(
            compile_foreign_executable_plan(&failed, &index, &policy, "hello", "x86_64-linux"),
            "unsupported-builtin",
        );
    }

    fn fixture_plan() -> ForeignExecutablePlan {
        let graph: ForeignDerivationGraph = serde_json::from_str(NIX_GRAPH).expect("graph fixture");
        let index: PackageIndex = serde_json::from_str(NIX_INDEX).expect("index fixture");
        let policy: TranslationPolicy = serde_json::from_str(POLICY).expect("policy fixture");
        compile_foreign_executable_plan(&graph, &index, &policy, "hello", "x86_64-linux")
            .expect("fixture plan")
            .0
    }

    fn refresh_plan_identity(plan: &mut ForeignExecutablePlan) {
        plan.plan_identity.value = plan_identity_digest(plan).expect("plan identity refresh");
    }

    fn assert_class<T>(result: Result<T, ImportDiagnostic>, expected: &str) {
        let error = result.err().expect("fixture must be rejected");
        assert_eq!(error.class, expected);
    }
}
