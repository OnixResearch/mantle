mod bootstrap;
mod errors;
mod self_build;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nix_compat::store_path::StorePath;
use tracing::info;

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
    },
}

#[derive(Subcommand, Debug)]
enum StoreAction {
    /// List all known store paths
    List,
    /// Show detailed PathInfo for a store path
    Info {
        /// Store path (full or fragment to match)
        path: String,
    },
    /// Verify NAR hash of stored paths against disk
    Verify {
        /// Optional: verify a specific path (default: all)
        path: Option<String>,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();

    let level = args
        .log_level
        .as_deref()
        .map(|l| l.parse().unwrap_or(tracing::Level::INFO))
        .unwrap_or(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::WARN
        });

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(level.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let json_errors = args.json;

    match run(args) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            if json_errors {
                eprintln!("{}", e.format_json());
            } else {
                eprintln!("{}", e.format_human());
            }
            e.exit_code()
        }
    }
}

fn run(args: Args) -> Result<(), RunError> {
    // Set CRUNCH_STATE_DIR early so state_dir()/log_dir() pick it up.
    // SAFETY: this runs before spawning threads; no concurrent readers.
    if let Some(ref sd) = args.state_dir {
        unsafe { std::env::set_var("CRUNCH_STATE_DIR", sd) };
    }
    let store_prefix = resolve_store_prefix(&args);
    match args.command {
        Command::Eval { file, import_paths } => {
            let import_paths = build_import_paths(&import_paths)?;
            let json = crunch_eval::evaluate_to_json(&file, &import_paths)
                .map_err(|e| RunError::Eval(format!("{e}")))?;
            println!("{json}");
            Ok(())
        }
        Command::Build { file, import_paths, fix, jobs, substituters, no_substitute } => {
            let import_paths = build_import_paths(&import_paths)?;
            let max_jobs = resolve_max_jobs(jobs);
            let sub_url = if no_substitute { None } else { Some(substituters) };
            cmd_build(&file, &import_paths, &args.store, &store_prefix, args.verbose, fix, max_jobs, sub_url.as_deref())
        }
        Command::Bootstrap { output, fetch, packages } => {
            if fetch {
                cmd_bootstrap_fetch(&output, &args.store, args.verbose)
            } else {
                cmd_bootstrap(&output, &packages)
            }
        }
        Command::Log { query, list } => cmd_log(query.as_deref(), list),
        Command::Store { action } => cmd_store(action),
        Command::SelfBuild { jobs, no_substitute, no_verify } => {
            let max_jobs = resolve_max_jobs(jobs);
            cmd_self_build(&args.store, args.verbose, max_jobs, no_substitute, no_verify)
        }
    }
}

// ── cmd_build: eval → convert → build ──────────────────────────────────

/// Resolve the effective store prefix from CLI args.
/// --nix-compat overrides --store-prefix to /nix/store.
fn resolve_store_prefix(args: &Args) -> String {
    if args.nix_compat {
        "/nix/store".to_string()
    } else {
        args.store_prefix.clone()
    }
}

/// Resolve --jobs: user value clamped to [1, 16], or available_parallelism.
fn resolve_max_jobs(user: Option<u32>) -> u32 {
    const MAX_JOBS_CAP: u32 = 16;
    match user {
        Some(j) => j.clamp(1, MAX_JOBS_CAP),
        None => std::thread::available_parallelism()
            .map(|n| (n.get() as u32).min(MAX_JOBS_CAP))
            .unwrap_or(1),
    }
}

