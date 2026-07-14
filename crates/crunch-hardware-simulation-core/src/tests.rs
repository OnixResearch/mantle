use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const REVISION_A: &str = "1111111111111111111111111111111111111111";
const REVISION_B: &str = "2222222222222222222222222222222222222222";
const REVISION_C: &str = "3333333333333333333333333333333333333333";
const STORE_A: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool-a";
const STORE_B: &str = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-tool-b";
const STORE_C: &str = "/nix/store/cccccccccccccccccccccccccccccccc-tool-c";
const STORE_D: &str = "/nix/store/dddddddddddddddddddddddddddddddd-tool-d";
const STORE_E: &str = "/nix/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-tool-e";
const STORE_GEN: &str = "/nix/store/ffffffffffffffffffffffffffffffff-generated";
const TEXT_BOUND: u32 = 256;
const PLAN_BOUND: u32 = 262_144;
const LOG_BOUND: u32 = 4_096;
const ACTION_BOUND: u32 = 32;
const EXPECTED_ACTION_COUNT: usize = 6;
const WRONG_EXIT_CODE: i32 = 1;

fn object_ref(seed: &str) -> String {
    format!("{OBJECT_REF_PREFIX}{}", blake3::hash(seed.as_bytes()).to_hex())
}

fn typed_ref(prefix: &str, seed: &str) -> String {
    format!("{prefix}{}", blake3::hash(seed.as_bytes()).to_hex())
}

fn tool(role: ToolRole, package: &str, store_path: &str, executable: &str) -> ToolMember {
    ToolMember {
        role,
        package: String::from(package),
        version: String::from("1.0"),
        store_path: String::from(store_path),
        executable: String::from(executable),
        binary_blake3: String::from(DIGEST_A),
    }
}

fn tool_cohort() -> ToolCohort {
    let mut cohort = ToolCohort {
        schema: String::from(TOOL_COHORT_SCHEMA),
        identity_blake3: String::from(DIGEST_A),
        seed_boundary: String::from(NIX_SEED_BOUNDARY),
        system: String::from(SYSTEM_X86_64_LINUX),
        members: vec![
            tool(ToolRole::Verilator, "verilator", STORE_A, "bin/verilator"),
            tool(ToolRole::CxxCompiler, "clang", STORE_B, "bin/clang++"),
            tool(ToolRole::Linker, "lld", STORE_C, "bin/ld.lld"),
            tool(ToolRole::RuntimeSupport, "verilator-runtime", STORE_D, "share/verilator/include/verilated.cpp"),
            tool(ToolRole::Shell, "bash", STORE_E, "bin/bash"),
        ],
        closure_paths_blake3: String::from(DIGEST_B),
    };
    cohort.identity_blake3 = cohort_ref(&cohort).unwrap().strip_prefix(COHORT_REF_PREFIX).unwrap().to_string();
    cohort
}

fn source(id: &str, revision: &str, digest: &str, dependencies: Vec<&str>, files: Vec<&str>) -> SourcePackage {
    SourcePackage {
        id: String::from(id),
        locator: format!("fixture-git://{id}"),
        revision: String::from(revision),
        recursive_blake3: String::from(digest),
        object_ref: object_ref(id),
        sentinel_blake3: String::from(digest),
        dependencies: dependencies.into_iter().map(String::from).collect(),
        files: files.into_iter().map(String::from).collect(),
    }
}

fn bounds() -> HardwareBounds {
    HardwareBounds {
        max_source_packages: 8,
        max_source_files: 16,
        max_generated_units: 8,
        max_options: 8,
        max_smoke_cases: 4,
        max_outputs: 8,
        max_text_bytes: TEXT_BOUND,
        max_plan_bytes: PLAN_BOUND,
        max_log_bytes: LOG_BOUND,
        max_actions: ACTION_BOUND,
    }
}

fn required_non_claims() -> Vec<String> {
    REQUIRED_NON_CLAIMS.iter().map(|value| String::from(*value)).collect()
}

