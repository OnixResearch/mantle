//! Test CLI adapter. The normal build pipeline owns execution and signed reuse.
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use clap::Args;
use crunch_pipeline::BuildConfig;
use crunch_pipeline::PipelineResult;
use crunch_test::Invocation;
use crunch_test::Outcome;
use crunch_test::Report;
use crunch_test::Target;
use crunch_test_core::SelectedRequest;
use mantle_application_contract::rust_operations::RustEvidence;
use mantle_application_contract::rust_operations::RustOperation;
use nix_compat::nixhash::HashAlgo;
use nix_compat::store_path::StorePath;

use crate::RunContext;
use crate::build_cmd::BuildOutputMode;
use crate::build_cmd::LocalBuildCommandConfig;
use crate::errors::RunError;
use crate::source_bundle::SourceStoreTrust;
use crate::source_bundle::verify_signed_store_root;

#[derive(Args, Debug, Clone)]
pub struct TestArgs {
    /// Closed, versioned declaration of the admitted libtest target.
    pub target: PathBuf,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long)]
    pub exact: bool,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
    #[arg(long, default_value_t = 0)]
    pub shard_index: u32,
    #[arg(long, default_value_t = 1)]
    pub shard_count: u32,
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    #[arg(long, default_value_t = 1)]
    pub repeat: u32,
    /// Explicit unique token; changes derivation identity without bypassing trust.
    #[arg(long)]
    pub force_rerun: Option<String>,
    #[arg(long)]
    pub coverage: bool,
    #[arg(long)]
    pub signing_key: Option<PathBuf>,
    /// Additional trusted PathInfo signer; configured local trust remains in force.
    #[arg(long = "trusted-public-key")]
    pub trusted_public_keys: Vec<String>,
}

impl TestArgs {
    fn invocation(&self) -> Invocation {
        Invocation {
            filter: self.filter.clone(),
            exact: self.exact,
            timeout_ms: self.timeout_ms,
            shard_index: self.shard_index,
            shard_count: self.shard_count,
            seed: self.seed,
            repeat: self.repeat,
            force_rerun: self.force_rerun.clone(),
            coverage: self.coverage,
        }
    }
}

