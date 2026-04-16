mod benchmark_support;

use std::path::PathBuf;
use std::process::ExitCode;

use benchmark_support::COMPARE_ENTRY_POINT;
use benchmark_support::CompareThresholds;
use benchmark_support::compare_benchmark_files;
use benchmark_support::default_compare_thresholds;
use benchmark_support::render_comparison_human;
use benchmark_support::render_comparison_json;

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<String, benchmark_support::Error> {
    let args: Vec<String> = std::env::args().collect();
    let parsed = parse_args(&args)?;
    let report = compare_benchmark_files(&parsed.baseline_path, &parsed.fresh_path, &parsed.thresholds)?;
    if parsed.json {
        return render_comparison_json(&report);
    }
    Ok(render_comparison_human(&report))
}

struct CompareArgs {
    baseline_path: PathBuf,
    fresh_path: PathBuf,
    thresholds: CompareThresholds,
    json: bool,
}

fn parse_args(args: &[String]) -> Result<CompareArgs, benchmark_support::Error> {
    if args.is_empty() {
        return Err(benchmark_support::Error::InvalidArgument("argv must contain program name".to_string()));
    }

    let mut baseline_path: Option<PathBuf> = None;
    let mut fresh_path: Option<PathBuf> = None;
    let mut thresholds = default_compare_thresholds();
    let mut json = false;
    let mut index: usize = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--baseline" => {
                baseline_path = Some(PathBuf::from(next_arg(args, index, "--baseline")?));
                index += 2;
            }
            "--fresh" => {
                fresh_path = Some(PathBuf::from(next_arg(args, index, "--fresh")?));
                index += 2;
            }
            "--absolute-threshold-ns" => {
                let value = next_arg(args, index, "--absolute-threshold-ns")?;
                thresholds.absolute_threshold_ns = value.parse::<u64>().map_err(|err| {
                    benchmark_support::Error::InvalidArgument(format!(
                        "invalid --absolute-threshold-ns value `{value}`: {err}"
                    ))
                })?;
                index += 2;
            }
            "--percent-threshold" => {
                let value = next_arg(args, index, "--percent-threshold")?;
                thresholds.percent_threshold = value.parse::<f64>().map_err(|err| {
                    benchmark_support::Error::InvalidArgument(format!(
                        "invalid --percent-threshold value `{value}`: {err}"
                    ))
                })?;
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            other => {
                return Err(benchmark_support::Error::InvalidArgument(format!("unknown argument `{other}`")));
            }
        }
    }

    let baseline_path = baseline_path
        .ok_or_else(|| benchmark_support::Error::InvalidArgument("missing required --baseline PATH".to_string()))?;
    let fresh_path = fresh_path
        .ok_or_else(|| benchmark_support::Error::InvalidArgument("missing required --fresh PATH".to_string()))?;

    Ok(CompareArgs {
        baseline_path,
        fresh_path,
        thresholds,
        json,
    })
}

fn next_arg<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, benchmark_support::Error> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| benchmark_support::Error::InvalidArgument(format!("missing value for {flag}")))?;
    Ok(value.as_str())
}

fn print_usage(program: &str) {
    eprintln!(
        "Usage: {program} --baseline PATH --fresh PATH [--absolute-threshold-ns N] [--percent-threshold P] [--json]\nentry_point={COMPARE_ENTRY_POINT}"
    );
}
