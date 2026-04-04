use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "crunch", about = "Nickel build system on the Nix store protocol")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true)]
    log_level: Option<String>,

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

    match run(args) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            match &e {
                RunError::Eval(msg) => {
                    eprintln!("error: evaluation failed\n{msg}");
                    ExitCode::from(2)
                }
                RunError::Build(msg) => {
                    eprintln!("error: build failed\n{msg}");
                    ExitCode::from(1)
                }
                RunError::Internal(msg) => {
                    eprintln!("error: {msg}");
                    ExitCode::from(3)
                }
            }
        }
    }
}

#[derive(Debug)]
enum RunError {
    Eval(String),
    Build(String),
    Internal(String),
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
    }
}

fn cmd_build(
    file: &std::path::Path,
    import_paths: &[OsString],
    store_dir: &std::path::Path,
    verbose: bool,
) -> Result<(), RunError> {
    // 1. Evaluate Nickel file
    info!(file = %file.display(), "evaluating");
    let drv: crunch_glue::CrunchDerivation =
        crunch_eval::evaluate_and_deserialize(file, import_paths)
            .map_err(|e| RunError::Eval(format!("{e}")))?;

    // 2. Convert to nix_compat::Derivation
    let mut known_paths = crunch_glue::KnownPaths::new();
    let (drv_path, _nix_drv) = crunch_glue::convert(&drv, &mut known_paths)
        .map_err(|e| RunError::Build(format!("{e}")))?;

    info!(drv = %drv_path, "derivation constructed");

    // 3. Set up build services and build
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

            let outcome = builder
                .build(&drv_path, &known_paths)
                .await
                .map_err(|e| RunError::Build(format!("{e}")))?;

            for (output_name, path_info) in &outcome.outputs {
                let path = path_info.store_path.to_absolute_path();
                if outcome.cached {
                    println!("{path} (cached)");
                } else {
                    println!("{path}");
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
