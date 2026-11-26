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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::fs;

    use crunch_build::BuildFailureClass;
    use crunch_build::BuildFailurePhase;
    use crunch_build::BuildOutcome;
    use crunch_build::FailedGoal;
    use crunch_pipeline::HermeticityMode;
    use crunch_store::PathInfo;
    use crunch_store::StoreBackend;
    use crunch_store::StoreConfig;
    use crunch_store::StoreHandle;
    use crunch_store::VerifiedSourceIngestRequest;
    use crunch_test_core::Attempt;
    use crunch_test_core::COHORT;
    use crunch_test_core::CollectionFailure;
    use crunch_test_core::REPORT_SCHEMA;
    use crunch_test_core::Resources;
    use crunch_test_core::TARGET_SCHEMA;
    use crunch_test_core::Tool;
    use crunch_test_core::select;

    use super::*;

    fn test_args(target_path: &Path) -> TestArgs {
        TestArgs {
            target: target_path.to_path_buf(),
            filter: None,
            exact: false,
            timeout_ms: None,
            shard_index: 0,
            shard_count: 1,
            seed: 0,
            repeat: 1,
            force_rerun: None,
            coverage: false,
            signing_key: None,
            trusted_public_keys: Vec::new(),
        }
    }

    fn test_ctx(root: &Path) -> RunContext {
        RunContext {
            store: root.join("store"),
            resolved_state_dir: root.join("state"),
            resolved_log_dir: root.join("logs"),
            store_prefix: "/nix/store".into(),
            store_backend: StoreBackend::Snix,
            remote_binding_config: None,
            remote_endpoint_admission: None,
            verbose: false,
            json: true,
            base_state_dirs: Vec::new(),
        }
    }

    fn hex(ch: char) -> String {
        std::iter::repeat_n(ch, 64).collect()
    }

    fn sample_target(store_prefix: &str, logical_input: &str, input_b3: &str) -> Target {
        let tool = Tool {
            root: logical_input.into(),
            member: "tool.sh".into(),
            blake3: hex('1'),
        };
        Target {
            schema: TARGET_SCHEMA.into(),
            name: "cli-test".into(),
            cohort: COHORT.into(),
            rustc_verbose_version: "rustc 1.90.0 (1159e78c4 2025-09-14)\nLLVM version: 20.1.8\n".into(),
            store_prefix: store_prefix.into(),
            platform: "x86_64-linux".into(),
            executable: tool.clone(),
            runner: tool,
            inputs: BTreeMap::from([(logical_input.into(), input_b3.into())]),
            runfiles: BTreeMap::new(),
            environment: BTreeMap::new(),
            tests: vec!["tests::smoke".into()],
            resources: Resources {
                memory_bytes: 67_108_864,
                output_bytes: 65_536,
                timeout_ms: 5_000,
            },
            deterministic: true,
            external_services: false,
            coverage: None,
        }
    }

    fn empty_pipeline_result() -> PipelineResult {
        PipelineResult {
            outcomes: Vec::new(),
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::new(),
            hermeticity_mode: HermeticityMode::Strict,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            finish_gate_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            plan_output_bindings: Vec::new(),
            priority_decisions: Vec::new(),
            pool_goal_observations: Vec::new(),
            overlay_report: None,
            store_layer_selections: Vec::new(),
        }
    }

    fn ingest_signed_tree(config: &BuildConfig, source_dir: &Path, name: &str, digest_byte: u8) -> (PathInfo, String) {
        let store_path: StorePath<String> =
            StorePath::from_name_and_digest_fixed(name, [digest_byte; nix_compat::store_path::DIGEST_SIZE]).unwrap();
        let logical = store_path.to_absolute_path_with_prefix(&config.store_dir);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let info = rt.block_on(async {
            let mut store = StoreHandle::open(StoreConfig {
                source_admission: None,
                backend: config.backend,
                state_dir: config.state_dir.clone(),
                output_dir: config.output_dir.clone(),
                remote_cache_urls: Vec::new(),
                fallback_mode: crunch_store::StoreFallbackMode::Practical,
                store_dir: config.store_dir.clone(),
                base_state_dirs: Vec::new(),
            })
            .await
            .unwrap();
            store
                .ingest_verified_source(VerifiedSourceIngestRequest {
                    source_path: source_dir,
                    logical_store_path: &logical,
                    source_name: name,
                    signing_key: &config.keypair.signing_key,
                })
                .await
                .unwrap()
        });
        let exported = config.output_dir.join(store_path.to_string());
        fs::create_dir_all(&exported).unwrap();
        for entry in fs::read_dir(source_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::copy(entry.path(), exported.join(entry.file_name())).unwrap();
            }
        }
        (info, logical)
    }

    fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) {
        let file = fs::File::create(path).unwrap();
        serde_json::to_writer(file, value).unwrap();
    }

    #[test]
    fn configuration_rejects_invalid_trusted_public_key_and_enforces_strict_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = test_ctx(dir.path());
        fs::create_dir_all(&ctx.store).unwrap();
        fs::create_dir_all(&ctx.resolved_state_dir).unwrap();
        let recipe = dir.path().join("test.ncl");

        let mut bad_args = test_args(&dir.path().join("target.json"));
        bad_args.trusted_public_keys = vec!["not-a-valid-key".into()];
        let err = configuration(&bad_args, &ctx, &recipe).err().unwrap();
        assert!(err.message().contains("test trusted key"), "unexpected error: {}", err.message());

        let ok_args = test_args(&dir.path().join("target.json"));
        let config = configuration(&ok_args, &ctx, &recipe).unwrap();
        assert!(config.hermeticity_mode.is_strict());
        assert!(!config.trust_unsigned);
        assert_eq!(config.max_jobs, 1);
    }

    #[test]
    fn admit_inputs_validates_prefix_signature_and_blake3_nar_digest() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = test_ctx(dir.path());
        fs::create_dir_all(&ctx.store).unwrap();
        fs::create_dir_all(&ctx.resolved_state_dir).unwrap();
        let recipe = dir.path().join("test.ncl");
        let mut config = configuration(&test_args(&dir.path().join("target.json")), &ctx, &recipe).unwrap();
        config.trusted_keys.push(config.keypair.verifying_key.clone());

        let src_dir = dir.path().join("input-src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("tool.sh"), b"#!/bin/sh\nexit 0\n").unwrap();
        let (info, logical) = ingest_signed_tree(&config, &src_dir, "test-input", 0x22);
        let physical = physical_root(&logical, &config).unwrap();
        let observed = crunch_nar::observe_path(
            &physical,
            crunch_nar::FilesystemNarRequest::new(
                HashAlgo::Blake3,
                crunch_nar::CaseHackPolicy::native(),
                info.nar_size,
            ),
        )
        .unwrap();
        let actual_b3 = data_encoding::HEXLOWER.encode(observed.digest.digest_as_bytes());

        let valid_target = sample_target(&config.store_dir, &logical, &actual_b3);
        admit_inputs(&valid_target, &config).unwrap();

        let wrong_prefix_target =
            sample_target("/other/store", "/other/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-x", &actual_b3);
        let err = admit_inputs(&wrong_prefix_target, &config).unwrap_err();
        assert!(err.message().contains("store prefix differs"), "{}", err.message());

        let wrong_digest_target = sample_target(&config.store_dir, &logical, &hex('f'));
        let err = admit_inputs(&wrong_digest_target, &config).unwrap_err();
        assert!(err.message().contains("NAR BLAKE3 mismatch"), "{}", err.message());

        let missing_logical = StorePath::<String>::from_name_and_digest_fixed("missing-input", [0x33; 20])
            .unwrap()
            .to_absolute_path_with_prefix(&config.store_dir);
        let unsigned_target = sample_target(&config.store_dir, &missing_logical, &actual_b3);
        assert!(admit_inputs(&unsigned_target, &config).is_err());
    }

    #[test]
    fn admitted_report_checks_pipeline_outputs_integrity_and_cache_eligibility() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = test_ctx(dir.path());
        fs::create_dir_all(&ctx.store).unwrap();
        fs::create_dir_all(&ctx.resolved_state_dir).unwrap();
        let recipe = dir.path().join("test.ncl");
        let mut config = configuration(&test_args(&dir.path().join("target.json")), &ctx, &recipe).unwrap();
        config.trusted_keys.push(config.keypair.verifying_key.clone());

        let logical_input = StorePath::<String>::from_name_and_digest_fixed("in-root", [0x44; 20])
            .unwrap()
            .to_absolute_path_with_prefix(&config.store_dir);
        let target = sample_target(&config.store_dir, &logical_input, &hex('9'));

        // 1. Failed pipeline returns Build error.
        let mut failed_result = empty_pipeline_result();
        failed_result.failed.push(FailedGoal {
            drv_key: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-t.drv".into(),
            origin_drv_key: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-t.drv".into(),
            phase: BuildFailurePhase::Build,
            error_class: BuildFailureClass::Builder,
            fod_mismatch: None,
            error: "boom".into(),
            origin_error: "boom".into(),
            build_log: None,
        });
        let req = select(target.clone(), Invocation::default()).unwrap();
        assert!(matches!(admitted_report(&failed_result, &config, req), Err(RunError::Build(_))));

        // 2. Outcomes count != 1 returns Internal error.
        let req = select(target.clone(), Invocation::default()).unwrap();
        let err = admitted_report(&empty_pipeline_result(), &config, req).unwrap_err();
        assert!(err.message().contains("exactly one root"), "{}", err.message());

        // 3. Missing "out" output returns Internal error.
        let drv_path = StorePath::<String>::from_name_and_digest_fixed("test.drv", [0x55; 20]).unwrap();
        let mut missing_out_result = empty_pipeline_result();
        missing_out_result.outcomes.push(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: BTreeMap::new(),
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
            sandbox_timing: None,
        });
        let req = select(target.clone(), Invocation::default()).unwrap();
        let err = admitted_report(&missing_out_result, &config, req).unwrap_err();
        assert!(err.message().contains("missing its report output"), "{}", err.message());

        // 4. Signed report output with post-realization mismatch, cached Timeout rejection, and cached Pass
        //    acceptance.
        let req = select(target.clone(), Invocation::default()).unwrap();
        let timeout_dir = dir.path().join("out-timeout");
        fs::create_dir_all(&timeout_dir).unwrap();
        let timeout_report = Report {
            schema: REPORT_SCHEMA.into(),
            identity: req.identity.clone(),
            target: req.target.name.clone(),
            selected: req.selected.clone(),
            outcome: Outcome::Timeout,
            attempts: Vec::new(),
            coverage: None,
            collection_failure: Some(CollectionFailure {
                outcome: Outcome::Timeout,
                reason: "deadline".into(),
            }),
        };
        write_json_file(&timeout_dir.join("report.json"), &timeout_report);
        let (timeout_info, _) = ingest_signed_tree(&config, &timeout_dir, "test-out-timeout", 0x66);

        let mut tampered_info = timeout_info.clone();
        tampered_info.nar_size += 1;
        let mut tampered_result = empty_pipeline_result();
        tampered_result.outcomes.push(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: BTreeMap::from([("out".into(), tampered_info)]),
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
            sandbox_timing: None,
        });
        let err = admitted_report(&tampered_result, &config, select(target.clone(), Invocation::default()).unwrap())
            .unwrap_err();
        assert!(err.message().contains("output changed after realization"), "{}", err.message());

        let mut cached_timeout_result = empty_pipeline_result();
        cached_timeout_result.outcomes.push(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: BTreeMap::from([("out".into(), timeout_info.clone())]),
            substitutions: BTreeMap::new(),
            cached: true,
            log: None,
            sandbox_timing: None,
        });
        let err =
            admitted_report(&cached_timeout_result, &config, select(target.clone(), Invocation::default()).unwrap())
                .unwrap_err();
        assert_eq!(err.message(), "test-cached-result-ineligible; use --force-rerun TOKEN");

        // Uncached Timeout is accepted.
        let mut uncached_timeout_result = empty_pipeline_result();
        uncached_timeout_result.outcomes.push(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: BTreeMap::from([("out".into(), timeout_info)]),
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
            sandbox_timing: None,
        });
        let uncached =
            admitted_report(&uncached_timeout_result, &config, select(target.clone(), Invocation::default()).unwrap())
                .unwrap();
        assert_eq!(uncached.outcome, Outcome::Timeout);

        // Cached Pass is accepted.
        let pass_dir = dir.path().join("out-pass");
        fs::create_dir_all(&pass_dir).unwrap();
        let pass_req = select(target, Invocation::default()).unwrap();
        let pass_report = Report {
            schema: REPORT_SCHEMA.into(),
            identity: pass_req.identity.clone(),
            target: pass_req.target.name.clone(),
            selected: pass_req.selected.clone(),
            outcome: Outcome::Pass,
            attempts: vec![Attempt {
                repeat: 0,
                test: "tests::smoke".into(),
                outcome: Outcome::Pass,
                reason: "complete".into(),
            }],
            coverage: None,
            collection_failure: None,
        };
        write_json_file(&pass_dir.join("report.json"), &pass_report);
        let (pass_info, _) = ingest_signed_tree(&config, &pass_dir, "test-out-pass", 0x77);
        let mut cached_pass_result = empty_pipeline_result();
        cached_pass_result.outcomes.push(BuildOutcome {
            drv_path,
            outputs: BTreeMap::from([("out".into(), pass_info)]),
            substitutions: BTreeMap::new(),
            cached: true,
            log: None,
            sandbox_timing: None,
        });
        let loaded = admitted_report(&cached_pass_result, &config, pass_req).unwrap();
        assert_eq!(loaded, pass_report);
    }
}
