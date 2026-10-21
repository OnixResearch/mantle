use std::path::Path;

use crunch_zephyr::Result;
use crunch_zephyr::build;
use crunch_zephyr::identity;
use crunch_zephyr::lower;
use crunch_zephyr::tree_blake3;
use crunch_zephyr_core::MAX_PLAN_BYTES;
use crunch_zephyr_core::Plan;
use crunch_zephyr_core::Tool;

fn plan_file(path: &str) -> Result<Plan> {
    if std::fs::metadata(path)?.len() > MAX_PLAN_BYTES as u64 {
        return Err("zephyr-plan-byte-limit".into());
    }
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}
fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn execute() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["seal-tree", path] => println!("{}", tree_blake3(Path::new(path))?),
        ["identity", path] => println!("{}", identity(plan_file(path)?)?),
        ["lower", path, role, member] => {
            let runner = Tool { role: (*role).into(), member: (*member).into(), version: env!("CARGO_PKG_VERSION").into() };
            println!("{}", serde_json::to_string_pretty(&lower(plan_file(path)?, runner)?)?);
        }
        ["run"] => {
            let json = std::env::var("MANTLE_ZEPHYR_PLAN")?;
            if json.len() > MAX_PLAN_BYTES { return Err("zephyr-plan-byte-limit".into()); }
            let plan: Plan = serde_json::from_str(&json)?;
            let out = std::env::var("out")?;
            build(plan, Path::new(&out))?;
        }
        _ => return Err("usage: mantle-zephyr seal-tree ROOT | identity PLAN.json | lower PLAN.json RUNNER_ROLE RUNNER_MEMBER | run (ordinary Mantle sandbox only)".into()),
    }
    Ok(())
}