fn profile() -> HardwareProfile {
    HardwareProfile {
        schema: String::from(HARDWARE_PROFILE_SCHEMA),
        profile_id: String::from("tiny-adder-verilator"),
        selected_target: String::from("tiny-adder-smoke"),
        source_packages: vec![
            source("selected-ip", REVISION_A, DIGEST_A, Vec::new(), vec!["rtl/tiny_adder.sv"]),
            source("selected-vip", REVISION_B, DIGEST_B, vec!["selected-ip"], vec![
                "tb/tiny_adder_tb.sv",
                "model/reference.cpp",
            ]),
            source("unrelated-vip", REVISION_C, DIGEST_C, Vec::new(), vec!["rtl/unrelated_counter.sv"]),
        ],
        targets: vec![HardwareTarget {
            id: String::from("tiny-adder-smoke"),
            top_module: String::from("tiny_adder"),
            source_packages: vec![String::from("selected-vip")],
            source_files: vec![
                String::from("selected-ip/rtl/tiny_adder.sv"),
                String::from("selected-vip/tb/tiny_adder_tb.sv"),
            ],
            reference_model: String::from("selected-vip/model/reference.cpp"),
        }],
        tool_cohort: tool_cohort(),
        generation: GenerationOptions {
            language: String::from("systemverilog"),
            trace: false,
            optimization: String::from("O1"),
            output_prefix: String::from("Vtiny_adder"),
            extra_args: vec![String::from("--cc"), String::from("--exe")],
        },
        compile_flags: vec![String::from("-std=c++17"), String::from("-O1")],
        link_flags: vec![String::from("-pthread")],
        smoke_cases: vec![
            SmokeCase {
                id: String::from("zero-plus-zero"),
                input: String::from("0,0"),
                expected: String::from("0"),
                max_log_bytes: LOG_BOUND,
            },
            SmokeCase {
                id: String::from("one-plus-two"),
                input: String::from("1,2"),
                expected: String::from("3"),
                max_log_bytes: LOG_BOUND,
            },
        ],
        expected_outputs: vec![
            ExpectedOutput {
                name: String::from("simulator"),
                kind: String::from("executable"),
            },
            ExpectedOutput {
                name: String::from("smoke-results"),
                kind: String::from("mantle-hardware-smoke-result-v1"),
            },
        ],
        bounds: bounds(),
        support_tier: String::from("heavy-capability-gated"),
        non_claims: required_non_claims(),
    }
}

fn validated_profile() -> ProfileValidation {
    validate_profile(profile()).unwrap()
}

fn generated_units() -> Vec<GeneratedTranslationUnit> {
    vec![
        GeneratedTranslationUnit {
            id: String::from("model"),
            relative_path: String::from("Vtiny_adder.cpp"),
            source_blake3: String::from(DIGEST_A),
        },
        GeneratedTranslationUnit {
            id: String::from("symbols"),
            relative_path: String::from("Vtiny_adder__Syms.cpp"),
            source_blake3: String::from(DIGEST_B),
        },
    ]
}

fn plan_request() -> HardwarePlanRequest {
    let validation = validated_profile();
    HardwarePlanRequest {
        profile_ref: validation.profile_ref,
        cohort_ref: validation.cohort_ref,
        source_refs: validation.selected_source_refs,
        generated_source_path: String::from(STORE_GEN),
        generated_source_blake3: String::from(DIGEST_C),
        shell_builder: format!("{STORE_E}/bin/bash"),
        cxx_executable: format!("{STORE_B}/bin/clang++"),
        linker_executable: format!("{STORE_C}/bin/ld.lld"),
        runtime_support_path: String::from(STORE_D),
        tool_closure_paths: vec![
            String::from(STORE_A),
            String::from(STORE_B),
            String::from(STORE_C),
            String::from(STORE_D),
            String::from(STORE_E),
        ],
        runtime_library_paths: vec![format!("{STORE_D}/lib")],
        compile_flags: vec![String::from("-std=c++17"), String::from("-O1")],
        link_flags: vec![String::from("-pthread")],
        generated_units: generated_units(),
        smoke_cases: profile().smoke_cases,
        bounds: bounds(),
    }
}

