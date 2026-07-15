use std::path::Path;

use crunch_kernelscript_adapter::read_request;
use crunch_kernelscript_adapter::run_request;
use crunch_kernelscript_adapter::write_report;

const CLI_ARGUMENT_COUNT_WITH_PROGRAM: usize = 5;

fn main() {
    if let Err(error) = run() {
        eprintln!("mantle-kernelscript-core-adapter: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os().collect::<Vec<_>>();
    if arguments.len() != CLI_ARGUMENT_COUNT_WITH_PROGRAM {
        return Err(
            "usage: mantle-kernelscript-core-adapter <request.json> <source.ks> <generated-root> <report.json>".into(),
        );
    }
    let request_path = Path::new(&arguments[1]);
    let source_path = Path::new(&arguments[2]);
    let generated_root = Path::new(&arguments[3]);
    let observation_path = Path::new(&arguments[4]);
    let mut request = read_request(request_path)?;
    request.source.path = source_path.display().to_string();
    let observation = run_request(request, generated_root)?;
    write_report(observation_path, &observation)?;
    debug_assert!(observation_path.is_file());
    debug_assert_eq!(observation.schema, crunch_kernelscript_adapter::ADAPTER_REPORT_SCHEMA);
    Ok(())
}
