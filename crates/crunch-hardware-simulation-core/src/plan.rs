use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::digest::ACTION_REF_PREFIX;
use crate::digest::BoundedValue;
use crate::digest::COHORT_REF_PREFIX;
use crate::digest::NamedValue;
use crate::digest::OBJECT_REF_PREFIX;
use crate::digest::PROFILE_REF_PREFIX;
use crate::digest::TypedRefValidation;
use crate::digest::digest_ref;
use crate::digest::domain_digest;
use crate::digest::validate_blake3;
use crate::digest::validate_identifier;
use crate::digest::validate_relative_path;
use crate::digest::validate_store_path;
use crate::digest::validate_typed_ref;
use crate::model::ActionNode;
use crate::model::ActionStage;
use crate::model::AddressingMode;
use crate::model::DeclaredSourceInput;
use crate::model::DynamicDerivation;
use crate::model::DynamicInput;
use crate::model::DynamicUnit;
use crate::model::DynamicUnitPolicy;
use crate::model::HardwarePlan;
use crate::model::HardwarePlanRequest;
use crate::model::INHERIT_POLICY;
use crate::model::MANTLE_PLAN_SCHEMA;
use crate::model::MantlePlanV1;
use crate::model::NO_HOST_PATHS_POLICY;
use crate::model::PlanProducer;
use crate::model::SYSTEM_X86_64_LINUX;
use crate::model::SandboxMode;

const PLAN_DOMAIN: &[u8] = b"mantle.hardware.plan.v1";
const ACTION_DOMAIN: &[u8] = b"mantle.hardware.action.v1";
const GENERATION_UNIT_ID: &str = "generation.selected-top";
const GENERATED_SOURCE_ID: &str = "generated.sources";
const LINK_UNIT_ID: &str = "link.simulator";
const OUTPUT_NAME: &str = "out";
const SHELL_COMMAND_FLAG: &str = "-c";
const MAX_ENV_KEY_BYTES: u32 = 128;
const MAX_TOOL_CLOSURE_PATHS: u32 = 256;
const MAX_VALIDATION_DIAGNOSTICS: usize = 128;

struct DynamicUnitInput<'a> {
    id: &'a str,
    name: &'a str,
    builder: &'a str,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    inputs: Vec<DynamicInput>,
}

pub fn build_hardware_plan(request: HardwarePlanRequest) -> Result<HardwarePlan, Vec<String>> {
    let mut diagnostics = validate_request(&request);
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }
    let generation_action = generation_action(&request)?;
    let mut action_graph = vec![generation_action.clone()];
    let compile = build_compile_units(&request, &generation_action, &mut action_graph)?;
    let link = build_link_unit(&request, &compile, &mut action_graph)?;
    let smoke = build_smoke_units(&request, &link, &mut action_graph)?;
    let mut units = compile;
    units.push(link);
    units.extend(smoke);
    let mut roots = vec![String::from(LINK_UNIT_ID)];
    roots.extend(request.smoke_cases.iter().map(|case| smoke_unit_id(&case.id)));
    roots.sort();
    let plan = MantlePlanV1 {
        schema: String::from(MANTLE_PLAN_SCHEMA),
        producer: PlanProducer {
            logical_name: String::from("hardware-simulation-generation"),
            goal_hint: Some(String::from("selected-target")),
        },
        sources: vec![DeclaredSourceInput {
            id: String::from(GENERATED_SOURCE_ID),
            path: request.generated_source_path.clone(),
            nar_blake3: Some(request.generated_source_blake3.clone()),
        }],
        units,
        roots,
        provenance: BTreeMap::from([
            (String::from("cohort_ref"), request.cohort_ref.clone()),
            (String::from("profile_ref"), request.profile_ref.clone()),
            (String::from("seed_boundary"), String::from("nix-produced-open-source-tool-cohort")),
        ]),
    };
    diagnostics = validate_hardware_plan(&plan, &request);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let canonical_bytes = serde_json::to_vec(&plan).map_err(|error| vec![format!("plan-json:{error}")])?;
    let plan_blake3 = domain_digest(PLAN_DOMAIN, &plan).map_err(|error| vec![String::from(error.code())])?;
    debug_assert!(!canonical_bytes.is_empty());
    debug_assert_eq!(action_graph.len(), plan.units.len().saturating_add(1));
    Ok(HardwarePlan {
        plan,
        canonical_bytes,
        plan_blake3,
        action_graph,
    })
}

