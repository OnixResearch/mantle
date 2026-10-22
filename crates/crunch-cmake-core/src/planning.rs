//! Pure, deterministic configure/build/install decisions.
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Admission;
use crate::Plan;
use crate::RejectCode;
use crate::Rejection;
use crate::StorePrefix;
use crate::admit_validated;
use crate::cmake_literal_ref;
use crate::reject;
use crate::store_root_ref;
use crate::validate_ref;

pub const WORK_DIRECTORY: &str = "/build/mantle-cmake";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lowered {
    pub source: String,
    pub toolchain_cmake: String,
    pub configure_args: Vec<String>,
    pub build_args: Vec<String>,
    pub install_args: Vec<String>,
    pub expected_elf_machine: u16,
}
#[derive(Debug)]
pub struct LowerRequest {
    pub plan: Plan,
    pub output: String,
}
#[derive(Debug)]
pub struct Lowering {
    pub admission: Admission,
    pub lowered: Lowered,
}
pub fn lower(request: LowerRequest) -> Result<Lowering, Rejection> {
    let LowerRequest { plan, output } = request;
    validate_ref(&plan)?;
    if !store_root_ref(&output, StorePrefix(&plan.store_prefix)) {
        return Err(reject(RejectCode("cmake-output-path-invalid"), &output));
    }
    let source = if plan.source_subdir.is_empty() {
        plan.roles[&plan.source].clone()
    } else {
        format!("{}/{}", plan.roles[&plan.source], plan.source_subdir)
    };
    let mut configure_args = vec![
        "-G".into(),
        "Ninja".into(),
        "-S".into(),
        source.clone(),
        "-B".into(),
        "build".into(),
        "--no-warn-unused-cli".into(),
        format!("-DCMAKE_TOOLCHAIN_FILE={WORK_DIRECTORY}/toolchain.cmake"),
        format!("-DCMAKE_BUILD_TYPE={}", plan.build_type),
        format!("-DCMAKE_INSTALL_PREFIX={output}"),
        "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON".into(),
        "-C".into(),
        format!("{WORK_DIRECTORY}/guard.cmake"),
        format!("-DCMAKE_MODULE_PATH={WORK_DIRECTORY}/modules"),
        "-DFETCHCONTENT_FULLY_DISCONNECTED=ON".into(),
        "-DFETCHCONTENT_UPDATES_DISCONNECTED=ON".into(),
        "-DCMAKE_POLICY_DEFAULT_CMP0170=NEW".into(),
    ];
    for (key, value) in &plan.definitions {
        let value = if let Some(rest) = value.strip_prefix("{{role:") {
            let (role, suffix) =
                rest.split_once("}}").ok_or_else(|| reject(RejectCode("cmake-definition-invalid"), key))?;
            format!("{}{suffix}", plan.roles[role])
        } else {
            value.clone()
        };
        configure_args.push(format!("-D{key}={value}"));
    }
    let mut build_args = vec![
        "--build".into(),
        "build".into(),
        "--parallel".into(),
        plan.jobs.to_string(),
    ];
    if !plan.build_targets.is_empty() {
        build_args.push("--target".into());
        build_args.extend(plan.build_targets.clone());
    }
    let lowered = Lowered {
        source,
        toolchain_cmake: toolchain(&plan)?,
        configure_args,
        build_args,
        install_args: vec!["--install".into(), "build".into()],
        expected_elf_machine: plan.elf_machine(),
    };
    let admission = admit_validated(plan)?;
    debug_assert!(!lowered.configure_args.is_empty());
    debug_assert!(lowered.toolchain_cmake.contains("CMAKE_SYSTEM_NAME"));
    debug_assert!(admission.tool_paths.iter().all(|path| !path.is_empty()));
    Ok(Lowering { admission, lowered })
}
/// Write one owned CMake cache setting; the entry is a key/value pair so the
/// two texts cannot be swapped by accident.
fn set(text: &mut String, entry: (&str, &str)) {
    let (key, value) = entry;
    text.push_str(&format!("set({key} {} CACHE STRING \"Mantle owned\" FORCE)\n", cmake_literal_ref(value)));
}
fn toolchain(plan: &Plan) -> Result<String, Rejection> {
    let mut text = String::new();
    if let Some(member) = &plan.toolchain_file {
        text.push_str(&format!("include({})\n", cmake_literal_ref(&plan.member(member)?)));
    }
    // Always cross mode, including native architecture: try_run needs emulator.
    set(&mut text, ("CMAKE_SYSTEM_NAME", &plan.target.system));
    set(&mut text, ("CMAKE_SYSTEM_PROCESSOR", &plan.target.processor));
    set(&mut text, ("CMAKE_SYSROOT", &plan.roles[&plan.sysroot]));
    set(&mut text, ("CMAKE_C_COMPILER", &format!("{WORK_DIRECTORY}/wrappers/mantle-cc")));
    set(&mut text, ("CMAKE_CXX_COMPILER", &format!("{WORK_DIRECTORY}/wrappers/mantle-cxx")));
    toolchain_compilers(&mut text, plan)?;
    toolchain_search(&mut text, plan)?;
    toolchain_policy(&mut text, plan)?;
    debug_assert!(!text.is_empty());
    debug_assert!(text.contains("CMAKE_SYSTEM_NAME"));
    Ok(text)
}
/// Compiler, archiver and linker selections; each resolved tool is written verbatim.
fn toolchain_compilers(text: &mut String, plan: &Plan) -> Result<(), Rejection> {
    let archiver = plan.tool(&plan.tools.ar)?;
    let ranlib = plan.tool(&plan.tools.ranlib)?;
    let linker = plan.tool(&plan.tools.linker)?;
    let ninja = plan.tool(&plan.tools.ninja)?;
    set(text, ("CMAKE_AR", &archiver));
    set(text, ("CMAKE_RANLIB", &ranlib));
    for language in ["C", "CXX"] {
        set(text, (&format!("CMAKE_{language}_COMPILER_AR"), &archiver));
        set(text, (&format!("CMAKE_{language}_COMPILER_RANLIB"), &ranlib));
    }
    set(text, ("CMAKE_TRY_COMPILE_TARGET_TYPE", "STATIC_LIBRARY"));
    set(text, ("CMAKE_LINKER", &linker));
    set(text, ("CMAKE_MAKE_PROGRAM", &ninja));
    debug_assert!(text.contains(&archiver));
    debug_assert!(text.contains(&ranlib));
    debug_assert!(text.contains(&linker));
    debug_assert!(text.contains(&ninja));
    Ok(())
}
/// Search roots, find modes and the admitted host tool directories.
fn toolchain_search(text: &mut String, plan: &Plan) -> Result<(), Rejection> {
    let mut roots = vec![plan.roles[&plan.sysroot].clone()];
    roots.extend(plan.dependencies.iter().map(|r| plan.roles[r].clone()));
    set(text, ("CMAKE_FIND_ROOT_PATH", &roots.join(";")));
    set(
        text,
        (
            "CMAKE_PREFIX_PATH",
            &plan.dependencies.iter().map(|r| plan.roles[r].clone()).collect::<Vec<_>>().join(";"),
        ),
    );
    for kind in ["PACKAGE", "LIBRARY", "INCLUDE"] {
        set(text, (&format!("CMAKE_FIND_ROOT_PATH_MODE_{kind}"), "ONLY"));
    }
    set(text, ("CMAKE_FIND_ROOT_PATH_MODE_PROGRAM", "NEVER"));
    let dirs = plan
        .host_tools
        .iter()
        .map(|tool| {
            let path = plan.tool(tool)?;
            Ok(path.rsplit_once('/').ok_or_else(|| reject(RejectCode("cmake-path-escape"), &path))?.0.to_string())
        })
        .collect::<Result<Vec<_>, Rejection>>()?;
    set(text, ("CMAKE_PROGRAM_PATH", &dirs.join(";")));
    debug_assert!(text.contains("CMAKE_FIND_ROOT_PATH"));
    debug_assert!(dirs.iter().all(|dir| text.contains(dir.as_str())));
    Ok(())
}
/// Registry, RPATH, module path and disconnected content policies.
fn toolchain_policy(text: &mut String, plan: &Plan) -> Result<(), Rejection> {
    for key in [
        "CMAKE_FIND_USE_PACKAGE_REGISTRY",
        "CMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY",
        "CMAKE_EXPORT_PACKAGE_REGISTRY",
        "CMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH",
        "CMAKE_FIND_USE_CMAKE_ENVIRONMENT_PATH",
        "CMAKE_FIND_USE_CMAKE_SYSTEM_PATH",
        "CMAKE_INSTALL_RPATH_USE_LINK_PATH",
    ] {
        set(text, (key, "OFF"));
    }
    set(text, ("CMAKE_INSTALL_LIBDIR", "lib"));
    set(text, ("CMAKE_BUILD_RPATH_USE_ORIGIN", "ON"));
    set(text, ("CMAKE_INSTALL_RPATH", "$ORIGIN/../lib"));
    set(text, ("FETCHCONTENT_FULLY_DISCONNECTED", "ON"));
    set(text, ("FETCHCONTENT_UPDATES_DISCONNECTED", "ON"));
    set(text, ("CMAKE_POLICY_DEFAULT_CMP0170", "NEW"));
    set(text, ("CMAKE_MODULE_PATH", &format!("{WORK_DIRECTORY}/modules")));
    set(text, ("CMAKE_PROJECT_INCLUDE_BEFORE", &format!("{WORK_DIRECTORY}/guard.cmake")));
    if let Some(emulator) = &plan.emulator {
        let emulator_path = plan.tool(emulator)?;
        set(text, ("CMAKE_CROSSCOMPILING_EMULATOR", &emulator_path));
        debug_assert!(text.contains(&emulator_path));
    }
    for (name, role) in &plan.fetch_sources {
        set(text, (&format!("FETCHCONTENT_SOURCE_DIR_{}", name.to_ascii_uppercase()), &plan.roles[role]));
    }
    debug_assert!(text.contains("CMAKE_MODULE_PATH"));
    Ok(())
}
