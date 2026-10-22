#![no_std]
//! Bounded CMake authoring decisions; no filesystem, process, store or network authority.
extern crate alloc;
mod planning;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

pub use planning::LowerRequest;
pub use planning::Lowered;
pub use planning::Lowering;
pub use planning::WORK_DIRECTORY;
pub use planning::lower;
use serde::Deserialize;
use serde::Serialize;

pub const MAX_PLAN_BYTES: usize = 131_072;
pub const MAX_ROLES: usize = 64;
pub const MAX_ENTRIES: usize = 64;
pub const MAX_VALUE_BYTES: usize = 4096;

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
    /// Exact first-line version substring, checked before configure.
    pub version: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub cmake: Tool,
    pub ninja: Tool,
    pub cc: Tool,
    pub cxx: Tool,
    pub ar: Tool,
    pub ranlib: Tool,
    pub linker: Tool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Compiler {
    Clang,
    Gcc,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub system: String,
    pub processor: String,
    pub triple: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: String,
    pub name: String,
    pub store_prefix: String,
    pub source: String,
    pub source_subdir: String,
    pub roles: BTreeMap<String, String>,
    pub tools: Tools,
    pub compiler: Compiler,
    pub build_platform: String,
    pub target: Target,
    pub sysroot: String,
    pub runtime_roles: Vec<String>,
    pub dependencies: Vec<String>,
    /// Only these executable directories enter PATH. They execute on the build platform.
    pub host_tools: Vec<Tool>,
    /// Explicit compiler builtin/C/C++ include directories; no ambient defaults.
    pub c_includes: Vec<Member>,
    pub cxx_includes: Vec<Member>,
    /// Optional admitted cross-toolchain fragment, included before owned settings.
    pub toolchain_file: Option<Member>,
    pub emulator: Option<Tool>,
    /// FetchContent name -> separately admitted source role. No acquisition at configure.
    pub fetch_sources: BTreeMap<String, String>,
    pub definitions: BTreeMap<String, String>,
    pub build_targets: Vec<String>,
    /// Exact installed files required; all other installed entries are still audited.
    pub outputs: Vec<String>,
    pub build_type: String,
    pub jobs: u32,
    pub source_date_epoch: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rejection {
    pub code: String,
    pub detail: String,
}
impl core::fmt::Display for Rejection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}
/// A rejection code. The newtype keeps the code distinct from the detail text
/// so the two arguments of [`reject`] cannot be swapped by accident.
struct RejectCode(&'static str);
fn reject(code: RejectCode, detail: &str) -> Rejection {
    Rejection {
        code: code.0.to_string(),
        detail: detail.to_string(),
    }
}
fn identifier_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"_+.-".contains(&b))
        && value != "."
        && value != ".."
}
fn relative_ref(value: &str) -> bool {
    value.len() <= 1024 && !value.is_empty() && value.split('/').all(identifier_ref)
}
/// An admitted store prefix. The newtype keeps it distinct from a candidate
/// path so the two arguments of [`store_root_ref`] cannot be swapped.
struct StorePrefix<'a>(&'a str);
fn store_root_ref(value: &str, prefix: StorePrefix<'_>) -> bool {
    let StorePrefix(prefix) = prefix;
    if !prefix.starts_with('/') || !prefix[1..].split('/').all(identifier_ref) {
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
        && identifier_ref(name)
        && !rest.contains('/')
}
impl Plan {
    fn role(&self, role: &str) -> Result<&str, Rejection> {
        self.roles
            .get(role)
            .map(String::as_str)
            .ok_or_else(|| reject(RejectCode("cmake-role-undeclared"), role))
    }
    fn check_member(&self, member: &Member) -> Result<(), Rejection> {
        if !relative_ref(&member.member) {
            return Err(reject(RejectCode("cmake-path-escape"), &member.member));
        }
        self.role(&member.role)?;
        Ok(())
    }
    fn member(&self, member: &Member) -> Result<String, Rejection> {
        self.check_member(member)?;
        Ok(format!("{}/{}", self.roles[&member.role], member.member))
    }
    fn check_tool(&self, tool: &Tool) -> Result<(), Rejection> {
        if tool.version.is_empty() || tool.version.len() > 128 || tool.version.contains(['\n', '\r', '\0']) {
            return Err(reject(RejectCode("cmake-tool-version"), &tool.role));
        }
        if !relative_ref(&tool.member) {
            return Err(reject(RejectCode("cmake-path-escape"), &tool.member));
        }
        self.role(&tool.role)?;
        Ok(())
    }
    fn tool(&self, tool: &Tool) -> Result<String, Rejection> {
        self.check_tool(tool)?;
        Ok(format!("{}/{}", self.roles[&tool.role], tool.member))
    }
    fn tool_list(&self) -> impl Iterator<Item = &Tool> {
        let t = &self.tools;
        [&t.cmake, &t.ninja, &t.cc, &t.cxx, &t.ar, &t.ranlib, &t.linker]
            .into_iter()
            .chain(&self.host_tools)
            .chain(&self.emulator)
    }
    fn elf_machine(&self) -> u16 {
        if self.target.processor == "x86_64" { 62 } else { 183 }
    }
}
/// Reject before any tool executes. The shell must additionally bind these roots
/// through normal store admission and run the ordinary network-denied sandbox.
fn validate_ref(plan: &Plan) -> Result<(), Rejection> {
    if plan.schema != "mantle-cmake-plan-v1" || !identifier_ref(&plan.name) {
        return Err(reject(RejectCode("cmake-plan-invalid"), "schema or name"));
    }
    let has_valid_store_prefix =
        plan.store_prefix.starts_with('/') && plan.store_prefix[1..].split('/').all(identifier_ref);
    let has_bounded_roles = !plan.roles.is_empty() && plan.roles.len() <= MAX_ROLES;
    if !has_valid_store_prefix || !has_bounded_roles {
        return Err(reject(RejectCode("cmake-plan-invalid"), "store prefix or role count"));
    }
    for (role, root) in &plan.roles {
        if !identifier_ref(role) || !store_root_ref(root, StorePrefix(&plan.store_prefix)) {
            return Err(reject(RejectCode("cmake-path-escape"), root));
        }
    }
    validate_platform(plan)?;
    validate_collections(plan)?;
    plan.role(&plan.source)?;
    plan.role(&plan.sysroot)?;
    if !plan.source_subdir.is_empty() && !relative_ref(&plan.source_subdir) {
        return Err(reject(RejectCode("cmake-path-escape"), &plan.source_subdir));
    }
    for tool in plan.tool_list() {
        plan.check_tool(tool)?;
    }
    for member in plan.c_includes.iter().chain(&plan.cxx_includes).chain(plan.toolchain_file.iter()) {
        plan.check_member(member)?;
    }
    for role in plan.runtime_roles.iter().chain(&plan.dependencies).chain(plan.fetch_sources.values()) {
        plan.role(role)?;
    }
    for output in &plan.outputs {
        if !relative_ref(output)
            || !["include", "lib", "bin", "share"].contains(&output.split('/').next().unwrap_or(""))
        {
            return Err(reject(RejectCode("cmake-output-invalid"), output));
        }
    }
    for name in plan.fetch_sources.keys() {
        if !identifier_ref(name) {
            return Err(reject(RejectCode("cmake-fetch-source-invalid"), name));
        }
    }
    validate_definitions(plan)?;
    debug_assert!(!plan.roles.is_empty());
    debug_assert!(plan.jobs > 0);
    Ok(())
}
fn validate_platform(plan: &Plan) -> Result<(), Rejection> {
    let is_known_build_platform = ["x86_64-linux", "aarch64-linux"].contains(&plan.build_platform.as_str());
    let is_known_target =
        plan.target.system == "Linux" && ["x86_64", "aarch64"].contains(&plan.target.processor.as_str());
    let has_valid_triple = identifier_ref(&plan.target.triple)
        && plan.target.triple.starts_with(&format!("{}-", plan.target.processor))
        && plan.target.triple.contains("linux");
    if !is_known_build_platform || !is_known_target || !has_valid_triple {
        return Err(reject(RejectCode("cmake-platform-unsupported"), &plan.target.triple));
    }
    if plan.jobs == 0
        || plan.jobs > 64
        || !["Release", "Debug", "RelWithDebInfo", "MinSizeRel"].contains(&plan.build_type.as_str())
    {
        return Err(reject(RejectCode("cmake-plan-invalid"), "jobs or build type"));
    }
    debug_assert!(plan.elf_machine() != 0);
    debug_assert!(!plan.target.system.is_empty());
    Ok(())
}
fn validate_collections(plan: &Plan) -> Result<(), Rejection> {
    let counts = [
        plan.runtime_roles.len(),
        plan.dependencies.len(),
        plan.host_tools.len(),
        plan.c_includes.len(),
        plan.cxx_includes.len(),
        plan.fetch_sources.len(),
        plan.definitions.len(),
        plan.build_targets.len(),
        plan.outputs.len(),
    ];
    if counts.iter().any(|n| *n > MAX_ENTRIES) || plan.outputs.is_empty() {
        return Err(reject(RejectCode("cmake-plan-limit"), "collection exceeds 64 or no outputs"));
    }
    if plan.build_targets.iter().any(|target| !identifier_ref(target)) {
        return Err(reject(RejectCode("cmake-target-invalid"), "build target"));
    }
    let unique: BTreeSet<_> = plan.outputs.iter().collect();
    if unique.len() != plan.outputs.len() {
        return Err(reject(RejectCode("cmake-output-invalid"), "duplicate output"));
    }
    debug_assert!(!plan.outputs.is_empty());
    debug_assert!(plan.outputs.len() <= MAX_ENTRIES);
    Ok(())
}
fn validate_definitions(plan: &Plan) -> Result<(), Rejection> {
    for (key, value) in &plan.definitions {
        let has_valid_key = identifier_ref(key) && !key.starts_with("CMAKE_") && !key.starts_with("FETCHCONTENT_");
        let has_valid_value = value.len() <= MAX_VALUE_BYTES
            && !value.contains(['\n', '\r', '\0', ';'])
            && (key != "BUILD_SHARED_LIBS" || ["ON", "OFF"].contains(&value.as_str()));
        if !has_valid_key || !has_valid_value {
            return Err(reject(RejectCode("cmake-definition-invalid"), key));
        }
        // Definitions may reference immutable role roots only via a whole-value marker.
        if let Some(rest) = value.strip_prefix("{{role:") {
            let Some((role, suffix)) = rest.split_once("}}") else {
                return Err(reject(RejectCode("cmake-definition-invalid"), key));
            };
            plan.role(role)?;
            if !suffix.is_empty() && !suffix.strip_prefix('/').is_some_and(relative_ref) {
                return Err(reject(RejectCode("cmake-path-escape"), value));
            }
        } else if value.contains('/') || value.contains("{{") {
            return Err(reject(RejectCode("cmake-path-escape"), value));
        }
    }
    debug_assert!(plan.definitions.len() <= MAX_ENTRIES);
    debug_assert!(!plan.roles.is_empty());
    Ok(())
}
/// Construct a CMake bracket literal without permitting CMake-language injection.
fn cmake_literal_ref(value: &str) -> String {
    let mut bracket_run = String::from("=");
    while value.contains(&format!("]{bracket_run}]")) {
        bracket_run.push('=');
    }
    format!("[{bracket_run}[{value}]{bracket_run}]")
}

/// Owned lexical requests for shell observations outside an authoring plan.
pub struct StoreRootRequest {
    pub value: String,
    pub prefix: String,
}
pub fn identifier(value: String) -> bool {
    identifier_ref(&value)
}
pub fn store_root(request: StoreRootRequest) -> bool {
    store_root_ref(&request.value, StorePrefix(&request.prefix))
}
pub fn cmake_literal(value: String) -> String {
    cmake_literal_ref(&value)
}

/// Owned runner resolution preserves the unchanged authoring wire plan.
#[derive(Debug)]
pub struct RunnerRequest {
    pub plan: Plan,
    pub runner: Tool,
}
#[derive(Debug)]
pub struct RunnerAdmission {
    pub plan: Plan,
    pub builder: String,
}
/// Admission preserves the wire plan and returns its resolved, ordered paths.
#[derive(Debug)]
pub struct Admission {
    pub plan: Plan,
    /// Same order as fixed tools, host tools, then optional emulator.
    pub tool_paths: Vec<String>,
    /// Same order as C includes, C++ includes, then optional toolchain file.
    pub member_paths: Vec<String>,
}
pub fn validate(plan: Plan) -> Result<Plan, Rejection> {
    validate_ref(&plan)?;
    Ok(plan)
}
pub fn admit_runner(request: RunnerRequest) -> Result<RunnerAdmission, Rejection> {
    let RunnerRequest { plan, runner } = request;
    validate_ref(&plan)?;
    let builder = plan.tool(&runner)?;
    Ok(RunnerAdmission { plan, builder })
}
pub fn admit(plan: Plan) -> Result<Admission, Rejection> {
    validate_ref(&plan)?;
    admit_validated(plan)
}
fn admit_validated(plan: Plan) -> Result<Admission, Rejection> {
    let tool_paths = plan.tool_list().map(|tool| plan.tool(tool)).collect::<Result<_, _>>()?;
    let member_paths = plan
        .c_includes
        .iter()
        .chain(&plan.cxx_includes)
        .chain(plan.toolchain_file.iter())
        .map(|member| plan.member(member))
        .collect::<Result<_, _>>()?;
    Ok(Admission {
        plan,
        tool_paths,
        member_paths,
    })
}
