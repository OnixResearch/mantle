pub(crate) fn application_command(args: &crate::Args) -> mantle_application_core::ApplicationCommand {
    let command_root = crate::cli_application::command_root(&args.command);
    let family = command_family(command_root);
    let operation = operation_label(&args.command).to_string();
    let request_blake3 = blake3::hash(format!("{args:?}").as_bytes()).to_hex().to_string();
    debug_assert!(!operation.is_empty());
    debug_assert_eq!(request_blake3.len(), mantle_application_core::BLAKE3_HEX_CHARS);
    mantle_application_core::ApplicationCommand {
        schema: mantle_application_core::APPLICATION_COMMAND_SCHEMA.to_string(),
        family,
        operation,
        request_blake3,
        mutation: mutation_class(family, command_root),
    }
}

fn operation_label(command: &crate::Command) -> &'static str {
    if let crate::Command::Build { plan: true, .. } = command {
        return "build.plan";
    }
    crate::cli_application::command_label(command)
}

pub(super) fn command_family(root: &str) -> mantle_application_core::CommandFamily {
    match root {
        "build" => mantle_application_core::CommandFamily::Build,
        "rust-plan" => mantle_application_core::CommandFamily::RustPlan,
        "remote" | "__remote-secret-worker" => mantle_application_core::CommandFamily::Remote,
        "store" => mantle_application_core::CommandFamily::Store,
        "source" => mantle_application_core::CommandFamily::Source,
        "release" | "attest" | "receipt" => mantle_application_core::CommandFamily::Release,
        "init" | "check" | "show" | "refresh" | "list-stale" | "upgrade" => {
            mantle_application_core::CommandFamily::Project
        }
        "bootstrap" | "stage0-inventory" | "self-build" => mantle_application_core::CommandFamily::Bootstrap,
        "artifact" | "wasm-component" => mantle_application_core::CommandFamily::Artifact,
        "eval" | "export" | "__evaluator-worker" | "__evaluator-worker-fixture" => {
            mantle_application_core::CommandFamily::Evaluation
        }
        "run" | "shell" | "develop" | "mantlepkgs" => mantle_application_core::CommandFamily::Package,
        _ => mantle_application_core::CommandFamily::Utility,
    }
}

pub(super) fn mutation_class(
    family: mantle_application_core::CommandFamily,
    root: &str,
) -> mantle_application_core::MutationClass {
    if matches!(root, "doctor" | "graph" | "why" | "dependents" | "log" | "__operator-contract") {
        return mantle_application_core::MutationClass::ReadOnly;
    }
    if matches!(
        family,
        mantle_application_core::CommandFamily::Remote
            | mantle_application_core::CommandFamily::Release
            | mantle_application_core::CommandFamily::Artifact
    ) {
        return mantle_application_core::MutationClass::ExternalEffect;
    }
    mantle_application_core::MutationClass::LocalMutation
}
