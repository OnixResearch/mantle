#![no_std]
//! Offline HIP authoring policy, with no filesystem, process or device authority.
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub role: String,
    pub member: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub role: String,
    pub member: String,
    pub version: String,
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"_+.-".contains(&b))
        && value != "."
        && value != ".."
}
fn relative(value: &str) -> bool {
    value.len() <= 1024 && !value.is_empty() && value.split('/').all(identifier)
}
fn store_root(value: &str, prefix: &str) -> bool {
    if !prefix.starts_with('/') || !prefix[1..].split('/').all(identifier) {
        return false;
    }
    let Some(rest) = value.strip_prefix(prefix).and_then(|s| s.strip_prefix('/')) else {
        return false;
    };
    let Some((hash, name)) = rest.split_once('-') else {
        return false;
    };
    hash.len() == 32
        && hash.bytes().all(|b| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&b))
        && identifier(name)
        && !rest.contains('/')
}

pub const MAX_PLAN_BYTES: usize = 131_072;
pub const MAX_ENTRIES: usize = 64;
pub const DEVICE_LIBRARIES: &[&str] = &[
    "hip.bc",
    "ocml.bc",
    "ockl.bc",
    "oclc_daz_opt_off.bc",
    "oclc_unsafe_math_off.bc",
    "oclc_finite_only_off.bc",
    "oclc_correctly_rounded_sqrt_on.bc",
    "oclc_wavefrontsize64_on.bc",
    "oclc_isa_version_900.bc",
    "oclc_abi_version_400.bc",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: String,
    pub name: String,
    pub backend: String,
    pub build_platform: String,
    pub store_prefix: String,
    /// Exact signed store roots; labels and revision strings are not content authority.
    pub roles: BTreeMap<String, String>,
    pub kernel: Member,
    pub headers: Vec<Member>,
    /// Separately admitted generated headers, force-included in this order.
    pub generated: Vec<Member>,
    pub clang: Tool,
    pub host_compiler: Tool,
    pub inspector: Tool,
    pub runner: Tool,
    pub host_target: String,
    pub sysroot: String,
    pub resource_dir: Member,
    pub toolkit: String,
    pub toolkit_revision: String,
    pub device_libraries: Member,
    /// Host shared libraries needed by the admitted compiler/inspector closure.
    pub runtime_libraries: Vec<Member>,
    pub architecture: String,
    pub optimization: u8,
    pub defines: BTreeMap<String, String>,
    pub entry_points: Vec<String>,
    pub source_date_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejection(pub String);
impl core::fmt::Display for Rejection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}
fn reject(code: &str) -> Rejection {
    Rejection(code.to_string())
}
impl Plan {
    fn role(&self, role: &str) -> Result<&str, Rejection> {
        self.roles.get(role).map(String::as_str).ok_or_else(|| reject("gpu-role-undeclared"))
    }
    fn member(&self, member: &Member) -> Result<(), Rejection> {
        self.path(&member.role, &member.member)
    }
    fn tool(&self, tool: &Tool) -> Result<(), Rejection> {
        self.path(&tool.role, &tool.member)
    }
    fn path(&self, role: &str, member: &str) -> Result<(), Rejection> {
        if !relative(member) {
            return Err(reject("gpu-path-escape"));
        }
        self.role(role)?;
        Ok(())
    }
    fn resolved(&self, role: &str, member: &str) -> String {
        format!("{}/{member}", self.roles[role])
    }
}

/// Absolute lexical paths resolved by policy, not by the filesystem shell.
#[derive(Debug)]
pub struct Paths {
    pub kernel: String,
    pub headers: Vec<String>,
    pub generated: Vec<String>,
    pub clang: String,
    pub host_compiler: String,
    pub inspector: String,
    pub runner: String,
    pub sysroot: String,
    pub toolkit: String,
    pub resource_dir: String,
    pub device_libraries: String,
    pub runtime_libraries: Vec<String>,
}

