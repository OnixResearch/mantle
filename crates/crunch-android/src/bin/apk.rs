//! Render a validated APK plan into a Mantle-native build graph.
//!
//! A subsequent `mantle build <output>.ncl --offline-source-preflight --no-substitute`
//! can execute the graph only after every declared source and pinned archive
//! has been imported into the selected store's offline source bundle.
//! Rendering alone neither executes Android binaries nor grants provenance to
//! third-party prebuilts.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use crunch_android::ApkInputs;
use crunch_android::prepare_apk;
use crunch_android_core::ApkPlan;
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "crunch-android-apk",
    about = "Render a pinned Android APK plan as a Mantle derivation graph"
)]
struct Args {
    /// Nickel expression evaluating to android.mkApk's published typed plan record.
    #[arg(long)]
    plan: PathBuf,
    /// JSON bindings for declared app input, locally checked archive bytes, and explicit runtime
    /// libraries.
    #[arg(long)]
    inputs: PathBuf,
    /// The absolute store root used by Mantle for every declared input.
    #[arg(long)]
    store_prefix: String,
    /// New .ncl output; a .json sibling holds the inert derivation data.
    #[arg(long)]
    output: PathBuf,
}

#[derive(Serialize)]
struct PlanReceipt<'a> {
    derivation: &'a Path,
    final_output: &'a str,
    apk_stages: usize,
    prebuilt_toolchain_identities: [&'a crunch_android_core::ToolchainIdentity; 3],
    non_claim: &'static str,
}

fn output_json_name(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let name = path.file_name().and_then(|name| name.to_str()).ok_or("output needs a UTF-8 file name")?;
    if !name.ends_with(".ncl")
        || name.starts_with('.')
        || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err("output file name must be an ASCII [A-Za-z0-9._-]+.ncl name".into());
    }
    Ok(format!("{name}.json"))
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    if let Err(error) = file.write_all(bytes) {
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    Ok(())
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let json_name = output_json_name(&args.output)?;
    let plan: ApkPlan = crunch_eval::evaluate_and_deserialize(&args.plan, &[])?;
    let inputs: ApkInputs = serde_json::from_slice(&std::fs::read(&args.inputs)?)?;
    let prepared = prepare_apk(&plan, &inputs, &args.store_prefix)?;
    let json_path = args.output.with_file_name(&json_name);
    let json = serde_json::to_vec_pretty(&prepared.final_derivation)?;

    // Nickel imports JSON as data: `%{...}` in source members, scripts, or
    // derivation environment never becomes Nickel string interpolation.
    let expression = format!("import \"./{json_name}\"\n");
    write_new(&json_path, &json)?;
    if let Err(error) = write_new(&args.output, expression.as_bytes()) {
        let _ = std::fs::remove_file(&json_path);
        return Err(error.into());
    }

    let report = PlanReceipt {
        derivation: &args.output,
        final_output: &prepared.final_output_path,
        apk_stages: prepared.stages.len(),
        prebuilt_toolchain_identities: [
            &plan.toolchains.jdk,
            &plan.toolchains.build_tools,
            &plan.toolchains.platform,
        ],
        non_claim: "SHA-256 matches pinned third-party prebuilt archives; no source-built or StageX provenance is claimed",
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("crunch-android-apk: {error}");
            ExitCode::FAILURE
        }
    }
}
