use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::cargo_profile::CargoProfileError;
use crate::cargo_profile::CargoProfileSettings;
use crate::cargo_profile::build_override_policy_from_settings;
use crate::cargo_profile::resolve_builtin_profile;

const PROFILE_INHERITANCE_DEPTH_MAX: usize = 32;
const ROOT_MANIFEST_BYTES_MAX: usize = 4 * 1024 * 1024;
const PROFILE_COUNT_MAX: usize = 64;
const PACKAGE_OVERRIDE_COUNT_MAX: usize = 4096;
const CARGO_DEBUG_INFO_NONE: u8 = 0;
const CARGO_DEBUG_INFO_FULL: u8 = 2;
const CARGO_OPT_LEVEL_MAX: i64 = 3;
const INVALID_PROFILE_CLASS: &str = "invalid-profile";
const UNKNOWN_PROFILE_SETTING_CLASS: &str = "unknown-profile-setting";
const UNSUPPORTED_PROFILE_SETTING_CLASS: &str = "unsupported-profile-setting";
const FORBIDDEN_OVERRIDE_CLASS: &str = "forbidden-profile-override";
const UNSUPPORTED_OVERRIDE_SPEC_CLASS: &str = "unsupported-override-spec";
const DEPENDENCY_PROFILE_NON_CLAIM: &str = "dependency-manifest-profile-settings-present-and-ignored";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CargoProfileTable {
    pub(crate) profiles: BTreeMap<String, CargoProfileDefinition>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CargoProfileDefinition {
    pub(crate) inherits: Option<String>,
    pub(crate) settings: CargoProfileSettingsPatch,
    pub(crate) package: BTreeMap<String, CargoProfileSettingsPatch>,
    pub(crate) wildcard: Option<CargoProfileSettingsPatch>,
    pub(crate) build_override: Option<CargoProfileSettingsPatch>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CargoProfileSettingsPatch {
    pub(crate) opt_level: Option<String>,
    pub(crate) debuginfo: Option<u8>,
    pub(crate) debug_assertions: Option<bool>,
    pub(crate) overflow_checks: Option<bool>,
    pub(crate) lto: Option<String>,
    pub(crate) panic: Option<String>,
    pub(crate) cargo_incremental: Option<bool>,
    pub(crate) codegen_units: Option<u16>,
    pub(crate) rpath: Option<bool>,
    pub(crate) strip: Option<String>,
    pub(crate) split_debuginfo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CargoProfileUnitRole {
    Target,
    Host { dual_use: bool },
}

pub(crate) fn parse_profile_table(root_manifest: &str) -> Result<CargoProfileTable, CargoProfileError> {
    if root_manifest.len() > ROOT_MANIFEST_BYTES_MAX {
        return Err(profile_error(INVALID_PROFILE_CLASS, "root Cargo manifest exceeds the profile parser byte limit"));
    }
    let manifest = root_manifest.parse::<toml::Value>().map_err(|error| CargoProfileError {
        class: INVALID_PROFILE_CLASS,
        message: format!("parsing root Cargo manifest profile table: {error}"),
    })?;
    let Some(profile_table) = manifest.get("profile") else {
        return Ok(CargoProfileTable::default());
    };
    let profile_table = require_table(profile_table, "root `profile`")?;
    if profile_table.len() > PROFILE_COUNT_MAX {
        return Err(profile_error(INVALID_PROFILE_CLASS, "root Cargo manifest exceeds the profile count limit"));
    }
    let mut profiles = BTreeMap::new();
    for (name, value) in profile_table {
        profiles.insert(name.clone(), parse_profile_definition(name, value)?);
    }
    Ok(CargoProfileTable { profiles })
}

fn parse_profile_definition(name: &str, value: &toml::Value) -> Result<CargoProfileDefinition, CargoProfileError> {
    let table = require_table(value, &format!("profile `{name}`"))?;
    validate_profile_keys(table)?;
    let settings = parse_settings_patch(table, false)?;
    validate_lowerable_profile_settings(&settings)?;
    let inherits = table.get("inherits").map(|value| parse_string(value, "inherits")).transpose()?;
    if inherits.is_some() && resolve_builtin_profile(name).is_ok() {
        return Err(profile_error(
            INVALID_PROFILE_CLASS,
            format!("built-in Cargo profile `{name}` cannot declare `inherits`"),
        ));
    }
    let build_override =
        table.get("build-override").map(|value| parse_override_patch(value, "build-override")).transpose()?;
    let (package, wildcard) = parse_package_overrides(table.get("package"))?;
    Ok(CargoProfileDefinition {
        inherits,
        settings,
        package,
        wildcard,
        build_override,
    })
}

fn validate_profile_keys(table: &toml::value::Table) -> Result<(), CargoProfileError> {
    const PROFILE_CONTAINER_KEYS: [&str; 3] = ["inherits", "package", "build-override"];
    for key in table.keys() {
        if PROFILE_CONTAINER_KEYS.contains(&key.as_str()) || is_profile_setting_key(key) {
            continue;
        }
        return Err(profile_error(
            UNKNOWN_PROFILE_SETTING_CLASS,
            format!("Cargo profile setting `{key}` is not supported"),
        ));
    }
    Ok(())
}

fn is_profile_setting_key(key: &str) -> bool {
    matches!(
        key,
        "opt-level"
            | "debug"
            | "debug-assertions"
            | "overflow-checks"
            | "lto"
            | "panic"
            | "incremental"
            | "codegen-units"
            | "rpath"
            | "strip"
            | "split-debuginfo"
    )
}

fn parse_package_overrides(
    value: Option<&toml::Value>,
) -> Result<(BTreeMap<String, CargoProfileSettingsPatch>, Option<CargoProfileSettingsPatch>), CargoProfileError> {
    let Some(value) = value else {
        return Ok((BTreeMap::new(), None));
    };
    let table = require_table(value, "profile package override")?;
    if table.len() > PACKAGE_OVERRIDE_COUNT_MAX {
        return Err(profile_error(INVALID_PROFILE_CLASS, "Cargo profile exceeds the package override count limit"));
    }
    let mut package = BTreeMap::new();
    let mut wildcard = None;
    for (spec, override_value) in table {
        if spec.contains(':') || spec.contains('@') {
            return Err(profile_error(
                UNSUPPORTED_OVERRIDE_SPEC_CLASS,
                format!("version-qualified Cargo profile override `{spec}` is unsupported"),
            ));
        }
        let patch = parse_override_patch(override_value, spec)?;
        if spec == "*" {
            wildcard = Some(patch);
        } else {
            package.insert(spec.clone(), patch);
        }
    }
    Ok((package, wildcard))
}

fn validate_lowerable_profile_settings(settings: &CargoProfileSettingsPatch) -> Result<(), CargoProfileError> {
    let unsupported = [
        settings.lto.as_ref().filter(|value| value.as_str() != "false").map(|_| "lto"),
        settings.panic.as_ref().filter(|value| value.as_str() != "unwind").map(|_| "panic"),
        settings.rpath.filter(|value| *value).map(|_| "rpath"),
        settings.strip.as_ref().filter(|value| value.as_str() != "none").map(|_| "strip"),
        settings.split_debuginfo.as_ref().map(|_| "split-debuginfo"),
    ]
    .into_iter()
    .flatten()
    .next();
    if let Some(key) = unsupported {
        return Err(profile_error(
            UNSUPPORTED_PROFILE_SETTING_CLASS,
            format!("Cargo profile setting `{key}` is parsed but not lowered by Mantle"),
        ));
    }
    Ok(())
}

fn parse_override_patch(value: &toml::Value, label: &str) -> Result<CargoProfileSettingsPatch, CargoProfileError> {
    let table = require_table(value, &format!("profile override `{label}`"))?;
    for key in table.keys() {
        if !is_profile_setting_key(key) {
            return Err(profile_error(
                UNKNOWN_PROFILE_SETTING_CLASS,
                format!("Cargo profile override setting `{key}` is not supported"),
            ));
        }
        if matches!(key.as_str(), "panic" | "lto" | "rpath") {
            return Err(profile_error(
                FORBIDDEN_OVERRIDE_CLASS,
                format!("Cargo profile override `{label}` cannot set `{key}`"),
            ));
        }
    }
    let patch = parse_settings_patch(table, true)?;
    validate_lowerable_profile_settings(&patch)?;
    Ok(patch)
}

fn parse_settings_patch(
    table: &toml::value::Table,
    is_override: bool,
) -> Result<CargoProfileSettingsPatch, CargoProfileError> {
    let (lto, panic, rpath) = if is_override {
        (None, None, None)
    } else {
        (
            table.get("lto").map(parse_lto).transpose()?,
            table.get("panic").map(parse_panic).transpose()?,
            table.get("rpath").map(|value| parse_bool(value, "rpath")).transpose()?,
        )
    };
    Ok(CargoProfileSettingsPatch {
        opt_level: table.get("opt-level").map(parse_opt_level).transpose()?,
        debuginfo: table.get("debug").map(parse_debuginfo).transpose()?,
        debug_assertions: table
            .get("debug-assertions")
            .map(|value| parse_bool(value, "debug-assertions"))
            .transpose()?,
        overflow_checks: table.get("overflow-checks").map(|value| parse_bool(value, "overflow-checks")).transpose()?,
        lto,
        panic,
        cargo_incremental: table.get("incremental").map(|value| parse_bool(value, "incremental")).transpose()?,
        codegen_units: table.get("codegen-units").map(|value| parse_u16(value, "codegen-units")).transpose()?,
        rpath,
        strip: table.get("strip").map(parse_strip).transpose()?,
        split_debuginfo: table.get("split-debuginfo").map(parse_split_debuginfo).transpose()?,
    })
}

pub(crate) fn resolve_profile(
    table: &CargoProfileTable,
    name: &str,
) -> Result<CargoProfileSettings, CargoProfileError> {
    resolve_profile_inner(table, name, &mut BTreeSet::new(), 0)
}

fn resolve_profile_inner(
    table: &CargoProfileTable,
    name: &str,
    visiting: &mut BTreeSet<String>,
    depth: usize,
) -> Result<CargoProfileSettings, CargoProfileError> {
    if depth >= PROFILE_INHERITANCE_DEPTH_MAX {
        return Err(profile_error(INVALID_PROFILE_CLASS, "Cargo profile inheritance exceeds its depth limit"));
    }
    if !visiting.insert(name.to_string()) {
        return Err(profile_error(INVALID_PROFILE_CLASS, format!("Cargo profile inheritance cycle includes `{name}`")));
    }
    let definition = table.profiles.get(name);
    let mut settings = match (resolve_builtin_profile(name), definition) {
        (Ok(settings), _) => settings,
        (Err(_), Some(definition)) => {
            let Some(base) = definition.inherits.as_deref() else {
                return Err(profile_error(
                    INVALID_PROFILE_CLASS,
                    format!("custom Cargo profile `{name}` must declare `inherits`"),
                ));
            };
            resolve_profile_inner(table, base, visiting, depth.saturating_add(1))?
        }
        (Err(error), None) => return Err(error),
    };
    if let Some(definition) = definition {
        apply_patch(&mut settings, &definition.settings);
    }
    visiting.remove(name);
    Ok(settings)
}

pub(crate) fn select_unit_profile(
    table: &CargoProfileTable,
    name: &str,
    role: CargoProfileUnitRole,
    package_name: &str,
    is_workspace_member: bool,
) -> Result<CargoProfileSettings, CargoProfileError> {
    let mut settings = resolve_profile(table, name)?;
    let lineage = profile_lineage(table, name)?;
    if let CargoProfileUnitRole::Host { dual_use } = role {
        settings = built_in_build_override(settings, dual_use);
        for definition in &lineage {
            if let Some(patch) = &definition.build_override {
                apply_patch(&mut settings, patch);
            }
        }
    }
    if !is_workspace_member {
        for definition in &lineage {
            if let Some(patch) = &definition.wildcard {
                apply_patch(&mut settings, patch);
            }
        }
    }
    for definition in lineage {
        if let Some(patch) = definition.package.get(package_name) {
            apply_patch(&mut settings, patch);
        }
    }
    Ok(settings)
}

fn profile_lineage<'a>(
    table: &'a CargoProfileTable,
    name: &str,
) -> Result<Vec<&'a CargoProfileDefinition>, CargoProfileError> {
    let mut names = Vec::new();
    let mut current = name;
    for _ in 0..PROFILE_INHERITANCE_DEPTH_MAX {
        let Some(definition) = table.profiles.get(current) else {
            break;
        };
        names.push(current.to_string());
        let Some(parent) = definition.inherits.as_deref() else {
            break;
        };
        current = parent;
    }
    if names.len() >= PROFILE_INHERITANCE_DEPTH_MAX {
        return Err(profile_error(INVALID_PROFILE_CLASS, "Cargo profile inheritance exceeds its depth limit"));
    }
    names.reverse();
    Ok(names.iter().filter_map(|name| table.profiles.get(name)).collect())
}

fn built_in_build_override(settings: CargoProfileSettings, dual_use: bool) -> CargoProfileSettings {
    build_override_policy_from_settings(settings, dual_use).settings
}

fn apply_patch(settings: &mut CargoProfileSettings, patch: &CargoProfileSettingsPatch) {
    if let Some(value) = &patch.opt_level {
        settings.opt_level = value.clone();
    }
    if let Some(value) = patch.debuginfo {
        settings.debuginfo = value;
    }
    if let Some(value) = patch.debug_assertions {
        settings.debug_assertions = value;
    }
    if let Some(value) = patch.overflow_checks {
        settings.overflow_checks = value;
    }
    if let Some(value) = &patch.lto {
        settings.lto = value.clone();
    }
    if let Some(value) = &patch.panic {
        settings.panic = value.clone();
    }
    if let Some(value) = patch.cargo_incremental {
        settings.cargo_incremental = value;
    }
    if let Some(value) = patch.codegen_units {
        settings.codegen_units = value;
    }
    if let Some(value) = patch.rpath {
        settings.rpath = value;
    }
    if let Some(value) = &patch.strip {
        settings.strip = value.clone();
    }
    if let Some(value) = &patch.split_debuginfo {
        settings.split_debuginfo = value.clone();
    }
}

pub(crate) fn select_command_profile(
    command: &str,
    release_flag: bool,
    profile_flag: Option<&str>,
) -> Result<String, CargoProfileError> {
    if release_flag && profile_flag.is_some_and(|profile| profile != "release") {
        return Err(profile_error(INVALID_PROFILE_CLASS, "`--release` conflicts with an explicit non-release profile"));
    }
    if release_flag {
        return Ok("release".to_string());
    }
    if let Some(profile) = profile_flag {
        return Ok(profile.to_string());
    }
    let selected = match command {
        "test" => "test",
        "bench" => "bench",
        "install" => "release",
        _ => "dev",
    };
    Ok(selected.to_string())
}

pub(crate) fn dependency_profile_non_claims(dependency_manifests: &[&str]) -> Vec<String> {
    let profile_is_present = dependency_manifests
        .iter()
        .any(|text| text.parse::<toml::Value>().ok().and_then(|manifest| manifest.get("profile").cloned()).is_some());
    if profile_is_present {
        vec![DEPENDENCY_PROFILE_NON_CLAIM.to_string()]
    } else {
        Vec::new()
    }
}

fn require_table<'a>(value: &'a toml::Value, label: &str) -> Result<&'a toml::value::Table, CargoProfileError> {
    value
        .as_table()
        .ok_or_else(|| profile_error(INVALID_PROFILE_CLASS, format!("Cargo {label} must be a table")))
}

