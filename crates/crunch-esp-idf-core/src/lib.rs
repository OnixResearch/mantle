#![no_std]
//! ESP-IDF release-specific, effect-free authoring and validation.
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const RELEASE: &str = "5.5.1";
pub const SDK_COMMIT: &str = "fcae32885b0296b32044cb99ecbdc50d98dddb83";
pub const MAX_ROLES: usize = 128;
pub const MAX_MEMBERS: usize = 65536;
pub const MAX_PATH_BYTES: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    /// Whole immutable admitted output, never an arbitrary executable path.
    pub root: String,
    /// BLAKE3 of the canonical file inventory documented by the adapter.
    pub tree_blake3: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub role: String,
    pub member: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: String,
    pub source: Member,
    pub managed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub name: String,
    pub host: String,
    pub release: String,
    pub sdk_commit: String,
    pub target: String,
    pub roles: BTreeMap<String, Role>,
    pub project: Member,
    pub sdk: Member,
    pub python: Member,
    pub python_constraints: Member,
    pub cmake: Member,
    pub ninja: Member,
    pub compiler: Member,
    /// Tool and runtime bin directories in search order; no inherited PATH.
    pub path: Vec<Member>,
    pub sdkconfig: Option<String>,
    /// All are relative to project; common first, target-specific second.
    pub common_defaults: Vec<String>,
    pub target_defaults: Vec<String>,
    pub partition_csv: String,
    pub components: Vec<Component>,
    pub dependencies_lock: Option<String>,
    pub source_date_epoch: u64,
    pub project_version: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedCohort,
    InvalidName,
    InvalidRole,
    UndeclaredMember,
    PathEscape,
    TargetToolchainMismatch,
    MissingLock,
    DuplicateComponent,
    UnboundedInput,
}
fn safe_member_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PATH_BYTES
        && value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.+".contains(&b))
        })
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && value.bytes().any(|b| b != b'0')
}
fn member(plan: &Plan, value: &Member) -> Result<(), Error> {
    if !plan.roles.contains_key(&value.role) {
        return Err(Error::UndeclaredMember);
    }
    if !safe_member_ref(&value.member) {
        return Err(Error::PathEscape);
    }
    Ok(())
}
fn validate_ref(plan: &Plan) -> Result<(), Error> {
    if plan.host != "x86_64-linux" || plan.release != RELEASE || plan.sdk_commit != SDK_COMMIT {
        return Err(Error::UnsupportedCohort);
    }
    let compiler = match plan.target.as_str() {
        "esp32s3" => "xtensa-esp32s3-elf-gcc",
        "esp32c3" => "riscv32-esp-elf-gcc",
        _ => return Err(Error::UnsupportedCohort),
    };
    if plan.compiler.member.rsplit('/').next() != Some(compiler) {
        return Err(Error::TargetToolchainMismatch);
    }
    if !safe_member_ref(&plan.name)
        || plan.name.contains('/')
        || !safe_member_ref(&plan.project_version)
        || plan.project_version.contains('/')
    {
        return Err(Error::InvalidName);
    }
    if plan.roles.is_empty()
        || plan.roles.len() > MAX_ROLES
        || plan.components.len() > 256
        || plan.common_defaults.len() + plan.target_defaults.len() > 128
        || plan.path.len() > MAX_ROLES
    {
        return Err(Error::UnboundedInput);
    }
    for (name, role) in &plan.roles {
        if !safe_member_ref(name) || name.contains('/') || !digest(&role.tree_blake3) {
            return Err(Error::InvalidRole);
        }
    }
    for value in [
        &plan.project,
        &plan.sdk,
        &plan.python,
        &plan.python_constraints,
        &plan.cmake,
        &plan.ninja,
        &plan.compiler,
    ] {
        member(plan, value)?;
    }
    for value in &plan.path {
        member(plan, value)?;
    }
    for value in plan
        .common_defaults
        .iter()
        .chain(&plan.target_defaults)
        .chain(plan.sdkconfig.iter())
        .chain(plan.dependencies_lock.iter())
        .chain(core::iter::once(&plan.partition_csv))
    {
        if !safe_member_ref(value) {
            return Err(Error::PathEscape);
        }
    }
    for (index, component) in plan.components.iter().enumerate() {
        member(plan, &component.source)?;
        if !safe_member_ref(&component.name) || component.name.contains('/') {
            return Err(Error::InvalidName);
        }
        if plan.components[..index].iter().any(|other| other.name == component.name) {
            return Err(Error::DuplicateComponent);
        }
        if component.managed && plan.dependencies_lock.is_none() {
            return Err(Error::MissingLock);
        }
    }
    Ok(())
}