pub fn validate_hardware_plan(plan: &MantlePlanV1, request: &HardwarePlanRequest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if plan.schema != MANTLE_PLAN_SCHEMA {
        diagnostics.push(String::from("plan-schema-unsupported"));
    }
    validate_plan_count(plan.units.len(), request.bounds.max_actions, "plan-action", &mut diagnostics);
    validate_plan_bytes(plan, request.bounds.max_plan_bytes, &mut diagnostics);
    validate_plan_sources(plan, request, &mut diagnostics);
    validate_plan_units(plan, request, &mut diagnostics);
    validate_plan_graph(plan, request, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.len() > MAX_VALIDATION_DIAGNOSTICS {
        diagnostics.truncate(MAX_VALIDATION_DIAGNOSTICS);
        diagnostics.push(String::from("plan-diagnostic-limit-exceeded"));
    }
    debug_assert!(diagnostics.len() <= MAX_VALIDATION_DIAGNOSTICS.saturating_add(1));
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
    diagnostics
}

pub fn validate_generated_files(plan: &MantlePlanV1, observed_relative_paths: Vec<String>) -> Vec<String> {
    let observed = observed_relative_paths.into_iter().collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::with_capacity(plan.units.len().min(MAX_VALIDATION_DIAGNOSTICS));
    for unit in plan.units.iter().filter(|unit| unit.id.starts_with("compile.")) {
        let Some(relative_path) = unit.derivation.env.get("SOURCE_RELATIVE_PATH") else {
            diagnostics.push(String::from("compile-source-path-missing"));
            continue;
        };
        if !observed.contains(relative_path) {
            diagnostics.push(String::from("generated-file-missing"));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
    debug_assert!(diagnostics.len() <= plan.units.len());
    diagnostics
}

fn validate_request(request: &HardwarePlanRequest) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(MAX_VALIDATION_DIAGNOSTICS);
    validate_request_identity(request, &mut diagnostics);
    validate_request_tool_paths(request, &mut diagnostics);
    validate_tool_closure(request, &mut diagnostics);
    validate_runtime_library_paths(request, &mut diagnostics);
    validate_plan_count(
        request.generated_units.len(),
        request.bounds.max_generated_units,
        "generated-unit",
        &mut diagnostics,
    );
    validate_plan_count(request.smoke_cases.len(), request.bounds.max_smoke_cases, "smoke-case", &mut diagnostics);
    let total_action_count = request.generated_units.len().saturating_add(request.smoke_cases.len()).saturating_add(1);
    validate_plan_count(total_action_count, request.bounds.max_actions, "plan-action", &mut diagnostics);
    validate_generated_unit_requests(request, &mut diagnostics);
    debug_assert!(total_action_count >= request.generated_units.len());
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
    diagnostics
}

fn validate_request_identity(request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    push_result(
        diagnostics,
        validate_typed_ref(TypedRefValidation {
            value: &request.profile_ref,
            prefix: PROFILE_REF_PREFIX,
            field: "profile-ref",
        }),
    );
    push_result(
        diagnostics,
        validate_typed_ref(TypedRefValidation {
            value: &request.cohort_ref,
            prefix: COHORT_REF_PREFIX,
            field: "cohort-ref",
        }),
    );
    for source_ref in &request.source_refs {
        push_result(
            diagnostics,
            validate_typed_ref(TypedRefValidation {
                value: source_ref,
                prefix: OBJECT_REF_PREFIX,
                field: "source-ref",
            }),
        );
    }
    push_result(
        diagnostics,
        validate_store_path(NamedValue {
            value: &request.generated_source_path,
            field: "generated-source-path",
        }),
    );
    push_result(
        diagnostics,
        validate_blake3(NamedValue {
            value: &request.generated_source_blake3,
            field: "generated-source",
        }),
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_request_tool_paths(request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    for path in [
        &request.shell_builder,
        &request.cxx_executable,
        &request.linker_executable,
        &request.runtime_support_path,
    ] {
        push_result(
            diagnostics,
            validate_store_path(NamedValue {
                value: path,
                field: "tool-path",
            }),
        );
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_generated_unit_requests(request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    let mut unit_ids = BTreeSet::new();
    for unit in &request.generated_units {
        if !unit_ids.insert(unit.id.as_str()) {
            diagnostics.push(String::from("duplicate-generated-unit"));
        }
        push_result(
            diagnostics,
            validate_identifier(BoundedValue {
                value: &unit.id,
                field: "generated-unit-id",
                maximum_bytes: request.bounds.max_text_bytes,
            }),
        );
        push_result(
            diagnostics,
            validate_relative_path(BoundedValue {
                value: &unit.relative_path,
                field: "generated-unit-path",
                maximum_bytes: request.bounds.max_text_bytes,
            }),
        );
        push_result(
            diagnostics,
            validate_blake3(NamedValue {
                value: &unit.source_blake3,
                field: "generated-unit-source",
            }),
        );
    }
    debug_assert!(unit_ids.len() <= request.generated_units.len());
    debug_assert!(diagnostics.len() >= diagnostics_before);
}

fn generation_action(request: &HardwarePlanRequest) -> Result<ActionNode, Vec<String>> {
    let direct_input_refs = canonical_refs(
        request
            .source_refs
            .iter()
            .cloned()
            .chain([request.profile_ref.clone(), request.cohort_ref.clone()])
            .collect(),
    );
    let action_ref = action_ref(ActionStage::Generation, GENERATION_UNIT_ID, &direct_input_refs, &[])?;
    Ok(ActionNode {
        action_ref,
        unit_id: String::from(GENERATION_UNIT_ID),
        stage: ActionStage::Generation,
        direct_input_refs,
        dependency_action_refs: Vec::new(),
    })
}

fn build_compile_units(
    request: &HardwarePlanRequest,
    generation: &ActionNode,
    action_graph: &mut Vec<ActionNode>,
) -> Result<Vec<DynamicUnit>, Vec<String>> {
    let mut units = Vec::with_capacity(request.generated_units.len());
    for source in &request.generated_units {
        let unit_id = compile_unit_id(&source.id);
        let direct = canonical_refs(vec![
            request.profile_ref.clone(),
            request.cohort_ref.clone(),
            format!("{OBJECT_REF_PREFIX}{}", source.source_blake3),
        ]);
        let dependency_refs = vec![generation.action_ref.clone()];
        let action_ref = action_ref(ActionStage::Compile, &unit_id, &direct, &dependency_refs)?;
        let source_env = String::from("{{mantle-source:generated.sources}}");
        let command = compile_command(request, source);
        let env = common_env(request, &action_ref)
            .into_iter()
            .chain([
                (String::from("GENERATED_ROOT"), source_env),
                (String::from("SOURCE_RELATIVE_PATH"), source.relative_path.clone()),
            ])
            .collect();
        let mut inputs = vec![DynamicInput::Source {
            source: String::from(GENERATED_SOURCE_ID),
        }];
        inputs.extend(tool_closure_inputs(request));
        units.push(dynamic_unit(DynamicUnitInput {
            id: &unit_id,
            name: &format!("hardware-{}", sanitize_name(&source.id)),
            builder: &request.shell_builder,
            args: vec![String::from(SHELL_COMMAND_FLAG), command],
            env,
            inputs,
        }));
        action_graph.push(ActionNode {
            action_ref,
            unit_id,
            stage: ActionStage::Compile,
            direct_input_refs: direct,
            dependency_action_refs: dependency_refs,
        });
    }
    debug_assert_eq!(units.len(), request.generated_units.len());
    debug_assert!(units.iter().all(|unit| unit.id.starts_with("compile.")));
    Ok(units)
}

fn build_link_unit(
    request: &HardwarePlanRequest,
    compile_units: &[DynamicUnit],
    action_graph: &mut Vec<ActionNode>,
) -> Result<DynamicUnit, Vec<String>> {
    let compile_actions = action_graph
        .iter()
        .filter(|node| node.stage == ActionStage::Compile)
        .map(|node| node.action_ref.clone())
        .collect::<Vec<_>>();
    let direct = canonical_refs(vec![request.profile_ref.clone(), request.cohort_ref.clone()]);
    let action_ref = action_ref(ActionStage::Link, LINK_UNIT_ID, &direct, &compile_actions)?;
    let mut inputs = Vec::with_capacity(compile_units.len().saturating_add(request.tool_closure_paths.len()));
    let mut env = common_env(request, &action_ref);
    let mut object_variables = Vec::with_capacity(compile_units.len());
    for (index, unit) in compile_units.iter().enumerate() {
        let key = format!("OBJECT_{index}");
        env.insert(key.clone(), format!("{{{{mantle-unit-output:{}:{OUTPUT_NAME}}}}}", unit.id));
        object_variables.push(format!("\"${key}\""));
        inputs.push(DynamicInput::UnitOutput {
            unit: unit.id.clone(),
            output: String::from(OUTPUT_NAME),
        });
    }
    inputs.extend(tool_closure_inputs(request));
    let command = link_command(request, &object_variables);
    let unit = dynamic_unit(DynamicUnitInput {
        id: LINK_UNIT_ID,
        name: "hardware-simulator",
        builder: &request.shell_builder,
        args: vec![String::from(SHELL_COMMAND_FLAG), command],
        env,
        inputs,
    });
    action_graph.push(ActionNode {
        action_ref,
        unit_id: String::from(LINK_UNIT_ID),
        stage: ActionStage::Link,
        direct_input_refs: direct,
        dependency_action_refs: compile_actions,
    });
    debug_assert_eq!(unit.id, LINK_UNIT_ID);
    debug_assert!(!unit.derivation.inputs.is_empty());
    Ok(unit)
}

fn build_smoke_units(
    request: &HardwarePlanRequest,
    link_unit: &DynamicUnit,
    action_graph: &mut Vec<ActionNode>,
) -> Result<Vec<DynamicUnit>, Vec<String>> {
    let link_action = action_graph
        .iter()
        .find(|node| node.stage == ActionStage::Link)
        .map(|node| node.action_ref.clone())
        .ok_or_else(|| vec![String::from("link-action-missing")])?;
    let mut units = Vec::with_capacity(request.smoke_cases.len());
    for case in &request.smoke_cases {
        let unit_id = smoke_unit_id(&case.id);
        let direct = canonical_refs(vec![request.profile_ref.clone(), request.cohort_ref.clone()]);
        let dependencies = vec![link_action.clone()];
        let action_ref = action_ref(ActionStage::Smoke, &unit_id, &direct, &dependencies)?;
        let simulator_ref =
            digest_ref(crate::digest::SIMULATOR_REF_PREFIX, b"mantle.hardware.simulator.v1", &link_action)
                .map_err(|error| vec![String::from(error.code())])?;
        let smoke_result = crate::smoke::successful_smoke_result(crate::smoke::SuccessfulSmokeResultInput {
            case,
            simulator_ref: simulator_ref.clone(),
            action_ref: action_ref.clone(),
            profile_ref: request.profile_ref.clone(),
            cohort_ref: request.cohort_ref.clone(),
            source_refs: request.source_refs.clone(),
            maximum_text_bytes: request.bounds.max_text_bytes,
        })?;
        let smoke_result_json =
            serde_json::to_string(&smoke_result).map_err(|error| vec![format!("smoke-result-json:{error}")])?;
        let mut env = common_env(request, &action_ref);
        env.extend([
            (String::from("SIMULATOR"), format!("{{{{mantle-unit-output:{}:{OUTPUT_NAME}}}}}", link_unit.id)),
            (String::from("SIMULATOR_REF"), simulator_ref),
            (String::from("SMOKE_CASE_ID"), case.id.clone()),
            (String::from("SMOKE_INPUT"), case.input.clone()),
            (String::from("SMOKE_EXPECTED"), case.expected.clone()),
            (String::from("SMOKE_RESULT_JSON"), smoke_result_json),
            (String::from("LD_LIBRARY_PATH"), request.runtime_library_paths.join(":")),
        ]);
        let mut inputs = vec![DynamicInput::UnitOutput {
            unit: link_unit.id.clone(),
            output: String::from(OUTPUT_NAME),
        }];
        inputs.extend(tool_closure_inputs(request));
        let unit = dynamic_unit(DynamicUnitInput {
            id: &unit_id,
            name: &format!("hardware-smoke-{}", sanitize_name(&case.id)),
            builder: &request.shell_builder,
            args: vec![String::from(SHELL_COMMAND_FLAG), smoke_command()],
            env,
            inputs,
        });
        units.push(unit);
        action_graph.push(ActionNode {
            action_ref,
            unit_id,
            stage: ActionStage::Smoke,
            direct_input_refs: direct,
            dependency_action_refs: dependencies,
        });
    }
    debug_assert_eq!(units.len(), request.smoke_cases.len());
    debug_assert!(units.iter().all(|unit| unit.id.starts_with("smoke.")));
    Ok(units)
}

fn dynamic_unit(input: DynamicUnitInput<'_>) -> DynamicUnit {
    DynamicUnit {
        id: String::from(input.id),
        derivation: DynamicDerivation {
            name: String::from(input.name),
            builder: String::from(input.builder),
            system: String::from(SYSTEM_X86_64_LINUX),
            args: input.args,
            outputs: vec![String::from(OUTPUT_NAME)],
            env: input.env,
            inputs: input.inputs,
            fixed_output: None,
            addressing_mode: AddressingMode::InputAddressed,
            sandbox: SandboxMode::Native,
            dynamic_plan_outputs: Vec::new(),
        },
        requested_outputs: vec![String::from(OUTPUT_NAME)],
        policy: DynamicUnitPolicy {
            sandbox: String::from(INHERIT_POLICY),
            substitutions: String::from(INHERIT_POLICY),
            store_prefix: String::from(INHERIT_POLICY),
            host_paths: String::from(NO_HOST_PATHS_POLICY),
        },
    }
}

fn common_env(request: &HardwarePlanRequest, action_ref: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (String::from("HARDWARE_ACTION_REF"), String::from(action_ref)),
        (String::from("HARDWARE_COHORT_REF"), request.cohort_ref.clone()),
        (String::from("HARDWARE_PROFILE_REF"), request.profile_ref.clone()),
    ])
}

fn compile_command(request: &HardwarePlanRequest, source: &crate::model::GeneratedTranslationUnit) -> String {
    let flags = request.compile_flags.join(" ");
    format!(
        "set -eu; exec {} {} -I\"$GENERATED_ROOT\" -I{}/share/verilator/include -c \"$GENERATED_ROOT/{}\" -o \"$out\"",
        request.cxx_executable, flags, request.runtime_support_path, source.relative_path
    )
}

fn link_command(request: &HardwarePlanRequest, objects: &[String]) -> String {
    let flags = request.link_flags.join(" ");
    format!(
        "set -eu; exec {} -fuse-ld={} {} {} -o \"$out\"",
        request.cxx_executable,
        request.linker_executable,
        flags,
        objects.join(" ")
    )
}

fn smoke_command() -> String {
    String::from(
        "set -eu; stderr_file=./smoke.stderr; : > \"$stderr_file\"; set +e; observed=\"$($SIMULATOR \"$SMOKE_INPUT\" 2>\"$stderr_file\")\"; status=$?; set -e; if test \"$status\" -ne 0; then while IFS= read -r line; do printf '%s\\n' \"$line\" >&2; done < \"$stderr_file\"; exit \"$status\"; fi; if test -s \"$stderr_file\"; then while IFS= read -r line; do printf '%s\\n' \"$line\" >&2; done < \"$stderr_file\"; exit 1; fi; test \"$observed\" = \"$SMOKE_EXPECTED\"; printf '%s\\n' \"$SMOKE_RESULT_JSON\" > \"$out\"",
    )
}

fn validate_plan_sources(plan: &MantlePlanV1, request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    if plan.sources.len() != 1 {
        diagnostics.push(String::from("plan-generated-source-count-invalid"));
        return;
    }
    let source = &plan.sources[0];
    if source.id != GENERATED_SOURCE_ID || source.path != request.generated_source_path {
        diagnostics.push(String::from("plan-generated-source-mismatch"));
    }
    if source.nar_blake3.as_deref() != Some(request.generated_source_blake3.as_str()) {
        diagnostics.push(String::from("plan-generated-source-digest-mismatch"));
    }
}

fn validate_plan_units(plan: &MantlePlanV1, request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    let mut unit_ids = BTreeSet::new();
    let expected_compile_paths =
        request.generated_units.iter().map(|unit| unit.relative_path.as_str()).collect::<BTreeSet<_>>();
    let mut observed_compile_paths = BTreeSet::new();
    let mut link_count = 0_u32;
    let mut smoke_count = 0_u32;
    for unit in &plan.units {
        if !unit_ids.insert(unit.id.as_str()) {
            diagnostics.push(String::from("duplicate-plan-unit"));
        }
        validate_unit_common(unit, request, diagnostics);
        if unit.id.starts_with("compile.") {
            if let Some(path) = unit.derivation.env.get("SOURCE_RELATIVE_PATH") {
                observed_compile_paths.insert(path.as_str());
            }
            validate_compile_unit(unit, request, diagnostics);
        } else if unit.id == LINK_UNIT_ID {
            link_count = link_count.saturating_add(1);
            validate_link_unit(unit, request, diagnostics);
        } else if unit.id.starts_with("smoke.") {
            smoke_count = smoke_count.saturating_add(1);
            validate_smoke_unit(unit, diagnostics);
        } else {
            diagnostics.push(String::from("unsupported-plan-command"));
        }
    }
    if observed_compile_paths != expected_compile_paths {
        diagnostics.push(String::from("plan-generated-unit-set-mismatch"));
    }
    if link_count != 1 {
        diagnostics.push(String::from("plan-link-root-count-invalid"));
    }
    let is_smoke_count_valid =
        u32::try_from(request.smoke_cases.len()).is_ok_and(|expected_smoke_count| smoke_count == expected_smoke_count);
    if !is_smoke_count_valid {
        diagnostics.push(String::from("plan-smoke-count-invalid"));
    }
    debug_assert!(unit_ids.len() <= plan.units.len());
    debug_assert!(diagnostics.len() >= diagnostics_before);
}

fn validate_unit_common(unit: &DynamicUnit, request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    let expected_outputs = vec![String::from(OUTPUT_NAME)];
    let is_builder_valid = unit.derivation.builder == request.shell_builder;
    let is_system_valid = unit.derivation.system == SYSTEM_X86_64_LINUX;
    let is_derivation_output_valid = unit.derivation.outputs == expected_outputs;
    let is_requested_output_valid = unit.requested_outputs == expected_outputs;
    let is_host_path_policy_valid = unit.policy.host_paths == NO_HOST_PATHS_POLICY;
    let is_unit_policy_valid = [
        is_builder_valid,
        is_system_valid,
        is_derivation_output_valid,
        is_requested_output_valid,
        is_host_path_policy_valid,
    ]
    .into_iter()
    .all(|is_valid| is_valid);
    if !is_unit_policy_valid {
        diagnostics.push(String::from("plan-unit-policy-invalid"));
    }
    if unit.derivation.args.first().map(String::as_str) != Some(SHELL_COMMAND_FLAG) {
        diagnostics.push(String::from("unsupported-plan-command"));
    }
    for (key, value) in &unit.derivation.env {
        let normalized_key = key.to_ascii_lowercase().replace('_', "-");
        push_result(
            diagnostics,
            validate_identifier(BoundedValue {
                value: &normalized_key,
                field: "plan-env-key",
                maximum_bytes: MAX_ENV_KEY_BYTES,
            }),
        );
        let maximum_bytes = if key == "SMOKE_RESULT_JSON" {
            request.bounds.max_plan_bytes
        } else {
            request.bounds.max_text_bytes
        };
        let is_value_within_limit_bytes =
            u32::try_from(value.len()).is_ok_and(|value_bytes| value_bytes <= maximum_bytes);
        if !is_value_within_limit_bytes {
            diagnostics.push(String::from("plan-env-value-too-large"));
        }
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_compile_unit(unit: &DynamicUnit, request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    let command = unit.derivation.args.get(1).map(String::as_str).unwrap_or_default();
    let is_cxx_declared = command.contains(&request.cxx_executable);
    let is_generated_root_declared = command.contains("$GENERATED_ROOT");
    let is_compile_mode_declared = command.contains(" -c ");
    let is_generated_source_declared = unit
        .derivation
        .inputs
        .iter()
        .any(|input| matches!(input, DynamicInput::Source { source } if source == GENERATED_SOURCE_ID));
    let is_compile_command_accepted = [
        is_cxx_declared,
        is_generated_root_declared,
        is_compile_mode_declared,
        is_generated_source_declared,
    ]
    .into_iter()
    .all(|is_supported| is_supported);
    if !is_compile_command_accepted {
        diagnostics.push(String::from("unsupported-compile-command"));
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_link_unit(unit: &DynamicUnit, request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let command = unit.derivation.args.get(1).map(String::as_str).unwrap_or_default();
    let dependency_count = unit
        .derivation
        .inputs
        .iter()
        .filter(|input| matches!(input, DynamicInput::UnitOutput { .. }))
        .count();
    if !command.contains(&request.cxx_executable)
        || !command.contains(&request.linker_executable)
        || dependency_count != request.generated_units.len()
    {
        diagnostics.push(String::from("unsupported-link-command"));
    }
}

fn validate_smoke_unit(unit: &DynamicUnit, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    let command = unit.derivation.args.get(1).map(String::as_str).unwrap_or_default();
    let dependency_count = unit
        .derivation
        .inputs
        .iter()
        .filter(|input| matches!(input, DynamicInput::UnitOutput { unit, .. } if unit == LINK_UNIT_ID))
        .count();
    let is_result_schema_present = unit
        .derivation
        .env
        .get("SMOKE_RESULT_JSON")
        .is_some_and(|value| value.contains(crate::model::SMOKE_RESULT_SCHEMA));
    let is_runtime_libraries_declared =
        unit.derivation.env.get("LD_LIBRARY_PATH").is_some_and(|value| !value.is_empty());
    let is_simulator_declared = command.contains("$SIMULATOR");
    let is_expected_value_declared = command.contains("$SMOKE_EXPECTED");
    let is_result_json_declared = command.contains("$SMOKE_RESULT_JSON");
    let is_dependency_count_valid = dependency_count == 1;
    let is_smoke_command_accepted = [
        is_simulator_declared,
        is_expected_value_declared,
        is_result_json_declared,
        is_result_schema_present,
        is_runtime_libraries_declared,
        is_dependency_count_valid,
    ]
    .into_iter()
    .all(|is_supported| is_supported);
    if !is_smoke_command_accepted {
        diagnostics.push(String::from("unsupported-smoke-command"));
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_plan_graph(plan: &MantlePlanV1, request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    let units = plan.units.iter().map(|unit| unit.id.as_str()).collect::<BTreeSet<_>>();
    let roots = plan.roots.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_root_count = request.smoke_cases.len().saturating_add(1);
    let smoke_root_count = roots.iter().filter(|root| root.starts_with("smoke.")).count();
    if roots.len() != expected_root_count
        || smoke_root_count != request.smoke_cases.len()
        || !roots.contains(LINK_UNIT_ID)
    {
        diagnostics.push(String::from("plan-root-set-invalid"));
    }
    for unit in &plan.units {
        for input in &unit.derivation.inputs {
            if let DynamicInput::UnitOutput { unit, output } = input
                && (!units.contains(unit.as_str()) || output != OUTPUT_NAME)
            {
                diagnostics.push(String::from("plan-unit-edge-undeclared"));
            }
        }
    }
    if plan.units.iter().any(|unit| unit.derivation.outputs.iter().any(|output| output != OUTPUT_NAME)) {
        diagnostics.push(String::from("plan-undeclared-output"));
    }
    debug_assert!(roots.len() <= plan.roots.len());
    debug_assert!(diagnostics.len() >= diagnostics_before);
}

fn validate_plan_bytes(plan: &MantlePlanV1, maximum_bytes: u32, diagnostics: &mut Vec<String>) {
    match serde_json::to_vec(plan) {
        Ok(bytes) => match u32::try_from(bytes.len()) {
            Ok(actual_bytes) if actual_bytes <= maximum_bytes => {}
            Ok(_) | Err(_) => diagnostics.push(String::from("plan-byte-bound-exceeded")),
        },
        Err(_) => diagnostics.push(String::from("plan-json-invalid")),
    }
}

fn validate_plan_count(actual: usize, maximum: u32, field: &str, diagnostics: &mut Vec<String>) {
    let is_count_valid = u32::try_from(actual).is_ok_and(|actual_count| actual_count > 0 && actual_count <= maximum);
    if !is_count_valid {
        diagnostics.push(format!("{field}-count-invalid"));
    }
}

fn action_ref(
    stage: ActionStage,
    unit_id: &str,
    direct_input_refs: &[String],
    dependency_action_refs: &[String],
) -> Result<String, Vec<String>> {
    let hashable = (stage, unit_id, direct_input_refs, dependency_action_refs);
    digest_ref(ACTION_REF_PREFIX, ACTION_DOMAIN, &hashable).map_err(|error| vec![String::from(error.code())])
}

fn canonical_refs(values: Vec<String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn validate_tool_closure(request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    validate_plan_count(request.tool_closure_paths.len(), MAX_TOOL_CLOSURE_PATHS, "tool-closure-path", diagnostics);
    let roots = request.tool_closure_paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if roots.len() != request.tool_closure_paths.len() {
        diagnostics.push(String::from("tool-closure-path-duplicate"));
    }
    for path in &request.tool_closure_paths {
        push_result(
            diagnostics,
            validate_store_path(NamedValue {
                value: path,
                field: "tool-closure-path",
            }),
        );
        if store_root(path).as_deref() != Ok(path.as_str()) {
            diagnostics.push(String::from("tool-closure-path-not-root"));
        }
    }
    for executable in [
        &request.shell_builder,
        &request.cxx_executable,
        &request.linker_executable,
        &request.runtime_support_path,
    ] {
        if let Ok(root) = store_root(executable)
            && !roots.contains(root.as_str())
        {
            diagnostics.push(String::from("tool-closure-required-root-missing"));
        }
    }
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
    debug_assert!(roots.len() <= request.tool_closure_paths.len());
}

fn validate_runtime_library_paths(request: &HardwarePlanRequest, diagnostics: &mut Vec<String>) {
    validate_plan_count(
        request.runtime_library_paths.len(),
        MAX_TOOL_CLOSURE_PATHS,
        "runtime-library-path",
        diagnostics,
    );
    let closure_roots = request.tool_closure_paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut unique_paths = BTreeSet::new();
    for path in &request.runtime_library_paths {
        push_result(
            diagnostics,
            validate_store_path(NamedValue {
                value: path,
                field: "runtime-library-path",
            }),
        );
        if !unique_paths.insert(path.as_str()) {
            diagnostics.push(String::from("runtime-library-path-duplicate"));
        }
        if let Ok(root) = store_root(path)
            && !closure_roots.contains(root.as_str())
        {
            diagnostics.push(String::from("runtime-library-root-missing"));
        }
    }
    debug_assert!(unique_paths.len() <= request.runtime_library_paths.len());
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn tool_closure_inputs(request: &HardwarePlanRequest) -> Vec<DynamicInput> {
    let inputs = request
        .tool_closure_paths
        .iter()
        .cloned()
        .map(|path| DynamicInput::StorePath { path })
        .collect::<Vec<_>>();
    debug_assert_eq!(inputs.len(), request.tool_closure_paths.len());
    debug_assert!(!inputs.is_empty());
    inputs
}

fn store_root(path: &str) -> Result<String, Vec<String>> {
    const STORE_PREFIX: &str = "/nix/store/";
    let component = path
        .strip_prefix(STORE_PREFIX)
        .and_then(|suffix| suffix.split('/').next())
        .filter(|component| !component.is_empty())
        .ok_or_else(|| vec![String::from("tool-store-root-invalid")])?;
    let root = format!("{STORE_PREFIX}{component}");
    debug_assert!(path.starts_with(&root));
    debug_assert_eq!(root.matches('/').count(), 3);
    Ok(root)
}

fn compile_unit_id(id: &str) -> String {
    format!("compile.{id}")
}

fn smoke_unit_id(id: &str) -> String {
    format!("smoke.{id}")
}

fn sanitize_name(value: &str) -> String {
    value.replace(['.', '_'], "-")
}

fn push_result<E: core::fmt::Display>(diagnostics: &mut Vec<String>, result: Result<(), E>) {
    if let Err(error) = result {
        diagnostics.push(alloc::format!("{error}"));
    }
}
