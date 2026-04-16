mod benchmark_support;

use std::path::PathBuf;
use std::process::ExitCode;

use benchmark_support::BenchmarkBundle;
use benchmark_support::DEFAULT_REPEAT_COUNT;
use benchmark_support::EVAL_PHASE_METRIC_NAME;
use benchmark_support::TOTAL_PHASE_METRIC_NAME;
use benchmark_support::default_eval_smoke_bundle_path;
use benchmark_support::eval_smoke_request;
use benchmark_support::run_eval_smoke_benchmark;

fn main() -> ExitCode {
    match run() {
        Ok(bundle) => {
            print_metrics(&bundle);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<BenchmarkBundle, benchmark_support::Error> {
    let args: Vec<String> = std::env::args().collect();
    let (bundle_out_path, repeat_count) = parse_args(&args)?;
    let request = eval_smoke_request(bundle_out_path, repeat_count, args);
    run_eval_smoke_benchmark(&request)
}

fn parse_args(args: &[String]) -> Result<(PathBuf, u32), benchmark_support::Error> {
    assert!(!args.is_empty(), "argv must contain program name");

    let mut bundle_out_path = default_eval_smoke_bundle_path();
    let mut repeat_count = DEFAULT_REPEAT_COUNT;
    let mut index: usize = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--bundle-out" => {
                let value = next_arg(args, index, "--bundle-out")?;
                bundle_out_path = PathBuf::from(value);
                index += 2;
            }
            "--repeat-count" => {
                let value = next_arg(args, index, "--repeat-count")?;
                repeat_count = value.parse::<u32>().map_err(|err| {
                    benchmark_support::Error::InvalidArgument(format!("invalid --repeat-count value `{value}`: {err}"))
                })?;
                index += 2;
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

    Ok((bundle_out_path, repeat_count))
}

fn next_arg<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, benchmark_support::Error> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| benchmark_support::Error::InvalidArgument(format!("missing value for {flag}")))?;
    Ok(value.as_str())
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} [--bundle-out PATH] [--repeat-count N]");
}

fn print_metrics(bundle: &BenchmarkBundle) {
    assert_eq!(bundle.results.len(), 1, "eval smoke benchmark must emit one result");
    let result = &bundle.results[0];
    println!("bundle={}", bundle.bundle_path);
    println!(
        "workload={} repeat_count={} root_count={}",
        result.workload_name, result.repeat_count, result.root_count
    );

    for metric in &result.phase_metrics {
        println!("METRIC {}={}", metric.name, metric.value);
    }
    println!("METRIC {}={}", TOTAL_PHASE_METRIC_NAME, result.total_wall_ns);

    assert!(
        result.phase_metrics.iter().any(|metric| metric.name == EVAL_PHASE_METRIC_NAME),
        "eval smoke benchmark must emit evaluation metric"
    );
    assert!(!bundle.bundle_path.is_empty(), "eval smoke benchmark must report bundle path");
}
