use std::path::PathBuf;

use crunch_rustc_wrapper::DaemonOptions;

const EXIT_USAGE: i32 = 64;
const REQUIRED_ARGUMENT_COUNT: usize = 8;
const OPTION_PAIR_BYTES: usize = 2;

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let options = match parse_options(&arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("mantle-rust-cache-daemon:{error}");
            std::process::exit(EXIT_USAGE);
        }
    };
    if let Err(error) = crunch_rustc_wrapper::run_daemon(options) {
        eprintln!("mantle-rust-cache-daemon:{error}");
        std::process::exit(1);
    }
}

fn parse_options(arguments: &[String]) -> Result<DaemonOptions, String> {
    let run_once = arguments.iter().any(|argument| argument == "--once");
    let positional = arguments.iter().filter(|argument| argument.as_str() != "--once").cloned().collect::<Vec<_>>();
    if positional.len() != REQUIRED_ARGUMENT_COUNT {
        return Err(usage());
    }
    let policy_path = option_path(&positional, "--policy")?;
    let state_dir = option_path(&positional, "--state-dir")?;
    let store_output_dir = option_path(&positional, "--store-output-dir")?;
    let receipt_dir = option_path(&positional, "--receipt-dir")?;
    Ok(DaemonOptions {
        policy_path,
        state_dir,
        store_output_dir,
        receipt_dir,
        run_once,
    })
}

fn option_path(arguments: &[String], name: &str) -> Result<PathBuf, String> {
    let (pairs, remainder) = arguments.as_chunks::<OPTION_PAIR_BYTES>();
    debug_assert!(remainder.is_empty());
    for [option, value] in pairs {
        if option == name {
            return Ok(PathBuf::from(value));
        }
    }
    Err(format!("missing-option:{name}:{}", usage()))
}

fn usage() -> String {
    "usage: mantle-rust-cache-daemon --policy PATH --state-dir PATH --store-output-dir PATH --receipt-dir PATH [--once]"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_parse_with_once() {
        let arguments = [
            "--policy",
            "/policy.json",
            "--state-dir",
            "/state",
            "--store-output-dir",
            "/store",
            "--receipt-dir",
            "/receipts",
            "--once",
        ]
        .map(str::to_string);
        let options = parse_options(&arguments).unwrap();
        assert!(options.run_once);
        assert_eq!(options.policy_path, PathBuf::from("/policy.json"));
    }

    #[test]
    fn options_reject_missing_value() {
        let arguments = ["--policy", "/policy.json"].map(str::to_string);
        let error = parse_options(&arguments).unwrap_err();
        assert!(error.starts_with("usage:"));
        assert!(error.contains("--receipt-dir"));
    }
}