fn parse_opt_level(value: &toml::Value) -> Result<String, CargoProfileError> {
    if let Some(value) = value.as_str()
        && matches!(value, "0" | "1" | "2" | "3" | "s" | "z")
    {
        return Ok(value.to_string());
    }
    if let Some(value) = value.as_integer()
        && (0..=CARGO_OPT_LEVEL_MAX).contains(&value)
    {
        return Ok(value.to_string());
    }
    Err(profile_error(INVALID_PROFILE_CLASS, "Cargo profile `opt-level` must be 0, 1, 2, 3, `s`, or `z`"))
}

fn parse_debuginfo(value: &toml::Value) -> Result<u8, CargoProfileError> {
    if let Some(value) = value.as_bool() {
        return Ok(if value {
            CARGO_DEBUG_INFO_FULL
        } else {
            CARGO_DEBUG_INFO_NONE
        });
    }
    let value = value
        .as_integer()
        .ok_or_else(|| profile_error(INVALID_PROFILE_CLASS, "Cargo profile `debug` must be a boolean or integer"))?;
    let value = u8::try_from(value)
        .map_err(|_| profile_error(INVALID_PROFILE_CLASS, "Cargo profile `debug` is out of range"))?;
    if value > CARGO_DEBUG_INFO_FULL {
        return Err(profile_error(INVALID_PROFILE_CLASS, "Cargo profile `debug` must be 0, 1, or 2"));
    }
    Ok(value)
}