#[derive(Debug)]
pub struct AdmittedPlan {
    pub plan: Plan,
    pub paths: Paths,
}
fn c_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(b) if b.is_ascii_alphabetic() || b == b'_')
        && value.len() <= 128
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
/// Only the bounded AMD LLVM 22.0.0 / ROCm 7.2.3 / gfx900 fragment is
/// admitted. Other combinations require independent real-tool evidence.
pub fn validate(plan: Plan) -> Result<AdmittedPlan, Rejection> {
    validate_plan(&plan)?;
    let member = |value: &Member| plan.resolved(&value.role, &value.member);
    let tool = |value: &Tool| plan.resolved(&value.role, &value.member);
    let paths = Paths {
        kernel: member(&plan.kernel),
        headers: plan.headers.iter().map(member).collect(),
        generated: plan.generated.iter().map(member).collect(),
        clang: tool(&plan.clang),
        host_compiler: tool(&plan.host_compiler),
        inspector: tool(&plan.inspector),
        runner: tool(&plan.runner),
        sysroot: plan.roles[&plan.sysroot].clone(),
        toolkit: plan.roles[&plan.toolkit].clone(),
        resource_dir: member(&plan.resource_dir),
        device_libraries: member(&plan.device_libraries),
        runtime_libraries: plan.runtime_libraries.iter().map(member).collect(),
    };
    Ok(AdmittedPlan { plan, paths })
}

