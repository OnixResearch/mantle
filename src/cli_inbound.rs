//! Clap inbound DTOs.

use std::ffi::OsString;

use clap::Subcommand;
use clap::ValueEnum;
use foreign_import_cmd::ForeignImportAction;
use mantlepkgs_cmd::MantlepkgsAction;
use nix_free_demo_cmd::NixFreeDemoAction;
use operator_diagnostics::DoctorProfile;

use super::*;

const SOURCE_BUILT_FIXED_POINT_ELAPSED_SECONDS_MAX_DEFAULT: u64 = 86_400;
const SOURCE_BUILT_FIXED_POINT_DISK_BYTES_MAX_DEFAULT: u64 = 1_099_511_627_776;
const SOURCE_BUILT_FIXED_POINT_PROTECTED_EXEC_EVENTS_MAX_DEFAULT: u32 = 262_144;
const SOURCE_BUILT_FIXED_POINT_SOURCE_RECORDS_MAX_DEFAULT: u32 = 65_536;

pub(super) fn has_conflicting_machine_output_modes(args: &Args) -> bool {
    args.json
        && matches!(args.command, Command::Build {
            evaluation_stream: true,
            ..
        })
}

#[derive(Parser, Debug)]
#[command(
    name = "mantle",
    bin_name = "mantle",
    version,
    about = "Mantle build system on the Nix store protocol"
)]
pub(super) struct Args {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub(super) verbose: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true)]
    pub(super) log_level: Option<String>,

    /// Emit errors as JSON objects for tooling integration
    #[arg(long, global = true)]
    pub(super) json: bool,

    /// Where to export final build outputs on disk. Intermediate
    /// deps stay in castore. Source inputs always come from /nix/store.
    /// If not writable, builds still succeed (outputs in castore).
    #[arg(long, global = true, default_value = "/nix/store")]
    pub(super) store: PathBuf,

    /// Logical store path prefix for derivation hashes. All output
    /// paths and ATerm hashes are computed against this prefix.
    /// Default: /mantle/store. Use --nix-compat for /nix/store.
    #[arg(long, global = true, default_value = "/mantle/store")]
    pub(super) store_prefix: String,

    /// Shorthand for --store-prefix=/nix/store. For interop testing
    /// with Nix-computed derivations.
    #[arg(long, global = true)]
    pub(super) nix_compat: bool,

    /// State directory for pathinfo.redb, blobs, and logs.
    /// Default: legacy $CRUNCH_STATE_DIR or $XDG_STATE_HOME/crunch or
    /// ~/.local/state/crunch. Use --state-dir for an explicit Mantle path.
    #[arg(long, global = true)]
    pub(super) state_dir: Option<PathBuf>,

    /// Ordered list of read-only base store state directories for overlay
    /// composition. Declared in priority order (base A before base B).
    #[arg(long = "base-store", global = true)]
    pub(super) base_stores: Vec<PathBuf>,

    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RustLocalCacheMode {
    Off,
    Read,
    ReadWrite,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RustSharedCacheMode {
    Off,
    Read,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum OperatorContractMode {
    RawDescriptors,
    Catalog,
    Reference,
    Workflow,
    Check,
}

#[derive(Subcommand, Debug)]
pub(super) enum RustCacheAction {
    /// Serve bounded rustc wrapper requests from one user-owned cache daemon.
    Serve {
        /// Exported and reviewed daemon policy JSON.
        #[arg(long)]
        policy: PathBuf,

        /// Directory for content-addressed wrapper receipts.
        #[arg(long)]
        receipt_dir: PathBuf,

        /// Stop after one accepted connection. Intended for tests.
        #[arg(long)]
        once: bool,
    },
}

// CLI variants retain their complete clap payloads to preserve flag and help compatibility.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub(super) enum Command {
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

        /// Emit the bounded mantle-evaluation-stream-v1 NDJSON contract on stdout
        #[arg(long, conflicts_with_all = ["fix", "plan", "builder"])]
        evaluation_stream: bool,

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

        /// Caller-owned non-terminal descriptor that provides one bearer credential
        #[arg(long, requires = "builder")]
        ticket_fd: Option<i32>,

        /// Program to spawn for stdio remote-build dispatch. Defaults to this mantle binary.
        #[arg(long)]
        builder_program: Option<PathBuf>,

        /// Argument passed to --builder-program; repeat to pass multiple arguments.
        #[arg(long = "builder-arg")]
        builder_args: Vec<String>,

        /// Trusted remote builder signing key name; repeat for multiple keys.
        #[arg(long = "trusted-builder-key")]
        trusted_builder_keys: Vec<String>,

        /// SecretSpec metadata manifest for the local remote service
        #[arg(long, default_value = "secretspec.toml", requires = "builder")]
        remote_secret_manifest: PathBuf,

        /// Explicit SecretSpec profile for the local remote service
        #[arg(
            long,
            default_value = remote_service_secrets::REMOTE_SECRET_PRODUCTION_PROFILE,
            requires = "builder"
        )]
        remote_secret_profile: String,

        /// Explicit bounded SecretSpec provider for the local remote service
        #[arg(
            long,
            default_value = remote_service_secrets::REMOTE_SYSTEMD_CREDENTIAL_PROVIDER,
            requires = "builder"
        )]
        remote_secret_provider: String,

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

    /// Internal command-graph export for the checked operator catalog.
    #[command(name = "__operator-contract", hide = true)]
    OperatorContract {
        /// Select the internal export or freshness operation.
        #[arg(long, value_enum)]
        mode: OperatorContractMode,
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

        /// Checked evaluation budget policy JSON
        #[arg(long, requires = "budget_report")]
        budget_policy: Option<PathBuf>,

        /// Write the bounded evaluation budget report to this path
        #[arg(long, requires = "budget_policy")]
        budget_report: Option<PathBuf>,

        /// Force one selected top-level root; repeat for multiple roots
        #[arg(long = "root", requires = "budget_policy", conflicts_with = "all_roots")]
        roots: Vec<String>,

        /// Discover and force all top-level roots through the bounded worker
        #[arg(long, requires = "budget_policy")]
        all_roots: bool,
    },

    /// Internal owned evaluator worker
    #[command(hide = true)]
    EvaluatorWorker,

    /// Internal evaluator-worker process fixture
    #[command(hide = true)]
    EvaluatorWorkerFixture { behavior: String },

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

    /// Internal bounded SecretSpec worker
    #[command(hide = true)]
    RemoteSecretWorker {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        profile: String,
        #[arg(long)]
        provider: String,
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

        /// Receipt-validated promoted provider checkpoint store.
        #[arg(
            long,
            requires = "source_built_fixed_point",
            conflicts_with_all = ["dev_provider_cache", "dev_resume", "dev_fast_fail"]
        )]
        proof_checkpoint_store: Option<PathBuf>,

        /// Import completed provider stages from one preserved promoted attempt.
        #[arg(long, requires = "proof_checkpoint_store")]
        proof_checkpoint_import_attempt: Option<PathBuf>,

        /// Reuse a receipt-validated StageX and native prefix from one stopped attempt.
        #[arg(
            long,
            requires = "proof_checkpoint_store",
            conflicts_with = "proof_checkpoint_import_attempt"
        )]
        proof_native_checkpoint_attempt: Option<PathBuf>,

        /// Opt-in dev-only provider-output cache and store snapshot directory.
        #[arg(long, requires = "source_built_fixed_point")]
        dev_provider_cache: Option<PathBuf>,

        /// Resume a dev run from verified per-stage completion markers.
        #[arg(long, requires = "source_built_fixed_point")]
        dev_resume: bool,

        /// Fast-fail a dev run when the current source profile is unchanged.
        #[arg(long, requires = "source_built_fixed_point")]
        dev_fast_fail: bool,

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

    /// Operate the daemon-backed Rust compiler cache.
    RustCache {
        #[command(subcommand)]
        action: RustCacheAction,
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

        /// Pre-execution fixed authority for Rust child actions
        #[arg(long, hide = true, requires = "rust_child_action_evidence_dir")]
        rust_child_action_authority: Option<PathBuf>,

        /// Output directory for Rust child-action plan, audit, and reconciliation evidence
        #[arg(long, hide = true, requires = "rust_child_action_authority")]
        rust_child_action_evidence_dir: Option<PathBuf>,

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

        /// Explicit local Rust unit cache policy
        #[arg(long, value_enum, default_value_t = RustLocalCacheMode::Off)]
        local_rust_cache: RustLocalCacheMode,

        /// Explicit shared Rust unit cache policy
        #[arg(long, value_enum, default_value_t = RustSharedCacheMode::Off)]
        shared_rust_cache: RustSharedCacheMode,

        /// Ordered shared Rust result source: an HTTP(S) base URL or directory
        #[arg(long = "shared-rust-cache-source")]
        shared_rust_cache_sources: Vec<String>,

        /// Shared Rust publication target: an HTTP(S) base URL or directory
        #[arg(long)]
        shared_rust_cache_publish_target: Option<String>,

        /// Trusted shared Rust verifier in name:base64 format; repeatable
        #[arg(long = "shared-rust-cache-trusted-key")]
        shared_rust_cache_trusted_keys: Vec<String>,

        /// Accepted shared Rust producer-policy identity; repeatable
        #[arg(long = "shared-rust-cache-producer-policy")]
        shared_rust_cache_producer_policies: Vec<String>,

        /// Nix-format Ed25519 key file for shared Rust result publication
        #[arg(long)]
        shared_rust_cache_signing_key: Option<PathBuf>,

        /// Keep remote shared Rust result and object sources unopened
        #[arg(long)]
        shared_rust_cache_offline: bool,
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
pub(super) enum WasmComponentAction {
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
pub(super) enum FilegenCommandAction {
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
pub(super) enum ImportAction {
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
pub(super) enum PinImportAction {
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
pub(super) enum PinImporter {
    Nixtamal,
    Flake,
    Npins,
    Niv,
}

impl PinImporter {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            PinImporter::Nixtamal => pin_import::PIN_IMPORTER_NIXTAMAL,
            PinImporter::Flake => pin_import::PIN_IMPORTER_FLAKE,
            PinImporter::Npins => pin_import::PIN_IMPORTER_NPINS,
            PinImporter::Niv => pin_import::PIN_IMPORTER_NIV,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub(super) enum RefactorAction {
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
pub(super) enum TranscriptAction {
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
pub(super) enum BootstrapAction {
    /// Report executable and unsupported source-root operations from live host observations
    Capabilities,

    /// Produce the deterministic whole-bootstrap parity gap report
    ParityReport {
        /// Fail closed unless the requested parity axes are complete
        #[arg(long = "require")]
        require: Vec<bootstrap_parity::ParityAxis>,
    },

    /// Prepare, run, or verify the optional offline Radiance reference proof
    RadianceReference {
        #[command(subcommand)]
        action: RadianceReferenceAction,
    },

    /// Verify and render the bounded source-built proof trust result
    TrustReport {
        /// Source-built fixed-point proof root
        #[arg(long, value_name = "PATH")]
        proof_root: PathBuf,
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

#[derive(Subcommand, Debug)]
pub(super) enum RadianceReferenceAction {
    /// Authenticate three connected Git SHA-256 checkouts and publish one offline bundle
    Prepare {
        #[arg(long)]
        git: PathBuf,
        #[arg(long)]
        radiance_checkout: PathBuf,
        #[arg(long)]
        bootstrap_compiler_checkout: PathBuf,
        #[arg(long)]
        emulator_checkout: PathBuf,
        #[arg(long)]
        source_bundle_out: PathBuf,
        #[arg(long)]
        cohort_out: PathBuf,
    },

    /// Run both routes from one authenticated offline source bundle
    Prove {
        #[arg(long)]
        source_bundle: PathBuf,
        #[arg(long)]
        expected_source_bundle_blake3: String,
        #[arg(long)]
        cohort: PathBuf,
        #[arg(long)]
        expected_cohort_blake3: String,
        /// Explicit Clang-compatible C compiler launcher; PATH lookup is forbidden
        #[arg(long)]
        cc: PathBuf,
        /// Exact compiler driver executed by the explicit launcher
        #[arg(long)]
        cc_driver: PathBuf,
        /// Explicit linker executable; PATH lookup is forbidden
        #[arg(long)]
        linker: PathBuf,
        /// Directory holding Scrt1.o, crti.o, crtn.o, libc, and the dynamic loader
        #[arg(long)]
        crt_dir: PathBuf,
        /// Directory holding crtbeginS.o, crtendS.o, and libgcc
        #[arg(long)]
        libgcc_dir: PathBuf,
        /// Absent absolute proof-output directory
        #[arg(long)]
        output: PathBuf,
    },

    /// Replay receipt, protected-audit, and publication checks without execution
    Verify {
        #[arg(long)]
        output: PathBuf,
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

        /// Source-review attachment JSON produced by an admitted external reviewer; bundled
        /// without modifying its external authority fields
        #[arg(long = "source-review-attachment")]
        source_review_attachment: Option<PathBuf>,

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

        /// Source-review attachment JSON overriding the bundled member for this verification
        #[arg(long = "source-review")]
        source_review: Option<PathBuf>,

        /// Source-review policy mode; `required` selects an explicit reviewed-source policy
        #[arg(long = "reviewed-source", value_parser = ["optional", "required"], default_value = "optional")]
        reviewed_source: String,

        /// Named reviewed-source preset defining the distinct reviewer threshold
        #[arg(long = "review-preset")]
        review_preset: Option<String>,

        /// Explicit distinct reviewer threshold when no preset is selected
        #[arg(long = "review-threshold")]
        review_threshold: Option<u16>,

        /// Trusted reviewer public key as label:base64 (repeatable)
        #[arg(long = "trusted-reviewer")]
        trusted_reviewer: Vec<String>,

        /// Revoked reviewer public key as label:base64 (repeatable)
        #[arg(long = "revoked-reviewer")]
        revoked_reviewer: Vec<String>,

        /// Change-author public key (base64) excluded from approval counting
        #[arg(long = "review-author-key")]
        review_author_key: Option<String>,

        /// Current Cairn claim root BLAKE3 the review must bind
        #[arg(long = "review-claim-root")]
        review_claim_root: Option<String>,

        /// Producer identity review statements must carry
        #[arg(long = "review-producer-id")]
        review_producer_id: Option<String>,
    },
    /// Pack, inspect, or unpack an opt-in chaptered release transport
    Transport {
        #[command(subcommand)]
        action: ReleaseTransportAction,
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
pub enum ReleaseTransportAction {
    /// Pack a verified release directory into a chaptered transport directory
    Pack {
        /// Verified release evidence directory
        bundle_dir: PathBuf,

        /// New transport directory containing release.tgz and receipt.json
        #[arg(long)]
        to: PathBuf,
    },
    /// Inspect and validate a chaptered release transport directory
    Inspect {
        /// Transport directory containing release.tgz and receipt.json
        transport_dir: PathBuf,
    },
    /// Unpack a validated chaptered transport into a verified release directory
    Unpack {
        /// Transport directory containing release.tgz and receipt.json
        transport_dir: PathBuf,

        /// New destination for the verified release evidence directory
        #[arg(long)]
        to: PathBuf,
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

        /// Trusted witness identities recorded in policy.json for optional or witness-count
        /// profiles
        #[arg(long = "trusted-witness-identity")]
        trusted_witness_identity: Vec<String>,

        /// Positive witness minimum required by the witness-quorum profile
        #[arg(long)]
        min_matching_witnesses: Option<u32>,

        /// Supported policy field used to count independent witness domains
        #[arg(long)]
        independence_field: Option<String>,

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
    OptionalWitness,
    SingleWitness,
    WitnessQuorum,
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
    /// Nix compatibility and bounded asynchronous Build API gateway
    Gateway {
        #[command(subcommand)]
        action: RemoteGatewayAction,
    },
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

        /// SecretSpec metadata manifest
        #[arg(long, default_value = "secretspec.toml")]
        secret_manifest: PathBuf,

        /// Explicit SecretSpec profile
        #[arg(long, default_value = remote_service_secrets::REMOTE_SECRET_PRODUCTION_PROFILE)]
        secret_profile: String,

        /// Explicit bounded SecretSpec provider
        #[arg(long, default_value = remote_service_secrets::REMOTE_SYSTEMD_CREDENTIAL_PROVIDER)]
        secret_provider: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RemoteGatewayAction {
    /// Print supported versions, operations, bounds, and non-claims
    Metadata,
    /// Validate one saved API request and print a side-effect-free typed plan
    Plan {
        /// Versioned gateway API request JSON
        request: PathBuf,
    },
    /// Validate gateway endpoint policy and print redacted operator status
    Status {
        /// Gateway policy JSON
        policy: PathBuf,
    },
    /// Read one bounded API request from stdin and print a typed no-effect plan
    ApiStdioOnce,
    /// Execute one admitted asynchronous API operation against durable coordinator state
    ApiDispatchStdioOnce {
        /// Verifier-produced ticket or UCAN authority facts JSON
        #[arg(long)]
        authority: PathBuf,
        /// Caller-owned descriptor containing exactly 32 cursor-MAC key bytes
        #[arg(long)]
        cursor_key_fd: i32,
        /// SecretSpec metadata manifest for completion-event signing
        #[arg(long, default_value = "secretspec.toml")]
        secret_manifest: PathBuf,
        /// Explicit SecretSpec profile
        #[arg(long, default_value = remote_service_secrets::REMOTE_SECRET_PRODUCTION_PROFILE)]
        secret_profile: String,
        /// Explicit bounded SecretSpec provider
        #[arg(long, default_value = remote_service_secrets::REMOTE_SYSTEMD_CREDENTIAL_PROVIDER)]
        secret_provider: String,
    },
    /// Serve one Nix daemon-store session on stdin/stdout
    NixStdioOnce {
        /// Verified ticket or UCAN authority facts JSON
        #[arg(long)]
        authority: PathBuf,
        /// Gateway policy JSON
        #[arg(long)]
        policy: PathBuf,
        /// Trusted Nix store public key; repeat for key rotation overlap
        #[arg(long = "trusted-store-key", required = true)]
        trusted_store_keys: Vec<String>,
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
        /// Caller-owned non-terminal descriptor that provides one bearer credential
        #[arg(long)]
        ticket_fd: i32,
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
        /// SecretSpec metadata manifest for the local remote service
        #[arg(long, default_value = "secretspec.toml")]
        remote_secret_manifest: PathBuf,
        /// Explicit SecretSpec profile for the local remote service
        #[arg(long, default_value = remote_service_secrets::REMOTE_SECRET_PRODUCTION_PROFILE)]
        remote_secret_profile: String,
        /// Explicit bounded SecretSpec provider for the local remote service
        #[arg(long, default_value = remote_service_secrets::REMOTE_SYSTEMD_CREDENTIAL_PROVIDER)]
        remote_secret_provider: String,
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

        /// Caller-owned file descriptor that receives the bearer credential once
        #[arg(long, conflicts_with = "interactive_operator_terminal_reveal")]
        ticket_fd: Option<i32>,

        /// Reveal the new bearer once through the controlling terminal
        #[arg(long, conflicts_with = "ticket_fd", required_unless_present = "ticket_fd")]
        interactive_operator_terminal_reveal: bool,

        /// SecretSpec metadata manifest
        #[arg(long, default_value = "secretspec.toml")]
        secret_manifest: PathBuf,

        /// Explicit SecretSpec profile
        #[arg(long, default_value = remote_service_secrets::REMOTE_SECRET_PRODUCTION_PROFILE)]
        secret_profile: String,

        /// Explicit bounded SecretSpec provider
        #[arg(long, default_value = remote_service_secrets::REMOTE_SYSTEMD_CREDENTIAL_PROVIDER)]
        secret_provider: String,
    },
    /// List verifier-only ticket records
    List,
    /// Inspect one verifier-only ticket record
    Inspect { id: String },
    /// Revoke one ticket
    Revoke { id: String },
    /// Invalidate every plaintext-era ticket and upgrade state to verifier-only schema
    MigrateLegacy {
        /// Confirm that all legacy ticket credentials become invalid
        #[arg(long)]
        invalidate_legacy_tickets: bool,
        /// Report the migration without changing state
        #[arg(long)]
        dry_run: bool,
    },
    /// Rotate service keys and invalidate tickets tied to older verifier key ids
    RotateKeys {
        /// SecretSpec metadata manifest
        #[arg(long, default_value = "secretspec.toml")]
        secret_manifest: PathBuf,
        /// Explicit SecretSpec rotation profile
        #[arg(long, default_value = remote_service_secrets::REMOTE_SECRET_ROTATION_PROFILE)]
        secret_profile: String,
        /// Explicit bounded SecretSpec provider
        #[arg(long)]
        secret_provider: String,
    },
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
    /// List retained GC roots with provenance
    Roots {
        /// Persist protected versioned records for legacy path-only roots
        #[arg(long)]
        migrate: bool,
    },
    /// Report bounded retained, reclaimable, shared, and unknown store usage
    Usage,
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
    /// Plan garbage collection, or execute one accepted unchanged plan
    Gc {
        /// Execute the accepted plan instead of creating a non-mutating plan
        #[arg(long, requires = "plan_id")]
        execute: bool,

        /// Exact BLAKE3 plan identity returned by a prior planning command
        #[arg(long, value_name = "BLAKE3_PLAN_ID", requires = "execute")]
        plan_id: Option<String>,

        /// Compatibility spelling for the now-default non-mutating plan
        #[arg(long = "dry-run", hide = true, conflicts_with = "execute")]
        legacy_dry_run: bool,
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
    /// Plan or realize an experimental frontend-neutral castore composition root
    Composition {
        #[command(subcommand)]
        action: StoreCompositionAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum StoreCompositionAction {
    /// Validate a bounded generic projection and report canonical identities
    Plan {
        /// JSON composition request
        #[arg(long)]
        from: PathBuf,
    },
    /// Realize a validated composition from complete local castore roots
    Realize {
        /// JSON composition request
        #[arg(long)]
        from: PathBuf,
        /// Atomic realization receipt output
        #[arg(long)]
        receipt_out: PathBuf,
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
    /// Replace the Mantle source and its checked vendor record in a verified source-built profile
    RefreshMantleSource {
        /// Existing source-built fixed-point profile
        #[arg(long)]
        from: std::path::PathBuf,

        /// Mantle source tree to bind into the refreshed profile
        #[arg(long = "mantle-source")]
        mantle_source: std::path::PathBuf,

        /// Materialized fetch-source bundle to add; repeat for multiple bundles
        #[arg(long = "include-bundle")]
        include_bundles: Vec<std::path::PathBuf>,

        /// New profile output path, which must not exist
        #[arg(long)]
        to: std::path::PathBuf,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum StoreArchiveFormat {
    #[default]
    MantleNative,
    NarioV2,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StoreArchiveAction {
    /// Export selected store paths and recursive closure to an archive
    Export {
        /// Archive format. Nario v2 export is unsupported.
        #[arg(long, value_enum, default_value_t)]
        format: StoreArchiveFormat,

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
    /// Import a Mantle-native or supported compatibility archive
    Import {
        /// Archive format to read
        #[arg(long, value_enum, default_value_t)]
        format: StoreArchiveFormat,

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
        /// Archive format to read
        #[arg(long, value_enum, default_value_t)]
        format: StoreArchiveFormat,

        /// Archive input path, or '-' for stdin
        #[arg(long)]
        from: std::path::PathBuf,
    },
}