pub fn validate_member(value: String) -> Result<String, Error> {
    if safe_member_ref(&value) {
        Ok(value)
    } else {
        Err(Error::PathEscape)
    }
}
pub fn validate(plan: Plan) -> Result<Plan, Error> {
    validate_ref(&plan)?;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan(target: &str) -> Plan {
        let input = |name: &str| Member {
            role: "cohort".into(),
            member: name.into(),
        };
        Plan {
            name: "mantle_firmware".into(),
            host: "x86_64-linux".into(),
            release: RELEASE.into(),
            sdk_commit: SDK_COMMIT.into(),
            target: target.into(),
            roles: BTreeMap::from([("cohort".into(), Role {
                root: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-sdk".into(),
                tree_blake3: "a".repeat(64),
            })]),
            project: input("project"),
            sdk: input("sdk"),
            python: input("bin/python"),
            python_constraints: input("constraints.txt"),
            cmake: input("bin/cmake"),
            ninja: input("bin/ninja"),
            compiler: input(if target == "esp32s3" {
                "bin/xtensa-esp32s3-elf-gcc"
            } else {
                "bin/riscv32-esp-elf-gcc"
            }),
            path: alloc::vec![input("bin")],
            sdkconfig: None,
            common_defaults: Vec::new(),
            target_defaults: Vec::new(),
            partition_csv: "partitions.csv".into(),
            components: Vec::new(),
            dependencies_lock: None,
            source_date_epoch: 1700000000,
            project_version: "1.0.0".into(),
        }
    }
    #[test]
    fn target_cohort_and_toolchain_are_jointly_validated() {
        for target in ["esp32s3", "esp32c3"] {
            let mut value = plan(target);
            assert!(validate(value.clone()).is_ok());
            value.target = if target == "esp32s3" {
                "esp32c3".into()
            } else {
                "esp32s3".into()
            };
            assert_eq!(validate(value), Err(Error::TargetToolchainMismatch));
        }
        let mut value = plan("esp32s3");
        value.release = "5.5.2".into();
        assert_eq!(validate(value), Err(Error::UnsupportedCohort));
    }
    #[test]
    fn managed_components_require_lock_and_declared_content() {
        let mut value = plan("esp32s3");
        value.components.push(Component {
            name: "vendor__component".into(),
            source: Member {
                role: "cohort".into(),
                member: "component".into(),
            },
            managed: true,
        });
        assert_eq!(validate(value.clone()), Err(Error::MissingLock));
        value.dependencies_lock = Some("locks/esp32s3.lock".into());
        assert!(validate(value.clone()).is_ok());
        value.components[0].source.role = "ambient".into();
        assert_eq!(validate(value.clone()), Err(Error::UndeclaredMember));
        value.components[0].source.role = "cohort".into();
        value.dependencies_lock = Some("../other-project.lock".into());
        assert_eq!(validate(value), Err(Error::PathEscape));
    }
    #[test]
    fn members_cannot_escape_or_inject_cmake_lists() {
        for value in ["../main", "/source", "a//b", "a/./b", "a\\b", "a;b", "a\nb", "C:src"] {
            assert!(validate_member(value.into()).is_err(), "{value}");
        }
        assert!(validate_member("main/idf_component.yml".into()).is_ok());
    }
    #[test]
    fn identities_are_content_digests_not_versions() {
        assert!(!digest("esp-14.2.0_20241119"));
        assert!(!digest(&"0".repeat(64)));
        assert!(digest(&"a".repeat(64)));
    }
}
