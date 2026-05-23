#![feature(register_tool)]
#![register_tool(tigerstyle)]
mod attest_cmd;
mod bootstrap;
mod bootstrap_parity;
mod bootstrap_source_root;
mod bootstrap_validate;
mod build_cmd;
mod build_failure;
mod build_log;
mod build_plan;
mod build_report;
mod errors;
mod fix;
mod log_cmd;
mod operator_diagnostics;
mod project_build;
mod project_cmd;
mod project_resolve;
#[allow(dead_code)]
mod protected_exec;
#[allow(dead_code)]
mod protected_exec_seccomp;
mod release_attestation;
mod release_cmd;
mod release_evidence;
mod release_nix_witness;
mod release_reproducibility;
mod release_source;
mod rust_plan;
mod self_build;
#[allow(dead_code)]
mod semantic_graph;
mod shell_cmd;
mod store_cmd;
mod structured_refactor;
mod system_cmd;
mod transcript_cmd;
mod witness_handoff;
mod witness_rebuild;

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const EXPECTED_RUN_OUTCOME_COUNT: usize = 1;
const CHILD_NO_EXIT_CODE_STATUS: i32 = 1;
#[cfg(unix)]
const UNIX_EXECUTE_BITS: u32 = 0o111;

use build_cmd::BuildOutputMode;
use build_cmd::build_import_paths;
use build_cmd::cmd_build;
use build_cmd::state_dir;
use clap::Parser;
use clap::Subcommand;
use errors::RunError;
use operator_diagnostics::DoctorProfile;
use system_cmd::SystemBuildOptions;
use system_cmd::SystemEvalFormat;
use system_cmd::SystemEvalOptions;
use system_cmd::SystemStopAfter;

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
    },

    /// Run no-mutate operator preflight checks for a workflow profile
    Doctor {
        /// Workflow profile to check
        #[arg(long, value_enum, default_value_t = DoctorProfile::Build)]
        profile: DoctorProfile,
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

    /// Evaluate or build a machine inventory through the system-config pipeline
    System {
        #[command(subcommand)]
        action: SystemAction,
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
    Check,

    /// Show resolved input state from the lockfile
    Show,

    /// Refresh selected or all project inputs
    Refresh {
        /// Input names to refresh (default: all non-frozen)
        names: Vec<String>,
    },

    /// List inputs that would change on refresh (read-only)
    ListStale,

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

        /// Execute the first supported lib/bin unit from the captured explicit derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_dependency_chain", "execute_target_topology", "execute_host_artifact_topology"])]
        execute_first_supported_unit: bool,

        /// Execute the first bounded dependency chain from the captured explicit derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_target_topology", "execute_host_artifact_topology"])]
        execute_first_dependency_chain: bool,

        /// Execute a bounded target-only unit topology from the captured explicit derivation graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_host_artifact_topology"])]
        execute_target_topology: bool,

        /// Execute a bounded host-artifact unit topology from the captured explicit derivation
        /// graph
        #[arg(long, conflicts_with_all = ["execute_first_supported_unit", "execute_first_dependency_chain", "execute_target_topology"])]
        execute_host_artifact_topology: bool,

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

        /// Optional canonical reproducibility report to package in the bundle
        #[arg(long)]
        reproducibility_report: Option<PathBuf>,

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
pub enum SystemAction {
    /// Dry-run the system-config pipeline and print fragments or derivations
    Eval {
        /// Path to the inventory Nickel file
        inventory: PathBuf,

        /// Module directory (default: ./modules relative to the inventory file)
        #[arg(long)]
        modules: Option<PathBuf>,

        /// Restrict evaluation to one or more machines
        #[arg(long = "machine")]
        machine: Vec<String>,

        /// Override assembler backend selection for all selected machines
        #[arg(long = "assembler")]
        assembler: Option<String>,

        /// Stop after merged fragments or dry-run derivations
        #[arg(long = "stop-after", value_enum, default_value_t = SystemStopAfter::Derivations)]
        stop_after: SystemStopAfter,

        /// Stdout serialization format for the result envelope
        #[arg(long = "format", value_enum, default_value_t = SystemEvalFormat::Json)]
        format: SystemEvalFormat,
    },
    /// Build a machine inventory through the system-config pipeline
    Build {
        /// Path to the inventory Nickel file
        inventory: PathBuf,

        /// Module directory (default: ./modules relative to the inventory file)
        #[arg(long)]
        modules: Option<PathBuf>,

        /// Restrict evaluation to one or more machines
        #[arg(long = "machine")]
        machine: Vec<String>,

        /// Override assembler backend selection for all selected machines
        #[arg(long = "assembler")]
        assembler: Option<String>,
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
        Command::System { action } => run_system_command(ctx, action),
        Command::Build { .. } => run_build_from_command(ctx, &args.command),
        Command::Bootstrap { .. } => run_bootstrap_from_command(ctx, &args.command),
        Command::Release { action } => run_release_command(ctx, action.clone()),
        Command::Log { query, list } => log_cmd::cmd_log(query.as_deref(), *list),
        Command::Store { action } => {
            store_cmd::cmd_store(action.clone(), &ctx.store, &ctx.resolved_state_dir, &ctx.store_prefix)
        }
        Command::Attest { action } => run_attest_command(ctx, action.clone()),
        Command::Init
        | Command::Check
        | Command::Show
        | Command::Refresh { .. }
        | Command::ListStale
        | Command::Upgrade => run_project_command(&args.command),
        Command::SelfBuild { .. } => run_self_build_from_command(ctx, &args.command),
        Command::RustPlan { .. } => run_rust_plan_command(ctx, &args.command),
        Command::Shell { .. } => run_shell_from_command(ctx, &args.command),
        Command::Develop { .. } => run_develop_from_command(ctx, &args.command),
        Command::Run { .. } => run_run_from_command(ctx, &args.command),
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

fn run_system_command(ctx: &RunContext, action: &SystemAction) -> Result<(), RunError> {
    match action {
        SystemAction::Eval {
            inventory,
            modules,
            machine,
            assembler,
            stop_after,
            format,
        } => system_cmd::cmd_system_eval(ctx, SystemEvalOptions {
            inventory_path: inventory.clone(),
            modules_dir: modules.clone(),
            machine_filter: machine.clone(),
            assembler_override: assembler.clone(),
            stop_after: *stop_after,
            format: *format,
        }),
        SystemAction::Build {
            inventory,
            modules,
            machine,
            assembler,
        } => system_cmd::cmd_system_build(ctx, SystemBuildOptions {
            inventory_path: inventory.clone(),
            modules_dir: modules.clone(),
            machine_filter: machine.clone(),
            assembler_override: assembler.clone(),
        }),
    }
}

fn run_build_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Build {
            file,
            import_paths,
            fix,
            plan,
            jobs,
            substituters,
            no_substitute,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
            impure,
        } => run_build_command(
            ctx,
            file.as_ref(),
            import_paths,
            *fix,
            *plan,
            *jobs,
            substituters,
            *no_substitute,
            signing_key.as_deref(),
            trusted_public_keys,
            *trust_unsigned,
            *strict_hermetic,
            *impure,
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
    jobs: Option<u32>,
    substituters: &str,
    no_substitute: bool,
    signing_key: Option<&std::path::Path>,
    trusted_public_keys: &[String],
    trust_unsigned: bool,
    strict_hermetic: bool,
    impure: bool,
) -> Result<(), RunError> {
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let substituter_url = (!no_substitute).then_some(substituters);
    let hermeticity_mode = select_hermeticity_mode(strict_hermetic, impure)?;
    let parsed_trusted = parse_trusted_keys(trusted_public_keys)?;
    let target = project_build::parse_build_target(file.map(PathBuf::as_path));
    match target {
        project_build::BuildTarget::File(path) => {
            let import_paths = build_import_paths(import_paths)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRootManifestCheck {
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
    let empty_provider_trace = bootstrap_source_root::ProviderDependencyTrace {
        urls: Vec::new(),
        hashes: Vec::new(),
        emitted_output_roles: Vec::new(),
        provider_metadata: Vec::new(),
    };
    bootstrap_source_root::validate_provider_dependency_trace(&manifest, &empty_provider_trace)
        .map_err(|errors| RunError::Internal(bootstrap_source_root::format_diagnostics(&errors)))?;
    let expected_output_role_count = u32::try_from(validation.expected_output_roles.len()).map_err(|_| {
        RunError::Internal("source-root manifest expected output role count overflowed u32".to_string())
    })?;
    let manifest_digest = bootstrap_source_root::source_root_manifest_digest(&manifest_bytes);
    Ok(SourceRootManifestCheck {
        manifest_digest,
        expected_output_role_count,
    })
}

fn bootstrap_source_root_provider(output: &Path, manifest_path: &Path) -> Result<(), RunError> {
    let checked = validate_source_root_manifest_file(manifest_path)?;
    Err(RunError::Build(format!(
        "{}: manifest_digest={} expected_output_roles={} output={}",
        bootstrap_source_root::SOURCE_ROOT_BLOCKED_REASON,
        checked.manifest_digest,
        checked.expected_output_role_count,
        output.display()
    )))
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
            bootstrap_source_root_provider(output, manifest_path)
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

fn run_project_command(command: &Command) -> Result<(), RunError> {
    let cwd = current_dir_or_error()?;
    match command {
        Command::Init => project_cmd::cmd_init(&cwd),
        Command::Check => project_cmd::cmd_check(&cwd),
        Command::Show => project_cmd::cmd_show(&cwd),
        Command::Refresh { names } => project_cmd::cmd_refresh(&cwd, names),
        Command::ListStale => project_cmd::cmd_list_stale(&cwd),
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
        targets,
        profile,
        features,
        all_features,
        no_default_features,
        execute_first_supported_unit,
        execute_first_dependency_chain,
        execute_target_topology,
        execute_host_artifact_topology,
        execution_output_root,
    } = command
    else {
        return Err(RunError::Internal("run_rust_plan_command called with non-RustPlan command".to_string()));
    };
    let root = root.clone().unwrap_or(current_dir_or_error()?);
    let options = rust_plan::RustPlanOptions {
        root,
        cargo: cargo.clone(),
        rustc: rustc.clone(),
        targets: targets.clone(),
        profile: profile.clone(),
        features: features.clone(),
        all_features: *all_features,
        no_default_features: *no_default_features,
    };
    let receipt = rust_plan::capture_rust_plan(&options)?;
    if *execute_first_supported_unit {
        let output_root = execution_output_root.clone().ok_or_else(|| {
            RunError::Internal("--execute-first-supported-unit requires --execution-output-root".to_string())
        })?;
        let unit_execution = rust_plan::execute_first_supported_rust_unit(
            &receipt.unit_derivation_graph,
            &rust_plan::RustUnitExecutionOptions {
                rustc: rustc.clone(),
                output_root,
            },
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
            &rust_plan::RustUnitExecutionOptions {
                rustc: rustc.clone(),
                output_root,
            },
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
            &rust_plan::RustUnitExecutionOptions {
                rustc: rustc.clone(),
                output_root,
            },
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
            &receipt.unit_derivation_graph,
            &rust_plan::RustUnitExecutionOptions {
                rustc: rustc.clone(),
                output_root,
            },
        )?;
        return rust_plan::print_rust_plan_host_artifact_topology_execution_receipt(
            &rust_plan::RustPlanHostArtifactTopologyExecutionReceipt {
                rust_plan: receipt,
                host_artifact_topology_execution,
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
) -> Result<(), RunError> {
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
    fn self_build_cli_accepts_impure_mode() {
        let args = Args::parse_from(["mantle", "self-build", "--impure"]);
        assert!(matches!(args.command, Command::SelfBuild {
            impure: true,
            strict_hermetic: false,
            ..
        }));
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
