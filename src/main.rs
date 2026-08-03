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
// Build-report variants intentionally carry complete stable JSON payloads rather than indirect
// boxed fragments.
#[allow(clippy::large_enum_variant)]
mod build_report;
#[allow(dead_code)]
mod cache_substitution;
mod cairn_release_handoff;
mod cargo_free_self_build;
mod cargo_import;
mod early_native_row_receipt;
mod early_native_row_receipt_shell;
mod elf_local_symbol_core;
mod elf_local_symbol_shell;
mod errors;
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
mod native_toolchain_closure;
mod nickel_export;
mod nickel_export_core_adapter;
// Nix evidence keeps compatibility-only typed proofs available even when a selected command does
// not consume them.
#[allow(dead_code, clippy::type_complexity)]
mod nix_evidence_core;
#[allow(dead_code)]
mod nix_free_demo_bundle;
// Demo command variants retain complete validation inputs so clap and JSON compatibility stay
// stable.
#[allow(clippy::large_enum_variant)]
mod nix_free_demo_cmd;
mod oci_projection;
mod oci_projection_shell;
mod oci_registry;
mod oci_registry_shell;
#[allow(dead_code)]
mod offline_cargo;
mod operator_diagnostics;
mod pin_import;
#[allow(dead_code)]
mod portable_receipt;
// Preserves carrier types intentionally encode the full external evidence graph and optional
// compatibility surfaces.
mod content_bound_requirement_evidence;
#[allow(dead_code, clippy::type_complexity)]
mod preserves_release_carrier;
mod project_build;
mod project_cmd;
mod project_resolve;
mod proof_clock_seccomp;
#[allow(dead_code)]
mod protected_exec;
#[allow(dead_code)]
mod protected_exec_seccomp;
#[allow(dead_code)]
mod realization_routing;
mod rebuild_authority;
mod release_attestation;
#[allow(dead_code)]
mod release_capability;
mod release_cmd;
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
// Remote build messages retain complete protocol payloads; boxing would change established internal
// handoff shapes.
#[allow(dead_code, clippy::large_enum_variant)]
mod remote_build;
mod remote_failure_debug;
mod remote_farm_config;
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
#[allow(dead_code)]
mod source_built_fixed_point;
mod source_built_fixed_point_receipt;
mod source_built_fixed_point_shell;
mod source_bundle;
mod source_root_capability;
mod source_root_provider;
mod source_toolchain_closure;
#[allow(dead_code)]
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

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const EXPECTED_RUN_OUTCOME_COUNT: usize = 1;
const CHILD_NO_EXIT_CODE_STATUS: i32 = 1;
const DEFAULT_SUBSTITUTER_COUNT: u32 = 1;
const EMPTY_TRUSTED_PUBLIC_KEY_COUNT: usize = 0;
const REMOTE_CLIENT_BUILD_REPORT_SCHEMA: &str = "mantle-remote-client-build-v1";
const BUILD_JSON_REPORT_SCHEMA: &str = "crunch-build-report-v1";
const REMOTE_BUILD_HERMETICITY_MODE: &str = "practical";
const REMOTE_TEST_INTERRUPT_AFTER_INPUT_CHUNKS_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_INPUT_CHUNKS";
const W3C_TRACEPARENT_ENV: &str = "TRACEPARENT";
const W3C_TRACESTATE_ENV: &str = "TRACESTATE";
const REMOTE_TEST_INTERRUPT_AFTER_OUTPUT_CHUNKS_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS";
const REMOTE_TRANSFER_INTERRUPTION_REASON_FRAGMENT: &str = "transfer-interrupted-after-checkpoint";
const REMOTE_FAILURE_REPLAY_LEASE_SECS: u64 = 3_600;
const REMOTE_FAILURE_REPLAY_RANDOM_BYTES: usize = 16;
const REMOTE_FAILURE_DEBUG_BUNDLE_REF_PREFIX: &str = "remote-failure-debug:";
const REMOTE_FAILURE_DEBUG_DIGEST_HEX_CHARS: usize = 64;
const REMOTE_FAILURE_DEBUG_STATUS_CODE_BYTES_MAX: usize = 128;
const SOURCE_BUILT_FIXED_POINT_ELAPSED_SECONDS_MAX_DEFAULT: u64 = 86_400;
const SOURCE_BUILT_FIXED_POINT_DISK_BYTES_MAX_DEFAULT: u64 = 1_099_511_627_776;
const SOURCE_BUILT_FIXED_POINT_PROTECTED_EXEC_EVENTS_MAX_DEFAULT: u32 = 262_144;
const SOURCE_BUILT_FIXED_POINT_SOURCE_RECORDS_MAX_DEFAULT: u32 = 65_536;
const SOURCE_BUILT_FIXED_POINT_JOBS_DEFAULT: u32 = 4;
#[cfg(unix)]
const UNIX_EXECUTE_BITS: u32 = 0o111;

use build_cmd::BuildOutputMode;
use build_cmd::build_import_paths;
use build_cmd::state_dir;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use errors::RunError;
use foreign_import_cmd::ForeignImportAction;
use mantlepkgs_cmd::MantlepkgsAction;
use nix_free_demo_cmd::NixFreeDemoAction;
use operator_diagnostics::DoctorProfile;
use operator_diagnostics::RuntimeFingerprintModeFields;

#[derive(Parser, Debug)]
#[command(
    name = "mantle",
    bin_name = "mantle",
    version,
    about = "Mantle build system on the Nix store protocol"
)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true)]
    log_level: Option<String>,

    /// Emit errors as JSON objects for tooling integration
    #[arg(long, global = true)]
    json: bool,

    /// Where to export final build outputs on disk. Intermediate
    /// deps stay in castore. Source inputs always come from /nix/store.
    /// If not writable, builds still succeed (outputs in castore).
    #[arg(long, global = true, default_value = "/nix/store")]
    store: PathBuf,

    /// Logical store path prefix for derivation hashes. All output
    /// paths and ATerm hashes are computed against this prefix.
    /// Default: /mantle/store. Use --nix-compat for /nix/store.
    #[arg(long, global = true, default_value = "/mantle/store")]
    store_prefix: String,

    /// Shorthand for --store-prefix=/nix/store. For interop testing
    /// with Nix-computed derivations.
    #[arg(long, global = true)]
    nix_compat: bool,

    /// State directory for pathinfo.redb, blobs, and logs.
    /// Default: legacy $CRUNCH_STATE_DIR or $XDG_STATE_HOME/crunch or
    /// ~/.local/state/crunch. Use --state-dir for an explicit Mantle path.
    #[arg(long, global = true)]
    state_dir: Option<PathBuf>,

    /// Ordered list of read-only base store state directories for overlay
    /// composition. Declared in priority order (base A before base B).
    #[arg(long = "base-store", global = true)]
    base_stores: Vec<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

// CLI variants retain their complete clap payloads to preserve flag and help compatibility.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
enum Command {
    /// Evaluate and build derivation(s).
    ///
    /// With no arguments: builds the default package from the compatibility-named crunch.ncl
    /// package root. With a .ncl file: builds derivations from that file.
    /// With .#name: builds a named output from the compatibility-named crunch.ncl package root.
    Build {
        /// Path to a .ncl file, or .#name selector, or omit for project default
        file: Option<PathBuf>,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Auto-fix FOD hash mismatches by rewriting the .ncl source
        #[arg(long)]
        fix: bool,

        /// Show planned per-root actions without building or mutating local store state
        #[arg(long)]
        plan: bool,

        /// Require imported source state for selected roots before planning or building
        #[arg(long)]
        offline_source_preflight: bool,

        /// Maximum number of concurrent builds (default: CPU count, max 16)
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Binary cache URLs for substitution (comma-separated)
        #[arg(long, default_value = "https://cache.nixos.org")]
        substituters: String,

        /// Disable remote binary cache substitution
        #[arg(long)]
        no_substitute: bool,

        /// Path to a Nix-format ed25519 signing keypair file.
        /// If omitted, auto-generates one at $CRUNCH_CONFIG_DIR/signing-key.
        #[arg(long)]
        signing_key: Option<PathBuf>,

        /// Trusted public keys for signature verification (name:base64).
        /// Overrides the default set (which includes cache.nixos.org-1).
        #[arg(long, value_delimiter = ',')]
        trusted_public_keys: Vec<String>,

        /// Accept unsigned/unverified PathInfo on cache hits.
        /// Escape hatch for migrating from unsigned stores.
        #[arg(long)]
        trust_unsigned: bool,

        /// Reject degraded hermetic behavior once strict-mode blockers exist.
        #[arg(long, conflicts_with = "impure")]
        strict_hermetic: bool,

        /// Permit ambient host dependencies; outputs are not proof-eligible.
        #[arg(long, conflicts_with = "strict_hermetic")]
        impure: bool,

        /// Remote builder endpoint id for stdio remote-build dispatch.
        #[arg(long)]
        builder: Option<String>,

        /// Bearer ticket as <ticket-id>:<secret> for remote-build dispatch.
        #[arg(long)]
        ticket: Option<String>,

        /// Program to spawn for stdio remote-build dispatch. Defaults to this mantle binary.
        #[arg(long)]
        builder_program: Option<PathBuf>,

        /// Argument passed to --builder-program; repeat to pass multiple arguments.
        #[arg(long = "builder-arg")]
        builder_args: Vec<String>,

        /// Trusted remote builder signing key name; repeat for multiple keys.
        #[arg(long = "trusted-builder-key")]
        trusted_builder_keys: Vec<String>,

        /// Remote request build-time limit in seconds.
        #[arg(long, default_value_t = remote_build::DEFAULT_TICKET_BUILD_TIME_SECS)]
        remote_build_time_secs: u64,

        /// Offer bounded delta transfer with full-NAR streaming fallback to the remote builder.
        #[arg(long)]
        remote_delta: bool,

        /// Typed Nickel configuration for disabled-by-default remote telemetry and trace
        /// propagation.
        #[arg(long, requires = "builder")]
        remote_observability_config: Option<PathBuf>,
    },

    /// Build a portable WebAssembly component through the pinned production cohort.
    WasmComponent {
        #[command(subcommand)]
        action: WasmComponentAction,
    },

    /// Run no-mutate operator preflight checks for a workflow profile
    Doctor {
        /// Workflow profile to check
        #[arg(long, value_enum, default_value = "build")]
        profile: DoctorProfile,
    },

    /// Import external project metadata into Mantle-owned build files
    Import {
        #[command(subcommand)]
        action: ImportAction,
    },

    /// Plan or explicitly apply project-declared generated files
    Filegen {
        #[command(subcommand)]
        action: FilegenCommandAction,
    },

    /// Show a semantic build graph rooted at an output/proof/source identity or alias
    Graph {
        /// Identity or alias to use as the graph root
        root: String,

        /// Semantic graph JSON file (default: <state-dir>/semantic-graph.json)
        #[arg(long = "graph-file")]
        graph_file: Option<PathBuf>,
    },

    /// Explain why an output/proof identity exists from semantic graph records
    Why {
        /// Identity or alias to explain
        target: String,

        /// Semantic graph JSON file (default: <state-dir>/semantic-graph.json)
        #[arg(long = "graph-file")]
        graph_file: Option<PathBuf>,
    },

    /// List graph dependents of an identity or alias
    Dependents {
        /// Identity or alias whose dependents should be listed
        target: String,

        /// Semantic graph JSON file (default: <state-dir>/semantic-graph.json)
        #[arg(long = "graph-file")]
        graph_file: Option<PathBuf>,
    },

    /// Plan, check, or apply structured refactor/migration sessions
    Refactor {
        #[command(subcommand)]
        action: RefactorAction,
    },

    /// Execute Markdown Mantle transcripts with isolated store/state defaults
    Transcript {
        #[command(subcommand)]
        action: TranscriptAction,
    },

    /// Generate a host-tool-free stage0 inventory from explicit seed paths.
    Stage0Inventory {
        /// Output inventory path.
        #[arg(short, long, default_value = "target/host-tool-free-stage0/stage0-inventory.ncl")]
        output: PathBuf,
    },

    /// Validate or render the bounded Nix-free fixed-point demo bundle profile.
    NixFreeDemo {
        #[command(subcommand)]
        action: NixFreeDemoAction,
    },

    /// Validate and plan from lowered foreign derivation import artifacts.
    ForeignImport {
        #[command(subcommand)]
        action: ForeignImportAction,
    },

    /// Generate, verify, plan, and build locked Mantlepkgs catalogs.
    Mantlepkgs {
        #[command(subcommand)]
        action: MantlepkgsAction,
    },

    /// Evaluate a .ncl file and print the derivation JSON (no build)
    Eval {
        /// Path to the .ncl file
        file: PathBuf,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,
    },

    /// Export declared Nickel data to a deterministic external JSON payload and receipt
    Export {
        /// Root Nickel source file to evaluate
        file: PathBuf,

        /// Declared dependency source file; repeatable
        #[arg(long = "dep")]
        deps: Vec<PathBuf>,

        /// Safe relative import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Export format (currently json)
        #[arg(long, default_value = "json")]
        format: String,

        /// Write the evaluated payload to a file instead of stdout
        #[arg(long)]
        out: Option<PathBuf>,

        /// Evaluator identity recorded in the receipt
        #[arg(long = "evaluator-id", default_value_t = nickel_export::default_evaluator_id())]
        evaluator_id: String,

        /// Evaluator version recorded in the receipt
        #[arg(long = "evaluator-version", default_value = env!("CARGO_PKG_VERSION"))]
        evaluator_version: String,
    },

    /// Generate bootstrap seeds or validate bootstrap runtime evidence
    Bootstrap {
        #[command(subcommand)]
        action: Option<BootstrapAction>,

        /// Output file path, or an absent absolute provider directory in StageX mode.
        #[arg(short, long, default_value = "seed.ncl")]
        output: PathBuf,

        /// Fetch the shared bootstrap seed provider instead of querying Nix.
        /// Builds the reduced normalized seed from the pinned musl.cc tarball,
        /// persists it in --store, and writes seed.ncl. No Nix required.
        #[arg(long)]
        fetch: bool,

        /// Validate a full-source root manifest and select the source-root provider path.
        #[arg(long)]
        source_root: Option<PathBuf>,

        /// Materialize from one explicit absolute StageX lineage manifest.
        #[arg(long)]
        stagex_lineage: Option<PathBuf>,

        /// Import one explicit absolute, complete protected StageX transition root.
        #[arg(long, requires = "stagex_lineage")]
        stagex_transition_root: Option<PathBuf>,

        /// Require imported/pinned bootstrap source state before fetch-mode source acquisition.
        #[arg(long)]
        offline_source_preflight: bool,

        /// Packages to include (nixpkgs attribute names, ignored with --fetch)
        #[arg(default_values_t = [
            "bash".to_string(),
            "coreutils".to_string(),
            "gcc".to_string(),
            "gnumake".to_string(),
            "binutils".to_string(),
        ])]
        packages: Vec<String>,
    },

    /// Show a stored build log
    Log {
        /// Derivation hash, store path, or path fragment to match
        query: Option<String>,

        /// List all stored logs
        #[arg(long)]
        list: bool,
    },

    /// Inspect the store's PathInfo database
    Store {
        #[command(subcommand)]
        action: StoreAction,
    },

    /// Plan, export, import, list, and verify source/input bundles
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },

    /// Export, list, verify, and import portable receipt bundles
    Receipt {
        #[command(subcommand)]
        action: ReceiptAction,
    },

    /// Remote builder access, ticket, and server commands
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// Frontend-neutral admitted artifact commands
    Artifact {
        #[command(subcommand)]
        action: ArtifactAction,
    },

    /// Attestation inspection and verification commands
    Attest {
        #[command(subcommand)]
        action: AttestAction,
    },

    /// Create or verify a release evidence bundle
    Release {
        #[command(subcommand)]
        action: ReleaseAction,
    },

    /// Initialize a new Mantle project (manifest, lockfile, .mantle/)
    Init,

    /// Validate project manifest, lockfile, and generated inputs
    Check {
        /// Label check as allowing freshness probes when probe adapters are available
        #[arg(long)]
        probes: bool,

        /// Label check as allowing trust verification when trust adapters are available
        #[arg(long)]
        trust: bool,
    },

    /// Show resolved input state from the lockfile
    Show,

    /// Refresh selected or all project inputs
    Refresh {
        /// Do not execute network-backed freshness probes
        #[arg(long)]
        no_network: bool,

        /// Input names to refresh (default: all non-frozen)
        names: Vec<String>,
    },

    /// List inputs that would change on refresh (read-only)
    ListStale {
        /// Do not execute network-backed freshness probes
        #[arg(long)]
        no_network: bool,
    },

    /// Migrate project files to the current schema version
    Upgrade,

    /// Build Mantle from its own source (self-hosting)
    SelfBuild {
        /// Maximum concurrent builds (default: CPU count, max 16)
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Disable remote binary cache substitution
        #[arg(long)]
        no_substitute: bool,

        /// Skip verification of the output binary
        #[arg(long)]
        no_verify: bool,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,

        /// Trusted public keys for signature verification (name:base64)
        #[arg(long, value_delimiter = ',')]
        trusted_public_keys: Vec<String>,

        /// Accept unsigned/unverified PathInfo on cache hits
        #[arg(long)]
        trust_unsigned: bool,

        /// Reject degraded hermetic behavior once strict-mode blockers exist.
        #[arg(long, conflicts_with = "impure")]
        strict_hermetic: bool,

        /// Permit ambient host dependencies; outputs are not proof-eligible.
        #[arg(long, conflicts_with = "strict_hermetic")]
        impure: bool,

        /// Require stage0 to use only declared inventory seed tools.
        #[arg(long)]
        no_host_tools: bool,

        /// Stage0 executable/source seed inventory for --no-host-tools.
        #[arg(long)]
        stage0_inventory: Option<PathBuf>,

        /// Internal: reuse an exact staged source tree from a prior self-build.
        #[arg(long, hide = true)]
        source_store_path: Option<PathBuf>,

        /// Internal: reuse the exact stage0-produced bwrap binary in later proof stages.
        #[arg(long, hide = true)]
        bootstrap_bwrap_path: Option<PathBuf>,

        /// Internal: reuse the exact stage0-produced busybox binary in later proof stages.
        #[arg(long, hide = true)]
        bootstrap_busybox_path: Option<PathBuf>,

        /// Require all builtin fetches to resolve from this pinned full-proof source manifest
        /// BLAKE3.
        #[arg(long, requires = "offline_source_state_dir")]
        offline_source_manifest_blake3: Option<String>,

        /// Read the pinned full-proof source closure from this independently hydrated source-only
        /// state.
        #[arg(long, requires = "offline_source_manifest_blake3")]
        offline_source_state_dir: Option<PathBuf>,

        /// Construct the full source lineage and prove the Cargo-free Mantle fixed point.
        #[arg(long, conflicts_with = "cargo_free")]
        source_built_fixed_point: bool,

        /// Authenticated source-built-fixed-point source profile.
        #[arg(long, requires = "source_built_fixed_point")]
        source_profile: Option<PathBuf>,

        /// Independently authenticated BLAKE3 of --source-profile.
        #[arg(long, requires = "source_built_fixed_point")]
        expected_source_profile_blake3: Option<String>,

        /// Independently authenticated StageX lineage-manifest BLAKE3.
        #[arg(long, requires = "source_built_fixed_point")]
        expected_stagex_lineage_blake3: Option<String>,

        /// Independently authenticated normalized native-provider tree BLAKE3.
        #[arg(long, requires = "source_built_fixed_point")]
        expected_native_provider_blake3: Option<String>,

        /// Explicit bubblewrap executable used by the native construction shell.
        #[arg(long, requires = "source_built_fixed_point")]
        proof_bwrap: Option<PathBuf>,

        /// Explicit static sandbox shell used by native derivations.
        #[arg(long, requires = "source_built_fixed_point")]
        proof_sandbox_shell: Option<PathBuf>,

        /// Maximum proof elapsed time recorded in the immutable plan.
        #[arg(long, default_value_t = SOURCE_BUILT_FIXED_POINT_ELAPSED_SECONDS_MAX_DEFAULT, requires = "source_built_fixed_point")]
        proof_elapsed_seconds_max: u64,

        /// Maximum proof disk bytes recorded in the immutable plan.
        #[arg(long, default_value_t = SOURCE_BUILT_FIXED_POINT_DISK_BYTES_MAX_DEFAULT, requires = "source_built_fixed_point")]
        proof_disk_bytes_max: u64,

        /// Maximum protected exec decisions recorded in the immutable plan.
        #[arg(long, default_value_t = SOURCE_BUILT_FIXED_POINT_PROTECTED_EXEC_EVENTS_MAX_DEFAULT, requires = "source_built_fixed_point")]
        proof_exec_events_max: u32,

        /// Maximum native source records recorded in the immutable plan.
        #[arg(long, default_value_t = SOURCE_BUILT_FIXED_POINT_SOURCE_RECORDS_MAX_DEFAULT, requires = "source_built_fixed_point")]
        proof_source_records_max: u32,

        /// Build Mantle through native Rust topology execution without invoking Cargo.
        #[arg(long)]
        cargo_free: bool,

        /// Run the bounded Cargo-free stage1/stage2 fixed-point proof.
        #[arg(long, requires = "cargo_free")]
        fixed_point: bool,

        /// Output directory for Cargo-free or source-built fixed-point evidence.
        #[arg(long)]
        out: Option<PathBuf>,

        /// rustc executable for --cargo-free topology execution.
        #[arg(long, default_value = "rustc", requires = "cargo_free")]
        rustc: PathBuf,

        /// Source-built toolchain closure manifest for --cargo-free proof input validation.
        #[arg(long, requires = "cargo_free")]
        toolchain_closure: Option<PathBuf>,

        /// Validated source-built Rust provider directory for --cargo-free rustc binding.
        #[arg(long, requires = "cargo_free")]
        rust_source_provider: Option<PathBuf>,

        /// Target triple for --cargo-free rust-plan execution; repeatable.
        #[arg(long = "target", requires = "cargo_free")]
        targets: Vec<String>,
    },

    /// Capture Cargo oracle metadata and unit graph as a normalized Rust package plan receipt
    RustPlan {
        /// Rust workspace root (default: current directory)
        #[arg(long)]
        root: Option<PathBuf>,

        /// Cargo executable to invoke for metadata and unit-graph capture
        #[arg(long, default_value = "cargo")]
        cargo: PathBuf,

        /// rustc executable to invoke for verbose toolchain identity
        #[arg(long, default_value = "rustc")]
        rustc: PathBuf,

        /// Compiler-policy mode for Rust unit execution: plain, audit, deny, or required
        #[arg(long, default_value = "plain")]
        compiler_policy_mode: String,

        /// Compiler-policy provider manifest JSON consumed at the rustc invocation boundary
        #[arg(long)]
        compiler_policy_provider_manifest: Option<PathBuf>,

        /// Expected BLAKE3 digest for the compiler-policy provider manifest
        #[arg(long)]
        compiler_policy_provider_manifest_digest: Option<String>,

        /// Target triple to pass to Cargo; repeatable
        #[arg(long = "target")]
        targets: Vec<String>,

        /// Cargo profile for unit graph capture
        #[arg(long, default_value_t = rust_plan::default_profile())]
        profile: String,

        /// Comma-separated features to enable
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,

        /// Enable all Cargo features
        #[arg(long, conflicts_with = "features")]
        all_features: bool,

        /// Disable default Cargo features
        #[arg(long)]
        no_default_features: bool,

        /// Use Mantle's native Rust planner/executor only; do not invoke Cargo as oracle or
        /// orchestrator
        #[arg(long)]
        no_cargo_oracle: bool,

        /// Receipt-bound source-built C compiler route JSON for provider proof execution
        #[arg(long = "source-built-c-compiler-route-json", hide = true)]
        source_built_c_compiler_route_json: Option<String>,

        /// Normalize source/execution/provider paths for release replay proofs
        #[arg(long = "deterministic-release-paths", hide = true)]
        deterministic_release_paths: bool,

        /// Execute the first supported lib/bin unit from the captured explicit derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_dependency_chain", "execute_target_topology", "execute_host_artifact_topology", "execute_topology", "execute_dev_dependency_test_topology", "execute_workspace_dependency_topology", "execute_patch_source_topology"])]
        execute_first_supported_unit: bool,

        /// Execute the first bounded dependency chain from the captured explicit derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_target_topology", "execute_host_artifact_topology", "execute_topology", "execute_dev_dependency_test_topology", "execute_workspace_dependency_topology", "execute_patch_source_topology"])]
        execute_first_dependency_chain: bool,

        /// Execute a bounded target-only unit topology from the captured explicit derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_host_artifact_topology", "execute_topology", "execute_dev_dependency_test_topology", "execute_workspace_dependency_topology", "execute_patch_source_topology"])]
        execute_target_topology: bool,

        /// Execute a bounded host-artifact unit topology from the captured explicit derivation
        /// graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_target_topology", "execute_topology", "execute_dev_dependency_test_topology", "execute_workspace_dependency_topology", "execute_patch_source_topology"])]
        execute_host_artifact_topology: bool,

        /// Execute a bounded unified host+target unit topology from the captured explicit
        /// derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_target_topology", "execute_host_artifact_topology", "execute_dev_dependency_test_topology", "execute_workspace_dependency_topology", "execute_patch_source_topology"])]
        execute_topology: bool,

        /// Execute a bounded native dev-dependency test topology from explicit receipts
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_target_topology", "execute_host_artifact_topology", "execute_topology", "execute_workspace_dependency_topology", "execute_patch_source_topology"])]
        execute_dev_dependency_test_topology: bool,

        /// Execute a bounded native workspace-dependency topology from explicit receipts
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_target_topology", "execute_host_artifact_topology", "execute_topology", "execute_dev_dependency_test_topology", "execute_patch_source_topology"])]
        execute_workspace_dependency_topology: bool,

        /// Execute a bounded native patch-source topology from explicit receipts
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_target_topology", "execute_host_artifact_topology", "execute_topology", "execute_dev_dependency_test_topology", "execute_workspace_dependency_topology"])]
        execute_patch_source_topology: bool,

        /// Output directory for execution artifacts
        #[arg(long)]
        execution_output_root: Option<PathBuf>,
    },

    /// Enter a development shell from the compatibility-named crunch.ncl devShells
    Shell {
        /// Shell name or .#name selector (default: default.shell or only shell)
        name: Option<String>,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Maximum number of concurrent builds
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Disable remote binary cache substitution
        #[arg(long)]
        no_substitute: bool,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,

        /// Accept unsigned/unverified PathInfo on cache hits
        #[arg(long)]
        trust_unsigned: bool,

        /// Execute a command inside the shell environment and exit
        #[arg(long, conflicts_with = "run_script", num_args = 1.., value_name = "CMD")]
        command: Vec<OsString>,

        /// Pass a script string to $SHELL -c inside the shell environment
        #[arg(long = "run", conflicts_with = "command")]
        run_script: Option<String>,

        /// Layer additional store paths into the shell environment
        #[arg(long = "with")]
        with_paths: Vec<PathBuf>,

        /// Suppress hook execution
        #[arg(long)]
        no_hook: bool,

        /// Make hook failure fatal (exit immediately)
        #[arg(long)]
        strict_hooks: bool,
    },

    /// Alias for `shell` (deprecated, use `crunch shell`)
    Develop {
        /// Shell name or .#name selector (default: default.shell or only shell)
        name: Option<String>,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Maximum number of concurrent builds
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Disable remote binary cache substitution
        #[arg(long)]
        no_substitute: bool,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,

        /// Accept unsigned/unverified PathInfo on cache hits
        #[arg(long)]
        trust_unsigned: bool,
    },

    /// Build and run an executable from a project package or .ncl file
    Run {
        /// Package name or .#name selector
        name: Option<String>,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Maximum number of concurrent builds
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Disable remote binary cache substitution
        #[arg(long)]
        no_substitute: bool,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,

        /// Accept unsigned/unverified PathInfo on cache hits
        #[arg(long)]
        trust_unsigned: bool,

        /// Select a specific executable from the output's bin/ directory
        #[arg(long)]
        bin: Option<String>,

        /// Arguments to pass to the executable (after --)
        #[arg(last = true)]
        run_args: Vec<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum WasmComponentAction {
    /// Evaluate a typed Nickel request and execute every available component stage.
    Build {
        /// Typed Nickel expression producing a ComponentPipelineRequest.
        request: PathBuf,

        /// New output directory for portable artifacts and the execution report.
        #[arg(long)]
        out: PathBuf,

        /// Additional Nickel import path; repeatable.
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Scratch parent; must equal the output parent for atomic publication.
        #[arg(long)]
        scratch_parent: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum FilegenCommandAction {
    /// Render a no-mutate generated-file plan
    Plan {
        /// Project manifest to inspect
        #[arg(long, default_value_os_t = filegen_cmd::default_manifest_path())]
        manifest: PathBuf,

        /// Write the reviewed plan JSON to this path
        #[arg(long = "plan-out")]
        plan_out: Option<PathBuf>,
    },

    /// Apply a reviewed generated-file plan after drift checks
    Apply {
        /// Project manifest to inspect
        #[arg(long, default_value_os_t = filegen_cmd::default_manifest_path())]
        manifest: PathBuf,

        /// Reviewed plan JSON produced by `mantle filegen plan --plan-out`
        #[arg(long)]
        plan: PathBuf,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum ImportAction {
    /// Plan or apply external pin files into Mantle project files
    Pins {
        #[command(subcommand)]
        action: PinImportAction,
    },

    /// Plan or apply a Cargo workspace scaffold for the offline Cargo build lane
    Cargo {
        /// Render the no-mutate plan (default when --apply is absent)
        #[arg(long)]
        plan: bool,

        /// Write the accepted scaffold files if the plan has no blockers
        #[arg(long)]
        apply: bool,

        /// Select a package when the workspace has multiple members
        #[arg(long)]
        package: Option<String>,

        /// Select a binary when the package has multiple binary targets
        #[arg(long)]
        binary: Option<String>,

        /// Generated Mantle project file
        #[arg(long, default_value = "mantle-project.ncl")]
        project_file: String,

        /// Generated Mantle inputs file
        #[arg(long, default_value = ".mantle/inputs.ncl")]
        inputs_file: String,

        /// Cargo target triple for the offline build package
        #[arg(long, default_value = "x86_64-unknown-linux-musl")]
        target: String,

        /// Cargo profile for the generated package
        #[arg(long, default_value = "release")]
        profile: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum PinImportAction {
    /// Render a no-mutate pin import plan
    Plan {
        /// External pin importer to use
        #[arg(long, value_enum, default_value = "nixtamal")]
        importer: PinImporter,

        /// External pin facts file to import
        #[arg(long, default_value = pin_import::PinImportShellOptions::default_pins_file())]
        pins_file: PathBuf,

        /// Planned Mantle project manifest path
        #[arg(long, default_value = crunch_project::PIN_IMPORT_DEFAULT_PROJECT_FILE)]
        project_file: String,

        /// Planned Mantle lockfile path
        #[arg(long, default_value = crunch_project::PIN_IMPORT_DEFAULT_LOCK_FILE)]
        lock_file: String,

        /// Planned generated inputs path
        #[arg(long, default_value = crunch_project::PIN_IMPORT_DEFAULT_INPUTS_FILE)]
        inputs_file: String,
    },

    /// Apply a blocker-free pin import plan immediately after recomputing it
    Apply {
        /// External pin importer to use
        #[arg(long, value_enum, default_value = "nixtamal")]
        importer: PinImporter,

        /// External pin facts file to import
        #[arg(long, default_value = pin_import::PinImportShellOptions::default_pins_file())]
        pins_file: PathBuf,

        /// Planned Mantle project manifest path
        #[arg(long, default_value = crunch_project::PIN_IMPORT_DEFAULT_PROJECT_FILE)]
        project_file: String,

        /// Planned Mantle lockfile path
        #[arg(long, default_value = crunch_project::PIN_IMPORT_DEFAULT_LOCK_FILE)]
        lock_file: String,

        /// Planned generated inputs path
        #[arg(long, default_value = crunch_project::PIN_IMPORT_DEFAULT_INPUTS_FILE)]
        inputs_file: String,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum PinImporter {
    Nixtamal,
    Flake,
    Npins,
    Niv,
}

impl PinImporter {
    fn as_str(self) -> &'static str {
        match self {
            PinImporter::Nixtamal => pin_import::PIN_IMPORTER_NIXTAMAL,
            PinImporter::Flake => pin_import::PIN_IMPORTER_FLAKE,
            PinImporter::Npins => pin_import::PIN_IMPORTER_NPINS,
            PinImporter::Niv => pin_import::PIN_IMPORTER_NIV,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum RefactorAction {
    /// Show available structured refactor sessions
    List,
    /// Plan/check a structured refactor session without mutating files
    Plan {
        /// Refactor session id
        session: String,
        /// Project root to inspect (default: current directory)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Store prefixes to validate, repeatable
        #[arg(long = "refactor-store-prefix")]
        store_prefixes: Vec<String>,
    },
    /// Alias for plan; exits non-zero when conflicts exist
    Check {
        /// Refactor session id
        session: String,
        /// Project root to inspect (default: current directory)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Store prefixes to validate, repeatable
        #[arg(long = "refactor-store-prefix")]
        store_prefixes: Vec<String>,
    },
    /// Explicitly apply a bounded structured refactor session
    Apply {
        /// Refactor session id; required to avoid accidental migration
        session: Option<String>,
        /// Project root to mutate (default: current directory)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Store prefixes to validate, repeatable
        #[arg(long = "refactor-store-prefix")]
        store_prefixes: Vec<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum TranscriptAction {
    /// Run a Markdown executable transcript
    Run {
        /// Transcript markdown file to execute
        transcript: PathBuf,

        /// Output artifact path for transcript evidence (defaults to `<transcript>.output.json`)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Mantle binary to execute for visible `mantle` blocks (defaults to current executable)
        #[arg(long = "mantle-bin")]
        mantle_bin: Option<PathBuf>,

        /// Permit transcripts marked `in_place: true` to run against caller state
        #[arg(long)]
        allow_in_place: bool,
    },
}

#[derive(Subcommand, Debug)]
enum BootstrapAction {
    /// Report executable and unsupported source-root operations from live host observations
    Capabilities,

    /// Produce the deterministic whole-bootstrap parity gap report
    ParityReport {
        /// Fail closed unless the requested parity axes are complete
        #[arg(long = "require")]
        require: Vec<bootstrap_parity::ParityAxis>,
    },

    /// Materialize or import a source-built Rust provider after validation
    RustSourceProvider {
        /// Source-built Rust provider recipe anchor
        #[arg(long, default_value = "bootstrap/rust-source.ncl")]
        recipe: PathBuf,

        /// Explicit Rust source provider route plan; defaults to recipe-relative
        /// rust-source-plan.ncl
        #[arg(long)]
        route_plan: Option<PathBuf>,

        /// Independently produced full-source native-provider admission report.
        /// When present, materialization must publish a validated binding receipt.
        #[arg(
            long,
            conflicts_with = "import_dir",
            requires_all = ["full_source_host_tools", "full_source_rust_sources"]
        )]
        full_source_admission: Option<PathBuf>,

        /// Receipt-bound Make, CMake, Python, Perl, and BusyBox manifest for the full-source route.
        #[arg(long, conflicts_with = "import_dir", requires = "full_source_admission")]
        full_source_host_tools: Option<PathBuf>,

        /// Directory of verified `<source-id>.tar.gz` archives; full-source mode forbids network
        /// fetches.
        #[arg(long, conflicts_with = "import_dir", requires = "full_source_admission")]
        full_source_rust_sources: Option<PathBuf>,

        /// Existing provider directory to validate and import instead of materializing
        #[arg(long)]
        import_dir: Option<PathBuf>,

        /// Compile a provider-local no_std smoke crate after validation/import
        #[arg(long)]
        smoke: bool,

        /// Directory where provider smoke stdout/stderr, metadata, output, and JSON summary are
        /// written
        #[arg(long, requires = "smoke")]
        smoke_evidence_dir: Option<PathBuf>,

        /// Planned output directory for the Rust provider
        #[arg(long)]
        output_dir: PathBuf,

        /// Create and retain the materialization work directory for source and stage evidence.
        #[arg(long, conflicts_with = "import_dir")]
        scratch_dir: Option<PathBuf>,
    },

    /// Materialize receipt-bound Make, CMake, Python, Perl, and BusyBox evidence.
    FullSourceRustHostTools {
        #[arg(long)]
        admission_report: PathBuf,
        #[arg(long)]
        make_root: PathBuf,
        #[arg(long)]
        make_attestation: PathBuf,
        #[arg(long)]
        cmake_root: PathBuf,
        #[arg(long)]
        cmake_attestation: PathBuf,
        #[arg(long)]
        python_root: PathBuf,
        #[arg(long)]
        python_attestation: PathBuf,
        #[arg(long)]
        perl_root: PathBuf,
        #[arg(long)]
        perl_attestation: PathBuf,
        #[arg(long)]
        busybox_root: PathBuf,
        #[arg(long)]
        busybox_attestation: PathBuf,
        #[arg(long)]
        linux_headers_root: PathBuf,
        #[arg(long)]
        linux_headers_attestation: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Bind a normalized full-source C/C++ provider to an expected BLAKE3 identity
    FullSourceProviderAdmit {
        /// Materialized normalized provider directory
        #[arg(long)]
        provider_dir: PathBuf,

        /// Independently authenticated expected provider-tree BLAKE3
        #[arg(long)]
        expected_output_blake3: String,

        /// Complete materialized fixed-fetch source closure for this provider graph
        #[arg(long)]
        source_closure: PathBuf,

        /// Independently authenticated expected source-closure manifest BLAKE3
        #[arg(long)]
        expected_source_closure_blake3: String,

        /// New admission report path; existing files are never replaced
        #[arg(long)]
        report: PathBuf,

        /// Adopt the independently admitted provider into the selected local store state.
        #[arg(long)]
        adopt_state: bool,
    },

    /// Materialize a zero-seed source-built native toolchain closure manifest
    NativeToolchainClosure {
        /// Validated source-built Rust provider directory
        #[arg(long)]
        rust_source_provider: PathBuf,

        /// Source-built host-native provider root containing cc/ld/libc/sysroot closure
        #[arg(long)]
        host_root: PathBuf,

        /// Source-built target musl provider root containing target-prefixed helpers
        #[arg(long)]
        target_root: PathBuf,

        /// Output manifest path; must not already exist
        #[arg(long)]
        output: PathBuf,
    },

    /// Run build-profile preflight, build a bootstrap derivation, and save evidence
    Validate {
        /// Bootstrap .ncl derivation to validate
        target: PathBuf,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Directory where doctor/build/summary evidence is written
        #[arg(long)]
        evidence_dir: Option<PathBuf>,

        /// Bootstrap .ncl prerequisites to build before the target, reusing the same store/state
        #[arg(long = "warmup")]
        warmups: Vec<PathBuf>,

        /// Reuse the existing state/store directories instead of treating this as a fresh run
        #[arg(long)]
        resume: bool,

        /// Maximum number of concurrent builds
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Reject degraded hermetic behavior once strict-mode blockers exist.
        #[arg(long, conflicts_with = "impure")]
        strict_hermetic: bool,

        /// Permit ambient host dependencies; outputs are not proof-eligible.
        #[arg(long, conflicts_with = "strict_hermetic")]
        impure: bool,
    },
}

// Release commands retain full evidence selections so clap and release JSON compatibility remain
// explicit.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug, Clone)]
pub enum ReleaseAction {
    /// Create a release evidence bundle from local artifacts
    Create {
        /// Release identifier recorded in the bundle manifest
        #[arg(long)]
        release_id: String,

        /// Output directory for the bundle (default: target/release-evidence/<release-id>)
        #[arg(long)]
        bundle_dir: Option<PathBuf>,

        /// Release binary artifact to bundle (repeat for multiple binaries)
        #[arg(long = "binary", required = true)]
        binary: Vec<PathBuf>,

        /// Full self-hosting proof bundle directory from ./scripts/prove-self-hosting.sh
        #[arg(long)]
        proof_bundle: PathBuf,

        /// Provider-backed Cargo-free fixed-point proof bundle to package as bounded
        /// release-adjacent evidence
        #[arg(long)]
        provider_fixed_point_proof: Option<PathBuf>,

        /// Optional canonical reproducibility report to package in the bundle
        #[arg(long)]
        reproducibility_report: Option<PathBuf>,

        /// External source archive URL witnesses can fetch before rebuilding
        #[arg(long, conflicts_with = "git_source_url")]
        source_acquisition_url: Option<String>,

        /// Git remote URL witnesses can fetch to derive the release source archive
        #[arg(long, requires = "git_source_commit")]
        git_source_url: Option<String>,

        /// Pinned Git commit for --git-source-url
        #[arg(long, requires = "git_source_url")]
        git_source_commit: Option<String>,

        /// Optional Git ref that must resolve to --git-source-commit during witness replay
        #[arg(long, requires = "git_source_url")]
        git_source_ref: Option<String>,

        /// Optional Git tag that must resolve to --git-source-commit during witness replay
        #[arg(long, requires = "git_source_url")]
        git_source_tag: Option<String>,

        /// Workflow command identity recorded in the manifest
        #[arg(long, default_value = "./scripts/prove-self-hosting.sh")]
        workflow_command: String,

        /// Cairn handoff descriptor whose referenced artifact and policy bytes are measured,
        /// bundled, and bound to this exact release
        #[arg(long = "cairn-handoff")]
        cairn_handoff: Option<PathBuf>,

        /// External evidence sidecar file to bundle opaquely (repeat with matching
        /// role/schema/scope)
        #[arg(long = "external-evidence")]
        external_evidence: Vec<PathBuf>,

        /// Role for each --external-evidence entry, e.g. stack-provenance-trace
        #[arg(long = "external-evidence-role")]
        external_evidence_role: Vec<String>,

        /// Schema identifier for each --external-evidence entry
        #[arg(long = "external-evidence-schema")]
        external_evidence_schema: Vec<String>,

        /// Claim scope for each --external-evidence entry
        #[arg(long = "external-evidence-claim-scope")]
        external_evidence_claim_scope: Vec<String>,

        /// Non-claim text applied to bundled external evidence entries
        #[arg(long = "external-evidence-non-claim")]
        external_evidence_non_claim: Vec<String>,

        /// Kani toolchain identity metadata JSON linked to a bundled Kani receipt
        #[arg(long = "kani-toolchain-evidence")]
        kani_toolchain_evidence: Vec<PathBuf>,

        /// Valence stack-provenance sidecar to package as bounded external evidence
        #[arg(long = "stack-provenance-sidecar", requires = "stack_provenance_valence_receipt")]
        stack_provenance_sidecar: Option<PathBuf>,

        /// Valence stack-provenance graph report/receipt for the sidecar
        #[arg(long = "stack-provenance-valence-receipt", requires = "stack_provenance_sidecar")]
        stack_provenance_valence_receipt: Option<PathBuf>,

        /// Release binary input this stack-provenance sidecar names; required when multiple
        /// binaries are bundled
        #[arg(long = "stack-provenance-binary", requires = "stack_provenance_sidecar")]
        stack_provenance_binary: Option<PathBuf>,

        /// Capability root for content-bound requirement input and evidence files
        #[arg(long = "requirement-evidence-root", requires = "requirement_evidence_input")]
        requirement_evidence_root: Option<PathBuf>,

        /// Capability-relative content-bound requirement creation input
        #[arg(long = "requirement-evidence-input", requires = "requirement_evidence_root")]
        requirement_evidence_input: Option<String>,

        /// Workflow version recorded in the manifest
        #[arg(long, default_value = "mantle-self-hosting-proof-v2")]
        workflow_version: String,
    },
    /// Verify a release evidence bundle using bundle-local contents only
    Verify {
        /// Bundle directory to verify
        bundle_dir: PathBuf,

        /// Fail unless a verified reproducibility report is present and matched
        #[arg(long)]
        require_reproducible: bool,

        /// Require StageX-class lineage proof with no-quorum profile
        #[arg(long)]
        require_stagex_no_quorum: bool,

        /// Canonical deterministic build proof artifact from `release reproduce`
        #[arg(long)]
        deterministic_proof: Option<PathBuf>,

        /// Canonical deterministic sandbox isolation evidence artifact from `release reproduce`
        #[arg(long)]
        deterministic_sandbox_isolation_evidence: Option<PathBuf>,

        /// Fail unless deterministic proof artifacts prove release-artifact determinism
        #[arg(long)]
        require_deterministic_release: bool,

        /// Cargo-free provider fixed-point proof bundle to validate as bounded release-adjacent
        /// evidence
        #[arg(long)]
        provider_fixed_point_proof: Option<PathBuf>,

        /// Fail unless a bundled external evidence entry with this role is present (repeatable)
        #[arg(long = "require-external-evidence-role")]
        require_external_evidence_role: Vec<String>,

        /// Fail unless a valid provider fixed-point proof bundle is supplied
        #[arg(long)]
        require_provider_fixed_point_proof: bool,

        /// Release verification profile; onix-stack requires Valence stack-provenance evidence
        #[arg(long = "release-profile", default_value = "generic")]
        release_profile: String,

        /// Stack-provenance policy for generic releases; release-profile can require a stricter
        /// mode
        #[arg(long = "stack-provenance", value_parser = ["optional", "required"], default_value = "optional")]
        stack_provenance: String,

        /// Content-bound requirement coverage policy for this verification
        #[arg(long = "requirement-coverage", value_parser = ["optional", "required"], default_value = "optional")]
        requirement_coverage: String,
    },
    /// Bind bundle-local function-address evidence into a Cairn-ready Mantle receipt
    FunctionAddressBind {
        /// Verified release evidence bundle containing the selected external evidence rows
        bundle_dir: PathBuf,

        /// Function-address policy mode recorded in the receipt
        #[arg(long, value_parser = ["optional", "required"], default_value = "optional")]
        mode: String,

        /// Render the canonical Preserves-backed function-address binding already declared by the
        /// manifest
        #[arg(long = "from-preserves-binding")]
        from_preserves_binding: bool,

        /// Bundle-relative function-address sidecar path declared in external evidence
        #[arg(
            long,
            required_unless_present = "from_preserves_binding",
            conflicts_with = "from_preserves_binding"
        )]
        sidecar: Option<String>,

        /// Bundle-relative Valence receipt path declared in external evidence
        #[arg(
            long = "valence-receipt",
            required_unless_present = "from_preserves_binding",
            conflicts_with = "from_preserves_binding"
        )]
        valence_receipt: Option<String>,

        /// Bundle-relative Kamacite receipt path declared in external evidence
        #[arg(long = "kamacite-receipt", conflicts_with = "from_preserves_binding")]
        kamacite_receipt: Option<String>,

        /// Bundle-relative release binary path; required when the bundle has multiple binaries
        #[arg(long = "release-binary", conflicts_with = "from_preserves_binding")]
        release_binary: Option<String>,

        /// New output path for the canonical Mantle binding receipt
        #[arg(long = "receipt-out")]
        receipt_out: PathBuf,
    },
    /// Rebuild and compare published release artifacts, then write a reproducibility report
    Reproduce {
        /// Bundle directory to verify and reproduce
        bundle_dir: PathBuf,

        /// Empty output directory where the rebuild command writes reproduced bundle artifact paths
        #[arg(long)]
        rebuild_output_dir: PathBuf,

        /// Rebuild command to execute with CRUNCH_REPRODUCE_* environment variables
        #[arg(long)]
        rebuild_command: PathBuf,

        /// Additional argument passed to the rebuild command (repeatable)
        #[arg(long = "rebuild-arg")]
        rebuild_args: Vec<OsString>,

        /// Workflow version recorded in the reproducibility report
        #[arg(long, default_value = "mantle-release-reproducibility-v1")]
        workflow_version: String,

        /// Output path for the canonical reproducibility report
        #[arg(long)]
        report_path: Option<PathBuf>,

        /// Run a repeated clean-store deterministic proof attempt after the main rebuild (0
        /// disables)
        #[arg(long, default_value_t = 0)]
        deterministic_proof_runs: u32,

        /// Empty directory for repeated deterministic proof run work areas
        #[arg(long)]
        deterministic_proof_dir: Option<PathBuf>,
    },
    /// Evaluate a digest-bound universe before making any global reproducibility claim
    GlobalReproducibility {
        /// Global reproducibility universe manifest JSON
        #[arg(long)]
        universe: PathBuf,

        /// Global reproducibility policy JSON
        #[arg(long)]
        policy: PathBuf,

        /// Surface evidence JSON file; may be repeated and may contain one object or an array
        #[arg(long = "evidence")]
        evidence: Vec<PathBuf>,

        /// Optional path where the canonical report JSON is written
        #[arg(long)]
        report_path: Option<PathBuf>,
    },
    /// Derive global reproducibility surface evidence from a verified release bundle
    GlobalReproducibilityEvidence {
        /// Global reproducibility universe manifest JSON
        #[arg(long)]
        universe: PathBuf,

        /// Global reproducibility policy JSON
        #[arg(long)]
        policy: PathBuf,

        /// Release evidence bundle directory
        #[arg(long)]
        bundle_dir: PathBuf,

        /// Release verification directory containing release/witness attestations
        #[arg(long)]
        verification_dir: PathBuf,

        /// JSON output from `mantle attest release-verify --json`
        #[arg(long)]
        release_verify_json: PathBuf,

        /// Output path for generated surface evidence JSON
        #[arg(long)]
        evidence_path: PathBuf,
    },
    /// Validate, canonicalize, and aggregate reproducibility gauntlet evidence
    Gauntlet {
        #[command(subcommand)]
        action: ReleaseGauntletAction,
    },
    /// Compare release bundle artifacts against located Nix-built artifacts and emit a Nix witness
    /// receipt
    NixWitness {
        /// Bundle directory whose release artifacts are the Mantle side of the comparison
        bundle_dir: PathBuf,

        /// Directory containing Nix-built artifacts at the same relative paths as the release
        /// bundle binaries
        #[arg(long)]
        nix_output_dir: PathBuf,

        /// Canonical deterministic build proof artifact proving the Mantle side first
        #[arg(long)]
        deterministic_proof: PathBuf,

        /// Output path for the canonical Nix cross-builder witness receipt
        #[arg(long)]
        receipt_path: Option<PathBuf>,

        /// Rust toolchain identity shared by the recorded policy
        #[arg(long)]
        rust_toolchain_identity: String,

        /// Target triple shared by the recorded policy
        #[arg(long)]
        target_triple: String,

        /// Ordered build flag/RUSTFLAGS entry recorded in the policy (repeatable, order-sensitive)
        #[arg(long = "build-flag", allow_hyphen_values = true)]
        build_flags: Vec<String>,

        /// Linker identity recorded in the policy, when known
        #[arg(long)]
        linker_identity: Option<String>,

        /// Strip/debug normalization policy
        #[arg(long, default_value = "recorded-release-policy")]
        strip_debug_policy: String,

        /// SOURCE_DATE_EPOCH normalization policy
        #[arg(long, default_value = "recorded-release-policy")]
        source_date_epoch_policy: String,

        /// Nix derivation identity used for the cross-builder comparison
        #[arg(long)]
        nix_derivation_identity: String,

        /// Nix output/store identity used for the cross-builder comparison
        #[arg(long)]
        nix_output_identity: String,

        /// Fail if the Nix-built artifact digest set does not exactly match the Mantle release
        /// artifacts
        #[arg(long)]
        require_match: bool,
    },
    /// Create and sign a release attestation for a verified release bundle
    Attest {
        /// Bundle directory to attest
        bundle_dir: PathBuf,

        /// Output directory for attestation sidecars (default:
        /// target/release-verification/<release-id>)
        #[arg(long)]
        verification_dir: Option<PathBuf>,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,
    },
    /// Export a portable witness-request directory from verified public artifacts
    WitnessExport {
        /// Verified release-evidence bundle directory to export
        bundle_dir: PathBuf,

        /// Verification directory containing the signed release attestation
        #[arg(long)]
        verification_dir: Option<PathBuf>,

        /// Output directory for the witness request (default:
        /// target/release-witness-requests/<release-id>)
        #[arg(long)]
        request_dir: Option<PathBuf>,
    },
    /// Replay an exported witness request into witness sidecars and rebuild audit evidence
    WitnessRebuild {
        /// Exported witness request directory produced by `crunch release witness-export`
        request_dir: PathBuf,

        /// Scratch root for the rebuild work area (default: $MANTLE_WITNESS_SCRATCH_DIR or
        /// <request-dir>.work)
        #[arg(long)]
        scratch_dir: Option<PathBuf>,

        /// Run request validation and workflow preflight only, without rebuilding or writing
        /// sidecars
        #[arg(long)]
        check: bool,

        /// Require source acquisition metadata and fetch the source archive independently
        /// before rebuilding
        #[arg(long, conflicts_with = "require_git_source")]
        require_independent_source: bool,

        /// Require Git source metadata and derive the source archive from the declared Git origin
        /// before rebuilding
        #[arg(long)]
        require_git_source: bool,

        /// Stable witness identity recorded in the attestation (default: signer key name)
        #[arg(long)]
        identity: Option<String>,

        /// Short target-system label for the rebuild environment summary
        #[arg(long)]
        system: Option<String>,

        /// Short toolchain label for the rebuild environment summary
        #[arg(long)]
        toolchain: Option<String>,

        /// Short host-class label for the rebuild environment summary
        #[arg(long)]
        host_class: Option<String>,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ArtifactAction {
    /// Validate and receipt a spec-admitted frontend artifact export
    Export {
        /// Artifact ref to export, e.g. mantle://blake3/...
        #[arg(long = "artifact-ref")]
        artifact_ref: String,

        /// JSON frontend artifact admission attestation or sidecar
        #[arg(long)]
        attestation: PathBuf,

        /// Already-materialized artifact path; skips storage lookup when provided
        #[arg(long = "materialized-path")]
        materialized_path: Option<PathBuf>,

        /// Storage-backed export destination path when --materialized-path is not provided
        #[arg(long = "out")]
        out: Option<PathBuf>,

        /// Expected artifact digest, formatted as blake3:<lowercase-hex>
        #[arg(long = "artifact-digest")]
        artifact_digest: String,

        /// Expected frontend spec id
        #[arg(long = "spec-id")]
        spec_id: Option<String>,

        /// Expected frontend spec version
        #[arg(long = "spec-version")]
        spec_version: Option<String>,

        /// Expected frontend spec BLAKE3 hash
        #[arg(long = "spec-hash")]
        spec_hash: Option<String>,

        /// Export destination mode recorded in the receipt
        #[arg(long = "destination-mode", default_value = frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY)]
        destination_mode: String,

        /// Content provenance entries in KEY=VALUE form; may be repeated
        #[arg(long = "content-provenance")]
        content_provenance: Vec<String>,

        /// Write the export receipt JSON to this path
        #[arg(long = "receipt-out")]
        receipt_out: Option<PathBuf>,
    },

    /// Import local content into Mantle's frontend artifact store
    Import {
        /// File, symlink, or directory to import
        path: PathBuf,

        /// Write the import report JSON to this path
        #[arg(long = "report-out")]
        report_out: Option<PathBuf>,
    },

    /// Project admitted frontend objects into an atomic local OCI image layout.
    OciExport {
        /// Sealed `mantle-oci-projection-v1` document.
        #[arg(long)]
        projection: PathBuf,

        /// Exact frontend specification material named by the admission binding.
        #[arg(long = "spec-material")]
        spec_material: PathBuf,

        /// Exact source admission bundle reduced into the sealed projection.
        #[arg(long = "source-admissions")]
        source_admissions: PathBuf,

        /// New OCI image-layout directory; existing destinations are rejected.
        #[arg(long)]
        out: PathBuf,
    },

    /// Verify and admit a local OCI image layout into the Mantle object store.
    OciImport {
        /// Local non-symlink OCI image-layout directory.
        #[arg(long)]
        layout: PathBuf,

        /// Atomic destination for the import/reconstruction report.
        #[arg(long = "report-out")]
        report_out: PathBuf,
    },

    /// Publish an admitted local OCI layout through a bounded registry transport.
    OciPush {
        /// Verified local OCI image-layout directory to publish.
        #[arg(long)]
        layout: PathBuf,

        /// Registry origin; HTTPS is required unless --allow-http is explicit.
        #[arg(long)]
        registry: String,

        /// Lowercase OCI repository path.
        #[arg(long)]
        repository: String,

        /// Image tag; Mantle also publishes metadata and signature companion tags.
        #[arg(long)]
        reference: String,

        /// Typed Nickel trust policy authorizing this repository and signer set.
        #[arg(long = "trust-policy")]
        trust_policy: PathBuf,

        /// Existing Ed25519 signing key; repeat to satisfy multi-key policy.
        #[arg(long = "signing-key", required = true)]
        signing_keys: Vec<PathBuf>,

        /// Optional bounded file containing one bearer token; its path and bytes are not receipted.
        #[arg(long = "bearer-token-file")]
        bearer_token_file: Option<PathBuf>,

        /// Permit explicit plain HTTP, intended only for controlled local registries.
        #[arg(long)]
        allow_http: bool,

        /// New destination for the contracted registry push receipt.
        #[arg(long = "receipt-out")]
        receipt_out: PathBuf,
    },

    /// Pull immutable OCI image and Mantle metadata manifests, then admit the exact layout.
    OciPull {
        /// Registry origin; HTTPS is required unless --allow-http is explicit.
        #[arg(long)]
        registry: String,

        /// Lowercase OCI repository path.
        #[arg(long)]
        repository: String,

        /// Image tag; Mantle also resolves TAG.mantle-metadata.
        #[arg(long)]
        reference: String,

        /// Immutable image-manifest SHA-256 from the push receipt.
        #[arg(long = "expected-manifest-digest")]
        expected_manifest_digest: String,

        /// Immutable Mantle metadata-manifest SHA-256 from the push receipt.
        #[arg(long = "expected-metadata-manifest-digest")]
        expected_metadata_manifest_digest: String,

        /// Immutable Mantle signature-manifest SHA-256 from the push receipt.
        #[arg(long = "expected-signature-manifest-digest")]
        expected_signature_manifest_digest: String,

        /// Typed Nickel trust policy authorizing this repository and signer set.
        #[arg(long = "trust-policy")]
        trust_policy: PathBuf,

        /// Optional bounded file containing one bearer token; its path and bytes are not receipted.
        #[arg(long = "bearer-token-file")]
        bearer_token_file: Option<PathBuf>,

        /// Permit explicit plain HTTP, intended only for controlled local registries.
        #[arg(long)]
        allow_http: bool,

        /// New destination for the reconstructed verified OCI image layout.
        #[arg(long)]
        out: PathBuf,

        /// New destination for the ordinary OCI import/admission report.
        #[arg(long = "report-out")]
        report_out: PathBuf,

        /// New destination for the contracted registry pull receipt.
        #[arg(long = "receipt-out")]
        receipt_out: PathBuf,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ReleaseGauntletAction {
    /// Validate and rewrite a gauntlet report as canonical compact JSON
    Canonicalize {
        /// Report kind to parse and canonicalize
        #[arg(long, value_enum)]
        kind: GauntletReportKind,

        /// Input report JSON
        input: PathBuf,

        /// Output path for canonical JSON; stdout when omitted
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Aggregate current track evidence into a continuous gauntlet report
    Continuous {
        /// Current claim-boundary context JSON
        #[arg(long)]
        context: PathBuf,

        /// Track evidence JSON object or array; repeat for multiple files
        #[arg(long = "track", required = true)]
        track: Vec<PathBuf>,

        /// Output path for the canonical aggregate report
        #[arg(long)]
        report_path: Option<PathBuf>,
    },

    /// Evaluate the strict hermeticity regression suite plan and fixture evidence
    StrictHermeticityRegression {
        /// Stable run identifier for this suite execution
        #[arg(long)]
        run_id: String,

        /// Suite plan JSON
        #[arg(long)]
        plan: PathBuf,

        /// Case evidence JSON object or array; repeat for multiple files
        #[arg(long = "evidence", required = true)]
        evidence: Vec<PathBuf>,

        /// Output path for the canonical suite report
        #[arg(long)]
        report_path: Option<PathBuf>,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GauntletReportKind {
    AdversarialHermeticity,
    BootstrapPressure,
    SubstitutionCacheAttack,
    NixMantleComparison,
    ReleaseRepeatability,
    Continuous,
    StrictHermeticityRegression,
}

impl GauntletReportKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::AdversarialHermeticity => "adversarial-hermeticity",
            Self::BootstrapPressure => "bootstrap-pressure",
            Self::SubstitutionCacheAttack => "substitution-cache-attack",
            Self::NixMantleComparison => "nix-mantle-comparison",
            Self::ReleaseRepeatability => "release-repeatability",
            Self::Continuous => "continuous",
            Self::StrictHermeticityRegression => "strict-hermeticity-regression",
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AttestAction {
    /// Show an artifact attestation for a store path
    Show {
        /// Logical store path, exported store path, or store path string
        path: String,
    },
    /// Assemble and print a runtime closure attestation for one or more roots
    Closure {
        /// Root logical store paths or exported store paths
        roots: Vec<String>,
    },
    /// Verify attestation bytes against canonical reconstruction
    Verify {
        #[command(subcommand)]
        target: AttestVerifyAction,
    },
    /// Diff two attestation documents or artifact selectors
    Diff { left: String, right: String },
    /// Render a project attestation from crunch-project.ncl, crunch.lock, and selected roots
    Project {
        /// Selected built roots for the project view
        roots: Vec<String>,
    },
    /// Show a signed release attestation from a verification directory
    ReleaseShow {
        /// Verification directory containing release-attestation.json
        verification_dir: PathBuf,
    },
    /// Show the trusted public key token for an existing signing keypair
    KeyShow {
        /// Path to a Nix-format ed25519 signing keypair file (defaults to the configured signing
        /// key)
        #[arg(long)]
        signing_key: Option<PathBuf>,
    },
    /// Create and sign a witness attestation under a verification directory
    WitnessCreate {
        /// Verification directory containing release-attestation.json
        verification_dir: PathBuf,

        /// Rebuilt binary outputs in the same order as the published release artifacts
        #[arg(long = "rebuilt-binary", required = true)]
        rebuilt_binary: Vec<PathBuf>,

        /// Stable witness identity recorded in the attestation (default: signer key name)
        #[arg(long)]
        identity: Option<String>,

        /// Short target-system label for the rebuild environment summary
        #[arg(long)]
        system: String,

        /// Short toolchain label for the rebuild environment summary
        #[arg(long)]
        toolchain: String,

        /// Short host-class label for the rebuild environment summary
        #[arg(long)]
        host_class: String,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<PathBuf>,
    },
    /// Show witness attestations from a verification directory
    WitnessShow {
        /// Verification directory containing witnesses/*.json
        verification_dir: PathBuf,

        /// Optional witness identity to show
        identity: Option<String>,
    },
    /// Import returned witness sidecars into a verification directory
    WitnessImport {
        /// Verification directory containing release-attestation.json
        verification_dir: PathBuf,

        /// Source witness json file, witnesses/ directory, or verification dir with witnesses/
        source: PathBuf,
    },
    /// Initialize verifier-local policy and revocation files in a verification directory
    PolicyInit {
        /// Verification directory containing release-attestation.json
        verification_dir: PathBuf,

        /// Named policy profile to scaffold
        #[arg(long, value_enum)]
        profile: AttestPolicyProfileArg,

        /// Trusted release signer names recorded in policy.json
        #[arg(long = "trusted-release-signer", required = true)]
        trusted_release_signer: Vec<String>,

        /// Trusted witness identities recorded in policy.json for witness-count profiles
        #[arg(long = "trusted-witness-identity")]
        trusted_witness_identity: Vec<String>,

        /// Overwrite existing policy.json and revocations.json
        #[arg(long)]
        force: bool,
    },
    /// Verify a release attestation, witness set, and policy from a verification directory
    ReleaseVerify {
        /// Verification directory containing attestation material and policy files
        verification_dir: PathBuf,

        /// Trusted public keys for release and witness signature verification (name:base64)
        #[arg(long = "trusted-public-key", value_delimiter = ',')]
        trusted_public_keys: Vec<String>,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum AttestPolicyProfileArg {
    SelfProofOnly,
    SingleWitness,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AttestVerifyAction {
    /// Verify a persisted artifact attestation
    Artifact { path: String },
    /// Verify a persisted runtime closure attestation
    Closure { roots: Vec<String> },
    /// Verify a synthesized project attestation against a saved envelope/file or expected digest
    Project {
        /// Saved project attestation JSON or envelope to compare against
        #[arg(long, conflicts_with = "digest")]
        file: Option<PathBuf>,

        /// Expected canonical digest hex to compare against
        #[arg(long, conflicts_with = "file")]
        digest: Option<String>,

        /// Selected built roots used to reconstruct the project attestation
        roots: Vec<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RemoteAction {
    /// Ticket management
    Ticket {
        #[command(subcommand)]
        action: RemoteTicketAction,
    },
    /// Print redacted remote queue, worker, and ticket status
    Status {
        /// Builder or coordinator endpoint id for the status snapshot
        #[arg(long, default_value = "local-builder")]
        endpoint_id: String,

        /// Configured server concurrency for the status snapshot
        #[arg(long, default_value_t = remote_build::DEFAULT_REMOTE_CONCURRENCY)]
        concurrency: u32,
    },
    /// Inspect, plan replay, execute replay, or retain remote failure debug bundles
    Debug {
        #[command(subcommand)]
        action: RemoteFailureDebugAction,
    },
    /// Print remote server protocol metadata or serve one framed stdio session
    Serve {
        /// Builder endpoint id expected by clients
        #[arg(long, default_value = "local-builder")]
        endpoint_id: String,

        /// Transport binding to expose
        #[arg(long, value_enum, default_value = "metadata")]
        binding: RemoteServeBinding,

        /// Builder signing key id advertised in fixture framed responses
        #[arg(long, default_value = "builder-key")]
        signing_key_id: String,

        /// Executor used by stdio-once; fixture is deterministic, local-build runs derivation
        /// payloads locally
        #[arg(long, value_enum, default_value = "fixture")]
        executor: RemoteServeExecutor,

        /// Input ref already present on this builder; repeat for multiple refs
        #[arg(long = "present-input-ref")]
        present_input_refs: Vec<String>,

        /// Separate worker execution/CAS state; defaults to the global state directory.
        #[arg(long, hide = true)]
        execution_state_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RemoteFailureDebugAction {
    /// Validate and render one bounded redacted bundle summary without executing anything
    Inspect {
        /// Bundle digest, remote-failure-debug:<digest> reference, or explicit bundle directory
        bundle: String,
    },
    /// Validate and render a side-effect-free replay plan
    ReplayPlan {
        /// Bundle digest, remote-failure-debug:<digest> reference, or explicit bundle directory
        bundle: String,
    },
    /// Execute the stored request as a new fenced attempt through ordinary admission
    Replay {
        /// Bundle digest, remote-failure-debug:<digest> reference, or explicit bundle directory
        bundle: String,
        /// Remote builder endpoint id
        #[arg(long)]
        builder: String,
        /// Bearer ticket as <ticket-id>:<secret>
        #[arg(long)]
        ticket: String,
        /// Program to spawn for stdio dispatch; defaults to this Mantle binary
        #[arg(long)]
        builder_program: Option<PathBuf>,
        /// Argument passed to --builder-program; repeat for multiple arguments
        #[arg(long = "builder-arg")]
        builder_args: Vec<String>,
        /// Trusted remote builder signing key name; repeat for multiple keys
        #[arg(long = "trusted-builder-key")]
        trusted_builder_keys: Vec<String>,
        /// Replay build-time limit in seconds
        #[arg(long, default_value_t = remote_build::DEFAULT_TICKET_BUILD_TIME_SECS)]
        remote_build_time_secs: u64,
    },
    /// Delete only expired unleased remote failure debug roots
    Gc {
        /// Retention decision time; defaults to the current Unix timestamp
        #[arg(long)]
        now_unix_s: Option<u64>,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteServeBinding {
    Metadata,
    StdioOnce,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteServeExecutor {
    Fixture,
    LocalBuild,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RemoteTicketAction {
    /// Create a redacted bearer ticket record
    Create {
        #[arg(long, default_value = "ticket")]
        display_name: String,

        #[arg(long, default_value_t = 1)]
        now_unix_s: u64,

        #[arg(long, default_value_t = remote_build::DEFAULT_TICKET_TTL_SECS)]
        ttl_secs: u64,

        #[arg(long, default_value_t = remote_build::DEFAULT_TICKET_USES)]
        uses: u32,

        #[arg(long, default_value_t = remote_build::DEFAULT_TICKET_BUILD_TIME_SECS)]
        max_build_time_secs: u64,

        #[arg(long, default_value_t = remote_build::DEFAULT_TICKET_UPLOAD_BYTES)]
        max_upload_bytes: u64,

        #[arg(long)]
        bound_client_endpoint: Option<String>,
    },
    /// List tickets with bearer secrets redacted
    List,
    /// Inspect one ticket with bearer secret redacted
    Inspect { id: String },
    /// Reveal one ticket bearer secret explicitly
    Reveal { id: String },
    /// Revoke one ticket
    Revoke { id: String },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ReceiptAction {
    /// Receipt bundle operations
    Bundle {
        #[command(subcommand)]
        action: ReceiptBundleAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ReceiptBundleAction {
    /// Export a portable receipt bundle from explicit evidence records and local output state
    Export {
        /// Record specs as kind:identity:digest
        #[arg(long = "record")]
        records: Vec<String>,

        /// Logical store output path or store-path basename whose local evidence should be gathered
        #[arg(long = "output")]
        outputs: Vec<String>,

        /// Bundle output path
        #[arg(long)]
        to: std::path::PathBuf,

        /// Policy hash or policy identifier bound to the bundle
        #[arg(long, default_value = "policy-unspecified")]
        policy_hash: String,

        /// Require complete strong action-correctness evidence
        #[arg(long)]
        strong: bool,

        /// Trusted public key token name:base64 to snapshot into the bundle; repeat for multiple
        /// keys
        #[arg(long = "trusted-public-key")]
        trusted_public_keys: Vec<String>,
    },
    /// List receipt bundle metadata
    List {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,
    },
    /// Verify receipt bundle structure, trust snapshot, completeness, and optional local output
    /// facts
    Verify {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,

        /// Logical store output path or store-path basename to match against local PathInfo
        #[arg(long = "output")]
        outputs: Vec<String>,

        /// Expected policy hash or policy identifier for output verification
        #[arg(long)]
        policy_hash: Option<String>,

        /// Store archive whose listed output facts should be used instead of local PathInfo
        #[arg(long)]
        archive: Option<std::path::PathBuf>,

        /// Verification timestamp for replayed trust-window checks (defaults to current time)
        #[arg(long)]
        valid_at_unix_s: Option<u64>,

        /// Expected revocation-list reference for replayed trust checks
        #[arg(long)]
        revocation_ref: Option<String>,

        /// Revoked public key digest; repeat for multiple revoked keys
        #[arg(long = "revoked-public-key-digest")]
        revoked_public_key_digests: Vec<String>,

        /// Trusted public key token name:base64 for signature verification; repeat for multiple
        /// keys
        #[arg(long = "trusted-public-key")]
        trusted_public_keys: Vec<String>,
    },
    /// Import a verified receipt bundle into Mantle evidence state
    Import {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,

        /// Logical store output path or store-path basename to verify before import
        #[arg(long = "output")]
        outputs: Vec<String>,

        /// Expected policy hash or policy identifier for verified import
        #[arg(long)]
        policy_hash: Option<String>,

        /// Store archive whose listed output facts should be used instead of local PathInfo
        #[arg(long)]
        archive: Option<std::path::PathBuf>,

        /// Verification timestamp for replayed trust-window checks (defaults to current time)
        #[arg(long)]
        valid_at_unix_s: Option<u64>,

        /// Expected revocation-list reference for replayed trust checks
        #[arg(long)]
        revocation_ref: Option<String>,

        /// Revoked public key digest; repeat for multiple revoked keys
        #[arg(long = "revoked-public-key-digest")]
        revoked_public_key_digests: Vec<String>,

        /// Trusted public key token name:base64 for signature verification; repeat for multiple
        /// keys
        #[arg(long = "trusted-public-key")]
        trusted_public_keys: Vec<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum StoreAction {
    /// List all known store paths
    List,
    /// Show detailed PathInfo for a store path
    Info {
        /// Store path (full or fragment to match)
        path: String,
    },
    /// List retained GC roots
    Roots,
    /// Pin a logical store path as a retained GC root
    Pin {
        /// Logical store path to retain
        path: String,
    },
    /// Remove a retained GC root
    Unpin {
        /// Logical store path to remove
        path: String,
    },
    /// Run manual garbage collection
    Gc {
        /// Preview removals without mutating state
        #[arg(long = "dry-run")]
        is_dry_run: bool,
    },
    /// Verify NAR hash and trusted signatures of stored paths
    Verify {
        /// Optional: verify a specific path (default: all)
        path: Option<String>,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<std::path::PathBuf>,

        /// Trusted public keys for signature verification (name:base64)
        #[arg(long, value_delimiter = ',')]
        trusted_public_keys: Vec<String>,

        /// Accept unsigned/unverified PathInfo
        #[arg(long)]
        trust_unsigned: bool,
    },
    /// Sign PathInfo entries with an ed25519 key
    Sign {
        /// Store path to sign (omit for --all)
        path: Option<String>,

        /// Sign all unsigned PathInfo entries
        #[arg(long)]
        all: bool,

        /// Path to a Nix-format ed25519 signing keypair file
        #[arg(long)]
        signing_key: Option<std::path::PathBuf>,
    },
    /// Repair stale signed final-NAR metadata for one exact local path
    RepairFinalNar {
        /// Exact full logical store path to inspect or repair
        path: String,

        /// Persist freshly measured facts and replace stale signatures
        #[arg(long)]
        execute: bool,

        /// Path to a Nix-format ed25519 signing keypair file (execute mode only)
        #[arg(long)]
        signing_key: Option<std::path::PathBuf>,
    },
    /// Push store paths to a binary cache directory
    Push {
        /// Target directory for the binary cache
        #[arg(long)]
        to: std::path::PathBuf,

        /// Push all signed paths
        #[arg(long)]
        all: bool,

        /// Include unsigned PathInfo entries
        #[arg(long)]
        trust_unsigned: bool,

        /// Store paths to push (full or fragment)
        paths: Vec<String>,
    },
    /// Pull (import) store paths from a binary cache directory or HTTP cache URL
    Pull {
        /// Source binary cache directory or HTTP/HTTPS cache URL
        #[arg(long)]
        from: String,

        /// Import all paths from the cache directory
        #[arg(long)]
        all: bool,

        /// Recursively import one complete signed HTTP runtime closure
        #[arg(long, conflicts_with = "all")]
        closure: bool,

        /// Accept unsigned/unverified narinfos
        #[arg(long)]
        trust_unsigned: bool,

        /// Trusted public keys for signature verification (name:base64)
        #[arg(long, value_delimiter = ',')]
        trusted_public_keys: Vec<String>,

        /// Use selected root paths from a complete foreign realization receipt
        #[arg(long = "foreign-realization-receipt", requires = "closure", conflicts_with_all = ["all", "paths"])]
        foreign_realization_receipt: Option<PathBuf>,

        /// Store paths to import (full or fragment)
        paths: Vec<String>,
    },
    /// Export, import, or list a Mantle-native single-file store archive
    Archive {
        #[command(subcommand)]
        action: StoreArchiveAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SourceAction {
    /// Source bundle operations
    Bundle {
        #[command(subcommand)]
        action: SourceBundleAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SourceBundleAction {
    /// Plan source records without mutating source state
    Plan {
        /// Source specs as kind:identity:path
        #[arg(long = "source")]
        sources: Vec<String>,

        /// Evaluate a .ncl build root and derive source records from fixed fetcher and store-path
        /// inputs
        #[arg(long = "build-root")]
        build_roots: Vec<std::path::PathBuf>,

        /// Additional Nickel import paths for --build-root evaluation
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<std::path::PathBuf>,
    },
    /// Export a source bundle JSON file
    Export {
        /// Source specs as kind:identity:path
        #[arg(long = "source")]
        sources: Vec<String>,

        /// Evaluate a .ncl build root and derive source records from fixed fetcher and store-path
        /// inputs
        #[arg(long = "build-root")]
        build_roots: Vec<std::path::PathBuf>,

        /// Additional Nickel import paths for --build-root evaluation
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<std::path::PathBuf>,

        /// Bundle output path
        #[arg(long)]
        to: std::path::PathBuf,

        /// Explicitly fetch and fixed-output-validate non-local declared sources missing from
        /// imported state
        #[arg(long)]
        fetch_missing: bool,
    },
    /// Build a named bootstrap source-bundle profile from local inputs
    BootstrapProfile {
        /// Profile mode: legacy-seed, source-root, self-build-proof, fresh-clone-inputs,
        /// fresh-clone-fixed-point, or source-built-fixed-point
        #[arg(long, default_value = "legacy-seed")]
        mode: String,

        /// Provider archive payload path
        #[arg(long = "provider-archive")]
        provider_archive: std::path::PathBuf,

        /// Provider manifest JSON path
        #[arg(long = "provider-manifest")]
        provider_manifest: std::path::PathBuf,

        /// Bootstrap source archive/root payload; repeat for multiple inputs
        #[arg(long = "bootstrap-source")]
        bootstrap_sources: Vec<std::path::PathBuf>,

        /// Exact authenticated StageX source-bundle manifest for source-built fixed-point mode
        #[arg(long = "stagex-source-bundle")]
        stagex_source_bundle: Option<std::path::PathBuf>,

        /// Mantle source tree for self-build proof mode
        #[arg(long = "mantle-source")]
        mantle_source: Option<std::path::PathBuf>,

        /// Vendored Cargo input directory for self-build proof mode
        #[arg(long = "vendor-deps")]
        vendor_deps: Option<std::path::PathBuf>,

        /// Toolchain/source-root record for source-root or proof modes
        #[arg(long = "toolchain-source-root")]
        toolchain_source_root: Option<std::path::PathBuf>,

        /// Proof input payload; repeat for multiple proof inputs
        #[arg(long = "proof-input")]
        proof_inputs: Vec<std::path::PathBuf>,

        /// Materialized source bundle whose records are merged into the profile; repeat for
        /// multiple bundles
        #[arg(long = "include-bundle")]
        include_bundles: Vec<std::path::PathBuf>,

        /// Bundle output path. If omitted, print the profile plan report.
        #[arg(long)]
        to: Option<std::path::PathBuf>,

        /// Compare the generated profile with imported/pinned source state
        #[arg(long)]
        preflight: bool,
    },
    /// List a source bundle without mutating state
    List {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,
    },
    /// Import a source bundle into Mantle source state
    Import {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,

        /// Pin imported records for planned use
        #[arg(long)]
        pin: bool,
    },
    /// Hydrate a fresh checkout's explicit self-build inputs from a verified bundle
    HydrateSelfBuild {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,

        /// Expected manifest BLAKE3 obtained independently of the bundle
        #[arg(long)]
        expected_manifest_blake3: String,

        /// Fresh checkout whose absent vendor-deps directory will be published
        #[arg(long)]
        checkout: std::path::PathBuf,
    },
    /// Verify a bundle and optionally imported source state
    Verify {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,

        /// Require matching imported source state
        #[arg(long)]
        imported: bool,
    },
    /// Compare build-root source requirements with imported source state before building
    Preflight {
        /// Evaluate a .ncl build root and derive source records from fixed fetcher and store-path
        /// inputs
        #[arg(long = "build-root")]
        build_roots: Vec<std::path::PathBuf>,

        /// Additional Nickel import paths for --build-root evaluation
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum StoreArchiveAction {
    /// Export selected store paths and recursive closure to an archive
    Export {
        /// Archive output path, or '-' for stdout
        #[arg(long)]
        to: std::path::PathBuf,

        /// Export every known signed path as a root
        #[arg(long)]
        all: bool,

        /// Include unsigned PathInfo entries
        #[arg(long)]
        trust_unsigned: bool,

        /// Store path selectors to export (full or fragment)
        paths: Vec<String>,
    },
    /// Import a Mantle-native store archive
    Import {
        /// Archive input path, or '-' for stdin
        #[arg(long)]
        from: std::path::PathBuf,

        /// Accept unsigned/unverified archive records
        #[arg(long)]
        trust_unsigned: bool,

        /// Trusted public keys for signature verification (name:base64)
        #[arg(long, value_delimiter = ',')]
        trusted_public_keys: Vec<String>,

        /// Do not materialize imported outputs into the physical output store
        #[arg(long)]
        no_materialize: bool,
    },
    /// List archive contents without importing
    List {
        /// Archive input path, or '-' for stdin
        #[arg(long)]
        from: std::path::PathBuf,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();
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

fn run(args: Args) -> Result<(), RunError> {
    apply_state_dir_override(&args);
    let ctx = build_run_context(&args);
    emit_runtime_fingerprint(&args, &ctx)?;
    dispatch_command(&args, &ctx)
}

fn apply_state_dir_override(args: &Args) {
    if let Some(state_dir) = &args.state_dir {
        unsafe { std::env::set_var("CRUNCH_STATE_DIR", state_dir) };
    }
}

fn build_run_context(args: &Args) -> RunContext {
    let store_prefix = resolve_store_prefix(args);
    debug_assert!(!store_prefix.is_empty(), "store_prefix must not be empty");
    debug_assert!(store_prefix.starts_with('/'), "store_prefix must be absolute");
    RunContext {
        store: args.store.clone(),
        resolved_state_dir: state_dir(),
        store_prefix,
        verbose: args.verbose,
        json: args.json,
        base_state_dirs: args.base_stores.clone(),
    }
}

fn emit_runtime_fingerprint(args: &Args, ctx: &RunContext) -> Result<(), RunError> {
    debug_assert_eq!(ctx.json, args.json);
    debug_assert_eq!(ctx.verbose, args.verbose);
    let Some(verbosity_source) = operator_diagnostics::runtime_fingerprint_verbosity_source(
        operator_diagnostics::RuntimeFingerprintTriggerInput {
            verbose: args.verbose,
            log_level: args.log_level.as_deref(),
            diagnostic_mode: false,
        },
    ) else {
        return Ok(());
    };
    let fingerprint = operator_diagnostics::build_runtime_fingerprint(operator_diagnostics::RuntimeFingerprintInput {
        mantle_version: env!("CARGO_PKG_VERSION").to_string(),
        command: command_label(&args.command).to_string(),
        logical_store_prefix: ctx.store_prefix.clone(),
        physical_store_dir: ctx.store.display().to_string(),
        state_dir: ctx.resolved_state_dir.display().to_string(),
        json_mode: ctx.json,
        verbosity_source,
        modes: runtime_fingerprint_mode_fields(args),
    });
    let rendered = operator_diagnostics::render_runtime_fingerprint(&fingerprint)
        .map_err(|err| RunError::Internal(format!("rendering runtime fingerprint: {err}")))?;
    eprintln!("{rendered}");
    Ok(())
}

fn command_label(command: &Command) -> &'static str {
    match command {
        Command::Build { .. } => "build",
        Command::WasmComponent { .. } => "wasm-component.build",
        Command::Doctor { .. } => "doctor",
        Command::Import { .. } => "import",
        Command::Filegen { action } => filegen_command_label(action),
        Command::Graph { .. } => "graph",
        Command::Why { .. } => "why",
        Command::Dependents { .. } => "dependents",
        Command::Refactor { .. } => "refactor",
        Command::Transcript { .. } => "transcript",
        Command::Stage0Inventory { .. } => "stage0-inventory",
        Command::NixFreeDemo { .. } => "nix-free-demo",
        Command::ForeignImport { .. } => "foreign-import",
        Command::Mantlepkgs { .. } => "mantlepkgs",
        Command::Eval { .. } => "eval",
        Command::Export { .. } => "export",
        Command::Bootstrap { action, .. } => bootstrap_command_label(action.as_ref()),
        Command::Log { .. } => "log",
        Command::Store { action } => store_command_label(action),
        Command::Source { action } => source_command_label(action),
        Command::Receipt { action } => receipt_command_label(action),
        Command::Remote { action } => remote_command_label(action),
        Command::Artifact { .. } => "artifact",
        Command::Attest { .. } => "attest",
        Command::Release { .. } => "release",
        Command::Init => "init",
        Command::Check { .. } => "check",
        Command::Show => "show",
        Command::Refresh { .. } => "refresh",
        Command::ListStale { .. } => "list-stale",
        Command::Upgrade => "upgrade",
        Command::SelfBuild { .. } => "self-build",
        Command::RustPlan { .. } => "rust-plan",
        Command::Shell { .. } => "shell",
        Command::Develop { .. } => "develop",
        Command::Run { .. } => "run",
    }
}

fn filegen_command_label(action: &FilegenCommandAction) -> &'static str {
    match action {
        FilegenCommandAction::Plan { .. } => "filegen.plan",
        FilegenCommandAction::Apply { .. } => "filegen.apply",
    }
}

fn bootstrap_command_label(action: Option<&BootstrapAction>) -> &'static str {
    match action {
        Some(BootstrapAction::Capabilities) => "bootstrap.capabilities",
        Some(BootstrapAction::ParityReport { .. }) => "bootstrap.parity-report",
        Some(BootstrapAction::RustSourceProvider { .. }) => "bootstrap.rust-source-provider",
        Some(BootstrapAction::FullSourceRustHostTools { .. }) => "bootstrap.full-source-rust-host-tools",
        Some(BootstrapAction::FullSourceProviderAdmit { .. }) => "bootstrap.full-source-provider-admit",
        Some(BootstrapAction::NativeToolchainClosure { .. }) => "bootstrap.native-toolchain-closure",
        Some(BootstrapAction::Validate { .. }) => "bootstrap.validate",
        None => "bootstrap",
    }
}

fn remote_command_label(action: &RemoteAction) -> &'static str {
    match action {
        RemoteAction::Ticket { action } => remote_ticket_command_label(action),
        RemoteAction::Status { .. } => "remote.status",
        RemoteAction::Debug { action } => match action {
            RemoteFailureDebugAction::Inspect { .. } => "remote.debug.inspect",
            RemoteFailureDebugAction::ReplayPlan { .. } => "remote.debug.replay-plan",
            RemoteFailureDebugAction::Replay { .. } => "remote.debug.replay",
            RemoteFailureDebugAction::Gc { .. } => "remote.debug.gc",
        },
        RemoteAction::Serve { .. } => "remote.serve",
    }
}

fn remote_ticket_command_label(action: &RemoteTicketAction) -> &'static str {
    match action {
        RemoteTicketAction::Create { .. } => "remote.ticket.create",
        RemoteTicketAction::List => "remote.ticket.list",
        RemoteTicketAction::Inspect { .. } => "remote.ticket.inspect",
        RemoteTicketAction::Reveal { .. } => "remote.ticket.reveal",
        RemoteTicketAction::Revoke { .. } => "remote.ticket.revoke",
    }
}

fn receipt_command_label(action: &ReceiptAction) -> &'static str {
    match action {
        ReceiptAction::Bundle { action } => receipt_bundle_command_label(action),
    }
}

fn receipt_bundle_command_label(action: &ReceiptBundleAction) -> &'static str {
    match action {
        ReceiptBundleAction::Export { .. } => "receipt.bundle.export",
        ReceiptBundleAction::List { .. } => "receipt.bundle.list",
        ReceiptBundleAction::Verify { .. } => "receipt.bundle.verify",
        ReceiptBundleAction::Import { .. } => "receipt.bundle.import",
    }
}

fn source_command_label(action: &SourceAction) -> &'static str {
    match action {
        SourceAction::Bundle { action } => source_bundle_command_label(action),
    }
}

fn source_bundle_command_label(action: &SourceBundleAction) -> &'static str {
    match action {
        SourceBundleAction::Plan { .. } => "source.bundle.plan",
        SourceBundleAction::Export { .. } => "source.bundle.export",
        SourceBundleAction::BootstrapProfile { .. } => "source.bundle.bootstrap-profile",
        SourceBundleAction::List { .. } => "source.bundle.list",
        SourceBundleAction::Import { .. } => "source.bundle.import",
        SourceBundleAction::HydrateSelfBuild { .. } => "source.bundle.hydrate-self-build",
        SourceBundleAction::Verify { .. } => "source.bundle.verify",
        SourceBundleAction::Preflight { .. } => "source.bundle.preflight",
    }
}

fn store_command_label(action: &StoreAction) -> &'static str {
    match action {
        StoreAction::List => "store.list",
        StoreAction::Info { .. } => "store.info",
        StoreAction::Roots => "store.roots",
        StoreAction::Pin { .. } => "store.pin",
        StoreAction::Unpin { .. } => "store.unpin",
        StoreAction::Gc { .. } => "store.gc",
        StoreAction::Verify { .. } => "store.verify",
        StoreAction::Sign { .. } => "store.sign",
        StoreAction::RepairFinalNar { .. } => "store.repair-final-nar",
        StoreAction::Push { .. } => "store.push",
        StoreAction::Pull { .. } => "store.pull",
        StoreAction::Archive { action } => store_archive_command_label(action),
    }
}

fn store_archive_command_label(action: &StoreArchiveAction) -> &'static str {
    match action {
        StoreArchiveAction::Export { .. } => "store.archive.export",
        StoreArchiveAction::Import { .. } => "store.archive.import",
        StoreArchiveAction::List { .. } => "store.archive.list",
    }
}

#[derive(Debug, Clone, Copy)]
struct HermeticitySelection {
    strict_hermetic: bool,
    impure: bool,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeBuildModeInput<'a> {
    hermeticity: HermeticitySelection,
    no_substitute: bool,
    substituters: Option<&'a str>,
    signing_key_selected: bool,
    trusted_public_key_count: usize,
    trust_unsigned: bool,
}

fn runtime_fingerprint_mode_fields(args: &Args) -> RuntimeFingerprintModeFields {
    let mut modes = RuntimeFingerprintModeFields {
        nix_compat_mode: args.nix_compat.then_some(true),
        ..RuntimeFingerprintModeFields::default()
    };
    apply_runtime_fingerprint_command_fields(&mut modes, args);
    modes
}

fn apply_runtime_fingerprint_command_fields(modes: &mut RuntimeFingerprintModeFields, args: &Args) {
    debug_assert_eq!(modes.nix_compat_mode.is_some(), args.nix_compat);
    debug_assert!(modes.substituter_count.is_none());
    if apply_primary_build_fingerprint_fields(modes, &args.command) {
        return;
    }
    if apply_interactive_build_fingerprint_fields(modes, &args.command) {
        return;
    }
    apply_remaining_fingerprint_fields(modes, &args.command);
}

fn apply_primary_build_fingerprint_fields(modes: &mut RuntimeFingerprintModeFields, command: &Command) -> bool {
    debug_assert!(modes.hermeticity_mode.is_none());
    debug_assert!(modes.substitution_mode.is_none());
    match command {
        Command::Build {
            substituters,
            no_substitute,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
            impure,
            builder,
            ticket,
            ..
        } => {
            apply_build_mode_fields(modes, RuntimeBuildModeInput {
                hermeticity: HermeticitySelection {
                    strict_hermetic: *strict_hermetic,
                    impure: *impure,
                },
                no_substitute: *no_substitute,
                substituters: Some(substituters),
                signing_key_selected: signing_key.is_some(),
                trusted_public_key_count: trusted_public_keys.len(),
                trust_unsigned: *trust_unsigned,
            });
            if builder.is_some() || ticket.is_some() {
                modes.bearer_ticket_count =
                    Some(operator_diagnostics::bounded_runtime_count(usize::from(ticket.is_some())));
            }
            true
        }
        Command::SelfBuild {
            no_substitute,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
            impure,
            ..
        } => {
            apply_build_mode_fields(modes, RuntimeBuildModeInput {
                hermeticity: HermeticitySelection {
                    strict_hermetic: *strict_hermetic,
                    impure: *impure,
                },
                no_substitute: *no_substitute,
                substituters: None,
                signing_key_selected: signing_key.is_some(),
                trusted_public_key_count: trusted_public_keys.len(),
                trust_unsigned: *trust_unsigned,
            });
            true
        }
        _ => false,
    }
}

fn apply_interactive_build_fingerprint_fields(modes: &mut RuntimeFingerprintModeFields, command: &Command) -> bool {
    debug_assert!(modes.hermeticity_mode.is_none());
    debug_assert!(modes.substitution_mode.is_none());
    let (no_substitute, signing_key_selected, trust_unsigned) = match command {
        Command::Shell {
            no_substitute,
            signing_key,
            trust_unsigned,
            ..
        }
        | Command::Develop {
            no_substitute,
            signing_key,
            trust_unsigned,
            ..
        }
        | Command::Run {
            no_substitute,
            signing_key,
            trust_unsigned,
            ..
        } => (*no_substitute, signing_key.is_some(), *trust_unsigned),
        _ => return false,
    };
    apply_build_mode_fields(modes, RuntimeBuildModeInput {
        hermeticity: HermeticitySelection {
            strict_hermetic: false,
            impure: false,
        },
        no_substitute,
        substituters: None,
        signing_key_selected,
        trusted_public_key_count: EMPTY_TRUSTED_PUBLIC_KEY_COUNT,
        trust_unsigned,
    });
    true
}

fn apply_remaining_fingerprint_fields(modes: &mut RuntimeFingerprintModeFields, command: &Command) {
    match command {
        Command::Bootstrap {
            action:
                Some(BootstrapAction::Validate {
                    strict_hermetic,
                    impure,
                    ..
                }),
            ..
        } => {
            modes.hermeticity_mode = Some(
                selected_hermeticity_mode_label(HermeticitySelection {
                    strict_hermetic: *strict_hermetic,
                    impure: *impure,
                })
                .to_string(),
            );
        }
        Command::Store { action } => apply_store_mode_fields(modes, action),
        _ => {}
    }
}

fn selected_hermeticity_mode_label(selection: HermeticitySelection) -> &'static str {
    match (selection.strict_hermetic, selection.impure) {
        (true, false) => "strict",
        (false, true) => "impure",
        (false, false) => "practical",
        (true, true) => "conflicting",
    }
}

fn apply_build_mode_fields(modes: &mut RuntimeFingerprintModeFields, input: RuntimeBuildModeInput<'_>) {
    modes.hermeticity_mode = Some(selected_hermeticity_mode_label(input.hermeticity).to_string());
    modes.substitution_mode = Some(substitution_mode_label(input.no_substitute).to_string());
    modes.substituter_count = Some(substituter_count_for_mode(input.no_substitute, input.substituters));
    modes.signing_key_selected = Some(input.signing_key_selected);
    modes.trusted_public_key_count = Some(operator_diagnostics::bounded_runtime_count(input.trusted_public_key_count));
    modes.trust_unsigned = Some(input.trust_unsigned);
}

fn substitution_mode_label(no_substitute: bool) -> &'static str {
    if no_substitute { "disabled" } else { "enabled" }
}

fn substituter_count_for_mode(no_substitute: bool, substituters: Option<&str>) -> u32 {
    if no_substitute {
        return Default::default();
    }
    substituters.map(operator_diagnostics::count_substituters).unwrap_or(DEFAULT_SUBSTITUTER_COUNT)
}

fn apply_store_mode_fields(modes: &mut RuntimeFingerprintModeFields, action: &StoreAction) {
    match action {
        StoreAction::Verify {
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            ..
        } => {
            modes.signing_key_selected = Some(signing_key.is_some());
            modes.trusted_public_key_count =
                Some(operator_diagnostics::bounded_runtime_count(trusted_public_keys.len()));
            modes.trust_unsigned = Some(*trust_unsigned);
        }
        StoreAction::Sign { signing_key, .. } | StoreAction::RepairFinalNar { signing_key, .. } => {
            modes.signing_key_selected = Some(signing_key.is_some());
        }
        StoreAction::Push { trust_unsigned, .. } => {
            modes.trust_unsigned = Some(*trust_unsigned);
        }
        StoreAction::Pull {
            trust_unsigned,
            trusted_public_keys,
            ..
        } => {
            modes.trusted_public_key_count =
                Some(operator_diagnostics::bounded_runtime_count(trusted_public_keys.len()));
            modes.trust_unsigned = Some(*trust_unsigned);
        }
        StoreAction::Archive { action } => apply_store_archive_mode_fields(modes, action),
        _ => {}
    }
}

fn apply_store_archive_mode_fields(modes: &mut RuntimeFingerprintModeFields, action: &StoreArchiveAction) {
    match action {
        StoreArchiveAction::Export { trust_unsigned, .. } => modes.trust_unsigned = Some(*trust_unsigned),
        StoreArchiveAction::Import {
            trust_unsigned,
            trusted_public_keys,
            ..
        } => {
            modes.trusted_public_key_count =
                Some(operator_diagnostics::bounded_runtime_count(trusted_public_keys.len()));
            modes.trust_unsigned = Some(*trust_unsigned);
        }
        StoreArchiveAction::List { .. } => {}
    }
}

fn dispatch_command(args: &Args, ctx: &RunContext) -> Result<(), RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert_eq!(ctx.json, args.json);
    match &args.command {
        Command::Doctor { profile } => run_doctor_command(ctx, *profile),
        Command::Graph { root, graph_file } => run_semantic_graph_command(
            ctx,
            SemanticGraphCommandInput::new(SemanticGraphQueryKind::Graph, root, graph_file.as_deref()),
        ),
        Command::Why { target, graph_file } => run_semantic_graph_command(
            ctx,
            SemanticGraphCommandInput::new(SemanticGraphQueryKind::Why, target, graph_file.as_deref()),
        ),
        Command::Dependents { target, graph_file } => run_semantic_graph_command(
            ctx,
            SemanticGraphCommandInput::new(SemanticGraphQueryKind::Dependents, target, graph_file.as_deref()),
        ),
        Command::Refactor { action } => run_refactor_command(ctx, action.clone()),
        Command::Transcript { action } => run_transcript_command(action.clone()),
        Command::Stage0Inventory { output } => run_stage0_inventory_command(ctx, output),
        Command::NixFreeDemo { action } => nix_free_demo_cmd::cmd_nix_free_demo(action.clone(), ctx.json),
        Command::ForeignImport { action } => {
            foreign_import_cmd::cmd_foreign_import(action.clone(), foreign_import_cmd::ForeignImportContext {
                output_dir: &ctx.store,
                state_dir: &ctx.resolved_state_dir,
                base_state_dirs: &ctx.base_state_dirs,
                verbose: ctx.verbose,
                json: ctx.json,
            })
        }
        Command::Mantlepkgs { action } => {
            mantlepkgs_cmd::cmd_mantlepkgs(action.clone(), mantlepkgs_cmd::MantlepkgsContext {
                output_dir: &ctx.store,
                state_dir: &ctx.resolved_state_dir,
                base_state_dirs: &ctx.base_state_dirs,
                verbose: ctx.verbose,
                json: ctx.json,
            })
        }
        Command::Eval { file, import_paths } => run_eval(file, import_paths),
        Command::Export { .. } => run_nickel_export_from_command(ctx, &args.command),
        Command::Build { .. } => run_build_from_command(ctx, &args.command),
        Command::WasmComponent { action } => run_wasm_component_command(ctx, action),
        Command::Bootstrap { .. } => run_bootstrap_from_command(ctx, &args.command),
        Command::Release { action } => run_release_command(ctx, action.clone()),
        Command::Log { query, list } => log_cmd::cmd_log(query.as_deref(), *list),
        Command::Store { action } => {
            store_cmd::cmd_store(action.clone(), &ctx.store, &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json)
        }
        Command::Source { action } => {
            source_bundle::cmd_source(action.clone(), &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json)
        }
        Command::Receipt { action } => portable_receipt::cmd_receipt(
            action.clone(),
            &ctx.resolved_state_dir,
            &ctx.store_prefix,
            ctx.json,
            unix_time_now_s()?,
        ),
        Command::Remote { action } => run_remote_command(ctx, action.clone()),
        Command::Artifact { action } => {
            artifact_cmd::cmd_artifact(action.clone(), &current_dir_or_error()?, &ctx.resolved_state_dir, ctx.json)
        }
        Command::Attest { action } => run_attest_command(ctx, action.clone()),
        Command::Import { action } => run_import_command(ctx, action.clone()),
        Command::Filegen { action } => run_filegen_command(ctx, action.clone()),
        Command::Init
        | Command::Check { .. }
        | Command::Show
        | Command::Refresh { .. }
        | Command::ListStale { .. }
        | Command::Upgrade => run_project_command(ctx, &args.command),
        Command::SelfBuild { .. } => run_self_build_from_command(ctx, &args.command),
        Command::RustPlan { .. } => run_rust_plan_command(ctx, &args.command),
        Command::Shell { .. } => run_shell_from_command(ctx, &args.command),
        Command::Develop { .. } => run_develop_from_command(ctx, &args.command),
        Command::Run { .. } => run_run_from_command(ctx, &args.command),
    }
}

fn run_remote_command(ctx: &RunContext, action: RemoteAction) -> Result<(), RunError> {
    match action {
        RemoteAction::Debug {
            action:
                RemoteFailureDebugAction::Replay {
                    bundle,
                    builder,
                    ticket,
                    builder_program,
                    builder_args,
                    trusted_builder_keys,
                    remote_build_time_secs,
                },
        } => cmd_remote_failure_debug_replay(
            RemoteFailureReplayCommandInput {
                bundle_selector: &bundle,
                builder: &builder,
                ticket: &ticket,
                builder_program: builder_program.as_deref(),
                builder_args: &builder_args,
                trusted_builder_keys: &trusted_builder_keys,
                build_time_limit_secs: remote_build_time_secs,
            },
            ctx,
        ),
        other => remote_build::cmd_remote(other, &ctx.store, &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json),
    }
}

fn run_wasm_component_command(ctx: &RunContext, action: &WasmComponentAction) -> Result<(), RunError> {
    match action {
        WasmComponentAction::Build {
            request,
            out,
            import_paths,
            scratch_parent,
        } => wasm_component_cmd::cmd_wasm_component_build(wasm_component_cmd::WasmComponentBuildOptions {
            request_path: request,
            import_paths,
            output_dir: out,
            scratch_parent: scratch_parent.as_deref(),
            json: ctx.json,
        }),
    }
}

fn run_filegen_command(ctx: &RunContext, action: FilegenCommandAction) -> Result<(), RunError> {
    let root = current_dir_or_error()?;
    debug_assert!(root.is_absolute());
    debug_assert!(ctx.store_prefix.starts_with('/'));
    match action {
        FilegenCommandAction::Plan { manifest, plan_out } => {
            filegen_cmd::cmd_filegen_plan(filegen_cmd::FilegenPlanOptions {
                root: &root,
                manifest: &manifest,
                plan_out: plan_out.as_deref(),
                json: ctx.json,
            })
        }
        FilegenCommandAction::Apply { manifest, plan } => {
            filegen_cmd::cmd_filegen_apply(filegen_cmd::FilegenApplyOptions {
                root: &root,
                manifest: &manifest,
                reviewed_plan: &plan,
                json: ctx.json,
            })
        }
    }
}

fn run_import_command(ctx: &RunContext, action: ImportAction) -> Result<(), RunError> {
    match action {
        ImportAction::Pins { action } => run_pin_import_command(ctx, action),
        ImportAction::Cargo {
            plan,
            apply,
            package,
            binary,
            project_file,
            inputs_file,
            target,
            profile,
        } => {
            if plan && apply {
                return Err(RunError::Internal("cargo import accepts either --plan or --apply, not both".to_string()));
            }
            let root = current_dir_or_error()?;
            cargo_import::run_cargo_import(cargo_import::CargoImportShellOptions {
                root: &root,
                selected_package: package.as_deref(),
                selected_binary: binary.as_deref(),
                project_file: &project_file,
                inputs_file: &inputs_file,
                target_triple: &target,
                profile: &profile,
                apply,
                json: ctx.json,
            })
        }
    }
}

fn run_pin_import_command(ctx: &RunContext, action: PinImportAction) -> Result<(), RunError> {
    let root = current_dir_or_error()?;
    debug_assert!(root.is_absolute());
    debug_assert!(ctx.store_prefix.starts_with('/'));
    match action {
        PinImportAction::Plan {
            importer,
            pins_file,
            project_file,
            lock_file,
            inputs_file,
        } => pin_import::run_pin_import(pin_import::PinImportShellOptions {
            root: &root,
            importer: importer.as_str(),
            pins_file: &pins_file,
            project_file: &project_file,
            lock_file: &lock_file,
            inputs_file: &inputs_file,
            apply: false,
            json: ctx.json,
        }),
        PinImportAction::Apply {
            importer,
            pins_file,
            project_file,
            lock_file,
            inputs_file,
        } => pin_import::run_pin_import(pin_import::PinImportShellOptions {
            root: &root,
            importer: importer.as_str(),
            pins_file: &pins_file,
            project_file: &project_file,
            lock_file: &lock_file,
            inputs_file: &inputs_file,
            apply: true,
            json: ctx.json,
        }),
    }
}

fn run_refactor_command(ctx: &RunContext, action: RefactorAction) -> Result<(), RunError> {
    match action {
        RefactorAction::List => {
            let session = structured_refactor::crunch_to_mantle_session();
            if ctx.json {
                let rendered =
                    serde_json::to_string_pretty(&vec![session]).map_err(|err| RunError::Internal(err.to_string()))?;
                println!("{rendered}");
            } else {
                println!("{} — {}", session.id, session.risk_summary);
            }
            Ok(())
        }
        RefactorAction::Plan {
            session,
            root,
            store_prefixes,
        } => {
            let root = root.unwrap_or(current_dir_or_error()?);
            let plan = plan_refactor_session(&session, &root, &store_prefixes)?;
            print_refactor_plan(ctx, &plan)?;
            Ok(())
        }
        RefactorAction::Check {
            session,
            root,
            store_prefixes,
        } => {
            let root = root.unwrap_or(current_dir_or_error()?);
            let plan = plan_refactor_session(&session, &root, &store_prefixes)?;
            print_refactor_plan(ctx, &plan)?;
            if plan.has_conflicts() {
                return Err(RunError::Reported(1));
            }
            Ok(())
        }
        RefactorAction::Apply {
            session,
            root,
            store_prefixes,
        } => {
            let session = session.ok_or_else(|| {
                RunError::Internal(
                    structured_refactor::RefactorError::MissingExplicitSession {
                        available: vec![structured_refactor::CRUNCH_TO_MANTLE_SESSION_ID],
                    }
                    .to_string(),
                )
            })?;
            let root = root.unwrap_or(current_dir_or_error()?);
            match session.as_str() {
                structured_refactor::CRUNCH_TO_MANTLE_SESSION_ID => {
                    let plan = structured_refactor::apply_crunch_to_mantle(&root, &store_prefixes)
                        .map_err(|err| RunError::Internal(err.to_string()))?;
                    print_refactor_plan(ctx, &plan)
                }
                _ => Err(RunError::Internal(structured_refactor::RefactorError::UnknownSession(session).to_string())),
            }
        }
    }
}

fn plan_refactor_session(
    session: &str,
    root: &Path,
    store_prefixes: &[String],
) -> Result<structured_refactor::RefactorPlan, RunError> {
    match session {
        structured_refactor::CRUNCH_TO_MANTLE_SESSION_ID => {
            Ok(structured_refactor::plan_crunch_to_mantle(root, store_prefixes))
        }
        _ => Err(RunError::Internal(
            structured_refactor::RefactorError::UnknownSession(session.to_string()).to_string(),
        )),
    }
}

fn print_refactor_plan(ctx: &RunContext, plan: &structured_refactor::RefactorPlan) -> Result<(), RunError> {
    debug_assert!(!plan.session_id.is_empty());
    debug_assert!(!plan.root.is_empty());
    if ctx.json {
        let rendered = serde_json::to_string_pretty(plan).map_err(|err| RunError::Internal(err.to_string()))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("refactor session: {}", plan.session_id);
    println!("root: {}", plan.root);
    println!("dry-run: {}", plan.dry_run);
    if plan.operations.is_empty() {
        println!("operations: none");
    } else {
        println!("operations:");
        for operation in &plan.operations {
            let suffix = if operation.apply_supported { "" } else { " (plan-only)" };
            println!("  {}: {} -> {}{}", operation.kind, operation.from, operation.to, suffix);
        }
    }
    for diagnostic in &plan.diagnostics {
        println!("diagnostic {}: {}", diagnostic.code, diagnostic.message);
        println!("  remediation: {}", diagnostic.remediation);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum SemanticGraphQueryKind {
    Graph,
    Why,
    Dependents,
}

#[derive(Debug, Clone, Copy)]
struct SemanticGraphCommandInput<'a> {
    query: SemanticGraphQueryKind,
    target: &'a str,
    graph_file: Option<&'a Path>,
}

impl<'a> SemanticGraphCommandInput<'a> {
    fn new(query: SemanticGraphQueryKind, target: &'a str, graph_file: Option<&'a Path>) -> Self {
        Self {
            query,
            target,
            graph_file,
        }
    }
}

fn run_semantic_graph_command(ctx: &RunContext, input: SemanticGraphCommandInput<'_>) -> Result<(), RunError> {
    if input.target.is_empty() {
        return Err(RunError::Internal("semantic graph target must not be empty".to_string()));
    }
    debug_assert!(!input.target.is_empty());
    debug_assert!(ctx.store_prefix.starts_with('/'));
    let path = input
        .graph_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| ctx.resolved_state_dir.join("semantic-graph.json"));
    let graph = load_semantic_graph(ctx, &path)?;
    let rendered = execute_semantic_graph_query(ctx, &graph, input)?;
    println!("{rendered}");
    Ok(())
}

fn load_semantic_graph(ctx: &RunContext, path: &Path) -> Result<semantic_graph::SemanticGraph, RunError> {
    match semantic_graph::SemanticGraph::load(path) {
        Ok(graph) => Ok(graph),
        Err(semantic_graph::SemanticGraphError::Incomplete(diag)) => {
            report_incomplete_semantic_graph(ctx, &diag)?;
            Err(RunError::Reported(1))
        }
        Err(err) => Err(RunError::Internal(err.to_string())),
    }
}

fn report_incomplete_semantic_graph(
    ctx: &RunContext,
    diag: &semantic_graph::IncompleteGraphDiagnostic,
) -> Result<(), RunError> {
    if ctx.json {
        let rendered = serde_json::to_string_pretty(diag).map_err(|err| RunError::Internal(err.to_string()))?;
        println!("{rendered}");
    } else {
        eprintln!(
            "incomplete semantic graph for {} `{}`: missing {}",
            diag.query,
            diag.target,
            diag.missing.join(", ")
        );
    }
    Ok(())
}

fn execute_semantic_graph_query(
    ctx: &RunContext,
    graph: &semantic_graph::SemanticGraph,
    input: SemanticGraphCommandInput<'_>,
) -> Result<String, RunError> {
    match input.query {
        SemanticGraphQueryKind::Graph => graph
            .graph_for_root(input.target)
            .map_err(|err| report_semantic_graph_query_error_value(ctx, err))
            .and_then(|result| render_graph_result(ctx.json, &result)),
        SemanticGraphQueryKind::Why => graph
            .why(input.target)
            .map_err(|err| report_semantic_graph_query_error_value(ctx, err))
            .and_then(|result| render_why_result(ctx.json, &result)),
        SemanticGraphQueryKind::Dependents => graph
            .dependents(input.target)
            .map_err(|err| report_semantic_graph_query_error_value(ctx, err))
            .and_then(|result| render_dependents_result(ctx.json, &result)),
    }
}

fn report_semantic_graph_query_error_value(ctx: &RunContext, err: semantic_graph::SemanticGraphError) -> RunError {
    match report_semantic_graph_query_error(ctx, err) {
        Ok(()) => RunError::Internal("semantic graph error reporting returned success".to_string()),
        Err(error) => error,
    }
}

fn report_semantic_graph_query_error(
    ctx: &RunContext,
    err: semantic_graph::SemanticGraphError,
) -> Result<(), RunError> {
    if let semantic_graph::SemanticGraphError::Incomplete(diag) = err {
        if ctx.json {
            let rendered = serde_json::to_string_pretty(&diag).map_err(|err| RunError::Internal(err.to_string()))?;
            println!("{rendered}");
        } else {
            eprintln!(
                "incomplete semantic graph for {} `{}`: missing {}",
                diag.query,
                diag.target,
                diag.missing.join(", ")
            );
        }
        return Err(RunError::Reported(1));
    }
    Err(RunError::Internal(err.to_string()))
}

fn render_graph_result(json: bool, result: &semantic_graph::GraphQueryResult<'_>) -> Result<String, RunError> {
    debug_assert!(result.nodes.iter().all(|node| !node.id.is_empty()));
    debug_assert!(result.edges.iter().all(|edge| !edge.from.is_empty()));
    if json {
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("semantic graph root: {}\n", result.root);
    out.push_str("nodes:\n");
    for node in &result.nodes {
        out.push_str(&format!("  {} [{}]\n", node.id, node.kind));
    }
    out.push_str("edges:\n");
    for edge in &result.edges {
        out.push_str(&format!("  {} --{}--> {}\n", edge.from, edge.kind, edge.to));
    }
    if !result.aliases.is_empty() {
        out.push_str("aliases:\n");
        for alias in &result.aliases {
            out.push_str(&format!("  {} -> {}\n", alias.alias, alias.target));
        }
    }
    Ok(out)
}

fn render_why_result(json: bool, result: &semantic_graph::WhyResult<'_>) -> Result<String, RunError> {
    if json {
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("why {}\nnode: {} [{}]\n", result.target, result.node.id, result.node.kind);
    if let Some(recipe) = result.producing_recipe {
        out.push_str(&format!("produced by: {}\n", recipe.id));
    }
    append_node_list(&mut out, "sources", &result.sources);
    append_node_list(&mut out, "providers", &result.providers);
    append_node_list(&mut out, "sandboxes", &result.sandboxes);
    append_node_list(&mut out, "proof receipts", &result.proof_receipts);
    append_node_list(&mut out, "witness requests", &result.witness_requests);
    append_node_list(&mut out, "release evidence", &result.release_evidence);
    Ok(out)
}

fn render_dependents_result(json: bool, result: &semantic_graph::DependentsResult<'_>) -> Result<String, RunError> {
    if json {
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("dependents of {}:\n", result.target);
    for dependent in &result.dependents {
        out.push_str(&format!("  {} [{}]\n", dependent.id, dependent.kind));
    }
    Ok(out)
}

fn append_node_list(out: &mut String, label: &str, nodes: &[&semantic_graph::SemanticNode]) {
    if nodes.is_empty() {
        return;
    }
    out.push_str(&format!("{label}:\n"));
    for node in nodes {
        out.push_str(&format!("  {} [{}]\n", node.id, node.kind));
    }
}

fn run_transcript_command(action: TranscriptAction) -> Result<(), RunError> {
    match action {
        TranscriptAction::Run {
            transcript,
            output,
            mantle_bin,
            allow_in_place,
        } => transcript_cmd::cmd_transcript_run(transcript_cmd::TranscriptRunOptions {
            transcript,
            output,
            mantle_bin,
            allow_in_place,
        }),
    }
}

fn run_doctor_command(ctx: &RunContext, profile: DoctorProfile) -> Result<(), RunError> {
    let doctor_result = operator_diagnostics::collect_doctor_report(operator_diagnostics::DoctorRequest {
        profile,
        store_dir: &ctx.store,
        state_dir: &ctx.resolved_state_dir,
    });
    emit_doctor_report(ctx, &doctor_result)?;
    doctor_report_result(doctor_result.ok)
}

fn emit_doctor_report(ctx: &RunContext, doctor_report: &operator_diagnostics::PreflightReport) -> Result<(), RunError> {
    if ctx.json {
        let rendered = doctor_report
            .render_json()
            .map_err(|error| RunError::Internal(format!("serializing doctor report: {error}")))?;
        println!("{rendered}");
    } else {
        let rendered = doctor_report.render_human();
        if doctor_report.ok {
            println!("{rendered}");
        } else {
            eprintln!("{rendered}");
        }
    }
    Ok(())
}

fn doctor_report_result(is_ok: bool) -> Result<(), RunError> {
    if is_ok { Ok(()) } else { Err(RunError::Reported(3)) }
}

fn run_stage0_inventory_command(ctx: &RunContext, output: &Path) -> Result<(), RunError> {
    let output_path = if output.is_absolute() {
        output.to_path_buf()
    } else {
        current_dir_or_error()?.join(output)
    };
    let inventory = protected_exec::write_generated_stage0_inventory_from_env(&output_path)
        .map_err(|err| RunError::Build(format!("generating stage0 inventory from explicit seed paths: {err}")))?;
    let inventory_digest_blake3 = protected_exec::stage0_inventory_digest_blake3(&inventory);
    let risks = protected_exec::seed_closure_risk_report(&inventory);
    debug_assert!(output_path.is_absolute());
    debug_assert_eq!(inventory_digest_blake3.len(), blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
    if ctx.json {
        let risk_json = risks
            .iter()
            .map(|risk| {
                serde_json::json!({
                    "entry_id": risk.entry_id,
                    "path": risk.executable_path,
                    "risk": risk.risk.to_string(),
                })
            })
            .collect::<Vec<_>>();
        let rendered = serde_json::json!({
            "schema": "crunch-stage0-inventory-preflight-v1",
            "inventory_path": output_path,
            "inventory_digest_blake3": inventory_digest_blake3,
            "executable_entries": inventory.executable_entries.len(),
            "source_entries": inventory.source_entries.len(),
            "closure_risks": risk_json,
            "seed_input_policy": "explicit CRUNCH_STAGE0_SEED_* paths only; no PATH or /nix/store discovery",
        });
        println!("{}", serde_json::to_string_pretty(&rendered).map_err(|err| RunError::Internal(err.to_string()))?);
        return Ok(());
    }
    println!("stage0 inventory written: {}", output_path.display());
    println!("stage0 inventory digest: {inventory_digest_blake3}");
    println!("seed input policy: explicit CRUNCH_STAGE0_SEED_* paths only; no PATH or /nix/store discovery");
    println!("executable seed entries: {}", inventory.executable_entries.len());
    println!("source seed entries: {}", inventory.source_entries.len());
    for risk in risks {
        println!("seed closure risk: id={} path={} risk={}", risk.entry_id, risk.executable_path.display(), risk.risk,);
    }
    Ok(())
}

fn run_eval(file: &Path, import_paths: &[PathBuf]) -> Result<(), RunError> {
    let nickel_search_dirs = build_import_paths(import_paths)?;
    let json = crunch_eval::evaluate_to_json(file, &nickel_search_dirs).map_err(|e| RunError::Eval(format!("{e}")))?;
    println!("{json}");
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct NickelExportCommandInput<'a> {
    file: &'a Path,
    deps: &'a [PathBuf],
    import_paths: &'a [PathBuf],
    format: &'a str,
    out: Option<&'a Path>,
    evaluator_id: &'a str,
    evaluator_version: &'a str,
}

fn run_nickel_export_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert!(!ctx.resolved_state_dir.as_os_str().is_empty());
    let Command::Export {
        file,
        deps,
        import_paths,
        format,
        out,
        evaluator_id,
        evaluator_version,
    } = command
    else {
        return Err(RunError::Internal("Nickel export helper requires an export command".to_string()));
    };
    run_nickel_export_command(ctx, NickelExportCommandInput {
        file,
        deps,
        import_paths,
        format,
        out: out.as_deref(),
        evaluator_id,
        evaluator_version,
    })
}

fn run_nickel_export_command(ctx: &RunContext, input: NickelExportCommandInput<'_>) -> Result<(), RunError> {
    let root = current_dir_or_error()?;
    nickel_export::cmd_nickel_export(nickel_export::NickelExportOptions {
        root: &root,
        file: input.file,
        deps: input.deps,
        import_paths: input.import_paths,
        format: input.format,
        out: input.out,
        evaluator_id: input.evaluator_id,
        evaluator_version: input.evaluator_version,
        json: ctx.json,
    })
}

#[derive(Debug, Clone, Copy)]
struct BuildCommandInput<'a> {
    file: Option<&'a Path>,
    import_paths: &'a [PathBuf],
    fix: bool,
    plan: bool,
    offline_source_preflight: bool,
    jobs: Option<u32>,
    substituters: &'a str,
    no_substitute: bool,
    signing_key: Option<&'a Path>,
    trusted_public_keys: &'a [String],
    trust_unsigned: bool,
    hermeticity: HermeticitySelection,
    remote_builder: Option<&'a str>,
    remote_ticket: Option<&'a str>,
    remote_builder_program: Option<&'a Path>,
    remote_builder_args: &'a [String],
    trusted_builder_keys: &'a [String],
    remote_build_time_secs: u64,
    remote_delta: bool,
    remote_observability_config: Option<&'a Path>,
}

fn run_build_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert!(!ctx.store.as_os_str().is_empty());
    let Command::Build {
        file,
        import_paths,
        fix,
        plan,
        offline_source_preflight,
        jobs,
        substituters,
        no_substitute,
        signing_key,
        trusted_public_keys,
        trust_unsigned,
        strict_hermetic,
        impure,
        builder,
        ticket,
        builder_program,
        builder_args,
        trusted_builder_keys,
        remote_build_time_secs,
        remote_delta,
        remote_observability_config,
    } = command
    else {
        return Err(RunError::Internal("build helper requires a build command".to_string()));
    };
    run_build_command(ctx, BuildCommandInput {
        file: file.as_deref(),
        import_paths,
        fix: *fix,
        plan: *plan,
        offline_source_preflight: *offline_source_preflight,
        jobs: *jobs,
        substituters,
        no_substitute: *no_substitute,
        signing_key: signing_key.as_deref(),
        trusted_public_keys,
        trust_unsigned: *trust_unsigned,
        hermeticity: HermeticitySelection {
            strict_hermetic: *strict_hermetic,
            impure: *impure,
        },
        remote_builder: builder.as_deref(),
        remote_ticket: ticket.as_deref(),
        remote_builder_program: builder_program.as_deref(),
        remote_builder_args: builder_args,
        trusted_builder_keys,
        remote_build_time_secs: *remote_build_time_secs,
        remote_delta: *remote_delta,
        remote_observability_config: remote_observability_config.as_deref(),
    })
}

fn select_hermeticity_mode(selection: HermeticitySelection) -> Result<crunch_pipeline::HermeticityMode, RunError> {
    match (selection.strict_hermetic, selection.impure) {
        (true, true) => Err(RunError::Internal("--strict-hermetic and --impure are mutually exclusive".to_string())),
        (true, false) => Ok(crunch_pipeline::HermeticityMode::Strict),
        (false, true) => Ok(crunch_pipeline::HermeticityMode::Impure),
        (false, false) => Ok(crunch_pipeline::HermeticityMode::Practical),
    }
}

struct PreparedBuildCommand<'a> {
    ctx: &'a RunContext,
    target: project_build::BuildTarget,
    import_path_args: &'a [PathBuf],
    fix: bool,
    plan: bool,
    offline_source_preflight: bool,
    max_jobs: u32,
    substituter_urls: Vec<String>,
    signing_key: Option<&'a Path>,
    parsed_trusted: Option<Vec<nix_compat::narinfo::VerifyingKey>>,
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    remote_plan_facts: Option<realization_routing::RemoteBuilderPlanFacts>,
    remote_selection: Option<RemoteBuildSelection>,
}

fn run_build_command<'a>(ctx: &'a RunContext, input: BuildCommandInput<'a>) -> Result<(), RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert!(!ctx.store.as_os_str().is_empty());
    let prepared = prepare_build_command(ctx, input)?;
    match &prepared.target {
        project_build::BuildTarget::File(path) => run_file_build_target(&prepared, path),
        project_build::BuildTarget::ProjectDefault | project_build::BuildTarget::Selector(_) => {
            run_project_build_target(&prepared)
        }
    }
}

fn prepare_build_command<'a>(
    ctx: &'a RunContext,
    input: BuildCommandInput<'a>,
) -> Result<PreparedBuildCommand<'a>, RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert!(!input.substituters.trim().is_empty() || input.no_substitute);
    let substituter_urls = if input.no_substitute {
        Vec::new()
    } else {
        cache_substitution::split_substituter_urls(input.substituters)
    };
    let remote_plan_facts =
        remote_plan_facts_for_cli(input.remote_builder, input.remote_ticket, input.trusted_builder_keys)?;
    let remote_selection = if input.plan {
        None
    } else {
        remote_build_selection(
            RemoteBuildSelectionInput {
                builder: input.remote_builder,
                ticket: input.remote_ticket,
                builder_program: input.remote_builder_program,
                builder_args: input.remote_builder_args,
                trusted_builder_keys: input.trusted_builder_keys,
                build_time_limit_secs: input.remote_build_time_secs,
                remote_delta: input.remote_delta,
                observability_config_path: input.remote_observability_config,
            },
            ctx,
        )?
    };
    if remote_selection.is_some() && input.fix {
        return Err(RunError::Internal("remote build dispatch does not support --fix yet".to_string()));
    }
    Ok(PreparedBuildCommand {
        ctx,
        target: project_build::parse_build_target(input.file),
        import_path_args: input.import_paths,
        fix: input.fix,
        plan: input.plan,
        offline_source_preflight: input.offline_source_preflight,
        max_jobs: crunch_pipeline::resolve_max_jobs(input.jobs),
        substituter_urls,
        signing_key: input.signing_key,
        parsed_trusted: parse_trusted_keys(input.trusted_public_keys)?,
        trust_unsigned: input.trust_unsigned,
        hermeticity_mode: select_hermeticity_mode(input.hermeticity)?,
        remote_plan_facts,
        remote_selection,
    })
}

fn run_file_build_target(prepared: &PreparedBuildCommand<'_>, path: &Path) -> Result<(), RunError> {
    debug_assert!(prepared.ctx.store_prefix.starts_with('/'));
    debug_assert!(!path.as_os_str().is_empty());
    let nickel_search_dirs = build_import_paths(prepared.import_path_args)?;
    let source_preflight = run_offline_source_preflight_if_requested(OfflineSourcePreflightRequest {
        enabled: prepared.offline_source_preflight,
        file: path,
        import_entries: &nickel_search_dirs,
        state_dir: &prepared.ctx.resolved_state_dir,
        store_prefix: &prepared.ctx.store_prefix,
        output_mode: prepared.ctx.output_mode(),
    })?;
    let source_fetch_plan =
        if prepared.offline_source_preflight && !prepared.plan && prepared.remote_selection.is_none() {
            Some(source_bundle::source_fetch_override_plan_for_file(
                path,
                &nickel_search_dirs,
                &prepared.ctx.resolved_state_dir,
                &prepared.ctx.store_prefix,
            )?)
        } else {
            None
        };
    if let Some(selection) = &prepared.remote_selection {
        return run_remote_build_command(RemoteBuildCommandRequest {
            selection,
            source: RemoteBuildSource::File(path),
            import_entries: &nickel_search_dirs,
            ctx: prepared.ctx,
        });
    }
    if prepared.plan {
        return run_file_build_plan(prepared, path, &nickel_search_dirs, source_preflight.as_ref());
    }
    run_local_file_build(prepared, path, &nickel_search_dirs, source_fetch_plan.as_ref())
}

fn run_file_build_plan(
    prepared: &PreparedBuildCommand<'_>,
    path: &Path,
    import_entries: &[OsString],
    source_preflight: Option<&source_bundle::SourceOfflinePreflightReport>,
) -> Result<(), RunError> {
    build_plan::cmd_build_plan(build_plan::BuildPlanConfig {
        file: path,
        import_paths: import_entries,
        output_dir: &prepared.ctx.store,
        state_dir: &prepared.ctx.resolved_state_dir,
        store_dir: &prepared.ctx.store_prefix,
        substituter_urls: &prepared.substituter_urls,
        signing_key_path: prepared.signing_key,
        trusted_public_keys: prepared.parsed_trusted.as_deref(),
        trust_unsigned: prepared.trust_unsigned,
        remote_builder: prepared.remote_plan_facts.as_ref(),
        source_preflight,
        output_mode: prepared.ctx.output_mode(),
    })
}

fn run_local_file_build(
    prepared: &PreparedBuildCommand<'_>,
    path: &Path,
    import_entries: &[OsString],
    source_fetch_plan: Option<&source_bundle::SourceFetchOverridePlan>,
) -> Result<(), RunError> {
    debug_assert!(prepared.max_jobs > 0);
    debug_assert!(prepared.ctx.store_prefix.starts_with('/'));
    debug_assert!(!path.as_os_str().is_empty());
    if let Some(source_fetch_plan) = source_fetch_plan {
        return build_cmd::cmd_build_with_source_fetch_overrides(
            path,
            import_entries,
            &prepared.ctx.store,
            &prepared.ctx.resolved_state_dir,
            &prepared.ctx.store_prefix,
            prepared.ctx.verbose,
            prepared.fix,
            prepared.max_jobs,
            &prepared.substituter_urls,
            prepared.signing_key,
            prepared.parsed_trusted.as_deref(),
            prepared.trust_unsigned,
            prepared.hermeticity_mode,
            prepared.ctx.output_mode(),
            source_fetch_plan.overrides.clone(),
            prepared.ctx.base_state_dirs.clone(),
        );
    }
    build_cmd::cmd_build(
        path,
        import_entries,
        &prepared.ctx.store,
        &prepared.ctx.resolved_state_dir,
        &prepared.ctx.store_prefix,
        prepared.ctx.verbose,
        prepared.fix,
        prepared.max_jobs,
        &prepared.substituter_urls,
        prepared.signing_key,
        prepared.parsed_trusted.as_deref(),
        prepared.trust_unsigned,
        prepared.hermeticity_mode,
        prepared.ctx.output_mode(),
    )
}

fn run_project_build_target(prepared: &PreparedBuildCommand<'_>) -> Result<(), RunError> {
    debug_assert!(prepared.ctx.store_prefix.starts_with('/'));
    debug_assert!(!matches!(prepared.target, project_build::BuildTarget::File(_)));
    let cwd = current_dir_or_error()?;
    let resolved = project_build::resolve_project_target(&prepared.target, &cwd, prepared.import_path_args)?;
    let expr = project_build::generate_extraction_expr(&resolved.root_file, &resolved.target);
    let mut nickel_search_dirs = build_import_paths(&[])?;
    nickel_search_dirs.extend(resolved.import_paths);
    let source_preflight = run_offline_source_preflight_for_expr_if_requested(OfflineExprPreflightRequest {
        enabled: prepared.offline_source_preflight,
        expr: &expr,
        import_entries: &nickel_search_dirs,
        state_dir: &prepared.ctx.resolved_state_dir,
        store_prefix: &prepared.ctx.store_prefix,
        output_mode: prepared.ctx.output_mode(),
    })?;
    let source_fetch_plan = source_fetch_override_plan_for_expr_if_requested(
        prepared.offline_source_preflight && !prepared.plan && prepared.remote_selection.is_none(),
        &expr,
        &nickel_search_dirs,
        &prepared.ctx.resolved_state_dir,
        &prepared.ctx.store_prefix,
    )?;
    if let Some(selection) = &prepared.remote_selection {
        return run_remote_build_command(RemoteBuildCommandRequest {
            selection,
            source: RemoteBuildSource::Expr(&expr),
            import_entries: &nickel_search_dirs,
            ctx: prepared.ctx,
        });
    }
    if prepared.plan {
        return build_plan_from_expr(InlineBuildPlanRequest {
            expr: &expr,
            import_entries: &nickel_search_dirs,
            prepared,
            source_preflight: source_preflight.as_ref(),
        });
    }
    build_from_expr(InlineBuildRequest {
        expr: &expr,
        import_entries: &nickel_search_dirs,
        prepared,
        source_fetch_overrides: source_fetch_plan.map(|plan| plan.overrides).unwrap_or_default(),
    })
}

struct RemoteBuildSelection {
    options: remote_build::RemoteClientBuildOptions,
    worker_generation: u64,
    worker_concurrency: u32,
    resource_inventory: Option<crunch_build::distributed::RemoteWorkerResourceInventory>,
    telemetry: remote_telemetry_export::RemoteTelemetryExportConfig,
    failure_debug: crunch_build::distributed::RemoteFailureDebugPolicy,
    trace_context: Option<remote_trace_context::RemoteTraceContext>,
    trace_health: remote_trace_context::RemoteTraceContextHealth,
}

fn remote_plan_facts_for_cli(
    builder: Option<&str>,
    ticket: Option<&str>,
    trusted_builder_keys: &[String],
) -> Result<Option<realization_routing::RemoteBuilderPlanFacts>, RunError> {
    let trusted_output_key_count = u32::try_from(trusted_builder_keys.len()).map_err(|_| {
        RunError::Internal(format!(
            "trusted remote builder key count does not fit in u32: {}",
            trusted_builder_keys.len()
        ))
    })?;
    Ok(realization_routing::RemoteBuilderPlanFacts::cli_configured(
        builder,
        ticket.is_some(),
        trusted_output_key_count,
    ))
}

#[derive(Debug, Clone, Copy)]
struct RemoteBuildSelectionInput<'a> {
    builder: Option<&'a str>,
    ticket: Option<&'a str>,
    builder_program: Option<&'a Path>,
    builder_args: &'a [String],
    trusted_builder_keys: &'a [String],
    build_time_limit_secs: u64,
    remote_delta: bool,
    observability_config_path: Option<&'a Path>,
}

impl RemoteBuildSelectionInput<'_> {
    fn is_disabled(self) -> bool {
        [
            self.builder.is_none(),
            self.ticket.is_none(),
            self.builder_program.is_none(),
            self.builder_args.is_empty(),
        ]
        .into_iter()
        .all(|is_absent| is_absent)
    }
}

fn remote_build_selection(
    input: RemoteBuildSelectionInput<'_>,
    ctx: &RunContext,
) -> Result<Option<RemoteBuildSelection>, RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert!(!ctx.resolved_state_dir.as_os_str().is_empty());
    if input.is_disabled() {
        return Ok(None);
    }
    let builder = input
        .builder
        .ok_or_else(|| RunError::Internal("remote build dispatch requires --builder".to_string()))?;
    let ticket = input
        .ticket
        .ok_or_else(|| RunError::Internal("remote build dispatch requires --ticket".to_string()))?;
    let ticket = remote_build::parse_remote_ticket_credential(ticket).map_err(RunError::Internal)?;
    let (program, args) = remote_stdio_builder_command(builder, input.builder_program, input.builder_args, ctx)?;
    let trusted_output_keys =
        remote_trusted_builder_keys(input.trusted_builder_keys, input.builder_program, builder, ctx)?;
    let transfer_capabilities = if input.remote_delta {
        remote_build::RemoteTransferCapabilities::delta_and_full().with_streaming()
    } else {
        remote_build::RemoteTransferCapabilities::full_only().with_streaming()
    };
    let farm_config = input
        .observability_config_path
        .map(remote_farm_config::load_remote_build_farm_config)
        .transpose()
        .map_err(RunError::Internal)?
        .unwrap_or_default();
    let selected_profile = select_remote_builder_profile(&farm_config, builder)?;
    let worker_generation = selected_profile
        .as_ref()
        .map_or(crunch_build::distributed::RemoteFenceGeneration::INITIAL.get(), |profile| profile.worker_generation);
    let worker_concurrency = selected_profile
        .as_ref()
        .map_or(remote_build::DEFAULT_REMOTE_CONCURRENCY, |profile| profile.max_concurrency);
    let resource_inventory = selected_profile.and_then(|profile| profile.resource_inventory);
    let traceparent = std::env::var(W3C_TRACEPARENT_ENV).ok();
    let tracestate = std::env::var(W3C_TRACESTATE_ENV).ok();
    let (trace_context, trace_health) = remote_trace_context::accept_remote_trace_context(
        farm_config.trace_context.enabled,
        traceparent.as_deref(),
        tracestate.as_deref(),
    );
    let options = remote_build::RemoteClientBuildOptions {
        store_prefix: ctx.store_prefix.clone(),
        ticket,
        builder: remote_build::RemoteStdioBuilderCommand {
            endpoint_id: builder.to_string(),
            program,
            args,
        },
        trusted_output_keys,
        now_unix_s: unix_time_now_s()?,
        build_time_limit_secs: input.build_time_limit_secs,
        client_endpoint: None,
        transfer_capabilities,
    };
    Ok(Some(RemoteBuildSelection {
        options,
        worker_generation,
        worker_concurrency,
        resource_inventory,
        telemetry: farm_config.telemetry,
        failure_debug: farm_config.failure_debug,
        trace_context,
        trace_health,
    }))
}

fn select_remote_builder_profile(
    farm_config: &remote_farm_config::RemoteBuildFarmConfig,
    builder: &str,
) -> Result<Option<remote_farm_config::RemoteCapabilityProfile>, RunError> {
    let mut matching_profiles = farm_config
        .pools
        .iter()
        .flat_map(|pool| pool.endpoints.iter())
        .filter(|endpoint| endpoint.endpoint_id == builder)
        .map(|endpoint| endpoint.profile.clone());
    let selected = matching_profiles.next();
    if matching_profiles.next().is_some() {
        return Err(RunError::Internal(format!("remote builder {builder:?} has ambiguous capability profiles")));
    }
    Ok(selected)
}

fn remote_stdio_builder_command(
    builder: &str,
    builder_program: Option<&Path>,
    builder_args: &[String],
    ctx: &RunContext,
) -> Result<(PathBuf, Vec<String>), RunError> {
    if builder.is_empty() {
        return Err(RunError::Internal("remote builder identity must not be empty".to_string()));
    }
    debug_assert!(!builder.is_empty());
    debug_assert!(ctx.store_prefix.starts_with('/'));
    if let Some(program) = builder_program {
        return Ok((program.to_path_buf(), builder_args.to_vec()));
    }
    if !builder_args.is_empty() {
        return Err(RunError::Internal(
            "--builder-arg requires --builder-program; default local builder args are generated automatically"
                .to_string(),
        ));
    }
    let program =
        std::env::current_exe().map_err(|err| RunError::Internal(format!("resolving current executable: {err}")))?;
    let worker_root = local_remote_worker_root(ctx, builder);
    let worker_state_dir = worker_root.join("state");
    let worker_store_dir = worker_root.join("store");
    let args = vec![
        "--state-dir".to_string(),
        ctx.resolved_state_dir.display().to_string(),
        "--store".to_string(),
        worker_store_dir.display().to_string(),
        "--store-prefix".to_string(),
        ctx.store_prefix.clone(),
        "remote".to_string(),
        "serve".to_string(),
        "--endpoint-id".to_string(),
        builder.to_string(),
        "--binding".to_string(),
        "stdio-once".to_string(),
        "--executor".to_string(),
        "local-build".to_string(),
        "--execution-state-dir".to_string(),
        worker_state_dir.display().to_string(),
    ];
    Ok((program, args))
}

fn local_remote_worker_root(ctx: &RunContext, builder: &str) -> PathBuf {
    let identity = blake3::hash(builder.as_bytes()).to_hex().to_string();
    ctx.resolved_state_dir.join("remote-workers").join(identity)
}

fn remote_trusted_builder_keys(
    trusted_builder_keys: &[String],
    builder_program: Option<&Path>,
    builder: &str,
    ctx: &RunContext,
) -> Result<Vec<String>, RunError> {
    if !trusted_builder_keys.is_empty() {
        return Ok(trusted_builder_keys.to_vec());
    }
    if builder_program.is_some() {
        return Err(RunError::Internal(
            "remote build dispatch with --builder-program requires --trusted-builder-key".to_string(),
        ));
    }
    let worker_state_dir = local_remote_worker_root(ctx, builder).join("state");
    let keypair = build_cmd::load_or_generate_signing_keypair(None, &worker_state_dir, false)?;
    Ok(vec![keypair.verifying_key.name().to_string()])
}

#[allow(
    tigerstyle::ambient_clock,
    reason = "CLI shell captures wall time once before passing explicit seconds into deterministic cores"
)]
fn unix_time_now_s() -> Result<u64, RunError> {
    let elapsed_since_epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RunError::Internal(format!("system clock before unix epoch: {err}")))?;
    Ok(elapsed_since_epoch_secs.as_secs())
}

enum RemoteBuildSource<'a> {
    File(&'a Path),
    Expr(&'a str),
}

struct RemoteBuildCommandRequest<'a> {
    selection: &'a RemoteBuildSelection,
    source: RemoteBuildSource<'a>,
    import_entries: &'a [OsString],
    ctx: &'a RunContext,
}

fn run_remote_build_command(request: RemoteBuildCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(request.ctx.store_prefix.starts_with('/'));
    debug_assert!(!request.ctx.store.as_os_str().is_empty());
    let expression_file;
    let file = match request.source {
        RemoteBuildSource::File(file) => file,
        RemoteBuildSource::Expr(expr) => {
            expression_file = tempfile::NamedTempFile::with_suffix(".ncl")
                .map_err(|err| RunError::Internal(format!("creating temp file: {err}")))?;
            std::fs::write(expression_file.path(), expr)
                .map_err(|err| RunError::Internal(format!("writing temp file: {err}")))?;
            expression_file.path()
        }
    };
    let inputs = evaluate_remote_client_derivation_inputs(file, request.import_entries, &request.ctx.store_prefix)?;
    let build_outcome = run_remote_build_dispatches(
        request.selection,
        inputs,
        &request.ctx.store,
        &request.ctx.resolved_state_dir,
        &request.ctx.store_prefix,
    )?;
    print_remote_client_build_report(
        &build_outcome,
        file,
        &request.ctx.store,
        &request.ctx.resolved_state_dir,
        request.ctx.output_mode(),
    )
}

fn evaluate_remote_client_derivation_inputs(
    file: &Path,
    import_paths: &[OsString],
    store_prefix: &str,
) -> Result<Vec<remote_build::RemoteClientDerivationInput>, RunError> {
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal("remote build store prefix must be absolute".to_string()));
    }
    debug_assert!(store_prefix.starts_with('/'));
    debug_assert!(!store_prefix.is_empty());
    let mut session = crunch_eval::session::EvaluationSession::open_file(file, import_paths)
        .map_err(|err| RunError::Eval(format!("{err}")))?;
    let derivations = session
        .force_all_roots::<crunch_glue::CrunchDerivation>()
        .map_err(|err| RunError::Eval(format!("{err}")))?;
    let derivation_count = derivations.len();
    let mut cache = crunch_glue::ConversionCache::new(store_prefix);
    let mut inputs = Vec::with_capacity(derivation_count);
    for (label, crunch_derivation) in derivations {
        let (drv_path, nix_derivation) = crunch_glue::convert(&crunch_derivation, &mut cache)
            .map_err(|err| RunError::Build(format!("{label}: {err}")))?;
        inputs.push(remote_build::RemoteClientDerivationInput {
            label,
            drv_path,
            crunch_derivation,
            nix_derivation,
        });
    }
    debug_assert_eq!(inputs.len(), derivation_count);
    Ok(inputs)
}

fn run_remote_build_dispatches(
    selection: &RemoteBuildSelection,
    inputs: Vec<remote_build::RemoteClientDerivationInput>,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<remote_build::RemoteClientBuildReport, RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|err| RunError::Internal(format!("tokio runtime: {err}")))?;
    rt.block_on(run_remote_build_dispatches_async(selection, inputs, output_dir, state_dir, store_prefix))
}

#[derive(Debug, Clone, Copy)]
struct RemoteFailureReplayCommandInput<'a> {
    bundle_selector: &'a str,
    builder: &'a str,
    ticket: &'a str,
    builder_program: Option<&'a Path>,
    builder_args: &'a [String],
    trusted_builder_keys: &'a [String],
    build_time_limit_secs: u64,
}

fn cmd_remote_failure_debug_replay(
    input: RemoteFailureReplayCommandInput<'_>,
    ctx: &RunContext,
) -> Result<(), RunError> {
    if input.bundle_selector.is_empty() {
        return Err(RunError::Internal("remote failure replay bundle selector must not be empty".to_string()));
    }
    if input.builder.is_empty() {
        return Err(RunError::Internal("remote failure replay builder must not be empty".to_string()));
    }
    debug_assert!(!input.bundle_selector.is_empty());
    debug_assert!(!input.builder.is_empty());
    let bundle_dir = remote_failure_debug::resolve_bundle_selector(&ctx.resolved_state_dir, input.bundle_selector)?;
    let (bundle, policy) =
        remote_failure_debug::load_and_validate_remote_failure_debug_bundle(&bundle_dir).map_err(RunError::Internal)?;
    let now_unix_s = unix_time_now_s()?;
    let _lease = remote_failure_debug::acquire_remote_failure_debug_lease(
        &bundle_dir,
        now_unix_s,
        REMOTE_FAILURE_REPLAY_LEASE_SECS,
    )
    .map_err(RunError::Internal)?;
    let mut request =
        remote_failure_debug::load_remote_failure_replay_request(&bundle_dir).map_err(RunError::Internal)?;
    request.failure_replay = Some(remote_build::RemoteFailureReplayBinding {
        source_bundle_blake3: bundle.bundle_blake3.clone(),
        execution_blake3: new_remote_failure_replay_execution_identity(&bundle.bundle_blake3)?,
    });
    let parsed_ticket = remote_build::parse_remote_ticket_credential(input.ticket).map_err(RunError::Internal)?;
    let (program, args) = remote_stdio_builder_command(input.builder, input.builder_program, input.builder_args, ctx)?;
    let trusted_output_keys =
        remote_trusted_builder_keys(input.trusted_builder_keys, input.builder_program, input.builder, ctx)?;
    let options = remote_build::RemoteClientBuildOptions {
        store_prefix: ctx.store_prefix.clone(),
        ticket: parsed_ticket,
        builder: remote_build::RemoteStdioBuilderCommand {
            endpoint_id: input.builder.to_string(),
            program,
            args,
        },
        trusted_output_keys,
        now_unix_s,
        build_time_limit_secs: input.build_time_limit_secs,
        client_endpoint: None,
        transfer_capabilities: remote_build::RemoteTransferCapabilities::full_only().with_streaming(),
    };
    let rt = tokio::runtime::Runtime::new()
        .map_err(|error| RunError::Internal(format!("remote failure replay tokio runtime: {error}")))?;
    let result = rt.block_on(run_remote_failure_debug_replay_async(RemoteFailureReplayExecutionInput {
        bundle,
        policy,
        request,
        options,
        output_dir: &ctx.store,
        state_dir: &ctx.resolved_state_dir,
        store_prefix: &ctx.store_prefix,
    }))?;
    emit_remote_failure_replay_result(&result, ctx.json)
}

fn emit_remote_failure_replay_result(result: &serde_json::Value, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string(result)
            .map_err(|error| RunError::Internal(format!("serializing remote failure replay report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("source-bundle: {}", result["source_bundle_blake3"].as_str().unwrap_or("invalid"));
        println!("replay-attempt: {}", result["replay_attempt_identity"].as_str().unwrap_or("invalid"));
        println!("comparison: {}", result["comparison"]["class"].as_str().unwrap_or("invalid"));
        println!("output-count: {}", result["admitted_output_count"].as_u64().unwrap_or(0));
    }
    Ok(())
}

fn new_remote_failure_replay_execution_identity(
    source_bundle: &crunch_build::distributed::RemoteFailureDebugDigest,
) -> Result<crunch_build::distributed::RemoteFailureDebugDigest, RunError> {
    use rand::RngCore as _;

    let mut random = [0_u8; REMOTE_FAILURE_REPLAY_RANDOM_BYTES];
    rand::rngs::OsRng
        .try_fill_bytes(&mut random)
        .map_err(|error| RunError::Internal(format!("remote failure replay randomness: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-remote-failure-replay-execution-v1\0");
    hasher.update(source_bundle.as_str().as_bytes());
    hasher.update(&random);
    crunch_build::distributed::RemoteFailureDebugDigest::new(hasher.finalize().to_hex().to_string())
        .map_err(|reason| RunError::Internal(reason.as_str().to_string()))
}

struct RemoteFailureReplayExecutionInput<'a> {
    bundle: crunch_build::distributed::RemoteFailureDebugBundle,
    policy: crunch_build::distributed::RemoteFailureDebugPolicy,
    request: remote_build::ConcreteBuildRequest,
    options: remote_build::RemoteClientBuildOptions,
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
}

struct RemoteFailureReplayRuntime {
    store: crunch_store::StoreHandle,
    coordinator: remote_build::RemoteCoordinatorState,
    dispatch: remote_build::RemoteClientDispatchPlan,
    attempt: remote_build::RemoteProductionAttemptBinding,
    replay_request: remote_build::ConcreteBuildRequest,
    now_unix_s: u64,
}

struct CompletedRemoteFailureReplay {
    admission: remote_build::RemoteOutputAdmissionReport,
    output_admission: remote_build::RemoteOutputImportReport,
}

async fn run_remote_failure_debug_replay_async(
    input: RemoteFailureReplayExecutionInput<'_>,
) -> Result<serde_json::Value, RunError> {
    debug_assert!(input.store_prefix.starts_with('/'));
    debug_assert!(!input.state_dir.as_os_str().is_empty());
    if input.request.store_prefix != input.store_prefix {
        return Err(RunError::Internal("remote failure replay store-prefix mismatch".to_string()));
    }
    let _coordinator_mutation_guard =
        remote_build::acquire_remote_coordinator_mutation_guard(input.state_dir).map_err(RunError::Internal)?;
    let bundle = input.bundle.clone();
    let mut runtime = prepare_remote_failure_replay_runtime(input).await?;
    let completed = execute_remote_failure_replay(&mut runtime).await?;
    complete_remote_failure_replay_report(bundle, &mut runtime, completed)
}

async fn open_remote_failure_replay_store(
    input: &RemoteFailureReplayExecutionInput<'_>,
) -> Result<crunch_store::StoreHandle, RunError> {
    debug_assert!(input.store_prefix.starts_with('/'));
    debug_assert!(!input.state_dir.as_os_str().is_empty());
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: input.state_dir.to_path_buf(),
        output_dir: input.output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: input.store_prefix.to_string(),
        base_state_dirs: Vec::new(),
    })
    .await
    .map_err(|error| RunError::Internal(format!("opening replay store: {error}")))
}

async fn prepare_remote_failure_replay_runtime(
    input: RemoteFailureReplayExecutionInput<'_>,
) -> Result<RemoteFailureReplayRuntime, RunError> {
    debug_assert!(input.store_prefix.starts_with('/'));
    debug_assert_eq!(input.request.store_prefix, input.store_prefix);
    let store = open_remote_failure_replay_store(&input).await?;
    let mut coordinator = remote_build::load_coordinator_state(input.state_dir)?;
    let mut dispatch =
        remote_build::plan_remote_stdio_replay_dispatch(input.request, &input.options).map_err(RunError::Internal)?;
    let replay_worker = coordinator
        .workers
        .get(&input.options.builder.endpoint_id)
        .cloned()
        .ok_or_else(|| RunError::Internal("remote failure replay worker registration missing".to_string()))?;
    let attempt = remote_build::admit_remote_production_dispatch(
        &mut coordinator,
        &dispatch.client.request,
        &input.options.builder.endpoint_id,
        replay_worker.worker_generation,
        replay_worker.concurrency,
        replay_worker.resource_inventory,
        &dispatch.client.trusted_output_keys,
        input.options.now_unix_s,
    )
    .map_err(|error| RunError::Internal(format!("remote failure replay assignment: {error}")))?;
    if let Err(error) = remote_build::bind_remote_production_dispatch(
        &mut dispatch,
        attempt.clone(),
        standard_remote_transfer_policy(),
        input.policy,
        input.state_dir,
    ) {
        return Err(fail_remote_failure_replay(
            &mut coordinator,
            &attempt,
            input.options.now_unix_s,
            format!("remote failure replay dispatch binding: {error}"),
        ));
    }
    let replay_request = dispatch.client.request.clone();
    if let Err(error) = remote_build::prepare_remote_production_input_transfer(
        &store,
        &mut dispatch.command,
        &replay_request,
        input.state_dir,
    )
    .await
    {
        return Err(fail_remote_failure_replay(
            &mut coordinator,
            &attempt,
            input.options.now_unix_s,
            format!("remote failure replay input transfer: {error}"),
        ));
    }
    Ok(RemoteFailureReplayRuntime {
        store,
        coordinator,
        dispatch,
        attempt,
        replay_request,
        now_unix_s: input.options.now_unix_s,
    })
}

async fn execute_remote_failure_replay(
    runtime: &mut RemoteFailureReplayRuntime,
) -> Result<CompletedRemoteFailureReplay, RunError> {
    debug_assert_eq!(runtime.dispatch.client.request.request_id, runtime.replay_request.request_id);
    debug_assert!(!runtime.dispatch.client.trusted_output_keys.is_empty());
    let transcript = match remote_build::run_stdio_remote_child(&runtime.dispatch.command) {
        Ok(transcript) => transcript,
        Err(error) => {
            return Err(fail_remote_failure_replay(
                &mut runtime.coordinator,
                &runtime.attempt,
                runtime.now_unix_s,
                format!("remote failure replay child: {error}"),
            ));
        }
    };
    let admission = match remote_build::admit_fenced_remote_stdio_output(
        &mut runtime.coordinator,
        &runtime.attempt,
        true,
        &runtime.replay_request,
        &runtime.dispatch.client.trusted_output_keys,
        &transcript,
        runtime.now_unix_s,
    ) {
        Ok(admission) => admission,
        Err(error) => {
            return Err(fail_remote_failure_replay(
                &mut runtime.coordinator,
                &runtime.attempt,
                runtime.now_unix_s,
                format!("remote failure replay output admission: {error}"),
            ));
        }
    };
    let output_admission = match remote_build::import_admitted_remote_stdio_outputs(
        &mut runtime.store,
        &runtime.replay_request,
        &admission,
        &transcript,
        true,
        Some(crunch_store::GcRootSource::Build),
    )
    .await
    {
        Ok(report) => report,
        Err(error) => {
            return Err(fail_remote_failure_replay(
                &mut runtime.coordinator,
                &runtime.attempt,
                runtime.now_unix_s,
                format!("remote failure replay output import: {error}"),
            ));
        }
    };
    complete_remote_failure_replay_attempt(runtime, &admission)?;
    Ok(CompletedRemoteFailureReplay {
        admission,
        output_admission,
    })
}

fn complete_remote_failure_replay_attempt(
    runtime: &mut RemoteFailureReplayRuntime,
    admission: &remote_build::RemoteOutputAdmissionReport,
) -> Result<(), RunError> {
    debug_assert_eq!(admission.request_id, runtime.replay_request.request_id);
    debug_assert!(!admission.output_digest_blake3.is_empty());
    if let Err(error) = remote_build::complete_remote_production_attempt(
        &mut runtime.coordinator,
        &runtime.attempt,
        &admission.output_digest_blake3,
        runtime.now_unix_s.saturating_add(1),
    ) {
        return Err(fail_remote_failure_replay(
            &mut runtime.coordinator,
            &runtime.attempt,
            runtime.now_unix_s,
            format!("remote failure replay completion: {error}"),
        ));
    }
    Ok(())
}

fn complete_remote_failure_replay_report(
    bundle: crunch_build::distributed::RemoteFailureDebugBundle,
    runtime: &mut RemoteFailureReplayRuntime,
    completed: CompletedRemoteFailureReplay,
) -> Result<serde_json::Value, RunError> {
    debug_assert_eq!(completed.admission.request_id, runtime.replay_request.request_id);
    debug_assert!(!completed.output_admission.outputs.is_empty());
    let comparison = compare_completed_remote_failure_replay(&bundle, &completed.admission)?;
    let attempt_identity = format!(
        "{}:{}:{}",
        runtime.attempt.job_id.as_str(),
        runtime.attempt.attempt_id.as_str(),
        runtime.attempt.fence_generation.get()
    );
    let status = remote_build::RemoteFailureDebugStatus {
        bundle_ref: format!("remote-failure-debug:{}", bundle.bundle_blake3.as_str()),
        capture_outcome_code: bundle.capture_outcome_code.clone(),
        cleanup_status_code: bundle.cleanup_status_code.clone(),
        immutable_log_available: bundle.immutable_log.is_some(),
        non_claim: crunch_build::distributed::REMOTE_FAILURE_DEBUG_NON_CLAIM.to_string(),
        worker_bundle_ref: None,
        replay_attempt_identity: Some(attempt_identity.clone()),
        replay_comparison_class: Some(format!("{:?}", comparison.class).to_ascii_lowercase()),
    };
    remote_build::record_remote_failure_debug_status(&mut runtime.coordinator, &runtime.attempt, status)
        .map_err(RunError::Internal)?;
    Ok(serde_json::json!({
        "schema": "mantle-remote-failure-replay-report-v1",
        "source_bundle_blake3": bundle.bundle_blake3,
        "replay_attempt_identity": attempt_identity,
        "admitted_output_digest_blake3": completed.admission.output_digest_blake3,
        "admitted_output_count": completed.output_admission.outputs.len(),
        "comparison": comparison,
        "original_result_immutable": true,
        "ordinary_output_admission_applied": true,
        "non_claim": crunch_build::distributed::REMOTE_FAILURE_DEBUG_NON_CLAIM,
    }))
}

fn fail_remote_failure_replay(
    coordinator: &mut remote_build::RemoteCoordinatorState,
    attempt: &remote_build::RemoteProductionAttemptBinding,
    now_unix_s: u64,
    reason: String,
) -> RunError {
    match remote_build::fail_remote_production_attempt(coordinator, attempt, now_unix_s.saturating_add(1)) {
        Ok(()) => RunError::Internal(reason),
        Err(failure_error) => RunError::Internal(format!(
            "{reason}; remote failure replay failure-state persistence degraded: {failure_error}"
        )),
    }
}

fn compare_completed_remote_failure_replay(
    bundle: &crunch_build::distributed::RemoteFailureDebugBundle,
    admission: &remote_build::RemoteOutputAdmissionReport,
) -> Result<crunch_build::distributed::RemoteFailureReplayComparison, RunError> {
    debug_assert!(!bundle.bundle_blake3.as_str().is_empty());
    debug_assert!(!admission.output_digest_blake3.is_empty());
    let original_log = bundle
        .immutable_log
        .as_ref()
        .and_then(|log| log.head_record_blake3.as_ref())
        .map(|digest| crunch_build::distributed::RemoteFailureDebugDigest::new(digest.as_str().to_string()))
        .transpose()
        .map_err(|reason| RunError::Internal(reason.as_str().to_string()))?;
    let replay_output =
        crunch_build::distributed::RemoteFailureDebugDigest::new(admission.output_digest_blake3.clone())
            .map_err(|reason| RunError::Internal(reason.as_str().to_string()))?;
    crunch_build::distributed::compare_remote_failure_replay(
        &crunch_build::distributed::RemoteFailureExecutionSummary {
            failure_phase: Some(bundle.failure_phase),
            failure_reason_code: Some(bundle.failure_reason_code.clone()),
            exit_status_class: "failure".to_string(),
            admitted_output_digests: Vec::new(),
            log_head_blake3: original_log,
            captured_manifest_blake3: bundle
                .captured_artifact_manifest_ref
                .as_ref()
                .map(|reference| reference.digest_blake3.clone()),
        },
        &crunch_build::distributed::RemoteFailureExecutionSummary {
            failure_phase: None,
            failure_reason_code: None,
            exit_status_class: "success".to_string(),
            admitted_output_digests: vec![replay_output],
            log_head_blake3: None,
            captured_manifest_blake3: None,
        },
    )
    .map_err(|reason| RunError::Internal(reason.as_str().to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteWorkerFailureDebugReport {
    bundle_ref: String,
    capture_outcome_code: String,
    cleanup_status_code: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct RemoteFailureObservabilityOutcome {
    immutable_log: Result<remote_build::RemoteAttemptLogControlSummary, String>,
    exporters: remote_telemetry_export::RemoteTelemetryExportReport,
    failure_debug: Option<Result<remote_failure_debug::RemoteFailureDebugPublishOutcome, String>>,
    attempt_failure: Option<Result<(), String>>,
    worker_debug: Option<RemoteWorkerFailureDebugReport>,
}

struct RemoteFailureDebugEmissionContext<'a> {
    policy: &'a crunch_build::distributed::RemoteFailureDebugPolicy,
    request: &'a remote_build::ConcreteBuildRequest,
    route_class: &'a str,
    created_unix_s: u64,
    worker_debug: Option<RemoteWorkerFailureDebugReport>,
}

fn parse_remote_worker_failure_debug_report(reason: &str) -> Option<RemoteWorkerFailureDebugReport> {
    debug_assert!(!REMOTE_FAILURE_DEBUG_BUNDLE_REF_PREFIX.is_empty());
    let bundle_ref = remote_failure_debug_report_field(RemoteFailureFieldQuery {
        reason,
        prefix: "worker_bundle_ref=",
    })?;
    let capture_outcome_code = remote_failure_debug_report_field(RemoteFailureFieldQuery {
        reason,
        prefix: "worker_capture_outcome=",
    })?;
    let cleanup_status_code = if reason.contains("remote-failure-workspace-cleanup-degraded") {
        "cleanup-degraded".to_string()
    } else {
        remote_failure_debug_report_field(RemoteFailureFieldQuery {
            reason,
            prefix: "worker_cleanup_status=",
        })?
    };
    let digest = bundle_ref.strip_prefix(REMOTE_FAILURE_DEBUG_BUNDLE_REF_PREFIX)?;
    if digest.len() != REMOTE_FAILURE_DEBUG_DIGEST_HEX_CHARS
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return None;
    }
    if !valid_remote_failure_status_code(&capture_outcome_code)
        || !valid_remote_failure_status_code(&cleanup_status_code)
    {
        return None;
    }
    Some(RemoteWorkerFailureDebugReport {
        bundle_ref,
        capture_outcome_code,
        cleanup_status_code,
    })
}

struct RemoteFailureFieldQuery<'a> {
    reason: &'a str,
    prefix: &'a str,
}

fn remote_failure_debug_report_field(query: RemoteFailureFieldQuery<'_>) -> Option<String> {
    query
        .reason
        .split(';')
        .map(str::trim)
        .find_map(|field| field.strip_prefix(query.prefix).map(str::to_string))
}

fn valid_remote_failure_status_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= REMOTE_FAILURE_DEBUG_STATUS_CODE_BYTES_MAX
        && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn remote_protocol_failure_fact(reason: &str) -> remote_build::RemoteProductionTelemetryFact {
    if reason.contains(remote_build::RemoteAttemptReasonCode::StaleReportRejected.as_str()) {
        return remote_build::RemoteProductionTelemetryFact::StaleFenceRejected;
    }
    if reason.contains(REMOTE_TRANSFER_INTERRUPTION_REASON_FRAGMENT) {
        return remote_build::RemoteProductionTelemetryFact::TransferCutoff { accepted: false };
    }
    remote_build::RemoteProductionTelemetryFact::ExecutionFailed
}

struct RemoteFailureObservabilityInput<'a> {
    export_config: &'a remote_telemetry_export::RemoteTelemetryExportConfig,
    telemetry_policy: crunch_build::distributed::RemoteTelemetryPolicy,
    state_dir: &'a Path,
    coordinator: &'a mut remote_build::RemoteCoordinatorState,
    attempt: &'a remote_build::RemoteProductionAttemptBinding,
    telemetry: &'a mut crunch_build::distributed::RemoteTelemetryBuffer,
    fact: remote_build::RemoteProductionTelemetryFact,
    debug_context: Option<RemoteFailureDebugEmissionContext<'a>>,
}

fn record_remote_failure_observability(
    input: RemoteFailureObservabilityInput<'_>,
) -> RemoteFailureObservabilityOutcome {
    let RemoteFailureObservabilityInput {
        export_config,
        telemetry_policy,
        state_dir,
        coordinator,
        attempt,
        telemetry,
        fact,
        debug_context,
    } = input;
    assert!(matches!(
        fact,
        remote_build::RemoteProductionTelemetryFact::StaleFenceRejected
            | remote_build::RemoteProductionTelemetryFact::ExecutionFailed
            | remote_build::RemoteProductionTelemetryFact::OutputRejected
            | remote_build::RemoteProductionTelemetryFact::TransferCutoff { accepted: false }
    ));
    let accepted_events_before = telemetry.accepted_events;
    *telemetry = remote_build::record_remote_production_telemetry(telemetry, fact, telemetry_policy);
    let immutable_log = if fact == remote_build::RemoteProductionTelemetryFact::StaleFenceRejected {
        remote_build::append_remote_rejected_production_observability_log(state_dir, attempt, telemetry)
    } else {
        remote_build::append_remote_production_observability_log(coordinator, attempt, telemetry)
    };
    let telemetry_delivery = remote_telemetry_export::export_remote_telemetry(export_config, &telemetry.events);
    let worker_debug = debug_context.as_ref().and_then(|context| context.worker_debug.clone());
    let failure_debug = debug_context.map(|context| {
        let result = publish_remote_failure_debug_observability(
            state_dir,
            attempt,
            immutable_log.as_ref().ok().cloned(),
            fact,
            context,
        );
        if let Ok(outcome) = &result {
            let status = remote_build::RemoteFailureDebugStatus {
                bundle_ref: outcome.bundle_ref.clone(),
                capture_outcome_code: worker_debug
                    .as_ref()
                    .map_or_else(|| outcome.capture_outcome_code.clone(), |worker| worker.capture_outcome_code.clone()),
                cleanup_status_code: worker_debug
                    .as_ref()
                    .map_or_else(|| outcome.cleanup_status_code.clone(), |worker| worker.cleanup_status_code.clone()),
                immutable_log_available: outcome.immutable_log_available,
                non_claim: outcome.non_claim.clone(),
                worker_bundle_ref: worker_debug.as_ref().map(|worker| worker.bundle_ref.clone()),
                replay_attempt_identity: None,
                replay_comparison_class: None,
            };
            let _diagnostic_status = remote_build::record_remote_failure_debug_status(coordinator, attempt, status);
        }
        result
    });
    let attempt_failure = fail_remote_attempt_after_observability(coordinator, attempt, telemetry, fact);
    debug_assert!(telemetry.accepted_events >= accepted_events_before);
    RemoteFailureObservabilityOutcome {
        immutable_log,
        exporters: telemetry_delivery,
        failure_debug,
        attempt_failure,
        worker_debug,
    }
}

fn fail_remote_attempt_after_observability(
    coordinator: &mut remote_build::RemoteCoordinatorState,
    attempt: &remote_build::RemoteProductionAttemptBinding,
    telemetry: &crunch_build::distributed::RemoteTelemetryBuffer,
    fact: remote_build::RemoteProductionTelemetryFact,
) -> Option<Result<(), String>> {
    debug_assert!(!attempt.job_id.as_str().is_empty());
    debug_assert!(!attempt.attempt_id.as_str().is_empty());
    if matches!(
        fact,
        remote_build::RemoteProductionTelemetryFact::StaleFenceRejected
            | remote_build::RemoteProductionTelemetryFact::TransferCutoff { accepted: false }
    ) {
        return None;
    }
    Some(remote_build::fail_remote_production_attempt(
        coordinator,
        attempt,
        telemetry.accepted_events.saturating_add(1),
    ))
}

fn annotate_remote_failure_error(error: RunError, observability: &RemoteFailureObservabilityOutcome) -> RunError {
    debug_assert!(!observability.exporters.schema.is_empty());
    debug_assert!(observability.exporters.events_received >= observability.exporters.events_dropped);
    let Some(Ok(debug)) = observability.failure_debug.as_ref() else {
        return error;
    };
    let worker = observability.worker_debug.as_ref().map_or_else(String::new, |worker| {
        format!(
            "; worker_bundle_ref={}; worker_capture_outcome={}; worker_cleanup_status={}",
            worker.bundle_ref, worker.capture_outcome_code, worker.cleanup_status_code,
        )
    });
    RunError::Internal(format!(
        "{error}; bundle_ref={}; capture_outcome={}; cleanup_status={}; immutable_log_available={}; non_claim={}{}",
        debug.bundle_ref,
        debug.capture_outcome_code,
        debug.cleanup_status_code,
        debug.immutable_log_available,
        debug.non_claim,
        worker,
    ))
}

fn publish_remote_failure_debug_observability(
    state_dir: &Path,
    attempt: &remote_build::RemoteProductionAttemptBinding,
    immutable_log: Option<remote_build::RemoteAttemptLogControlSummary>,
    fact: remote_build::RemoteProductionTelemetryFact,
    context: RemoteFailureDebugEmissionContext<'_>,
) -> Result<remote_failure_debug::RemoteFailureDebugPublishOutcome, String> {
    debug_assert!(!context.route_class.is_empty());
    debug_assert!(context.created_unix_s > 0);
    let (failure_phase, failure_reason_code, transfer_status_class, admission_status_class) =
        remote_failure_debug_fact_classes(fact);
    remote_failure_debug::publish_remote_failure_debug_bundle(remote_failure_debug::RemoteFailureDebugPublishRequest {
        state_dir,
        capture_root: None,
        facts: remote_failure_debug::RemoteFailureDebugSourceFacts {
            request: context.request.clone(),
            attempt: attempt.clone(),
            immutable_log,
            route_class: context.route_class.to_string(),
            worker_capability_classes: vec!["fenced-output-admission".to_string(), "framed-stdio".to_string()],
            sandbox_policy_class: "remote-builder-policy".to_string(),
            network_policy_class: "remote-builder-policy".to_string(),
            transfer_status_class: transfer_status_class.to_string(),
            admission_status_class: admission_status_class.to_string(),
            workspace_mode: crunch_build::distributed::RemoteFailureWorkspaceMode::Ephemeral,
            failure_phase,
            failure_reason_code: failure_reason_code.to_string(),
            cleanup_status_code: "cleanup-not-observed".to_string(),
            created_unix_s: context.created_unix_s,
        },
        policy: context.policy.clone(),
    })
}

fn standard_remote_transfer_policy() -> crunch_build::distributed::RemoteTransferPolicy {
    crunch_build::distributed::RemoteTransferPolicy {
        chunk_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CHUNK_BYTES,
        in_flight_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_BYTES,
        in_flight_chunks_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS,
        buffered_chunks_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_BUFFERED_CHUNKS,
        artifact_count_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_ARTIFACTS,
        chunk_count_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CHUNKS,
        total_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_TOTAL_BYTES,
        checkpoint_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CHECKPOINT_BYTES,
        idle_progress_steps_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS,
        replay_rounds_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_REPLAY_ROUNDS,
        control_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CONTROL_BYTES,
    }
}

fn standard_scheduling_policy() -> crunch_build::scheduling::SchedulingPolicy {
    crunch_build::scheduling::SchedulingPolicy {
        schema: crunch_build::scheduling::SCHEDULING_POLICY_SCHEMA.to_string(),
        policy_id: crunch_build::scheduling::DEFAULT_SCHEDULING_POLICY_ID.to_string(),
        preference_order: vec![
            crunch_build::scheduling::PreferenceField::KnownGraph,
            crunch_build::scheduling::PreferenceField::ResourceFit,
            crunch_build::scheduling::PreferenceField::LocalityTransfer,
        ],
        aged_after_epochs: crunch_build::scheduling::DEFAULT_AGED_AFTER_EPOCHS,
        protected_after_epochs: crunch_build::scheduling::DEFAULT_PROTECTED_AFTER_EPOCHS,
    }
}

fn empty_remote_telemetry_buffer() -> crunch_build::distributed::RemoteTelemetryBuffer {
    crunch_build::distributed::RemoteTelemetryBuffer {
        events: Vec::new(),
        accepted_events: 0,
        dropped_events: 0,
        last_drop_reason: None,
    }
}

fn remote_failure_debug_fact_classes(
    fact: remote_build::RemoteProductionTelemetryFact,
) -> (crunch_build::distributed::RemoteFailureDebugPhase, &'static str, &'static str, &'static str) {
    use crunch_build::distributed::RemoteFailureDebugPhase;
    match fact {
        remote_build::RemoteProductionTelemetryFact::StaleFenceRejected => {
            (RemoteFailureDebugPhase::OutputAdmission, "stale-fence-rejected", "not-available", "rejected")
        }
        remote_build::RemoteProductionTelemetryFact::ExecutionFailed => {
            (RemoteFailureDebugPhase::Execution, "execution-failed", "complete", "not-admitted")
        }
        remote_build::RemoteProductionTelemetryFact::OutputRejected => {
            (RemoteFailureDebugPhase::OutputAdmission, "output-rejected", "complete", "rejected")
        }
        remote_build::RemoteProductionTelemetryFact::TransferCutoff { accepted: false } => {
            (RemoteFailureDebugPhase::InputTransfer, "transfer-cutoff", "interrupted", "not-available")
        }
        _ => (RemoteFailureDebugPhase::Route, "unsupported-failure-fact", "not-available", "not-available"),
    }
}

struct PreparedRemoteDispatch {
    plan: remote_build::RemoteClientDispatchPlan,
    attempt: remote_build::RemoteProductionAttemptBinding,
    telemetry: crunch_build::distributed::RemoteTelemetryBuffer,
    telemetry_policy: crunch_build::distributed::RemoteTelemetryPolicy,
    request: remote_build::ConcreteBuildRequest,
}

struct RemoteDispatchPreparationInput<'a> {
    selection: &'a RemoteBuildSelection,
    state_dir: &'a Path,
    coordinator: &'a mut remote_build::RemoteCoordinatorState,
    input: remote_build::RemoteClientDerivationInput,
    priority_competing_goal_count: Option<u32>,
}

struct RemoteDispatchExecutionInput<'a> {
    selection: &'a RemoteBuildSelection,
    state_dir: &'a Path,
    store: &'a mut crunch_store::StoreHandle,
    coordinator: &'a mut remote_build::RemoteCoordinatorState,
    prepared: &'a mut PreparedRemoteDispatch,
}

async fn run_remote_build_dispatches_async(
    selection: &RemoteBuildSelection,
    inputs: Vec<remote_build::RemoteClientDerivationInput>,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<remote_build::RemoteClientBuildReport, RunError> {
    assert!(store_prefix.starts_with('/'));
    assert_eq!(store_prefix, selection.options.store_prefix);
    let _coordinator_mutation_guard =
        remote_build::acquire_remote_coordinator_mutation_guard(state_dir).map_err(RunError::Internal)?;
    let mut store = open_remote_dispatch_store(output_dir, state_dir, store_prefix).await?;
    let mut coordinator = remote_build::load_coordinator_state(state_dir)?;
    let priority_plan = plan_remote_dispatch_priority(selection, &inputs)?;
    let competing_goal_count = priority_plan.evidence.competing_goal_count;
    let mut input_slots = inputs.into_iter().map(Some).collect::<Vec<_>>();
    let mut accepted_builds = Vec::with_capacity(input_slots.len());
    for (priority_index, selected_input_index) in priority_plan.ordered_input_indices.iter().enumerate() {
        let selected_slot = usize::try_from(*selected_input_index)
            .map_err(|_| RunError::Internal("remote root index does not fit in usize".to_string()))?;
        let input = input_slots
            .get_mut(selected_slot)
            .and_then(Option::take)
            .ok_or_else(|| RunError::Internal("remote priority selected missing input".to_string()))?;
        let mut prepared = prepare_remote_dispatch(RemoteDispatchPreparationInput {
            selection,
            state_dir,
            coordinator: &mut coordinator,
            input,
            priority_competing_goal_count: (priority_index == 0).then_some(competing_goal_count),
        })?;
        let accepted_build = execute_remote_dispatch(RemoteDispatchExecutionInput {
            selection,
            state_dir,
            store: &mut store,
            coordinator: &mut coordinator,
            prepared: &mut prepared,
        })
        .await?;
        accepted_builds.push(accepted_build);
    }
    assert_eq!(accepted_builds.len(), input_slots.len());
    assert!(input_slots.iter().all(Option::is_none));
    Ok(remote_build::RemoteClientBuildReport {
        schema: REMOTE_CLIENT_BUILD_REPORT_SCHEMA.to_string(),
        builder: selection.options.builder.endpoint_id.clone(),
        store_prefix: store_prefix.to_string(),
        priority_decisions: vec![priority_plan.evidence],
        imported: accepted_builds,
    })
}

async fn open_remote_dispatch_store(
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<crunch_store::StoreHandle, RunError> {
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_prefix.to_string(),
        base_state_dirs: Vec::new(),
    })
    .await
    .map_err(|err| RunError::Internal(format!("opening local store for remote build dispatch: {err}")))
}

fn plan_remote_dispatch_priority(
    selection: &RemoteBuildSelection,
    inputs: &[remote_build::RemoteClientDerivationInput],
) -> Result<remote_build::RemoteRootPriorityPlan, RunError> {
    debug_assert!(selection.options.store_prefix.starts_with('/'));
    debug_assert!(!inputs.is_empty());
    let ready_roots = inputs
        .iter()
        .enumerate()
        .map(|(input_index, input)| {
            let input_index = u32::try_from(input_index)
                .map_err(|_| RunError::Internal("remote root index does not fit in u32".to_string()))?;
            let root_identity_blake3 = remote_build::remote_root_priority_identity(
                &input.label,
                &input.drv_path,
                &selection.options.store_prefix,
            )
            .map_err(RunError::Internal)?;
            Ok(remote_build::RemoteRootPriorityCandidate {
                input_index,
                root_identity_blake3,
            })
        })
        .collect::<Result<Vec<_>, RunError>>()?;
    remote_build::plan_remote_root_priority(&standard_scheduling_policy(), &ready_roots).map_err(RunError::Internal)
}

fn prepare_remote_dispatch(input: RemoteDispatchPreparationInput<'_>) -> Result<PreparedRemoteDispatch, RunError> {
    debug_assert!(!input.selection.options.builder.endpoint_id.is_empty());
    debug_assert!(input.selection.worker_concurrency > 0);
    let telemetry_policy = input.selection.telemetry.telemetry;
    let mut telemetry = empty_remote_telemetry_buffer();
    if let Some(competing_goal_count) = input.priority_competing_goal_count {
        telemetry = remote_build::record_remote_production_telemetry(
            &telemetry,
            remote_build::RemoteProductionTelemetryFact::PrioritySelected { competing_goal_count },
            telemetry_policy,
        );
    }
    telemetry = remote_build::record_remote_production_telemetry(
        &telemetry,
        remote_build::RemoteProductionTelemetryFact::RouteSelected,
        telemetry_policy,
    );
    let mut plan = remote_build::plan_remote_stdio_client_dispatch(input.input, &input.selection.options)
        .map_err(RunError::Internal)?;
    let attempt = remote_build::admit_remote_production_dispatch(
        input.coordinator,
        &plan.client.request,
        &input.selection.options.builder.endpoint_id,
        input.selection.worker_generation,
        input.selection.worker_concurrency,
        input.selection.resource_inventory.clone(),
        &plan.client.trusted_output_keys,
        input.selection.options.now_unix_s,
    )
    .map_err(|err| RunError::Internal(format!("remote coordinator dispatch failed: {err}")))?;
    for fact in [
        remote_build::RemoteProductionTelemetryFact::QueueAdmitted,
        remote_build::RemoteProductionTelemetryFact::WorkerAssigned,
        remote_build::RemoteProductionTelemetryFact::FenceAccepted,
    ] {
        telemetry = remote_build::record_remote_production_telemetry(&telemetry, fact, telemetry_policy);
    }
    remote_build::bind_remote_production_dispatch(
        &mut plan,
        attempt.clone(),
        standard_remote_transfer_policy(),
        input.selection.failure_debug.clone(),
        input.state_dir,
    )
    .map_err(|err| RunError::Internal(format!("remote production transfer binding failed: {err}")))?;
    configure_remote_dispatch_diagnostics(input.selection, &mut plan)?;
    let request = plan.client.request.clone();
    Ok(PreparedRemoteDispatch {
        plan,
        attempt,
        telemetry,
        telemetry_policy,
        request,
    })
}

fn configure_remote_dispatch_diagnostics(
    selection: &RemoteBuildSelection,
    plan: &mut remote_build::RemoteClientDispatchPlan,
) -> Result<(), RunError> {
    remote_build::set_remote_production_telemetry_policy(&mut plan.command, selection.telemetry.telemetry)
        .map_err(RunError::Internal)?;
    if selection.trace_health.status != remote_trace_context::RemoteTraceContextStatus::Disabled {
        remote_build::set_remote_diagnostic_trace_context(&mut plan.command, selection.trace_context.clone())
            .map_err(RunError::Internal)?;
    }
    configure_remote_test_interruption(&mut plan.command, REMOTE_TEST_INTERRUPT_AFTER_INPUT_CHUNKS_ENV, true)?;
    configure_remote_test_interruption(&mut plan.command, REMOTE_TEST_INTERRUPT_AFTER_OUTPUT_CHUNKS_ENV, false)
}

fn configure_remote_test_interruption(
    command: &mut remote_build::RemoteStdioCommand,
    environment_variable: &str,
    is_input: bool,
) -> Result<(), RunError> {
    if !cfg!(debug_assertions) {
        return Ok(());
    }
    let Some(value) = std::env::var_os(environment_variable) else {
        return Ok(());
    };
    let chunk_count = value
        .to_str()
        .ok_or_else(|| RunError::Internal("remote test interruption count is not UTF-8".to_string()))?
        .parse::<u32>()
        .map_err(|err| RunError::Internal(format!("remote test interruption count is invalid: {err}")))?;
    if is_input {
        remote_build::set_remote_production_interrupt_after_input_chunks(command, chunk_count)
    } else {
        remote_build::set_remote_production_interrupt_after_output_chunks(command, chunk_count)
    }
    .map_err(RunError::Internal)
}

async fn execute_remote_dispatch(
    mut input: RemoteDispatchExecutionInput<'_>,
) -> Result<remote_build::RemoteClientImportedBuild, RunError> {
    debug_assert_eq!(input.prepared.request.request_id, input.prepared.plan.client.request.request_id);
    debug_assert!(!input.prepared.plan.client.trusted_output_keys.is_empty());
    remote_build::prepare_remote_production_input_transfer(
        input.store,
        &mut input.prepared.plan.command,
        &input.prepared.request,
        input.state_dir,
    )
    .await
    .map_err(|err| RunError::Internal(format!("remote input stream preparation failed: {err}")))?;
    let transcript = run_remote_dispatch_child(&mut input)?;
    for event in &transcript.telemetry.events {
        input.prepared.telemetry = crunch_build::distributed::record_remote_telemetry(
            &input.prepared.telemetry,
            event.clone(),
            input.prepared.telemetry_policy,
        );
    }
    let (admission, output_admission) = admit_remote_dispatch_outputs(&mut input, &transcript).await?;
    finalize_remote_dispatch(input, transcript, admission, output_admission)
}

fn run_remote_dispatch_child(
    input: &mut RemoteDispatchExecutionInput<'_>,
) -> Result<remote_build::RemoteStdioTranscript, RunError> {
    match remote_build::run_stdio_remote_child(&input.prepared.plan.command) {
        Ok(transcript) => Ok(transcript),
        Err(error) => {
            let failure_fact = remote_protocol_failure_fact(&error.to_string());
            let failure_observability = record_remote_failure_observability(RemoteFailureObservabilityInput {
                export_config: &input.selection.telemetry,
                telemetry_policy: input.prepared.telemetry_policy,
                state_dir: input.state_dir,
                coordinator: input.coordinator,
                attempt: &input.prepared.attempt,
                telemetry: &mut input.prepared.telemetry,
                fact: failure_fact,
                debug_context: Some(RemoteFailureDebugEmissionContext {
                    policy: &input.selection.failure_debug,
                    request: &input.prepared.request,
                    route_class: &input.selection.options.builder.endpoint_id,
                    created_unix_s: input.selection.options.now_unix_s,
                    worker_debug: parse_remote_worker_failure_debug_report(&error.to_string()),
                }),
            });
            Err(annotate_remote_failure_error(error, &failure_observability))
        }
    }
}

async fn admit_remote_dispatch_outputs(
    input: &mut RemoteDispatchExecutionInput<'_>,
    transcript: &remote_build::RemoteStdioTranscript,
) -> Result<(remote_build::RemoteOutputAdmissionReport, remote_build::RemoteOutputImportReport), RunError> {
    debug_assert_eq!(input.prepared.request.request_id, input.prepared.plan.client.request.request_id);
    debug_assert!(!input.prepared.plan.client.trusted_output_keys.is_empty());
    let admission = match remote_build::admit_fenced_remote_stdio_output(
        input.coordinator,
        &input.prepared.attempt,
        true,
        &input.prepared.request,
        &input.prepared.plan.client.trusted_output_keys,
        transcript,
        input.selection.options.now_unix_s,
    ) {
        Ok(admission) => admission,
        Err(error) => {
            let failure_observability = record_remote_failure_observability(RemoteFailureObservabilityInput {
                export_config: &input.selection.telemetry,
                telemetry_policy: input.prepared.telemetry_policy,
                state_dir: input.state_dir,
                coordinator: input.coordinator,
                attempt: &input.prepared.attempt,
                telemetry: &mut input.prepared.telemetry,
                fact: remote_build::RemoteProductionTelemetryFact::OutputRejected,
                debug_context: Some(RemoteFailureDebugEmissionContext {
                    policy: &input.selection.failure_debug,
                    request: &input.prepared.request,
                    route_class: &input.selection.options.builder.endpoint_id,
                    created_unix_s: input.selection.options.now_unix_s,
                    worker_debug: None,
                }),
            });
            return Err(annotate_remote_failure_error(
                RunError::Internal(format!("remote output import admission failed: {error}")),
                &failure_observability,
            ));
        }
    };
    let output_admission = remote_build::import_admitted_remote_stdio_outputs(
        input.store,
        &input.prepared.request,
        &admission,
        transcript,
        true,
        Some(crunch_store::GcRootSource::Build),
    )
    .await
    .map_err(|err| RunError::Internal(format!("remote output import failed: {err}")))?;
    debug_assert_eq!(admission.request_id, input.prepared.request.request_id);
    debug_assert_eq!(output_admission.request_id, input.prepared.request.request_id);
    Ok((admission, output_admission))
}

fn finalize_remote_dispatch(
    input: RemoteDispatchExecutionInput<'_>,
    transcript: remote_build::RemoteStdioTranscript,
    admission: remote_build::RemoteOutputAdmissionReport,
    output_admission: remote_build::RemoteOutputImportReport,
) -> Result<remote_build::RemoteClientImportedBuild, RunError> {
    debug_assert_eq!(admission.request_id, input.prepared.request.request_id);
    debug_assert_eq!(output_admission.request_id, input.prepared.request.request_id);
    debug_assert!(!output_admission.outputs.is_empty());
    let telemetry = remote_build::record_remote_production_telemetry(
        &input.prepared.telemetry,
        remote_build::RemoteProductionTelemetryFact::OutputAdmitted,
        input.prepared.telemetry_policy,
    );
    remote_build::complete_remote_production_attempt(
        input.coordinator,
        &input.prepared.attempt,
        &admission.output_digest_blake3,
        input.selection.options.now_unix_s.saturating_add(1),
    )
    .map_err(|err| RunError::Internal(format!("remote coordinator completion failed: {err}")))?;
    let immutable_log_result = remote_build::append_remote_production_observability_log(
        input.coordinator,
        &input.prepared.attempt,
        &telemetry,
    );
    let (immutable_log, immutable_log_health) = remote_immutable_log_health(immutable_log_result);
    let telemetry_delivery =
        remote_telemetry_export::export_remote_telemetry(&input.selection.telemetry, &telemetry.events);
    let health = remote_build::RemoteAttemptObservabilityHealth {
        telemetry: remote_build::remote_telemetry_buffer_health(&telemetry),
        exporters: telemetry_delivery,
        trace_context: production_trace_health(input.selection, &transcript),
        immutable_log: immutable_log_health,
        non_claim: remote_build::REMOTE_OBSERVABILITY_NON_CLAIM.to_string(),
    };
    let _health_persistence = remote_build::update_remote_production_observability_health(
        input.coordinator,
        &input.prepared.attempt,
        health.clone(),
    );
    Ok(remote_build::RemoteClientImportedBuild {
        label: input.prepared.plan.label.clone(),
        request_id: input.prepared.request.request_id.clone(),
        imported: output_admission,
        observability: remote_build::RemoteAttemptObservabilityReport {
            events: telemetry.events,
            health,
            immutable_log,
        },
    })
}

fn remote_immutable_log_health(
    result: Result<remote_build::RemoteAttemptLogControlSummary, String>,
) -> (
    Option<remote_build::RemoteAttemptLogControlSummary>,
    remote_telemetry_export::RemoteTelemetryAdapterHealth,
) {
    match result {
        Ok(summary) => (
            Some(summary),
            remote_build::remote_observability_adapter_health(
                remote_telemetry_export::RemoteTelemetryAdapterStatus::Succeeded,
                "remote-observability-log-persisted",
            ),
        ),
        Err(reason) => (
            None,
            remote_build::remote_observability_adapter_health(
                remote_telemetry_export::RemoteTelemetryAdapterStatus::Failed,
                &reason,
            ),
        ),
    }
}

fn production_trace_health(
    selection: &RemoteBuildSelection,
    transcript: &remote_build::RemoteStdioTranscript,
) -> remote_trace_context::RemoteTraceContextHealth {
    if selection.trace_health.status != remote_trace_context::RemoteTraceContextStatus::Accepted {
        return selection.trace_health.clone();
    }
    let Some(server_health) = transcript.trace_context_health.as_ref() else {
        return remote_trace_context::RemoteTraceContextHealth {
            status: remote_trace_context::RemoteTraceContextStatus::Dropped,
            reason_code: "remote-trace-context-ack-missing".to_string(),
            context_digest_blake3: None,
            non_claim: remote_trace_context::REMOTE_TRACE_CONTEXT_NON_CLAIM.to_string(),
        };
    };
    if server_health.status != remote_trace_context::RemoteTraceContextStatus::Accepted
        || server_health.context_digest_blake3 != selection.trace_health.context_digest_blake3
    {
        return remote_trace_context::RemoteTraceContextHealth {
            status: remote_trace_context::RemoteTraceContextStatus::Dropped,
            reason_code: "remote-trace-context-ack-mismatch".to_string(),
            context_digest_blake3: None,
            non_claim: remote_trace_context::REMOTE_TRACE_CONTEXT_NON_CLAIM.to_string(),
        };
    }
    debug_assert_eq!(server_health.context_digest_blake3, selection.trace_health.context_digest_blake3);
    debug_assert_eq!(server_health.status, remote_trace_context::RemoteTraceContextStatus::Accepted);
    server_health.clone()
}

fn print_remote_client_build_report(
    report: &remote_build::RemoteClientBuildReport,
    file: &Path,
    output_dir: &Path,
    state_dir: &Path,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    match output_mode {
        BuildOutputMode::Json => {
            let json_document = remote_client_build_json_report(report, file, output_dir, state_dir)?;
            let rendered = serde_json::to_string_pretty(&json_document)
                .map_err(|err| RunError::Internal(format!("serializing remote build report: {err}")))?;
            println!("{rendered}");
        }
        BuildOutputMode::Human => {
            for build in &report.imported {
                for output in &build.imported.outputs {
                    println!("{}", output.logical_path);
                }
            }
        }
    }
    Ok(())
}

fn remote_client_build_json_report(
    report: &remote_build::RemoteClientBuildReport,
    file: &Path,
    output_dir: &Path,
    state_dir: &Path,
) -> Result<serde_json::Value, RunError> {
    debug_assert!(report.store_prefix.starts_with('/'));
    debug_assert!(!file.as_os_str().is_empty());
    let outcomes = report
        .imported
        .iter()
        .map(|build| remote_client_build_outcome_json(report, build, output_dir))
        .collect::<Result<Vec<_>, _>>()?;
    let succeeded_total = remote_report_count(outcomes.len())?;
    let telemetry_take_count = usize::try_from(crunch_build::distributed::MAX_REMOTE_TELEMETRY_EVENT_CAPACITY)
        .map_err(|_| RunError::Internal("remote telemetry event capacity does not fit in usize".to_string()))?;
    let telemetry_events = report
        .imported
        .iter()
        .flat_map(|build| build.observability.events.iter().cloned())
        .take(telemetry_take_count)
        .collect::<Vec<_>>();
    let remote_observability = report
        .imported
        .iter()
        .map(|build| {
            serde_json::json!({
                "health": &build.observability.health,
                "immutable_log": &build.observability.immutable_log,
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema": BUILD_JSON_REPORT_SCHEMA,
        "file": file.display().to_string(),
        "output_dir": output_dir.display().to_string(),
        "state_dir": state_dir.display().to_string(),
        "store_dir": &report.store_prefix,
        "scheduler_policy": standard_scheduling_policy(),
        "hermeticity_mode": REMOTE_BUILD_HERMETICITY_MODE,
        "hermeticity_audit_events": [],
        "build_environment_reports": [],
        "network_policy_reports": [],
        "action_result_reports": [],
        "native_dynamic_plans": [],
        "scheduler_priority_decisions": &report.priority_decisions,
        "remote_telemetry_events": telemetry_events,
        "remote_observability": remote_observability,
        "frontend_artifact_attestations": [],
        "ast_grep_structural_evidence": [],
        "ast_grep_structural_evidence_diagnostics": [],
        "cargo_build_evidence": [],
        "cargo_build_evidence_diagnostics": [],
        "diagnostic_persistence_failures": [],
        "counts": {
            "succeeded_total": succeeded_total,
            "built_total": 0,
            "cached_total": succeeded_total,
            "failed_total": 0,
        },
        "outcomes": outcomes,
        "failed": [],
        "fod_mismatches": [],
    }))
}

fn remote_client_build_outcome_json(
    report: &remote_build::RemoteClientBuildReport,
    build: &remote_build::RemoteClientImportedBuild,
    output_dir: &Path,
) -> Result<serde_json::Value, RunError> {
    let outputs = build
        .imported
        .outputs
        .iter()
        .map(|output| remote_client_build_output_json(report, &build.imported.transfer, output, output_dir))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(serde_json::json!({
        "drv_key": &build.request_id,
        "label": &build.label,
        "cached": true,
        "log_file": null,
        "outputs": outputs,
    }))
}

fn remote_client_build_output_json(
    report: &remote_build::RemoteClientBuildReport,
    transfer: &remote_build::RemoteTransferReport,
    output: &remote_build::RemoteImportedOutput,
    output_dir: &Path,
) -> Result<serde_json::Value, RunError> {
    Ok(serde_json::json!({
        "name": &output.name,
        "path": remote_client_output_path(RemoteClientOutputPathRequest {
            logical_path: &output.logical_path,
            store_prefix: &report.store_prefix,
            output_dir,
        })?,
        "artifact_attestation": {
            "logical_path": &output.logical_path,
            "path": &output.artifact_attestation_path,
        },
        "substitution": remote_client_substitution_json(transfer),
    }))
}

fn remote_client_substitution_json(transfer: &remote_build::RemoteTransferReport) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    fields.insert("mode".to_string(), serde_json::json!(&transfer.mode_label));
    fields.insert("transferred_bytes".to_string(), serde_json::json!(transfer.transferred_bytes));
    fields.insert("reused_bytes".to_string(), serde_json::json!(transfer.reused_bytes));
    if let Some(reason) = &transfer.fallback_reason {
        fields.insert("fallback_reason".to_string(), serde_json::json!(reason));
    }
    serde_json::Value::Object(fields)
}

struct RemoteClientOutputPathRequest<'a> {
    logical_path: &'a str,
    store_prefix: &'a str,
    output_dir: &'a Path,
}

fn remote_client_output_path(request: RemoteClientOutputPathRequest<'_>) -> Result<String, RunError> {
    let suffix = request
        .logical_path
        .strip_prefix(request.store_prefix)
        .ok_or_else(|| RunError::Internal("remote output path has wrong store prefix".to_string()))?;
    let suffix = suffix.strip_prefix('/').unwrap_or(suffix);
    Ok(request.output_dir.join(suffix).display().to_string())
}

fn remote_report_count(count: usize) -> Result<u32, RunError> {
    u32::try_from(count).map_err(|_| RunError::Internal("remote build report count overflow".to_string()))
}

struct OfflineSourcePreflightRequest<'a> {
    enabled: bool,
    file: &'a Path,
    import_entries: &'a [OsString],
    state_dir: &'a Path,
    store_prefix: &'a str,
    output_mode: BuildOutputMode,
}

fn run_offline_source_preflight_if_requested(
    request: OfflineSourcePreflightRequest<'_>,
) -> Result<Option<source_bundle::SourceOfflinePreflightReport>, RunError> {
    if !request.enabled {
        return Ok(None);
    }
    let preflight = source_bundle::offline_preflight_for_file(
        request.file,
        request.import_entries,
        request.state_dir,
        request.store_prefix,
    )?;
    if source_bundle::source_offline_preflight_is_ready(&preflight) {
        return Ok(Some(preflight));
    }
    source_bundle::print_offline_preflight_report(&preflight, request.output_mode == BuildOutputMode::Json)?;
    Err(RunError::Reported(1))
}

struct OfflineExprPreflightRequest<'a> {
    enabled: bool,
    expr: &'a str,
    import_entries: &'a [OsString],
    state_dir: &'a Path,
    store_prefix: &'a str,
    output_mode: BuildOutputMode,
}

fn run_offline_source_preflight_for_expr_if_requested(
    request: OfflineExprPreflightRequest<'_>,
) -> Result<Option<source_bundle::SourceOfflinePreflightReport>, RunError> {
    if !request.enabled {
        return Ok(None);
    }
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), request.expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;
    run_offline_source_preflight_if_requested(OfflineSourcePreflightRequest {
        enabled: true,
        file: tmp.path(),
        import_entries: request.import_entries,
        state_dir: request.state_dir,
        store_prefix: request.store_prefix,
        output_mode: request.output_mode,
    })
}

fn source_fetch_override_plan_for_expr_if_requested(
    enabled: bool,
    expr: &str,
    import_paths: &[std::ffi::OsString],
    state_dir: &Path,
    store_prefix: &str,
) -> Result<Option<source_bundle::SourceFetchOverridePlan>, RunError> {
    if !enabled {
        return Ok(None);
    }
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;
    source_bundle::source_fetch_override_plan_for_file(tmp.path(), import_paths, state_dir, store_prefix).map(Some)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRootManifestCheck {
    manifest: bootstrap_source_root::SourceRootManifest,
    manifest_bytes: Vec<u8>,
    manifest_digest: String,
    expected_output_role_count: u32,
}

fn validate_source_root_manifest_file(manifest_path: &Path) -> Result<SourceRootManifestCheck, RunError> {
    debug_assert!(!manifest_path.as_os_str().is_empty());
    let manifest_bytes = fs::read(manifest_path).map_err(|err| {
        RunError::Internal(format!("reading source-root manifest {}: {err}", manifest_path.display()))
    })?;
    if manifest_bytes.is_empty() {
        return Err(RunError::Internal(format!("source-root manifest {} is empty", manifest_path.display())));
    }
    let manifest =
        bootstrap_source_root::parse_source_root_manifest_bytes(&manifest_bytes).map_err(RunError::Internal)?;
    let validation = bootstrap_source_root::validate_source_root_manifest(&manifest)
        .map_err(|errors| RunError::Internal(bootstrap_source_root::format_diagnostics(&errors)))?;
    let expected_output_role_count = u32::try_from(validation.expected_output_roles.len()).map_err(|_| {
        RunError::Internal("source-root manifest expected output role count overflowed u32".to_string())
    })?;
    let manifest_digest = bootstrap_source_root::source_root_manifest_digest(&manifest_bytes);
    Ok(SourceRootManifestCheck {
        manifest,
        manifest_bytes,
        manifest_digest,
        expected_output_role_count,
    })
}

fn bootstrap_source_root_provider(
    output: &Path,
    manifest_path: &Path,
    store_dir: &Path,
    verbose: bool,
) -> Result<(), RunError> {
    debug_assert!(!output.as_os_str().is_empty());
    debug_assert!(!manifest_path.as_os_str().is_empty());
    if !store_dir.exists() {
        return Err(RunError::Internal(format!(
            "store directory {} does not exist.\nCreate it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            store_dir.display()
        )));
    }
    let checked = validate_source_root_manifest_file(manifest_path)?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-source-root-")
        .tempdir_in(store_dir)
        .map_err(|err| RunError::Internal(format!("creating source-root scratch in {}: {err}", store_dir.display())))?;
    let provisional_output = scratch.path().join("provider-output");
    let materialized = source_root_provider::materialize_source_root_provider(
        &checked.manifest,
        &checked.manifest_bytes,
        &provisional_output,
        scratch.path(),
        verbose,
    )
    .map_err(|err| RunError::Build(format!("source-root provider materialization failed: {err}")))?;
    if materialized.manifest_digest != checked.manifest_digest {
        return Err(RunError::Internal(
            "source-root provider manifest digest drifted during materialization".to_string(),
        ));
    }
    let store_name = source_root_provider_store_name(&materialized.output_digest)?;
    let final_output = store_dir.join(store_name);
    if final_output.exists() {
        if !final_output.is_dir() {
            return Err(RunError::Build(format!(
                "source-root provider output path exists but is not a directory: {}",
                final_output.display()
            )));
        }
        fs::remove_dir_all(&materialized.output_path).map_err(|err| {
            RunError::Internal(format!(
                "removing duplicate source-root output {}: {err}",
                materialized.output_path.display()
            ))
        })?;
    } else {
        fs::rename(&materialized.output_path, &final_output).map_err(|err| {
            RunError::Internal(format!(
                "moving source-root provider {} to {}: {err}",
                materialized.output_path.display(),
                final_output.display()
            ))
        })?;
    }

    let logical_path = final_output.display().to_string();
    let seed_ncl =
        bootstrap::generate_source_root_seed_ncl(&logical_path, &checked.manifest_digest, &materialized.output_digest);
    fs::write(output, seed_ncl).map_err(|err| RunError::Internal(format!("writing {}: {err}", output.display())))?;
    eprintln!("Materialized source-root provider {}", final_output.display());
    eprintln!("  manifest_digest: {}", checked.manifest_digest);
    eprintln!("  output_digest: {}", materialized.output_digest);
    eprintln!("  expected_output_roles: {}", checked.expected_output_role_count);
    eprintln!("  dependency_trace_urls: {}", materialized.dependency_trace.urls.len());
    eprintln!("Wrote {}", output.display());
    Ok(())
}

const SOURCE_ROOT_STORE_DIGEST_BYTES: usize = 20;
#[cfg(test)]
const NIX_BASE32_STORE_HASH_CHARS: usize = 32;
#[cfg(test)]
const FIRST_SOURCE_ROOT_PATCH_ORDER: u32 = 1;
const HEX_CHARS_PER_BYTE: usize = 2;
const HEX_RADIX: u32 = 16;

fn source_root_provider_store_name(output_digest_hex: &str) -> Result<String, RunError> {
    let digest_bytes = decode_blake3_hex(output_digest_hex)?;
    let hash_part = nix_compat::nixbase32::encode(&digest_bytes[..SOURCE_ROOT_STORE_DIGEST_BYTES]);
    Ok(format!("{hash_part}-{}", bootstrap_source_root::PROVIDER_NAME))
}

fn decode_blake3_hex(value: &str) -> Result<[u8; blake3::OUT_LEN], RunError> {
    let expected_hex_chars = blake3::OUT_LEN
        .checked_mul(HEX_CHARS_PER_BYTE)
        .ok_or_else(|| RunError::Internal("BLAKE3 hex length overflowed usize".to_string()))?;
    if value.len() != expected_hex_chars {
        return Err(RunError::Internal(format!(
            "source-root provider output digest must be {expected_hex_chars} lowercase hex chars"
        )));
    }
    debug_assert_eq!(value.len(), expected_hex_chars);
    let mut bytes = [0u8; blake3::OUT_LEN];
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let start = idx
            .checked_mul(HEX_CHARS_PER_BYTE)
            .ok_or_else(|| RunError::Internal("BLAKE3 hex byte offset overflowed usize".to_string()))?;
        let end = start
            .checked_add(HEX_CHARS_PER_BYTE)
            .ok_or_else(|| RunError::Internal("BLAKE3 hex byte end overflowed usize".to_string()))?;
        let pair = &value[start..end];
        if !pair.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
            return Err(RunError::Internal("source-root provider output digest must be lowercase hex".to_string()));
        }
        *byte = u8::from_str_radix(pair, HEX_RADIX).map_err(|err| {
            RunError::Internal(format!("parsing source-root provider output digest byte {idx}: {err}"))
        })?;
    }
    debug_assert!(value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
    Ok(bytes)
}

struct LoadedStage0InventoryPolicy {
    policy: protected_exec::ProtectedExecPolicy,
    digest_blake3: String,
}

fn load_stage0_inventory_policy(path: &Path) -> Result<LoadedStage0InventoryPolicy, RunError> {
    let nickel_search_dirs: Vec<OsString> = Vec::new();
    let inventory: protected_exec::Stage0Inventory =
        crunch_eval::evaluate_and_deserialize(path, &nickel_search_dirs)
            .map_err(|e| RunError::Eval(format!("loading stage0 inventory {}: {e}", path.display())))?;
    let policy = protected_exec::ProtectedExecPolicy::from_inventory(inventory)
        .map_err(|e| RunError::Internal(format!("validating stage0 inventory {}: {e}", path.display())))?;
    let digest_blake3 = policy.inventory_digest_blake3().to_string();
    Ok(LoadedStage0InventoryPolicy { policy, digest_blake3 })
}

fn run_bootstrap_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Bootstrap {
            action,
            output,
            fetch,
            source_root,
            stagex_lineage,
            stagex_transition_root,
            offline_source_preflight,
            packages,
        } => {
            if let Some(action) = action {
                return run_bootstrap_action(ctx, action);
            }
            run_bootstrap_command(BootstrapCommandRequest {
                ctx,
                output,
                fetch: *fetch,
                source_root: source_root.as_deref(),
                stagex_lineage: stagex_lineage.as_deref(),
                stagex_transition_root: stagex_transition_root.as_deref(),
                offline_source_preflight: *offline_source_preflight,
                packages,
            })
        }
        _ => Err(RunError::Internal("bootstrap helper requires a bootstrap command".to_string())),
    }
}

fn run_bootstrap_action(ctx: &RunContext, action: &BootstrapAction) -> Result<(), RunError> {
    match action {
        BootstrapAction::Capabilities => source_root_capability::cmd_source_root_capabilities(ctx.json),
        BootstrapAction::ParityReport { require } => {
            bootstrap_parity::cmd_bootstrap_parity_report(&current_dir_or_error()?, require, ctx.json)
        }
        BootstrapAction::RustSourceProvider {
            recipe,
            route_plan,
            full_source_admission,
            full_source_host_tools,
            full_source_rust_sources,
            import_dir,
            smoke,
            smoke_evidence_dir,
            output_dir,
            scratch_dir,
        } => cmd_bootstrap_rust_source_provider(RustSourceProviderCommandRequest {
            recipe,
            route_plan: route_plan.as_deref(),
            full_source_admission: full_source_admission.as_deref(),
            full_source_host_tools: full_source_host_tools.as_deref(),
            full_source_rust_sources: full_source_rust_sources.as_deref(),
            import_dir: import_dir.as_deref(),
            output_dir,
            scratch_dir: scratch_dir.as_deref(),
            verbose: ctx.verbose,
            smoke: *smoke,
            smoke_evidence_dir: smoke_evidence_dir.as_deref(),
        }),
        BootstrapAction::FullSourceRustHostTools {
            admission_report,
            make_root,
            make_attestation,
            cmake_root,
            cmake_attestation,
            python_root,
            python_attestation,
            perl_root,
            perl_attestation,
            busybox_root,
            busybox_attestation,
            linux_headers_root,
            linux_headers_attestation,
            output_dir,
        } => {
            let manifest = full_source_rust_binding_shell::materialize_full_source_rust_host_tools(
                full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest {
                    admission_report_path: admission_report,
                    make_root,
                    make_attestation_path: make_attestation,
                    cmake_root,
                    cmake_attestation_path: cmake_attestation,
                    python_root,
                    python_attestation_path: python_attestation,
                    perl_root,
                    perl_attestation_path: perl_attestation,
                    busybox_root,
                    busybox_attestation_path: busybox_attestation,
                    linux_headers_root,
                    linux_headers_attestation_path: linux_headers_attestation,
                    output_dir,
                },
            )
            .map_err(|error| RunError::Build(format!("full-source Rust host-tool evidence: {error}")))?;
            eprintln!("Materialized full-source Rust host-tool manifest {}", manifest.display());
            Ok(())
        }
        BootstrapAction::FullSourceProviderAdmit {
            provider_dir,
            expected_output_blake3,
            source_closure,
            expected_source_closure_blake3,
            report,
            adopt_state,
        } => {
            let admission = full_source_provider::cmd_admit_full_source_provider(
                provider_dir,
                expected_output_blake3,
                source_closure,
                expected_source_closure_blake3,
                report,
                ctx.json,
            )?;
            if *adopt_state {
                let logical_path = full_source_provider::adopt_admitted_full_source_provider(
                    &admission,
                    &ctx.store,
                    &ctx.resolved_state_dir,
                    &ctx.store_prefix,
                )?;
                eprintln!("Adopted admitted provider into local state: {logical_path}");
            }
            Ok(())
        }
        BootstrapAction::NativeToolchainClosure {
            rust_source_provider,
            host_root,
            target_root,
            output,
        } => native_toolchain_closure::cmd_materialize_native_toolchain_closure(
            native_toolchain_closure::NativeToolchainClosureOptions {
                rust_source_provider,
                host_root,
                target_root,
                output,
            },
        ),
        BootstrapAction::Validate {
            target,
            import_paths,
            evidence_dir,
            warmups,
            resume,
            jobs,
            strict_hermetic,
            impure,
        } => bootstrap_validate::cmd_bootstrap_validate(ctx, bootstrap_validate::BootstrapValidateOptions {
            target: target.clone(),
            import_paths: import_paths.clone(),
            evidence_dir: evidence_dir.clone(),
            warmups: warmups.clone(),
            resume: *resume,
            jobs: *jobs,
            strict_hermetic: *strict_hermetic,
            impure: *impure,
        }),
    }
}

struct RustSourceProviderCommandRequest<'a> {
    recipe: &'a Path,
    route_plan: Option<&'a Path>,
    full_source_admission: Option<&'a Path>,
    full_source_host_tools: Option<&'a Path>,
    full_source_rust_sources: Option<&'a Path>,
    import_dir: Option<&'a Path>,
    output_dir: &'a Path,
    scratch_dir: Option<&'a Path>,
    verbose: bool,
    smoke: bool,
    smoke_evidence_dir: Option<&'a Path>,
}

#[derive(Debug)]
enum RustSourceProviderScratch {
    Ephemeral(tempfile::TempDir),
    Persistent(PathBuf),
}

impl RustSourceProviderScratch {
    fn path(&self) -> &Path {
        match self {
            Self::Ephemeral(scratch) => scratch.path(),
            Self::Persistent(path) => path,
        }
    }

    fn preserve(self) -> PathBuf {
        match self {
            Self::Ephemeral(scratch) => preserve_rust_source_provider_scratch(scratch),
            Self::Persistent(path) => path,
        }
    }
}

fn prepare_rust_source_provider_scratch(requested_path: Option<&Path>) -> Result<RustSourceProviderScratch, RunError> {
    if let Some(path) = requested_path {
        fs::create_dir(path).map_err(|error| {
            RunError::Internal(format!("creating persistent Rust provider scratch {}: {error}", path.display()))
        })?;
        let canonical = fs::canonicalize(path).map_err(|error| {
            RunError::Internal(format!("canonicalizing persistent Rust provider scratch {}: {error}", path.display()))
        })?;
        return Ok(RustSourceProviderScratch::Persistent(canonical));
    }
    let scratch = tempfile::Builder::new()
        .prefix("mantle-rust-source-provider-")
        .tempdir()
        .map_err(|error| RunError::Internal(format!("creating Rust provider scratch: {error}")))?;
    Ok(RustSourceProviderScratch::Ephemeral(scratch))
}

fn cmd_bootstrap_rust_source_provider(request: RustSourceProviderCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(!request.recipe.as_os_str().is_empty());
    debug_assert!(!request.output_dir.as_os_str().is_empty());
    if let Some(import_dir) = request.import_dir {
        return cmd_import_rust_source_provider(
            import_dir,
            request.output_dir,
            request.smoke,
            request.smoke_evidence_dir,
        );
    }
    let scratch = prepare_rust_source_provider_scratch(request.scratch_dir)?;
    match materialize_requested_rust_source_provider(&request, scratch.path()) {
        Ok(materialized) => {
            eprintln!("Materialized Rust source provider {}", materialized.output_path.display());
            eprintln!("  recipe_digest_blake3: {}", materialized.recipe_digest_blake3);
            eprintln!("  metadata_path: {}", materialized.metadata_path.display());
            eprintln!("  metadata_digest_blake3: {}", materialized.metadata_digest_blake3);
            if matches!(&scratch, RustSourceProviderScratch::Persistent(_)) {
                eprintln!("  retained_scratch: {}", scratch.path().display());
            }
            if request.smoke {
                cmd_smoke_rust_source_provider(&materialized.output_path, request.smoke_evidence_dir)?;
            }
            Ok(())
        }
        Err(err) => {
            let preserved_scratch = scratch.preserve();
            Err(RunError::Build(format!(
                "Rust source provider materialization failed closed: {err} preserved_scratch={}",
                preserved_scratch.display()
            )))
        }
    }
}

fn materialize_requested_rust_source_provider(
    request: &RustSourceProviderCommandRequest<'_>,
    scratch_dir: &Path,
) -> Result<rust_source_provider::RustSourceProviderMaterialization, rust_source_provider::RustSourceProviderError> {
    if request.full_source_admission.is_none() && request.full_source_host_tools.is_some() {
        return Err(rust_source_provider::RustSourceProviderError::Validate(
            "--full-source-host-tools requires --full-source-admission".to_string(),
        ));
    }
    if request.full_source_admission.is_none() && request.full_source_rust_sources.is_some() {
        return Err(rust_source_provider::RustSourceProviderError::Validate(
            "--full-source-rust-sources requires --full-source-admission".to_string(),
        ));
    }
    if let Some(admission_report_path) = request.full_source_admission {
        let host_tool_manifest_path = request.full_source_host_tools.ok_or_else(|| {
            rust_source_provider::RustSourceProviderError::Validate(
                "full-source native admission requires --full-source-host-tools".to_string(),
            )
        })?;
        let rust_source_archive_dir = request.full_source_rust_sources.ok_or_else(|| {
            rust_source_provider::RustSourceProviderError::Validate(
                "full-source native admission requires --full-source-rust-sources".to_string(),
            )
        })?;
        return rust_source_provider::materialize_full_source_bound_rust_provider_with_route_plan(
            request.recipe,
            request.route_plan,
            admission_report_path,
            host_tool_manifest_path,
            rust_source_archive_dir,
            request.output_dir,
            scratch_dir,
            request.verbose,
        );
    }
    rust_source_provider::materialize_rust_source_provider_with_route_plan(
        request.recipe,
        request.route_plan,
        request.output_dir,
        scratch_dir,
        request.verbose,
    )
}

fn preserve_rust_source_provider_scratch(scratch: tempfile::TempDir) -> PathBuf {
    scratch.keep()
}

fn cmd_import_rust_source_provider(
    import_dir: &Path,
    output_dir: &Path,
    smoke: bool,
    smoke_evidence_dir: Option<&Path>,
) -> Result<(), RunError> {
    let provider_copy = rust_source_provider::import_rust_source_provider(import_dir, output_dir)
        .map_err(|err| RunError::Build(format!("Rust source provider import failed closed: {err}")))?;
    eprintln!("Imported Rust source provider {}", provider_copy.output_path.display());
    eprintln!("  input: {}", provider_copy.input_path.display());
    eprintln!("  metadata_path: {}", provider_copy.validation.metadata_path.display());
    eprintln!("  metadata_digest_blake3: {}", provider_copy.validation.metadata_digest_blake3);
    eprintln!("  policy_digest_blake3: {}", provider_copy.validation.validation.policy_digest_blake3);
    if smoke {
        cmd_smoke_rust_source_provider(&provider_copy.output_path, smoke_evidence_dir)?;
    }
    Ok(())
}

fn cmd_smoke_rust_source_provider(provider_dir: &Path, evidence_dir: Option<&Path>) -> Result<(), RunError> {
    debug_assert!(!provider_dir.as_os_str().is_empty());
    let validation = rust_source_provider::validate_materialized_rust_source_provider(provider_dir)
        .map_err(|err| RunError::Build(format!("Rust source provider validation failed closed: {err}")))?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-rust-provider-smoke-")
        .tempdir()
        .map_err(|err| RunError::Internal(format!("creating Rust provider smoke scratch: {err}")))?;
    let smoke = rust_source_provider::smoke_rust_source_provider(provider_dir, scratch.path())
        .map_err(|err| RunError::Build(format!("Rust source provider smoke failed closed: {err}")))?;
    let evidence = match evidence_dir {
        Some(evidence_dir) => Some(
            rust_source_provider::persist_rust_source_provider_smoke_evidence(evidence_dir, &smoke, &validation)
                .map_err(|err| RunError::Build(format!("Rust source provider smoke evidence failed closed: {err}")))?,
        ),
        None => None,
    };
    eprintln!("Smoked Rust source provider {}", smoke.provider_path.display());
    eprintln!("  rustc_path: {}", smoke.rustc_path.display());
    eprintln!("  target_triple: {}", smoke.target_triple);
    eprintln!("  smoke_source: {}", smoke.source_path.display());
    eprintln!("  smoke_output: {}", smoke.output_path.display());
    eprintln!("  smoke_output_digest_blake3: {}", smoke.output_digest_blake3);
    if !smoke.stdout.is_empty() {
        eprintln!("  smoke_stdout: {}", smoke.stdout.trim_end());
    }
    if !smoke.stderr.is_empty() {
        eprintln!("  smoke_stderr: {}", smoke.stderr.trim_end());
    }
    if let Some(evidence) = evidence {
        eprintln!("  smoke_evidence_dir: {}", evidence.evidence_dir.display());
        eprintln!("  smoke_evidence_summary: {}", evidence.summary_path.display());
        eprintln!("  smoke_evidence_metadata_digest_blake3: {}", evidence.metadata_digest_blake3);
        eprintln!("  smoke_evidence_policy_digest_blake3: {}", evidence.policy_digest_blake3);
    }
    Ok(())
}

struct BootstrapCommandRequest<'a> {
    ctx: &'a RunContext,
    output: &'a Path,
    fetch: bool,
    source_root: Option<&'a Path>,
    stagex_lineage: Option<&'a Path>,
    stagex_transition_root: Option<&'a Path>,
    offline_source_preflight: bool,
    packages: &'a [String],
}

fn run_bootstrap_command(request: BootstrapCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(!request.output.as_os_str().is_empty());
    debug_assert!(!request.ctx.store.as_os_str().is_empty());
    let mode =
        bootstrap_source_root::select_bootstrap_provider(request.fetch, request.source_root, request.stagex_lineage)
            .map_err(|err| RunError::Internal(err.to_string()))?;
    match mode {
        bootstrap_source_root::BootstrapProviderMode::LegacyFetch => cmd_bootstrap_fetch(BootstrapFetchRequest {
            output: request.output,
            store_dir: &request.ctx.store,
            verbose: request.ctx.verbose,
            offline_source_preflight: request.offline_source_preflight,
        }),
        bootstrap_source_root::BootstrapProviderMode::SourceRoot => {
            let manifest_path = request
                .source_root
                .ok_or_else(|| RunError::Internal("source-root mode requires a manifest path".to_string()))?;
            bootstrap_source_root_provider(request.output, manifest_path, &request.ctx.store, request.ctx.verbose)
        }
        bootstrap_source_root::BootstrapProviderMode::StagexLineage => {
            let manifest_path = request
                .stagex_lineage
                .ok_or_else(|| RunError::Internal("stagex-lineage mode requires a manifest path".to_string()))?;
            let transition_root = request.stagex_transition_root.ok_or_else(|| {
                RunError::Internal("stagex-lineage provider publication requires --stagex-transition-root".to_string())
            })?;
            cmd_bootstrap_stagex_lineage(request.output, manifest_path, transition_root)
        }
        bootstrap_source_root::BootstrapProviderMode::FullSource => Err(RunError::Internal(
            "full-source mode is selected by bootstrap/seed.ncl, not the compatibility bootstrap command".to_string(),
        )),
        bootstrap_source_root::BootstrapProviderMode::NixPackages => cmd_bootstrap(request.output, request.packages),
    }
}

fn cmd_bootstrap_stagex_lineage(output: &Path, manifest_path: &Path, transition_root: &Path) -> Result<(), RunError> {
    let report = stagex_provider::materialize_stagex_provider(stagex_provider::StagexProviderRequest {
        lineage_manifest_path: manifest_path,
        transition_root,
        output_path: output,
    })
    .map_err(|error| RunError::Internal(error.to_string()))?;
    eprintln!("StageX intermediate provider published:");
    eprintln!("  output: {}", report.output_path.display());
    eprintln!("  receipt: {}", report.receipt_path.display());
    eprintln!("  normalized_provider_digest_blake3: {}", report.normalized_provider_digest_blake3);
    eprintln!("  output_digest_blake3: {}", report.output_digest_blake3);
    eprintln!("  final_bundle_digest_blake3: {}", report.final_bundle_digest_blake3);
    eprintln!("  provider_validation_audit_digest_blake3: {}", report.provider_validation_audit_digest_blake3);
    eprintln!("  provider_validation_report_digest_blake3: {}", report.provider_validation_report_digest_blake3);
    Ok(())
}

fn run_project_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    let cwd = current_dir_or_error()?;
    match command {
        Command::Init => project_cmd::cmd_init(&cwd),
        Command::Check { probes, trust } => {
            let output = if ctx.json {
                project_cmd::ProjectCheckOutput::Json
            } else {
                project_cmd::ProjectCheckOutput::Human
            };
            project_cmd::cmd_check(&cwd, output, *probes, *trust)
        }
        Command::Show => project_cmd::cmd_show(&cwd),
        Command::Refresh { no_network, names } => project_cmd::cmd_refresh(&cwd, names, *no_network),
        Command::ListStale { no_network } => project_cmd::cmd_list_stale(&cwd, *no_network),
        Command::Upgrade => project_cmd::cmd_upgrade(&cwd),
        _ => Err(RunError::Internal("project helper requires a project command".to_string())),
    }
}

fn run_release_command(ctx: &RunContext, action: ReleaseAction) -> Result<(), RunError> {
    release_cmd::cmd_release(action, &current_dir_or_error()?, &ctx.resolved_state_dir, ctx.json)
}

fn run_attest_command(ctx: &RunContext, action: AttestAction) -> Result<(), RunError> {
    attest_cmd::cmd_attest(
        action,
        &current_dir_or_error()?,
        &ctx.store,
        &ctx.resolved_state_dir,
        &ctx.store_prefix,
        ctx.json,
    )
}

struct RustPlanCaptureRequest<'a> {
    root: PathBuf,
    cargo: &'a Path,
    rustc: &'a Path,
    compiler_policy_mode: &'a str,
    compiler_policy_provider_manifest: Option<&'a Path>,
    compiler_policy_provider_manifest_digest: Option<&'a str>,
    targets: &'a [String],
    profile: &'a str,
    features: &'a [String],
    all_features: bool,
    no_default_features: bool,
    no_cargo_oracle: bool,
    c_compiler_route_json: Option<&'a str>,
    deterministic_release_paths: bool,
    execution_output_root: Option<&'a Path>,
}

struct CapturedRustPlan {
    receipt: rust_plan::RustPlanReceipt,
    compiler_policy: rust_plan::RustCompilerPolicySelection,
    c_compiler_route: Option<source_toolchain_closure::ReceiptBoundCCompilerRoute>,
}

fn capture_rust_plan_command(request: RustPlanCaptureRequest<'_>) -> Result<CapturedRustPlan, RunError> {
    debug_assert!(!request.root.as_os_str().is_empty());
    debug_assert!(!request.cargo.as_os_str().is_empty());
    debug_assert!(!request.rustc.as_os_str().is_empty());
    let c_compiler_route = rust_plan::receipt_bound_c_compiler_route_from_cli_json(request.c_compiler_route_json)?;
    let path_remaps = if request.deterministic_release_paths {
        rust_plan::deterministic_release_path_remaps_with_c_compiler(
            &request.root,
            request.execution_output_root,
            c_compiler_route.as_ref(),
        )
    } else {
        Vec::new()
    };
    let compiler_policy = rust_plan::rust_compiler_policy_selection(
        request.compiler_policy_mode,
        request.compiler_policy_provider_manifest.map(Path::to_path_buf),
        request.compiler_policy_provider_manifest_digest.map(str::to_string),
    )?;
    let options = rust_plan::RustPlanOptions {
        root: request.root,
        cargo: request.cargo.to_path_buf(),
        rustc: request.rustc.to_path_buf(),
        targets: request.targets.to_vec(),
        profile: request.profile.to_string(),
        features: request.features.to_vec(),
        all_features: request.all_features,
        no_default_features: request.no_default_features,
        no_cargo_oracle: request.no_cargo_oracle,
        deterministic_release_paths: request.deterministic_release_paths,
        path_remaps,
    };
    let receipt = rust_plan::capture_rust_plan(&options)?;
    Ok(CapturedRustPlan {
        receipt,
        compiler_policy,
        c_compiler_route,
    })
}

fn run_rust_plan_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    debug_assert!(ctx.store_prefix.starts_with('/'));
    debug_assert!(!ctx.resolved_state_dir.as_os_str().is_empty());
    let Command::RustPlan {
        root,
        cargo,
        rustc,
        compiler_policy_mode,
        compiler_policy_provider_manifest,
        compiler_policy_provider_manifest_digest,
        targets,
        profile,
        features,
        all_features,
        no_default_features,
        no_cargo_oracle,
        source_built_c_compiler_route_json,
        deterministic_release_paths,
        execute_first_supported_unit,
        execute_first_dependency_chain,
        execute_target_topology,
        execute_host_artifact_topology,
        execute_topology,
        execute_dev_dependency_test_topology,
        execute_workspace_dependency_topology,
        execute_patch_source_topology,
        execution_output_root,
    } = command
    else {
        return Err(RunError::Internal("run_rust_plan_command called with non-RustPlan command".to_string()));
    };
    let root = root.clone().unwrap_or(current_dir_or_error()?);
    let captured = capture_rust_plan_command(RustPlanCaptureRequest {
        root,
        cargo,
        rustc,
        compiler_policy_mode,
        compiler_policy_provider_manifest: compiler_policy_provider_manifest.as_deref(),
        compiler_policy_provider_manifest_digest: compiler_policy_provider_manifest_digest.as_deref(),
        targets,
        profile,
        features,
        all_features: *all_features,
        no_default_features: *no_default_features,
        no_cargo_oracle: *no_cargo_oracle,
        c_compiler_route_json: source_built_c_compiler_route_json.as_deref(),
        deterministic_release_paths: *deterministic_release_paths,
        execution_output_root: execution_output_root.as_deref(),
    })?;
    rust_plan::set_receipt_bound_c_compiler_route_override(captured.c_compiler_route)?;
    let execution_mode = select_rust_plan_execution_mode(RustPlanExecutionSelection {
        first_supported: *execute_first_supported_unit,
        first_dependency_chain: *execute_first_dependency_chain,
        target_topology: *execute_target_topology,
        host_artifact_topology: *execute_host_artifact_topology,
        topology: *execute_topology,
        dev_dependency_test_topology: *execute_dev_dependency_test_topology,
        workspace_dependency_topology: *execute_workspace_dependency_topology,
        patch_source_topology: *execute_patch_source_topology,
    });
    execute_rust_plan_receipt(RustPlanExecutionRequest {
        receipt: captured.receipt,
        rustc,
        compiler_policy: captured.compiler_policy,
        output_root: execution_output_root.as_deref(),
        mode: execution_mode,
        json: ctx.json,
    })
}

#[derive(Debug, Clone, Copy)]
enum RustPlanExecutionMode {
    FirstSupported,
    FirstDependencyChain,
    TargetTopology,
    HostArtifactTopology,
    Topology,
    DevDependencyTestTopology,
    WorkspaceDependencyTopology,
    PatchSourceTopology,
    PrintOnly,
}

#[derive(Debug, Clone, Copy)]
struct RustPlanExecutionSelection {
    first_supported: bool,
    first_dependency_chain: bool,
    target_topology: bool,
    host_artifact_topology: bool,
    topology: bool,
    dev_dependency_test_topology: bool,
    workspace_dependency_topology: bool,
    patch_source_topology: bool,
}

fn select_rust_plan_execution_mode(selection: RustPlanExecutionSelection) -> RustPlanExecutionMode {
    match selection {
        selected if selected.first_supported => RustPlanExecutionMode::FirstSupported,
        selected if selected.first_dependency_chain => RustPlanExecutionMode::FirstDependencyChain,
        selected if selected.target_topology => RustPlanExecutionMode::TargetTopology,
        selected if selected.host_artifact_topology => RustPlanExecutionMode::HostArtifactTopology,
        selected if selected.topology => RustPlanExecutionMode::Topology,
        selected if selected.dev_dependency_test_topology => RustPlanExecutionMode::DevDependencyTestTopology,
        selected if selected.workspace_dependency_topology => RustPlanExecutionMode::WorkspaceDependencyTopology,
        selected if selected.patch_source_topology => RustPlanExecutionMode::PatchSourceTopology,
        _ => RustPlanExecutionMode::PrintOnly,
    }
}

struct RustPlanExecutionRequest<'a> {
    receipt: rust_plan::RustPlanReceipt,
    rustc: &'a Path,
    compiler_policy: rust_plan::RustCompilerPolicySelection,
    output_root: Option<&'a Path>,
    mode: RustPlanExecutionMode,
    json: bool,
}

fn execute_rust_plan_receipt(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    match request.mode {
        RustPlanExecutionMode::FirstSupported => execute_first_supported_rust_plan(request),
        RustPlanExecutionMode::FirstDependencyChain => execute_first_dependency_chain_rust_plan(request),
        RustPlanExecutionMode::TargetTopology => execute_target_topology_rust_plan(request),
        RustPlanExecutionMode::HostArtifactTopology => execute_host_artifact_topology_rust_plan(request),
        RustPlanExecutionMode::Topology => execute_topology_rust_plan(request),
        RustPlanExecutionMode::DevDependencyTestTopology => execute_dev_dependency_test_topology_rust_plan(request),
        RustPlanExecutionMode::WorkspaceDependencyTopology => execute_workspace_dependency_topology_rust_plan(request),
        RustPlanExecutionMode::PatchSourceTopology => execute_patch_source_topology_rust_plan(request),
        RustPlanExecutionMode::PrintOnly => rust_plan::print_rust_plan_receipt(&request.receipt, request.json),
    }
}

fn rust_plan_execution_options(
    request: &RustPlanExecutionRequest<'_>,
    flag: &str,
) -> Result<rust_plan::RustUnitExecutionOptions, RunError> {
    let output_root = request
        .output_root
        .ok_or_else(|| RunError::Internal(format!("{flag} requires --execution-output-root")))?;
    Ok(rust_plan::RustUnitExecutionOptions {
        rustc: request.rustc.to_path_buf(),
        output_root: output_root.to_path_buf(),
        compiler_policy: request.compiler_policy.clone(),
    })
}

fn execute_first_supported_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-first-supported-unit")?;
    let unit_execution =
        rust_plan::execute_first_supported_rust_unit(&request.receipt.unit_derivation_graph, &options)?;
    rust_plan::print_rust_plan_execution_receipt(
        &rust_plan::RustPlanExecutionReceipt {
            rust_plan: request.receipt,
            unit_execution,
        },
        request.json,
    )
}

fn execute_first_dependency_chain_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-first-dependency-chain")?;
    let dependency_chain_execution =
        rust_plan::execute_first_rust_unit_dependency_chain(&request.receipt.unit_derivation_graph, &options)?;
    rust_plan::print_rust_plan_dependency_chain_execution_receipt(
        &rust_plan::RustPlanDependencyChainExecutionReceipt {
            rust_plan: request.receipt,
            dependency_chain_execution,
        },
        request.json,
    )
}

fn execute_target_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-target-topology")?;
    let target_topology_execution =
        rust_plan::execute_rust_target_unit_topology(&request.receipt.unit_derivation_graph, &options)?;
    rust_plan::print_rust_plan_target_topology_execution_receipt(
        &rust_plan::RustPlanTargetTopologyExecutionReceipt {
            rust_plan: request.receipt,
            target_topology_execution,
        },
        request.json,
    )
}

fn execute_host_artifact_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-host-artifact-topology")?;
    let execution = rust_plan::execute_rust_host_artifact_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_host_unit_graph_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_host_artifact_topology_execution_receipt(
        &rust_plan::RustPlanHostArtifactTopologyExecutionReceipt {
            rust_plan: request.receipt,
            host_artifact_topology_execution: execution,
        },
        request.json,
    )
}

fn execute_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-topology")?;
    let topology_execution = rust_plan::execute_rust_unit_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_host_unit_graph_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_topology_execution_receipt(
        &rust_plan::RustPlanTopologyExecutionReceipt {
            rust_plan: request.receipt,
            topology_execution,
        },
        request.json,
    )
}

fn execute_dev_dependency_test_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-dev-dependency-test-topology")?;
    let execution = rust_plan::execute_native_rust_dev_dependency_test_topology(
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_dev_dependency_test_topology_execution_receipt(
        &rust_plan::RustPlanDevDependencyTestTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_rust_dev_dependency_test_topology_execution: execution,
        },
        request.json,
    )
}

fn execute_workspace_dependency_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-workspace-dependency-topology")?;
    let execution = rust_plan::execute_native_registry_workspace_dependency_topology(
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_workspace_dependency_topology_execution_receipt(
        &rust_plan::RustPlanWorkspaceDependencyTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_registry_workspace_dependency_topology_execution: execution,
        },
        request.json,
    )
}

fn execute_patch_source_topology_rust_plan(request: RustPlanExecutionRequest<'_>) -> Result<(), RunError> {
    let options = rust_plan_execution_options(&request, "--execute-patch-source-topology")?;
    let execution = rust_plan::execute_native_registry_patch_source_topology(
        &request.receipt.native_registry_source_planning,
        &request.receipt.native_package_target_planning,
        &request.receipt.unit_derivation_graph,
        &options,
    )?;
    rust_plan::print_rust_plan_patch_source_topology_execution_receipt(
        &rust_plan::RustPlanPatchSourceTopologyExecutionReceipt {
            rust_plan: request.receipt,
            native_registry_patch_source_topology_execution: execution,
        },
        request.json,
    )
}

fn run_shell_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Shell {
            name,
            import_paths,
            jobs,
            no_substitute,
            signing_key,
            trust_unsigned,
            command: cmd_argv,
            run_script,
            with_paths,
            no_hook,
            strict_hooks,
        } => shell_cmd::cmd_shell(
            name.as_deref(),
            import_paths,
            *jobs,
            *no_substitute,
            signing_key.as_deref(),
            *trust_unsigned,
            cmd_argv,
            run_script.as_deref(),
            with_paths,
            *no_hook,
            *strict_hooks,
            &ctx.store,
            &ctx.resolved_state_dir,
            &ctx.store_prefix,
            ctx.verbose,
        ),
        _ => Err(RunError::Internal("shell helper requires a shell command".to_string())),
    }
}

fn run_develop_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Develop {
            name,
            import_paths,
            jobs,
            no_substitute,
            signing_key,
            trust_unsigned,
        } => run_develop_command(DevelopCommandRequest {
            ctx,
            name: name.as_deref(),
            import_paths,
            jobs: *jobs,
            no_substitute: *no_substitute,
            signing_key: signing_key.as_deref(),
            trust_unsigned: *trust_unsigned,
        }),
        _ => Err(RunError::Internal("develop helper requires a develop command".to_string())),
    }
}

struct DevelopCommandRequest<'a> {
    ctx: &'a RunContext,
    name: Option<&'a str>,
    import_paths: &'a [PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&'a Path>,
    trust_unsigned: bool,
}

fn run_develop_command(request: DevelopCommandRequest<'_>) -> Result<(), RunError> {
    shell_cmd::cmd_shell(
        request.name,
        request.import_paths,
        request.jobs,
        request.no_substitute,
        request.signing_key,
        request.trust_unsigned,
        &[],
        None,
        &[],
        false,
        false,
        &request.ctx.store,
        &request.ctx.resolved_state_dir,
        &request.ctx.store_prefix,
        request.ctx.verbose,
    )
}

fn run_run_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Run {
            name,
            import_paths,
            jobs,
            no_substitute,
            signing_key,
            trust_unsigned,
            bin,
            run_args,
        } => run_package_command(PackageCommandRequest {
            ctx,
            name: name.as_deref(),
            import_paths,
            jobs: *jobs,
            no_substitute: *no_substitute,
            signing_key: signing_key.as_deref(),
            trust_unsigned: *trust_unsigned,
            bin: bin.as_deref(),
            run_args,
        }),
        _ => Err(RunError::Internal("run helper requires a run command".to_string())),
    }
}

struct PackageCommandRequest<'a> {
    ctx: &'a RunContext,
    name: Option<&'a str>,
    import_paths: &'a [PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&'a Path>,
    trust_unsigned: bool,
    bin: Option<&'a str>,
    run_args: &'a [String],
}

fn run_package_command(request: PackageCommandRequest<'_>) -> Result<(), RunError> {
    cmd_run(RunCommandRequest {
        name: request.name,
        import_paths: request.import_paths,
        jobs: request.jobs,
        no_substitute: request.no_substitute,
        signing_key: request.signing_key,
        trust_unsigned: request.trust_unsigned,
        bin: request.bin,
        run_args: request.run_args,
        output_dir: &request.ctx.store,
        state_dir: &request.ctx.resolved_state_dir,
        store_prefix: &request.ctx.store_prefix,
        verbose: request.ctx.verbose,
    })
}

fn run_self_build_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::SelfBuild {
            jobs,
            no_substitute,
            no_verify,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
            impure,
            no_host_tools,
            stage0_inventory,
            source_store_path,
            bootstrap_bwrap_path,
            bootstrap_busybox_path,
            offline_source_manifest_blake3,
            offline_source_state_dir,
            source_built_fixed_point,
            source_profile,
            expected_source_profile_blake3,
            expected_stagex_lineage_blake3,
            expected_native_provider_blake3,
            proof_bwrap,
            proof_sandbox_shell,
            proof_elapsed_seconds_max,
            proof_disk_bytes_max,
            proof_exec_events_max,
            proof_source_records_max,
            cargo_free,
            fixed_point,
            out,
            rustc,
            toolchain_closure,
            rust_source_provider,
            targets,
        } => run_self_build_command(SelfBuildCommandRequest {
            ctx,
            jobs: *jobs,
            no_substitute: *no_substitute,
            no_verify: *no_verify,
            signing_key: signing_key.as_deref(),
            trusted_public_keys,
            trust_unsigned: *trust_unsigned,
            hermeticity: HermeticitySelection {
                strict_hermetic: *strict_hermetic,
                impure: *impure,
            },
            no_host_tools: *no_host_tools,
            stage0_inventory: stage0_inventory.as_deref(),
            source_store_path: source_store_path.as_deref(),
            bootstrap_bwrap_path: bootstrap_bwrap_path.as_deref(),
            bootstrap_busybox_path: bootstrap_busybox_path.as_deref(),
            offline_source_manifest_blake3: offline_source_manifest_blake3.as_deref(),
            offline_source_state_dir: offline_source_state_dir.as_deref(),
            source_built_fixed_point: *source_built_fixed_point,
            source_profile: source_profile.as_deref(),
            expected_source_profile_blake3: expected_source_profile_blake3.as_deref(),
            expected_stagex_lineage_blake3: expected_stagex_lineage_blake3.as_deref(),
            expected_native_provider_blake3: expected_native_provider_blake3.as_deref(),
            proof_bwrap: proof_bwrap.as_deref(),
            proof_sandbox_shell: proof_sandbox_shell.as_deref(),
            proof_elapsed_seconds_max: *proof_elapsed_seconds_max,
            proof_disk_bytes_max: *proof_disk_bytes_max,
            proof_exec_events_max: *proof_exec_events_max,
            proof_source_records_max: *proof_source_records_max,
            cargo_free: *cargo_free,
            fixed_point: *fixed_point,
            out: out.as_deref(),
            rustc,
            toolchain_closure: toolchain_closure.as_deref(),
            rust_source_provider: rust_source_provider.as_deref(),
            targets,
        }),
        _ => Err(RunError::Internal("self-build helper requires a self-build command".to_string())),
    }
}

struct SelfBuildCommandRequest<'a> {
    ctx: &'a RunContext,
    jobs: Option<u32>,
    no_substitute: bool,
    no_verify: bool,
    signing_key: Option<&'a Path>,
    trusted_public_keys: &'a [String],
    trust_unsigned: bool,
    hermeticity: HermeticitySelection,
    no_host_tools: bool,
    stage0_inventory: Option<&'a Path>,
    source_store_path: Option<&'a Path>,
    bootstrap_bwrap_path: Option<&'a Path>,
    bootstrap_busybox_path: Option<&'a Path>,
    offline_source_manifest_blake3: Option<&'a str>,
    offline_source_state_dir: Option<&'a Path>,
    source_built_fixed_point: bool,
    source_profile: Option<&'a Path>,
    expected_source_profile_blake3: Option<&'a str>,
    expected_stagex_lineage_blake3: Option<&'a str>,
    expected_native_provider_blake3: Option<&'a str>,
    proof_bwrap: Option<&'a Path>,
    proof_sandbox_shell: Option<&'a Path>,
    proof_elapsed_seconds_max: u64,
    proof_disk_bytes_max: u64,
    proof_exec_events_max: u32,
    proof_source_records_max: u32,
    cargo_free: bool,
    fixed_point: bool,
    out: Option<&'a Path>,
    rustc: &'a Path,
    toolchain_closure: Option<&'a Path>,
    rust_source_provider: Option<&'a Path>,
    targets: &'a [String],
}

fn run_self_build_command(request: SelfBuildCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(request.ctx.store_prefix.starts_with('/'));
    debug_assert!(!request.rustc.as_os_str().is_empty());
    if request.source_built_fixed_point {
        return run_source_built_fixed_point(&request);
    }
    if request.cargo_free {
        return run_cargo_free_self_build(&request);
    }
    run_legacy_self_build(&request)
}

fn run_source_built_fixed_point(request: &SelfBuildCommandRequest<'_>) -> Result<(), RunError> {
    let source_profile = request
        .source_profile
        .ok_or_else(|| RunError::Build("--source-built-fixed-point requires --source-profile".to_string()))?;
    let expected_source_profile_blake3 = request.expected_source_profile_blake3.ok_or_else(|| {
        RunError::Build("--source-built-fixed-point requires --expected-source-profile-blake3".to_string())
    })?;
    let expected_stagex_lineage_blake3 = request.expected_stagex_lineage_blake3.ok_or_else(|| {
        RunError::Build("--source-built-fixed-point requires --expected-stagex-lineage-blake3".to_string())
    })?;
    let expected_native_provider_blake3 = request.expected_native_provider_blake3.ok_or_else(|| {
        RunError::Build("--source-built-fixed-point requires --expected-native-provider-blake3".to_string())
    })?;
    let output_dir = request
        .out
        .ok_or_else(|| RunError::Build("--source-built-fixed-point requires --out".to_string()))?;
    let proof_bwrap = request
        .proof_bwrap
        .ok_or_else(|| RunError::Build("--source-built-fixed-point requires --proof-bwrap".to_string()))?;
    let proof_sandbox_shell = request
        .proof_sandbox_shell
        .ok_or_else(|| RunError::Build("--source-built-fixed-point requires --proof-sandbox-shell".to_string()))?;
    if request.hermeticity.impure {
        return Err(RunError::Build("--source-built-fixed-point rejects --impure".to_string()));
    }
    if request.no_host_tools || request.no_verify || request.fixed_point {
        return Err(RunError::Build(
            "--source-built-fixed-point rejects legacy --no-host-tools, --no-verify, and --fixed-point flags"
                .to_string(),
        ));
    }
    source_built_fixed_point_shell::cmd_source_built_fixed_point(
        source_built_fixed_point_shell::SourceBuiltFixedPointOptions {
            source_profile,
            expected_source_profile_blake3,
            expected_stagex_lineage_blake3,
            expected_native_provider_blake3,
            output_dir,
            bwrap: proof_bwrap,
            sandbox_shell: proof_sandbox_shell,
            jobs: request.jobs.unwrap_or(SOURCE_BUILT_FIXED_POINT_JOBS_DEFAULT),
            elapsed_seconds_max: request.proof_elapsed_seconds_max,
            disk_bytes_max: request.proof_disk_bytes_max,
            protected_exec_events_max: request.proof_exec_events_max,
            source_records_max: request.proof_source_records_max,
            verbose: request.ctx.verbose,
            json: request.ctx.json,
        },
    )
}

fn run_cargo_free_self_build(request: &SelfBuildCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(request.cargo_free);
    debug_assert!(!request.rustc.as_os_str().is_empty());
    validate_cargo_free_legacy_self_build_args(CargoFreeLegacyOptions {
        jobs: request.jobs,
        no_substitute: request.no_substitute,
        no_verify: request.no_verify,
        signing_key: request.signing_key,
        trusted_public_keys: request.trusted_public_keys,
        trust_unsigned: request.trust_unsigned,
        impure: request.hermeticity.impure,
        no_host_tools: request.no_host_tools,
        stage0_inventory: request.stage0_inventory,
        source_store_path: request.source_store_path,
        bootstrap_bwrap_path: request.bootstrap_bwrap_path,
        bootstrap_busybox_path: request.bootstrap_busybox_path,
        offline_source_manifest_blake3: request.offline_source_manifest_blake3,
        offline_source_state_dir: request.offline_source_state_dir,
    })?;
    let out_dir = request.out.ok_or_else(|| RunError::Build("--cargo-free requires --out <dir>".to_string()))?;
    let root = current_dir_or_error()?;
    let options = cargo_free_self_build::CargoFreeSelfBuildOptions {
        root: &root,
        out_dir,
        rustc: request.rustc,
        targets: request.targets,
        toolchain_closure: request.toolchain_closure,
        rust_source_provider: request.rust_source_provider,
        hermeticity_mode: select_hermeticity_mode(request.hermeticity)?,
        json: request.ctx.json,
    };
    if request.fixed_point {
        cargo_free_self_build::cmd_cargo_free_fixed_point_self_build(options)
    } else {
        cargo_free_self_build::cmd_cargo_free_self_build(options)
    }
}

fn run_legacy_self_build(request: &SelfBuildCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(!request.cargo_free);
    debug_assert!(request.ctx.store_prefix.starts_with('/'));
    validate_stage0_inventory_args(request.no_host_tools, request.stage0_inventory)?;
    let loaded_stage0_policy = request.stage0_inventory.map(load_stage0_inventory_policy).transpose()?;
    let parsed_trusted = parse_trusted_keys(request.trusted_public_keys)?;
    let source_override_plan = request
        .offline_source_manifest_blake3
        .map(|manifest_blake3| {
            let source_state_dir = request.offline_source_state_dir.ok_or_else(|| {
                RunError::Build("--offline-source-manifest-blake3 requires --offline-source-state-dir".to_string())
            })?;
            source_bundle::full_proof_source_fetch_override_plan(source_state_dir, manifest_blake3)
        })
        .transpose()?;
    let source_evidence = source_override_plan
        .as_ref()
        .map(|plan| self_build::SelfBuildSourceEvidence::from_override_plan(&plan.report, plan.overrides.len()))
        .transpose()?;
    let source_fetch_overrides = source_override_plan.as_ref().map(|plan| plan.overrides.as_slice()).unwrap_or(&[]);
    self_build::cmd_self_build(
        &request.ctx.store,
        &request.ctx.resolved_state_dir,
        &request.ctx.store_prefix,
        request.ctx.verbose,
        crunch_pipeline::resolve_max_jobs(request.jobs),
        request.no_substitute,
        request.no_verify,
        request.signing_key,
        parsed_trusted.as_deref(),
        request.trust_unsigned,
        select_hermeticity_mode(request.hermeticity)?,
        request.source_store_path,
        loaded_stage0_policy.as_ref().map(|loaded| &loaded.policy),
        loaded_stage0_policy.as_ref().map(|loaded| loaded.digest_blake3.clone()),
        request.bootstrap_bwrap_path,
        request.bootstrap_busybox_path,
        bootstrap_source_root::BootstrapProviderMode::FullSource,
        source_fetch_overrides,
        source_evidence,
    )
    .map(|_report| ())
}

struct CargoFreeLegacyOptions<'a> {
    jobs: Option<u32>,
    no_substitute: bool,
    no_verify: bool,
    signing_key: Option<&'a Path>,
    trusted_public_keys: &'a [String],
    trust_unsigned: bool,
    impure: bool,
    no_host_tools: bool,
    stage0_inventory: Option<&'a Path>,
    source_store_path: Option<&'a Path>,
    bootstrap_bwrap_path: Option<&'a Path>,
    bootstrap_busybox_path: Option<&'a Path>,
    offline_source_manifest_blake3: Option<&'a str>,
    offline_source_state_dir: Option<&'a Path>,
}

fn validate_cargo_free_legacy_self_build_args(options: CargoFreeLegacyOptions<'_>) -> Result<(), RunError> {
    let has_legacy_option = options.jobs.is_some()
        || options.no_substitute
        || options.no_verify
        || options.signing_key.is_some()
        || !options.trusted_public_keys.is_empty()
        || options.trust_unsigned
        || options.impure
        || options.no_host_tools
        || options.stage0_inventory.is_some()
        || options.source_store_path.is_some()
        || options.bootstrap_bwrap_path.is_some()
        || options.bootstrap_busybox_path.is_some()
        || options.offline_source_manifest_blake3.is_some()
        || options.offline_source_state_dir.is_some();
    if has_legacy_option {
        return Err(RunError::Build(
            "--cargo-free self-build cannot be combined with legacy stage0/store self-build options".to_string(),
        ));
    }
    debug_assert!(options.jobs.is_none());
    debug_assert!(!options.no_substitute);
    Ok(())
}

fn validate_stage0_inventory_args(no_host_tools: bool, stage0_inventory: Option<&Path>) -> Result<(), RunError> {
    if stage0_inventory.is_some() && !no_host_tools {
        return Err(RunError::Build("--stage0-inventory requires --no-host-tools".to_string()));
    }
    if no_host_tools && stage0_inventory.is_none() {
        return Err(RunError::Build(
            "--no-host-tools requires --stage0-inventory <path> before any host bwrap lookup".to_string(),
        ));
    }
    Ok(())
}

fn resolve_store_prefix(args: &Args) -> String {
    if args.nix_compat {
        "/nix/store".to_string()
    } else {
        args.store_prefix.clone()
    }
}

fn current_dir_or_error() -> Result<PathBuf, RunError> {
    std::env::current_dir().map_err(|e| RunError::Internal(format!("current_dir: {e}")))
}

fn parse_trusted_keys(keys: &[String]) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if keys.is_empty() {
        return Ok(None);
    }
    let mut parsed = Vec::with_capacity(keys.len());
    for key_str in keys {
        parsed.push(
            nix_compat::narinfo::VerifyingKey::parse(key_str)
                .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?,
        );
    }
    Ok(Some(parsed))
}

struct InlineBuildRequest<'a> {
    expr: &'a str,
    import_entries: &'a [OsString],
    prepared: &'a PreparedBuildCommand<'a>,
    source_fetch_overrides: Vec<crunch_build::FetchSourceOverride>,
}

/// Build from an inline Nickel expression string (for project selectors).
fn build_from_expr(request: InlineBuildRequest<'_>) -> Result<(), RunError> {
    debug_assert!(request.prepared.max_jobs > 0, "max_jobs must be positive");
    debug_assert!(request.prepared.ctx.store_prefix.starts_with('/'), "store prefix must be absolute");
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), request.expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;
    build_cmd::cmd_build_with_source_fetch_overrides(
        tmp.path(),
        request.import_entries,
        &request.prepared.ctx.store,
        &request.prepared.ctx.resolved_state_dir,
        &request.prepared.ctx.store_prefix,
        request.prepared.ctx.verbose,
        request.prepared.fix,
        request.prepared.max_jobs,
        &request.prepared.substituter_urls,
        request.prepared.signing_key,
        request.prepared.parsed_trusted.as_deref(),
        request.prepared.trust_unsigned,
        request.prepared.hermeticity_mode,
        request.prepared.ctx.output_mode(),
        request.source_fetch_overrides,
        Vec::new(),
    )
}

struct InlineBuildPlanRequest<'a> {
    expr: &'a str,
    import_entries: &'a [OsString],
    prepared: &'a PreparedBuildCommand<'a>,
    source_preflight: Option<&'a source_bundle::SourceOfflinePreflightReport>,
}

fn build_plan_from_expr(request: InlineBuildPlanRequest<'_>) -> Result<(), RunError> {
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), request.expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;
    build_plan::cmd_build_plan(build_plan::BuildPlanConfig {
        file: tmp.path(),
        import_paths: request.import_entries,
        output_dir: &request.prepared.ctx.store,
        state_dir: &request.prepared.ctx.resolved_state_dir,
        store_dir: &request.prepared.ctx.store_prefix,
        substituter_urls: &request.prepared.substituter_urls,
        signing_key_path: request.prepared.signing_key,
        trusted_public_keys: request.prepared.parsed_trusted.as_deref(),
        trust_unsigned: request.prepared.trust_unsigned,
        remote_builder: request.prepared.remote_plan_facts.as_ref(),
        source_preflight: request.source_preflight,
        output_mode: request.prepared.ctx.output_mode(),
    })
}

struct RawInlineBuildRequest<'a> {
    expr: &'a str,
    import_entries: &'a [OsString],
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
    verbose: bool,
    max_jobs: u32,
    substituter_urls: &'a [String],
    signing_key_path: Option<&'a Path>,
    trusted_public_keys: Option<&'a [nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
}

/// Build from an inline Nickel expression and return the pipeline result.
fn build_from_expr_raw(request: RawInlineBuildRequest<'_>) -> Result<crunch_pipeline::PipelineResult, RunError> {
    debug_assert!(request.max_jobs > 0, "max_jobs must be positive");
    debug_assert!(request.store_prefix.starts_with('/'), "store prefix must be absolute");
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), request.expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;
    let keypair = build_cmd::load_or_generate_signing_keypair(request.signing_key_path, request.state_dir, true)?;
    let configured_trusted_keys =
        build_cmd::load_configured_trusted_public_keys(request.trusted_public_keys, request.state_dir)?;
    let trusted_keys = crunch_build::signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());
    let config = crunch_pipeline::BuildConfig {
        file: tmp.path().to_path_buf(),
        import_paths: request.import_entries.to_vec(),
        output_dir: request.output_dir.to_path_buf(),
        state_dir: request.state_dir.to_path_buf(),
        base_state_dirs: Vec::new(),
        store_dir: request.store_prefix.to_string(),
        verbose: request.verbose,
        max_jobs: request.max_jobs,
        scheduling_policy: standard_scheduling_policy(),
        substituter_urls: request.substituter_urls.to_vec(),
        hermeticity_mode: request.hermeticity_mode,
        keypair,
        trusted_keys,
        trust_unsigned: request.trust_unsigned,
        root_retention_source: None,
        source_fetch_overrides: Vec::new(),
        remote_enabled: false,
    };
    build_cmd::run_build(&config)
}

fn output_host_path(path_info: &snix_store::path_info::PathInfo, output_dir: &Path, store_dir: &str) -> PathBuf {
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    let abs = path_info.store_path.to_absolute_path_with_prefix(store_dir);
    if output_dir == Path::new(store_dir) {
        return PathBuf::from(&abs);
    }
    let rel = abs.strip_prefix(store_dir).unwrap_or(&abs);
    let rel = rel.strip_prefix('/').unwrap_or(rel);
    output_dir.join(rel)
}

fn first_output_path(result: &crunch_pipeline::PipelineResult, output_dir: &Path, store_dir: &str) -> Option<PathBuf> {
    for outcome in &result.outcomes {
        let path_info = outcome.outputs.get("out").or_else(|| outcome.outputs.values().next())?;
        let host_path = output_host_path(path_info, output_dir, store_dir);
        if host_path.exists() {
            return Some(host_path);
        }
    }
    None
}

struct SelectedRunOutputRequest<'a> {
    result: &'a crunch_pipeline::PipelineResult,
    output_dir: &'a Path,
    store_prefix: &'a str,
    target_label: &'a str,
}

fn selected_run_output_path(request: SelectedRunOutputRequest<'_>) -> Result<PathBuf, RunError> {
    debug_assert!(request.store_prefix.starts_with('/'));
    debug_assert!(!request.target_label.is_empty());
    let outcome_count = request.result.outcomes.len();
    if outcome_count != EXPECTED_RUN_OUTCOME_COUNT {
        return Err(RunError::Internal(format!(
            "run target {} produced {outcome_count} derivations; expected exactly {EXPECTED_RUN_OUTCOME_COUNT}",
            request.target_label,
        )));
    }
    let outcome = request
        .result
        .outcomes
        .first()
        .ok_or_else(|| RunError::Internal(format!("run target {} produced no outputs", request.target_label)))?;
    let path_info = outcome
        .outputs
        .get("out")
        .or_else(|| outcome.outputs.values().next())
        .ok_or_else(|| RunError::Internal(format!("run target {} produced no outputs", request.target_label)))?;
    let host_path = output_host_path(path_info, request.output_dir, request.store_prefix);
    if !host_path.exists() {
        return Err(RunError::Internal(format!("selected run output is missing on disk: {}", host_path.display())));
    }
    Ok(host_path)
}

fn valid_run_bin_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name == "." || name == ".." {
        return false;
    }
    if name.starts_with('/') {
        return false;
    }
    if name.contains('/') || name.contains('\\') {
        return false;
    }
    true
}

#[cfg(unix)]
fn metadata_has_execute_bit(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & UNIX_EXECUTE_BITS != 0
}

#[cfg(not(unix))]
fn metadata_has_execute_bit(_metadata: &std::fs::Metadata) -> bool {
    true
}

fn executable_file_or_symlink(path: &Path) -> bool {
    let Ok(symlink_metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if symlink_metadata.is_dir() {
        return false;
    }
    if !symlink_metadata.is_file() && !symlink_metadata.file_type().is_symlink() {
        return false;
    }
    let Ok(target_metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !target_metadata.is_file() {
        return false;
    }
    metadata_has_execute_bit(&target_metadata)
}

fn select_run_binary(out_path: &Path, bin: Option<&str>) -> Result<PathBuf, RunError> {
    debug_assert!(!out_path.as_os_str().is_empty());
    let bin_dir = out_path.join("bin");
    if !bin_dir.is_dir() {
        return Err(RunError::Internal(format!("no bin/ directory in {}", out_path.display())));
    }

    if let Some(name) = bin {
        if !valid_run_bin_name(name) {
            return Err(RunError::Internal(format!("invalid --bin value '{name}': expected a single bin/ entry name")));
        }
        let exe_path = bin_dir.join(name);
        if !executable_file_or_symlink(&exe_path) {
            return Err(RunError::Internal(format!("selected binary is not executable: {}", exe_path.display())));
        }
        return Ok(exe_path);
    }

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&bin_dir)
        .map_err(|e| RunError::Internal(format!("reading {}: {e}", bin_dir.display())))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| executable_file_or_symlink(path))
        .collect();
    candidates.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));

    candidates
        .first()
        .cloned()
        .ok_or_else(|| RunError::Internal(format!("no executables in {}", bin_dir.display())))
}

fn child_exit_code(status: std::process::ExitStatus) -> i32 {
    status.code().unwrap_or(CHILD_NO_EXIT_CODE_STATUS)
}

fn exec_run(out_path: &Path, bin: Option<&str>, args: &[String]) -> Result<(), RunError> {
    let exe_path = select_run_binary(out_path, bin)?;
    eprintln!("running: {}", exe_path.display());

    let status = std::process::Command::new(&exe_path)
        .args(args)
        .status()
        .map_err(|e| RunError::Internal(format!("exec {}: {e}", exe_path.display())))?;

    std::process::exit(child_exit_code(status));
}

fn explicit_run_file_target(target: &str) -> bool {
    target.starts_with("./") || target.starts_with("../") || target.starts_with('/') || target.ends_with(".ncl")
}

/// Resolve a CLI name argument into a run build target.
fn name_to_build_target(name: Option<&str>) -> project_build::BuildTarget {
    match name {
        None => project_build::BuildTarget::ProjectDefault,
        Some(s) if s.starts_with(".#") => project_build::parse_build_target(Some(Path::new(s))),
        Some(s) if explicit_run_file_target(s) => project_build::BuildTarget::File(PathBuf::from(s)),
        Some(s) => project_build::BuildTarget::Selector(project_build::Selector {
            segments: vec![s.to_string()],
        }),
    }
}

fn generate_run_project_expr(root_file: &Path, target: &project_build::ProjectTarget) -> String {
    match target {
        project_build::ProjectTarget::Default => {
            let root_path = root_file.to_string_lossy();
            format!(
                r#"let _proj = import "{root_path}" in
let _pkg_name = if std.record.has_field "default" _proj
  then (if std.record.has_field "package" _proj.default then _proj.default.package else null)
  else null
in
if _pkg_name != null then
  _proj.packages."%{{_pkg_name}}"
else
  std.fail_with "no default.package defined in crunch.ncl""#
            )
        }
        _ => project_build::generate_extraction_expr(root_file, target),
    }
}

struct RunBuildSettings<'a> {
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    signing_key_path: Option<&'a Path>,
    trust_unsigned: bool,
}

struct ProjectExprBuildRequest<'a> {
    expr: &'a str,
    resolved_import_entries: Vec<OsString>,
    settings: &'a RunBuildSettings<'a>,
}

fn build_project_expr_request(
    request: ProjectExprBuildRequest<'_>,
) -> Result<crunch_pipeline::PipelineResult, RunError> {
    debug_assert!(request.settings.max_jobs > 0);
    debug_assert!(request.settings.store_prefix.starts_with('/'));
    let substituter_urls = run_substituter_urls(request.settings.no_substitute);
    let mut nickel_search_dirs = build_import_paths(&[])?;
    nickel_search_dirs.extend(request.resolved_import_entries);
    build_from_expr_raw(RawInlineBuildRequest {
        expr: request.expr,
        import_entries: &nickel_search_dirs,
        output_dir: request.settings.output_dir,
        state_dir: request.settings.state_dir,
        store_prefix: request.settings.store_prefix,
        verbose: request.settings.verbose,
        max_jobs: request.settings.max_jobs,
        substituter_urls: &substituter_urls,
        signing_key_path: request.settings.signing_key_path,
        trusted_public_keys: None,
        trust_unsigned: request.settings.trust_unsigned,
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
    })
}

// This adapter preserves the cross-module shell command callback signature while the core uses
// named request fields.
#[allow(
    clippy::too_many_arguments,
    tigerstyle::ambiguous_params,
    tigerstyle::too_many_parameters,
    reason = "stable shell callback delegates immediately to the named ProjectExprBuildRequest boundary"
)]
fn build_project_expr(
    expr: &str,
    resolved_import_entries: Vec<OsString>,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    signing_key_path: Option<&Path>,
    trust_unsigned: bool,
) -> Result<crunch_pipeline::PipelineResult, RunError> {
    debug_assert!(max_jobs > 0);
    debug_assert!(store_prefix.starts_with('/'));
    let settings = RunBuildSettings {
        output_dir,
        state_dir,
        store_prefix,
        verbose,
        max_jobs,
        no_substitute,
        signing_key_path,
        trust_unsigned,
    };
    build_project_expr_request(ProjectExprBuildRequest {
        expr,
        resolved_import_entries,
        settings: &settings,
    })
}

struct FileRawBuildRequest<'a> {
    file: &'a Path,
    import_paths: &'a [PathBuf],
    settings: &'a RunBuildSettings<'a>,
}

fn build_file_raw(request: FileRawBuildRequest<'_>) -> Result<crunch_pipeline::PipelineResult, RunError> {
    debug_assert!(request.settings.max_jobs > 0);
    debug_assert!(request.settings.store_prefix.starts_with('/'));
    let substituter_urls = run_substituter_urls(request.settings.no_substitute);
    let nickel_search_dirs = build_import_paths(request.import_paths)?;
    let keypair = build_cmd::load_or_generate_signing_keypair(
        request.settings.signing_key_path,
        request.settings.state_dir,
        true,
    )?;
    let configured_trusted_keys = build_cmd::load_configured_trusted_public_keys(None, request.settings.state_dir)?;
    let trusted_keys = crunch_build::signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());
    let config = crunch_pipeline::BuildConfig {
        file: request.file.to_path_buf(),
        import_paths: nickel_search_dirs,
        output_dir: request.settings.output_dir.to_path_buf(),
        state_dir: request.settings.state_dir.to_path_buf(),
        base_state_dirs: Vec::new(),
        store_dir: request.settings.store_prefix.to_string(),
        verbose: request.settings.verbose,
        max_jobs: request.settings.max_jobs,
        scheduling_policy: standard_scheduling_policy(),
        substituter_urls,
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
        keypair,
        trusted_keys,
        trust_unsigned: request.settings.trust_unsigned,
        root_retention_source: None,
        source_fetch_overrides: Vec::new(),
        remote_enabled: false,
    };
    build_cmd::run_build(&config)
}

fn run_substituter_urls(no_substitute: bool) -> Vec<String> {
    if no_substitute {
        Vec::new()
    } else {
        vec!["https://cache.nixos.org".to_string()]
    }
}

struct RunCommandRequest<'a> {
    name: Option<&'a str>,
    import_paths: &'a [PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&'a Path>,
    trust_unsigned: bool,
    bin: Option<&'a str>,
    run_args: &'a [String],
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
    verbose: bool,
}

fn cmd_run(request: RunCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(request.store_prefix.starts_with('/'));
    debug_assert!(!request.output_dir.as_os_str().is_empty());
    let cwd = current_dir_or_error()?;
    let target = name_to_build_target(request.name);
    let target_label = request.name.unwrap_or("default").to_string();
    let settings = RunBuildSettings {
        output_dir: request.output_dir,
        state_dir: request.state_dir,
        store_prefix: request.store_prefix,
        verbose: request.verbose,
        max_jobs: crunch_pipeline::resolve_max_jobs(request.jobs),
        no_substitute: request.no_substitute,
        signing_key_path: request.signing_key,
        trust_unsigned: request.trust_unsigned,
    };
    let result = match &target {
        project_build::BuildTarget::File(path) => build_file_raw(FileRawBuildRequest {
            file: path,
            import_paths: request.import_paths,
            settings: &settings,
        })?,
        project_build::BuildTarget::ProjectDefault | project_build::BuildTarget::Selector(_) => {
            let resolved = project_build::resolve_project_target(&target, &cwd, request.import_paths)?;
            let expr = generate_run_project_expr(&resolved.root_file, &resolved.target);
            build_project_expr_request(ProjectExprBuildRequest {
                expr: &expr,
                resolved_import_entries: resolved.import_paths,
                settings: &settings,
            })?
        }
    };
    let out_path = selected_run_output_path(SelectedRunOutputRequest {
        result: &result,
        output_dir: request.output_dir,
        store_prefix: request.store_prefix,
        target_label: &target_label,
    })?;
    exec_run(&out_path, request.bin, request.run_args)
}

struct BootstrapFetchRequest<'a> {
    output: &'a Path,
    store_dir: &'a Path,
    verbose: bool,
    offline_source_preflight: bool,
}

fn cmd_bootstrap_fetch(request: BootstrapFetchRequest<'_>) -> Result<(), RunError> {
    debug_assert!(!request.output.as_os_str().is_empty());
    debug_assert!(!request.store_dir.as_os_str().is_empty());
    if !request.store_dir.exists() {
        return Err(RunError::Internal(format!(
            "store directory {} does not exist.\nCreate it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            request.store_dir.display()
        )));
    }

    let source_fetch_plan = if request.offline_source_preflight {
        let state_dir = build_cmd::state_dir();
        let provider_url = bootstrap::fetch_seed_provider_raw_url()?;
        let plan = source_bundle::bootstrap_legacy_seed_fetch_override_plan(&state_dir, &provider_url)?;
        eprintln!(
            "  offline bootstrap source profile ready: records={} source_state_blake3={}",
            plan.report.record_count, plan.report.source_state_blake3
        );
        Some(plan)
    } else {
        None
    };
    let source_fetch_overrides = source_fetch_plan.as_ref().map(|plan| plan.overrides.clone()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    let result = rt.block_on(async {
        bootstrap::bootstrap_fetch(request.store_dir, request.output, request.verbose, source_fetch_overrides).await
    });
    drop(source_fetch_plan);
    result
}

fn cmd_bootstrap(output: &std::path::Path, packages: &[String]) -> Result<(), RunError> {
    eprintln!("Resolving store paths for {} packages...", packages.len());

    let entries = bootstrap::resolve_packages(packages, |pkg| {
        let path = bootstrap::nix_resolve(pkg)?;
        eprintln!("  {pkg} -> {path}");
        Ok(path)
    })?;

    let ncl = bootstrap::generate_seed_ncl(&entries);

    std::fs::write(output, &ncl).map_err(|e| RunError::Internal(format!("writing {}: {e}", output.display())))?;

    eprintln!("Wrote {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_observability_records_bounded_rejections_when_log_shell_is_unavailable() {
        const TEST_FENCE_GENERATION: u64 = 1;
        let temp = tempfile::tempdir().unwrap();
        let mut coordinator = remote_build::RemoteCoordinatorState::default();
        let attempt = remote_build::RemoteProductionAttemptBinding {
            job_id: remote_build::RemoteJobId::new("failure-observability-job").unwrap(),
            attempt_id: remote_build::RemoteAttemptId::new("failure-observability-attempt").unwrap(),
            fence_generation: remote_build::RemoteFenceGeneration::new(TEST_FENCE_GENERATION).unwrap(),
        };
        let export_config = remote_telemetry_export::RemoteTelemetryExportConfig::default();
        let telemetry_policy = crunch_build::distributed::RemoteTelemetryPolicy::default();
        let mut telemetry = crunch_build::distributed::RemoteTelemetryBuffer::default();

        let outcomes = [
            remote_build::RemoteProductionTelemetryFact::ExecutionFailed,
            remote_build::RemoteProductionTelemetryFact::OutputRejected,
        ]
        .into_iter()
        .map(|fact| {
            record_remote_failure_observability(RemoteFailureObservabilityInput {
                export_config: &export_config,
                telemetry_policy,
                state_dir: temp.path(),
                coordinator: &mut coordinator,
                attempt: &attempt,
                telemetry: &mut telemetry,
                fact,
                debug_context: None,
            })
        })
        .collect::<Vec<_>>();

        assert_eq!(telemetry.events.len(), 2);
        assert_eq!(telemetry.events[0].reason, crunch_build::distributed::RemoteTelemetryReasonClass::ExecutionFailed);
        assert_eq!(telemetry.events[1].reason, crunch_build::distributed::RemoteTelemetryReasonClass::OutputRejected);
        assert!(outcomes.iter().all(|outcome| outcome.immutable_log.is_err()));
        assert_eq!(outcomes[0].exporters.events_received, 1);
        assert_eq!(outcomes[1].exporters.events_received, 2);
        assert!(coordinator.jobs.is_empty());
        assert_eq!(
            remote_protocol_failure_fact("remote-production-input-transfer-interrupted-after-checkpoint"),
            remote_build::RemoteProductionTelemetryFact::TransferCutoff { accepted: false }
        );
        assert_eq!(
            remote_protocol_failure_fact("remote-production-transfer-interrupted-after-checkpoint"),
            remote_build::RemoteProductionTelemetryFact::TransferCutoff { accepted: false }
        );
        assert_eq!(
            remote_protocol_failure_fact("remote-protocol-frame-invalid"),
            remote_build::RemoteProductionTelemetryFact::ExecutionFailed
        );
        assert_eq!(
            remote_protocol_failure_fact(remote_build::RemoteAttemptReasonCode::StaleReportRejected.as_str()),
            remote_build::RemoteProductionTelemetryFact::StaleFenceRejected
        );
        let worker_report = parse_remote_worker_failure_debug_report(&format!(
            "build failed; worker_bundle_ref=remote-failure-debug:{}; worker_capture_outcome=captured; worker_cleanup_status=cleanup-attempted-after-capture",
            "a".repeat(REMOTE_FAILURE_DEBUG_DIGEST_HEX_CHARS),
        ))
        .unwrap();
        assert_eq!(worker_report.capture_outcome_code, "captured");
        assert!(
            parse_remote_worker_failure_debug_report(
                "worker_bundle_ref=remote-failure-debug:bad; worker_capture_outcome=captured; worker_cleanup_status=ok"
            )
            .is_none()
        );
    }

    #[cfg(unix)]
    const TEST_EXEC_MODE: u32 = 0o755;
    #[cfg(unix)]
    const TEST_READ_MODE: u32 = 0o644;
    const TEST_REMOTE_TRANSFERRED_BYTES: u64 = 11;
    const TEST_REMOTE_REUSED_BYTES: u64 = 0;
    const TEST_REMOTE_STATUS_CONCURRENCY: u32 = 2;
    const TEST_REMOTE_STATUS_CONCURRENCY_TEXT: &str = "2";
    const CLI_PARSE_TEST_STACK_BYTES: usize = 8_388_608;
    const _: () = assert!(CLI_PARSE_TEST_STACK_BYTES > 0);

    fn parse_args_with_cli_test_stack(args: Vec<&'static str>) -> Result<Args, String> {
        debug_assert!(!args.is_empty());
        std::thread::Builder::new()
            .stack_size(CLI_PARSE_TEST_STACK_BYTES)
            .spawn(move || Args::try_parse_from(args).map_err(|error| error.to_string()))
            .map_err(|error| format!("starting CLI parser test thread: {error}"))?
            .join()
            .map_err(|_| "CLI parser test thread panicked".to_string())?
    }

    fn args_with_store_prefix(store_prefix: &str, nix_compat: bool) -> Args {
        Args {
            verbose: false,
            log_level: None,
            json: false,
            store: PathBuf::from("/nix/store"),
            store_prefix: store_prefix.to_string(),
            nix_compat,
            state_dir: None,
            base_stores: Vec::new(),
            command: Command::Doctor {
                profile: DoctorProfile::Build,
            },
        }
    }

    fn remote_client_report_fixture() -> remote_build::RemoteClientBuildReport {
        let telemetry = crunch_build::distributed::RemoteTelemetryBuffer::default();
        let trace_context = remote_trace_context::accept_remote_trace_context(false, None, None).1;
        let priority =
            remote_build::plan_remote_root_priority(&crunch_build::scheduling::SchedulingPolicy::default(), &[
                remote_build::RemoteRootPriorityCandidate {
                    input_index: 0,
                    root_identity_blake3: blake3::hash(b"root").to_hex().to_string(),
                },
            ])
            .unwrap();
        let observability = remote_build::RemoteAttemptObservabilityReport {
            events: Vec::new(),
            health: remote_build::RemoteAttemptObservabilityHealth {
                telemetry: remote_build::remote_telemetry_buffer_health(&telemetry),
                exporters: remote_telemetry_export::export_remote_telemetry(
                    &remote_telemetry_export::RemoteTelemetryExportConfig::default(),
                    &[],
                ),
                trace_context,
                immutable_log: remote_build::remote_observability_adapter_health(
                    remote_telemetry_export::RemoteTelemetryAdapterStatus::Disabled,
                    "remote-observability-log-not-requested",
                ),
                non_claim: remote_build::REMOTE_OBSERVABILITY_NON_CLAIM.to_string(),
            },
            immutable_log: None,
        };
        remote_build::RemoteClientBuildReport {
            schema: REMOTE_CLIENT_BUILD_REPORT_SCHEMA.to_string(),
            builder: "builder-1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            priority_decisions: vec![priority.evidence],
            imported: vec![remote_build::RemoteClientImportedBuild {
                label: "root".to_string(),
                request_id: "request-1".to_string(),
                imported: remote_build::RemoteOutputImportReport {
                    request_id: "request-1".to_string(),
                    store_prefix: "/mantle/store".to_string(),
                    outputs: vec![remote_build::RemoteImportedOutput {
                        name: "out".to_string(),
                        logical_path: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture".to_string(),
                        path_info_signing_key_id: "builder-key".to_string(),
                        artifact_attestation_digest_blake3: blake3::hash(b"artifact").to_hex().to_string(),
                        artifact_attestation_path: "/tmp/remote-state/attestations/artifacts/out.json".to_string(),
                    }],
                    transfer: remote_build::RemoteTransferReport {
                        mode: remote_build::RemoteTransferMode::Full,
                        mode_label: "full".to_string(),
                        transferred_bytes: TEST_REMOTE_TRANSFERRED_BYTES,
                        reused_bytes: TEST_REMOTE_REUSED_BYTES,
                        fallback_reason: Some("delta-transfer-failed".to_string()),
                        verified_builder_key: "builder-key".to_string(),
                    },
                },
                observability,
            }],
        }
    }

    #[test]
    fn default_store_prefix_is_mantle() {
        let args = parse_args_with_cli_test_stack(Vec::from(["mantle", "doctor"])).expect("CLI parser test");
        assert_eq!(resolve_store_prefix(&args), "/mantle/store");
    }

    #[test]
    fn explicit_legacy_crunch_store_prefix_is_preserved() {
        let args = args_with_store_prefix("/crunch/store", false);
        assert_eq!(resolve_store_prefix(&args), "/crunch/store");
    }

    #[test]
    fn nix_compat_overrides_store_prefix_to_nix_store() {
        let args = args_with_store_prefix("/mantle/store", true);
        assert_eq!(resolve_store_prefix(&args), "/nix/store");
    }

    #[test]
    fn hermeticity_mode_selector_covers_all_modes() {
        assert_eq!(
            select_hermeticity_mode(HermeticitySelection {
                strict_hermetic: false,
                impure: false,
            })
            .unwrap(),
            crunch_pipeline::HermeticityMode::Practical
        );
        assert_eq!(
            select_hermeticity_mode(HermeticitySelection {
                strict_hermetic: true,
                impure: false,
            })
            .unwrap(),
            crunch_pipeline::HermeticityMode::Strict
        );
        assert_eq!(
            select_hermeticity_mode(HermeticitySelection {
                strict_hermetic: false,
                impure: true,
            })
            .unwrap(),
            crunch_pipeline::HermeticityMode::Impure
        );
        let err = select_hermeticity_mode(HermeticitySelection {
            strict_hermetic: true,
            impure: true,
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("--strict-hermetic and --impure are mutually exclusive"));
    }

    #[test]
    fn build_cli_rejects_strict_hermetic_with_impure() {
        let err =
            parse_args_with_cli_test_stack(Vec::from(["mantle", "build", "demo.ncl", "--strict-hermetic", "--impure"]))
                .unwrap_err();
        assert!(err.to_string().contains("cannot be used with"));
    }

    #[test]
    fn build_cli_project_selector_is_not_implicit_rust_plan() {
        let args = parse_args_with_cli_test_stack(Vec::from(["mantle", "build", ".#app"])).expect("CLI parser test");
        assert!(matches!(args.command, Command::Build { .. }));
        assert!(!matches!(args.command, Command::RustPlan { .. }));
    }

    #[test]
    fn receipt_bundle_cli_accepts_trusted_public_key_flags() {
        const TEST_KEY_A: &str = "cache.example.com-1:tLAEn+EeaBUJYqEpTd2yeerr7Ic6+0vWe+aXL/vYUpE=";
        const TEST_KEY_B: &str = "cache.example.com-2:tLAEn+EeaBUJYqEpTd2yeerr7Ic6+0vWe+aXL/vYUpE=";
        let export = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "receipt",
            "bundle",
            "export",
            "--output",
            "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo",
            "--to",
            "/tmp/receipt.json",
            "--trusted-public-key",
            TEST_KEY_A,
            "--trusted-public-key",
            TEST_KEY_B,
        ]))
        .expect("CLI parser test");
        let Command::Receipt {
            action:
                ReceiptAction::Bundle {
                    action:
                        ReceiptBundleAction::Export {
                            trusted_public_keys, ..
                        },
                },
        } = export.command
        else {
            panic!("expected receipt bundle export command");
        };
        assert_eq!(trusted_public_keys, vec![TEST_KEY_A.to_string(), TEST_KEY_B.to_string()]);

        let verify = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "receipt",
            "bundle",
            "verify",
            "--from",
            "/tmp/receipt.json",
            "--output",
            "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo",
            "--policy-hash",
            "policy-v1",
            "--trusted-public-key",
            TEST_KEY_A,
        ]))
        .expect("CLI parser test");
        assert!(matches!(verify.command, Command::Receipt {
            action: ReceiptAction::Bundle {
                action: ReceiptBundleAction::Verify { trusted_public_keys, .. },
            },
        } if trusted_public_keys == vec![TEST_KEY_A.to_string()]));

        let import = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "receipt",
            "bundle",
            "import",
            "--from",
            "/tmp/receipt.json",
            "--output",
            "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo",
            "--policy-hash",
            "policy-v1",
            "--trusted-public-key",
            TEST_KEY_B,
        ]))
        .expect("CLI parser test");
        assert!(matches!(import.command, Command::Receipt {
            action: ReceiptAction::Bundle {
                action: ReceiptBundleAction::Import { trusted_public_keys, .. },
            },
        } if trusted_public_keys == vec![TEST_KEY_B.to_string()]));
    }

    #[test]
    fn build_cli_accepts_remote_builder_ticket_dispatch_flags() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "build",
            "demo.ncl",
            "--builder",
            "builder-1",
            "--ticket",
            "ticket-1:secret-1",
            "--builder-program",
            "/bin/remote-builder",
            "--builder-arg",
            "serve",
            "--trusted-builder-key",
            "builder-key",
            "--remote-delta",
        ])
        .expect("remote builder CLI flags parse");
        let Command::Build {
            builder,
            ticket,
            builder_program,
            builder_args,
            trusted_builder_keys,
            remote_delta,
            ..
        } = args.command
        else {
            panic!("expected build command");
        };

        assert_eq!(builder.as_deref(), Some("builder-1"));
        assert_eq!(ticket.as_deref(), Some("ticket-1:secret-1"));
        assert_eq!(builder_program.as_deref(), Some(Path::new("/bin/remote-builder")));
        assert_eq!(builder_args, vec!["serve".to_string()]);
        assert_eq!(trusted_builder_keys, vec!["builder-key".to_string()]);
        assert!(remote_delta);
    }

    #[test]
    fn build_plan_remote_facts_are_pure_and_redacted() {
        let facts =
            remote_plan_facts_for_cli(Some("builder-1"), Some("ticket-1:super-secret"), &["builder-key".to_string()])
                .expect("remote plan facts construct")
                .expect("remote plan facts present");
        let candidate = realization_routing::remote_builder_candidate_from_facts(&facts);
        let detail = candidate.detail.as_deref().unwrap_or_default();

        assert_eq!(candidate.route, realization_routing::RouteClass::P2pRemoteBuilder);
        assert_eq!(candidate.reason_code, "builder-capability-and-output-trust-match");
        assert!(detail.contains("endpoint=builder-1"));
        assert!(!detail.contains("super-secret"));
        assert!(!detail.contains("ticket-1"));
    }

    #[test]
    fn remote_status_cli_accepts_endpoint_and_concurrency() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "remote",
            "status",
            "--endpoint-id",
            "builder-1",
            "--concurrency",
            TEST_REMOTE_STATUS_CONCURRENCY_TEXT,
        ]))
        .expect("CLI parser test");
        let Command::Remote {
            action: RemoteAction::Status {
                endpoint_id,
                concurrency,
            },
        } = args.command
        else {
            panic!("expected remote status command");
        };

        assert_eq!(endpoint_id, "builder-1");
        assert_eq!(concurrency, TEST_REMOTE_STATUS_CONCURRENCY);
    }

    #[test]
    fn remote_client_json_report_uses_build_report_schema_and_substitution_fields() {
        let report = remote_client_report_fixture();
        let json = remote_client_build_json_report(
            &report,
            Path::new("demo.ncl"),
            Path::new("/tmp/remote-store"),
            Path::new("/tmp/remote-state"),
        )
        .expect("remote build JSON report renders");

        assert_eq!(json["schema"], BUILD_JSON_REPORT_SCHEMA);
        assert_eq!(json["ast_grep_structural_evidence"], serde_json::json!([]));
        assert_eq!(json["ast_grep_structural_evidence_diagnostics"], serde_json::json!([]));
        assert_eq!(json["counts"]["succeeded_total"], 1);
        assert_eq!(json["counts"]["cached_total"], 1);
        assert_eq!(json["outcomes"][0]["cached"], true);
        assert_eq!(
            json["outcomes"][0]["outputs"][0]["path"],
            "/tmp/remote-store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture"
        );
        assert_eq!(json["outcomes"][0]["outputs"][0]["substitution"]["mode"], "full");
        assert_eq!(json["outcomes"][0]["outputs"][0]["substitution"]["fallback_reason"], "delta-transfer-failed");
        assert_eq!(
            json["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"],
            "/tmp/remote-state/attestations/artifacts/out.json"
        );
    }

    #[test]
    fn self_build_cli_accepts_impure_mode() {
        let args =
            parse_args_with_cli_test_stack(Vec::from(["mantle", "self-build", "--impure"])).expect("CLI parser test");
        assert!(matches!(args.command, Command::SelfBuild {
            impure: true,
            strict_hermetic: false,
            ..
        }));
    }

    #[test]
    fn source_root_provider_store_name_uses_nix_store_hash_shape() {
        let digest = blake3::hash(b"source-root-provider").to_hex().to_string();
        let name = source_root_provider_store_name(&digest).unwrap();
        let (hash, suffix) = name.split_once('-').unwrap();

        assert_eq!(hash.len(), NIX_BASE32_STORE_HASH_CHARS);
        assert_eq!(suffix, bootstrap_source_root::PROVIDER_NAME);
    }

    #[test]
    fn source_root_provider_store_name_rejects_bad_digest() {
        let err = source_root_provider_store_name("ABC").unwrap_err().to_string();

        assert!(err.contains("source-root provider output digest"));
    }

    #[test]
    fn bootstrap_source_root_provider_reaches_materializer_and_fails_closed_on_patches() {
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("store");
        fs::create_dir(&store).unwrap();
        let manifest_path = dir.path().join("source-root.json");
        let output = dir.path().join("seed.ncl");
        fs::write(&manifest_path, source_root_manifest_with_patch_json().to_string()).unwrap();

        let err = bootstrap_source_root_provider(&output, &manifest_path, &store, false).unwrap_err().to_string();

        assert!(err.contains("source-root provider materialization failed"));
        assert!(err.contains("patch"));
        assert!(err.contains("not implemented") || err.contains("placeholder"));
        assert!(!output.exists());
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--recipe",
            "bootstrap/rust-source.ncl",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap();

        let Command::Bootstrap {
            action:
                Some(BootstrapAction::RustSourceProvider {
                    recipe,
                    route_plan,
                    full_source_admission,
                    full_source_host_tools,
                    full_source_rust_sources,
                    import_dir,
                    smoke,
                    smoke_evidence_dir,
                    output_dir,
                    scratch_dir,
                }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert_eq!(recipe, PathBuf::from("bootstrap/rust-source.ncl"));
        assert!(route_plan.is_none());
        assert!(full_source_admission.is_none());
        assert!(full_source_host_tools.is_none());
        assert!(full_source_rust_sources.is_none());
        assert!(import_dir.is_none());
        assert!(!smoke);
        assert!(smoke_evidence_dir.is_none());
        assert_eq!(output_dir, PathBuf::from("/tmp/mantle-rust-provider"));
        assert!(scratch_dir.is_none());
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses_route_plan() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--recipe",
            "bootstrap/rust-source.ncl",
            "--route-plan",
            "bootstrap/rust-source-musl-host-plan.ncl",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap();

        let Command::Bootstrap {
            action: Some(BootstrapAction::RustSourceProvider {
                route_plan, output_dir, ..
            }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert_eq!(route_plan, Some(PathBuf::from("bootstrap/rust-source-musl-host-plan.ncl")));
        assert_eq!(output_dir, PathBuf::from("/tmp/mantle-rust-provider"));
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses_persistent_scratch() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--scratch-dir",
            "/tmp/mantle-rust-provider-work",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap();

        let Command::Bootstrap {
            action: Some(BootstrapAction::RustSourceProvider { scratch_dir, .. }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert_eq!(scratch_dir, Some(PathBuf::from("/tmp/mantle-rust-provider-work")));
    }

    #[test]
    fn persistent_rust_source_provider_scratch_is_create_new() {
        let dir = tempfile::tempdir().unwrap();
        let scratch_path = dir.path().join("retained-work");

        let scratch = prepare_rust_source_provider_scratch(Some(&scratch_path)).unwrap();
        let duplicate_error = prepare_rust_source_provider_scratch(Some(&scratch_path)).unwrap_err();

        assert!(matches!(&scratch, RustSourceProviderScratch::Persistent(_)));
        assert_eq!(scratch.path(), fs::canonicalize(&scratch_path).unwrap());
        assert!(duplicate_error.to_string().contains("creating persistent Rust provider scratch"));
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses_full_source_admission() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--route-plan",
            "bootstrap/rust-source-musl-host-plan.ncl",
            "--full-source-admission",
            "/evidence/full-source-admission.json",
            "--full-source-host-tools",
            "/evidence/full-source-host-tools.json",
            "--full-source-rust-sources",
            "/evidence/rust-sources",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap();

        let Command::Bootstrap {
            action:
                Some(BootstrapAction::RustSourceProvider {
                    full_source_admission,
                    full_source_host_tools,
                    full_source_rust_sources,
                    import_dir,
                    ..
                }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert_eq!(full_source_admission, Some(PathBuf::from("/evidence/full-source-admission.json")));
        assert_eq!(full_source_host_tools, Some(PathBuf::from("/evidence/full-source-host-tools.json")));
        assert_eq!(full_source_rust_sources, Some(PathBuf::from("/evidence/rust-sources")));
        assert!(import_dir.is_none());
    }

    #[test]
    fn bootstrap_rust_source_provider_rejects_admission_without_host_tools() {
        let error = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--full-source-admission",
            "/evidence/full-source-admission.json",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--full-source-host-tools"));
        assert!(error.contains("required"));
    }

    #[test]
    fn bootstrap_rust_source_provider_rejects_host_tools_without_admission() {
        let error = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--full-source-host-tools",
            "/evidence/full-source-host-tools.json",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--full-source-admission"));
        assert!(error.contains("required"));
    }

    #[test]
    fn bootstrap_rust_source_provider_rejects_sources_without_admission() {
        let error = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--full-source-rust-sources",
            "/evidence/rust-sources",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--full-source-admission"));
        assert!(error.contains("required"));
    }

    #[test]
    fn bootstrap_rust_source_provider_rejects_binding_an_imported_provider() {
        let error = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--import-dir",
            "/tmp/imported-provider",
            "--full-source-admission",
            "/evidence/full-source-admission.json",
            "--full-source-host-tools",
            "/evidence/full-source-host-tools.json",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--import-dir"));
        assert!(error.contains("--full-source-admission"));
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses_import_dir() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--import-dir",
            "/tmp/source-built-rust-provider",
            "--smoke",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap();

        let Command::Bootstrap {
            action:
                Some(BootstrapAction::RustSourceProvider {
                    import_dir,
                    smoke,
                    smoke_evidence_dir,
                    output_dir,
                    ..
                }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert_eq!(import_dir, Some(PathBuf::from("/tmp/source-built-rust-provider")));
        assert!(smoke);
        assert!(smoke_evidence_dir.is_none());
        assert_eq!(output_dir, PathBuf::from("/tmp/mantle-rust-provider"));
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses_smoke_evidence_dir() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--import-dir",
            "/tmp/source-built-rust-provider",
            "--smoke",
            "--smoke-evidence-dir",
            "/tmp/mantle-rust-provider-smoke-evidence",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap();

        let Command::Bootstrap {
            action:
                Some(BootstrapAction::RustSourceProvider {
                    smoke,
                    smoke_evidence_dir,
                    ..
                }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert!(smoke);
        assert_eq!(smoke_evidence_dir, Some(PathBuf::from("/tmp/mantle-rust-provider-smoke-evidence")));
    }

    #[test]
    fn bootstrap_offline_source_preflight_flag_parses() {
        let args =
            parse_args_with_cli_test_stack(Vec::from(["mantle", "bootstrap", "--fetch", "--offline-source-preflight"]))
                .expect("CLI parser test");

        let Command::Bootstrap {
            fetch,
            offline_source_preflight,
            ..
        } = args.command
        else {
            panic!("expected bootstrap command");
        };
        assert!(fetch);
        assert!(offline_source_preflight);
    }

    #[test]
    fn source_bundle_bootstrap_profile_subcommand_parses() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "source",
            "bundle",
            "bootstrap-profile",
            "--mode",
            "legacy-seed",
            "--provider-archive",
            "/tmp/provider",
            "--provider-manifest",
            "/tmp/provider.json",
            "--bootstrap-source",
            "/tmp/src",
        ]))
        .expect("CLI parser test");

        let Command::Source {
            action:
                SourceAction::Bundle {
                    action: SourceBundleAction::BootstrapProfile { mode, .. },
                },
        } = args.command
        else {
            panic!("expected source bundle bootstrap-profile");
        };
        assert_eq!(mode, "legacy-seed");
    }

    #[test]
    fn source_bundle_hydrate_self_build_subcommand_requires_explicit_identity() {
        const EXPECTED_MANIFEST_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "source",
            "bundle",
            "hydrate-self-build",
            "--from",
            "/tmp/source-bundle.json",
            "--expected-manifest-blake3",
            EXPECTED_MANIFEST_BLAKE3,
            "--checkout",
            "/tmp/fresh-clone",
        ]))
        .expect("CLI parser test");

        let Command::Source {
            action:
                SourceAction::Bundle {
                    action:
                        SourceBundleAction::HydrateSelfBuild {
                            expected_manifest_blake3: parsed,
                            ..
                        },
                },
        } = args.command
        else {
            panic!("expected source bundle hydrate-self-build");
        };
        assert_eq!(parsed, EXPECTED_MANIFEST_BLAKE3);
    }

    #[test]
    fn bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke() {
        let err = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--import-dir",
            "/tmp/source-built-rust-provider",
            "--smoke-evidence-dir",
            "/tmp/mantle-rust-provider-smoke-evidence",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ]))
        .unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--smoke"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn bootstrap_full_source_rust_binding_requires_host_tool_manifest_before_route_execution() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("missing-recipe.ncl");
        let admission = dir.path().join("missing-admission.json");
        let output_dir = dir.path().join("rust-provider");

        let error = cmd_bootstrap_rust_source_provider(RustSourceProviderCommandRequest {
            recipe: &recipe,
            route_plan: None,
            full_source_admission: Some(&admission),
            full_source_host_tools: None,
            full_source_rust_sources: None,
            import_dir: None,
            output_dir: &output_dir,
            scratch_dir: None,
            verbose: false,
            smoke: false,
            smoke_evidence_dir: None,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("requires --full-source-host-tools"));
        assert!(!error.contains("recipe"));
        assert!(!output_dir.exists());
    }

    #[test]
    fn bootstrap_rust_source_provider_fails_closed_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output_dir = dir.path().join("rust-provider");
        let store = dir.path().join("store");
        let missing_source = dir.path().join("missing-mrustc.tar.gz");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_fast_failing_rust_source_route(dir.path(), &missing_source);
        fs::create_dir(&store).unwrap();

        let err = cmd_bootstrap_rust_source_provider(RustSourceProviderCommandRequest {
            recipe: &recipe,
            route_plan: None,
            full_source_admission: None,
            full_source_host_tools: None,
            full_source_rust_sources: None,
            import_dir: None,
            output_dir: &output_dir,
            scratch_dir: None,
            verbose: false,
            smoke: false,
            smoke_evidence_dir: None,
        })
        .unwrap_err()
        .to_string();

        assert!(err.contains("Rust source provider materialization failed closed"));
        assert!(err.contains("mrustc-0.12.0"));
        assert!(err.contains("missing-mrustc.tar.gz"));
        assert!(err.contains("preserved_scratch="));
        assert!(!err.contains("Materialized Rust source provider"));
        assert!(!output_dir.exists());
        let preserved_scratch = preserved_scratch_from_error(&err);
        assert!(preserved_scratch.exists());
        fs::remove_dir_all(&preserved_scratch).unwrap();
    }

    fn preserved_scratch_from_error(message: &str) -> PathBuf {
        const PRESERVED_SCRATCH_FIELD: &str = "preserved_scratch=";
        let (_, value) = message.split_once(PRESERVED_SCRATCH_FIELD).unwrap();
        PathBuf::from(value.trim())
    }

    fn write_fast_failing_rust_source_route(root: &Path, missing_source: &Path) {
        const SAMPLE_SHA256_HEX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let missing_url = url::Url::from_file_path(missing_source).unwrap().to_string();
        let text = r#"
let Source = {
  id | String,
  kind | String,
  name | String,
  version | String,
  url | String,
  sha256_hex | String,
} in
let Stage = {
  id | String,
  kind | String,
  source_ids | Array String,
  bootstrap_stage_id | String,
  rust_version | String,
  outputs | Array String,
  notes | Array String,
} in
let Output = {
  role | String,
  path | String,
} in
let Policy = {
  source_built | Bool,
  uses_prebuilt_rust | Bool,
  forbids_prebuilt_rust | Bool,
  reference | String,
} in
let Plan = {
  schema | String,
  provider_id | String,
  route | String,
  host_triple | String,
  target_triple | String,
  final_version | String,
  policy | Policy,
  sources | Array Source,
  stages | Array Stage,
  final_outputs | Array Output,
} in
{
  schema = "mantle-rust-source-provider-bootstrap-plan-v1",
  provider_id = "mantle-rust-source-provider",
  route = "mrustc-source-route",
  host_triple = "x86_64-unknown-linux-gnu",
  target_triple = "x86_64-unknown-linux-musl",
  final_version = "1.94.0",
  policy = {
    source_built = true,
    uses_prebuilt_rust = false,
    forbids_prebuilt_rust = true,
    reference = "fast failing unit route",
  },
  sources = [
    {
      id = "mrustc-0.12.0",
      kind = "tarball",
      name = "mrustc-source",
      version = "0.12.0",
      url = "__MISSING_URL__",
      sha256_hex = "__SAMPLE_SHA256__",
    },
    {
      id = "rust-1.90.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.90.0",
      url = "__MISSING_URL__",
      sha256_hex = "__SAMPLE_SHA256__",
    },
    {
      id = "rust-1.94.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.94.0",
      url = "__MISSING_URL__",
      sha256_hex = "__SAMPLE_SHA256__",
    },
  ],
  stages = [
    {
      id = "mrustc-to-rust-1.90.0",
      kind = "mrustc-seed",
      source_ids = ["mrustc-0.12.0", "rust-1.90.0"],
      bootstrap_stage_id = "",
      rust_version = "1.90.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["fast failing source fetch boundary"],
    },
    {
      id = "rust-1.94.0-final",
      kind = "rustc-final",
      source_ids = ["rust-1.94.0"],
      bootstrap_stage_id = "mrustc-to-rust-1.90.0",
      rust_version = "1.94.0",
      outputs = ["rustc", "cargo", "rustdoc", "host-rustlib", "target-rustlib", "provider-receipt"],
      notes = ["install provider metadata and receipts"],
    },
  ],
  final_outputs = [
    { role = "rustc", path = "bin/rustc" },
    { role = "cargo", path = "bin/cargo" },
    { role = "rustdoc", path = "bin/rustdoc" },
    { role = "host-rustlib", path = "lib/rustlib/x86_64-unknown-linux-gnu/lib" },
    { role = "target-rustlib", path = "lib/rustlib/x86_64-unknown-linux-musl/lib" },
    { role = "provider-receipt", path = "share/mantle-rust-provider/receipts/build.json" },
  ],
} | Plan
"#
        .replace("__MISSING_URL__", &missing_url)
        .replace("__SAMPLE_SHA256__", SAMPLE_SHA256_HEX);
        fs::write(root.join("rust-source-plan.ncl"), text).unwrap();
    }

    fn source_root_manifest_with_patch_json() -> serde_json::Value {
        let digest_a = blake3::hash(b"artifact").to_hex().to_string();
        let digest_b = blake3::hash(b"patch").to_hex().to_string();
        let digest_c = blake3::hash(b"output").to_hex().to_string();
        let patch_name = "fix.patch";
        let artifacts: Vec<_> = source_root_provider::REQUIRED_ARTIFACTS
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let patches = if index == 0 { vec![patch_name] } else { Vec::new() };
                serde_json::json!({
                    "name": name,
                    "source": format!("https://example.invalid/{name}.tar.gz"),
                    "digest": {"algorithm": "blake3", "value": digest_a},
                    "extraction": {"kind": "tar.gz", "strip_prefix": name},
                    "provenance": "fixture source",
                    "patches": patches,
                })
            })
            .collect();
        let expected_outputs: Vec<_> = bootstrap_source_root::REQUIRED_PROVIDER_TOOL_ROLES
            .iter()
            .map(|role| {
                serde_json::json!({
                    "name": role,
                    "kind": "file-or-directory",
                    "digest": {"algorithm": "blake3", "value": digest_c},
                    "provenance": "fixture output",
                    "contract_role": role,
                })
            })
            .collect();
        serde_json::json!({
            "version": bootstrap_source_root::SOURCE_ROOT_MANIFEST_VERSION,
            "artifacts": artifacts,
            "patches": [{
                "name": patch_name,
                "digest": {"algorithm": "blake3", "value": digest_b},
                "provenance": "fixture patch",
                "apply_order": FIRST_SOURCE_ROOT_PATCH_ORDER,
            }],
            "network_trust_roots": [],
            "trust_notes": [],
            "expected_outputs": expected_outputs,
        })
    }

    #[test]
    fn flake_check_workflow_has_one_verification_command() {
        let workflow = include_str!("../.github/workflows/flake-check.yml");
        let run_lines = workflow.lines().map(str::trim).filter(|line| line.starts_with("run:")).collect::<Vec<_>>();

        assert_eq!(run_lines, vec!["run: nix flake check"]);
        assert!(!workflow.contains("cargo test"));
    }

    #[test]
    fn bootstrap_capabilities_is_discoverable_and_self_build_source_root_is_not_advertised() {
        let capabilities = parse_args_with_cli_test_stack(vec!["mantle", "bootstrap", "capabilities"]);
        let unsupported =
            parse_args_with_cli_test_stack(vec!["mantle", "self-build", "--source-root", "/tmp/source-root.json"]);

        assert!(capabilities.is_ok());
        assert!(unsupported.is_err());
    }

    #[test]
    fn release_create_accepts_cairn_handoff_flag() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-0.1.0-rc1",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--cairn-handoff",
            "/tmp/cairn-handoff.json",
        ])
        .unwrap();
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Create {
                cairn_handoff: Some(_),
                ..
            }
        }));
    }

    #[test]
    fn release_create_accepts_provider_fixed_point_proof_flag() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-0.1.0-rc1",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--provider-fixed-point-proof",
            "/tmp/provider-fixed-point-proof",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Create {
                provider_fixed_point_proof: Some(_),
                ..
            }
        }));
    }

    #[test]
    fn release_content_bound_requirement_flags_parse() {
        let create = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-content-bound",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--requirement-evidence-root",
            "/tmp/requirement-evidence",
            "--requirement-evidence-input",
            "input.json",
        ]))
        .expect("content-bound release create arguments must parse");
        assert!(matches!(create.command, Command::Release {
            action: ReleaseAction::Create {
                requirement_evidence_root: Some(_),
                requirement_evidence_input: Some(_),
                ..
            }
        }));

        let verify = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "verify",
            "/tmp/release-bundle",
            "--requirement-coverage",
            "required",
        ]))
        .expect("strict requirement coverage arguments must parse");
        assert!(matches!(
            verify.command,
            Command::Release {
                action: ReleaseAction::Verify {
                    requirement_coverage,
                    ..
                }
            } if requirement_coverage == "required"
        ));
    }

    #[test]
    fn release_create_accepts_source_acquisition_url_flag() {
        let source_url = "https://example.invalid/releases/mantle-src.tar";
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-0.1.0-rc1",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--source-acquisition-url",
            source_url,
        ]))
        .expect("CLI parser test");
        let Command::Release {
            action:
                ReleaseAction::Create {
                    source_acquisition_url: Some(url),
                    ..
                },
        } = args.command
        else {
            panic!("expected release create with source acquisition URL");
        };
        assert_eq!(url, source_url);
    }

    #[test]
    fn release_create_accepts_external_evidence_flags() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-0.1.0-rc1",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--external-evidence",
            "/tmp/stack-provenance.json",
            "--external-evidence-role",
            "stack-provenance-trace",
            "--external-evidence-schema",
            "valence.stack-provenance-adapter.v1",
            "--external-evidence-claim-scope",
            "identity-linkage-sidecar",
            "--external-evidence-non-claim",
            "not semantic validation by Mantle",
        ]))
        .expect("CLI parser test");
        let Command::Release {
            action:
                ReleaseAction::Create {
                    external_evidence,
                    external_evidence_role,
                    external_evidence_schema,
                    external_evidence_claim_scope,
                    external_evidence_non_claim,
                    ..
                },
        } = args.command
        else {
            panic!("expected release create with external evidence flags");
        };
        assert_eq!(external_evidence, vec![PathBuf::from("/tmp/stack-provenance.json")]);
        assert_eq!(external_evidence_role, vec!["stack-provenance-trace".to_string()]);
        assert_eq!(external_evidence_schema, vec!["valence.stack-provenance-adapter.v1".to_string()]);
        assert_eq!(external_evidence_claim_scope, vec!["identity-linkage-sidecar".to_string()]);
        assert_eq!(external_evidence_non_claim, vec!["not semantic validation by Mantle".to_string()]);
    }

    #[test]
    fn release_create_accepts_kani_toolchain_evidence_flag() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-0.1.0-rc1",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--kani-toolchain-evidence",
            "/tmp/kani-toolchain.json",
        ]))
        .expect("CLI parser test");
        let Command::Release {
            action: ReleaseAction::Create {
                kani_toolchain_evidence,
                ..
            },
        } = args.command
        else {
            panic!("expected release create with Kani toolchain evidence");
        };
        assert_eq!(kani_toolchain_evidence, vec![PathBuf::from("/tmp/kani-toolchain.json")]);
    }

    #[test]
    fn release_verify_accepts_required_external_evidence_role() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "verify",
            "/tmp/release-bundle",
            "--require-external-evidence-role",
            "stack-provenance-trace",
        ]))
        .expect("CLI parser test");
        let Command::Release {
            action:
                ReleaseAction::Verify {
                    require_external_evidence_role,
                    ..
                },
        } = args.command
        else {
            panic!("expected release verify with external evidence role");
        };
        assert_eq!(require_external_evidence_role, vec!["stack-provenance-trace".to_string()]);
    }

    #[test]
    fn release_function_address_bind_accepts_typed_artifact_selection() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "release",
            "function-address-bind",
            "/tmp/release-bundle",
            "--mode",
            "required",
            "--sidecar",
            "external/sidecar.json",
            "--valence-receipt",
            "external/valence.json",
            "--kamacite-receipt",
            "external/kamacite.json",
            "--release-binary",
            "binaries/mantle",
            "--receipt-out",
            "/tmp/mantle-binding.json",
        ])
        .expect("function-address binding arguments must parse");
        let Command::Release {
            action:
                ReleaseAction::FunctionAddressBind {
                    mode,
                    from_preserves_binding,
                    sidecar,
                    valence_receipt,
                    kamacite_receipt,
                    release_binary,
                    receipt_out,
                    ..
                },
        } = args.command
        else {
            panic!("expected release function-address-bind");
        };
        assert_eq!(mode, "required");
        assert!(!from_preserves_binding);
        assert_eq!(sidecar.as_deref(), Some("external/sidecar.json"));
        assert_eq!(valence_receipt.as_deref(), Some("external/valence.json"));
        assert_eq!(kamacite_receipt.as_deref(), Some("external/kamacite.json"));
        assert_eq!(release_binary.as_deref(), Some("binaries/mantle"));
        assert_eq!(receipt_out, PathBuf::from("/tmp/mantle-binding.json"));
    }

    #[test]
    fn release_function_address_bind_accepts_preserves_manifest_selection() {
        let args = parse_args_with_cli_test_stack(vec![
            "mantle",
            "release",
            "function-address-bind",
            "/tmp/release-bundle",
            "--mode",
            "required",
            "--from-preserves-binding",
            "--receipt-out",
            "/tmp/mantle-preserves-binding.json",
        ])
        .expect("Preserves binding arguments must parse");
        let Command::Release {
            action:
                ReleaseAction::FunctionAddressBind {
                    from_preserves_binding,
                    sidecar,
                    valence_receipt,
                    kamacite_receipt,
                    release_binary,
                    ..
                },
        } = args.command
        else {
            panic!("expected Preserves release function-address binding");
        };

        assert!(from_preserves_binding);
        assert!(sidecar.is_none());
        assert!(valence_receipt.is_none());
        assert!(kamacite_receipt.is_none());
        assert!(release_binary.is_none());
    }

    #[test]
    fn release_function_address_bind_rejects_mixed_preserves_and_explicit_selection() {
        let rendered = parse_args_with_cli_test_stack(vec![
            "mantle",
            "release",
            "function-address-bind",
            "/tmp/release-bundle",
            "--from-preserves-binding",
            "--sidecar",
            "external/sidecar.json",
            "--receipt-out",
            "/tmp/mantle-binding.json",
        ])
        .expect_err("mixed Preserves and explicit selection was accepted");
        assert!(rendered.contains("--from-preserves-binding"));
        assert!(rendered.contains("--sidecar"));
    }

    #[test]
    fn release_function_address_bind_rejects_unknown_mode() {
        let rendered = parse_args_with_cli_test_stack(vec![
            "mantle",
            "release",
            "function-address-bind",
            "/tmp/release-bundle",
            "--mode",
            "semantic-proof",
            "--sidecar",
            "external/sidecar.json",
            "--valence-receipt",
            "external/valence.json",
            "--receipt-out",
            "/tmp/mantle-binding.json",
        ])
        .expect_err("unknown function-address mode was accepted");
        assert!(rendered.contains("semantic-proof"));
        assert!(rendered.contains("invalid value"));
    }

    #[test]
    fn release_verify_accepts_onix_stack_release_profile() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "verify",
            "/tmp/release-bundle",
            "--release-profile",
            "onix-stack",
        ]))
        .expect("CLI parser test");
        let Command::Release {
            action:
                ReleaseAction::Verify {
                    release_profile,
                    stack_provenance,
                    ..
                },
        } = args.command
        else {
            panic!("expected release verify with Onix stack profile");
        };
        assert_eq!(release_profile, "onix-stack");
        assert_eq!(stack_provenance, "optional");
    }

    #[test]
    fn release_create_accepts_git_source_flags() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "create",
            "--release-id",
            "mantle-0.1.0-rc1",
            "--binary",
            "/tmp/mantle",
            "--proof-bundle",
            "/tmp/self-hosting-proof",
            "--git-source-url",
            "file:///tmp/mantle-origin.git",
            "--git-source-commit",
            "0123456789abcdef0123456789abcdef01234567",
            "--git-source-ref",
            "refs/heads/main",
            "--git-source-tag",
            "v0.1.0",
        ]))
        .expect("CLI parser test");
        let Command::Release {
            action:
                ReleaseAction::Create {
                    git_source_url: Some(url),
                    git_source_commit: Some(commit),
                    git_source_ref: Some(reference),
                    git_source_tag: Some(tag),
                    ..
                },
        } = args.command
        else {
            panic!("expected release create with Git source flags");
        };
        assert_eq!(url, "file:///tmp/mantle-origin.git");
        assert_eq!(commit, "0123456789abcdef0123456789abcdef01234567");
        assert_eq!(reference, "refs/heads/main");
        assert_eq!(tag, "v0.1.0");
    }

    #[test]
    fn release_gauntlet_continuous_accepts_track_inputs() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "gauntlet",
            "continuous",
            "--context",
            "/tmp/context.json",
            "--track",
            "/tmp/repeatability.json",
            "--report-path",
            "/tmp/report.json",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Gauntlet {
                action: ReleaseGauntletAction::Continuous { track, .. },
            }
        } if track.len() == EXPECTED_RUN_OUTCOME_COUNT));
    }

    #[test]
    fn release_gauntlet_canonicalize_accepts_report_kind() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "gauntlet",
            "canonicalize",
            "--kind",
            "substitution-cache-attack",
            "/tmp/report.json",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Gauntlet {
                action: ReleaseGauntletAction::Canonicalize {
                    kind: GauntletReportKind::SubstitutionCacheAttack,
                    ..
                },
            }
        }));
    }

    #[test]
    fn release_gauntlet_accepts_strict_hermeticity_regression_inputs() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "gauntlet",
            "strict-hermeticity-regression",
            "--run-id",
            "run-a",
            "--plan",
            "/tmp/plan.json",
            "--evidence",
            "/tmp/evidence.json",
            "--report-path",
            "/tmp/report.json",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Gauntlet {
                action: ReleaseGauntletAction::StrictHermeticityRegression { evidence, .. },
            }
        } if evidence.len() == EXPECTED_RUN_OUTCOME_COUNT));
    }

    #[test]
    fn release_witness_rebuild_accepts_require_independent_source_flag() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "witness-rebuild",
            "/tmp/witness-request",
            "--require-independent-source",
            "--check",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::WitnessRebuild {
                require_independent_source: true,
                check: true,
                ..
            }
        }));
    }

    #[test]
    fn release_witness_rebuild_accepts_require_git_source_flag() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "witness-rebuild",
            "/tmp/witness-request",
            "--require-git-source",
            "--check",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::WitnessRebuild {
                require_git_source: true,
                check: true,
                ..
            }
        }));
    }

    #[test]
    fn release_witness_rebuild_rejects_conflicting_source_replay_modes() {
        let result = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "witness-rebuild",
            "/tmp/witness-request",
            "--require-independent-source",
            "--require-git-source",
        ]));
        assert!(result.is_err());
    }

    #[test]
    fn release_verify_accepts_provider_fixed_point_proof_flags() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "release",
            "verify",
            "/tmp/release-bundle",
            "--provider-fixed-point-proof",
            "/tmp/provider-fixed-point-proof",
            "--require-provider-fixed-point-proof",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Verify {
                provider_fixed_point_proof: Some(_),
                require_provider_fixed_point_proof: true,
                ..
            }
        }));
    }

    #[test]
    fn self_build_cli_accepts_source_built_fixed_point_authority() {
        const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--source-built-fixed-point",
            "--source-profile",
            "/tmp/source-profile.json",
            "--expected-source-profile-blake3",
            DIGEST,
            "--expected-stagex-lineage-blake3",
            DIGEST,
            "--expected-native-provider-blake3",
            DIGEST,
            "--proof-bwrap",
            "/tmp/bwrap",
            "--proof-sandbox-shell",
            "/tmp/busybox",
            "--out",
            "/tmp/source-built-proof",
        ]))
        .expect("CLI parser test");

        assert!(matches!(args.command, Command::SelfBuild {
            source_built_fixed_point: true,
            cargo_free: false,
            source_profile: Some(_),
            out: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_rejects_source_built_and_legacy_cargo_free_modes_together() {
        let result = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--source-built-fixed-point",
            "--cargo-free",
        ]));

        assert!(result.is_err());
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_out_dir() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            fixed_point: false,
            out: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_fixed_point_out_dir() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--fixed-point",
            "--out",
            "/tmp/mantle-fixed-point",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            fixed_point: true,
            out: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_toolchain_closure_manifest() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
            "--toolchain-closure",
            "/tmp/toolchain.json",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            toolchain_closure: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_rust_source_provider() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
            "--rust-source-provider",
            "/tmp/rust-source-provider",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            rust_source_provider: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_target_triple() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
            "--target",
            "x86_64-unknown-linux-musl",
        ]))
        .expect("CLI parser test");
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            targets,
            ..
        } if targets == vec!["x86_64-unknown-linux-musl".to_string()]));
    }

    #[test]
    fn self_build_cli_rejects_toolchain_closure_without_cargo_free() {
        let err = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--toolchain-closure",
            "/tmp/toolchain.json",
        ]))
        .unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--cargo-free"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn self_build_cli_rejects_rust_source_provider_without_cargo_free() {
        let err = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--rust-source-provider",
            "/tmp/rust-source-provider",
        ]))
        .unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--cargo-free"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn self_build_cli_rejects_fixed_point_without_cargo_free() {
        let err = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--fixed-point",
            "--out",
            "/tmp/mantle-out",
        ]))
        .unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--cargo-free"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn cargo_free_fixed_point_accepts_strict_hermetic_mode() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--fixed-point",
            "--strict-hermetic",
            "--out",
            "/tmp/mantle-fixed-point",
        ]))
        .expect("CLI parser test");

        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            fixed_point: true,
            strict_hermetic: true,
            impure: false,
            ..
        }));
    }

    #[test]
    fn cargo_free_fixed_point_rejects_impure_mode() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--fixed-point",
            "--impure",
            "--out",
            "/tmp/mantle-fixed-point",
        ]))
        .expect("CLI parser test");
        let ctx = build_run_context(&args);
        let err = run_self_build_from_command(&ctx, &args.command).unwrap_err();

        assert!(err.to_string().contains("cannot be combined with legacy stage0/store self-build options"));
    }

    #[test]
    fn cargo_free_fixed_point_rejects_legacy_options() {
        let args = parse_args_with_cli_test_stack(Vec::from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--fixed-point",
            "--out",
            "/tmp/mantle-fixed-point",
            "--jobs",
            "1",
        ]))
        .expect("CLI parser test");

        let err = run(args).unwrap_err();

        assert_eq!(
            err.to_string(),
            "error: build failed\n--cargo-free self-build cannot be combined with legacy stage0/store self-build options"
        );
    }

    #[test]
    fn self_build_offline_source_manifest_requires_source_state() {
        const MANIFEST_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let error = parse_args_with_cli_test_stack(vec![
            "crunch",
            "self-build",
            "--offline-source-manifest-blake3",
            MANIFEST_BLAKE3,
        ])
        .unwrap_err();

        assert!(error.contains("--offline-source-state-dir"));
        assert!(error.contains("required"));
    }

    #[test]
    fn self_build_offline_source_state_requires_manifest() {
        let error = parse_args_with_cli_test_stack(vec![
            "crunch",
            "self-build",
            "--offline-source-state-dir",
            "/tmp/source-state",
        ])
        .unwrap_err();

        assert!(error.contains("--offline-source-manifest-blake3"));
        assert!(error.contains("required"));
    }

    #[test]
    fn cargo_free_self_build_rejects_legacy_options() {
        let err = validate_cargo_free_legacy_self_build_args(CargoFreeLegacyOptions {
            jobs: Some(1),
            no_substitute: false,
            no_verify: false,
            signing_key: None,
            trusted_public_keys: &[],
            trust_unsigned: false,
            impure: false,
            no_host_tools: false,
            stage0_inventory: None,
            source_store_path: None,
            bootstrap_bwrap_path: None,
            bootstrap_busybox_path: None,
            offline_source_manifest_blake3: None,
            offline_source_state_dir: None,
        })
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "error: build failed\n--cargo-free self-build cannot be combined with legacy stage0/store self-build options"
        );
    }

    #[cfg(unix)]
    fn write_file_with_mode(path: &Path, contents: &str, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, contents).unwrap();
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    fn no_host_tools_requires_stage0_inventory_before_host_bwrap_lookup() {
        let err = validate_stage0_inventory_args(true, None).unwrap_err();

        assert_eq!(
            err.to_string(),
            "error: build failed\n--no-host-tools requires --stage0-inventory <path> before any host bwrap lookup"
        );
    }

    #[test]
    fn stage0_inventory_requires_no_host_tools_mode() {
        let path = Path::new("/tmp/stage0-inventory.ncl");
        let err = validate_stage0_inventory_args(false, Some(path)).unwrap_err();

        assert_eq!(err.to_string(), "error: build failed\n--stage0-inventory requires --no-host-tools");
    }

    #[test]
    fn run_target_resolves_selectors_before_file_suffix() {
        let target = name_to_build_target(Some(".#foo.ncl"));
        assert!(matches!(target, project_build::BuildTarget::Selector(_)));
        assert!(!matches!(target, project_build::BuildTarget::File(_)));
    }

    #[test]
    fn run_target_resolves_explicit_files_and_bare_packages() {
        assert!(matches!(name_to_build_target(Some("tool.ncl")), project_build::BuildTarget::File(_)));
        assert!(matches!(name_to_build_target(Some("./tool")), project_build::BuildTarget::File(_)));
        assert!(matches!(name_to_build_target(Some("tool")), project_build::BuildTarget::Selector(_)));
        assert!(matches!(name_to_build_target(None), project_build::BuildTarget::ProjectDefault));
    }

    #[test]
    fn run_bin_name_rejects_path_like_values() {
        assert!(valid_run_bin_name("tool"));
        assert!(!valid_run_bin_name(""));
        assert!(!valid_run_bin_name("."));
        assert!(!valid_run_bin_name(".."));
        assert!(!valid_run_bin_name("/tool"));
        assert!(!valid_run_bin_name("../tool"));
        assert!(!valid_run_bin_name("dir/tool"));
    }

    #[cfg(unix)]
    #[test]
    fn run_binary_fallback_skips_invalid_entries() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        write_file_with_mode(&bin.join("aaa"), "no execute", TEST_READ_MODE);
        std::os::unix::fs::symlink(bin.join("missing"), bin.join("aab")).unwrap();
        write_file_with_mode(&bin.join("bbb"), "#!/bin/sh\n", TEST_EXEC_MODE);

        let selected = select_run_binary(temp.path(), None).unwrap();
        assert_eq!(selected.file_name().unwrap(), "bbb");
        assert!(selected.ends_with("bbb"));
    }

    #[cfg(unix)]
    #[test]
    fn run_binary_explicit_symlink_selects_executable_target() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        write_file_with_mode(&bin.join("real"), "#!/bin/sh\n", TEST_EXEC_MODE);
        std::os::unix::fs::symlink(bin.join("real"), bin.join("alias")).unwrap();

        let selected = select_run_binary(temp.path(), Some("alias")).unwrap();
        assert_eq!(selected.file_name().unwrap(), "alias");
        assert!(selected.ends_with("alias"));
    }

    #[cfg(unix)]
    #[test]
    fn run_binary_explicit_rejects_non_executable_target() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        write_file_with_mode(&bin.join("tool"), "no execute", TEST_READ_MODE);

        let err = select_run_binary(temp.path(), Some("tool")).unwrap_err().to_string();
        assert!(err.contains("not executable"));
        assert!(err.contains("tool"));
    }
}