fn log_ref(seed: &str, bytes: u32) -> BoundedLogRef {
    BoundedLogRef {
        log_ref: typed_ref(LOG_REF_PREFIX, seed),
        byte_count: bytes,
        truncated: false,
    }
}

fn passing_smoke_result() -> HardwareSmokeResult {
    let validation = validated_profile();
    HardwareSmokeResult {
        schema: String::from(SMOKE_RESULT_SCHEMA),
        case_id: String::from("one-plus-two"),
        input: String::from("1,2"),
        expected: String::from("3"),
        observed: String::from("3"),
        verdict: SmokeVerdict::Pass,
        process_exit_code: 0,
        simulator_ref: typed_ref(SIMULATOR_REF_PREFIX, "simulator"),
        action_ref: typed_ref(ACTION_REF_PREFIX, "smoke"),
        profile_ref: validation.profile_ref,
        cohort_ref: validation.cohort_ref,
        source_refs: validation.selected_source_refs,
        stdout: log_ref("stdout", 1),
        stderr: log_ref("stderr", 0),
        non_claims: required_non_claims(),
    }
}

#[test]
fn equivalent_profiles_normalize_to_one_identity() {
    let left = profile();
    let mut right = left.clone();
    right.source_packages.reverse();
    right.source_packages[0].files.reverse();
    right.targets.reverse();
    right.targets[0].source_files.reverse();
    right.tool_cohort.members.reverse();
    right.smoke_cases.reverse();
    right.expected_outputs.reverse();
    right.non_claims.reverse();

    let left = validate_profile(left).unwrap();
    let right = validate_profile(right).unwrap();

    assert_eq!(left.profile_ref, right.profile_ref);
    assert_eq!(left.normalized, right.normalized);
    assert_eq!(left.selected_source_ids, vec!["selected-ip", "selected-vip"]);
    assert!(!left.selected_source_ids.contains(&String::from("unrelated-vip")));
}

#[test]
fn profile_rejects_unknown_fields_duplicates_escapes_empty_sets_and_bounds() {
    let mut value = serde_json::to_value(profile()).unwrap();
    value.as_object_mut().unwrap().insert(String::from("hdl_magic"), serde_json::Value::Bool(true));
    let decode_error = serde_json::from_value::<HardwareProfile>(value).unwrap_err().to_string();

    let mut invalid = profile();
    invalid.source_packages.push(invalid.source_packages[0].clone());
    invalid.source_packages[0].files[0] = String::from("../escape.sv");
    invalid.targets.clear();
    invalid.bounds.max_generated_units = HARD_MAX_GENERATED_UNITS.saturating_add(1);
    let diagnostics = validate_profile(invalid).unwrap_err();

    assert!(decode_error.contains("unknown field"));
    assert!(diagnostics.contains(&String::from("duplicate-source-package")));
    assert!(diagnostics.contains(&String::from("source-file-not-relative-normal")));
    assert!(diagnostics.contains(&String::from("hardware-targets-empty")));
    assert!(diagnostics.contains(&String::from("generated-units-bound-invalid")));
}

#[test]
fn demand_driven_source_observation_excludes_unrelated_and_rejects_drift_and_edges() {
    let profile = profile();
    let selected = validate_source_observations(&profile, "tiny-adder-smoke", vec![
        observation(&profile.source_packages[0]),
        observation(&profile.source_packages[1]),
        SourceObservation {
            acquired: false,
            ..observation(&profile.source_packages[2])
        },
    ]);
    let mut drifted = observation(&profile.source_packages[1]);
    drifted.revision = String::from(REVISION_C);
    drifted.recursive_blake3 = String::from(DIGEST_C);
    drifted.declared_edges.push(String::from("unrelated-vip"));
    let rejected = validate_source_observations(&profile, "tiny-adder-smoke", vec![
        observation(&profile.source_packages[0]),
        drifted,
    ]);

    assert!(selected.rejected.is_empty());
    assert_eq!(selected.selected_source_ids, vec!["selected-ip", "selected-vip"]);
    assert!(!selected.selected_source_ids.contains(&String::from("unrelated-vip")));
    assert!(rejected.rejected.contains(&String::from("source-revision-drift")));
    assert!(rejected.rejected.contains(&String::from("source-recursive-digest-drift")));
    assert!(rejected.rejected.contains(&String::from("undeclared-source-edge")));
}

