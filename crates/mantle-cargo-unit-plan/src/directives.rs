//! Cargo build-script stdout is untrusted data until its directives are admitted.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blocker;

pub const MAX_DIRECTIVES: usize = 4096;
pub const MAX_DIRECTIVE_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildScriptFlags {
    pub schema: String,
    pub args_immediate: Vec<String>,
    pub args_transitive: Vec<String>,
    pub scoped_link_args: Vec<ScopedLinkArg>,
    pub env: BTreeMap<String, String>,
    pub links: BTreeMap<String, String>,
    pub rerun: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedLinkArg {
    pub kind: String,
    pub target: Option<String>,
    pub argument: String,
}

fn failure(code: &'static str, directive: &str, detail: &str) -> Blocker {
    Blocker {
        code,
        subject: directive.to_string(),
        detail: detail.to_string(),
    }
}

fn key_valid(key: &str) -> bool {
    !key.is_empty() && key.len() <= 128 && key.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn value_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_DIRECTIVE_BYTES && !value.chars().any(|ch| matches!(ch, '\0' | '\n' | '\r'))
}

fn add_option(flags: &mut BuildScriptFlags, name: &str, value: &str, transitive: bool) {
    flags.args_immediate.push(name.to_string());
    flags.args_immediate.push(value.to_string());
    if transitive {
        flags.args_transitive.push(name.to_string());
        flags.args_transitive.push(value.to_string());
    }
}

fn allowed_link_path(path: &str, roots: &[String]) -> bool {
    !path.split('/').any(|part| part == "." || part == "..")
        && roots
            .iter()
            .any(|root| path == root || path.strip_prefix(root).is_some_and(|rest| rest.starts_with('/')))
}

fn allowed_link_arg(value: &str, roots: &[String]) -> bool {
    value.split([',', '=', ':']).all(|part| !part.starts_with('/') || allowed_link_path(part, roots))
}

/// Parse exactly the supported Cargo directive vocabulary. Link paths must be
/// contained in a declared native root or the build script's output directory.
// r[impl mantle.rust_unit_plan.build_scripts.directive_propagation]
pub fn parse_directives(stdout: &str, allowed_roots: &[String]) -> Result<BuildScriptFlags, Blocker> {
    if stdout.len() > 4 * 1024 * 1024 || stdout.lines().count() > MAX_DIRECTIVES {
        return Err(failure("build-script-directive-unsupported", "stdout", "directive output limit"));
    }
    let mut flags = BuildScriptFlags {
        schema: "mantle-build-script-flags-v1".to_string(),
        ..BuildScriptFlags::default()
    };
    for line in stdout.lines() {
        let Some(payload) = line.strip_prefix("cargo::").or_else(|| line.strip_prefix("cargo:")) else {
            continue;
        };
        if !value_valid(payload) {
            return Err(failure("build-script-directive-unsupported", payload, "invalid directive value"));
        }
        let (key, value) = payload
            .split_once('=')
            .ok_or_else(|| failure("build-script-directive-unsupported", payload, "directive needs a value"))?;
        if !value_valid(value) {
            return Err(failure("build-script-directive-unsupported", key, "empty or invalid value"));
        }
        match key {
            "rustc-cfg" | "rustc-check-cfg" => {
                let option = if key == "rustc-cfg" { "--cfg" } else { "--check-cfg" };
                add_option(&mut flags, option, value, false);
            }
            "rustc-env" => {
                let (name, content) = value.split_once('=').ok_or_else(|| {
                    failure("build-script-directive-unsupported", key, "environment must be KEY=VALUE")
                })?;
                if !key_valid(name) || name.starts_with("MANTLE_") || name == "PATH" {
                    return Err(failure("build-script-directive-unsupported", key, "invalid environment key"));
                }
                flags.env.insert(name.to_string(), content.to_string());
            }
            "rustc-link-lib" | "rustc-link-search" => {
                if key == "rustc-link-search" {
                    let path = value.split_once('=').map_or(value, |(_, path)| path);
                    if !allowed_link_path(path, allowed_roots) {
                        return Err(failure(
                            "build-script-directive-unsupported",
                            key,
                            "link path outside declared inputs",
                        ));
                    }
                } else if value.contains('/') || value.starts_with('-') {
                    return Err(failure("build-script-directive-unsupported", key, "invalid library name"));
                }
                let option = if key == "rustc-link-lib" { "-l" } else { "-L" };
                add_option(&mut flags, option, value, true);
            }
            "rustc-link-arg" if allowed_link_arg(value, allowed_roots) => {
                add_option(&mut flags, "-C", &format!("link-arg={value}"), true);
            }
            "rustc-link-arg-bin"
            | "rustc-link-arg-bins"
            | "rustc-link-arg-tests"
            | "rustc-link-arg-examples"
            | "rustc-link-arg-benches"
            | "rustc-link-arg-cdylib" => {
                let (target, argument) = if key == "rustc-link-arg-bin" {
                    let (target, argument) = value
                        .split_once('=')
                        .ok_or_else(|| failure("build-script-directive-unsupported", key, "expected BIN=ARG"))?;
                    if !key_valid(target) {
                        return Err(failure("build-script-directive-unsupported", key, "invalid binary name"));
                    }
                    (Some(target.to_string()), argument)
                } else {
                    (None, value)
                };
                if !allowed_link_arg(argument, allowed_roots) {
                    return Err(failure(
                        "build-script-directive-unsupported",
                        key,
                        "link argument contains undeclared path",
                    ));
                }
                flags.scoped_link_args.push(ScopedLinkArg {
                    kind: key.to_string(),
                    target,
                    argument: argument.to_string(),
                });
            }
            "rustc-flags" => {
                let mut words = value.split_whitespace();
                while let Some(option) = words.next() {
                    let (kind, item) = if option == "-l" || option == "-L" {
                        (
                            option,
                            words.next().ok_or_else(|| {
                                failure("build-script-directive-unsupported", key, "missing flag value")
                            })?,
                        )
                    } else if let Some(item) = option.strip_prefix("-l") {
                        ("-l", item)
                    } else if let Some(item) = option.strip_prefix("-L") {
                        ("-L", item)
                    } else {
                        return Err(failure("build-script-directive-unsupported", key, "only -l and -L are supported"));
                    };
                    let path = item.split_once('=').map_or(item, |(_, path)| path);
                    if item.is_empty()
                        || (kind == "-L" && !allowed_link_path(path, allowed_roots))
                        || (kind == "-l" && (item.contains('/') || item.starts_with('-')))
                    {
                        return Err(failure("build-script-directive-unsupported", key, "invalid link flag"));
                    }
                    add_option(&mut flags, kind, item, true);
                }
            }
            "metadata" => {
                let (name, content) = value
                    .split_once('=')
                    .ok_or_else(|| failure("build-script-directive-unsupported", key, "expected KEY=VALUE"))?;
                if !key_valid(name) {
                    return Err(failure("build-script-directive-unsupported", key, "invalid metadata key"));
                }
                flags.links.insert(name.to_string(), content.to_string());
            }
            "warning" => flags.warnings.push(value.to_string()),
            "error" => return Err(failure("build-script-error-directive", key, value)),
            "rerun-if-changed" | "rerun-if-env-changed" => flags.rerun.push(payload.to_string()),
            _ if !key.starts_with("rustc-") && !key.starts_with("rerun-") && key_valid(key) => {
                flags.links.insert(key.to_string(), value.to_string());
            }
            _ => return Err(failure("build-script-directive-unsupported", key, "unknown directive")),
        }
    }
    Ok(flags)
}