fn cmd_build(
    file: &std::path::Path,
    import_paths: &[OsString],
    output_dir: &std::path::Path,
    store_prefix: &str,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_url: Option<&str>,
) -> Result<(), RunError> {
    if !output_dir.exists() {
        return Err(RunError::Internal(format!(
            "output store directory {} does not exist.\n\
             Create it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            output_dir.display()
        )));
    }

    // Phase 1: Evaluate Nickel expression → JSON → derivations.
    // Uses Nickel's JSON export which correctly serializes enum tags
    // to strings. Direct Expr::to_serde() fails on nested derivations
    // in inputs[] because serde's #[serde(untagged)] can't handle
    // Nickel enum variants.
    info!(file = %file.display(), "evaluating");
    let json_str = crunch_eval::evaluate_to_json(file, import_paths)
        .map_err(|e| RunError::Eval(format!("{e}")))?;
    let derivations = deserialize_derivations_from_json(&json_str)?;
    debug_assert!(!derivations.is_empty(), "must have at least one derivation");

    // Phase 2: Convert each root derivation and stream to build Worker.
    // Eval (convert) and building overlap: as soon as the first
    // derivation is converted, the Worker can start building its deps.
    let log_dir = log_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let output_dir_str = output_dir.to_str().unwrap_or(store_prefix);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async {
        execute_builds_streaming(
            derivations,
            output_dir,
            output_dir_str,
            store_prefix,
            &log_dir,
            file,
            verbose,
            fix,
            max_jobs,
            substituter_url,
        )
        .await
    })
}



/// Deserialize derivations from a JSON string (Nickel's JSON export).
///
/// Handles both single-derivation records (has a "name" field) and
/// package sets (each field is a derivation).
fn deserialize_derivations_from_json(
    json_str: &str,
) -> Result<Vec<(String, crunch_glue::CrunchDerivation)>, RunError> {
    let json_val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| RunError::Eval(format!("parsing JSON: {e}")))?;

    // Array of derivations: each element is a derivation record.
    if let Some(arr) = json_val.as_array() {
        let mut derivations = Vec::new();
        for (i, elem) in arr.iter().enumerate() {
            let drv: crunch_glue::CrunchDerivation = serde_json::from_value(elem.clone())
                .map_err(|e| RunError::Eval(format!("deserializing derivation [{i}]: {e}")))?;
            derivations.push((drv.name.clone(), drv));
        }
        return Ok(derivations);
    }

    let Some(obj) = json_val.as_object() else {
        return Err(RunError::Eval(
            "expected a Derivation record, array of Derivations, or record of Derivations".to_string(),
        ));
    };

    // Single derivation: has a "name" field with a string value.
    if obj.get("name").is_some_and(|v| v.is_string()) {
        let drv: crunch_glue::CrunchDerivation = serde_json::from_value(json_val)
            .map_err(|e| RunError::Eval(format!("deserializing derivation: {e}")))?;
        let name = drv.name.clone();
        return Ok(vec![(name, drv)]);
    }

    // Package set: each field is a derivation.
    let mut derivations = Vec::new();
    for (key, value) in obj {
        let drv: crunch_glue::CrunchDerivation = serde_json::from_value(value.clone())
            .map_err(|e| RunError::Eval(format!("deserializing derivation '{key}': {e}")))?;
        derivations.push((key.clone(), drv));
    }
    Ok(derivations)
}

