#![no_std]
//! Exact Zephyr firmware decisions. This crate grants no execution or store authority.
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
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
use serde::Deserialize;
use serde::Serialize;

pub const MAX_PLAN_BYTES: usize = 131_072;
pub const ZEPHYR_REVISION: &str = "36940db938a8f4a1e919496793ed439850a221c2";
pub const CMSIS_REVISION: &str = "4b96cbb174678dcd3ca86e11e1f24bc5f8726da0";
pub const BOARD: &str = "qemu_cortex_m3";
pub const ARTIFACTS: [&str; 7] = [
    "zephyr.elf",
    "zephyr.bin",
    "zephyr.hex",
    "zephyr.map",
    ".config",
    "zephyr.dts",
    "include/generated/zephyr/devicetree_generated.h",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub root: String,
    /// mantle-zephyr-tree-v1 canonical BLAKE3, independently pinned at acquisition.
    pub tree_blake3: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub role: String,
    pub revision: String,
    pub path: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedWest {
    pub schema: String,
    pub zephyr_revision: String,
    pub modules: BTreeMap<String, Module>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub cmake: Tool,
    pub ninja: Tool,
    pub python: Tool,
    pub dtc: Tool,
    /// Explicit build-host generators, including the required GNU gperf.
    pub host: Vec<Tool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: String,
    pub name: String,
    pub store_prefix: String,
    pub build_platform: String,
    pub roles: BTreeMap<String, Input>,
    pub zephyr: String,
    pub sdk: String,
    pub sdk_version: String,
    pub target: String,
    pub project: String,
    pub project_subdir: String,
    pub resolved_west: Member,
    pub west: ResolvedWest,
    pub board: String,
    pub qualifiers: Vec<String>,
    /// Ordered fragments and overlays; these exact paths enter CMake, never auto-discovery.
    pub kconfig: Vec<Member>,
    pub overlays: Vec<Member>,
    pub tools: Tools,
    pub source_date_epoch: u64,
    pub jobs: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub code: String,
    pub detail: String,
}
impl core::fmt::Display for Rejection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}
/// Rejection code and detail carried together so the two cannot be swapped.
struct RejectionRequest<'a> {
    code: &'a str,
    detail: &'a str,
}

fn reject(request: RejectionRequest<'_>) -> Rejection {
    Rejection {
        code: request.code.into(),
        detail: request.detail.into(),
    }
}
fn digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
impl core::error::Error for Rejection {}
impl Plan {
    fn root(&self, role: &str) -> Result<&str, Rejection> {
        self.roles.get(role).map(|input| input.root.as_str()).ok_or_else(|| {
            reject(RejectionRequest {
                code: "zephyr-missing-role",
                detail: role,
            })
        })
    }
    fn check_member(&self, member: &Member) -> Result<(), Rejection> {
        if !relative(&member.member) {
            return Err(reject(RejectionRequest {
                code: "zephyr-path-escape",
                detail: &member.member,
            }));
        }
        self.root(&member.role)?;
        Ok(())
    }
    fn member(&self, member: &Member) -> Result<String, Rejection> {
        self.check_member(member)?;
        Ok(format!("{}/{}", self.roles[&member.role].root, member.member))
    }
    fn check_tool(&self, tool: &Tool) -> Result<(), Rejection> {
        if tool.version.is_empty() || tool.version.len() > 128 || tool.version.contains(['\n', '\r', '\0']) {
            return Err(reject(RejectionRequest {
                code: "zephyr-tool-version",
                detail: &tool.role,
            }));
        }
        if !relative(&tool.member) {
            return Err(reject(RejectionRequest {
                code: "zephyr-path-escape",
                detail: &tool.member,
            }));
        }
        self.root(&tool.role)?;
        Ok(())
    }
    fn tool(&self, tool: &Tool) -> Result<String, Rejection> {
        self.check_tool(tool)?;
        Ok(format!("{}/{}", self.roles[&tool.role].root, tool.member))
    }
    fn tools(&self) -> impl Iterator<Item = &Tool> {
        [
            &self.tools.cmake,
            &self.tools.ninja,
            &self.tools.python,
            &self.tools.dtc,
        ]
        .into_iter()
        .chain(self.tools.host.iter())
    }
}
/// Supported cohort is intentionally finite, not an implied board compatibility promise.
fn validate_ref(plan: &Plan) -> Result<(), Rejection> {
    check_plan_identity(plan)?;
    check_store_roles(plan)?;
    check_configuration(plan)?;
    check_tool_cohort(plan)?;
    debug_assert!(plan.roles.len() <= 64);
    debug_assert!(!plan.kconfig.is_empty());
    Ok(())
}

