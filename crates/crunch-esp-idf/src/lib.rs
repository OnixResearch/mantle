//! ESP-IDF lowering to the ordinary signed-admission derivation path.
use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use crunch_esp_idf_core::Member;
use crunch_esp_idf_core::Plan;
use crunch_esp_idf_core::Role;
use serde::Serialize;

pub const DRIVER: &str = include_str!("driver.py");
#[derive(Debug, Serialize)]
pub struct Derivation {
    pub name: String,
    pub builder: String,
    pub args: Vec<String>,
    pub system: String,
    pub outputs: Vec<String>,
    pub addressing_mode: String,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<String>,
}
/// Root shape is not admission. Normal Mantle source admission must verify all
/// these whole objects and their closures before scheduling this derivation.
pub fn validate(plan: Plan, store_prefix: &str) -> Result<Plan, String> {
    let plan = crunch_esp_idf_core::validate(plan).map_err(|e| format!("esp-idf-plan: {e:?}"))?;
    let prefix = validate_store_prefix(store_prefix)?;
    for role in plan.roles.values() {
        validate_role_root(role, prefix, store_prefix)?;
    }
    debug_assert!(prefix.is_absolute());
    debug_assert!(prefix != Path::new("/"));
    Ok(plan)
}
/// The store prefix is an absolute directory below the store root.
fn validate_store_prefix(store_prefix: &str) -> Result<&Path, String> {
    let prefix = Path::new(store_prefix);
    let is_absolute = prefix.is_absolute();
    let is_store_root = prefix == Path::new("/");
    let has_only_root_and_normal =
        prefix.components().all(|part| matches!(part, Component::RootDir | Component::Normal(_)));
    if !is_absolute || is_store_root || !has_only_root_and_normal {
        return Err("invalid store prefix".into());
    }
    debug_assert!(is_absolute);
    debug_assert!(!is_store_root);
    Ok(prefix)
}
/// A role root names exactly one whole store object below the prefix.
fn validate_role_root(role: &Role, prefix: &Path, store_prefix: &str) -> Result<(), String> {
    let relative = Path::new(&role.root).strip_prefix(prefix).map_err(|_| "role outside store")?;
    let mut parts = relative.components();
    let Some(Component::Normal(name)) = parts.next() else {
        return Err("invalid store root".into());
    };
    if parts.next().is_some() {
        return Err("role must name whole store object".into());
    }
    let name = name.to_str().ok_or("non-UTF8 store root")?;
    if role.root != format!("{store_prefix}/{name}") {
        return Err("role root is not canonical".into());
    }
    debug_assert!(role.root.starts_with('/'));
    debug_assert!(role.root.starts_with(store_prefix));
    validate_store_object_name(name)
}
/// A store object name is a content hash followed by a single path segment.
fn validate_store_object_name(name: &str) -> Result<(), String> {
    let Some((hash, suffix)) = name.split_once('-') else {
        return Err("invalid store root".into());
    };
    let has_valid_hash = hash.len() == 32 && hash.bytes().all(|b| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&b));
    let has_valid_suffix = crunch_esp_idf_core::validate_member(suffix.into()).is_ok() && !suffix.contains('/');
    if !has_valid_hash || !has_valid_suffix {
        return Err("invalid store root".into());
    }
    debug_assert_eq!(hash.len(), 32);
    debug_assert!(!suffix.contains('/'));
    Ok(())
}
pub fn lower(plan: Plan, runner: Member, store_prefix: &str) -> Result<Derivation, String> {
    let plan = validate(plan, store_prefix)?;
    let Member {
        role: runner_role,
        member: runner_member,
    } = runner;
    let runner_member = crunch_esp_idf_core::validate_member(runner_member).map_err(|_| "invalid runner member")?;
    let role = plan.roles.get(&runner_role).ok_or("undeclared runner")?;
    let builder = format!("{}/{}", role.root, runner_member);
    debug_assert!(role.root.starts_with('/'));
    debug_assert!(builder.starts_with(&role.root));
    let json = serde_json::to_string(&plan).map_err(|e| e.to_string())?;
    Ok(Derivation {
        name: format!("{}-{}-idf-5.5.1", plan.name, plan.target),
        builder,
        args: vec!["run".into()],
        system: plan.host,
        outputs: vec!["out".into()],
        addressing_mode: "input-addressed".into(),
        env: BTreeMap::from([
            ("MANTLE_ESP_IDF_PLAN".into(), json),
            ("MANTLE_ESP_IDF_STORE_PREFIX".into(), store_prefix.into()),
        ]),
        inputs: plan.roles.into_values().map(|role| role.root).collect(),
    })
}
/// Run only *inside* the generic sandbox. Python and its complete import/runtime
/// closure are signed admitted inputs; no environment/package bootstrap occurs.
pub fn run(plan: Plan, store_prefix: &str) -> Result<(), String> {
    let plan = validate(plan, store_prefix)?;
    let python = format!("{}/{}", plan.roles[&plan.python.role].root, plan.python.member);
    let out = std::env::var("out").map_err(|_| "missing derivation output")?;
    let status = std::process::Command::new(python)
        .args(["-I", "-c", DRIVER])
        .env_clear()
        .env("MANTLE_ESP_IDF_CMAKE_GUARD", crunch_cmake::OFFLINE_GUARD)
        .env("out", out)
        .env("MANTLE_ESP_IDF_PLAN", serde_json::to_string(&plan).map_err(|e| e.to_string())?)
        .status()
        .map_err(|e| format!("starting admitted Python: {e}"))?;
    if !status.success() {
        return Err(format!("ESP-IDF driver failed: {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const STORE_PREFIX: &str = "/nix/store";
    const ROLE_ROOT: &str = "/nix/store/00000000000000000000000000000000-firmware";
    fn member(path: &str) -> Member {
        Member {
            role: "cohort".into(),
            member: path.into(),
        }
    }
    fn plan(root: &str) -> Plan {
        Plan {
            name: "firmware".into(),
            host: "x86_64-linux".into(),
            release: crunch_esp_idf_core::RELEASE.into(),
            sdk_commit: crunch_esp_idf_core::SDK_COMMIT.into(),
            target: "esp32s3".into(),
            roles: BTreeMap::from([("cohort".into(), Role {
                root: root.into(),
                tree_blake3: "a".repeat(64),
            })]),
            project: member("project"),
            sdk: member("sdk"),
            python: member("bin/python"),
            python_constraints: member("constraints.txt"),
            cmake: member("bin/cmake"),
            ninja: member("bin/ninja"),
            compiler: member("bin/xtensa-esp32s3-elf-gcc"),
            path: vec![member("bin")],
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
    fn store_prefix_and_role_roots_fail_closed() {
        assert!(validate(plan(ROLE_ROOT), STORE_PREFIX).is_ok());
        assert_eq!(validate(plan(ROLE_ROOT), "/"), Err("invalid store prefix".to_string()));
        assert_eq!(validate(plan(ROLE_ROOT), "relative/store"), Err("invalid store prefix".to_string()));
        assert_eq!(
            validate(plan("/other/store/00000000000000000000000000000000-firmware"), STORE_PREFIX),
            Err("role outside store".to_string())
        );
        assert_eq!(validate(plan("/nix/store/zzz-firmware"), STORE_PREFIX), Err("invalid store root".to_string()));
        assert_eq!(
            validate(plan("/nix/store/00000000000000000000000000000000-firmware/extra"), STORE_PREFIX),
            Err("role must name whole store object".to_string())
        );
        assert_eq!(
            validate(plan("/nix/store//00000000000000000000000000000000-firmware"), STORE_PREFIX),
            Err("role root is not canonical".to_string())
        );
    }
}
