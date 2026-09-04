mod profile;
#[cfg(target_os = "linux")]
mod publication;
#[cfg(target_os = "linux")]
mod runtime;
mod source;
#[cfg(test)]
mod tests;
mod workflow;

pub(crate) fn run_command(
    action: &crate::cli_inbound::RadianceReferenceAction,
    store_prefix: &str,
    is_json: bool,
) -> Result<(), crate::errors::RunError> {
    match action {
        crate::cli_inbound::RadianceReferenceAction::Prepare {
            git,
            radiance_checkout,
            bootstrap_compiler_checkout,
            emulator_checkout,
            source_bundle_out,
            cohort_out,
        } => {
            let prepare_result = source::prepare(source::PrepareRequest {
                git,
                radiance_checkout,
                bootstrap_compiler_checkout,
                emulator_checkout,
                source_bundle_out,
                cohort_out,
                store_prefix,
            })?;
            workflow::view::print_prepare(&prepare_result, is_json)
        }
        crate::cli_inbound::RadianceReferenceAction::Prove {
            source_bundle,
            expected_source_bundle_blake3,
            cohort,
            expected_cohort_blake3,
            cc,
            cc_driver,
            linker,
            crt_dir,
            libgcc_dir,
            output,
        } => {
            let proof_result = workflow::run_proof(workflow::ProofRequest {
                source_bundle,
                expected_source_bundle_blake3,
                cohort,
                expected_cohort_blake3,
                cc,
                cc_driver,
                linker,
                crt_dir,
                libgcc_dir,
                output,
            })?;
            workflow::view::print_proof(&proof_result, is_json)?;
            if proof_result.proof_success {
                Ok(())
            } else {
                Err(crate::errors::RunError::Build(format!(
                    "Radiance reference evidence is valid but the proof disposition is {:?}",
                    proof_result.disposition
                )))
            }
        }
        crate::cli_inbound::RadianceReferenceAction::Verify { output } => {
            let proof_result = workflow::verify_proof(output)?;
            workflow::view::print_proof(&proof_result, is_json)
        }
    }
}
