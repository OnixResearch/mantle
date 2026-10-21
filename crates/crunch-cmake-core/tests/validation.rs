use std::collections::BTreeMap;

use crunch_cmake_core::Compiler;
use crunch_cmake_core::Member;
use crunch_cmake_core::Plan;
use crunch_cmake_core::Target;
use crunch_cmake_core::Tool;
use crunch_cmake_core::Tools;
use crunch_cmake_core::validate;
fn plan() -> Plan {
    let tool = Tool {
        role: "tool".into(),
        member: "bin/tool".into(),
        version: "1.0".into(),
    };
    Plan {
        schema: "mantle-cmake-plan-v1".into(),
        name: "fixture".into(),
        store_prefix: "/mantle/store".into(),
        source: "source".into(),
        source_subdir: String::new(),
        roles: BTreeMap::from([
            ("source".into(), "/mantle/store/00000000000000000000000000000000-source".into()),
            ("tool".into(), "/mantle/store/11111111111111111111111111111111-tool".into()),
            ("sysroot".into(), "/mantle/store/22222222222222222222222222222222-sysroot".into()),
        ]),
        tools: Tools {
            cmake: tool.clone(),
            ninja: tool.clone(),
            cc: tool.clone(),
            cxx: tool.clone(),
            ar: tool.clone(),
            ranlib: tool.clone(),
            linker: tool,
        },
        compiler: Compiler::Clang,
        build_platform: "x86_64-linux".into(),
        target: Target {
            system: "Linux".into(),
            processor: "aarch64".into(),
            triple: "aarch64-unknown-linux-gnu".into(),
        },
        sysroot: "sysroot".into(),
        runtime_roles: Vec::new(),
        dependencies: Vec::new(),
        host_tools: Vec::new(),
        c_includes: Vec::new(),
        cxx_includes: Vec::new(),
        toolchain_file: None,
        emulator: None,
        fetch_sources: BTreeMap::new(),
        definitions: BTreeMap::new(),
        build_targets: Vec::new(),
        outputs: vec!["lib/libanswer.a".into()],
        build_type: "Release".into(),
        jobs: 1,
        source_date_epoch: 1,
    }
}
#[test]
fn undeclared_roles_never_acquire_ambient_authority() {
    let mut p = plan();
    assert!(validate(p.clone()).is_ok());
    p.dependencies.push("ambient".into());
    assert_eq!(validate(p.clone()).unwrap_err().code, "cmake-role-undeclared");
    p.dependencies.clear();
    p.host_tools.push(Tool {
        role: "ambient".into(),
        member: "bin/protoc".into(),
        version: "1".into(),
    });
    assert_eq!(validate(p).unwrap_err().code, "cmake-role-undeclared");
}
#[test]
fn tool_and_source_members_cannot_escape_admitted_roots() {
    for member in ["../bin/cc", "/usr/bin/cc", "bin/../../cc", "bin//cc", "bin/cc\n"] {
        let mut p = plan();
        p.tools.cc.member = member.into();
        assert_eq!(validate(p).unwrap_err().code, "cmake-path-escape", "{member}");
    }
    let mut p = plan();
    p.roles.insert("tool".into(), "/usr".into());
    assert_eq!(validate(p).unwrap_err().code, "cmake-path-escape");
    let mut p = plan();
    p.cxx_includes.push(Member {
        role: "sysroot".into(),
        member: "../include".into(),
    });
    assert_eq!(validate(p).unwrap_err().code, "cmake-path-escape");
}
#[test]
fn user_definitions_cannot_override_isolation_or_inject_host_roots() {
    for (key, value) in [
        ("CMAKE_C_COMPILER", "cc"),
        ("FETCHCONTENT_FULLY_DISCONNECTED", "OFF"),
        ("ZLIB_ROOT", "/usr/lib"),
        ("ZLIB_ROOT", "{{role:sysroot}}/../host"),
        ("LIST", "a;b"),
    ] {
        let mut p = plan();
        p.definitions.insert(key.into(), value.into());
        assert!(validate(p).is_err(), "{key}={value}");
    }
    let mut p = plan();
    p.definitions.insert("ZLIB_ROOT".into(), "{{role:sysroot}}/usr".into());
    assert!(validate(p).is_ok());
}
#[test]
fn platform_mismatch_and_unbounded_outputs_fail_before_execution() {
    let mut p = plan();
    p.target.triple = "x86_64-unknown-linux-gnu".into();
    assert_eq!(validate(p).unwrap_err().code, "cmake-platform-unsupported");
    let mut p = plan();
    p.outputs = vec!["../escape".into()];
    assert_eq!(validate(p).unwrap_err().code, "cmake-output-invalid");
    let mut p = plan();
    p.jobs = 65;
    assert_eq!(validate(p).unwrap_err().code, "cmake-plan-invalid");
}
