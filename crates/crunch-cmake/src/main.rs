use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;

use crunch_cmake_core::MAX_PLAN_BYTES;
use crunch_cmake_core::Plan;
use crunch_cmake_core::Tool;
use crunch_cmake_core::validate;

fn plan_text(text: &str) -> crunch_cmake::Result<Plan> {
    if text.len() > MAX_PLAN_BYTES {
        return Err(std::io::Error::other("cmake-plan-limit").into());
    }
    let plan: Plan = serde_json::from_str(text)?;
    validate(plan).map_err(|e| std::io::Error::other(e.to_string()).into())
}
fn read_plan(path: &str) -> crunch_cmake::Result<Plan> {
    let mut text = String::new();
    std::fs::File::open(path)?.take(MAX_PLAN_BYTES.saturating_add(1) as u64).read_to_string(&mut text)?;
    plan_text(&text)
}
fn main() {
    if let Err(error) = entry() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn entry() -> crunch_cmake::Result<()> {
    let raw: Vec<_> = std::env::args_os().collect();
    debug_assert!(!raw.is_empty());
    let executable_name = Path::new(&raw[0]).file_name();
    if executable_name == Some(OsStr::new("mantle-cc")) || executable_name == Some(OsStr::new("mantle-cxx")) {
        let plan = plan_text(&std::env::var("MANTLE_CMAKE_PLAN")?)?;
        debug_assert!(plan.schema == "mantle-cmake-plan-v1");
        let code = crunch_cmake::compile(plan, executable_name == Some(OsStr::new("mantle-cxx")), &raw[1..])?;
        std::process::exit(code);
    }
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "run" => {
            let plan = plan_text(&std::env::var("MANTLE_CMAKE_PLAN")?)?;
            crunch_cmake::build(plan, Path::new(&std::env::var("out")?))?;
        }
        [command, path] if command == "validate" => {
            let plan = read_plan(path)?;
            println!("{}", blake3::hash(&serde_json::to_vec(&plan)?).to_hex());
        }
        [command, path, role, member, version] if command == "lower" => {
            let derivation = crunch_cmake::lower(read_plan(path)?, Tool { role: role.clone(), member: member.clone(), version: version.clone() })?;
            println!("{}", serde_json::to_string_pretty(&derivation)?);
        }
        [command, path, processor] if command == "inspect" => {
            let machine = match processor.as_str() { "x86_64" => 62, "aarch64" => 183, _ => return Err(std::io::Error::other("cmake-platform-unsupported").into()) };
            crunch_cmake::inspect_architecture(Path::new(path), machine)?;
        }
        [command, root, role, env, prefix] if command == "native-input" => {
            let request = crunch_cmake::NativeInputRequest {
                output_root: root.clone(),
                role: role.clone(),
                root_env: env.clone(),
                store_prefix: prefix.clone(),
            };
            println!("{}", serde_json::to_string_pretty(&crunch_cmake::native_input(request)?)?);
        }
        _ => return Err(std::io::Error::other("usage: mantle-cmake validate PLAN | lower PLAN RUNNER_ROLE RUNNER_MEMBER RUNNER_VERSION | run | inspect ARTIFACT {x86_64|aarch64} | native-input ROOT ROLE ENV STORE_PREFIX").into()),
    }
    Ok(())
}