/// Plan identity: exact schema, plain name, Linux host and a bounded job count.
fn check_plan_identity(plan: &Plan) -> Result<(), Rejection> {
    let is_schema_admitted = plan.schema == "mantle-zephyr-plan-v1";
    let is_name_admitted = identifier(&plan.name);
    let is_build_platform_admitted = plan.build_platform == "x86_64-linux";
    let is_jobs_admitted = plan.jobs > 0 && plan.jobs <= 64;
    let is_plan_admitted = is_schema_admitted && is_name_admitted && is_build_platform_admitted && is_jobs_admitted;
    if !is_plan_admitted {
        return Err(reject(RejectionRequest {
            code: "zephyr-plan-invalid",
            detail: "schema, name, Linux host or jobs",
        }));
    }
    if plan.sdk_version != "0.16.8" || plan.target != "arm-zephyr-eabi" {
        return Err(reject(RejectionRequest {
            code: "zephyr-toolchain-unsupported",
            detail: "requires SDK 0.16.8 / arm-zephyr-eabi",
        }));
    }
    if plan.board != BOARD || !plan.qualifiers.is_empty() {
        return Err(reject(RejectionRequest {
            code: "zephyr-board-unsupported",
            detail: &plan.board,
        }));
    }
    debug_assert!(identifier(&plan.name));
    debug_assert!(plan.board == BOARD);
    Ok(())
}

/// Absolute store prefix with a bounded set of declared, digested role roots.
fn check_store_roles(plan: &Plan) -> Result<(), Rejection> {
    let is_prefix_absolute = plan.store_prefix.starts_with('/');
    let is_prefix_identifiers = is_prefix_absolute && plan.store_prefix[1..].split('/').all(identifier);
    let has_roles = !plan.roles.is_empty();
    let is_within_role_bound = plan.roles.len() <= 64;
    let is_store_admissible = is_prefix_absolute && is_prefix_identifiers && has_roles && is_within_role_bound;
    if !is_store_admissible {
        return Err(reject(RejectionRequest {
            code: "zephyr-plan-invalid",
            detail: "prefix or role bound",
        }));
    }
    for (role, input) in &plan.roles {
        let is_role_named = identifier(role);
        let is_root_resolved = store_root(&input.root, StorePrefix(&plan.store_prefix));
        let is_digested = digest(&input.tree_blake3);
        if !is_role_named || !is_root_resolved || !is_digested {
            return Err(reject(RejectionRequest {
                code: "zephyr-input-invalid",
                detail: role,
            }));
        }
    }
    for role in [&plan.zephyr, &plan.sdk, &plan.project] {
        plan.root(role)?;
    }
    debug_assert!(plan.roles.contains_key(&plan.zephyr));
    debug_assert!(plan.roles.contains_key(&plan.project));
    Ok(())
}

