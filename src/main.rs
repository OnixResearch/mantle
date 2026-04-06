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
            cmd_build(&file, &import_paths, &args.store, args.verbose, fix, max_jobs, sub_url.as_deref())
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

/// The logical store prefix. Derivation paths, output hashing, sandbox
/// layout, and KnownPaths lookups always use this.
const LOGICAL_STORE_DIR: &str = "/nix/store";

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

    let output_dir_str = output_dir.to_str().unwrap_or(LOGICAL_STORE_DIR);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async {
        execute_builds_streaming(
            derivations,
            output_dir,
            output_dir_str,
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

    let Some(obj) = json_val.as_object() else {
        return Err(RunError::Eval(
            "expected a Derivation record or a record of Derivations".to_string(),
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
    known_paths: &mut crunch_glue::KnownPaths,
    output_dir: &std::path::Path,
    output_dir_str: &str,
    log_dir: &std::path::Path,
    source_file: &std::path::Path,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
) -> Result<(), RunError> {
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};

    let state_dir = state_dir();
    let _ = std::fs::create_dir_all(&state_dir);

    let blob_service = open_blob_service(&state_dir)?;
    let directory_service = RedbDirectoryService::new_temporary(
        "crunch".to_string(),
        RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        },
    )
    .map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

    let pathinfo_service = open_pathinfo_service(&state_dir).await?;

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
            Some(state_dir.clone()),
            None, // no remote substitution in legacy path
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
                (sp.to_absolute_path_with_prefix(LOGICAL_STORE_DIR), label.as_str())
            })
            .collect();

        for outcome in &outcomes {
            let drv_abs = outcome.drv_path
                .to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
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
    log_dir: &std::path::Path,
    source_file: &std::path::Path,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_url: Option<&str>,
) -> Result<(), RunError> {
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
    use tokio::sync::mpsc;

    let state_dir = state_dir();
    let _ = std::fs::create_dir_all(&state_dir);

    let blob_service = open_blob_service(&state_dir)?;
    let directory_service = RedbDirectoryService::new_temporary(
        "crunch".to_string(),
        RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        },
    )
    .map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

    let pathinfo_service = open_pathinfo_service(&state_dir).await?;

    // Build the remote PathInfoService for binary cache substitution.
    let remote_pathinfo: Option<
        std::sync::Arc<dyn snix_store::pathinfoservice::PathInfoService>,
    > = match substituter_url {
        Some(url_str) => {
            match build_remote_pathinfo(
                url_str,
                blob_service.clone(),
                directory_service.clone(),
            ) {
                Ok(svc) => {
                    info!(url = %url_str, "binary cache substitution enabled");
                    Some(svc)
                }
                Err(e) => {
                    tracing::warn!(
                        url = %url_str,
                        err = %e,
                        "failed to configure remote cache, substitution disabled"
                    );
                    None
                }
            }
        }
        None => None,
    };

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
            Some(state_dir.clone()),
            remote_pathinfo,
            verbose,
        );

        // Convert all derivations (sync, fast — CPU-only, no I/O).
        // KnownPaths is fully populated before builds start.
        let mut known_paths = crunch_glue::KnownPaths::new(LOGICAL_STORE_DIR);
        let mut drv_paths: Vec<(String, StorePath<String>)> = Vec::new();

        for (label, drv) in &derivations {
            let (drv_path, _nix_drv) = crunch_glue::convert(drv, &mut known_paths)
                .map_err(|e| RunError::Build(format!("{label}: {e}")))?;
            info!(drv = %drv_path, label = %label, "derivation constructed");
            drv_paths.push((label.clone(), drv_path));
        }

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
                (sp.to_absolute_path_with_prefix(LOGICAL_STORE_DIR), label.as_str())
            })
            .collect();

        for outcome in &result.outcomes {
            let drv_abs = outcome.drv_path
                .to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
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

/// Open the persistent PathInfo database, falling back to in-memory.
async fn open_pathinfo_service(
    state_dir: &std::path::Path,
) -> Result<
    snix_store::pathinfoservice::RedbPathInfoService,
    RunError,
> {
    use snix_store::pathinfoservice::{RedbPathInfoService, RedbPathInfoServiceConfig};

    let db_path = state_dir.join("pathinfo.redb");
    match RedbPathInfoService::new(
        "crunch".to_string(),
        RedbPathInfoServiceConfig {
            path: Some(db_path.clone()),
            read_only: false,
            cache_size: None,
        },
    )
    .await
    {
        Ok(svc) => {
            info!(path = %db_path.display(), "PathInfo database opened");
            Ok(svc)
        }
        Err(e) => {
            tracing::warn!(
                path = %db_path.display(),
                err = %e,
                "failed to open PathInfo database, using in-memory fallback"
            );
            RedbPathInfoService::new_temporary(
                "crunch".to_string(),
                RedbPathInfoServiceConfig::default(),
            )
            .map_err(|e| RunError::Internal(format!("in-memory PathInfo: {e}")))
        }
    }
}

/// Open or create a persistent blob service backed by the local filesystem.
///
/// Blobs are stored as content-addressed chunks under `{state_dir}/blobs/`.
/// If the directory doesn't exist, it's created. Falls back to in-memory
/// if the filesystem path can't be opened (e.g., read-only mount).
fn open_blob_service(
    state_dir: &std::path::Path,
) -> Result<std::sync::Arc<snix_castore::blobservice::ObjectStoreBlobService>, RunError> {
    use snix_castore::blobservice::ObjectStoreBlobService;

    let blob_dir = state_dir.join("blobs");
    std::fs::create_dir_all(&blob_dir)
        .map_err(|e| RunError::Internal(format!("creating blob dir {}: {e}", blob_dir.display())))?;

    let svc = ObjectStoreBlobService::new_local(&blob_dir)
        .map_err(|e| RunError::Internal(format!("opening blob service at {}: {e}", blob_dir.display())))?;

    info!(path = %blob_dir.display(), "blob service opened");
    Ok(std::sync::Arc::new(svc))
}

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

    // Locate the source tree (directory containing Cargo.toml).
    let src_dir = find_source_dir()?;
    eprintln!("source: {}", src_dir.display());

    // Locate bootstrap/ directory (for import paths).
    let bootstrap_dir = src_dir.join("bootstrap");
    if !bootstrap_dir.exists() {
        return Err(RunError::Internal(format!(
            "bootstrap/ directory not found at {}",
            bootstrap_dir.display(),
        )));
    }

    // 1. Create the source tarball.
    eprintln!("\n[1/4] Creating source tarball...");
    let tmp_dir = tempfile::tempdir()
        .map_err(|e| RunError::Internal(format!("creating temp dir: {e}")))?;
    let tarball = self_build::create_source_tarball(&src_dir, tmp_dir.path())?;

    // 2. Compute NAR hash of the unpacked tarball.
    eprintln!("\n[2/4] Computing NAR hash...");
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio: {e}")))?;
    let tarball_hash = rt.block_on(self_build::hash_unpacked_tarball(&tarball))?;

    // 3. Generate the .ncl and build.
    eprintln!("\n[3/4] Building...");
    let tarball_url = format!("file://{}", tarball.display());
    let ncl_content = self_build::generate_self_build_ncl(&tarball_url, &tarball_hash);

    let ncl_path = tmp_dir.path().join("self-build.ncl");
    std::fs::write(&ncl_path, &ncl_content)
        .map_err(|e| RunError::Internal(format!("writing ncl: {e}")))?;

    // Import paths: bootstrap/ (for make.ncl etc) + lib/ (for lib.ncl).
    let lib_dir = src_dir.join("lib");
    let import_paths: Vec<OsString> = {
        let mut paths = build_import_paths(&[lib_dir.clone()])?;
        paths.push(bootstrap_dir.clone().into());
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
        verbose,
        false, // no --fix
        max_jobs,
        sub_url,
    )?;

    // 4. Verify the output binary.
    if !no_verify {
        eprintln!("\n[4/4] Verifying output...");
        // Find the crunch binary in the output dir.
        let crunch_bin = find_self_built_binary(output_dir)?;
        self_build::verify_binary(&crunch_bin)?;
    } else {
        eprintln!("\n[4/4] Verification skipped (--no-verify).");
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
    use futures::StreamExt;

    let mut stream = svc.list();
    let mut count: u32 = 0;
    while let Some(result) = stream.next().await {
        let pi = result.map_err(|e| RunError::Internal(format!("listing: {e}")))?;
        let deriver_name = pi
            .deriver
            .as_ref()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{}  deriver={}  nar_size={}",
            pi.store_path, deriver_name, pi.nar_size
        );
        count = count.saturating_add(1);
    }
    if count == 0 {
        eprintln!("No paths in PathInfo database.");
    } else {
        eprintln!("{count} path(s)");
    }
    Ok(())
}

