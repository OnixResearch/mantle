use serde::Deserialize;
use serde::Serialize;

const CARGO_PROFILE_DEV: &str = "dev";
const CARGO_PROFILE_RELEASE: &str = "release";
const CARGO_PROFILE_TEST: &str = "test";
const CARGO_PROFILE_BENCH: &str = "bench";
const CARGO_OPT_LEVEL_DEV: &str = "0";
const CARGO_OPT_LEVEL_RELEASE: &str = "3";
const CARGO_DEBUG_INFO_FULL: u8 = 2;
const CARGO_DEBUG_INFO_NONE: u8 = 0;
const CARGO_CODEGEN_UNITS_INCREMENTAL: u16 = 256;
const CARGO_CODEGEN_UNITS_NON_INCREMENTAL: u16 = 16;
const CARGO_LTO_FALSE: &str = "false";
const CARGO_PANIC_UNWIND: &str = "unwind";
const CARGO_STRIP_NONE: &str = "none";
const CARGO_SPLIT_DEBUG_INFO_PLATFORM_DEFAULT: &str = "platform-default";
const MANTLE_INCREMENTAL_DISABLED: bool = false;
#[cfg(test)]
const PROFILE_CODEGEN_SETTING_COUNT: usize = 5;
#[cfg(test)]
const RUSTC_CODEGEN_ARGUMENT_PAIR_WIDTH: usize = 2;
#[cfg(test)]
const PROFILE_CODEGEN_ARGUMENT_COUNT: usize = PROFILE_CODEGEN_SETTING_COUNT * RUSTC_CODEGEN_ARGUMENT_PAIR_WIDTH;
const UNKNOWN_PROFILE_CLASS: &str = "unknown-profile";
const UNSUPPORTED_PROFILE_SURFACE_CLASS: &str = "unsupported-profile-surface";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CargoProfileSettings {
    pub(crate) opt_level: String,
    pub(crate) debuginfo: u8,
    pub(crate) debug_assertions: bool,
    pub(crate) overflow_checks: bool,
    pub(crate) lto: String,
    pub(crate) panic: String,
    pub(crate) cargo_incremental: bool,
    pub(crate) codegen_units: u16,
    pub(crate) rpath: bool,
    pub(crate) strip: String,
    pub(crate) split_debuginfo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CargoProfilePolicy {
    pub(crate) selected_profile: String,
    pub(crate) settings: CargoProfileSettings,
    pub(crate) incremental: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CargoProfileError {
    pub(crate) class: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for CargoProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.class, self.message)
    }
}

fn dev_profile_settings() -> CargoProfileSettings {
    CargoProfileSettings {
        opt_level: CARGO_OPT_LEVEL_DEV.to_string(),
        debuginfo: CARGO_DEBUG_INFO_FULL,
        debug_assertions: true,
        overflow_checks: true,
        lto: CARGO_LTO_FALSE.to_string(),
        panic: CARGO_PANIC_UNWIND.to_string(),
        cargo_incremental: true,
        codegen_units: CARGO_CODEGEN_UNITS_INCREMENTAL,
        rpath: false,
        strip: CARGO_STRIP_NONE.to_string(),
        split_debuginfo: CARGO_SPLIT_DEBUG_INFO_PLATFORM_DEFAULT.to_string(),
    }
}

fn release_profile_settings() -> CargoProfileSettings {
    CargoProfileSettings {
        opt_level: CARGO_OPT_LEVEL_RELEASE.to_string(),
        debuginfo: CARGO_DEBUG_INFO_NONE,
        debug_assertions: false,
        overflow_checks: false,
        lto: CARGO_LTO_FALSE.to_string(),
        panic: CARGO_PANIC_UNWIND.to_string(),
        cargo_incremental: false,
        codegen_units: CARGO_CODEGEN_UNITS_NON_INCREMENTAL,
        rpath: false,
        strip: CARGO_STRIP_NONE.to_string(),
        split_debuginfo: CARGO_SPLIT_DEBUG_INFO_PLATFORM_DEFAULT.to_string(),
    }
}

