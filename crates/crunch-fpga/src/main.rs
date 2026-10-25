use std::path::Path;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        return Err("usage: mantle-fpga (plan PROFILE | run PROFILE FRESH_ABSOLUTE_OUTPUT | hash FILE)".into());
    }
    if args[1] == "hash" && args.len() == 3 {
        println!("{}", crunch_fpga::hash_file(Path::new(&args[2]), crunch_fpga_core::MAX_FILE_BYTES)?.0);
        return Ok(());
    }
    let profile = crunch_fpga::read_profile(Path::new(&args[2]))?;
    let value = match args[1].as_str() {
        "plan" if args.len() == 3 => {
            serde_json::to_value(crunch_fpga_core::identities(profile).map_err(|error| error.to_string())?.stages)
        }
        "run" if args.len() == 4 => serde_json::to_value(crunch_fpga::execute(profile, Path::new(&args[3]))?),
        _ => return Err("expected plan PROFILE or run PROFILE FRESH_ABSOLUTE_OUTPUT".into()),
    }
    .map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?);
    Ok(())
}