async fn cmd_store_info(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path: &str,
) -> Result<(), RunError> {
    use futures::StreamExt;

    let mut stream = svc.list();
    let mut found = false;
    while let Some(result) = stream.next().await {
        let pi = result.map_err(|e| RunError::Internal(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();
        if !sp_str.contains(path) {
            continue;
        }
        println!("store_path: {}", pi.store_path);
        println!("nar_size:   {}", pi.nar_size);
        println!(
            "nar_sha256: {}",
            data_encoding::HEXLOWER.encode(&pi.nar_sha256)
        );
        if let Some(ref d) = pi.deriver {
            println!("deriver:    {d}");
        }
        if !pi.references.is_empty() {
            println!("references:");
            for r in &pi.references {
                println!("  {r}");
            }
        }
        if let Some(ref ca) = pi.ca {
            println!("ca:         {ca:?}");
        }
        println!("node:       {:?}", pi.node);
        found = true;
    }
    if !found {
        return Err(RunError::Internal(format!(
            "no PathInfo matching '{path}'"
        )));
    }
    Ok(())
}

async fn cmd_store_verify(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
) -> Result<(), RunError> {
    use futures::StreamExt;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
    use snix_castore::import::fs::ingest_path;
    use snix_store::nar::{NarCalculationService, SimpleRenderer};

    let bs = MemoryBlobService::default();
    let ds = RedbDirectoryService::new_temporary(
        "verify".to_string(),
        RedbDirectoryServiceConfig::default(),
    )
    .map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

    let mut stream = svc.list();
    let mut checked: u32 = 0;
    let mut mismatches: u32 = 0;

    while let Some(result) = stream.next().await {
        let pi = result.map_err(|e| RunError::Internal(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();

        if let Some(filter) = path_filter {
            if !sp_str.contains(filter) {
                continue;
            }
        }

        let abs = std::path::Path::new("/nix/store").join(sp_str);
        if !abs.exists() {
            println!("MISSING {}", pi.store_path);
            mismatches = mismatches.saturating_add(1);
            checked = checked.saturating_add(1);
            continue;
        }

        let node = ingest_path::<_, _, _, &[u8]>(bs.clone(), ds.clone(), &abs, None)
            .await
            .map_err(|e| RunError::Internal(format!("ingest {}: {e}", pi.store_path)))?;

        let renderer = SimpleRenderer::new(bs.clone(), ds.clone());
        let (_nar_size, nar_sha256) = renderer
            .calculate_nar(&node)
            .await
            .map_err(|e| RunError::Internal(format!("NAR calc: {e}")))?;

        if nar_sha256 == pi.nar_sha256 {
            println!("OK {}", pi.store_path);
        } else {
            println!(
                "MISMATCH {}  stored={}  actual={}",
                pi.store_path,
                data_encoding::HEXLOWER.encode(&pi.nar_sha256),
                data_encoding::HEXLOWER.encode(&nar_sha256),
            );
            mismatches = mismatches.saturating_add(1);
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

/// Construct a `NixHTTPPathInfoService` from a cache URL string.
///
/// Shares the caller's blob/directory services so downloaded NARs are
/// immediately available to the Builder.
fn build_remote_pathinfo<BS, DS>(
    url_str: &str,
    blob_service: BS,
    directory_service: DS,
) -> Result<std::sync::Arc<dyn snix_store::pathinfoservice::PathInfoService>, RunError>
where
    BS: snix_castore::blobservice::BlobService + Send + Sync + Clone + 'static,
    DS: snix_castore::directoryservice::DirectoryService + Send + Sync + Clone + 'static,
{
    use snix_store::pathinfoservice::{NixHTTPPathInfoService, NixHTTPPathInfoServiceConfig};

    // NixHTTPPathInfoServiceConfig::try_from expects "nix+https://..." scheme.
    let nix_url_str = format!("nix+{url_str}");
    let nix_url: url::Url = nix_url_str.parse()
        .map_err(|e| RunError::Internal(format!("invalid substituter URL '{url_str}': {e}")))?;

    let config: NixHTTPPathInfoServiceConfig = nix_url.try_into()
        .map_err(|e| RunError::Internal(format!("remote cache config for '{url_str}': {e}")))?;

    let svc = NixHTTPPathInfoService::try_build(
        "crunch-remote".to_string(),
        config,
        blob_service,
        directory_service,
    ).map_err(|e| RunError::Internal(format!("building remote cache client: {e}")))?;

    Ok(std::sync::Arc::new(svc))
}
