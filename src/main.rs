#![feature(register_tool)]
// machine-artifact-public: eval.raw-json-output
#![register_tool(tigerstyle)]
mod artifact_cmd;
// Evidence enums preserve the stable machine-contract shape; boxing or narrowing errors would alter
// that boundary.
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
mod ast_grep_evidence;
mod attest_cmd;
mod bootstrap;
mod bootstrap_parity;
mod bootstrap_source_root;
mod bootstrap_validate;
mod build_cmd;
#[allow(dead_code)]
mod build_correctness;
mod build_failure;
mod build_log;
mod build_plan;
pub mod build_planning_hexagon;
// Build-report variants intentionally carry complete stable JSON payloads rather than indirect
// boxed fragments.
#[allow(clippy::large_enum_variant)]
mod build_report;
#[allow(dead_code)]
mod cache_substitution;
mod cairn_release_handoff;
mod cargo_free_self_build;
mod cargo_import;
mod cargo_profile;
mod cargo_profile_manifest;
mod early_native_row_receipt;
mod early_native_row_receipt_shell;
mod elf_local_symbol_core;
mod elf_local_symbol_shell;
mod errors;
mod evaluation_stream_output;
mod evaluator_budget;
#[allow(dead_code)]
mod external_batch_dispatch;
mod filegen_cmd;
mod final_native_row_receipt_shell;
mod fix;
mod foreign_derivation_import;
mod foreign_executable_plan;
#[allow(dead_code)]
mod foreign_graph_compiler;
#[allow(dead_code)]
mod foreign_realization;
mod foreign_realization_receipt;
mod foreign_realization_shell;
// Foreign-import adapters mirror external receipt fields and preserve their typed error payloads at
// the CLI boundary.
#[allow(clippy::result_large_err, clippy::too_many_arguments)]
mod foreign_import_cmd;
mod foreign_provenance_audit;
mod frontend_artifact_export;
mod frontend_artifact_spec;
mod frontend_artifact_store;
mod full_source_provider;
#[allow(dead_code)]
mod full_source_rust_binding;
mod full_source_rust_binding_shell;
mod function_address_binding_cmd;
mod global_reproducibility_cmd;
mod global_reproducibility_release;
mod linux_rename;
mod log_cmd;
#[cfg(test)]
mod machine_contract_producer_tests;
mod mantlepkgs_adapter;
#[allow(clippy::large_enum_variant, clippy::too_many_arguments)]
mod mantlepkgs_cmd;
mod mantlepkgs_version_cmd;
mod native_toolchain_closure;
mod nickel_export;
mod nickel_export_core_adapter;
mod nix_derivation_adapter;
// Nix evidence keeps compatibility-only typed proofs available even when a selected command does
// not consume them.
#[allow(dead_code, clippy::type_complexity)]
mod nix_evidence_core;
#[allow(dead_code)]
mod nix_free_demo_bundle;
mod nix_free_demo_cmd;
mod nix_producer;
mod nix_producer_shell;
mod oci_projection;
mod oci_projection_shell;
mod oci_registry;
mod oci_registry_shell;
#[allow(dead_code)]
mod offline_cargo;
mod operator_contract;
mod operator_diagnostics;
// The comparison report shell lands in task I5 of
// `evaluate-picolibc-stagex-runtime`; until then only core tests consume it.
#[allow(dead_code)]
mod picolibc_comparison;
// Core tests for the distributed-evaluation feasibility assessment; the probe
// shell (`distributed_eval_assess`) is the only other consumer.
#[allow(dead_code)]
mod distributed_eval_assessment;
mod pin_import;
#[allow(dead_code)]
mod portable_receipt;
// Preserves carrier types intentionally encode the full external evidence graph and optional
// compatibility surfaces.
mod content_bound_requirement_evidence;
mod preserved_evidence_tree;
#[allow(dead_code, clippy::type_complexity)]
mod preserves_release_carrier;
mod project_build;
mod project_cmd;
mod project_resolve;
mod proof_clock_seccomp;
#[allow(dead_code)]
mod protected_exec;
#[allow(dead_code)]
mod protected_exec_ptrace;
mod protected_exec_seccomp;
mod radiance;
#[allow(dead_code)]
mod realization_routing;
mod rebuild_authority;
mod release_attestation;
#[allow(dead_code)]
mod release_capability;
mod release_chapter_transport;
mod release_cmd;
#[allow(dead_code)]
mod release_current_pointer;
#[allow(dead_code)]
mod release_evidence;
mod release_nix_witness;
mod release_publication;
#[allow(dead_code)]
mod release_reproducibility;
mod release_source;
mod release_tree_copy;
#[allow(dead_code)]
mod remote_attempt_log_store;
mod remote_credential_state;
mod remote_credentials;
mod remote_nominal;
mod remote_service_secrets;
// Remote build messages retain complete protocol payloads; boxing would change established internal
// handoff shapes.
#[allow(dead_code, clippy::large_enum_variant)]
mod remote_build;
mod remote_failure_debug;
mod remote_farm_config;
pub mod remote_hexagon;
mod remote_telemetry_export;
mod remote_trace_context;
// Transfer variants preserve complete resumable protocol records, including compatibility-only
// states.
#[allow(dead_code, clippy::large_enum_variant)]
mod remote_transfer;
mod rust_bootstrap_patch_plan;
// Rust-plan compatibility receipts expose wide Cargo-shaped adapters and preserve detailed typed
// failures.
#[allow(dead_code, clippy::result_large_err, clippy::too_many_arguments)]
mod rust_plan;
pub mod rust_plan_hexagon;
// Source-provider receipts retain full stage payloads and detailed fail-closed errors across the
// shell boundary.
#[allow(dead_code, clippy::large_enum_variant, clippy::result_large_err)]
mod rust_source_provider;
// rustc-dev-guide evidence mirrors upstream compiler graph relationships that are intentionally
// structurally complex.
#[allow(dead_code, clippy::type_complexity)]
mod rustc_dev_guide;
mod self_build;
#[allow(dead_code)]
mod semantic_graph;
mod shell_cmd;
mod source_built_derivation_action_plan;
#[allow(dead_code)]
mod source_built_fixed_point;
mod source_built_fixed_point_checkpoint;
mod source_built_fixed_point_checkpoint_shell;
mod source_built_fixed_point_dev_cache;
mod source_built_fixed_point_receipt;
mod source_built_fixed_point_shell;
mod source_built_parity_promotion;
mod source_built_parity_promotion_shell;
mod source_built_root_action_trust;
mod source_built_rust_action_plan;
#[cfg(target_os = "linux")]
mod source_built_rust_action_shell;
mod source_built_rust_provider_action;
mod source_built_trust_report;
mod source_built_trust_report_shell;
mod source_bundle;
mod source_review_release;
mod source_root_capability;
mod source_root_provider;
mod source_toolchain_closure;
#[allow(dead_code)]
mod stagex_archive_core;
mod stagex_bash;
#[allow(dead_code)]
mod stagex_bash_full;
#[allow(dead_code)]
mod stagex_binutils;
#[allow(dead_code)]
mod stagex_bison;
#[allow(dead_code)]
mod stagex_bzip2;
#[allow(dead_code)]
mod stagex_coreutils;
#[allow(dead_code)]
mod stagex_diffutils;
#[allow(dead_code)]
mod stagex_flex;
#[allow(dead_code)]
mod stagex_gawk;
#[allow(dead_code)]
mod stagex_gnu_patch;
#[allow(dead_code)]
mod stagex_grep;
#[allow(dead_code)]
mod stagex_gzip;
#[allow(dead_code)]
mod stagex_m4;
#[allow(dead_code)]
mod stagex_make;
#[allow(dead_code)]
mod stagex_mes;
#[allow(dead_code)]
mod stagex_mes_lib;
#[allow(dead_code)]
mod stagex_mes_sources;
#[allow(dead_code)]
mod stagex_musl;
#[allow(dead_code)]
mod stagex_musl_native;
#[allow(dead_code)]
mod stagex_oyacc;
#[allow(dead_code)]
mod stagex_patch;
#[allow(dead_code)]
mod stagex_provider;
#[allow(dead_code)]
mod stagex_sed;
#[allow(dead_code)]
mod stagex_sources;
#[allow(dead_code)]
mod stagex_stage0;
#[allow(dead_code)]
mod stagex_stage0_full;
#[allow(dead_code)]
mod stagex_tar;
#[allow(dead_code)]
mod stagex_tcc_musl;
#[allow(dead_code)]
mod stagex_tcc_musl_prep;
#[allow(dead_code)]
mod stagex_tcc_musl_v2;
#[allow(dead_code)]
mod stagex_tcc_selfhost;
#[allow(dead_code)]
mod stagex_tinycc;
#[allow(dead_code)]
mod stagex_tinycc27;
#[allow(dead_code)]
mod stagex_transition;
mod store_cmd;
mod structured_refactor;
mod transcript_cmd;
// Vendor manifests preserve heterogeneous source identities and compatibility-only validation
// branches.
#[allow(dead_code, clippy::type_complexity)]
mod vendor_source_manifest;
mod verification_gauntlet_cmd;
mod wasm_component_cmd;
mod witness_handoff;
mod witness_rebuild;