fn configuration(args: &TestArgs, ctx: &RunContext, recipe: &Path) -> Result<BuildConfig, RunError> {
    let trusted = args
        .trusted_public_keys
        .iter()
        .map(|key| {
            nix_compat::narinfo::VerifyingKey::parse(key)
                .map_err(|error| RunError::Internal(format!("test trusted key: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let output = std::path::absolute(&ctx.store).map_err(|error| RunError::Internal(error.to_string()))?;
    let config = crate::build_cmd::prepare_local_build_config(
        LocalBuildCommandConfig {
            file: recipe,
            import_paths: &[],
            output_dir: &output,
            state_dir: &ctx.resolved_state_dir,
            store_dir: &ctx.store_prefix,
            backend: ctx.store_backend,
            verbose: ctx.verbose,
            max_jobs: 1,
            substituter_urls: &[],
            signing_key_path: args.signing_key.as_deref(),
            trusted_public_keys: if trusted.is_empty() { None } else { Some(&trusted) },
            trust_unsigned: false,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            base_state_dirs: ctx.base_state_dirs.clone(),
            source_admission: ctx.store_source_admission(),
        },
        BuildOutputMode::Json,
        None,
        Vec::new(),
        None,
    )?;
    debug_assert!(config.hermeticity_mode.is_strict());
    debug_assert!(!config.trust_unsigned);
    Ok(config)
}

fn physical_root(logical: &str, config: &BuildConfig) -> Result<PathBuf, RunError> {
    let path = StorePath::<String>::from_absolute_path_with_prefix(logical.as_bytes(), &config.store_dir)
        .map_err(|error| RunError::Internal(format!("test input store root: {error}")))?;
    Ok(config.output_dir.join(path.to_string()))
}

fn admit_inputs(target: &Target, config: &BuildConfig) -> Result<(), RunError> {
    if target.store_prefix != config.store_dir {
        return Err(RunError::Internal("test target store prefix differs from selected store".into()));
    }
    let trust = SourceStoreTrust {
        output_dir: &config.output_dir,
        backend: config.backend,
        trusted_keys: &config.trusted_keys,
    };
    for (logical, expected) in &target.inputs {
        let signed = verify_signed_store_root(logical, &config.state_dir, &config.store_dir, &trust)?;
        let physical = physical_root(logical, config)?;
        let observed = crunch_nar::observe_path(
            &physical,
            crunch_nar::FilesystemNarRequest::new(
                HashAlgo::Blake3,
                crunch_nar::CaseHackPolicy::native(),
                signed.nar_size,
            ),
        )
        .map_err(|error| RunError::Internal(format!("test input NAR observation: {error}")))?;
        if observed.nar_size != signed.nar_size
            || data_encoding::HEXLOWER.encode(observed.digest.digest_as_bytes()) != *expected
        {
            return Err(RunError::Internal(format!("test input NAR BLAKE3 mismatch: {logical}")));
        }
    }
    debug_assert!(!target.inputs.is_empty());
    debug_assert_eq!(target.store_prefix, config.store_dir);
    Ok(())
}

fn admitted_report(
    result: &PipelineResult,
    config: &BuildConfig,
    request: SelectedRequest,
) -> Result<Report, RunError> {
    if !result.failed.is_empty() || !result.fod_mismatches.is_empty() {
        return Err(RunError::Build("test realization failed; no complete admitted report".into()));
    }
    let [outcome] = result.outcomes.as_slice() else {
        return Err(RunError::Internal("test realization must produce exactly one root".into()));
    };
    let info = outcome
        .outputs
        .get("out")
        .ok_or_else(|| RunError::Internal("test realization is missing its report output".into()))?;
    let logical = info.store_path.to_absolute_path_with_prefix(&config.store_dir);
    let signed = verify_signed_store_root(&logical, &config.state_dir, &config.store_dir, &SourceStoreTrust {
        output_dir: &config.output_dir,
        backend: config.backend,
        trusted_keys: &config.trusted_keys,
    })?;
    if signed.nar_size != info.nar_size || signed.nar_sha256 != data_encoding::HEXLOWER.encode(&info.nar_sha256) {
        return Err(RunError::Internal("test output changed after realization".into()));
    }
    let report = crunch_test::read_report(&physical_root(&logical, config)?, request).map_err(RunError::Internal)?;
    if outcome.cached && !crunch_test_core::cache_eligible(report.outcome) {
        return Err(RunError::Internal("test-cached-result-ineligible; use --force-rerun TOKEN".into()));
    }
    tracing::info!(cached = outcome.cached, output = %logical, "admitted test result");
    debug_assert!(!outcome.cached || crunch_test_core::cache_eligible(report.outcome));
    debug_assert_eq!(report.schema, crunch_test_core::REPORT_SCHEMA);
    Ok(report)
}

fn realize(args: &TestArgs, ctx: &RunContext, request: SelectedRequest) -> Result<Report, RunError> {
    let scratch = tempfile::tempdir().map_err(|error| RunError::Internal(error.to_string()))?;
    let recipe = scratch.path().join("test.ncl");
    let config = configuration(args, ctx, &recipe)?;
    admit_inputs(&request.target, &config)?;
    let request = crunch_test::write_recipe(request, &recipe).map_err(RunError::Internal)?;
    let result = match crate::build_cmd::run_test_build(&config)? {
        crate::build_cmd::TestBuildCompletion::Completed(result) => result,
        crate::build_cmd::TestBuildCompletion::CancelledAfterTeardown => {
            return Err(RunError::Reported(130));
        }
    };
    admitted_report(&result, &config, request)
}

pub fn run(args: &TestArgs, ctx: &RunContext) -> Result<(), RunError> {
    let request = crate::rust_plan_execution::through_rust_port(
        RustOperation::ReadTestTarget,
        || crunch_test::load_target(&args.target, args.invocation()).map_err(RunError::Internal),
        |request| RustEvidence::TestTarget {
            identity: &request.identity,
            input_count: request.target.inputs.len(),
        },
    )?;
    let result = crate::rust_plan_execution::through_rust_port(
        RustOperation::RealizeTest,
        || realize(args, ctx, request),
        |report| RustEvidence::TestReport {
            schema_valid: report.schema == crunch_test_core::REPORT_SCHEMA,
            identity: &report.identity,
        },
    );
    if matches!(&result, Err(RunError::Reported(130))) {
        eprintln!("test-cancelled-after-owned-teardown");
    }
    let report = result?;
    let code = match report.outcome {
        Outcome::Pass => 0,
        Outcome::AssertionFailure => 1,
        Outcome::Timeout => 124,
        Outcome::Cancelled => 130,
        Outcome::InfrastructureFailure => 2,
    };
    let mut output = std::io::stdout().lock();
    if ctx.json {
        serde_json::to_writer_pretty(&mut output, &report).map_err(|error| RunError::Internal(error.to_string()))?;
        writeln!(output).map_err(|error| RunError::Internal(error.to_string()))?;
    } else {
        let summary = crunch_test_core::summary(report);
        writeln!(output, "{}", summary.text).map_err(|error| RunError::Internal(error.to_string()))?;
    }
    if code == 0 {
        Ok(())
    } else {
        Err(RunError::Reported(code))
    }
}
