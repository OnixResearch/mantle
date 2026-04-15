mod attest_cmd;
mod bootstrap;
mod build_cmd;
mod build_log;
mod build_report;
mod errors;
mod fix;
mod log_cmd;
mod project_build;
mod project_cmd;
mod project_resolve;
mod self_build;
mod shell_cmd;
mod store_cmd;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use build_cmd::BuildOutputMode;
use build_cmd::build_import_paths;
use build_cmd::cmd_build;
use build_cmd::state_dir;
use clap::Parser;
use clap::Subcommand;
use errors::RunError;

#[derive(Parser, Debug)]
#[command(name = "crunch", about = "Nickel build system on the Nix store protocol")]
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
    /// Default: /crunch/store. Use --nix-compat for /nix/store.
    #[arg(long, global = true, default_value = "/crunch/store")]
    store_prefix: String,

    /// Shorthand for --store-prefix=/nix/store. For interop testing
    /// with Nix-computed derivations.
    #[arg(long, global = true)]
    nix_compat: bool,

    /// State directory for pathinfo.redb, blobs, and logs.
    /// Default: $CRUNCH_STATE_DIR or $XDG_STATE_HOME/crunch or
    /// ~/.local/state/crunch.
    #[arg(long, global = true)]
    state_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Evaluate and build derivation(s).
    ///
    /// With no arguments: builds default package from crunch.ncl.
    /// With a .ncl file: builds derivations from that file.
    /// With .#name: builds a named output from crunch.ncl.
    Build {
        /// Path to a .ncl file, or .#name selector, or omit for project default
        file: Option<PathBuf>,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,

        /// Auto-fix FOD hash mismatches by rewriting the .ncl source
        #[arg(long)]
        fix: bool,

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
        #[arg(long)]
        strict_hermetic: bool,
    },

    /// Evaluate a .ncl file and print the derivation JSON (no build)
    Eval {
        /// Path to the .ncl file
        file: PathBuf,

        /// Additional import paths for Nickel
        #[arg(long = "import-path", short = 'I')]
        import_paths: Vec<PathBuf>,
    },

    /// Generate a seed.ncl from existing Nix store packages
    Bootstrap {
        /// Output file path
        #[arg(short, long, default_value = "seed.ncl")]
        output: PathBuf,

        /// Fetch the shared bootstrap seed provider instead of querying Nix.
        /// Builds the reduced normalized seed from the pinned musl.cc tarball,
        /// persists it in --store, and writes seed.ncl. No Nix required.
        #[arg(long)]
        fetch: bool,

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

    /// Initialize a new crunch project (manifest, lockfile, .crunch/)
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

    /// Build crunch from its own source (self-hosting)
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
        #[arg(long)]
        strict_hermetic: bool,

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

    /// Enter a development shell from crunch.ncl devShells
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

    /// Build and run an executable from crunch.ncl packages
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

        /// Arguments to pass to the executable (after --)
        #[arg(last = true)]
        run_args: Vec<String>,
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
pub enum StoreAction {
    /// List all known store paths
    List,
    /// Show detailed PathInfo for a store path
    Info {
        /// Store path (full or fragment to match)
        path: String,
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
}

fn main() -> ExitCode {
    let args = Args::parse();

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

    let json_errors = args.json;

    match run(args) {
        Ok(()) => ExitCode::from(0),
        Err(error) => {
            if json_errors {
                eprintln!("{}", error.format_json());
            } else {
                eprintln!("{}", error.format_human());
            }
            error.exit_code()
        }
    }
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
        Command::Eval { file, import_paths } => run_eval(file, import_paths),
        Command::Build { .. } => run_build_from_command(ctx, &args.command),
        Command::Bootstrap { .. } => run_bootstrap_from_command(ctx, &args.command),
        Command::Log { query, list } => log_cmd::cmd_log(query.as_deref(), *list),
        Command::Store { action } => store_cmd::cmd_store(action.clone()),
        Command::Attest { action } => run_attest_command(ctx, action.clone()),
        Command::Init
        | Command::Check
        | Command::Show
        | Command::Refresh { .. }
        | Command::ListStale
        | Command::Upgrade => run_project_command(&args.command),
        Command::SelfBuild { .. } => run_self_build_from_command(ctx, &args.command),
        Command::Shell { .. } => run_shell_from_command(ctx, &args.command),
        Command::Develop { .. } => run_develop_from_command(ctx, &args.command),
        Command::Run { .. } => run_run_from_command(ctx, &args.command),
    }
}

fn run_eval(file: &PathBuf, import_paths: &[PathBuf]) -> Result<(), RunError> {
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
            jobs,
            substituters,
            no_substitute,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
            strict_hermetic,
        } => run_build_command(
            ctx,
            file.as_ref(),
            import_paths,
            *fix,
            *jobs,
            substituters,
            *no_substitute,
            signing_key.as_deref(),
            trusted_public_keys,
            *trust_unsigned,
            *strict_hermetic,
        ),
        _ => unreachable!("build helper called with non-build command"),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_build_command(
    ctx: &RunContext,
    file: Option<&PathBuf>,
    import_paths: &[PathBuf],
    fix: bool,
    jobs: Option<u32>,
    substituters: &str,
    no_substitute: bool,
    signing_key: Option<&std::path::Path>,
    trusted_public_keys: &[String],
    trust_unsigned: bool,
    strict_hermetic: bool,
) -> Result<(), RunError> {
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let substituter_url = (!no_substitute).then_some(substituters);
    let hermeticity_mode = if strict_hermetic {
        crunch_pipeline::HermeticityMode::Strict
    } else {
        crunch_pipeline::HermeticityMode::Practical
    };
    let parsed_trusted = parse_trusted_keys(trusted_public_keys)?;
    let target = project_build::parse_build_target(file.map(PathBuf::as_path));
    match target {
        project_build::BuildTarget::File(path) => {
            let import_paths = build_import_paths(import_paths)?;
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

fn run_bootstrap_from_command(ctx: &RunContext, command: &Command) -> Result<(), RunError> {
    match command {
        Command::Bootstrap {
            output,
            fetch,
            packages,
        } => run_bootstrap_command(ctx, output, *fetch, packages),
        _ => unreachable!("bootstrap helper called with non-bootstrap command"),
    }
}

fn run_bootstrap_command(ctx: &RunContext, output: &PathBuf, fetch: bool, packages: &[String]) -> Result<(), RunError> {
    if fetch {
        cmd_bootstrap_fetch(output, &ctx.store, ctx.verbose)
    } else {
        cmd_bootstrap(output, packages)
    }
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

fn run_attest_command(ctx: &RunContext, action: AttestAction) -> Result<(), RunError> {
    attest_cmd::cmd_attest(action, &current_dir_or_error()?, &ctx.store, &ctx.resolved_state_dir, &ctx.store_prefix)
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
            run_args,
        } => run_package_command(
            ctx,
            name.as_deref(),
            import_paths,
            *jobs,
            *no_substitute,
            signing_key.as_deref(),
            *trust_unsigned,
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
    run_args: &[String],
) -> Result<(), RunError> {
    cmd_run(
        name,
        import_paths,
        jobs,
        no_substitute,
        signing_key,
        trust_unsigned,
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
            source_store_path.as_deref(),
            bootstrap_bwrap_path.as_deref(),
            bootstrap_busybox_path.as_deref(),
        ),
        _ => unreachable!("self-build helper called with non-self-build command"),
    }
}

fn run_self_build_command(
    ctx: &RunContext,
    jobs: Option<u32>,
    no_substitute: bool,
    no_verify: bool,
    signing_key: Option<&std::path::Path>,
    trusted_public_keys: &[String],
    trust_unsigned: bool,
    strict_hermetic: bool,
    source_store_path: Option<&std::path::Path>,
    bootstrap_bwrap_path: Option<&std::path::Path>,
    bootstrap_busybox_path: Option<&std::path::Path>,
) -> Result<(), RunError> {
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let hermeticity_mode = if strict_hermetic {
        crunch_pipeline::HermeticityMode::Strict
    } else {
        crunch_pipeline::HermeticityMode::Practical
    };
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
        bootstrap_bwrap_path,
        bootstrap_busybox_path,
    )
    .map(|_report| ())
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
    };

    build_cmd::run_build(&config)
}

/// Extract the first successful "out" output path from a pipeline result.
fn first_output_path(
    result: &crunch_pipeline::PipelineResult,
    output_dir: &std::path::Path,
    store_dir: &str,
) -> Option<PathBuf> {
    for outcome in &result.outcomes {
        // Prefer the "out" output, fall back to first alphabetically.
        let path_info = outcome.outputs.get("out").or_else(|| outcome.outputs.values().next())?;
        let abs = path_info.store_path.to_absolute_path_with_prefix(store_dir);
        let host_path = if output_dir == std::path::Path::new(store_dir) {
            PathBuf::from(&abs)
        } else {
            let rel = abs.strip_prefix(store_dir).unwrap_or(&abs);
            let rel = rel.strip_prefix('/').unwrap_or(rel);
            output_dir.join(rel)
        };
        if host_path.exists() {
            return Some(host_path);
        }
    }
    None
}

/// Find the first executable in out_path/bin and exec it with args.
fn exec_run(out_path: &std::path::Path, args: &[String]) -> Result<(), RunError> {
    let bin_dir = out_path.join("bin");
    if !bin_dir.is_dir() {
        return Err(RunError::Internal(format!("no bin/ directory in {}", out_path.display())));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&bin_dir)
        .map_err(|e| RunError::Internal(format!("reading {}: {e}", bin_dir.display())))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file() || t.is_symlink()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let exe = entries
        .first()
        .ok_or_else(|| RunError::Internal(format!("no executables in {}", bin_dir.display())))?;

    let exe_path = exe.path();
    eprintln!("running: {}", exe_path.display());

    let mut cmd_args: Vec<String> = vec![exe_path.to_string_lossy().to_string()];
    cmd_args.extend(args.iter().cloned());

    let status = std::process::Command::new(&exe_path)
        .args(args)
        .status()
        .map_err(|e| RunError::Internal(format!("exec {}: {e}", exe_path.display())))?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Resolve a CLI name argument into a project build target.
fn name_to_build_target(name: Option<&str>) -> project_build::BuildTarget {
    match name {
        None => project_build::BuildTarget::ProjectDefault,
        Some(s) if s.starts_with(".#") => project_build::parse_build_target(Some(std::path::Path::new(s))),
        Some(s) => project_build::BuildTarget::Selector(project_build::Selector {
            segments: vec![s.to_string()],
        }),
    }
}

/// Build a project expression and return the raw pipeline result.
#[allow(clippy::too_many_arguments)]
fn build_project_expr(
    expr: &str,
    resolved_import_paths: Vec<std::ffi::OsString>,
    output_dir: &std::path::Path,
    state_dir: &std::path::Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    signing_key_path: Option<&std::path::Path>,
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
fn cmd_run(
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&std::path::Path>,
    trust_unsigned: bool,
    run_args: &[String],
    output_dir: &std::path::Path,
    state_dir: &std::path::Path,
    store_prefix: &str,
    verbose: bool,
) -> Result<(), RunError> {
    let cwd = current_dir_or_error()?;
    let target = name_to_build_target(name);
    let resolved = project_build::resolve_project_target(&target, &cwd, import_paths)?;
    let expr = project_build::generate_extraction_expr(&resolved.root_file, &resolved.target);
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let result = build_project_expr(
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
    )?;
    let out_path = first_output_path(&result, output_dir, store_prefix)
        .ok_or_else(|| RunError::Internal("no outputs built".into()))?;
    exec_run(&out_path, run_args)
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