use std::path::PathBuf;
use std::process::ExitCode;

use build_cmd::BuildOutputMode;
use clap::Parser;
use errors::RunError;

mod cli_application;
pub mod cli_architecture;
mod cli_inbound;

pub(crate) use cli_application::build_project_expr;
pub(crate) use cli_application::current_dir_or_error;
pub(crate) use cli_application::first_output_path;
pub(crate) use cli_application::name_to_build_target;
pub(crate) use cli_application::project_output_root_registration;
pub(crate) use cli_application::project_retention_selector;
use cli_application::run;
pub(crate) use cli_application::unix_time_now_s;
use cli_inbound::*;

fn main() -> ExitCode {
    let args = Args::parse();
    if cli_inbound::has_conflicting_machine_output_modes(&args) {
        eprintln!("error: '--evaluation-stream' cannot be used with '--json'");
        return ExitCode::from(2);
    }
    init_tracing(&args);

    let is_json_error_output = args.json;

    match run(args) {
        Ok(()) => ExitCode::from(0),
        Err(error) => {
            let rendered = if is_json_error_output {
                error.format_json()
            } else {
                error.format_human()
            };
            if !rendered.is_empty() {
                eprintln!("{rendered}");
            }
            error.exit_code()
        }
    }
}

fn init_tracing(args: &Args) {
    let is_log_requested = args.verbose || args.log_level.is_some() || std::env::var_os("RUST_LOG").is_some();
    if args.json && !is_log_requested {
        return;
    }

    let level = args.log_level.as_deref().map(|value| value.parse().unwrap_or(tracing::Level::INFO)).unwrap_or(
        if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::WARN
        },
    );

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(level.into()))
        .with_writer(std::io::stderr)
        .init();
}

#[derive(Debug, Clone)]
struct RunContext {
    store: PathBuf,
    resolved_state_dir: PathBuf,
    store_prefix: String,
    verbose: bool,
    json: bool,
    base_state_dirs: Vec<PathBuf>,
}

impl RunContext {
    fn output_mode(&self) -> BuildOutputMode {
        if self.json {
            BuildOutputMode::Json
        } else {
            BuildOutputMode::Human
        }
    }
}
