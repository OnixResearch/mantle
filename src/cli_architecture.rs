//! Typed inbound, operation, and presentation adapters for CLI dispatch.

// r[impl application_architecture.thin_composition_root]
// r[impl application_architecture.application_owned_ports]
// r[impl application_architecture.typed_error_ownership]
// r[impl application_architecture.effect_observation_boundary]
pub mod inbound_adapter;
pub mod operation_adapter;
pub mod presentation_adapter;

pub(crate) fn run_application(args: &crate::Args, ctx: &crate::RunContext) -> Result<(), crate::RunError> {
    let command = inbound_adapter::application_command(args);
    let mut adapter = operation_adapter::CliOperationAdapter { args, ctx };
    let outcome = mantle_application::run(command, &mut adapter).map_err(presentation_adapter::application_failure)?;
    presentation_adapter::finish(outcome)
}

#[cfg(test)]
mod tests;
