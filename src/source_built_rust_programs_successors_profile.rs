//! Translation of the selected run_rustc release profile and bootstrap's
//! original compiler-profile/default inheritance on the admitted musl host.
use super::RecipeWord;
use super::error;
use super::lit;
use super::resolve;
use crate::errors::RunError;
pub(super) struct Profile {
    pub arguments: Vec<RecipeWord>,
    pub incremental: bool,
    pub debug: bool,
}
pub(super) fn select(
    sources: &resolve::Sources,
    version: &str,
    group: &str,
    stage_zero: bool,
) -> Result<Profile, RunError> {
    if version == "1.90.0" {
        return Ok(Profile {
            arguments: vec![
                lit("-Cdebug-assertions=off"),
                lit("-Coverflow-checks=off"),
                lit("-Cdebuginfo=0"),
                lit("-Clink-args=-Wl,-rpath,$ORIGIN/../lib"),
            ],
            incremental: false,
            debug: false,
        });
    }
    let mut selected = sources
        .policy
        .iter()
        .filter(|(path, _)| path.starts_with("src/bootstrap/defaults/") && path.ends_with(".compiler.toml"));
    let (path, text) = selected.next().ok_or_else(|| error("authenticated archive lacks original compiler profile"))?;
    if selected.next().is_some() {
        return Err(error("ambiguous original compiler profile"));
    }
    let profile = text
        .parse::<toml::Value>()
        .map_err(|reason| error(format!("original compiler profile {path}: {reason}")))?;
    let rust = profile.get("rust").ok_or_else(|| error("original compiler profile lacks rust table"))?;
    let boolean = |key: &str, fallback: bool| -> Result<bool, RunError> {
        match rust.get(key) {
            Some(value) => value.as_bool().ok_or_else(|| error(format!("non-boolean original compiler option {key}"))),
            None => Ok(fallback),
        }
    };
    let standard = group.starts_with("std-");
    let compiler = group == "compiler";
    let debug_global = boolean("debug", false)?;
    let debug_assertions = boolean(
        if standard {
            "debug-assertions-std"
        } else if compiler {
            "debug-assertions"
        } else {
            "debug-assertions-tools"
        },
        boolean("debug-assertions", debug_global)?,
    )?;
    let overflow = boolean(
        if standard {
            "overflow-checks-std"
        } else {
            "overflow-checks"
        },
        boolean("overflow-checks", debug_global)?,
    )?;
    let key = if standard {
        "debuginfo-level-std"
    } else if compiler {
        "debuginfo-level-rustc"
    } else {
        "debuginfo-level-tools"
    };
    let debug = rust.get(key).or_else(|| rust.get("debuginfo-level"));
    let debug = match debug {
        Some(toml::Value::String(value)) => value.clone(),
        Some(toml::Value::Integer(value)) if (0..=2).contains(value) => value.to_string(),
        None => (if debug_global { "1" } else { "0" }).into(),
        _ => return Err(error("unclassified original debuginfo profile")),
    };
    let mut arguments = vec![
        lit(format!("-Cdebug-assertions={}", if debug_assertions { "on" } else { "off" })),
        lit(format!("-Coverflow-checks={}", if overflow { "on" } else { "off" })),
        lit(format!("-Cdebuginfo={debug}")),
    ];
    if boolean("frame-pointers", false)? {
        arguments.push(lit("-Cforce-frame-pointers=true"));
    }
    if boolean("rpath", true)? {
        arguments.extend([
            lit("-Clink-args=-Wl,-z,origin"),
            lit("-Clink-args=-Wl,-rpath,$ORIGIN/../lib"),
        ]);
    }
    if standard {
        arguments.push(lit("-Cprefer-dynamic"));
        arguments.push(lit("-Zinline-mir"));
    } else if compiler {
        let algorithm = sources
            .policy
            .values()
            .find(|text| text.contains("fn link_std_into_rustc_driver("))
            .ok_or_else(|| error("original bootstrap lacks std-into-driver decision"))?;
        if !algorithm.contains("!target.triple.ends_with(\"-windows-gnu\")") {
            return Err(error("unclassified original std-into-driver algorithm"));
        }
        // The authenticated musl host does not end in -windows-gnu: std is
        // linked into the driver, so bootstrap does not prefer dynamic here.
    }
    if let Some(units) = rust
        .get(if standard { "codegen-units-std" } else { "codegen-units" })
        .or_else(|| rust.get("codegen-units"))
    {
        let units = units
            .as_integer()
            .filter(|value| *value > 0)
            .ok_or_else(|| error("original codegen unit option is not a positive integer"))?;
        arguments.push(lit(format!("-Ccodegen-units={units}")));
    }
    if let Some(lto) = rust.get("lto") {
        match lto.as_str() {
            Some("off") => arguments.push(lit("-Clto=off")),
            _ => return Err(error("selected original compiler profile changed LTO policy")),
        }
    }
    Ok(Profile {
        arguments,
        incremental: stage_zero && boolean("incremental", false)?,
        debug: debug != "0" && debug != "none",
    })
}