#[test]
fn tool_cohort_rejects_ambient_paths_missing_roles_and_identity_drift() {
    let mut invalid = profile();
    invalid.tool_cohort.members[0].store_path = String::from("/usr/bin/verilator");
    invalid.tool_cohort.members.pop();
    invalid.tool_cohort.identity_blake3 = String::from(DIGEST_C);
    let diagnostics = validate_profile(invalid).unwrap_err();

    assert!(diagnostics.contains(&String::from("tool-store-path-not-nix-store")));
    assert!(diagnostics.contains(&String::from("tool-cohort-role-set-invalid")));
    assert!(diagnostics.contains(&String::from("tool-cohort-identity-mismatch")));
    assert!(required_tool_roles().contains(&ToolRole::Verilator));
}

#[test]
fn plan_has_independent_compile_units_one_link_and_parameterized_smoke_roots() {
    let plan = build_hardware_plan(plan_request()).unwrap();
    let compile_count = plan.plan.units.iter().filter(|unit| unit.id.starts_with("compile.")).count();
    let link_count = plan.plan.units.iter().filter(|unit| unit.id == "link.simulator").count();
    let smoke_count = plan.plan.units.iter().filter(|unit| unit.id.starts_with("smoke.")).count();

    assert_eq!(compile_count, generated_units().len());
    assert_eq!(link_count, 1);
    assert_eq!(smoke_count, profile().smoke_cases.len());
    let smoke_results = plan
        .plan
        .units
        .iter()
        .filter(|unit| unit.id.starts_with("smoke."))
        .map(|unit| {
            let json = unit.derivation.env.get("SMOKE_RESULT_JSON").unwrap();
            let result = serde_json::from_str::<HardwareSmokeResult>(json).unwrap();
            let validation = validate_smoke_result(&result, TEXT_BOUND, LOG_BOUND);
            assert!(unit.derivation.args[1].contains("$SMOKE_RESULT_JSON"));
            assert!(validation.valid);
            assert!(validation.publishable);
            result
        })
        .collect::<Vec<_>>();

    assert_eq!(plan.action_graph.len(), EXPECTED_ACTION_COUNT);
    assert_eq!(plan.plan.roots, vec!["link.simulator", "smoke.one-plus-two", "smoke.zero-plus-zero"]);
    assert_eq!(smoke_results.len(), profile().smoke_cases.len());
    assert!(smoke_results.iter().all(|result| result.schema == SMOKE_RESULT_SCHEMA));
    assert!(plan.plan.units.iter().all(|unit| unit.derivation.addressing_mode == AddressingMode::InputAddressed));
    assert!(
        plan.plan
            .units
            .iter()
            .flat_map(|unit| &unit.derivation.inputs)
            .filter_map(|input| match input {
                DynamicInput::StorePath { path } => Some(path),
                _ => None,
            })
            .all(|path| path.matches('/').count() == 3)
    );
}

#[test]
fn plan_rejects_undeclared_output_missing_file_unsupported_command_duplicate_and_overflow() {
    let request = plan_request();
    let mut plan = build_hardware_plan(request.clone()).unwrap().plan;
    plan.units[0].derivation.outputs.push(String::from("escape"));
    plan.units[0].derivation.args[1] = String::from("curl https://example.invalid");
    plan.units.push(plan.units[0].clone());
    let diagnostics = validate_hardware_plan(&plan, &request);
    let missing = validate_generated_files(&plan, vec![String::from("Vtiny_adder.cpp")]);
    let mut overflow = request;
    overflow.bounds.max_generated_units = 1;
    overflow.runtime_library_paths = vec![String::from("/usr/lib"), format!("{STORE_GEN}/lib")];
    let overflow = build_hardware_plan(overflow).unwrap_err();

    assert!(diagnostics.contains(&String::from("plan-undeclared-output")));
    assert!(diagnostics.contains(&String::from("unsupported-compile-command")));
    assert!(diagnostics.contains(&String::from("duplicate-plan-unit")));
    assert!(missing.contains(&String::from("generated-file-missing")));
    assert!(overflow.contains(&String::from("generated-unit-count-invalid")));
    assert!(overflow.contains(&String::from("runtime-library-path-not-nix-store")));
    assert!(overflow.contains(&String::from("runtime-library-root-missing")));
}

