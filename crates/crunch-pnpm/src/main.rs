//! Builder entry point; CLI is intentionally not a source acquisition authority.
use std::env;
use std::path::Path;
fn main() {
    if let Err(error) = run() {
        eprintln!("crunch-pnpm: {error}");
        std::process::exit(1);
    }
}
fn run() -> crunch_pnpm::files::Result<()> {
    let args: Vec<_> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("digest") if args.len() == 3 => {
            println!("{}", crunch_pnpm::files::tree(Path::new(&args[2]))?.as_str());
        }
        Some("blob-digest") if args.len() == 3 => {
            println!("{}", crunch_pnpm::files::blob(Path::new(&args[2]))?.as_str());
        }
        Some("identity") if args.len() == 3 => {
            let request = serde_json::from_slice(&crunch_pnpm::files::document(Path::new(&args[2]))?)?;
            println!("{}", crunch_pnpm_core::validate(request)?.identity.as_str());
        }
        Some("build") if args.len() == 7 => {
            let plan = crunch_pnpm::files::document(Path::new(&args[2]))?;
            if crunch_source_core::Blake3Digest::from_slice(&plan).as_str() != args[3] {
                return Err("plan role byte identity mismatch".into());
            }
            let request = serde_json::from_slice(&plan)?;
            let output = env::var("out").map_err(|_| "build is a Mantle derivation builder: missing $out")?;
            crunch_pnpm::execute(request, Path::new(&args[4]), Path::new(&args[5]), Path::new(&args[6]), Path::new(&output))?;
        }
        _ => return Err("usage: crunch-pnpm digest PATH | blob-digest FILE | identity PLAN_FILE | build PLAN_FILE PLAN_BLAKE3 SOURCE MIRROR TOOLS (inside Mantle derivation only)".into()),
    }
    Ok(())
}