fn parse_lto(value: &toml::Value) -> Result<String, CargoProfileError> {
    if let Some(value) = value.as_bool() {
        return Ok(value.to_string());
    }
    parse_choice(value, "lto", &["off", "thin", "fat"])
}

fn parse_strip(value: &toml::Value) -> Result<String, CargoProfileError> {
    if let Some(value) = value.as_bool() {
        return Ok(if value { "symbols" } else { "none" }.to_string());
    }
    parse_choice(value, "strip", &["none", "debuginfo", "symbols"])
}

fn parse_panic(value: &toml::Value) -> Result<String, CargoProfileError> {
    parse_choice(value, "panic", &["unwind", "abort"])
}

fn parse_split_debuginfo(value: &toml::Value) -> Result<String, CargoProfileError> {
    parse_choice(value, "split-debuginfo", &["off", "unpacked", "packed"])
}

fn parse_choice(value: &toml::Value, key: &str, choices: &[&str]) -> Result<String, CargoProfileError> {
    let value = parse_string(value, key)?;
    if choices.contains(&value.as_str()) {
        return Ok(value);
    }
    Err(profile_error(
        INVALID_PROFILE_CLASS,
        format!("Cargo profile `{key}` has unsupported value `{value}`"),
    ))
}

fn parse_string(value: &toml::Value, key: &str) -> Result<String, CargoProfileError> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| profile_error(INVALID_PROFILE_CLASS, format!("Cargo profile `{key}` must be a string")))
}