#[test]
fn smoke_result_requires_verdict_exit_refs_logs_and_observation_agreement() {
    let passing = validate_smoke_result(&passing_smoke_result(), TEXT_BOUND, LOG_BOUND);
    let mut wrong_model = passing_smoke_result();
    wrong_model.observed = String::from("4");
    wrong_model.verdict = SmokeVerdict::Fail;
    wrong_model.process_exit_code = WRONG_EXIT_CODE;
    let wrong_model = validate_smoke_result(&wrong_model, TEXT_BOUND, LOG_BOUND);
    let mut contradictory = passing_smoke_result();
    contradictory.observed = String::from("4");
    contradictory.stdout.byte_count = LOG_BOUND.saturating_add(1);
    contradictory.action_ref = String::from("stale-ref");
    contradictory.source_refs.push(String::from("stale-source-ref"));
    contradictory.source_refs.push(contradictory.source_refs[0].clone());
    let contradictory = validate_smoke_result(&contradictory, TEXT_BOUND, LOG_BOUND);
    let generated = successful_smoke_result(
        &profile().smoke_cases[0],
        typed_ref(SIMULATOR_REF_PREFIX, "simulator"),
        typed_ref(ACTION_REF_PREFIX, "generated-smoke"),
        validated_profile().profile_ref,
        validated_profile().cohort_ref,
        validated_profile().selected_source_refs,
        TEXT_BOUND,
    )
    .unwrap();

    assert!(passing.valid);
    assert!(passing.publishable);
    assert!(wrong_model.valid);
    assert!(!wrong_model.publishable);
    assert!(!contradictory.valid);
    assert!(!contradictory.publishable);
    assert!(contradictory.diagnostics.contains(&String::from("smoke-pass-observation-mismatch")));
    assert!(contradictory.diagnostics.contains(&String::from("action-ref-prefix-invalid")));
    assert!(contradictory.diagnostics.contains(&String::from("source-ref-prefix-invalid")));
    assert!(contradictory.diagnostics.contains(&String::from("smoke-source-ref-duplicate")));
    assert!(contradictory.diagnostics.contains(&String::from("stdout-byte-bound-exceeded")));
    assert_eq!(generated.stdout.byte_count, 1);
    assert_eq!(generated.stderr.byte_count, 0);
    assert!(generated.stdout.log_ref.starts_with(LOG_REF_PREFIX));
}

#[test]
fn selective_invalidation_and_four_run_evidence_are_count_based_not_elapsed_promises() {
    let plan = build_hardware_plan(plan_request()).unwrap();
    let selected_ref = validated_profile().selected_source_refs[0].clone();
    let invalidated = invalidated_actions(&plan.action_graph, core::slice::from_ref(&selected_ref)).unwrap();
    let all_actions = plan.action_graph.iter().map(|node| node.action_ref.clone()).collect::<Vec<_>>();
    let fresh = run_evidence(RunClass::Fresh, &plan.action_graph, &all_actions, &[], &all_actions, 0, 0);
    let selected = run_evidence(
        RunClass::SelectedSourceChange,
        &plan.action_graph,
        &invalidated,
        &difference(&all_actions, &invalidated),
        &invalidated,
        0,
        0,
    );
    let unrelated = run_evidence(RunClass::UnrelatedSourceChange, &plan.action_graph, &[], &all_actions, &[], 0, 42);
    let shared = run_evidence(RunClass::FullSharedHit, &plan.action_graph, &[], &all_actions, &[], 128, 4_096);
    let bundle =
        build_evidence_bundle(&plan.action_graph, &[selected_ref], fresh, selected, unrelated, shared).unwrap();

    assert_eq!(invalidated.len(), all_actions.len());
    assert!(bundle.unrelated_source_change.invalidated_action_refs.is_empty());
    assert!(bundle.full_shared_hit.counts.values().all(|counts| counts.executed == 0));
    assert!(bundle.full_shared_hit.counts.values().all(|counts| counts.reused == counts.requested));
    assert!(!bundle.full_shared_hit.elapsed_is_gating);
    assert!(bundle.non_claims.contains(&String::from("no-specific-speedup-promise")));
}