pub(crate) fn resolve_builtin_profile(name: &str) -> Result<CargoProfileSettings, CargoProfileError> {
    match name {
        CARGO_PROFILE_DEV | CARGO_PROFILE_TEST => Ok(dev_profile_settings()),
        CARGO_PROFILE_RELEASE | CARGO_PROFILE_BENCH => Ok(release_profile_settings()),
        _ => Err(CargoProfileError {
            class: UNKNOWN_PROFILE_CLASS,
            message: format!("Cargo profile `{name}` is not a supported built-in profile"),
        }),
    }
}

pub(crate) fn profile_codegen_args(settings: &CargoProfileSettings) -> Vec<String> {
    vec![
        "-C".to_string(),
        format!("opt-level={}", settings.opt_level),
        "-C".to_string(),
        format!("debuginfo={}", settings.debuginfo),
        "-C".to_string(),
        format!("debug-assertions={}", settings.debug_assertions),
        "-C".to_string(),
        format!("overflow-checks={}", settings.overflow_checks),
        "-C".to_string(),
        format!("codegen-units={}", settings.codegen_units),
    ]
}

pub(crate) fn profile_metadata_material(name: &str, settings: &CargoProfileSettings) -> String {
    debug_assert!(!name.is_empty());
    debug_assert!(!settings.opt_level.is_empty());
    [
        name.to_string(),
        settings.opt_level.clone(),
        settings.debuginfo.to_string(),
        settings.debug_assertions.to_string(),
        settings.overflow_checks.to_string(),
        settings.codegen_units.to_string(),
        MANTLE_INCREMENTAL_DISABLED.to_string(),
    ]
    .join("\0")
}

