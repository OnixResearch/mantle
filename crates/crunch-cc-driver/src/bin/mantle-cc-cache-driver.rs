use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use crunch_cc_driver::run_os;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let mut compiler = None;
    let mut socket = None;
    let mut receipt = None;
    let mut digest = None;
    let mut probe_script = None;
    while let Some(arg) = args.next() {
        if arg == "--" {
            let Some(compiler) = compiler else { break };
            let Some(socket) = socket else { break };
            let Some(receipt) = receipt else { break };
            let Some(platform_digest) = digest else { break };
            let arguments = args.collect::<Vec<_>>();
            let status = run_os(compiler, socket, receipt, platform_digest, probe_script, arguments);
            return ExitCode::from(u8::try_from(status).unwrap_or(1));
        }
        let value = match args.next() {
            Some(value) => value,
            None => break,
        };
        match arg.to_str() {
            Some("--compiler") if compiler.is_none() => compiler = Some(PathBuf::from(value)),
            Some("--socket") if socket.is_none() => socket = Some(PathBuf::from(value)),
            Some("--receipt") if receipt.is_none() => receipt = Some(PathBuf::from(value)),
            Some("--platform-digest") if digest.is_none() => digest = value.into_string().ok(),
            Some("--probe-script") if probe_script.is_none() => probe_script = Some(PathBuf::from(value)),
            _ => break,
        }
    }
    eprintln!(
        "usage: mantle-cc-cache-driver --compiler /ABS --socket /ABS --receipt /ABS --platform-digest <64hex> [--probe-script /ABS] -- <cc flags>"
    );
    ExitCode::from(2)
}