#[test]
fn evidence_rejects_partial_counts_unrelated_invalidation_and_elapsed_gates() {
    let plan = build_hardware_plan(plan_request()).unwrap();
    let selected_ref = validated_profile().selected_source_refs[0].clone();
    let all_actions = plan.action_graph.iter().map(|node| node.action_ref.clone()).collect::<Vec<_>>();
    let invalidated = invalidated_actions(&plan.action_graph, core::slice::from_ref(&selected_ref)).unwrap();
    let fresh = run_evidence(RunClass::Fresh, &plan.action_graph, &all_actions, &[], &all_actions, 0, 0);
    let selected = run_evidence(
        RunClass::SelectedSourceChange,
        &plan.action_graph,
        &invalidated,
        &difference(&all_actions, &invalidated),
        &invalidated,
        0,
        0,
    );
    let mut unrelated = run_evidence(RunClass::UnrelatedSourceChange, &plan.action_graph, &[], &all_actions, &[], 0, 0);
    unrelated.elapsed_is_gating = true;
    unrelated.invalidated_action_refs.push(all_actions[0].clone());
    unrelated.counts.get_mut(&ActionStage::Generation).unwrap().invalidated = 1;
    let mut shared = run_evidence(RunClass::FullSharedHit, &plan.action_graph, &[], &all_actions, &[], 1, 1);
    shared.counts.get_mut(&ActionStage::Compile).unwrap().executed = 1;
    let diagnostics =
        build_evidence_bundle(&plan.action_graph, &[selected_ref], fresh, selected, unrelated, shared).unwrap_err();

    assert!(diagnostics.contains(&String::from("elapsed-time-promise-forbidden")));
    assert!(diagnostics.contains(&String::from("unrelated-change-invalidated-selected-graph")));
    assert!(diagnostics.contains(&String::from("run-request-execution-reuse-count-mismatch")));
    assert!(diagnostics.contains(&String::from("full-shared-hit-executed-action")));
}

fn observation(package: &SourcePackage) -> SourceObservation {
    SourceObservation {
        id: package.id.clone(),
        locator: package.locator.clone(),
        revision: package.revision.clone(),
        recursive_blake3: package.recursive_blake3.clone(),
        sentinel_blake3: package.sentinel_blake3.clone(),
        declared_edges: package.dependencies.clone(),
        acquired: true,
    }
}

fn run_evidence(
    run_class: RunClass,
    graph: &[ActionNode],
    executed: &[String],
    reused: &[String],
    invalidated: &[String],
    transferred_bytes: u64,
    reused_bytes: u64,
) -> RunEvidence {
    let validation = validated_profile();
    RunEvidence {
        run_class,
        environment_class: String::from("x86_64-linux-local-strict"),
        profile_ref: validation.profile_ref,
        cohort_ref: validation.cohort_ref,
        source_refs: validation.selected_source_refs,
        action_refs: graph.iter().map(|node| node.action_ref.clone()).collect(),
        counts: stage_counts(graph, executed, reused, invalidated),
        transferred_bytes,
        reused_bytes,
        invalidated_action_refs: invalidated.to_vec(),
        elapsed_diagnostic_ns: Some(1),
        elapsed_is_gating: false,
        non_claims: required_evidence_non_claims(),
    }
}

fn difference(all: &[String], removed: &[String]) -> Vec<String> {
    all.iter().filter(|value| !removed.contains(value)).cloned().collect()
}
