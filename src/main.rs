#![feature(register_tool)]
#![register_tool(tigerstyle)]
mod artifact_cmd;
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
mod build_report;
mod cargo_free_self_build;
mod cargo_import;
mod errors;
mod fix;
mod frontend_artifact_export;
mod frontend_artifact_spec;
mod frontend_artifact_store;
mod log_cmd;
mod native_toolchain_closure;
#[allow(dead_code)]
mod offline_cargo;
mod operator_diagnostics;
mod pin_import;
mod portable_receipt;
mod project_build;
mod project_cmd;
mod project_resolve;
#[allow(dead_code)]
mod protected_exec;
#[allow(dead_code)]
mod protected_exec_seccomp;
mod realization_routing;
mod release_attestation;
mod release_cmd;
mod release_evidence;
mod release_nix_witness;
mod release_reproducibility;
mod release_source;
#[allow(dead_code)]
mod remote_build;
mod rust_bootstrap_patch_plan;
mod rust_plan;
#[allow(dead_code)]
mod rust_source_provider;
mod self_build;
#[allow(dead_code)]
mod semantic_graph;
mod shell_cmd;
mod source_bundle;
mod source_root_provider;
mod source_toolchain_closure;
mod store_cmd;
mod structured_refactor;
mod transcript_cmd;
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
#[cfg(unix)]
const UNIX_EXECUTE_BITS: u32 = 0o111;

use build_cmd::BuildOutputMode;
use build_cmd::build_import_paths;
use build_cmd::cmd_build;
use build_cmd::state_dir;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use errors::RunError;
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

    #[command(subcommand)]
    command: Command,
}

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
    },

    /// Run no-mutate operator preflight checks for a workflow profile
    Doctor {
        /// Workflow profile to check
        #[arg(long, value_enum, default_value_t = DoctorProfile::Build)]
        profile: DoctorProfile,
    },

    /// Import external project metadata into Mantle-owned build files
    Import {
        #[command(subcommand)]
        action: ImportAction,
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

    /// Evaluate a .ncl file and print the derivation JSON (no build)
    Eval {
        /// Path to the .ncl file
        file: PathBuf,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,
    },

    /// Generate bootstrap seeds or validate bootstrap runtime evidence
    Bootstrap {
        #[command(subcommand)]
        action: Option<BootstrapAction>,

        /// Output file path
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

        /// Validate a StageX-class lineage manifest and select the lineage provider path.
        #[arg(long)]
        stagex_lineage: Option<PathBuf>,

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

        /// Validate a full-source root manifest and bind self-build proof to source-root provider.
        #[arg(long)]
        source_root: Option<PathBuf>,

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

        /// Build Mantle through native Rust topology execution without invoking Cargo.
        #[arg(long)]
        cargo_free: bool,

        /// Run the bounded Cargo-free stage1/stage2 fixed-point proof.
        #[arg(long, requires = "cargo_free")]
        fixed_point: bool,

        /// Output directory for --cargo-free binary and receipt evidence.
        #[arg(long, requires = "cargo_free")]
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
        #[arg(long, value_enum, default_value_t = PinImporter::Nixtamal)]
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
        #[arg(long, value_enum, default_value_t = PinImporter::Nixtamal)]
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

        /// Fail unless a valid provider fixed-point proof bundle is supplied
        #[arg(long)]
        require_provider_fixed_point_proof: bool,
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
    },
    /// List receipt bundle metadata
    List {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,
    },
    /// Verify receipt bundle structure, trust snapshot, and completeness
    Verify {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,
    },
    /// Import a verified receipt bundle into Mantle evidence state
    Import {
        /// Bundle input path
        #[arg(long)]
        from: std::path::PathBuf,
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

        /// Accept unsigned/unverified narinfos
        #[arg(long)]
        trust_unsigned: bool,

        /// Trusted public keys for signature verification (name:base64)
        #[arg(long, value_delimiter = ',')]
        trusted_public_keys: Vec<String>,

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

    let json_errors = args.json;

    match run(args) {
        Ok(()) => ExitCode::from(0),
        Err(error) => {
            let rendered = if json_errors {
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
    let log_requested = args.verbose || args.log_level.is_some() || std::env::var_os("RUST_LOG").is_some();
    if args.json && !log_requested {
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
    }
}

fn emit_runtime_fingerprint(args: &Args, ctx: &RunContext) -> Result<(), RunError> {
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
        Command::Doctor { .. } => "doctor",
        Command::Import { .. } => "import",
        Command::Graph { .. } => "graph",
        Command::Why { .. } => "why",
        Command::Dependents { .. } => "dependents",
        Command::Refactor { .. } => "refactor",
        Command::Transcript { .. } => "transcript",
        Command::Stage0Inventory { .. } => "stage0-inventory",
        Command::Eval { .. } => "eval",
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

fn bootstrap_command_label(action: Option<&BootstrapAction>) -> &'static str {
    match action {
        Some(BootstrapAction::ParityReport { .. }) => "bootstrap.parity-report",
        Some(BootstrapAction::RustSourceProvider { .. }) => "bootstrap.rust-source-provider",
        Some(BootstrapAction::NativeToolchainClosure { .. }) => "bootstrap.native-toolchain-closure",
        Some(BootstrapAction::Validate { .. }) => "bootstrap.validate",
        None => "bootstrap",
    }
}

fn remote_command_label(action: &RemoteAction) -> &'static str {
    match action {
        RemoteAction::Ticket { action } => remote_ticket_command_label(action),
        RemoteAction::Status { .. } => "remote.status",
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
        SourceBundleAction::List { .. } => "source.bundle.list",
        SourceBundleAction::Import { .. } => "source.bundle.import",
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

fn runtime_fingerprint_mode_fields(args: &Args) -> RuntimeFingerprintModeFields {
    let mut modes = RuntimeFingerprintModeFields::default();
    if args.nix_compat {
        modes.nix_compat_mode = Some(true);
    }
    match &args.command {
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
            apply_build_mode_fields(
                &mut modes,
                selected_hermeticity_mode_label(*strict_hermetic, *impure),
                *no_substitute,
                Some(substituters),
                signing_key.is_some(),
                trusted_public_keys.len(),
                *trust_unsigned,
            );
            if builder.is_some() || ticket.is_some() {
                modes.bearer_ticket_count =
                    Some(operator_diagnostics::bounded_runtime_count(usize::from(ticket.is_some())));
            }
        }
        Command::SelfBuild {
            no_substitute,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
            impure,
            ..
        } => apply_build_mode_fields(
            &mut modes,
            selected_hermeticity_mode_label(*strict_hermetic, *impure),
            *no_substitute,
            None,
            signing_key.is_some(),
            trusted_public_keys.len(),
            *trust_unsigned,
        ),
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
        } => apply_build_mode_fields(
            &mut modes,
            "practical",
            *no_substitute,
            None,
            signing_key.is_some(),
            EMPTY_TRUSTED_PUBLIC_KEY_COUNT,
            *trust_unsigned,
        ),
        Command::Bootstrap {
            action:
                Some(BootstrapAction::Validate {
                    strict_hermetic,
                    impure,
                    ..
                }),
            ..
        } => modes.hermeticity_mode = Some(selected_hermeticity_mode_label(*strict_hermetic, *impure).to_string()),
        Command::Store { action } => apply_store_mode_fields(&mut modes, action),
        _ => {}
    }
    modes
}

fn selected_hermeticity_mode_label(strict_hermetic: bool, impure: bool) -> &'static str {
    match (strict_hermetic, impure) {
        (true, false) => "strict",
        (false, true) => "impure",
        (false, false) => "practical",
        (true, true) => "conflicting",
    }
}

fn apply_build_mode_fields(
    modes: &mut RuntimeFingerprintModeFields,
    hermeticity_mode: &str,
    no_substitute: bool,
    substituters: Option<&str>,
    signing_key_selected: bool,
    trusted_public_key_count: usize,
    trust_unsigned: bool,
) {
    modes.hermeticity_mode = Some(hermeticity_mode.to_string());
    modes.substitution_mode = Some(substitution_mode_label(no_substitute).to_string());
    modes.substituter_count = Some(substituter_count_for_mode(no_substitute, substituters));
    modes.signing_key_selected = Some(signing_key_selected);
    modes.trusted_public_key_count = Some(operator_diagnostics::bounded_runtime_count(trusted_public_key_count));
    modes.trust_unsigned = Some(trust_unsigned);
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
        StoreAction::Sign { signing_key, .. } => modes.signing_key_selected = Some(signing_key.is_some()),
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
    match &args.command {
        Command::Doctor { profile } => run_doctor_command(ctx, *profile),
        Command::Graph { root, graph_file } => run_semantic_graph_command(ctx, "graph", root, graph_file.as_deref()),
        Command::Why { target, graph_file } => run_semantic_graph_command(ctx, "why", target, graph_file.as_deref()),
        Command::Dependents { target, graph_file } => {
            run_semantic_graph_command(ctx, "dependents", target, graph_file.as_deref())
        }
        Command::Refactor { action } => run_refactor_command(ctx, action.clone()),
        Command::Transcript { action } => run_transcript_command(action.clone()),
        Command::Stage0Inventory { output } => run_stage0_inventory_command(ctx, output),
        Command::Eval { file, import_paths } => run_eval(file, import_paths),
        Command::Build { .. } => run_build_from_command(ctx, &args.command),
        Command::Bootstrap { .. } => run_bootstrap_from_command(ctx, &args.command),
        Command::Release { action } => run_release_command(ctx, action.clone()),
        Command::Log { query, list } => log_cmd::cmd_log(query.as_deref(), *list),
        Command::Store { action } => {
            store_cmd::cmd_store(action.clone(), &ctx.store, &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json)
        }
        Command::Source { action } => {
            source_bundle::cmd_source(action.clone(), &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json)
        }
        Command::Receipt { action } => {
            portable_receipt::cmd_receipt(action.clone(), &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json)
        }
        Command::Remote { action } => {
            remote_build::cmd_remote(action.clone(), &ctx.store, &ctx.resolved_state_dir, &ctx.store_prefix, ctx.json)
        }
        Command::Artifact { action } => {
            artifact_cmd::cmd_artifact(action.clone(), &current_dir_or_error()?, &ctx.resolved_state_dir, ctx.json)
        }
        Command::Attest { action } => run_attest_command(ctx, action.clone()),
        Command::Import { action } => run_import_command(ctx, action.clone()),
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

fn run_semantic_graph_command(
    ctx: &RunContext,
    query: &str,
    target: &str,
    graph_file: Option<&Path>,
) -> Result<(), RunError> {
    let path = graph_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| ctx.resolved_state_dir.join("semantic-graph.json"));
    let graph = match semantic_graph::SemanticGraph::load(&path) {
        Ok(graph) => graph,
        Err(semantic_graph::SemanticGraphError::Incomplete(diag)) => {
            if ctx.json {
                let rendered =
                    serde_json::to_string_pretty(&diag).map_err(|err| RunError::Internal(err.to_string()))?;
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
        Err(err) => return Err(RunError::Internal(err.to_string())),
    };

    let rendered = match query {
        "graph" => match graph.graph_for_root(target) {
            Ok(result) => render_graph_result(ctx.json, &result)?,
            Err(err) => return report_semantic_graph_query_error(ctx, err),
        },
        "why" => match graph.why(target) {
            Ok(result) => render_why_result(ctx.json, &result)?,
            Err(err) => return report_semantic_graph_query_error(ctx, err),
        },
        "dependents" => match graph.dependents(target) {
            Ok(result) => render_dependents_result(ctx.json, &result)?,
            Err(err) => return report_semantic_graph_query_error(ctx, err),
        },
        _ => return Err(RunError::Internal(format!("unknown semantic graph query `{query}`"))),
    };
    println!("{rendered}");
    Ok(())
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
    let report = operator_diagnostics::collect_doctor_report(operator_diagnostics::DoctorRequest {
        profile,
        store_dir: &ctx.store,
        state_dir: &ctx.resolved_state_dir,
    });

    if ctx.json {
        let rendered =
            report.render_json().map_err(|e| RunError::Internal(format!("serializing doctor report: {e}")))?;
        println!("{rendered}");
    } else {
        let rendered = report.render_human();
        if report.ok {
            println!("{rendered}");
        } else {
            eprintln!("{rendered}");
        }
    }

    if report.ok {
        return Ok(());
    }
    Err(RunError::Reported(3))
}

fn run_stage0_inventory_command(ctx: &RunContext, output: &Path) -> Result<(), RunError> {
    let output_path = if output.is_absolute() {
        output.to_path_buf()
    } else {
        current_dir_or_error()?.join(output)
    };
    let inventory = protected_exec::write_generated_stage0_inventory_from_env(&output_path)
        .map_err(|err| RunError::Build(format!("generating stage0 inventory from explicit seed paths: {err}")))?;
    let risks = protected_exec::seed_closure_risk_report(&inventory);
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
            "executable_entries": inventory.executable_entries.len(),
            "source_entries": inventory.source_entries.len(),
            "closure_risks": risk_json,
            "seed_input_policy": "explicit CRUNCH_STAGE0_SEED_* paths only; no PATH or /nix/store discovery",
        });
        println!("{}", serde_json::to_string_pretty(&rendered).map_err(|err| RunError::Internal(err.to_string()))?);
        return Ok(());
    }
    println!("stage0 inventory written: {}", output_path.display());
    println!("seed input policy: explicit CRUNCH_STAGE0_SEED_* paths only; no PATH or /nix/store discovery");
    println!("executable seed entries: {}", inventory.executable_entries.len());
    println!("source seed entries: {}", inventory.source_entries.len());
    for risk in risks {
        println!("seed closure risk: id={} path={} risk={}", risk.entry_id, risk.executable_path.display(), risk.risk,);
    }
    Ok(())
}

fn run_eval(file: &Path, import_paths: &[PathBuf]) -> Result<(), RunError> {
    let import_paths = build_import_paths(import_paths)?;
    let json = crunch_eval::evaluate_to_json(file, &import_paths).map_err(|e| RunError::Eval(format!("{e}")))?;
    println!("{json}");
    Ok(())
}

fn run_build_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Build {
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
        } => run_build_command(
            ctx,
            file.as_ref(),
            import_paths,
            *fix,
            *plan,
            *offline_source_preflight,
            *jobs,
            substituters,
            *no_substitute,
            signing_key.as_deref(),
            trusted_public_keys,
            *trust_unsigned,
            *strict_hermetic,
            *impure,
            builder.as_deref(),
            ticket.as_deref(),
            builder_program.as_deref(),
            builder_args,
            trusted_builder_keys,
            *remote_build_time_secs,
        ),
        _ => unreachable!("build helper called with non-build command"),
    }
}

