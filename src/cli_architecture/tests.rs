use clap::Parser;

const CLI_PARSE_TEST_STACK_BYTES: usize = 16_777_216;

fn parse_args(arguments: &[&str]) -> crate::Args {
    let owned_arguments: Vec<String> = arguments.iter().map(|argument| (*argument).to_string()).collect();
    std::thread::Builder::new()
        .stack_size(CLI_PARSE_TEST_STACK_BYTES)
        .spawn(move || crate::Args::try_parse_from(owned_arguments).map_err(|error| error.to_string()))
        .expect("start CLI parser test thread")
        .join()
        .expect("join CLI parser test thread")
        .expect("parse CLI fixture")
}

#[test]
fn command_roots_map_to_capability_scoped_families() {
    let cases = [
        ("build", mantle_application_core::CommandFamily::Build),
        ("rust-plan", mantle_application_core::CommandFamily::RustPlan),
        ("remote", mantle_application_core::CommandFamily::Remote),
        ("store", mantle_application_core::CommandFamily::Store),
        ("source", mantle_application_core::CommandFamily::Source),
        ("release", mantle_application_core::CommandFamily::Release),
        ("check", mantle_application_core::CommandFamily::Project),
        ("bootstrap", mantle_application_core::CommandFamily::Bootstrap),
        ("artifact", mantle_application_core::CommandFamily::Artifact),
        ("eval", mantle_application_core::CommandFamily::Evaluation),
        ("run", mantle_application_core::CommandFamily::Package),
        ("doctor", mantle_application_core::CommandFamily::Utility),
    ];
    for (root, expected) in cases {
        assert_eq!(super::inbound_adapter::command_family(root), expected);
    }
    assert_eq!(cases.len(), 12);
}

#[test]
fn run_error_round_trip_preserves_class_message_and_exit() {
    let cases = [
        crate::RunError::Eval("eval-detail".to_string()),
        crate::RunError::Build("build-detail".to_string()),
        crate::RunError::Internal("internal-detail".to_string()),
        crate::RunError::Reported(9),
    ];
    for original in cases {
        let expected_kind = original.kind();
        let expected_message = original.message().to_string();
        let expected_exit = original.exit_code();
        let effect = mantle_application_core::plan_dispatch(mantle_application_core::ApplicationCommand {
            schema: mantle_application_core::APPLICATION_COMMAND_SCHEMA.to_string(),
            family: mantle_application_core::CommandFamily::Build,
            operation: "build".to_string(),
            request_blake3: "a".repeat(mantle_application_core::BLAKE3_HEX_CHARS),
            mutation: mantle_application_core::MutationClass::LocalMutation,
        })
        .unwrap();
        let port = super::operation_adapter::port_error(&effect, original);
        let failure = mantle_application_core::CapabilityFailure {
            family: port.family,
            effect_id_blake3: port.effect_id_blake3,
            class: port.class,
            code: port.code,
            message: port.message,
            reported_exit_code: port.reported_exit_code,
        };
        let restored = super::presentation_adapter::run_error(failure);
        assert_eq!(restored.kind(), expected_kind);
        assert_eq!(restored.message(), expected_message);
        assert_eq!(restored.exit_code(), expected_exit);
    }
}

#[test]
fn mutation_class_keeps_read_only_and_external_operations_separate() {
    let read_only = super::inbound_adapter::mutation_class(mantle_application_core::CommandFamily::Utility, "doctor");
    let external = super::inbound_adapter::mutation_class(mantle_application_core::CommandFamily::Remote, "remote");
    let local = super::inbound_adapter::mutation_class(mantle_application_core::CommandFamily::Build, "build");
    assert_eq!(read_only, mantle_application_core::MutationClass::ReadOnly);
    assert_eq!(external, mantle_application_core::MutationClass::ExternalEffect);
    assert_eq!(local, mantle_application_core::MutationClass::LocalMutation);
}

#[test]
fn inbound_validation_rejects_only_conflicting_machine_output_modes() {
    let conflict = parse_args(&["mantle", "--json", "build", "--evaluation-stream"]);
    let ordinary = parse_args(&["mantle", "build"]);
    let mapped = super::inbound_adapter::application_command(&ordinary);
    let repeated = super::inbound_adapter::application_command(&ordinary);
    assert!(crate::cli_inbound::has_conflicting_machine_output_modes(&conflict));
    assert!(!crate::cli_inbound::has_conflicting_machine_output_modes(&ordinary));
    assert_eq!(mapped, repeated);
    assert_eq!(mapped.family, mantle_application_core::CommandFamily::Build);
    assert_eq!(mapped.operation, "build");
}

#[test]
fn operation_adapter_rejects_a_request_identity_side_channel() {
    let args = parse_args(&["mantle", "build"]);
    let mut wrong_command = super::inbound_adapter::application_command(&args);
    wrong_command.request_blake3 = "b".repeat(mantle_application_core::BLAKE3_HEX_CHARS);
    let effect = mantle_application_core::plan_dispatch(wrong_command).unwrap();
    let ctx = crate::RunContext {
        store: std::path::PathBuf::from("/tmp/mantle-cli-adapter-store"),
        resolved_state_dir: std::path::PathBuf::from("/tmp/mantle-cli-adapter-state"),
        store_prefix: "/mantle/store".to_string(),
        verbose: false,
        json: false,
        base_state_dirs: Vec::new(),
    };
    let mut adapter = super::operation_adapter::CliOperationAdapter { args: &args, ctx: &ctx };
    let error =
        mantle_application::BuildOperationPort::execute_build_operation(&mut adapter, effect.clone()).unwrap_err();
    assert_eq!(error.code, "cli-operation-effect-mismatch");
    assert_eq!(error.family, mantle_application_core::CommandFamily::Build);
    assert_eq!(error.effect_id_blake3, effect.effect_id_blake3);
}

#[test]
fn presentation_rejects_malformed_success_and_failure_outcomes() {
    let malformed_success = mantle_application_core::ApplicationOutcome {
        family: mantle_application_core::CommandFamily::Build,
        operation: "build".to_string(),
        status: mantle_application_core::ApplicationObservationStatus::Succeeded,
        effect_id_blake3: "a".repeat(mantle_application_core::BLAKE3_HEX_CHARS),
        failure: Some(mantle_application_core::CapabilityFailure {
            family: mantle_application_core::CommandFamily::Build,
            effect_id_blake3: "a".repeat(mantle_application_core::BLAKE3_HEX_CHARS),
            class: mantle_application_core::ApplicationErrorClass::Build,
            code: "unexpected".to_string(),
            message: "unexpected".to_string(),
            reported_exit_code: None,
        }),
        non_claim: mantle_application_core::APPLICATION_NON_CLAIM.to_string(),
    };
    let malformed_failure = mantle_application_core::ApplicationOutcome {
        status: mantle_application_core::ApplicationObservationStatus::Failed,
        failure: None,
        ..malformed_success.clone()
    };
    let success_error = super::presentation_adapter::finish(malformed_success).unwrap_err();
    let failure_error = super::presentation_adapter::finish(malformed_failure).unwrap_err();
    assert_eq!(success_error.kind(), "internal");
    assert!(success_error.message().contains("successful application outcome carried a failure"));
    assert_eq!(failure_error.kind(), "internal");
    assert!(failure_error.message().contains("failed application outcome omitted its failure"));
}
