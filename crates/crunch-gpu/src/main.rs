use std::path::Path;

use crunch_gpu::Result;
use crunch_gpu_core::MAX_PLAN_BYTES;
use crunch_gpu_core::Plan;

fn read_plan(path: &Path) -> Result<Plan> {
    if std::fs::metadata(path)?.len() > MAX_PLAN_BYTES as u64 {
        return Err("gpu-plan-size-limit".into());
    }
    let plan = serde_json::from_slice(&std::fs::read(path)?)?;
    Ok(plan)
}
fn main() {
    if let Err(error) = entry() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn entry() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--version" => {
            println!("mantle-gpu-v1");
            Ok(())
        }
        [action, path] if action == "lower" => {
            let derivation = crunch_gpu::lower(read_plan(Path::new(path))?)?;
            println!("{}", serde_json::to_string_pretty(&derivation)?);
            Ok(())
        }
        [action, port] if action == "probe-network" => {
            let port: u16 = port.parse()?;
            if port == 0 {
                return Err("gpu-probe-port-invalid".into());
            }
            let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            match std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_secs(2)) {
                Ok(_) => Err("gpu-network-escape: host loopback listener was reachable".into()),
                Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
                    Err(format!("gpu-network-denied: {error}").into())
                }
                Err(error) => Err(format!("gpu-network-probe-inconclusive: {error}").into()),
            }
        }
        [action] if action == "run" => {
            let raw = std::env::var("MANTLE_GPU_PLAN")?;
            if raw.len() > MAX_PLAN_BYTES {
                return Err("gpu-plan-size-limit".into());
            }
            let plan = serde_json::from_str(&raw)?;
            let output = std::env::var("out")?;
            crunch_gpu::build(plan, Path::new(&output))
        }
        _ => Err("usage: mantle-gpu --version | lower PLAN.json | run (admitted sandbox only)".into()),
    }
}
