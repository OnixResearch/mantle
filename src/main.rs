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
    let log_dir = std::env::var("CRUNCH_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let state = std::env::var("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    PathBuf::from(home).join(".local/state")
                });
            state.join("crunch/logs")
        });
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
                        // Write build log on failure
                        let log_msg = format!("{e}");
                        let log_file = log_dir.join(format!("{}.log", drv_path));
                        let _ = std::fs::write(&log_file, &log_msg);
                        RunError::Build(log_msg)
                    })?;

                for (_output_name, path_info) in &outcome.outputs {
                    let path = path_info.store_path.to_absolute_path();
                    if outcome.cached {
                        println!("{path} (cached)");
                    } else {
                        println!("{path}");
                    }
                }

                // Write success log
                if verbose {
                    let log_file = log_dir.join(format!("{}.log", drv_path));
                    let msg = format!("build succeeded: {label}\n");
                    let _ = std::fs::write(&log_file, &msg);
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
