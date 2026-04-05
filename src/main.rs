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
        Command::Build { file, import_paths } => {
            let import_paths = build_import_paths(&import_paths)?;
            cmd_build(&file, &import_paths, &args.store, args.verbose)
        }
        Command::Bootstrap { output, packages } => cmd_bootstrap(&output, &packages),
        Command::Log { query, list } => cmd_log(query.as_deref(), list),
    }
}

fn cmd_build(
    file: &std::path::Path,
    import_paths: &[OsString],
    store_dir: &std::path::Path,
    verbose: bool,
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
    let mut known_paths = crunch_glue::KnownPaths::new();
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

            let mut builder = crunch_build::Builder::new(
                blob_service,
                directory_service,
                build_service,
                store_dir.to_path_buf(),
                verbose,
            );

            for (label, drv_path) in &drv_paths {
                let outcome = builder
                    .build(drv_path, &known_paths)
                    .await
                    .map_err(|e| {
                        let log_msg = format!("{e}");
                        write_log(&log_dir, drv_path, label, false, &log_msg);
                        RunError::Build(log_msg)
                    })?;

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
                    let path = path_info.store_path.to_absolute_path();
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

/// Write a build log file with metadata header.
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

/// Resolve log directory path.
fn log_dir() -> PathBuf {
    std::env::var("CRUNCH_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let state = std::env::var("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    PathBuf::from(home).join(".local/state")
                });
            state.join("crunch/logs")
        })
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