fn validate_plan(plan: &Plan) -> Result<(), Rejection> {
    if plan.schema != "mantle-gpu-plan-v1" || !identifier(&plan.name) {
        return Err(reject("gpu-plan-invalid"));
    }
    if plan.backend != "hip-clang"
        || plan.architecture != "gfx900"
        || plan.toolkit_revision != "7.2.3"
        || plan.build_platform != "x86_64-linux"
        || plan.host_target != "x86_64-unknown-linux-gnu"
    {
        return Err(reject("gpu-combination-unsupported"));
    }
    if !plan.store_prefix.starts_with('/')
        || !plan.store_prefix[1..].split('/').all(identifier)
        || plan.roles.is_empty()
        || plan.roles.len() > MAX_ENTRIES
    {
        return Err(reject("gpu-input-limit"));
    }
    for (role, root) in &plan.roles {
        if !identifier(role) || !store_root(root, &plan.store_prefix) {
            return Err(reject("gpu-path-escape"));
        }
    }
    for tool in [&plan.clang, &plan.host_compiler, &plan.inspector] {
        if tool.version != "22.0.0" {
            return Err(reject("gpu-compiler-unsupported"));
        }
        plan.tool(tool)?;
    }
    if plan.runner.version != "mantle-gpu-v1" {
        return Err(reject("gpu-runner-unsupported"));
    }
    plan.tool(&plan.runner)?;
    plan.member(&plan.kernel)?;
    plan.member(&plan.resource_dir)?;
    plan.member(&plan.device_libraries)?;
    plan.role(&plan.sysroot)?;
    plan.role(&plan.toolkit)?;
    if plan.headers.is_empty()
        || plan.runtime_libraries.is_empty()
        || plan.entry_points.is_empty()
        || plan.optimization > 3
        || plan.source_date_epoch > i64::MAX as u64
    {
        return Err(reject("gpu-plan-invalid"));
    }
    for count in [
        plan.headers.len(),
        plan.generated.len(),
        plan.runtime_libraries.len(),
        plan.entry_points.len(),
        plan.defines.len(),
    ] {
        if count > MAX_ENTRIES {
            return Err(reject("gpu-input-limit"));
        }
    }
    for member in plan.headers.iter().chain(&plan.generated).chain(&plan.runtime_libraries) {
        plan.member(member)?;
    }
    for (key, value) in &plan.defines {
        if !c_identifier(key)
            || value.len() > 128
            || value.is_empty()
            || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b"_+.-".contains(&b))
        {
            return Err(reject("gpu-option-unsupported"));
        }
    }
    let names: BTreeSet<_> = plan.entry_points.iter().collect();
    if names.len() != plan.entry_points.len() || names.iter().any(|s| !c_identifier(s)) {
        return Err(reject("gpu-entry-point-invalid"));
    }
    debug_assert!(!plan.roles.is_empty());
    debug_assert!(plan.optimization <= 3);
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    fn member(role: &str, path: &str) -> Member {
        Member {
            role: role.into(),
            member: path.into(),
        }
    }
    fn tool(path: &str) -> Tool {
        Tool {
            role: "closure".into(),
            member: path.into(),
            version: "22.0.0".into(),
        }
    }
    fn plan() -> Plan {
        Plan {
            schema: "mantle-gpu-plan-v1".into(),
            name: "hip-kernel".into(),
            backend: "hip-clang".into(),
            build_platform: "x86_64-linux".into(),
            store_prefix: "/mantle/store".into(),
            roles: BTreeMap::from([(
                "closure".into(),
                "/mantle/store/00000000000000000000000000000000-closure".into(),
            )]),
            kernel: member("closure", "kernel.hip"),
            headers: vec![member("closure", "include")],
            generated: vec![],
            clang: tool("bin/clang++"),
            host_compiler: tool("bin/clang++"),
            inspector: tool("bin/llvm-readobj"),
            runner: Tool {
                version: "mantle-gpu-v1".into(),
                ..tool("bin/mantle-gpu")
            },
            host_target: "x86_64-unknown-linux-gnu".into(),
            sysroot: "closure".into(),
            resource_dir: member("closure", "lib/clang/22"),
            toolkit: "closure".into(),
            toolkit_revision: "7.2.3".into(),
            device_libraries: member("closure", "amdgcn/bitcode"),
            runtime_libraries: vec![member("closure", "lib/libc.so.6")],
            architecture: "gfx900".into(),
            optimization: 2,
            defines: BTreeMap::new(),
            entry_points: vec!["mantle_add_one".into()],
            source_date_epoch: 1,
        }
    }
    #[test]
    fn rejects_unsupported_backend_architecture_and_tool_combinations() {
        let good = plan();
        assert!(validate(good.clone()).is_ok());
        for (field, value) in [
            ("architecture", "native"),
            ("architecture", "gfx1100"),
            ("backend", "cuda"),
            ("toolkit", "6.2.4"),
            ("host", "aarch64-linux-gnu"),
        ] {
            let mut bad = good.clone();
            match field {
                "architecture" => bad.architecture = value.into(),
                "backend" => bad.backend = value.into(),
                "toolkit" => bad.toolkit_revision = value.into(),
                _ => bad.host_target = value.into(),
            }
            assert_eq!(validate(bad).unwrap_err().0, "gpu-combination-unsupported");
        }
        let mut bad = good;
        bad.host_compiler.version = "19.1.0".into();
        assert_eq!(validate(bad).unwrap_err().0, "gpu-compiler-unsupported");
    }
    #[test]
    fn rejects_paths_missing_roles_and_flag_injection() {
        let mut bad = plan();
        bad.kernel.member = "../escape.hip".into();
        assert_eq!(validate(bad).unwrap_err().0, "gpu-path-escape");
        bad = plan();
        bad.generated.push(member("undeclared", "config.h"));
        assert_eq!(validate(bad).unwrap_err().0, "gpu-role-undeclared");
        bad = plan();
        bad.defines.insert("X".into(), "-include /etc/passwd".into());
        assert_eq!(validate(bad).unwrap_err().0, "gpu-option-unsupported");
        bad = plan();
        bad.entry_points.push("mantle_add_one".into());
        assert_eq!(validate(bad).unwrap_err().0, "gpu-entry-point-invalid");
    }
}
