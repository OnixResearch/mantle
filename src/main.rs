mod bootstrap;
mod build_cmd;
mod build_log;
mod build_report;
mod errors;
mod fix;
mod log_cmd;
mod project_cmd;
mod project_resolve;
mod self_build;
mod store_cmd;

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
    /// Evaluate a .ncl file and build the derivation(s)
    Build {
        /// Path to the .ncl file
        file: PathBuf,

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

        /// Fetch static toolchain tarballs instead of querying Nix.
        /// Downloads a statically-linked musl-gcc, persists it as a
        /// FOD in --store, and writes seed.ncl. No Nix required.
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
    },
}

#[derive(Subcommand, Debug)]
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

fn run(args: Args) -> Result<(), RunError> {
    if let Some(ref state_dir) = args.state_dir {
        unsafe { std::env::set_var("CRUNCH_STATE_DIR", state_dir) };
    }

    let resolved_state_dir = state_dir();
    let store_prefix = resolve_store_prefix(&args);

    match args.command {
        Command::Eval { file, import_paths } => {
            let import_paths = build_import_paths(&import_paths)?;
            let json =
                crunch_eval::evaluate_to_json(&file, &import_paths).map_err(|e| RunError::Eval(format!("{e}")))?;
            println!("{json}");
            Ok(())
        }
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
        } => {
            let import_paths = build_import_paths(&import_paths)?;
            let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
            let sub_url = if no_substitute { None } else { Some(substituters) };
            let parsed_trusted: Option<Vec<nix_compat::narinfo::VerifyingKey>> = if trusted_public_keys.is_empty() {
                None
            } else {
                let mut keys = Vec::new();
                for key_str in &trusted_public_keys {
                    keys.push(
                        nix_compat::narinfo::VerifyingKey::parse(key_str)
                            .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?,
                    );
                }
                Some(keys)
            };
            let output_mode = if args.json {
                BuildOutputMode::Json
            } else {
                BuildOutputMode::Human
            };
            cmd_build(
                &file,
                &import_paths,
                &args.store,
                &resolved_state_dir,
                &store_prefix,
                args.verbose,
                fix,
                max_jobs,
                sub_url.as_deref(),
                signing_key.as_deref(),
                parsed_trusted.as_deref(),
                trust_unsigned,
                output_mode,
            )
        }
        Command::Bootstrap {
            output,
            fetch,
            packages,
        } => {
            if fetch {
                cmd_bootstrap_fetch(&output, &args.store, args.verbose)
            } else {
                cmd_bootstrap(&output, &packages)
            }
        }
        Command::Log { query, list } => log_cmd::cmd_log(query.as_deref(), list),
        Command::Store { action } => store_cmd::cmd_store(action),
        Command::Init => project_cmd::cmd_init(&std::env::current_dir().unwrap()),
        Command::Check => project_cmd::cmd_check(&std::env::current_dir().unwrap()),
        Command::Show => project_cmd::cmd_show(&std::env::current_dir().unwrap()),
        Command::Refresh { names } => project_cmd::cmd_refresh(&std::env::current_dir().unwrap(), &names),
        Command::ListStale => project_cmd::cmd_list_stale(&std::env::current_dir().unwrap()),
        Command::Upgrade => project_cmd::cmd_upgrade(&std::env::current_dir().unwrap()),
        Command::SelfBuild {
            jobs,
            no_substitute,
            no_verify,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
        } => {
            let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
            let parsed_trusted: Option<Vec<nix_compat::narinfo::VerifyingKey>> = if trusted_public_keys.is_empty() {
                None
            } else {
                let mut keys = Vec::new();
                for key_str in &trusted_public_keys {
                    keys.push(
                        nix_compat::narinfo::VerifyingKey::parse(key_str)
                            .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?,
                    );
                }
                Some(keys)
            };
            self_build::cmd_self_build(
                &args.store,
                &resolved_state_dir,
                &store_prefix,
                args.verbose,
                max_jobs,
                no_substitute,
                no_verify,
                signing_key.as_deref(),
                parsed_trusted.as_deref(),
                trust_unsigned,
            )
            .map(|_report| ())
        }
    }
}

fn resolve_store_prefix(args: &Args) -> String {
    if args.nix_compat {
        "/nix/store".to_string()
    } else {
        args.store_prefix.clone()
    }
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
