use std::env;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().collect()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("mantle-spacewasm-reference: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<String>) -> Result<String, crunch_spacewasm::ShellError> {
    let command = arguments.get(1).map(String::as_str).unwrap_or("");
    match command {
        "materialize" if arguments.len() == 4 => {
            let request = crunch_spacewasm::read_materialization_request(Path::new(&arguments[2]))?;
            let summary = crunch_spacewasm::materialize(request, Path::new(&arguments[3]))?;
            serde_json::to_string_pretty(&summary).map_err(|error| crunch_spacewasm::ShellError::Json(error.to_string()))
        }
        "verify" if arguments.len() == 3 => {
            let summary = crunch_spacewasm::verify_bundle(Path::new(&arguments[2]))?;
            let text = serde_json::to_string_pretty(&summary)
                .map_err(|error| crunch_spacewasm::ShellError::Json(error.to_string()))?;
            if summary.valid {
                Ok(text)
            } else {
                Err(crunch_spacewasm::ShellError::Invalid(text))
            }
        }
        "profile-check" if arguments.len() == 3 => {
            let summary = crunch_spacewasm::check_profile(Path::new(&arguments[2]))?;
            let text = serde_json::to_string_pretty(&summary)
                .map_err(|error| crunch_spacewasm::ShellError::Json(error.to_string()))?;
            if summary.valid {
                Ok(text)
            } else {
                Err(crunch_spacewasm::ShellError::Invalid(text))
            }
        }
        _ => Err(crunch_spacewasm::ShellError::Invalid(String::from(
            "usage: mantle-spacewasm-reference materialize <request.json> <bundle-dir> | verify <bundle-dir> | profile-check <profile.json>",
        ))),
    }
}
