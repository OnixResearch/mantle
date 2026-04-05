mod bootstrap;
mod errors;

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

    /// Physical directory for build outputs (source inputs always
    /// come from /nix/store)
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

        /// Packages to include (nixpkgs attribute names)
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
        Command::Build { file, import_paths, fix, jobs } => {
            let import_paths = build_import_paths(&import_paths)?;
            let max_jobs = resolve_max_jobs(jobs);
            cmd_build(&file, &import_paths, &args.store, args.verbose, fix, max_jobs)
        }
        Command::Bootstrap { output, packages } => cmd_bootstrap(&output, &packages),
        Command::Log { query, list } => cmd_log(query.as_deref(), list),
        Command::Store { action } => cmd_store(action),
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
) -> Result<(), RunError> {
    if !output_dir.exists() {
        return Err(RunError::Internal(format!(
            "output store directory {} does not exist.\n\
             Create it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            output_dir.display()
        )));
    }

    // Phase 1: Evaluate Nickel expression (sync, fast).
    info!(file = %file.display(), "evaluating");
    let expr = crunch_eval::evaluate(file, import_paths)
        .map_err(|e| RunError::Eval(format!("{e}")))?;
    let derivations = deserialize_derivations(&expr)?;
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
        )
        .await
    })
}



/// Deserialize a Nickel expression into a list of named derivations.
///
/// Handles both single-derivation records (has a "name" field) and
/// package sets (each field is a derivation).
fn deserialize_derivations(
    expr: &crunch_eval::Expr,
) -> Result<Vec<(String, crunch_glue::CrunchDerivation)>, RunError> {
    let Some(record) = expr.as_record() else {
        return Err(RunError::Eval(
            "expected a Derivation record or a record of Derivations".to_string(),
        ));
    };

    if record.value_by_name("name").is_some_and(|v| v.as_str().is_some()) {
        let drv: crunch_glue::CrunchDerivation = expr.to_serde()
            .map_err(|e| RunError::Eval(format!("deserializing derivation: {e}")))?;
        let name = drv.name.clone();
        return Ok(vec![(name, drv)]);
    }

    let mut derivations = Vec::new();
    for (key, maybe_value) in record.iter() {
        let value = maybe_value.ok_or_else(|| {
            RunError::Eval(format!("field '{key}' has no value"))
        })?;
        let drv: crunch_glue::CrunchDerivation = value.to_serde()
            .map_err(|e| RunError::Eval(format!("deserializing derivation '{key}': {e}")))?;
        derivations.push((key.to_string(), drv));
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
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};

    let blob_service = MemoryBlobService::default();
    let directory_service = RedbDirectoryService::new_temporary(
        "crunch".to_string(),
        RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        },
    )
    .map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

    let state_dir = state_dir();
    let _ = std::fs::create_dir_all(&state_dir);
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

            for (_output_name, path_info) in &outcome.outputs {
                let path = path_info.store_path
                    .to_absolute_path_with_prefix(output_dir_str);
                if outcome.cached {
                    println!("{path} (cached)");
                } else {
                    println!("{path}");
                }
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
) -> Result<(), RunError> {
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
    use tokio::sync::mpsc;

    let blob_service = MemoryBlobService::default();
    let directory_service = RedbDirectoryService::new_temporary(
        "crunch".to_string(),
        RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        },
    )
    .map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

    let state_dir = state_dir();
    let _ = std::fs::create_dir_all(&state_dir);
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

            for (_output_name, path_info) in &outcome.outputs {
                let path = path_info.store_path
                    .to_absolute_path_with_prefix(output_dir_str);
                if outcome.cached {
                    println!("{path} (cached)");
                } else {
                    println!("{path}");
                }
            }
        }

        if !result.failed.is_empty() {
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

    let query = query.unwrap();
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

// ── cmd_bootstrap ──────────────────────────────────────────────────────

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
