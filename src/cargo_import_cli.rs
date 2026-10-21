//! Explicit Cargo acquisition command adapter; native signed admission stays in cargo_acquisition.
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use clap::Args;
use nix_compat::narinfo::VerifyingKey;

use crate::cargo_acquisition::CargoAcquisitionBlocker;
use crate::cargo_acquisition::CargoAcquisitionInputs;
use crate::cargo_acquisition::VerifiedCargoVendor;
use crate::errors::RunError;

#[derive(Args, Debug, Clone)]
pub struct CargoAcquisitionArgs {
    /// Inspect only: reviewed lock-hash table, with no acquisition or import writes.
    #[arg(long, conflicts_with_all = ["acquire", "apply"])]
    pub acquisition_plan: Option<PathBuf>,
    /// Explicit signed logical producer/source/tool roots for lock-driven acquisition.
    #[arg(long, requires_all = ["acquisition_report", "signing_key", "trusted_public_keys"])]
    pub acquire: Option<PathBuf>,
    /// Separate atomically published acquisition report; import remains stdout's sole report.
    #[arg(long, requires = "acquire")]
    pub acquisition_report: Option<PathBuf>,
    #[arg(long, requires = "acquire")]
    pub signing_key: Option<PathBuf>,
    #[arg(long, value_delimiter = ',', requires = "acquire")]
    pub trusted_public_keys: Vec<String>,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=16))]
    pub jobs: u32,
}

// This CLI result transfers the admitted vendor by value without a heap allocation.
#[allow(clippy::large_enum_variant)]
pub enum Acquisition {
    Inspected,
    Import(Option<VerifiedCargoVendor>),
}

fn structured(error: CargoAcquisitionBlocker) -> RunError {
    match serde_json::to_string(&error) {
        Ok(json) => eprintln!("{json}"),
        Err(serialization) => return RunError::Internal(format!("serializing acquisition blocker: {serialization}")),
    }
    RunError::Reported(3)
}

fn inputs(path: &Path) -> Result<CargoAcquisitionInputs, RunError> {
    const MAX_BYTES: u64 = 2 * 1024 * 1024;
    let file = std::fs::File::open(path).map_err(|error| RunError::Internal(format!("acquisition input: {error}")))?;
    let metadata = file.metadata().map_err(|error| RunError::Internal(error.to_string()))?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err(RunError::Internal("acquisition inputs require a regular file of at most 2 MiB".into()));
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(error.to_string()))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(RunError::Internal("acquisition inputs exceed 2 MiB".into()));
    }
    serde_json::from_slice(&bytes).map_err(|error| RunError::Internal(format!("acquisition input JSON: {error}")))
}

pub fn prepare(root: &Path, ctx: &crate::RunContext, args: &CargoAcquisitionArgs) -> Result<Acquisition, RunError> {
    if let Some(table) = &args.acquisition_plan {
        let inspection = crate::cargo_acquisition::inspect_cargo_workspace(root, table).map_err(structured)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&inspection).map_err(|error| RunError::Internal(error.to_string()))?
        );
        return Ok(Acquisition::Inspected);
    }
    let Some(path) = &args.acquire else {
        return Ok(Acquisition::Import(None));
    };
    if !cfg!(target_os = "linux") {
        return Err(RunError::Build("Cargo acquisition requires the Linux local executor".into()));
    }
    let inputs = inputs(path)?;
    let keys = args
        .trusted_public_keys
        .iter()
        .map(|key| {
            VerifyingKey::parse(key).map_err(|error| RunError::Internal(format!("acquisition trusted key: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if keys.is_empty() || args.signing_key.is_none() || args.acquisition_report.is_none() {
        return Err(RunError::Internal(
            "acquisition requires explicit signing key, trusted keys and report path".into(),
        ));
    }
    let config = build_config(root, ctx, args, &keys)?;
    let (verified, report) =
        crate::cargo_acquisition::realize_cargo_acquisition(root, &inputs, config).map_err(structured)?;
    publish(args.acquisition_report.as_ref().expect("required report checked"), &report)?;
    Ok(Acquisition::Import(Some(verified)))
}

fn build_config(
    root: &Path,
    ctx: &crate::RunContext,
    args: &CargoAcquisitionArgs,
    keys: &[VerifyingKey],
) -> Result<crunch_pipeline::BuildConfig, RunError> {
    crate::build_cmd::prepare_local_build_config(
        crate::build_cmd::LocalBuildCommandConfig {
            file: &root.join("Cargo.lock"),
            import_paths: &[],
            output_dir: &ctx.store,
            state_dir: &ctx.resolved_state_dir,
            store_dir: &ctx.store_prefix,
            backend: ctx.store_backend,
            verbose: ctx.verbose,
            max_jobs: args.jobs,
            substituter_urls: &[],
            signing_key_path: args.signing_key.as_deref(),
            trusted_public_keys: Some(keys),
            trust_unsigned: false,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            base_state_dirs: ctx.base_state_dirs.clone(),
            source_admission: ctx.store_source_admission(),
        },
        crate::build_cmd::BuildOutputMode::Json,
        None,
        Vec::new(),
        None,
    )
}

fn publish(path: &Path, report: &impl serde::Serialize) -> Result<(), RunError> {
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|error| RunError::Internal(error.to_string()))?;
    serde_json::to_writer_pretty(&mut file, report).map_err(|error| RunError::Internal(error.to_string()))?;
    file.write_all(b"\n")
        .and_then(|_| file.as_file().sync_all())
        .map_err(|error| RunError::Internal(error.to_string()))?;
    file.persist(path)
        .map_err(|error| RunError::Internal(format!("publishing acquisition report: {error}")))?;
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| RunError::Internal(error.to_string()))
}
