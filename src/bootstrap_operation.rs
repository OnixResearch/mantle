//! Bootstrap operation composition, separate from its shared capability port.
use std::path::Path;

use mantle_application_contract::bootstrap_effects::BootstrapEffect;

use crate::bootstrap_effect_port::BootstrapPort;
use crate::errors::RunError;

/// A phase may discover the next phase's inputs; each phase admits authority
/// before its capability, and reports no terminal output of its own.
pub(crate) fn capability<T>(effect: BootstrapEffect, run: impl FnOnce() -> Result<T, RunError>) -> Result<T, RunError> {
    let mut port = BootstrapPort::new(&[(effect, 1)])?;
    let result = port.call(effect, run);
    port.finish(result)
}

pub(crate) fn generate_inventory(
    output: &Path,
) -> Result<
    (
        std::path::PathBuf,
        crate::protected_exec::Stage0Inventory,
        Vec<crate::protected_exec::SeedClosureRiskReport>,
    ),
    RunError,
> {
    use BootstrapEffect::*;
    let mut port = BootstrapPort::new(&[
        (ReadEnvironment, 1),
        (ReadInput, 3),
        (WriteOutput, 1),
        (VerifyOutput, 1),
    ])?;
    let result = (|| {
        let output = if output.is_absolute() {
            output.to_path_buf()
        } else {
            port.call(ReadInput, crate::current_dir_or_error)?.join(output)
        };
        let config = port.call(ReadEnvironment, crate::protected_exec::stage0_seed_config_from_process_env).map_err(
            |error| RunError::Build(format!("generating stage0 inventory from explicit seed paths: {error}")),
        )?;
        let inventory = port
            .call(ReadInput, || crate::protected_exec::build_stage0_inventory_from_seed_config(&config))
            .map_err(|error| {
                RunError::Build(format!("generating stage0 inventory from explicit seed paths: {error}"))
            })?;
        let rendered = crate::protected_exec::render_stage0_inventory_nickel(&inventory)
            .map_err(|error| RunError::Build(error.to_string()))?;
        port.call(WriteOutput, || {
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    RunError::Build(format!("generating stage0 inventory from explicit seed paths: {error}"))
                })?;
            }
            crate::generated_file_write::write_generated_text(&output, &rendered)
        })?;
        port.verify_bytes(&output, rendered.as_bytes())?;
        let risks =
            port.call(ReadInput, || Ok::<_, RunError>(crate::protected_exec::seed_closure_risk_report(&inventory)))?;
        Ok((output, inventory, risks))
    })();
    port.finish(result)
}

pub(crate) fn resolve_packages(output: &Path, packages: &[String]) -> Result<(), RunError> {
    use BootstrapEffect::*;
    let count = u32::try_from(packages.len()).map_err(|_| RunError::Internal("too many bootstrap packages".into()))?;
    let mut effects = vec![(WriteOutput, 1), (VerifyOutput, 1)];
    if count > 0 {
        effects.push((ResolvePackages, count));
        effects.push((ReadInput, count));
    }
    let mut port = BootstrapPort::new(&effects)?;
    eprintln!("Resolving store paths for {} packages...", packages.len());
    let result = (|| {
        let mut entries = Vec::with_capacity(packages.len());
        for package in packages {
            let path = port.call(ResolvePackages, || crate::bootstrap::nix_resolve(package))?;
            port.call(ReadInput, || std::fs::metadata(&path)).map_err(|error| {
                RunError::Build(format!("resolved package {package} output {path} is unavailable: {error}"))
            })?;
            eprintln!("  {package} -> {path}");
            entries.push((package.clone(), path));
        }
        let ncl = crate::bootstrap::generate_seed_ncl(&entries);
        port.call(WriteOutput, || crate::generated_file_write::write_generated_text(output, &ncl))?;
        port.verify_bytes(output, ncl.as_bytes())
    })();
    port.finish(result)?;
    eprintln!("Wrote {}", output.display());
    Ok(())
}

pub(crate) struct PreparedProviderSmoke {
    pub(crate) smoke: crate::rust_source_provider::RustSourceProviderSmoke,
    pub(crate) evidence: Option<crate::rust_source_provider::RustSourceProviderSmokeEvidence>,
    _scratch: tempfile::TempDir,
}

pub(crate) fn prepare_provider_smoke(
    provider_dir: &Path,
    evidence_dir: Option<&Path>,
) -> Result<PreparedProviderSmoke, RunError> {
    let validation = capability(BootstrapEffect::ReadInput, || {
        crate::rust_source_provider::validate_materialized_rust_source_provider(provider_dir)
            .map_err(|error| RunError::Build(format!("Rust source provider validation failed closed: {error}")))
    })?;
    let scratch = capability(BootstrapEffect::WriteOutput, || {
        tempfile::Builder::new()
            .prefix("mantle-rust-provider-smoke-")
            .tempdir()
            .map_err(|error| RunError::Internal(format!("creating Rust provider smoke scratch: {error}")))
    })?;
    let smoke = crate::rust_source_provider::smoke_rust_source_provider(provider_dir, scratch.path())
        .map_err(|error| RunError::Build(format!("Rust source provider smoke failed closed: {error}")))?;
    let evidence = evidence_dir
        .map(|path| {
            crate::rust_source_provider::persist_rust_source_provider_smoke_evidence(path, &smoke, &validation)
                .map_err(|error| RunError::Build(format!("Rust source provider smoke evidence failed closed: {error}")))
        })
        .transpose()?;
    Ok(PreparedProviderSmoke {
        smoke,
        evidence,
        _scratch: scratch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tampered_publication_is_rejected_before_success() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("seed.ncl");
        let mut port =
            BootstrapPort::new(&[(BootstrapEffect::WriteOutput, 1), (BootstrapEffect::VerifyOutput, 1)]).unwrap();
        port.call(BootstrapEffect::WriteOutput, || std::fs::write(&output, b"tampered")).unwrap();
        let readback = port.verify_bytes(&output, b"expected");
        assert!(matches!(port.finish(readback), Err(RunError::Build(_))));
    }

    #[test]
    fn skipped_verification_rejects_successful_write() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("seed.ncl");
        let mut port =
            BootstrapPort::new(&[(BootstrapEffect::WriteOutput, 1), (BootstrapEffect::VerifyOutput, 1)]).unwrap();
        port.call(BootstrapEffect::WriteOutput, || std::fs::write(&output, b"expected")).unwrap();
        assert!(port.finish(Ok(())).is_err());
    }

    #[test]
    fn readback_rejects_unexpected_trailing_content() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("seed.ncl");
        std::fs::write(&output, b"expected plus unexpected bytes").unwrap();
        assert!(matches!(
            crate::bootstrap_effect_port::verify_content(&output, b"expected"),
            Err(RunError::Build(_))
        ));
    }
}