pub(crate) fn validate_supported_profile_surface(settings: &CargoProfileSettings) -> Result<(), CargoProfileError> {
    let unsupported = [
        ("lto", settings.lto.as_str(), CARGO_LTO_FALSE),
        ("panic", settings.panic.as_str(), CARGO_PANIC_UNWIND),
        ("strip", settings.strip.as_str(), CARGO_STRIP_NONE),
        ("split-debuginfo", settings.split_debuginfo.as_str(), CARGO_SPLIT_DEBUG_INFO_PLATFORM_DEFAULT),
    ]
    .into_iter()
    .find(|(_, actual, expected)| actual != expected);
    if let Some((setting, actual, expected)) = unsupported {
        return Err(CargoProfileError {
            class: UNSUPPORTED_PROFILE_SURFACE_CLASS,
            message: format!("Cargo profile setting `{setting}={actual}` is unsupported; expected `{expected}`"),
        });
    }
    if settings.rpath {
        return Err(CargoProfileError {
            class: UNSUPPORTED_PROFILE_SURFACE_CLASS,
            message: "Cargo profile setting `rpath=true` is unsupported; expected `false`".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn resolve_profile_policy(name: &str) -> Result<CargoProfilePolicy, CargoProfileError> {
    let settings = resolve_builtin_profile(name)?;
    validate_supported_profile_surface(&settings)?;
    Ok(CargoProfilePolicy {
        selected_profile: name.to_string(),
        settings,
        incremental: MANTLE_INCREMENTAL_DISABLED,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_resolve_with_documented_inheritance() {
        let dev = resolve_builtin_profile(CARGO_PROFILE_DEV).unwrap();
        let release = resolve_builtin_profile(CARGO_PROFILE_RELEASE).unwrap();
        let test = resolve_builtin_profile(CARGO_PROFILE_TEST).unwrap();
        let bench = resolve_builtin_profile(CARGO_PROFILE_BENCH).unwrap();

        assert_eq!(test, dev);
        assert_eq!(bench, release);
        assert_ne!(dev.opt_level, release.opt_level);
        assert_ne!(dev.debuginfo, release.debuginfo);
        assert_ne!(dev.debug_assertions, release.debug_assertions);
        assert_ne!(dev.overflow_checks, release.overflow_checks);
        assert_ne!(dev.codegen_units, release.codegen_units);
    }

    #[test]
    fn empty_and_unknown_profile_names_fail_closed() {
        let empty = resolve_builtin_profile("").unwrap_err();
        let unknown = resolve_builtin_profile("production").unwrap_err();

        assert_eq!(empty.class, UNKNOWN_PROFILE_CLASS);
        assert_eq!(unknown.class, UNKNOWN_PROFILE_CLASS);
        assert!(empty.message.contains("``"));
        assert!(unknown.message.contains("production"));
    }

    #[test]
    fn codegen_args_are_stable_and_explicit() {
        let settings = resolve_builtin_profile(CARGO_PROFILE_RELEASE).unwrap();
        let first = profile_codegen_args(&settings);
        let second = profile_codegen_args(&settings);

        assert_eq!(first, second);
        assert_eq!(first.len(), PROFILE_CODEGEN_ARGUMENT_COUNT);
        assert!(first.contains(&format!("opt-level={CARGO_OPT_LEVEL_RELEASE}")));
        assert!(first.contains(&format!("debuginfo={CARGO_DEBUG_INFO_NONE}")));
        assert!(first.contains(&"debug-assertions=false".to_string()));
        assert!(first.contains(&"overflow-checks=false".to_string()));
        assert!(first.contains(&format!("codegen-units={CARGO_CODEGEN_UNITS_NON_INCREMENTAL}")));
    }

    #[test]
    fn selected_profile_name_separates_equal_inherited_settings() {
        let dev = resolve_builtin_profile(CARGO_PROFILE_DEV).unwrap();
        let test = resolve_builtin_profile(CARGO_PROFILE_TEST).unwrap();
        let dev_material = profile_metadata_material(CARGO_PROFILE_DEV, &dev);
        let test_material = profile_metadata_material(CARGO_PROFILE_TEST, &test);

        assert_eq!(dev, test);
        assert_ne!(dev_material, test_material);
    }

    #[test]
    fn unsupported_profile_surfaces_fail_with_deterministic_class() {
        let baseline = resolve_builtin_profile(CARGO_PROFILE_RELEASE).unwrap();
        let cases = [
            ("lto", CargoProfileSettings {
                lto: "thin".to_string(),
                ..baseline.clone()
            }),
            ("panic", CargoProfileSettings {
                panic: "abort".to_string(),
                ..baseline.clone()
            }),
            ("rpath", CargoProfileSettings {
                rpath: true,
                ..baseline.clone()
            }),
            ("strip", CargoProfileSettings {
                strip: "symbols".to_string(),
                ..baseline.clone()
            }),
            ("split-debuginfo", CargoProfileSettings {
                split_debuginfo: "packed".to_string(),
                ..baseline
            }),
        ];

        for (setting, settings) in cases {
            let error = validate_supported_profile_surface(&settings).unwrap_err();
            assert_eq!(error.class, UNSUPPORTED_PROFILE_SURFACE_CLASS);
            assert!(error.message.contains(setting));
        }
    }

    #[test]
    fn mantle_policy_disables_incremental_and_records_codegen_units() {
        let dev = resolve_profile_policy(CARGO_PROFILE_DEV).unwrap();
        let release = resolve_profile_policy(CARGO_PROFILE_RELEASE).unwrap();

        assert!(dev.settings.cargo_incremental);
        assert!(!dev.incremental);
        assert_eq!(dev.settings.codegen_units, CARGO_CODEGEN_UNITS_INCREMENTAL);
        assert_eq!(release.settings.codegen_units, CARGO_CODEGEN_UNITS_NON_INCREMENTAL);
    }
}