#[allow(clippy::too_many_arguments)]
fn select_hermeticity_mode(strict_hermetic: bool, impure: bool) -> Result<crunch_pipeline::HermeticityMode, RunError> {
    match (strict_hermetic, impure) {
        (true, true) => Err(RunError::Internal("--strict-hermetic and --impure are mutually exclusive".to_string())),
        (true, false) => Ok(crunch_pipeline::HermeticityMode::Strict),
        (false, true) => Ok(crunch_pipeline::HermeticityMode::Impure),
        (false, false) => Ok(crunch_pipeline::HermeticityMode::Practical),
    }
}

fn run_build_command(
    ctx: &RunContext,
    file: Option<&PathBuf>,
    import_paths: &[PathBuf],
    fix: bool,
    plan: bool,
    offline_source_preflight: bool,
    jobs: Option<u32>,
    substituters: &str,
    no_substitute: bool,
    signing_key: Option<&std::path::Path>,
    trusted_public_keys: &[String],
    trust_unsigned: bool,
    strict_hermetic: bool,
    impure: bool,
    remote_builder: Option<&str>,
    remote_ticket: Option<&str>,
    remote_builder_program: Option<&Path>,
    remote_builder_args: &[String],
    trusted_builder_keys: &[String],
    remote_build_time_secs: u64,
) -> Result<(), RunError> {
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let substituter_url = (!no_substitute).then_some(substituters);
    let hermeticity_mode = select_hermeticity_mode(strict_hermetic, impure)?;
    let parsed_trusted = parse_trusted_keys(trusted_public_keys)?;
    let remote_selection = remote_build_selection(
        remote_builder,
        remote_ticket,
        remote_builder_program,
        remote_builder_args,
        trusted_builder_keys,
        remote_build_time_secs,
        ctx,
    )?;
    if remote_selection.is_some() && fix {
        return Err(RunError::Internal("remote build dispatch does not support --fix yet".to_string()));
    }
    if remote_selection.is_some() && plan {
        return Err(RunError::Internal("remote build dispatch does not support --plan yet".to_string()));
    }
    let target = project_build::parse_build_target(file.map(PathBuf::as_path));
    match target {
        project_build::BuildTarget::File(path) => {
            let import_paths = build_import_paths(import_paths)?;
            run_offline_source_preflight_if_requested(
                offline_source_preflight,
                &path,
                &import_paths,
                &ctx.resolved_state_dir,
                &ctx.store_prefix,
                ctx.output_mode(),
            )?;
            if let Some(remote_selection) = &remote_selection {
                return run_remote_build_file_command(
                    remote_selection,
                    &path,
                    &import_paths,
                    &ctx.store,
                    &ctx.resolved_state_dir,
                    &ctx.store_prefix,
                    ctx.output_mode(),
                );
            }
            if plan {
                return build_plan::cmd_build_plan(build_plan::BuildPlanConfig {
                    file: &path,
                    import_paths: &import_paths,
                    output_dir: &ctx.store,
                    state_dir: &ctx.resolved_state_dir,
                    store_dir: &ctx.store_prefix,
                    substituter_url,
                    signing_key_path: signing_key,
                    trusted_public_keys: parsed_trusted.as_deref(),
                    trust_unsigned,
                    output_mode: ctx.output_mode(),
                });
            }
            cmd_build(
                &path,
                &import_paths,
                &ctx.store,
                &ctx.resolved_state_dir,
                &ctx.store_prefix,
                ctx.verbose,
                fix,
                max_jobs,
                substituter_url,
                signing_key,
                parsed_trusted.as_deref(),
                trust_unsigned,
                hermeticity_mode,
                ctx.output_mode(),
            )
        }
        project_build::BuildTarget::ProjectDefault | project_build::BuildTarget::Selector(_) => {
            let cwd = current_dir_or_error()?;
            let resolved = project_build::resolve_project_target(&target, &cwd, import_paths)?;
            let expr = project_build::generate_extraction_expr(&resolved.root_file, &resolved.target);
            let mut full_import_paths = build_import_paths(&[])?;
            full_import_paths.extend(resolved.import_paths);
            run_offline_source_preflight_for_expr_if_requested(
                offline_source_preflight,
                &expr,
                &full_import_paths,
                &ctx.resolved_state_dir,
                &ctx.store_prefix,
                ctx.output_mode(),
            )?;
            if let Some(remote_selection) = &remote_selection {
                return run_remote_build_expr_command(
                    remote_selection,
                    &expr,
                    &full_import_paths,
                    &ctx.store,
                    &ctx.resolved_state_dir,
                    &ctx.store_prefix,
                    ctx.output_mode(),
                );
            }
            if plan {
                return build_plan_from_expr(
                    &expr,
                    &full_import_paths,
                    &ctx.store,
                    &ctx.resolved_state_dir,
                    &ctx.store_prefix,
                    substituter_url,
                    signing_key,
                    parsed_trusted.as_deref(),
                    trust_unsigned,
                    ctx.output_mode(),
                );
            }
            build_from_expr(
                &expr,
                &full_import_paths,
                &ctx.store,
                &ctx.resolved_state_dir,
                &ctx.store_prefix,
                ctx.verbose,
                fix,
                max_jobs,
                substituter_url,
                signing_key,
                parsed_trusted.as_deref(),
                trust_unsigned,
                hermeticity_mode,
                ctx.output_mode(),
            )
        }
    }
}