#[allow(dead_code)] // Kept as non-streaming fallback.
#[allow(clippy::too_many_arguments)]
async fn execute_builds(
    drv_paths: &[(String, StorePath<String>)],
    known_paths: &mut crunch_build::DerivationRegistry,
    output_dir: &std::path::Path,
    output_dir_str: &str,
    store_prefix: &str,
    log_dir: &std::path::Path,
    source_file: &std::path::Path,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
) -> Result<(), RunError> {
    let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: state_dir(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_url: None,
        store_dir: store_prefix.to_string(),
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening store: {e}")))?;

    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let pathinfo_service = store.pathinfo_service();

    #[cfg(target_os = "linux")]
    {
        use snix_build::buildservice::BubblewrapBuildService;

        let workdir = std::env::temp_dir().join("crunch-builds");
        std::fs::create_dir_all(&workdir)
            .map_err(|e| RunError::Internal(format!("create workdir: {e}")))?;

        let build_service = BubblewrapBuildService::new(
            workdir,
            blob_service.clone(),
            directory_service.clone(),
        );

        let mut builder = crunch_build::Builder::with_state_dir(
            blob_service,
            directory_service,
            build_service,
            pathinfo_service,
            output_dir.to_path_buf(),
            Some(store.state_dir().to_path_buf()),
            None, // no remote substitution in legacy path
            store_prefix,
            verbose,
        );

        // Collect just the StorePaths for build_all.
        let root_paths: Vec<StorePath<String>> =
            drv_paths.iter().map(|(_, sp)| sp.clone()).collect();

        let outcomes = match builder.build_all(&root_paths, known_paths, max_jobs).await {
            Ok(outcomes) => outcomes,
            Err(crunch_build::Error::FodHashMismatch {
                ref name,
                ref expected_sri,
                ref actual_sri,
            }) => {
                // Find the matching drv_path/label for the FOD error.
                let (label, drv_path) = drv_paths
                    .iter()
                    .find(|(_, sp)| sp.name().contains(name.as_str()))
                    .unwrap_or(&drv_paths[0]);
                return handle_fod_mismatch(
                    name, expected_sri, actual_sri, drv_path, label,
                    log_dir, source_file, fix,
                );
            }
            Err(e) => {
                let log_msg = format!("{e}");
                if let Some((label, drv_path)) = drv_paths.first() {
                    write_log(log_dir, drv_path, label, false, &log_msg);
                }
                return Err(RunError::Build(log_msg));
            }
        };

        // Build a label lookup from drv path string.
        let label_by_drv: std::collections::HashMap<String, &str> = drv_paths
            .iter()
            .map(|(label, sp)| {
                (sp.to_absolute_path_with_prefix(store_prefix), label.as_str())
            })
            .collect();

        for outcome in &outcomes {
            let drv_abs = outcome.drv_path
                .to_absolute_path_with_prefix(store_prefix);
            let label = label_by_drv.get(drv_abs.as_str()).copied().unwrap_or("?");

            if let Some(log) = &outcome.log {
                write_log(log_dir, &outcome.drv_path, label, true, log);
                if verbose {
                    eprintln!("--- build log: {label} ---");
                    eprintln!("{log}");
                    eprintln!("--- end log ---");
                }
            } else if !outcome.cached {
                write_log(
                    log_dir, &outcome.drv_path, label, true,
                    "(no output captured)",
                );
            }

            let multi = outcome.outputs.len() > 1;
            for (output_name, path_info) in &outcome.outputs {
                let path = path_info.store_path
                    .to_absolute_path_with_prefix(output_dir_str);
                let suffix = match (outcome.cached, multi && output_name != "out") {
                    (true, true) => format!(" ({output_name}, cached)"),
                    (true, false) => " (cached)".to_string(),
                    (false, true) => format!(" ({output_name})"),
                    (false, false) => String::new(),
                };
                println!("{path}{suffix}");
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        return Err(RunError::Internal(
            "building is only supported on Linux (requires bwrap)".to_string(),
        ));
    }

    Ok(())
}

/// Streaming build pipeline: converts derivations on a background thread
/// while the Worker builds them concurrently.
///
/// Each root derivation is converted and sent over a channel. The Worker
/// starts building as soon as the first leaf dependency is ready, even
/// while later roots are still being converted.
#[allow(clippy::too_many_arguments)]
async fn execute_builds_streaming(
    derivations: Vec<(String, crunch_glue::CrunchDerivation)>,
    output_dir: &std::path::Path,
    output_dir_str: &str,
    store_prefix: &str,
    log_dir: &std::path::Path,
    source_file: &std::path::Path,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_url: Option<&str>,
) -> Result<(), RunError> {
    use tokio::sync::mpsc;

    let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: state_dir(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_url: substituter_url.map(|s| s.to_string()),
        store_dir: store_prefix.to_string(),
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening store: {e}")))?;

    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let pathinfo_service = store.pathinfo_service();
    let remote_pathinfo = store.remote_pathinfo();

    #[cfg(target_os = "linux")]
    {
        use snix_build::buildservice::BubblewrapBuildService;

        let workdir = std::env::temp_dir().join("crunch-builds");
        std::fs::create_dir_all(&workdir)
            .map_err(|e| RunError::Internal(format!("create workdir: {e}")))?;

        let build_service = BubblewrapBuildService::new(
            workdir,
            blob_service.clone(),
            directory_service.clone(),
        );

        let mut builder = crunch_build::Builder::with_state_dir(
            blob_service,
            directory_service,
            build_service,
            pathinfo_service,
            output_dir.to_path_buf(),
            Some(store.state_dir().to_path_buf()),
            remote_pathinfo,
            store_prefix,
            verbose,
        );

        // Convert all derivations into ConversionCache, then bridge
        // to DerivationRegistry for the build engine.
        let mut cache = crunch_glue::ConversionCache::new(store_prefix);
        let mut drv_paths: Vec<(String, StorePath<String>)> = Vec::new();

        for (label, drv) in &derivations {
            let (drv_path, _nix_drv) = crunch_glue::convert(drv, &mut cache)
                .map_err(|e| RunError::Build(format!("{label}: {e}")))?;
            info!(drv = %drv_path, label = %label, "derivation constructed");
            drv_paths.push((label.clone(), drv_path));
        }

        // Bridge: populate DerivationRegistry from ConversionCache.
        let mut known_paths = crunch_build::DerivationRegistry::new(store_prefix);
        crunch_build::populate_registry(&mut known_paths, cache.iter_entries());

        // Stream roots to the Worker over a channel. The Worker starts
        // building each root's deps as soon as it arrives, concurrently
        // with processing later roots.
        let (tx, mut rx) = mpsc::channel::<crunch_build::EvalMessage>(16);

        // Send all roots, then close the channel.
        for (label, drv_path) in &drv_paths {
            tx.send(crunch_build::EvalMessage {
                label: label.clone(),
                drv_path: drv_path.clone(),
            }).await.map_err(|e| RunError::Internal(format!("channel send: {e}")))?;
        }
        drop(tx);

        // Run the Worker.
        let mut worker = crunch_build::Worker::new(max_jobs);
        let worker_result = worker
            .run_streaming(&mut builder, &mut known_paths, &mut rx)
            .await;

        let result = match worker_result {
            Ok(r) => r,
            Err(crunch_build::Error::FodHashMismatch {
                ref name,
                ref expected_sri,
                ref actual_sri,
            }) => {
                let (label, drv_path) = drv_paths
                    .iter()
                    .find(|(_, sp)| sp.name().contains(name.as_str()))
                    .unwrap_or(&drv_paths[0]);
                return handle_fod_mismatch(
                    name, expected_sri, actual_sri, drv_path, label,
                    log_dir, source_file, fix,
                );
            }
            Err(e) => {
                let log_msg = format!("{e}");
                if let Some((label, drv_path)) = drv_paths.first() {
                    write_log(log_dir, drv_path, label, false, &log_msg);
                }
                return Err(RunError::Build(log_msg));
            }
        };

        // Report results.
        let label_by_drv: std::collections::HashMap<String, &str> = drv_paths
            .iter()
            .map(|(label, sp)| {
                (sp.to_absolute_path_with_prefix(store_prefix), label.as_str())
            })
            .collect();

        for outcome in &result.outcomes {
            let drv_abs = outcome.drv_path
                .to_absolute_path_with_prefix(store_prefix);
            let label = label_by_drv.get(drv_abs.as_str()).copied().unwrap_or("?");

            if let Some(log) = &outcome.log {
                write_log(log_dir, &outcome.drv_path, label, true, log);
                if verbose {
                    eprintln!("--- build log: {label} ---");
                    eprintln!("{log}");
                    eprintln!("--- end log ---");
                }
            } else if !outcome.cached {
                write_log(
                    log_dir, &outcome.drv_path, label, true,
                    "(no output captured)",
                );
            }

            let multi = outcome.outputs.len() > 1;
            for (output_name, path_info) in &outcome.outputs {
                let path = path_info.store_path
                    .to_absolute_path_with_prefix(output_dir_str);
                let suffix = match (outcome.cached, multi && output_name != "out") {
                    (true, true) => format!(" ({output_name}, cached)"),
                    (true, false) => " (cached)".to_string(),
                    (false, true) => format!(" ({output_name})"),
                    (false, false) => String::new(),
                };
                println!("{path}{suffix}");
            }
        }

        if !result.failed.is_empty() {
            for fg in &result.failed {
                // Check for FOD hash mismatch in the error for --fix support.
                if fix {
                    if let Some((name, expected, actual)) = parse_fod_mismatch_error(&fg.error) {
                        let (label, drv_path) = drv_paths
                            .iter()
                            .find(|(_, sp)| sp.name().contains(name.as_str()))
                            .unwrap_or(&drv_paths[0]);
                        return handle_fod_mismatch(
                            &name, &expected, &actual, drv_path, label,
                            log_dir, source_file, fix,
                        );
                    }
                }
                eprintln!("FAILED: {}", fg.drv_key);
                eprintln!("  {}", fg.error);
            }
            return Err(RunError::Build(format!(
                "{} root build(s) failed",
                result.failed.len(),
            )));
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        return Err(RunError::Internal(
            "building is only supported on Linux (requires bwrap)".to_string(),
        ));
    }

    Ok(())
}

// Service construction moved to crunch_store::StoreHandle::open().

/// Handle a FOD hash mismatch: optionally auto-fix, always log.
fn handle_fod_mismatch(
    name: &str,
    expected_sri: &str,
    actual_sri: &str,
    drv_path: &StorePath<String>,
    label: &str,
    log_dir: &std::path::Path,
    source_file: &std::path::Path,
    fix: bool,
) -> Result<(), RunError> {
    let msg = format!(
        "hash mismatch for '{name}':\n\
         \x20 expected: {expected_sri}\n\
         \x20 got:      {actual_sri}"
    );

    if fix {
        match auto_fix_hash(source_file, expected_sri, actual_sri) {
            Ok(()) => {
                eprintln!("{msg}");
                eprintln!("  fixed: updated {} with correct hash", source_file.display());
                write_log(log_dir, drv_path, label, false, &msg);
                return Err(RunError::Build(format!(
                    "{msg}\n  fixed: re-run to build with the corrected hash"
                )));
            }
            Err(fix_err) => {
                eprintln!("{msg}");
                eprintln!("  --fix failed: {fix_err}");
            }
        }
    } else {
        eprintln!("{msg}");
        eprintln!(
            "  update {}: hash = \"{actual_sri}\"",
            source_file.display()
        );
    }

    write_log(log_dir, drv_path, label, false, &msg);
    Err(RunError::Build(msg))
}

// ── cmd_log ────────────────────────────────────────────────────────────

fn cmd_log(query: Option<&str>, list: bool) -> Result<(), RunError> {
    let dir = log_dir();
    if !dir.exists() {
        return Err(RunError::Internal(format!(
            "log directory {} does not exist (no builds yet?)",
            dir.display()
        )));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| RunError::Internal(format!("reading log dir: {e}")))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if list || query.is_none() {
        if entries.is_empty() {
            eprintln!("No build logs found in {}", dir.display());
            return Ok(());
        }
        for entry in &entries {
            let path = entry.path();
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            if let Ok(content) = std::fs::read_to_string(&path) {
                let status = content.lines()
                    .find(|l| l.starts_with("# status:"))
                    .map(|l| l.trim_start_matches("# status: "))
                    .unwrap_or("unknown");
                let drv_label = content.lines()
                    .find(|l| l.starts_with("# derivation:"))
                    .map(|l| l.trim_start_matches("# derivation: "))
                    .unwrap_or("");
                println!("{name}  [{status}]  {drv_label}");
            } else {
                println!("{name}");
            }
        }
        return Ok(());
    }

    let Some(query) = query else {
        return Err(RunError::Internal("log query is None after guard".to_string()));
    };
    let matched = entries.iter().find(|e| {
        let name = e.file_name();
        let name_str = name.to_string_lossy();
        name_str.contains(query)
    });

    match matched {
        Some(entry) => {
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| RunError::Internal(format!("reading log: {e}")))?;
            print!("{content}");
            Ok(())
        }
        None => Err(RunError::Internal(format!(
            "no log matching '{query}' in {}", dir.display()
        ))),
    }
}

// ── cmd_self_build ─────────────────────────────────────────────────────

fn cmd_self_build(
    output_dir: &std::path::Path,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    no_verify: bool,
) -> Result<(), RunError> {
    eprintln!("=== crunch self-build ===");

    // Locate the source tree.
    let src_dir = find_source_dir()?;
    eprintln!("source: {}", src_dir.display());

    let bootstrap_dir = src_dir.join("bootstrap");
    if !bootstrap_dir.exists() {
        return Err(RunError::Internal(format!(
            "bootstrap/ directory not found at {}",
            bootstrap_dir.display(),
        )));
    }

    // 1. Stage source tree into the output store.
    //    No tarball, no NAR hash, no FOD. Just copy the files.
    eprintln!("\n[1/3] Staging source...");
    let store_name = self_build::stage_source(&src_dir, output_dir)?;

    // 2. Generate .ncl and build.
    eprintln!("\n[2/3] Building...");
    let ncl_content = self_build::generate_self_build_ncl(
        &store_name,
        nix_compat::store_path::STORE_DIR,
    );

    let tmp_dir = tempfile::tempdir()
        .map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let ncl_path = tmp_dir.path().join("self-build.ncl");
    std::fs::write(&ncl_path, &ncl_content)
        .map_err(|e| RunError::Internal(format!("writing ncl: {e}")))?;

    let lib_dir = src_dir.join("lib");
    let import_paths: Vec<OsString> = {
        let mut paths = build_import_paths(&[lib_dir])?;
        paths.push(bootstrap_dir.into());
        paths
    };

    let sub_url = if no_substitute {
        None
    } else {
        Some("https://cache.nixos.org")
    };

    cmd_build(
        &ncl_path,
        &import_paths,
        output_dir,
        nix_compat::store_path::STORE_DIR,
        verbose,
        false,
        max_jobs,
        sub_url,
    )?;

    // 3. Verify the output binary.
    if !no_verify {
        eprintln!("\n[3/3] Verifying output...");
        let crunch_bin = find_self_built_binary(output_dir)?;
        self_build::verify_binary(&crunch_bin)?;
    } else {
        eprintln!("\n[3/3] Verification skipped (--no-verify).");
    }

    eprintln!("\n=== self-build complete ===");
    Ok(())
}

/// Find the crunch source directory.
///
/// Walks up from the current executable or cwd looking for Cargo.toml
/// with `name = "crunch"`. Falls back to cwd if it has bootstrap/.
fn find_source_dir() -> Result<PathBuf, RunError> {
    // Try cwd first.
    let cwd = std::env::current_dir()
        .map_err(|e| RunError::Internal(format!("cwd: {e}")))?;

    if cwd.join("Cargo.toml").exists() && cwd.join("bootstrap").exists() {
        return Ok(cwd);
    }

    // Check if we're inside the crunch tree.
    let mut dir = cwd.as_path();
    for _ in 0..8_u32 {
        if dir.join("Cargo.toml").exists() && dir.join("bootstrap").exists() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => break,
        }
    }

    Err(RunError::Internal(
        "could not locate crunch source directory.\n\
         Run `crunch self-build` from the crunch repo root."
            .to_string(),
    ))
}

/// Find the self-built crunch binary in the output directory.
///
/// Scans for `*-crunch/bin/crunch`.
fn find_self_built_binary(output_dir: &std::path::Path) -> Result<PathBuf, RunError> {
    use std::fs;

    let entries = fs::read_dir(output_dir)
        .map_err(|e| RunError::Internal(format!(
            "reading output dir {}: {e}", output_dir.display()
        )))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            let bin = entry.path().join("bin").join("crunch");
            if bin.exists() {
                return Ok(bin);
            }
        }
    }

    Err(RunError::Internal(format!(
        "could not find self-built crunch binary in {}",
        output_dir.display(),
    )))
}

// ── cmd_bootstrap ──────────────────────────────────────────────────────

fn cmd_bootstrap_fetch(
    output: &std::path::Path,
    store_dir: &std::path::Path,
    verbose: bool,
) -> Result<(), RunError> {
    if !store_dir.exists() {
        return Err(RunError::Internal(format!(
            "store directory {} does not exist.\n\
             Create it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            store_dir.display()
        )));
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async {
        bootstrap::bootstrap_fetch(store_dir, output, verbose).await
    })
}

fn cmd_bootstrap(output: &std::path::Path, packages: &[String]) -> Result<(), RunError> {
    eprintln!("Resolving store paths for {} packages...", packages.len());

    let entries = bootstrap::resolve_packages(packages, |pkg| {
        let path = bootstrap::nix_resolve(pkg)?;
        eprintln!("  {pkg} -> {path}");
        Ok(path)
    })?;

    let ncl = bootstrap::generate_seed_ncl(&entries);

    std::fs::write(output, &ncl)
        .map_err(|e| RunError::Internal(format!("writing {}: {e}", output.display())))?;

    eprintln!("Wrote {}", output.display());
    Ok(())
}

// ── cmd_store: list / info / verify ────────────────────────────────────

fn cmd_store(action: StoreAction) -> Result<(), RunError> {
    use snix_store::pathinfoservice::{RedbPathInfoService, RedbPathInfoServiceConfig};

    let state = state_dir();
    let db_path = state.join("pathinfo.redb");
    if !db_path.exists() {
        return Err(RunError::Internal(format!(
            "PathInfo database {} does not exist (no builds yet?)",
            db_path.display()
        )));
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async {
        let read_only = matches!(action, StoreAction::List | StoreAction::Info { .. });
        let svc = RedbPathInfoService::new(
            "crunch".to_string(),
            RedbPathInfoServiceConfig {
                path: Some(db_path.clone()),
                read_only,
                cache_size: None,
            },
        )
        .await
        .map_err(|e| RunError::Internal(format!("opening PathInfo database: {e}")))?;

        match action {
            StoreAction::List => cmd_store_list(&svc).await,
            StoreAction::Info { path } => cmd_store_info(&svc, &path).await,
            StoreAction::Verify { path } => cmd_store_verify(&svc, path.as_deref()).await,
        }
    })
}

async fn cmd_store_list(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
) -> Result<(), RunError> {
    let entries = crunch_store::store_list(svc).await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    if entries.is_empty() {
        eprintln!("No paths in PathInfo database.");
    } else {
        for (store_path, deriver, nar_size) in &entries {
            println!("{store_path}  deriver={deriver}  nar_size={nar_size}");
        }
        eprintln!("{} path(s)", entries.len());
    }
    Ok(())
}

async fn cmd_store_info(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path: &str,
) -> Result<(), RunError> {
    let details = crunch_store::store_info(svc, path).await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    if details.is_empty() {
        return Err(RunError::Internal(format!(
            "no PathInfo matching '{path}'"
        )));
    }
    for d in &details {
        println!("store_path: {}", d.store_path);
        println!("nar_size:   {}", d.nar_size);
        println!("nar_sha256: {}", data_encoding::HEXLOWER.encode(&d.nar_sha256));
        if let Some(ref deriver) = d.deriver {
            println!("deriver:    {deriver}");
        }
        if !d.references.is_empty() {
            println!("references:");
            for r in &d.references {
                println!("  {r}");
            }
        }
        if let Some(ref ca) = d.ca {
            println!("ca:         {ca}");
        }
        println!("node:       {}", d.node);
    }
    Ok(())
}

async fn cmd_store_verify(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
) -> Result<(), RunError> {
    let results = crunch_store::store_verify(svc, path_filter).await
        .map_err(|e| RunError::Internal(format!("{e}")))?;

    let mut checked: u32 = 0;
    let mut mismatches: u32 = 0;
    for result in &results {
        match result {
            crunch_store::VerifyResult::Ok(path) => {
                println!("OK {path}");
            }
            crunch_store::VerifyResult::Missing(path) => {
                println!("MISSING {path}");
                mismatches = mismatches.saturating_add(1);
            }
            crunch_store::VerifyResult::Mismatch { path, stored_hash, actual_hash } => {
                println!("MISMATCH {path}  stored={stored_hash}  actual={actual_hash}");
                mismatches = mismatches.saturating_add(1);
            }
        }
        checked = checked.saturating_add(1);
    }

    eprintln!("{checked} checked, {mismatches} mismatches");
    if mismatches > 0 {
        return Err(RunError::Build(format!(
            "{mismatches} path(s) failed verification"
        )));
    }
    Ok(())
}

// ── Utilities ──────────────────────────────────────────────────────────

/// Parse a FOD hash mismatch from an error string.
/// Returns `(name, expected_sri, actual_sri)` if the error matches.
fn parse_fod_mismatch_error(err: &str) -> Option<(String, String, String)> {
    // Format: "FOD hash mismatch for NAME: expected EXPECTED, got ACTUAL"
    let rest = err.strip_prefix("FOD hash mismatch for ")?;
    let (name, rest) = rest.split_once(": expected ")?;
    let (expected, actual) = rest.split_once(", got ")?;
    Some((name.to_string(), expected.to_string(), actual.to_string()))
}

/// Replace an old hash string with a new one in a .ncl file.
fn auto_fix_hash(
    file: &std::path::Path,
    old_hash: &str,
    new_hash: &str,
) -> Result<(), String> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| format!("reading {}: {e}", file.display()))?;

    let count = content.matches(old_hash).count();
    if count == 0 {
        return Err(format!(
            "hash '{}' not found in {}",
            old_hash, file.display()
        ));
    }
    if count > 1 {
        return Err(format!(
            "hash '{}' appears {} times in {} — ambiguous, not fixing",
            old_hash, count, file.display()
        ));
    }

    let fixed = content.replacen(old_hash, new_hash, 1);
    std::fs::write(file, &fixed)
        .map_err(|e| format!("writing {}: {e}", file.display()))?;
    Ok(())
}

fn write_log(
    log_dir: &std::path::Path,
    drv_path: &StorePath<String>,
    label: &str,
    success: bool,
    body: &str,
) {
    let log_file = log_dir.join(format!("{}.log", drv_path));
    let status = if success { "success" } else { "failure" };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let content = format!(
        "# crunch build log\n# derivation: {label}\n# drv_path: {drv_path}\n# status: {status}\n# timestamp: {timestamp}\n\n{body}\n"
    );
    let _ = std::fs::write(&log_file, &content);
}

/// Resolve the crunch state directory.
fn state_dir() -> PathBuf {
    std::env::var("CRUNCH_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let state = std::env::var("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    PathBuf::from(home).join(".local/state")
                });
            state.join("crunch")
        })
}

fn log_dir() -> PathBuf {
    std::env::var("CRUNCH_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| state_dir().join("logs"))
}

fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let stdlib_dir = crunch_eval::stdlib::stdlib_import_path()
        .map_err(|e| RunError::Internal(format!("stdlib: {e}")))?;
    let mut paths: Vec<OsString> = vec![stdlib_dir.into()];
    for p in extra {
        paths.push(p.into());
    }
    Ok(paths)
}

// Remote PathInfo construction moved to crunch_store::StoreHandle::open().
