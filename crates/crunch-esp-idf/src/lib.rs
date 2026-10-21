//! ESP-IDF lowering to the ordinary signed-admission derivation path.
use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use crunch_esp_idf_core::Member;
use crunch_esp_idf_core::Plan;
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
    let prefix = Path::new(store_prefix);
    if !prefix.is_absolute()
        || prefix == Path::new("/")
        || prefix.components().any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err("invalid store prefix".into());
    }
    for role in plan.roles.values() {
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
        let Some((hash, suffix)) = name.split_once('-') else {
            return Err("invalid store root".into());
        };
        if hash.len() != 32
            || !hash.bytes().all(|b| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&b))
            || crunch_esp_idf_core::validate_member(suffix.into()).is_err()
            || suffix.contains('/')
        {
            return Err("invalid store root".into());
        }
    }
    Ok(plan)
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