struct RemoteBuildSelection {
    options: remote_build::RemoteClientBuildOptions,
}

#[allow(clippy::too_many_arguments)]
fn remote_build_selection(
    builder: Option<&str>,
    ticket: Option<&str>,
    builder_program: Option<&Path>,
    builder_args: &[String],
    trusted_builder_keys: &[String],
    build_time_limit_secs: u64,
    ctx: &RunContext,
) -> Result<Option<RemoteBuildSelection>, RunError> {
    if builder.is_none() && ticket.is_none() && builder_program.is_none() && builder_args.is_empty() {
        return Ok(None);
    }
    let builder = builder.ok_or_else(|| RunError::Internal("remote build dispatch requires --builder".to_string()))?;
    let ticket = ticket.ok_or_else(|| RunError::Internal("remote build dispatch requires --ticket".to_string()))?;
    let ticket = remote_build::parse_remote_ticket_credential(ticket).map_err(RunError::Internal)?;
    let (program, args) = remote_stdio_builder_command(builder, builder_program, builder_args, ctx)?;
    let trusted_output_keys = remote_trusted_builder_keys(trusted_builder_keys, builder_program, ctx)?;
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
        build_time_limit_secs,
        client_endpoint: None,
        transfer_capabilities: remote_build::RemoteTransferCapabilities::delta_and_full(),
    };
    Ok(Some(RemoteBuildSelection { options }))
}

fn remote_stdio_builder_command(
    builder: &str,
    builder_program: Option<&Path>,
    builder_args: &[String],
    ctx: &RunContext,
) -> Result<(PathBuf, Vec<String>), RunError> {
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
    let args = vec![
        "--state-dir".to_string(),
        ctx.resolved_state_dir.display().to_string(),
        "--store".to_string(),
        ctx.store.display().to_string(),
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
    ];
    Ok((program, args))
}

fn remote_trusted_builder_keys(
    trusted_builder_keys: &[String],
    builder_program: Option<&Path>,
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
    let keypair = build_cmd::load_or_generate_signing_keypair(None, &ctx.resolved_state_dir, false)?;
    Ok(vec![keypair.verifying_key.name().to_string()])
}

fn unix_time_now_s() -> Result<u64, RunError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RunError::Internal(format!("system clock before unix epoch: {err}")))?;
    Ok(duration.as_secs())
}

fn run_remote_build_file_command(
    selection: &RemoteBuildSelection,
    file: &Path,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    let inputs = evaluate_remote_client_derivation_inputs(file, import_paths, store_prefix)?;
    let report = run_remote_build_dispatches(selection, inputs, output_dir, state_dir, store_prefix)?;
    print_remote_client_build_report(&report, file, output_dir, state_dir, output_mode)
}

fn run_remote_build_expr_command(
    selection: &RemoteBuildSelection,
    expr: &str,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|err| RunError::Internal(format!("creating temp file: {err}")))?;
    std::fs::write(tmp.path(), expr).map_err(|err| RunError::Internal(format!("writing temp file: {err}")))?;
    run_remote_build_file_command(selection, tmp.path(), import_paths, output_dir, state_dir, store_prefix, output_mode)
}