/// Project subdirectory, west closure and the bounded configuration cohort.
fn check_configuration(plan: &Plan) -> Result<(), Rejection> {
    if !plan.project_subdir.is_empty() && !relative(&plan.project_subdir) {
        return Err(reject(RejectionRequest {
            code: "zephyr-path-escape",
            detail: &plan.project_subdir,
        }));
    }
    validate_west(plan)?;
    let has_kconfig = !plan.kconfig.is_empty();
    let is_kconfig_bounded = plan.kconfig.len() <= 16;
    let is_overlay_bounded = plan.overlays.len() <= 16;
    let is_host_bounded = plan.tools.host.len() <= 32;
    let is_within_bounds = has_kconfig && is_kconfig_bounded && is_overlay_bounded && is_host_bounded;
    if !is_within_bounds {
        return Err(reject(RejectionRequest {
            code: "zephyr-plan-limit",
            detail: "configuration, overlay or host tool count",
        }));
    }
    for member in plan.kconfig.iter().chain(&plan.overlays).chain(core::iter::once(&plan.resolved_west)) {
        plan.check_member(member)?;
    }
    for tool in plan.tools() {
        plan.check_tool(tool)?;
    }
    debug_assert!(!plan.kconfig.is_empty());
    debug_assert!(plan.kconfig.len() <= 16);
    Ok(())
}

/// Exact host tool cohort.
fn check_tool_cohort(plan: &Plan) -> Result<(), Rejection> {
    // Content roots, not these version strings, are authority.
    let is_cmake_admitted = plan.tools.cmake.version == "3.30.5";
    let is_ninja_admitted = plan.tools.ninja.version == "1.12.1";
    let is_python_admitted = plan.tools.python.version == "3.11.10";
    let is_dtc_admitted = plan.tools.dtc.version == "1.7.0";
    let is_cohort_admitted = is_cmake_admitted && is_ninja_admitted && is_python_admitted && is_dtc_admitted;
    if !is_cohort_admitted {
        return Err(reject(RejectionRequest {
            code: "zephyr-tool-unsupported",
            detail: "requires CMake 3.30.5, Ninja 1.12.1, Python 3.11.10, dtc 1.7.0",
        }));
    }
    if plan.tools.host.iter().filter(|tool| tool.member == "bin/gperf" && tool.version == "3.1").count() != 1 {
        return Err(reject(RejectionRequest {
            code: "zephyr-tool-unsupported",
            detail: "requires exactly one declared bin/gperf 3.1",
        }));
    }
    debug_assert!(plan.tools.cmake.version == "3.30.5");
    debug_assert!(plan.tools.dtc.version == "1.7.0");
    Ok(())
}
fn validate_west(plan: &Plan) -> Result<(), Rejection> {
    if plan.west.schema != "mantle-zephyr-resolved-west-v1" || plan.west.zephyr_revision != ZEPHYR_REVISION {
        return Err(reject(RejectionRequest {
            code: "zephyr-manifest-mismatch",
            detail: "Zephyr v3.7.0 exact commit required",
        }));
    }
    if plan.west.modules.len() != 1 {
        return Err(reject(RejectionRequest {
            code: "zephyr-missing-module",
            detail: "exact explicit CMSIS closure required",
        }));
    }
    let cmsis = plan.west.modules.get("cmsis").ok_or_else(|| {
        reject(RejectionRequest {
            code: "zephyr-missing-module",
            detail: "cmsis",
        })
    })?;
    if cmsis.revision != CMSIS_REVISION || cmsis.path != "modules/hal/cmsis" {
        return Err(reject(RejectionRequest {
            code: "zephyr-manifest-mismatch",
            detail: "CMSIS revision/path differs from v3.7.0 west.yml",
        }));
    }
    plan.root(&cmsis.role)?;
    debug_assert_eq!(plan.west.modules.len(), 1);
    debug_assert!(!cmsis.revision.is_empty());
    Ok(())
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
/// Store prefix namespace; wrapped so a prefix cannot be swapped with a root path.
struct StorePrefix<'a>(&'a str);

