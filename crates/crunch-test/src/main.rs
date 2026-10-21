use std::path::Path;
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.as_slice().get(1).map(String::as_str) != Some("run") || args.len() != 2 {
        return Err("usage: mantle-test-worker run (declared derivation environment only)".into());
    }
    let target = std::env::var("MANTLE_TEST_TARGET").map_err(|_| "test-target-environment-missing")?;
    let invocation = std::env::var("MANTLE_TEST_INVOCATION").map_err(|_| "test-invocation-environment-missing")?;
    if target.len() > 262_144 || invocation.len() > 8192 {
        return Err("test-request-byte-limit".into());
    }
    let target = serde_json::from_str(&target).map_err(|e| e.to_string())?;
    let invocation = serde_json::from_str(&invocation).map_err(|e| e.to_string())?;
    let out = std::env::var("out").map_err(|_| "test-output-environment-missing")?;
    let request = crunch_test_core::select(target, invocation)?;
    crunch_test::run_worker(request, Path::new(&out))?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("test-infrastructure-failure: {error}");
        std::process::exit(2);
    }
}