fn evaluate_remote_client_derivation_inputs(
    file: &Path,
    import_paths: &[OsString],
    store_prefix: &str,
) -> Result<Vec<remote_build::RemoteClientDerivationInput>, RunError> {
    let mut session = crunch_eval::session::EvaluationSession::open_file(file, import_paths)
        .map_err(|err| RunError::Eval(format!("{err}")))?;
    let derivations = session
        .force_all_roots::<crunch_glue::CrunchDerivation>()
        .map_err(|err| RunError::Eval(format!("{err}")))?;
    let mut cache = crunch_glue::ConversionCache::new(store_prefix);
    let mut inputs = Vec::with_capacity(derivations.len());
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

async fn run_remote_build_dispatches_async(
    selection: &RemoteBuildSelection,
    inputs: Vec<remote_build::RemoteClientDerivationInput>,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<remote_build::RemoteClientBuildReport, RunError> {
    let mut store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_url: None,
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_prefix.to_string(),
    })
    .await
    .map_err(|err| RunError::Internal(format!("opening local store for remote build dispatch: {err}")))?;
    let mut imported = Vec::with_capacity(inputs.len());
    for input in inputs {
        let mut plan =
            remote_build::plan_remote_stdio_client_dispatch(input, &selection.options).map_err(RunError::Internal)?;
        remote_build::populate_remote_input_upload_artifacts_from_store_or_source_state(
            &store,
            &mut plan.command,
            &plan.client.request,
            state_dir,
        )
        .await
        .map_err(|err| RunError::Internal(format!("remote input upload preparation failed: {err}")))?;
        let transcript = remote_build::run_stdio_remote_child(&plan.command)?;
        let admission = remote_build::validate_remote_builder_frames_output_import(
            &plan.client.request,
            &plan.client.trusted_output_keys,
            &transcript.frames,
        )
        .map_err(|err| RunError::Internal(format!("remote output import admission failed: {err}")))?;
        let import_report = remote_build::import_admitted_remote_outputs(
            &mut store,
            &plan.client.request,
            &admission,
            true,
            Some(crunch_store::GcRootSource::Build),
        )
        .await
        .map_err(|err| RunError::Internal(format!("remote output import failed: {err}")))?;
        imported.push(remote_build::RemoteClientImportedBuild {
            label: plan.label,
            request_id: plan.client.request.request_id,
            imported: import_report,
        });
    }
    Ok(remote_build::RemoteClientBuildReport {
        schema: REMOTE_CLIENT_BUILD_REPORT_SCHEMA.to_string(),
        builder: selection.options.builder.endpoint_id.clone(),
        store_prefix: store_prefix.to_string(),
        imported,
    })
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
            let json_report = remote_client_build_json_report(report, file, output_dir, state_dir)?;
            let rendered = serde_json::to_string_pretty(&json_report)
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
    let outcomes = report
        .imported
        .iter()
        .map(|build| remote_client_build_outcome_json(report, build, output_dir))
        .collect::<Result<Vec<_>, _>>()?;
    let succeeded_total = remote_report_count(outcomes.len())?;
    Ok(serde_json::json!({
        "schema": BUILD_JSON_REPORT_SCHEMA,
        "file": file.display().to_string(),
        "output_dir": output_dir.display().to_string(),
        "state_dir": state_dir.display().to_string(),
        "store_dir": &report.store_prefix,
        "hermeticity_mode": REMOTE_BUILD_HERMETICITY_MODE,
        "hermeticity_audit_events": [],
        "native_dynamic_plans": [],
        "frontend_artifact_attestations": [],
        "cargo_build_evidence": [],
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
        "path": remote_client_output_path(&output.logical_path, &report.store_prefix, output_dir)?,
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

fn remote_client_output_path(logical_path: &str, store_prefix: &str, output_dir: &Path) -> Result<String, RunError> {
    let suffix = logical_path
        .strip_prefix(store_prefix)
        .ok_or_else(|| RunError::Internal("remote output path has wrong store prefix".to_string()))?;
    let suffix = suffix.strip_prefix('/').unwrap_or(suffix);
    Ok(output_dir.join(suffix).display().to_string())
}

fn remote_report_count(count: usize) -> Result<u32, RunError> {
    u32::try_from(count).map_err(|_| RunError::Internal("remote build report count overflow".to_string()))
}

fn run_offline_source_preflight_if_requested(
    enabled: bool,
    file: &Path,
    import_paths: &[std::ffi::OsString],
    state_dir: &Path,
    store_prefix: &str,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    if !enabled {
        return Ok(());
    }
    let report = source_bundle::offline_preflight_for_file(file, import_paths, state_dir, store_prefix)?;
    if source_bundle::source_offline_preflight_is_ready(&report) {
        return Ok(());
    }
    source_bundle::print_offline_preflight_report(&report, output_mode == BuildOutputMode::Json)?;
    Err(RunError::Reported(1))
}

fn run_offline_source_preflight_for_expr_if_requested(
    enabled: bool,
    expr: &str,
    import_paths: &[std::ffi::OsString],
    state_dir: &Path,
    store_prefix: &str,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    if !enabled {
        return Ok(());
    }
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;
    run_offline_source_preflight_if_requested(true, tmp.path(), import_paths, state_dir, store_prefix, output_mode)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRootManifestCheck {
    manifest: bootstrap_source_root::SourceRootManifest,
    manifest_bytes: Vec<u8>,
    manifest_digest: String,
    expected_output_role_count: u32,
}

fn validate_source_root_manifest_file(manifest_path: &Path) -> Result<SourceRootManifestCheck, RunError> {
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
    if value.len() != blake3::OUT_LEN * HEX_CHARS_PER_BYTE {
        return Err(RunError::Internal(format!(
            "source-root provider output digest must be {} lowercase hex chars",
            blake3::OUT_LEN * HEX_CHARS_PER_BYTE
        )));
    }
    let mut bytes = [0u8; blake3::OUT_LEN];
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let start = idx * HEX_CHARS_PER_BYTE;
        let end = start + HEX_CHARS_PER_BYTE;
        let pair = &value[start..end];
        if !pair.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
            return Err(RunError::Internal("source-root provider output digest must be lowercase hex".to_string()));
        }
        *byte = u8::from_str_radix(pair, HEX_RADIX).map_err(|err| {
            RunError::Internal(format!("parsing source-root provider output digest byte {idx}: {err}"))
        })?;
    }
    Ok(bytes)
}

fn load_stage0_inventory_policy(path: &Path) -> Result<protected_exec::ProtectedExecPolicy, RunError> {
    let import_paths: Vec<OsString> = Vec::new();
    let inventory: protected_exec::Stage0Inventory = crunch_eval::evaluate_and_deserialize(path, &import_paths)
        .map_err(|e| RunError::Eval(format!("loading stage0 inventory {}: {e}", path.display())))?;
    protected_exec::ProtectedExecPolicy::from_inventory(inventory)
        .map_err(|e| RunError::Internal(format!("validating stage0 inventory {}: {e}", path.display())))
}

fn run_bootstrap_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Bootstrap {
            action,
            output,
            fetch,
            source_root,
            stagex_lineage,
            packages,
        } => {
            if let Some(action) = action {
                return run_bootstrap_action(ctx, action);
            }
            run_bootstrap_command(ctx, output, *fetch, source_root.as_deref(), stagex_lineage.as_deref(), packages)
        }
        _ => unreachable!("bootstrap helper called with non-bootstrap command"),
    }
}

fn run_bootstrap_action(ctx: &RunContext, action: &BootstrapAction) -> Result<(), RunError> {
    match action {
        BootstrapAction::ParityReport { require } => {
            bootstrap_parity::cmd_bootstrap_parity_report(&current_dir_or_error()?, require, ctx.json)
        }
        BootstrapAction::RustSourceProvider {
            recipe,
            route_plan,
            import_dir,
            smoke,
            smoke_evidence_dir,
            output_dir,
        } => cmd_bootstrap_rust_source_provider(
            recipe,
            route_plan.as_deref(),
            import_dir.as_deref(),
            output_dir,
            &ctx.store,
            ctx.verbose,
            *smoke,
            smoke_evidence_dir.as_deref(),
        ),
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

fn cmd_bootstrap_rust_source_provider(
    recipe: &Path,
    route_plan: Option<&Path>,
    import_dir: Option<&Path>,
    output_dir: &Path,
    _store_dir: &Path,
    verbose: bool,
    smoke: bool,
    smoke_evidence_dir: Option<&Path>,
) -> Result<(), RunError> {
    if let Some(import_dir) = import_dir {
        return cmd_import_rust_source_provider(import_dir, output_dir, smoke, smoke_evidence_dir);
    }
    let scratch = tempfile::Builder::new()
        .prefix("mantle-rust-source-provider-")
        .tempdir()
        .map_err(|err| RunError::Internal(format!("creating Rust provider scratch: {err}")))?;
    match rust_source_provider::materialize_rust_source_provider_with_route_plan(
        recipe,
        route_plan,
        output_dir,
        scratch.path(),
        verbose,
    ) {
        Ok(materialized) => {
            eprintln!("Materialized Rust source provider {}", materialized.output_path.display());
            eprintln!("  recipe_digest_blake3: {}", materialized.recipe_digest_blake3);
            eprintln!("  metadata_path: {}", materialized.metadata_path.display());
            eprintln!("  metadata_digest_blake3: {}", materialized.metadata_digest_blake3);
            if smoke {
                cmd_smoke_rust_source_provider(&materialized.output_path, smoke_evidence_dir)?;
            }
            Ok(())
        }
        Err(err) => {
            let preserved_scratch = preserve_rust_source_provider_scratch(scratch);
            Err(RunError::Build(format!(
                "Rust source provider materialization failed closed: {err} preserved_scratch={}",
                preserved_scratch.display()
            )))
        }
    }
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
    let imported = rust_source_provider::import_rust_source_provider(import_dir, output_dir)
        .map_err(|err| RunError::Build(format!("Rust source provider import failed closed: {err}")))?;
    eprintln!("Imported Rust source provider {}", imported.output_path.display());
    eprintln!("  input: {}", imported.input_path.display());
    eprintln!("  metadata_path: {}", imported.validation.metadata_path.display());
    eprintln!("  metadata_digest_blake3: {}", imported.validation.metadata_digest_blake3);
    eprintln!("  policy_digest_blake3: {}", imported.validation.validation.policy_digest_blake3);
    if smoke {
        cmd_smoke_rust_source_provider(&imported.output_path, smoke_evidence_dir)?;
    }
    Ok(())
}

fn cmd_smoke_rust_source_provider(provider_dir: &Path, evidence_dir: Option<&Path>) -> Result<(), RunError> {
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

fn run_bootstrap_command(
    ctx: &RunContext,
    output: &Path,
    fetch: bool,
    source_root: Option<&Path>,
    stagex_lineage: Option<&Path>,
    packages: &[String],
) -> Result<(), RunError> {
    match bootstrap_source_root::select_bootstrap_provider(fetch, source_root, stagex_lineage)
        .map_err(|err| RunError::Internal(err.to_string()))?
    {
        bootstrap_source_root::BootstrapProviderMode::LegacyFetch => {
            cmd_bootstrap_fetch(output, &ctx.store, ctx.verbose)
        }
        bootstrap_source_root::BootstrapProviderMode::SourceRoot => {
            let manifest_path = source_root.expect("source-root mode must carry manifest path");
            bootstrap_source_root_provider(output, manifest_path, &ctx.store, ctx.verbose)
        }
        bootstrap_source_root::BootstrapProviderMode::StagexLineage => {
            let manifest_path = stagex_lineage.expect("stagex-lineage mode must carry manifest path");
            cmd_bootstrap_stagex_lineage(output, manifest_path)
        }
        bootstrap_source_root::BootstrapProviderMode::NixPackages => cmd_bootstrap(output, packages),
    }
}

fn cmd_bootstrap_stagex_lineage(output: &Path, manifest_path: &Path) -> Result<(), RunError> {
    let manifest_bytes = std::fs::read(manifest_path)
        .map_err(|e| RunError::Internal(format!("reading {}: {e}", manifest_path.display())))?;
    let manifest = bootstrap_source_root::validate_stagex_lineage_manifest(&manifest_bytes)
        .map_err(|diags| RunError::Internal(bootstrap_source_root::format_diagnostics(&diags)))?;

    let evidence = bootstrap_source_root::classify_stagex_provider_evidence(&manifest);
    eprintln!("StageX lineage manifest validated:");
    eprintln!("  seed_class: {}", evidence.seed_class);
    eprintln!("  audit_seed_max_bytes: {}", evidence.audit_seed_max_bytes);
    eprintln!("  provider_outputs: {}", evidence.provider_output_count);
    eprintln!("  environment_assumptions: {}", evidence.environment_assumptions.len());

    Err(RunError::Internal(format!(
        "{}: output={}",
        bootstrap_source_root::STAGEX_LINEAGE_PROVIDER_NOT_MATERIALIZED,
        output.display()
    )))
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
        _ => unreachable!("project command helper called with non-project command"),
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

fn run_rust_plan_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
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
    let selected_c_compiler_route =
        rust_plan::receipt_bound_c_compiler_route_from_cli_json(source_built_c_compiler_route_json.as_deref())?;
    let path_remaps = if *deterministic_release_paths {
        rust_plan::deterministic_release_path_remaps_with_c_compiler(
            &root,
            execution_output_root.as_deref(),
            selected_c_compiler_route.as_ref(),
        )
    } else {
        Vec::new()
    };
    let compiler_policy = rust_plan::rust_compiler_policy_selection(
        compiler_policy_mode,
        compiler_policy_provider_manifest.clone(),
        compiler_policy_provider_manifest_digest.clone(),
    )?;
    let options = rust_plan::RustPlanOptions {
        root,
        cargo: cargo.clone(),
        rustc: rustc.clone(),
        targets: targets.clone(),
        profile: profile.clone(),
        features: features.clone(),
        all_features: *all_features,
        no_default_features: *no_default_features,
        no_cargo_oracle: *no_cargo_oracle,
        deterministic_release_paths: *deterministic_release_paths,
        path_remaps,
    };
    let receipt = rust_plan::capture_rust_plan(&options)?;
    rust_plan::set_receipt_bound_c_compiler_route_override(selected_c_compiler_route)?;
    let execution_options = |output_root: PathBuf| rust_plan::RustUnitExecutionOptions {
        rustc: rustc.clone(),
        output_root,
        compiler_policy: compiler_policy.clone(),
    };
    if *execute_first_supported_unit {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-first-supported-unit requires --execution-output-root".to_string())
        })?;
        let unit_execution = rust_plan::execute_first_supported_rust_unit(
            &receipt.unit_derivation_graph,
            &execution_options(output_root),
        )?;
        return rust_plan::print_rust_plan_execution_receipt(
            &rust_plan::RustPlanExecutionReceipt {
                rust_plan: receipt,
                unit_execution,
            },
            ctx.json,
        );
    }
    if *execute_first_dependency_chain {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-first-dependency-chain requires --execution-output-root".to_string())
        })?;
        let dependency_chain_execution = rust_plan::execute_first_rust_unit_dependency_chain(
            &receipt.unit_derivation_graph,
            &execution_options(output_root),
        )?;
        return rust_plan::print_rust_plan_dependency_chain_execution_receipt(
            &rust_plan::RustPlanDependencyChainExecutionReceipt {
                rust_plan: receipt,
                dependency_chain_execution,
            },
            ctx.json,
        );
    }
    if *execute_target_topology {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-target-topology requires --execution-output-root".to_string())
        })?;
        let target_topology_execution = rust_plan::execute_rust_target_unit_topology(
            &receipt.unit_derivation_graph,
            &execution_options(output_root),
        )?;
        return rust_plan::print_rust_plan_target_topology_execution_receipt(
            &rust_plan::RustPlanTargetTopologyExecutionReceipt {
                rust_plan: receipt,
                target_topology_execution,
            },
            ctx.json,
        );
    }
    if *execute_host_artifact_topology {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-host-artifact-topology requires --execution-output-root".to_string())
        })?;
        let host_artifact_topology_execution = rust_plan::execute_rust_host_artifact_topology(
            &receipt.native_registry_source_planning,
            &receipt.native_host_unit_graph_planning,
            &receipt.unit_derivation_graph,
            &execution_options(output_root),
        )?;
        return rust_plan::print_rust_plan_host_artifact_topology_execution_receipt(
            &rust_plan::RustPlanHostArtifactTopologyExecutionReceipt {
                rust_plan: receipt,
                host_artifact_topology_execution,
            },
            ctx.json,
        );
    }
    if *execute_topology {
        let output_root = execution_output_root
            .clone()
            .ok_or_else(|| RunError::Internal("--execute-topology requires --execution-output-root".to_string()))?;
        let topology_execution = rust_plan::execute_rust_unit_topology(
            &receipt.native_registry_source_planning,
            &receipt.native_host_unit_graph_planning,
            &receipt.unit_derivation_graph,
            &execution_options(output_root),
        )?;
        return rust_plan::print_rust_plan_topology_execution_receipt(
            &rust_plan::RustPlanTopologyExecutionReceipt {
                rust_plan: receipt,
                topology_execution,
            },
            ctx.json,
        );
    }
    if *execute_dev_dependency_test_topology {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-dev-dependency-test-topology requires --execution-output-root".to_string())
        })?;
        let native_rust_dev_dependency_test_topology_execution =
            rust_plan::execute_native_rust_dev_dependency_test_topology(
                &receipt.native_package_target_planning,
                &receipt.unit_derivation_graph,
                &execution_options(output_root),
            )?;
        return rust_plan::print_rust_plan_dev_dependency_test_topology_execution_receipt(
            &rust_plan::RustPlanDevDependencyTestTopologyExecutionReceipt {
                rust_plan: receipt,
                native_rust_dev_dependency_test_topology_execution,
            },
            ctx.json,
        );
    }
    if *execute_workspace_dependency_topology {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-workspace-dependency-topology requires --execution-output-root".to_string())
        })?;
        let native_registry_workspace_dependency_topology_execution =
            rust_plan::execute_native_registry_workspace_dependency_topology(
                &receipt.native_package_target_planning,
                &receipt.unit_derivation_graph,
                &execution_options(output_root),
            )?;
        return rust_plan::print_rust_plan_workspace_dependency_topology_execution_receipt(
            &rust_plan::RustPlanWorkspaceDependencyTopologyExecutionReceipt {
                rust_plan: receipt,
                native_registry_workspace_dependency_topology_execution,
            },
            ctx.json,
        );
    }
    if *execute_patch_source_topology {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-patch-source-topology requires --execution-output-root".to_string())
        })?;
        let native_registry_patch_source_topology_execution = rust_plan::execute_native_registry_patch_source_topology(
            &receipt.native_registry_source_planning,
            &receipt.native_package_target_planning,
            &receipt.unit_derivation_graph,
            &execution_options(output_root),
        )?;
        return rust_plan::print_rust_plan_patch_source_topology_execution_receipt(
            &rust_plan::RustPlanPatchSourceTopologyExecutionReceipt {
                rust_plan: receipt,
                native_registry_patch_source_topology_execution,
            },
            ctx.json,
        );
    }
    rust_plan::print_rust_plan_receipt(&receipt, ctx.json)
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
        _ => unreachable!("shell helper called with non-shell command"),
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
        } => run_develop_command(
            ctx,
            name.as_deref(),
            import_paths,
            *jobs,
            *no_substitute,
            signing_key.as_deref(),
            *trust_unsigned,
        ),
        _ => unreachable!("develop helper called with non-develop command"),
    }
}