fn parse_bool(value: &toml::Value, key: &str) -> Result<bool, CargoProfileError> {
    value
        .as_bool()
        .ok_or_else(|| profile_error(INVALID_PROFILE_CLASS, format!("Cargo profile `{key}` must be a boolean")))
}

fn parse_u16(value: &toml::Value, key: &str) -> Result<u16, CargoProfileError> {
    let value = value
        .as_integer()
        .ok_or_else(|| profile_error(INVALID_PROFILE_CLASS, format!("Cargo profile `{key}` must be an integer")))?;
    let value = u16::try_from(value)
        .map_err(|_| profile_error(INVALID_PROFILE_CLASS, format!("Cargo profile `{key}` is out of range")))?;
    if value == 0 {
        return Err(profile_error(INVALID_PROFILE_CLASS, format!("Cargo profile `{key}` must be positive")));
    }
    Ok(value)
}

fn profile_error(class: &'static str, message: impl Into<String>) -> CargoProfileError {
    CargoProfileError {
        class,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_profile_inherits_release_defaults_and_overrides_one_setting() {
        let table = parse_profile_table(
            r#"
[profile.release-lto]
inherits = "release"
debug = 1
"#,
        )
        .unwrap();

        let settings = resolve_profile(&table, "release-lto").unwrap();
        let release = resolve_builtin_profile("release").unwrap();

        assert_eq!(settings.debuginfo, 1);
        assert_eq!(settings.opt_level, release.opt_level);
        assert_eq!(settings.codegen_units, release.codegen_units);
        assert_eq!(settings.debug_assertions, release.debug_assertions);
    }

    #[test]
    fn invalid_custom_profiles_and_unknown_keys_fail_closed() {
        let missing = parse_profile_table("[profile.custom]\nopt-level = 2\n").unwrap();
        let cycle = parse_profile_table("[profile.a]\ninherits = 'b'\n[profile.b]\ninherits = 'a'\n").unwrap();
        let unknown = parse_profile_table("[profile.dev]\nmystery = true\n").unwrap_err();

        let missing_error = resolve_profile(&missing, "custom").unwrap_err();
        let cycle_error = resolve_profile(&cycle, "a").unwrap_err();

        assert_eq!(missing_error.class, INVALID_PROFILE_CLASS);
        assert_eq!(cycle_error.class, INVALID_PROFILE_CLASS);
        assert_eq!(unknown.class, UNKNOWN_PROFILE_SETTING_CLASS);
        assert!(missing_error.message.contains("inherits"));
        assert!(cycle_error.message.contains("cycle"));
        assert!(unknown.message.contains("mystery"));
    }

    #[test]
    fn override_precedence_and_workspace_wildcard_exclusion_match_cargo() {
        let table = parse_profile_table(
            r#"
[profile.dev]
opt-level = 1
[profile.dev.build-override]
opt-level = 2
[profile.dev.package."*"]
opt-level = 3
[profile.dev.package.named]
opt-level = "s"
"#,
        )
        .unwrap();

        let named =
            select_unit_profile(&table, "dev", CargoProfileUnitRole::Host { dual_use: false }, "named", false).unwrap();
        let wildcard =
            select_unit_profile(&table, "dev", CargoProfileUnitRole::Host { dual_use: false }, "external", false)
                .unwrap();
        let workspace =
            select_unit_profile(&table, "dev", CargoProfileUnitRole::Host { dual_use: false }, "member", true).unwrap();
        let target = select_unit_profile(&table, "dev", CargoProfileUnitRole::Target, "member", true).unwrap();

        assert_eq!(named.opt_level, "s");
        assert_eq!(wildcard.opt_level, "3");
        assert_eq!(workspace.opt_level, "2");
        assert_eq!(target.opt_level, "1");
    }

    #[test]
    fn inherited_overrides_apply_before_child_overrides() {
        let table = parse_profile_table(
            r#"
[profile.parent]
inherits = "dev"
[profile.parent.build-override]
opt-level = 1
[profile.parent.package."*"]
codegen-units = 4
[profile.child]
inherits = "parent"
[profile.child.build-override]
opt-level = 2
[profile.child.package.named]
codegen-units = 3
"#,
        )
        .unwrap();

        let named =
            select_unit_profile(&table, "child", CargoProfileUnitRole::Host { dual_use: false }, "named", false)
                .unwrap();
        let wildcard =
            select_unit_profile(&table, "child", CargoProfileUnitRole::Host { dual_use: false }, "external", false)
                .unwrap();

        assert_eq!(named.opt_level, "2");
        assert_eq!(named.codegen_units, 3);
        assert_eq!(wildcard.opt_level, "2");
        assert_eq!(wildcard.codegen_units, 4);
    }

    #[test]
    fn profile_collection_limit_fails_closed() {
        let manifest = (0..=PROFILE_COUNT_MAX)
            .map(|index| format!("[profile.p{index}]\ninherits = 'dev'\n"))
            .collect::<String>();

        let error = parse_profile_table(&manifest).unwrap_err();

        assert_eq!(error.class, INVALID_PROFILE_CLASS);
        assert!(error.message.contains("profile count limit"));
    }

    #[test]
    fn invalid_setting_values_fail_closed() {
        for manifest in [
            "[profile.dev]\nopt-level = 4\n",
            "[profile.dev]\ndebug = 3\n",
            "[profile.dev]\ncodegen-units = 0\n",
            "[profile.dev]\npanic = 'unknown'\n",
            "[profile.dev]\nstrip = 'unknown'\n",
        ] {
            let error = parse_profile_table(manifest).unwrap_err();
            assert_eq!(error.class, INVALID_PROFILE_CLASS);
            assert!(error.message.contains("Cargo profile"));
        }
    }

    #[test]
    fn parsed_but_unlowered_profile_settings_fail_closed() {
        for manifest in [
            "[profile.release]\nlto = 'thin'\n",
            "[profile.release]\npanic = 'abort'\n",
            "[profile.release]\nrpath = true\n",
            "[profile.release]\nstrip = 'symbols'\n",
            "[profile.release]\nsplit-debuginfo = 'packed'\n",
        ] {
            let error = parse_profile_table(manifest).unwrap_err();
            assert_eq!(error.class, UNSUPPORTED_PROFILE_SETTING_CLASS);
            assert!(error.message.contains("not lowered"));
        }
    }

    #[test]
    fn forbidden_and_versioned_overrides_fail_closed() {
        for setting in ["panic = 'abort'", "lto = true", "rpath = true"] {
            let manifest = format!("[profile.dev.package.foo]\n{setting}\n");
            let error = parse_profile_table(&manifest).unwrap_err();
            assert_eq!(error.class, FORBIDDEN_OVERRIDE_CLASS);
            assert!(error.message.contains(setting.split_once(' ').unwrap().0));
        }

        let versioned = parse_profile_table("[profile.dev.package.'foo:2.1.0']\nopt-level = 1\n").unwrap_err();
        assert_eq!(versioned.class, UNSUPPORTED_OVERRIDE_SPEC_CLASS);
        assert!(versioned.message.contains("foo:2.1.0"));
    }

    #[test]
    fn command_selection_matches_cargo_defaults_and_release_equivalence() {
        assert_eq!(select_command_profile("build", false, None).unwrap(), "dev");
        assert_eq!(select_command_profile("test", false, None).unwrap(), "test");
        assert_eq!(select_command_profile("bench", false, None).unwrap(), "bench");
        assert_eq!(select_command_profile("install", false, None).unwrap(), "release");
        assert_eq!(select_command_profile("build", true, None).unwrap(), "release");
        assert_eq!(select_command_profile("build", false, Some("release")).unwrap(), "release");
        assert!(select_command_profile("build", true, Some("dev")).is_err());
    }

    #[test]
    fn dependency_profile_tables_are_ignored_with_a_non_claim() {
        let claims = dependency_profile_non_claims(&[
            "[package]\nname = 'dep'\nversion = '1.0.0'\n[profile.release]\nopt-level = 0\n",
        ]);
        let absent = dependency_profile_non_claims(&["[package]\nname = 'dep'\nversion = '1.0.0'\n"]);

        assert_eq!(claims, vec![DEPENDENCY_PROFILE_NON_CLAIM.to_string()]);
        assert!(absent.is_empty());
    }
}
