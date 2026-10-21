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
use crate::Rejection;
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
    if !store_root_ref(&output, &plan.store_prefix) {
        return Err(reject("cmake-output-path-invalid", &output));
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
            let (role, suffix) = rest.split_once("}}").ok_or_else(|| reject("cmake-definition-invalid", key))?;
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
    Ok(Lowering {
        admission: admit_validated(plan)?,
        lowered,
    })
}
fn set(text: &mut String, key: &str, value: &str) {
    text.push_str(&format!("set({key} {} CACHE STRING \"Mantle owned\" FORCE)\n", cmake_literal_ref(value)));
}
fn toolchain(plan: &Plan) -> Result<String, Rejection> {
    let mut text = String::new();
    if let Some(member) = &plan.toolchain_file {
        text.push_str(&format!("include({})\n", cmake_literal_ref(&plan.member(member)?)));
    }
    // Always cross mode, including native architecture: try_run needs emulator.
    set(&mut text, "CMAKE_SYSTEM_NAME", &plan.target.system);
    set(&mut text, "CMAKE_SYSTEM_PROCESSOR", &plan.target.processor);
    set(&mut text, "CMAKE_SYSROOT", &plan.roles[&plan.sysroot]);
    set(&mut text, "CMAKE_C_COMPILER", &format!("{WORK_DIRECTORY}/wrappers/mantle-cc"));
    set(&mut text, "CMAKE_CXX_COMPILER", &format!("{WORK_DIRECTORY}/wrappers/mantle-cxx"));
    set(&mut text, "CMAKE_AR", &plan.tool(&plan.tools.ar)?);
    set(&mut text, "CMAKE_RANLIB", &plan.tool(&plan.tools.ranlib)?);
    for language in ["C", "CXX"] {
        set(&mut text, &format!("CMAKE_{language}_COMPILER_AR"), &plan.tool(&plan.tools.ar)?);
        set(&mut text, &format!("CMAKE_{language}_COMPILER_RANLIB"), &plan.tool(&plan.tools.ranlib)?);
    }
    set(&mut text, "CMAKE_TRY_COMPILE_TARGET_TYPE", "STATIC_LIBRARY");
    set(&mut text, "CMAKE_LINKER", &plan.tool(&plan.tools.linker)?);
    set(&mut text, "CMAKE_MAKE_PROGRAM", &plan.tool(&plan.tools.ninja)?);
    let mut roots = vec![plan.roles[&plan.sysroot].clone()];
    roots.extend(plan.dependencies.iter().map(|r| plan.roles[r].clone()));
    set(&mut text, "CMAKE_FIND_ROOT_PATH", &roots.join(";"));
    set(
        &mut text,
        "CMAKE_PREFIX_PATH",
        &plan.dependencies.iter().map(|r| plan.roles[r].clone()).collect::<Vec<_>>().join(";"),
    );
    for kind in ["PACKAGE", "LIBRARY", "INCLUDE"] {
        set(&mut text, &format!("CMAKE_FIND_ROOT_PATH_MODE_{kind}"), "ONLY");
    }
    set(&mut text, "CMAKE_FIND_ROOT_PATH_MODE_PROGRAM", "NEVER");
    let dirs = plan
        .host_tools
        .iter()
        .map(|tool| {
            let path = plan.tool(tool)?;
            Ok(path.rsplit_once('/').ok_or_else(|| reject("cmake-path-escape", &path))?.0.to_string())
        })
        .collect::<Result<Vec<_>, Rejection>>()?;
    set(&mut text, "CMAKE_PROGRAM_PATH", &dirs.join(";"));
    for key in [
        "CMAKE_FIND_USE_PACKAGE_REGISTRY",
        "CMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY",
        "CMAKE_EXPORT_PACKAGE_REGISTRY",
        "CMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH",
        "CMAKE_FIND_USE_CMAKE_ENVIRONMENT_PATH",
        "CMAKE_FIND_USE_CMAKE_SYSTEM_PATH",
        "CMAKE_INSTALL_RPATH_USE_LINK_PATH",
    ] {
        set(&mut text, key, "OFF");
    }
    set(&mut text, "CMAKE_INSTALL_LIBDIR", "lib");
    set(&mut text, "CMAKE_BUILD_RPATH_USE_ORIGIN", "ON");
    set(&mut text, "CMAKE_INSTALL_RPATH", "$ORIGIN/../lib");
    set(&mut text, "FETCHCONTENT_FULLY_DISCONNECTED", "ON");
    set(&mut text, "FETCHCONTENT_UPDATES_DISCONNECTED", "ON");
    set(&mut text, "CMAKE_POLICY_DEFAULT_CMP0170", "NEW");
    set(&mut text, "CMAKE_MODULE_PATH", &format!("{WORK_DIRECTORY}/modules"));
    set(&mut text, "CMAKE_PROJECT_INCLUDE_BEFORE", &format!("{WORK_DIRECTORY}/guard.cmake"));
    if let Some(emulator) = &plan.emulator {
        set(&mut text, "CMAKE_CROSSCOMPILING_EMULATOR", &plan.tool(emulator)?);
    }
    for (name, role) in &plan.fetch_sources {
        set(&mut text, &format!("FETCHCONTENT_SOURCE_DIR_{}", name.to_ascii_uppercase()), &plan.roles[role]);
    }
    Ok(text)
}