fn run_develop_command(
    ctx: &RunContext,
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&std::path::Path>,
    trust_unsigned: bool,
) -> Result<(), RunError> {
    shell_cmd::cmd_shell(
        name,
        import_paths,
        jobs,
        no_substitute,
        signing_key,
        trust_unsigned,
        &[],
        None,
        &[],
        false,
        false,
        &ctx.store,
        &ctx.resolved_state_dir,
        &ctx.store_prefix,
        ctx.verbose,
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
        } => run_package_command(
            ctx,
            name.as_deref(),
            import_paths,
            *jobs,
            *no_substitute,
            signing_key.as_deref(),
            *trust_unsigned,
            bin.as_deref(),
            run_args,
        ),
        _ => unreachable!("run helper called with non-run command"),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_package_command(
    ctx: &RunContext,
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&std::path::Path>,
    trust_unsigned: bool,
    bin: Option<&str>,
    run_args: &[String],
) -> Result<(), RunError> {
    cmd_run(
        name,
        import_paths,
        jobs,
        no_substitute,
        signing_key,
        trust_unsigned,
        bin,
        run_args,
        &ctx.store,
        &ctx.resolved_state_dir,
        &ctx.store_prefix,
        ctx.verbose,
    )
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
            source_root,
            no_host_tools,
            stage0_inventory,
            source_store_path,
            bootstrap_bwrap_path,
            bootstrap_busybox_path,
            cargo_free,
            fixed_point,
            out,
            rustc,
            toolchain_closure,
            rust_source_provider,
            targets,
        } => run_self_build_command(
            ctx,
            *jobs,
            *no_substitute,
            *no_verify,
            signing_key.as_deref(),
            trusted_public_keys,
            *trust_unsigned,
            *strict_hermetic,
            *impure,
            source_root.as_deref(),
            *no_host_tools,
            stage0_inventory.as_deref(),
            source_store_path.as_deref(),
            bootstrap_bwrap_path.as_deref(),
            bootstrap_busybox_path.as_deref(),
            *cargo_free,
            *fixed_point,
            out.as_deref(),
            rustc,
            toolchain_closure.as_deref(),
            rust_source_provider.as_deref(),
            targets,
        ),
        _ => unreachable!("self-build helper called with non-self-build command"),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_self_build_command(
    ctx: &RunContext,
    jobs: Option<u32>,
    no_substitute: bool,
    no_verify: bool,
    signing_key: Option<&std::path::Path>,
    trusted_public_keys: &[String],
    trust_unsigned: bool,
    strict_hermetic: bool,
    impure: bool,
    source_root: Option<&std::path::Path>,
    no_host_tools: bool,
    stage0_inventory: Option<&std::path::Path>,
    source_store_path: Option<&std::path::Path>,
    bootstrap_bwrap_path: Option<&std::path::Path>,
    bootstrap_busybox_path: Option<&std::path::Path>,
    cargo_free: bool,
    fixed_point: bool,
    out: Option<&std::path::Path>,
    rustc: &std::path::Path,
    toolchain_closure: Option<&std::path::Path>,
    rust_source_provider: Option<&std::path::Path>,
    targets: &[String],
) -> Result<(), RunError> {
    if cargo_free {
        validate_cargo_free_legacy_self_build_args(
            jobs,
            no_substitute,
            no_verify,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
            impure,
            source_root,
            no_host_tools,
            stage0_inventory,
            source_store_path,
            bootstrap_bwrap_path,
            bootstrap_busybox_path,
        )?;
        let out_dir = out.ok_or_else(|| RunError::Build("--cargo-free requires --out <dir>".to_string()))?;
        let root = current_dir_or_error()?;
        let options = cargo_free_self_build::CargoFreeSelfBuildOptions {
            root: &root,
            out_dir,
            rustc,
            targets,
            toolchain_closure,
            rust_source_provider,
            json: ctx.json,
        };
        if fixed_point {
            return cargo_free_self_build::cmd_cargo_free_fixed_point_self_build(options);
        }
        return cargo_free_self_build::cmd_cargo_free_self_build(options);
    }

    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let hermeticity_mode = select_hermeticity_mode(strict_hermetic, impure)?;
    if let Some(manifest_path) = source_root {
        let checked = validate_source_root_manifest_file(manifest_path)?;
        return Err(RunError::Build(format!(
            "{}: manifest_digest={} expected_output_roles={}",
            bootstrap_source_root::SOURCE_ROOT_BLOCKED_REASON,
            checked.manifest_digest,
            checked.expected_output_role_count
        )));
    }
    validate_stage0_inventory_args(no_host_tools, stage0_inventory)?;
    let stage0_policy = stage0_inventory.map(load_stage0_inventory_policy).transpose()?;

    let parsed_trusted = parse_trusted_keys(trusted_public_keys)?;
    self_build::cmd_self_build(
        &ctx.store,
        &ctx.resolved_state_dir,
        &ctx.store_prefix,
        ctx.verbose,
        max_jobs,
        no_substitute,
        no_verify,
        signing_key,
        parsed_trusted.as_deref(),
        trust_unsigned,
        hermeticity_mode,
        source_store_path,
        stage0_policy.as_ref(),
        bootstrap_bwrap_path,
        bootstrap_busybox_path,
        bootstrap_source_root::BootstrapProviderMode::LegacyFetch,
    )
    .map(|_report| ())
}

#[allow(clippy::too_many_arguments)]
fn validate_cargo_free_legacy_self_build_args(
    jobs: Option<u32>,
    no_substitute: bool,
    no_verify: bool,
    signing_key: Option<&Path>,
    trusted_public_keys: &[String],
    trust_unsigned: bool,
    strict_hermetic: bool,
    impure: bool,
    source_root: Option<&Path>,
    no_host_tools: bool,
    stage0_inventory: Option<&Path>,
    source_store_path: Option<&Path>,
    bootstrap_bwrap_path: Option<&Path>,
    bootstrap_busybox_path: Option<&Path>,
) -> Result<(), RunError> {
    let has_legacy_option = jobs.is_some()
        || no_substitute
        || no_verify
        || signing_key.is_some()
        || !trusted_public_keys.is_empty()
        || trust_unsigned
        || strict_hermetic
        || impure
        || source_root.is_some()
        || no_host_tools
        || stage0_inventory.is_some()
        || source_store_path.is_some()
        || bootstrap_bwrap_path.is_some()
        || bootstrap_busybox_path.is_some();
    if has_legacy_option {
        return Err(RunError::Build(
            "--cargo-free self-build cannot be combined with legacy stage0/store self-build options".to_string(),
        ));
    }
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
    let mut parsed = Vec::new();
    for key_str in keys {
        parsed.push(
            nix_compat::narinfo::VerifyingKey::parse(key_str)
                .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?,
        );
    }
    Ok(Some(parsed))
}

/// Build from an inline Nickel expression string (for project selectors).
#[allow(clippy::too_many_arguments)]
fn build_from_expr(
    expr: &str,
    import_paths: &[std::ffi::OsString],
    output_dir: &std::path::Path,
    state_dir: &std::path::Path,
    store_dir: &str,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_url: Option<&str>,
    signing_key_path: Option<&std::path::Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    debug_assert!(max_jobs > 0, "max_jobs must be positive");
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;

    cmd_build(
        tmp.path(),
        import_paths,
        output_dir,
        state_dir,
        store_dir,
        verbose,
        fix,
        max_jobs,
        substituter_url,
        signing_key_path,
        trusted_public_keys,
        trust_unsigned,
        hermeticity_mode,
        output_mode,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_plan_from_expr(
    expr: &str,
    import_paths: &[std::ffi::OsString],
    output_dir: &std::path::Path,
    state_dir: &std::path::Path,
    store_dir: &str,
    substituter_url: Option<&str>,
    signing_key_path: Option<&std::path::Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;

    build_plan::cmd_build_plan(build_plan::BuildPlanConfig {
        file: tmp.path(),
        import_paths,
        output_dir,
        state_dir,
        store_dir,
        substituter_url,
        signing_key_path,
        trusted_public_keys,
        trust_unsigned,
        output_mode,
    })
}

/// Build from an inline Nickel expression and return the pipeline result.
#[allow(clippy::too_many_arguments)]
fn build_from_expr_raw(
    expr: &str,
    import_paths: &[std::ffi::OsString],
    output_dir: &std::path::Path,
    state_dir: &std::path::Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    substituter_url: Option<&str>,
    signing_key_path: Option<&std::path::Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<crunch_pipeline::PipelineResult, RunError> {
    debug_assert!(max_jobs > 0, "max_jobs must be positive");
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    let tmp = tempfile::NamedTempFile::with_suffix(".ncl")
        .map_err(|e| RunError::Internal(format!("creating temp file: {e}")))?;
    std::fs::write(tmp.path(), expr).map_err(|e| RunError::Internal(format!("writing temp file: {e}")))?;

    let keypair = build_cmd::load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let configured_trusted_keys = build_cmd::load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let trusted_keys = crunch_build::signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    let config = crunch_pipeline::BuildConfig {
        file: tmp.path().to_path_buf(),
        import_paths: import_paths.to_vec(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        substituter_url: substituter_url.map(str::to_owned),
        hermeticity_mode,
        keypair,
        trusted_keys,
        trust_unsigned,
        root_retention_source: None,
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

fn selected_run_output_path(
    result: &crunch_pipeline::PipelineResult,
    output_dir: &Path,
    store_dir: &str,
    target_label: &str,
) -> Result<PathBuf, RunError> {
    let outcome_count = result.outcomes.len();
    if outcome_count != EXPECTED_RUN_OUTCOME_COUNT {
        return Err(RunError::Internal(format!(
            "run target {target_label} produced {outcome_count} derivations; expected exactly {EXPECTED_RUN_OUTCOME_COUNT}",
        )));
    }
    let outcome = result
        .outcomes
        .first()
        .ok_or_else(|| RunError::Internal(format!("run target {target_label} produced no outputs")))?;
    let path_info = outcome
        .outputs
        .get("out")
        .or_else(|| outcome.outputs.values().next())
        .ok_or_else(|| RunError::Internal(format!("run target {target_label} produced no outputs")))?;
    let host_path = output_host_path(path_info, output_dir, store_dir);
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

/// Build a project expression and return the raw pipeline result.
#[allow(clippy::too_many_arguments)]
fn build_project_expr(
    expr: &str,
    resolved_import_paths: Vec<OsString>,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    signing_key_path: Option<&Path>,
    trust_unsigned: bool,
) -> Result<crunch_pipeline::PipelineResult, RunError> {
    let sub_url = if no_substitute {
        None
    } else {
        Some("https://cache.nixos.org".to_string())
    };
    let mut full_import_paths = build_import_paths(&[])?;
    full_import_paths.extend(resolved_import_paths);
    build_from_expr_raw(
        expr,
        &full_import_paths,
        output_dir,
        state_dir,
        store_dir,
        verbose,
        max_jobs,
        sub_url.as_deref(),
        signing_key_path,
        None, // trusted keys — project commands don't accept custom keys yet
        trust_unsigned,
        crunch_pipeline::HermeticityMode::Practical, // develop/run do not expose a strict flag yet
    )
}

#[allow(clippy::too_many_arguments)]
fn build_file_raw(
    file: &Path,
    import_paths: &[PathBuf],
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    signing_key_path: Option<&Path>,
    trust_unsigned: bool,
) -> Result<crunch_pipeline::PipelineResult, RunError> {
    let sub_url = if no_substitute {
        None
    } else {
        Some("https://cache.nixos.org".to_string())
    };
    let import_paths = build_import_paths(import_paths)?;
    let keypair = build_cmd::load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let configured_trusted_keys = build_cmd::load_configured_trusted_public_keys(None, state_dir)?;
    let trusted_keys = crunch_build::signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    let config = crunch_pipeline::BuildConfig {
        file: file.to_path_buf(),
        import_paths,
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        substituter_url: sub_url,
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
        keypair,
        trusted_keys,
        trust_unsigned,
        root_retention_source: None,
    };

    build_cmd::run_build(&config)
}

#[allow(clippy::too_many_arguments)]
fn cmd_run(
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&Path>,
    trust_unsigned: bool,
    bin: Option<&str>,
    run_args: &[String],
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    verbose: bool,
) -> Result<(), RunError> {
    let cwd = current_dir_or_error()?;
    let target = name_to_build_target(name);
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let target_label = name.unwrap_or("default").to_string();
    let result = match &target {
        project_build::BuildTarget::File(path) => build_file_raw(
            path,
            import_paths,
            output_dir,
            state_dir,
            store_prefix,
            verbose,
            max_jobs,
            no_substitute,
            signing_key,
            trust_unsigned,
        )?,
        project_build::BuildTarget::ProjectDefault | project_build::BuildTarget::Selector(_) => {
            let resolved = project_build::resolve_project_target(&target, &cwd, import_paths)?;
            let expr = generate_run_project_expr(&resolved.root_file, &resolved.target);
            build_project_expr(
                &expr,
                resolved.import_paths,
                output_dir,
                state_dir,
                store_prefix,
                verbose,
                max_jobs,
                no_substitute,
                signing_key,
                trust_unsigned,
            )?
        }
    };
    let out_path = selected_run_output_path(&result, output_dir, store_prefix, &target_label)?;
    exec_run(&out_path, bin, run_args)
}

fn cmd_bootstrap_fetch(output: &std::path::Path, store_dir: &std::path::Path, verbose: bool) -> Result<(), RunError> {
    if !store_dir.exists() {
        return Err(RunError::Internal(format!(
            "store directory {} does not exist.\nCreate it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            store_dir.display()
        )));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async { bootstrap::bootstrap_fetch(store_dir, output, verbose).await })
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

    #[cfg(unix)]
    const TEST_EXEC_MODE: u32 = 0o755;
    #[cfg(unix)]
    const TEST_READ_MODE: u32 = 0o644;
    const TEST_REMOTE_TRANSFERRED_BYTES: u64 = 11;
    const TEST_REMOTE_REUSED_BYTES: u64 = 0;
    const TEST_REMOTE_STATUS_CONCURRENCY: u32 = 2;
    const TEST_REMOTE_STATUS_CONCURRENCY_TEXT: &str = "2";

    fn args_with_store_prefix(store_prefix: &str, nix_compat: bool) -> Args {
        Args {
            verbose: false,
            log_level: None,
            json: false,
            store: PathBuf::from("/nix/store"),
            store_prefix: store_prefix.to_string(),
            nix_compat,
            state_dir: None,
            command: Command::Doctor {
                profile: DoctorProfile::Build,
            },
        }
    }

    fn remote_client_report_fixture() -> remote_build::RemoteClientBuildReport {
        remote_build::RemoteClientBuildReport {
            schema: REMOTE_CLIENT_BUILD_REPORT_SCHEMA.to_string(),
            builder: "builder-1".to_string(),
            store_prefix: "/mantle/store".to_string(),
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
            }],
        }
    }

    #[test]
    fn default_store_prefix_is_mantle() {
        let args = Args::parse_from(["mantle", "doctor"]);
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
        assert_eq!(select_hermeticity_mode(false, false).unwrap(), crunch_pipeline::HermeticityMode::Practical);
        assert_eq!(select_hermeticity_mode(true, false).unwrap(), crunch_pipeline::HermeticityMode::Strict);
        assert_eq!(select_hermeticity_mode(false, true).unwrap(), crunch_pipeline::HermeticityMode::Impure);
        let err = select_hermeticity_mode(true, true).unwrap_err().to_string();
        assert!(err.contains("--strict-hermetic and --impure are mutually exclusive"));
    }

    #[test]
    fn build_cli_rejects_strict_hermetic_with_impure() {
        let err = Args::try_parse_from(["mantle", "build", "demo.ncl", "--strict-hermetic", "--impure"]).unwrap_err();
        assert!(err.to_string().contains("cannot be used with"));
    }

    #[test]
    fn build_cli_project_selector_is_not_implicit_rust_plan() {
        let args = Args::parse_from(["mantle", "build", ".#app"]);
        assert!(matches!(args.command, Command::Build { .. }));
        assert!(!matches!(args.command, Command::RustPlan { .. }));
    }

    #[test]
    fn build_cli_accepts_remote_builder_ticket_dispatch_flags() {
        let args = Args::parse_from([
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
        ]);
        let Command::Build {
            builder,
            ticket,
            builder_program,
            builder_args,
            trusted_builder_keys,
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
    }

    #[test]
    fn remote_status_cli_accepts_endpoint_and_concurrency() {
        let args = Args::parse_from([
            "mantle",
            "remote",
            "status",
            "--endpoint-id",
            "builder-1",
            "--concurrency",
            TEST_REMOTE_STATUS_CONCURRENCY_TEXT,
        ]);
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
        let args = Args::parse_from(["mantle", "self-build", "--impure"]);
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
        assert!(!err.contains(bootstrap_source_root::SOURCE_ROOT_BLOCKED_REASON));
        assert!(!output.exists());
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses() {
        let args = Args::parse_from([
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--recipe",
            "bootstrap/rust-source.ncl",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ]);

        let Command::Bootstrap {
            action:
                Some(BootstrapAction::RustSourceProvider {
                    recipe,
                    route_plan,
                    import_dir,
                    smoke,
                    smoke_evidence_dir,
                    output_dir,
                }),
            ..
        } = args.command
        else {
            panic!("expected rust-source-provider bootstrap action");
        };
        assert_eq!(recipe, PathBuf::from("bootstrap/rust-source.ncl"));
        assert!(route_plan.is_none());
        assert!(import_dir.is_none());
        assert!(!smoke);
        assert!(smoke_evidence_dir.is_none());
        assert_eq!(output_dir, PathBuf::from("/tmp/mantle-rust-provider"));
    }

    #[test]
    fn bootstrap_rust_source_provider_action_parses_route_plan() {
        let args = Args::parse_from([
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--recipe",
            "bootstrap/rust-source.ncl",
            "--route-plan",
            "bootstrap/rust-source-musl-host-plan.ncl",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ]);

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
    fn bootstrap_rust_source_provider_action_parses_import_dir() {
        let args = Args::parse_from([
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--import-dir",
            "/tmp/source-built-rust-provider",
            "--smoke",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ]);

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
        let args = Args::parse_from([
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
        ]);

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
    fn bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke() {
        let err = Args::try_parse_from([
            "mantle",
            "bootstrap",
            "rust-source-provider",
            "--import-dir",
            "/tmp/source-built-rust-provider",
            "--smoke-evidence-dir",
            "/tmp/mantle-rust-provider-smoke-evidence",
            "--output-dir",
            "/tmp/mantle-rust-provider",
        ])
        .unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--smoke"));
        assert!(rendered.contains("required"));
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

        let err = cmd_bootstrap_rust_source_provider(&recipe, None, None, &output_dir, &store, false, false, None)
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
    fn release_create_accepts_provider_fixed_point_proof_flag() {
        let args = Args::parse_from([
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
        ]);
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Create {
                provider_fixed_point_proof: Some(_),
                ..
            }
        }));
    }

    #[test]
    fn release_create_accepts_source_acquisition_url_flag() {
        let source_url = "https://example.invalid/releases/mantle-src.tar";
        let args = Args::parse_from([
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
        ]);
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
    fn release_create_accepts_git_source_flags() {
        let args = Args::parse_from([
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
        ]);
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
    fn release_witness_rebuild_accepts_require_independent_source_flag() {
        let args = Args::parse_from([
            "mantle",
            "release",
            "witness-rebuild",
            "/tmp/witness-request",
            "--require-independent-source",
            "--check",
        ]);
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
        let args = Args::parse_from([
            "mantle",
            "release",
            "witness-rebuild",
            "/tmp/witness-request",
            "--require-git-source",
            "--check",
        ]);
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
        let result = Args::try_parse_from([
            "mantle",
            "release",
            "witness-rebuild",
            "/tmp/witness-request",
            "--require-independent-source",
            "--require-git-source",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn release_verify_accepts_provider_fixed_point_proof_flags() {
        let args = Args::parse_from([
            "mantle",
            "release",
            "verify",
            "/tmp/release-bundle",
            "--provider-fixed-point-proof",
            "/tmp/provider-fixed-point-proof",
            "--require-provider-fixed-point-proof",
        ]);
        assert!(matches!(args.command, Command::Release {
            action: ReleaseAction::Verify {
                provider_fixed_point_proof: Some(_),
                require_provider_fixed_point_proof: true,
                ..
            }
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_out_dir() {
        let args = Args::parse_from(["mantle", "self-build", "--cargo-free", "--out", "/tmp/mantle-out"]);
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            fixed_point: false,
            out: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_fixed_point_out_dir() {
        let args = Args::parse_from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--fixed-point",
            "--out",
            "/tmp/mantle-fixed-point",
        ]);
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            fixed_point: true,
            out: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_toolchain_closure_manifest() {
        let args = Args::parse_from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
            "--toolchain-closure",
            "/tmp/toolchain.json",
        ]);
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            toolchain_closure: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_rust_source_provider() {
        let args = Args::parse_from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
            "--rust-source-provider",
            "/tmp/rust-source-provider",
        ]);
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            rust_source_provider: Some(_),
            ..
        }));
    }

    #[test]
    fn self_build_cli_accepts_cargo_free_target_triple() {
        let args = Args::parse_from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--out",
            "/tmp/mantle-out",
            "--target",
            "x86_64-unknown-linux-musl",
        ]);
        assert!(matches!(args.command, Command::SelfBuild {
            cargo_free: true,
            targets,
            ..
        } if targets == vec!["x86_64-unknown-linux-musl".to_string()]));
    }

    #[test]
    fn self_build_cli_rejects_toolchain_closure_without_cargo_free() {
        let err =
            Args::try_parse_from(["mantle", "self-build", "--toolchain-closure", "/tmp/toolchain.json"]).unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--cargo-free"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn self_build_cli_rejects_rust_source_provider_without_cargo_free() {
        let err = Args::try_parse_from([
            "mantle",
            "self-build",
            "--rust-source-provider",
            "/tmp/rust-source-provider",
        ])
        .unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--cargo-free"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn self_build_cli_rejects_fixed_point_without_cargo_free() {
        let err =
            Args::try_parse_from(["mantle", "self-build", "--fixed-point", "--out", "/tmp/mantle-out"]).unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("--cargo-free"));
        assert!(rendered.contains("required"));
    }

    #[test]
    fn cargo_free_fixed_point_rejects_legacy_options() {
        let args = Args::parse_from([
            "mantle",
            "self-build",
            "--cargo-free",
            "--fixed-point",
            "--out",
            "/tmp/mantle-fixed-point",
            "--jobs",
            "1",
        ]);

        let err = run(args).unwrap_err();

        assert_eq!(
            err.to_string(),
            "error: build failed\n--cargo-free self-build cannot be combined with legacy stage0/store self-build options"
        );
    }

    #[test]
    fn cargo_free_self_build_rejects_legacy_options() {
        let err = validate_cargo_free_legacy_self_build_args(
            Some(1),
            false,
            false,
            None,
            &[],
            false,
            false,
            false,
            None,
            false,
            None,
            None,
            None,
            None,
        )
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