fn store_root(value: &str, prefix: StorePrefix<'_>) -> bool {
    let StorePrefix(prefix) = prefix;
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
#[derive(Debug)]
pub struct Admission {
    pub plan: Plan,
    /// Fixed tool order followed by declared host tools.
    pub tool_paths: Vec<String>,
    /// Kconfig fragments, overlays, then the resolved west file.
    pub member_paths: Vec<String>,
    /// Explicit acquired CMSIS metadata and header paths, in module order.
    pub module_paths: Vec<String>,
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
    let tool_paths = plan.tools().map(|tool| plan.tool(tool)).collect::<Result<_, _>>()?;
    let member_paths = plan
        .kconfig
        .iter()
        .chain(&plan.overlays)
        .chain(core::iter::once(&plan.resolved_west))
        .map(|member| plan.member(member))
        .collect::<Result<_, _>>()?;
    let mut module_paths = Vec::with_capacity(plan.west.modules.len().saturating_mul(2));
    for module in plan.west.modules.values() {
        for required in ["zephyr/module.yml", "CMSIS/Core/Include/cmsis_gcc.h"] {
            module_paths.push(format!("{}/{}", plan.root(&module.role)?, required));
        }
    }
    debug_assert_eq!(module_paths.len(), plan.west.modules.len().saturating_mul(2));
    debug_assert_eq!(plan.west.modules.len(), 1);
    Ok(Admission {
        plan,
        tool_paths,
        member_paths,
        module_paths,
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    fn fixture() -> Plan {
        let role = String::from("input");
        let tool = |version: &str| Tool {
            role: role.clone(),
            member: "bin/tool".into(),
            version: version.into(),
        };
        Plan {
            schema: "mantle-zephyr-plan-v1".into(),
            name: "firmware".into(),
            store_prefix: "/mantle/store".into(),
            build_platform: "x86_64-linux".into(),
            roles: BTreeMap::from([(role.clone(), Input {
                root: "/mantle/store/00000000000000000000000000000000-input".into(),
                tree_blake3: "a".repeat(64),
            })]),
            zephyr: role.clone(),
            sdk: role.clone(),
            project: role.clone(),
            project_subdir: String::new(),
            sdk_version: "0.16.8".into(),
            target: "arm-zephyr-eabi".into(),
            resolved_west: Member {
                role: role.clone(),
                member: "resolved-west.json".into(),
            },
            west: ResolvedWest {
                schema: "mantle-zephyr-resolved-west-v1".into(),
                zephyr_revision: ZEPHYR_REVISION.into(),
                modules: BTreeMap::from([("cmsis".into(), Module {
                    role: role.clone(),
                    revision: CMSIS_REVISION.into(),
                    path: "modules/hal/cmsis".into(),
                })]),
            },
            board: BOARD.into(),
            qualifiers: vec![],
            kconfig: vec![Member {
                role: role.clone(),
                member: "prj.conf".into(),
            }],
            overlays: vec![],
            tools: Tools {
                cmake: tool("3.30.5"),
                ninja: tool("1.12.1"),
                python: tool("3.11.10"),
                dtc: tool("1.7.0"),
                host: vec![Tool {
                    role: role.clone(),
                    member: "bin/gperf".into(),
                    version: "3.1".into(),
                }],
            },
            source_date_epoch: 1721992744,
            jobs: 1,
        }
    }
    #[test]
    fn rejects_incomplete_or_misresolved_module_closure() {
        let mut plan = fixture();
        plan = validate(plan).unwrap();
        plan.west.modules.clear();
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-missing-module");
        let mut plan = fixture();
        plan.west.modules.get_mut("cmsis").unwrap().revision = "0".repeat(40);
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-manifest-mismatch");
    }
    #[test]
    fn rejects_board_qualifiers_and_escaping_configuration() {
        let mut plan = fixture();
        plan.qualifiers.push("cpu0".into());
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-board-unsupported");
        plan = fixture();
        plan.kconfig[0].member = "../prj.conf".into();
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-path-escape");
        plan = fixture();
        plan.kconfig[0].role = "undeclared".into();
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-missing-role");
    }
    #[test]
    fn roots_and_versions_cannot_substitute_for_content() {
        let mut plan = fixture();
        plan.roles.get_mut("input").unwrap().tree_blake3.clear();
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-input-invalid");
        plan = fixture();
        plan.tools.python.version = "3.11".into();
        assert_eq!(validate(plan).unwrap_err().code, "zephyr-tool-unsupported");
    }
}
