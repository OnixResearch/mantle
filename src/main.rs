mod bootstrap;
mod errors;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
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

    /// Store directory
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
        Command::Build { file, import_paths, fix } => {
            let import_paths = build_import_paths(&import_paths)?;
            cmd_build(&file, &import_paths, &args.store, args.verbose, fix)
        }
        Command::Bootstrap { output, packages } => cmd_bootstrap(&output, &packages),
        Command::Log { query, list } => cmd_log(query.as_deref(), list),
        Command::Store { action } => cmd_store(action),
    }
}

fn cmd_build(
    file: &std::path::Path,
    import_paths: &[OsString],
    store_dir: &std::path::Path,
    verbose: bool,
    fix: bool,
) -> Result<(), RunError> {
    // 0. Verify store directory exists (or explain how to create it)
    if !store_dir.exists() {
        return Err(RunError::Internal(format!(
            "store directory {} does not exist.\n\
             Create it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            store_dir.display()
        )));
    }

    // 1. Evaluate Nickel file
    info!(file = %file.display(), "evaluating");
    let expr = crunch_eval::evaluate(file, import_paths)
        .map_err(|e| RunError::Eval(format!("{e}")))?;

    // 2. Detect single vs multi-derivation output.
    //    If the top-level record has a "name" field, treat as single derivation.
    //    Otherwise, treat each field as a named derivation.
    let derivations: Vec<(String, crunch_glue::CrunchDerivation)> = if let Some(record) = expr.as_record() {
        if record.value_by_name("name").is_some_and(|v| v.as_str().is_some()) {
            // Single derivation -- deserialize the whole expression directly
            let drv: crunch_glue::CrunchDerivation = expr.to_serde()
                .map_err(|e| RunError::Eval(format!("deserializing derivation: {e}")))?;
            let name = drv.name.clone();
            vec![(name, drv)]
        } else {
            // Package set -- each field is a derivation
            let mut derivations = Vec::new();
            for (key, maybe_value) in record.iter() {
                let value = maybe_value.ok_or_else(|| RunError::Eval(
                    format!("field '{key}' has no value")
                ))?;
                let drv: crunch_glue::CrunchDerivation = value.to_serde()
                    .map_err(|e| RunError::Eval(format!(
                        "deserializing derivation '{key}': {e}"
                    )))?;
                derivations.push((key.to_string(), drv));
            }
            derivations
        }
    } else {
        return Err(RunError::Eval(
            "expected a Derivation record or a record of Derivations".to_string(),
        ));
    };

    // 3. Convert all derivations
    let store_dir_str = store_dir.to_str().unwrap_or("/nix/store");
    let mut known_paths = crunch_glue::KnownPaths::new(store_dir_str);
    let mut drv_paths = Vec::new();
    for (label, drv) in &derivations {
        let (drv_path, _nix_drv) = crunch_glue::convert(drv, &mut known_paths)
            .map_err(|e| RunError::Build(format!("{label}: {e}")))?;
        info!(drv = %drv_path, label = %label, "derivation constructed");
        drv_paths.push((label.clone(), drv_path));
    }

    // 4. Set up build log directory
    let log_dir = log_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    // 5. Set up build services and build each derivation
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async {
        use snix_castore::blobservice::MemoryBlobService;
        use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
        use snix_store::pathinfoservice::{RedbPathInfoService, RedbPathInfoServiceConfig};

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

        // PathInfoService: persistent redb, fallback to in-memory
        let state_dir = state_dir();
        let _ = std::fs::create_dir_all(&state_dir);
        let pathinfo_db_path = state_dir.join("pathinfo.redb");
        let pathinfo_service = match RedbPathInfoService::new(
            "crunch".to_string(),
            RedbPathInfoServiceConfig {
                path: Some(pathinfo_db_path.clone()),
                read_only: false,
                cache_size: None,
            },
        ).await {
            Ok(svc) => {
                info!(path = %pathinfo_db_path.display(), "PathInfo database opened");
                svc
            }
            Err(e) => {
                tracing::warn!(
                    path = %pathinfo_db_path.display(),
                    err = %e,
                    "failed to open PathInfo database, using in-memory fallback"
                );
                RedbPathInfoService::new_temporary(
                    "crunch".to_string(),
                    RedbPathInfoServiceConfig::default(),
                ).map_err(|e| RunError::Internal(format!("in-memory PathInfo: {e}")))?            }
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
                store_dir.to_path_buf(),
                Some(state_dir.clone()),
                verbose,
            );

            for (label, drv_path) in &drv_paths {
                let outcome = match builder
                    .build(drv_path, &mut known_paths)
                    .await
                {
                    Ok(o) => o,
                    Err(crunch_build::Error::FodHashMismatch {
                        ref name,
                        ref expected_sri,
                        ref actual_sri,
                    }) => {
                        let msg = format!(
                            "hash mismatch for '{name}':\n\
                             \x20 expected: {expected_sri}\n\
                             \x20 got:      {actual_sri}"
                        );
                        if fix {
                            match auto_fix_hash(file, expected_sri, actual_sri) {
                                Ok(()) => {
                                    eprintln!("{msg}");
                                    eprintln!("  fixed: updated {} with correct hash", file.display());
                                    // Rebuild: return the mismatch as an error so the
                                    // user re-runs. A full retry loop is future work.
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
                                file.display()
                            );
                        }
                        write_log(&log_dir, drv_path, label, false, &msg);
                        return Err(RunError::Build(msg));
                    }
                    Err(e) => {
                        let log_msg = format!("{e}");
                        write_log(&log_dir, drv_path, label, false, &log_msg);
                        return Err(RunError::Build(log_msg));
                    }
                };

                // Persist the build log (success)
                if let Some(ref log) = outcome.log {
                    write_log(&log_dir, drv_path, label, true, log);
                    if verbose {
                        eprintln!("--- build log: {label} ---");
                        eprintln!("{log}");
                        eprintln!("--- end log ---");
                    }
                } else if !outcome.cached {
                    write_log(&log_dir, drv_path, label, true, "(no output captured)");
                }

                for (_output_name, path_info) in &outcome.outputs {
                    let path = path_info.store_path.to_absolute_path_with_prefix(store_dir_str);
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
    })
}

fn cmd_log(query: Option<&str>, list: bool) -> Result<(), RunError> {
    let dir = log_dir();
    if !dir.exists() {
        return Err(RunError::Internal(format!(
            "log directory {} does not exist (no builds yet?)",
            dir.display()
        )));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| RunError::Internal(format!("reading log dir: {e}")))?.
        filter_map(|e| e.ok())
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
            // Read first few lines to get metadata
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

fn cmd_store(action: StoreAction) -> Result<(), RunError> {
    use snix_store::pathinfoservice::{PathInfoService, RedbPathInfoService, RedbPathInfoServiceConfig};
    use futures::StreamExt;

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
            StoreAction::List => {
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
            }

            StoreAction::Info { path } => {
                // Find the matching PathInfo by scanning
                let mut stream = svc.list();
                let mut found = false;
                while let Some(result) = stream.next().await {
                    let pi = result.map_err(|e| RunError::Internal(format!("listing: {e}")))?;
                    let sp_str = pi.store_path.to_string();
                    if sp_str.contains(&path) {
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
                }
                if !found {
                    return Err(RunError::Internal(format!(
                        "no PathInfo matching '{path}'"
                    )));
                }
            }

            StoreAction::Verify { path } => {
                use snix_castore::blobservice::MemoryBlobService;
                use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
                use snix_castore::import::fs::ingest_path;
                use snix_store::nar::{NarCalculationService, SimpleRenderer};

                let bs = MemoryBlobService::default();
                let ds = RedbDirectoryService::new_temporary(
                    "verify".to_string(),
                    RedbDirectoryServiceConfig::default(),
                ).map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

                let mut stream = svc.list();
                let mut checked: u32 = 0;
                let mut mismatches: u32 = 0;

                while let Some(result) = stream.next().await {
                    let pi = result.map_err(|e| RunError::Internal(format!("listing: {e}")))?;
                    let sp_str = pi.store_path.to_string();

                    if let Some(ref filter) = path {
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

                    let node = ingest_path::<_, _, _, &[u8]>(
                        bs.clone(), ds.clone(), &abs, None,
                    )
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
            }
        }

        Ok(())
    })
}

/// Write a build log file with metadata header.
/// Replace an old hash string with a new one in a .ncl file.
///
/// Reads the file, finds the old hash (must appear exactly once),
/// replaces it, and writes back. Returns an error if the old hash
/// appears zero or more than one time.
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
            old_hash,
            file.display()
        ));
    }
    if count > 1 {
        return Err(format!(
            "hash '{}' appears {} times in {} — ambiguous, not fixing",
            old_hash,
            count,
            file.display()
        ));
    }

    let fixed = content.replacen(old_hash, new_hash, 1);
    std::fs::write(file, &fixed)
        .map_err(|e| format!("writing {}: {e}", file.display()))?;

    Ok(())
}

fn write_log(
    log_dir: &std::path::Path,
    drv_path: &nix_compat::store_path::StorePath<String>,
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
///
/// Precedence: $CRUNCH_STATE_DIR > $XDG_STATE_HOME/crunch > ~/.local/state/crunch
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

/// Resolve log directory path.
fn log_dir() -> PathBuf {
    std::env::var("CRUNCH_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| state_dir().join("logs"))
}

/// Build the import path list: stdlib dir + user-specified paths.
fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let stdlib_dir = crunch_eval::stdlib::stdlib_import_path()
        .map_err(|e| RunError::Internal(format!("stdlib: {e}")))?;

    let mut paths: Vec<OsString> = vec![stdlib_dir.into()];
    for p in extra {
        paths.push(p.into());
    }
    Ok(paths)
}
