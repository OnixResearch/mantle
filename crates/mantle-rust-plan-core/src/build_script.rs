//! Cargo build-script directive grammar, propagation and native-input contract.
//!
//! The adapter captures script output and reads `flags` files; this module only
//! interprets bounded text. Supported directives are enumerated: anything else
//! fails closed so a new directive cannot change compilation without review.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

/// Maximum captured build-script stdout bytes admitted to the parser.
pub const MAX_BUILD_SCRIPT_OUTPUT_BYTES: usize = 8_388_608;
/// Maximum captured build-script stdout lines admitted to the parser.
pub const MAX_BUILD_SCRIPT_OUTPUT_LINES: usize = 65_536;
/// Maximum bytes of one directive value (matches the plan string bound).
pub const MAX_BUILD_SCRIPT_DIRECTIVE_BYTES: usize = 16_384;
/// Maximum `flags/links` files merged for one dependent build script.
pub const MAX_LINKS_DEPENDENCIES: usize = 256;
/// Maximum `rustc --print cfg` lines admitted as target facts.
pub const MAX_TARGET_FACT_LINES: usize = 1024;
/// Maximum native-input roles, environment entries, or PATH entries per package.
pub const MAX_NATIVE_INPUT_ENTRIES: usize = 64;
/// Maximum bytes of one native-input value template.
pub const MAX_NATIVE_INPUT_VALUE_BYTES: usize = 16_384;
/// Role marker opening; a marker is `{{role:<name>}}`.
pub const NATIVE_ROLE_MARKER_START: &str = "{{role:";
/// Typed target-facts file name inside the target-description unit output.
pub const TARGET_FACTS_FILE: &str = "target-facts.env";

const MARKER_END: &str = "}}";
const RESERVED_SCRIPT_ENV_KEYS: [&str; 13] = [
    "PATH",
    "OUT_DIR",
    "TARGET",
    "HOST",
    "RUSTC",
    "RUSTDOC",
    "RUSTC_LINKER",
    "NUM_JOBS",
    "OPT_LEVEL",
    "DEBUG",
    "PROFILE",
    "HOME",
    "TMPDIR",
];
const RESERVED_SCRIPT_ENV_PREFIXES: [&str; 3] = ["CARGO_", "DEP_", "MANTLE_"];

/// Which compilations of the script's own package receive an immediate argument.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArgScope {
    /// Every compilation of the package (`--cfg`, `--check-cfg`, `-l`, `-L`).
    All,
    /// `rustc-link-arg`: binaries, cdylibs, tests, examples and benches.
    Link,
    Bins,
    Bin(String),
    Cdylib,
    Tests,
    Examples,
    Benches,
}

/// One rustc argument for the package's own compilations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedArg {
    pub scope: ArgScope,
    pub arg: String,
}

/// Parsed directives in script output order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildScriptDirectives {
    pub immediate_args: Vec<ScopedArg>,
    /// `-L` search paths propagated to every dependent compilation.
    pub transitive_args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub metadata: Vec<(String, String)>,
    pub warnings: Vec<String>,
    /// Recorded `rerun-if-changed=`/`rerun-if-env-changed=` lines; never acted on.
    pub rerun: Vec<String>,
}

/// Stable typed failures of the directive contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildScriptDirectiveError {
    Unsupported {
        line: u32,
        directive: String,
        reason: String,
    },
    ErrorDirective {
        line: u32,
        message: String,
    },
    LinksConflict {
        key: String,
        detail: String,
    },
    OutputLimit {
        reason: String,
    },
    TargetFacts {
        reason: String,
    },
}

impl BuildScriptDirectiveError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported { .. } => "build-script-directive-unsupported",
            Self::ErrorDirective { .. } => "build-script-error-directive",
            Self::LinksConflict { .. } => "build-script-links-conflict",
            Self::OutputLimit { .. } => "build-script-output-limit",
            Self::TargetFacts { .. } => "build-script-target-facts-invalid",
        }
    }
}

impl fmt::Display for BuildScriptDirectiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Unsupported {
                line,
                directive,
                reason,
            } => {
                write!(formatter, "{code}: line {line}: `{directive}`: {reason}")
            }
            Self::ErrorDirective { line, message } => write!(formatter, "{code}: line {line}: {message}"),
            Self::LinksConflict { key, detail } => write!(formatter, "{code}: {key}: {detail}"),
            Self::OutputLimit { reason } | Self::TargetFacts { reason } => write!(formatter, "{code}: {reason}"),
        }
    }
}

#[allow(tigerstyle::ambiguous_params)] // directive name vs failure reason: distinct fields
fn unsupported(line: u32, directive: &str, reason: &str) -> BuildScriptDirectiveError {
    BuildScriptDirectiveError::Unsupported {
        line,
        directive: directive.to_string(),
        reason: reason.to_string(),
    }
}

fn output_limit(reason: &str) -> BuildScriptDirectiveError {
    BuildScriptDirectiveError::OutputLimit {
        reason: reason.to_string(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Syntax {
    /// `cargo::KEY=VALUE`; unknown keys fail.
    Current,
    /// `cargo:KEY=VALUE`; unknown non-reserved keys are `links` metadata.
    Legacy,
}

/// Parse captured build-script stdout. Lines without a `cargo:` prefix are
/// ignored, as Cargo does; every recognised line keeps its output order.
// r[impl mantle.rust_unit_plan.build_scripts.directive_propagation]
// r[impl mantle.rust_unit_plan.build_scripts.sandbox_boundary]
pub fn parse_build_script_output(stdout: &str) -> Result<BuildScriptDirectives, BuildScriptDirectiveError> {
    if stdout.len() > MAX_BUILD_SCRIPT_OUTPUT_BYTES {
        return Err(output_limit("build-script stdout exceeds 8 MiB"));
    }
    let mut directives = BuildScriptDirectives::default();
    for (index, line) in stdout.lines().enumerate() {
        if index >= MAX_BUILD_SCRIPT_OUTPUT_LINES {
            return Err(output_limit("build-script stdout exceeds 65536 lines"));
        }
        let Some((payload, syntax)) = directive_payload(line) else {
            continue;
        };
        let line_number = u32::try_from(index.saturating_add(1)).map_err(|_| output_limit("line number overflow"))?;
        apply_directive(&mut directives, payload, syntax, line_number)?;
    }
    debug_assert!(directives.transitive_args.len() <= directives.immediate_args.len());
    debug_assert!(directives.rerun.len() <= MAX_BUILD_SCRIPT_OUTPUT_LINES);
    Ok(directives)
}

fn directive_payload(line: &str) -> Option<(&str, Syntax)> {
    if let Some(payload) = line.strip_prefix("cargo::") {
        return Some((payload, Syntax::Current));
    }
    line.strip_prefix("cargo:").map(|payload| (payload, Syntax::Legacy))
}

fn apply_directive(
    directives: &mut BuildScriptDirectives,
    payload: &str,
    syntax: Syntax,
    line: u32,
) -> Result<(), BuildScriptDirectiveError> {
    let Some((key, raw_value)) = payload.split_once('=') else {
        return Err(unsupported(line, payload, "directive has no `=`"));
    };
    let value = raw_value.strip_suffix('\r').unwrap_or(raw_value);
    if value.len() > MAX_BUILD_SCRIPT_DIRECTIVE_BYTES {
        return Err(output_limit("directive value exceeds 16384 bytes"));
    }
    debug_assert!(line > 0);
    debug_assert!(!payload.is_empty());
    if apply_compile_directive(directives, key, value, line)? {
        return Ok(());
    }
    match key {
        "warning" => directives.warnings.push(value.to_string()),
        "error" if syntax == Syntax::Current => {
            return Err(BuildScriptDirectiveError::ErrorDirective {
                line,
                message: value.to_string(),
            });
        }
        "rerun-if-changed" | "rerun-if-env-changed" => directives.rerun.push(format!("{key}={value}")),
        "metadata" if syntax == Syntax::Current => {
            let Some((name, metadata)) = value.split_once('=') else {
                return Err(unsupported(line, key, "metadata must be KEY=VALUE"));
            };
            push_metadata(directives, name, metadata, line)?;
        }
        _ if syntax == Syntax::Legacy && !is_reserved_legacy_key(key) => push_metadata(directives, key, value, line)?,
        _ => return Err(unsupported(line, key, "directive is outside the supported set")),
    }
    Ok(())
}

/// Directives that change the package's compilation. Returns whether `key` matched.
#[allow(tigerstyle::ambiguous_params)] // directive key vs directive value: distinct grammar parts
fn apply_compile_directive(
    directives: &mut BuildScriptDirectives,
    key: &str,
    value: &str,
    line: u32,
) -> Result<bool, BuildScriptDirectiveError> {
    debug_assert!(!key.contains('='));
    debug_assert!(value.len() <= MAX_BUILD_SCRIPT_DIRECTIVE_BYTES);
    match key {
        "rustc-cfg" => push_scoped(directives, ArgScope::All, "--cfg=", require_value(line, key, value)?),
        "rustc-check-cfg" => push_scoped(directives, ArgScope::All, "--check-cfg=", require_value(line, key, value)?),
        "rustc-env" => {
            let Some((name, env_value)) = value.split_once('=') else {
                return Err(unsupported(line, key, "rustc-env must be KEY=VALUE"));
            };
            if !is_env_name(name) {
                return Err(unsupported(line, key, "rustc-env key must be an environment name"));
            }
            directives.env.push((name.to_string(), env_value.to_string()));
        }
        "rustc-link-lib" => push_link_lib(directives, value, line)?,
        "rustc-link-search" => push_link_search(directives, value, line)?,
        "rustc-flags" => push_rustc_flags(directives, value, line)?,
        _ => return apply_link_arg_directive(directives, key, value, line),
    }
    Ok(true)
}

#[allow(tigerstyle::ambiguous_params)] // directive key vs directive value: distinct grammar parts
fn apply_link_arg_directive(
    directives: &mut BuildScriptDirectives,
    key: &str,
    value: &str,
    line: u32,
) -> Result<bool, BuildScriptDirectiveError> {
    let scope = match key {
        "rustc-link-arg" => ArgScope::Link,
        "rustc-link-arg-bins" => ArgScope::Bins,
        "rustc-link-arg-cdylib" | "rustc-cdylib-link-arg" => ArgScope::Cdylib,
        "rustc-link-arg-tests" => ArgScope::Tests,
        "rustc-link-arg-examples" => ArgScope::Examples,
        "rustc-link-arg-benches" => ArgScope::Benches,
        "rustc-link-arg-bin" => {
            let Some((bin, flag)) = value.split_once('=') else {
                return Err(unsupported(line, key, "rustc-link-arg-bin must be BIN=FLAG"));
            };
            if !is_safe_token(bin) {
                return Err(unsupported(line, key, "binary name must be a safe token"));
            }
            push_scoped(directives, ArgScope::Bin(bin.to_string()), "-Clink-arg=", require_value(line, key, flag)?);
            return Ok(true);
        }
        _ => return Ok(false),
    };
    debug_assert!(key.starts_with("rustc-"));
    debug_assert!(scope != ArgScope::All);
    push_scoped(directives, scope, "-Clink-arg=", require_value(line, key, value)?);
    Ok(true)
}

#[allow(tigerstyle::ambiguous_params)] // directive key (diagnostic) vs directive value
fn require_value<'a>(line: u32, key: &str, value: &'a str) -> Result<&'a str, BuildScriptDirectiveError> {
    if value.is_empty() {
        return Err(unsupported(line, key, "directive value must not be empty"));
    }
    Ok(value)
}

#[allow(tigerstyle::ambiguous_params)] // rustc option prefix vs directive value: distinct domains
fn push_scoped(directives: &mut BuildScriptDirectives, scope: ArgScope, prefix: &str, value: &str) {
    debug_assert!(!prefix.is_empty());
    debug_assert!(!value.is_empty());
    directives.immediate_args.push(ScopedArg {
        scope,
        arg: format!("{prefix}{value}"),
    });
}

#[allow(tigerstyle::ambiguous_params)] // metadata key vs metadata value: distinct grammar parts
fn push_metadata(
    directives: &mut BuildScriptDirectives,
    key: &str,
    value: &str,
    line: u32,
) -> Result<(), BuildScriptDirectiveError> {
    let is_key_valid =
        !key.is_empty() && key.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    if !is_key_valid {
        return Err(unsupported(line, key, "metadata key must be ASCII alphanumeric, `_`, or `-`"));
    }
    debug_assert!(!key.contains('='));
    directives.metadata.push((key.to_string(), value.to_string()));
    Ok(())
}

fn is_reserved_legacy_key(key: &str) -> bool {
    key.starts_with("rustc-") || key.starts_with("rerun-")
}

fn push_link_lib(
    directives: &mut BuildScriptDirectives,
    value: &str,
    line: u32,
) -> Result<(), BuildScriptDirectiveError> {
    let directive = "rustc-link-lib";
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(unsupported(line, directive, "link library must be a non-empty token"));
    }
    let name = match value.split_once('=') {
        Some((prefix, name)) => {
            let (kind, modifiers) = prefix.split_once(':').map_or((prefix, None), |(kind, rest)| (kind, Some(rest)));
            if !matches!(kind, "static" | "dylib" | "framework") {
                return Err(unsupported(line, directive, "link kind must be static, dylib, or framework"));
            }
            if modifiers.is_some_and(|modifiers| !modifiers.split(',').all(is_link_modifier)) {
                return Err(unsupported(line, directive, "link modifiers must be +token or -token"));
            }
            name
        }
        None => value,
    };
    if !is_safe_token(name) {
        return Err(unsupported(line, directive, "link library name must be a safe token without renames"));
    }
    debug_assert!(!name.contains(':'));
    push_scoped(directives, ArgScope::All, "-l", value);
    Ok(())
}

fn push_link_search(
    directives: &mut BuildScriptDirectives,
    value: &str,
    line: u32,
) -> Result<(), BuildScriptDirectiveError> {
    let directive = "rustc-link-search";
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(unsupported(line, directive, "link search path must be a non-empty token"));
    }
    let path = match value.split_once('=') {
        Some(("dependency" | "crate" | "native" | "framework" | "all", path)) => path,
        Some((kind, _)) if !kind.contains('/') => {
            return Err(unsupported(line, directive, "unsupported link search kind"));
        }
        _ => value,
    };
    if path.is_empty() || path.split('/').any(|part| part == "..") {
        return Err(unsupported(line, directive, "link search path must be non-empty without parent traversal"));
    }
    debug_assert!(!path.is_empty());
    push_scoped(directives, ArgScope::All, "-L", value);
    directives.transitive_args.push(format!("-L{value}"));
    Ok(())
}

/// `rustc-flags` admits only `-l` and `-L`, attached or separated.
fn push_rustc_flags(
    directives: &mut BuildScriptDirectives,
    value: &str,
    line: u32,
) -> Result<(), BuildScriptDirectiveError> {
    let mut words = value.split_whitespace();
    let mut word_count = 0_usize;
    for _ in 0..MAX_BUILD_SCRIPT_DIRECTIVE_BYTES {
        let Some(word) = words.next() else {
            break;
        };
        word_count = word_count.saturating_add(1);
        let (flag, attached) = word.split_at(word.len().min(2));
        let operand = if attached.is_empty() {
            words.next().unwrap_or_default()
        } else {
            attached
        };
        match flag {
            "-l" => push_link_lib(directives, operand, line)?,
            "-L" => push_link_search(directives, operand, line)?,
            _ => return Err(unsupported(line, "rustc-flags", "only -l and -L flags are supported")),
        }
    }
    debug_assert!(word_count <= value.len());
    debug_assert!(line > 0);
    Ok(())
}

fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_-.+".contains(&byte))
}

fn is_link_modifier(value: &str) -> bool {
    let token = value.strip_prefix('+').or_else(|| value.strip_prefix('-'));
    token.is_some_and(|token| {
        !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

fn is_env_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    let is_first_valid = bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    is_first_valid && name.len() <= 255 && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn scope_token(scope: &ArgScope) -> String {
    match scope {
        ArgScope::All => "all".to_string(),
        ArgScope::Link => "link".to_string(),
        ArgScope::Bins => "bins".to_string(),
        ArgScope::Bin(name) => format!("bin:{name}"),
        ArgScope::Cdylib => "cdylib".to_string(),
        ArgScope::Tests => "tests".to_string(),
        ArgScope::Examples => "examples".to_string(),
        ArgScope::Benches => "benches".to_string(),
    }
}

fn parse_scope(token: &str) -> Option<ArgScope> {
    let scope = match token {
        "all" => ArgScope::All,
        "link" => ArgScope::Link,
        "bins" => ArgScope::Bins,
        "cdylib" => ArgScope::Cdylib,
        "tests" => ArgScope::Tests,
        "examples" => ArgScope::Examples,
        "benches" => ArgScope::Benches,
        _ => ArgScope::Bin(token.strip_prefix("bin:").filter(|name| is_safe_token(name))?.to_string()),
    };
    Some(scope)
}

/// Rendered `flags` output files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildScriptFlagsFiles {
    /// `flags/args-immediate`: `<scope>\t<rustc-arg>` lines.
    pub args_immediate: String,
    /// `flags/args-transitive`: one `-L<kind>=<path>` argument per line.
    pub args_transitive: String,
    /// `flags/env`: `KEY=VALUE` lines from `rustc-env`.
    pub env: String,
    /// `flags/links`: `DEP_<LINKS>_<KEY>=VALUE` lines; empty without `links`.
    pub links: String,
    /// `flags/rerun`: recorded `rerun-if-*` directives.
    pub rerun: String,
    /// `flags/warnings`: one warning per line.
    pub warnings: String,
}

/// Cargo's `DEP_<LINKS>_<KEY>` component: ASCII alphanumerics upper-cased,
/// every other character replaced by `_` (the `rust-plan` host rule).
pub fn links_env_component(value: &str) -> String {
    let component = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    debug_assert_eq!(component.chars().count(), value.chars().count());
    debug_assert!(component.bytes().all(|byte| !byte.is_ascii_lowercase()));
    component
}

/// Render the `flags` files. `links` is the package's `links` key; metadata
/// without one is not propagated, as in Cargo.
// r[impl mantle.rust_unit_plan.build_scripts.directive_propagation]
pub fn render_build_script_flags(directives: &BuildScriptDirectives, links: Option<&str>) -> BuildScriptFlagsFiles {
    let mut files = BuildScriptFlagsFiles::default();
    for scoped in &directives.immediate_args {
        files.args_immediate.push_str(&format!("{}\t{}\n", scope_token(&scoped.scope), scoped.arg));
    }
    for arg in &directives.transitive_args {
        files.args_transitive.push_str(&format!("{arg}\n"));
    }
    for (key, value) in &directives.env {
        files.env.push_str(&format!("{key}={value}\n"));
    }
    if let Some(links) = links {
        let prefix = format!("DEP_{}_", links_env_component(links));
        for (key, value) in &directives.metadata {
            files.links.push_str(&format!("{prefix}{}={value}\n", links_env_component(key)));
        }
    }
    for line in &directives.rerun {
        files.rerun.push_str(&format!("{line}\n"));
    }
    for warning in &directives.warnings {
        files.warnings.push_str(&format!("{warning}\n"));
    }
    debug_assert_eq!(files.args_immediate.lines().count(), directives.immediate_args.len());
    debug_assert_eq!(files.env.lines().count(), directives.env.len());
    files
}

/// The compilation that consumes `flags/args-immediate`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImmediateArgTarget<'a> {
    /// Cargo target kind: `lib`, `bin`, `proc-macro`, `test`, `example`, `bench`.
    pub kind: &'a str,
    pub name: &'a str,
    pub is_cdylib: bool,
}

fn scope_applies(scope: &ArgScope, target: &ImmediateArgTarget<'_>) -> bool {
    let is_linked = matches!(target.kind, "bin" | "test" | "example" | "bench") || target.is_cdylib;
    match scope {
        ArgScope::All => true,
        ArgScope::Link => is_linked,
        ArgScope::Bins => target.kind == "bin",
        ArgScope::Bin(name) => target.kind == "bin" && name == target.name,
        ArgScope::Cdylib => target.is_cdylib,
        ArgScope::Tests => target.kind == "test",
        ArgScope::Examples => target.kind == "example",
        ArgScope::Benches => target.kind == "bench",
    }
}

/// Select the immediate arguments of a package's own build script that apply
/// to one of its compilations, in script output order.
pub fn select_immediate_args(
    file: &str,
    target: &ImmediateArgTarget<'_>,
) -> Result<Vec<String>, BuildScriptDirectiveError> {
    let lines = bounded_lines(file)?;
    let mut selected = Vec::with_capacity(lines.len());
    for line in lines {
        let Some((token, arg)) = line.split_once('\t') else {
            return Err(output_limit("args-immediate line has no scope"));
        };
        let scope = parse_scope(token).ok_or_else(|| output_limit("args-immediate scope is unknown"))?;
        if arg.is_empty() {
            return Err(output_limit("args-immediate argument is empty"));
        }
        if scope_applies(&scope, target) {
            selected.push(arg.to_string());
        }
    }
    debug_assert!(selected.len() <= file.len());
    debug_assert!(selected.iter().all(|arg| !arg.is_empty()));
    Ok(selected)
}

/// Read `flags/args-transitive`: every line must be a `-L` argument.
pub fn parse_transitive_args(file: &str) -> Result<Vec<String>, BuildScriptDirectiveError> {
    let lines = bounded_lines(file)?;
    let mut args = Vec::with_capacity(lines.len());
    for line in lines {
        if !line.starts_with("-L") || line.len() <= 2 {
            return Err(output_limit("args-transitive admits only -L arguments"));
        }
        args.push(line.to_string());
    }
    debug_assert_eq!(args.len(), file.lines().count());
    debug_assert!(args.iter().all(|arg| arg.starts_with("-L")));
    Ok(args)
}

/// Read a `KEY=VALUE` flags file (`flags/env`, `flags/links`) in file order.
pub fn parse_flags_env(file: &str) -> Result<Vec<(String, String)>, BuildScriptDirectiveError> {
    let lines = bounded_lines(file)?;
    let mut entries = Vec::with_capacity(lines.len());
    for line in lines {
        let Some((key, value)) = line.split_once('=') else {
            return Err(output_limit("flags environment line is not KEY=VALUE"));
        };
        if !is_env_name(key) {
            return Err(output_limit("flags environment key is not an environment name"));
        }
        entries.push((key.to_string(), value.to_string()));
    }
    debug_assert_eq!(entries.len(), file.lines().count());
    debug_assert!(entries.iter().all(|(key, _)| !key.is_empty()));
    Ok(entries)
}

fn bounded_lines(file: &str) -> Result<Vec<&str>, BuildScriptDirectiveError> {
    if file.len() > MAX_BUILD_SCRIPT_OUTPUT_BYTES {
        return Err(output_limit("flags file exceeds 8 MiB"));
    }
    let lines = file.lines().take(MAX_BUILD_SCRIPT_OUTPUT_LINES.saturating_add(1)).collect::<Vec<_>>();
    if lines.len() > MAX_BUILD_SCRIPT_OUTPUT_LINES {
        return Err(output_limit("flags file exceeds 65536 lines"));
    }
    debug_assert!(lines.len() <= MAX_BUILD_SCRIPT_OUTPUT_LINES);
    debug_assert!(lines.iter().all(|line| !line.contains('\n')));
    Ok(lines)
}

/// Merge the `flags/links` files of a script's direct `links` dependencies into
/// `DEP_*` environment entries. Within one file the last value wins; the same
/// key from two dependencies is a `links` conflict.
#[allow(tigerstyle::unbounded_collection_growth)] // BTree maps cannot reserve; MAX_LINKS_DEPENDENCIES bounds the files.
pub fn merge_links_env(files: &[&str]) -> Result<BTreeMap<String, String>, BuildScriptDirectiveError> {
    if files.len() > MAX_LINKS_DEPENDENCIES {
        return Err(output_limit("links dependencies exceed 256"));
    }
    let mut merged = BTreeMap::new();
    for (index, file) in files.iter().enumerate() {
        let mut own = BTreeMap::new();
        for (key, value) in parse_flags_env(file)? {
            if !key.starts_with("DEP_") {
                return Err(output_limit("links file admits only DEP_* keys"));
            }
            own.insert(key, value);
        }
        for (key, value) in own {
            if merged.insert(key.clone(), value).is_some() {
                return Err(BuildScriptDirectiveError::LinksConflict {
                    key,
                    detail: format!("defined again by links dependency {index}"),
                });
            }
        }
    }
    debug_assert!(merged.keys().all(|key| key.starts_with("DEP_")));
    debug_assert!(files.len() <= MAX_LINKS_DEPENDENCIES);
    Ok(merged)
}

/// Two packages in one plan may not declare the same `links` key. Entries are
/// `(package_id, links)`; one package listed twice (host and target) is allowed.
#[allow(tigerstyle::unbounded_collection_growth)] // BTree maps cannot reserve; one entry per lowered build script.
pub fn check_links_unique(entries: &[(&str, &str)]) -> Result<(), BuildScriptDirectiveError> {
    let mut owners = BTreeMap::<&str, &str>::new();
    for &(package, links) in entries {
        if let Some(owner) = owners.insert(links, package)
            && owner != package
        {
            return Err(BuildScriptDirectiveError::LinksConflict {
                key: links.to_string(),
                detail: format!("declared by {owner} and {package}"),
            });
        }
    }
    debug_assert!(owners.len() <= entries.len());
    debug_assert!(owners.values().all(|package| !package.is_empty()));
    Ok(())
}

/// Every `-L` path must resolve inside an admitted root (the unit's `OUT_DIR`,
/// its source slice, or a declared native input) so dependents never receive
/// host search paths.
pub fn check_link_search_roots(
    directives: &BuildScriptDirectives,
    roots: &[&str],
) -> Result<(), BuildScriptDirectiveError> {
    for arg in &directives.transitive_args {
        let value = arg.strip_prefix("-L").unwrap_or(arg);
        let path = value.split_once('=').filter(|(kind, _)| !kind.contains('/')).map_or(value, |(_, path)| path);
        let is_admitted = roots.iter().any(|root| {
            !root.is_empty() && path.strip_prefix(root).is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        });
        if !is_admitted {
            return Err(unsupported(
                0,
                "rustc-link-search",
                "search path is outside OUT_DIR, source, and native inputs",
            ));
        }
    }
    debug_assert!(directives.transitive_args.iter().all(|arg| arg.starts_with("-L")));
    debug_assert!(directives.transitive_args.is_empty() || !roots.is_empty());
    Ok(())
}

/// Render the typed target-facts file from `rustc --print cfg --target <triple>`
/// output: `TARGET`, `HOST`, then sorted `CARGO_CFG_*` entries whose values
/// join repeated cfg names with `,` in rustc order. `debug_assertions` is a
/// profile fact and `feature` a package fact, so neither is a target fact.
// r[impl mantle.rust_unit_plan.build_scripts.target_facts]
#[allow(tigerstyle::unbounded_collection_growth)] // BTree maps cannot reserve; MAX_TARGET_FACT_LINES bounds entries.
#[allow(tigerstyle::ambiguous_params)] // rustc cfg output vs target triple vs host triple: distinct domains
pub fn render_target_facts(cfg_output: &str, target: &str, host: &str) -> Result<String, BuildScriptDirectiveError> {
    if !is_safe_token(target) || !is_safe_token(host) {
        return Err(target_facts_error("target and host must be triple tokens"));
    }
    let mut cfg = BTreeMap::<String, Vec<String>>::new();
    for (index, line) in cfg_output.lines().enumerate() {
        if index >= MAX_TARGET_FACT_LINES {
            return Err(target_facts_error("rustc cfg output exceeds 1024 lines"));
        }
        let (name, value) = match line.split_once('=') {
            Some((name, quoted)) => {
                let value = quoted.strip_prefix('"').and_then(|rest| rest.strip_suffix('"'));
                (name, Some(value.ok_or_else(|| target_facts_error("cfg value is not quoted"))?))
            }
            None => (line, None),
        };
        if !is_env_name(name) || value.is_some_and(|value| value.contains(['"', '\\'])) {
            return Err(target_facts_error("cfg line is not name or name=\"value\""));
        }
        if name == "debug_assertions" || name == "feature" {
            continue;
        }
        let values = cfg.entry(format!("CARGO_CFG_{}", name.to_ascii_uppercase())).or_default();
        values.extend(value.map(ToString::to_string));
    }
    let mut rendered = format!("TARGET={target}\nHOST={host}\n");
    for (key, values) in &cfg {
        rendered.push_str(&format!("{key}={}\n", values.join(",")));
    }
    debug_assert!(rendered.starts_with("TARGET="));
    debug_assert_eq!(rendered.lines().count(), cfg.len().saturating_add(2));
    Ok(rendered)
}

/// Read a target-facts file; only `TARGET`, `HOST`, and `CARGO_CFG_*` keys.
pub fn parse_target_facts(file: &str) -> Result<Vec<(String, String)>, BuildScriptDirectiveError> {
    let entries = parse_flags_env(file).map_err(|_| target_facts_error("target facts are not KEY=VALUE lines"))?;
    let has_target = entries.iter().any(|(key, _)| key == "TARGET");
    let has_host = entries.iter().any(|(key, _)| key == "HOST");
    if !has_target || !has_host {
        return Err(target_facts_error("target facts lack TARGET or HOST"));
    }
    if entries.iter().any(|(key, _)| key != "TARGET" && key != "HOST" && !key.starts_with("CARGO_CFG_")) {
        return Err(target_facts_error("target facts admit only TARGET, HOST, and CARGO_CFG_*"));
    }
    debug_assert!(entries.len() >= 2);
    debug_assert!(entries.len() <= MAX_BUILD_SCRIPT_OUTPUT_LINES);
    Ok(entries)
}

fn target_facts_error(reason: &str) -> BuildScriptDirectiveError {
    BuildScriptDirectiveError::TargetFacts {
        reason: reason.to_string(),
    }
}

/// Typed native-input declaration for one Cargo package: role-named store
/// paths, plus environment and PATH values that refer to roles only through
/// `{{role:<name>}}` markers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeInputDeclaration {
    #[serde(default = "no_native_values")]
    pub inputs: BTreeMap<String, String>,
    #[serde(default = "no_native_values")]
    pub env: BTreeMap<String, String>,
    #[serde(default = "no_native_path")]
    pub path: Vec<String>,
}

fn no_native_values() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn no_native_path() -> Vec<String> {
    Vec::new()
}

/// A declaration with every role marker replaced by its store path.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedNativeInputs {
    /// Declared role store paths, sorted and unique.
    pub inputs: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub path: Vec<String>,
}

/// Stable typed failures of the native-input contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeInputError {
    RoleUndeclared { role: String },
    HostPath { value: String },
    ReservedEnv { key: String },
    Invalid { reason: String },
}

impl NativeInputError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::RoleUndeclared { .. } => "native-input-role-undeclared",
            Self::HostPath { .. } => "native-input-host-path",
            Self::ReservedEnv { .. } => "native-input-env-reserved",
            Self::Invalid { .. } => "native-input-invalid",
        }
    }
}

impl fmt::Display for NativeInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::RoleUndeclared { role } => write!(formatter, "{code}: {role}"),
            Self::HostPath { value } => write!(formatter, "{code}: {value}"),
            Self::ReservedEnv { key } => write!(formatter, "{code}: {key}"),
            Self::Invalid { reason } => write!(formatter, "{code}: {reason}"),
        }
    }
}

fn native_invalid(reason: &str) -> NativeInputError {
    NativeInputError::Invalid {
        reason: reason.to_string(),
    }
}

/// Resolve every marker of one declaration. Store-prefix admission of the role
/// paths themselves belongs to the adapter that knows the store.
// r[impl mantle.rust_unit_plan.build_scripts.native_inputs]
#[allow(tigerstyle::unbounded_collection_growth)] // BTree collections cannot reserve; MAX_NATIVE_INPUT_ENTRIES was checked.
pub fn resolve_native_inputs(declaration: &NativeInputDeclaration) -> Result<ResolvedNativeInputs, NativeInputError> {
    let counts = [declaration.inputs.len(), declaration.env.len(), declaration.path.len()];
    if counts.iter().any(|count| *count > MAX_NATIVE_INPUT_ENTRIES) {
        return Err(native_invalid("native inputs, env, or path exceed 64 entries"));
    }
    for (role, path) in &declaration.inputs {
        let is_role_valid = !role.is_empty()
            && role.len() <= 64
            && role.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte));
        if !is_role_valid {
            return Err(native_invalid("role names are lowercase ASCII, digits, `_`, or `-`"));
        }
        if !path.starts_with('/') || path.split('/').any(|part| part == "..") {
            return Err(NativeInputError::HostPath { value: path.clone() });
        }
    }
    let mut env = BTreeMap::new();
    for (key, template) in &declaration.env {
        check_native_env_key(key)?;
        env.insert(key.clone(), resolve_native_value(template, &declaration.inputs)?);
    }
    let mut path = Vec::with_capacity(declaration.path.len());
    for template in &declaration.path {
        let resolved = resolve_native_value(template, &declaration.inputs)?;
        if !template.starts_with(NATIVE_ROLE_MARKER_START) || resolved.contains(':') {
            return Err(NativeInputError::HostPath {
                value: template.clone(),
            });
        }
        path.push(resolved);
    }
    let inputs = declaration.inputs.values().cloned().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    debug_assert_eq!(env.len(), declaration.env.len());
    debug_assert!(inputs.len() <= declaration.inputs.len());
    Ok(ResolvedNativeInputs { inputs, env, path })
}

fn check_native_env_key(key: &str) -> Result<(), NativeInputError> {
    if !is_env_name(key) {
        return Err(native_invalid("environment keys must be environment names"));
    }
    let is_reserved = RESERVED_SCRIPT_ENV_KEYS.contains(&key)
        || RESERVED_SCRIPT_ENV_PREFIXES.iter().any(|prefix| key.starts_with(prefix));
    if is_reserved {
        return Err(NativeInputError::ReservedEnv { key: key.to_string() });
    }
    debug_assert!(!key.is_empty());
    debug_assert!(!key.contains('='));
    Ok(())
}

/// Whether a key is owned by the lane rather than by a native declaration.
pub fn is_reserved_build_script_env(key: &str) -> bool {
    check_native_env_key(key).is_err()
}

/// Replace `{{role:<name>}}` markers. A `/` may appear only in the path suffix
/// that directly follows a marker, so a template cannot name a host path.
pub fn resolve_native_value(template: &str, roles: &BTreeMap<String, String>) -> Result<String, NativeInputError> {
    if template.len() > MAX_NATIVE_INPUT_VALUE_BYTES || template.contains('\0') {
        return Err(native_invalid("native value exceeds 16384 bytes or contains NUL"));
    }
    let mut resolved = String::with_capacity(template.len());
    let mut rest = template;
    let mut is_role_suffix = false;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix(NATIVE_ROLE_MARKER_START) {
            let (role, tail) =
                after.split_once(MARKER_END).ok_or_else(|| native_invalid("unterminated role marker"))?;
            let path = roles.get(role).ok_or_else(|| NativeInputError::RoleUndeclared { role: role.to_string() })?;
            resolved.push_str(path);
            rest = tail;
            is_role_suffix = true;
            continue;
        }
        let ch = rest.chars().next().unwrap_or_default();
        let is_path_char = ch.is_ascii_alphanumeric() || "._+-/".contains(ch);
        is_role_suffix &= is_path_char;
        if ch == '/' && !is_role_suffix {
            return Err(NativeInputError::HostPath {
                value: template.to_string(),
            });
        }
        resolved.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    if resolved.split('/').any(|segment| segment == "..") {
        return Err(NativeInputError::HostPath {
            value: template.to_string(),
        });
    }
    debug_assert!(rest.is_empty());
    debug_assert!(!template.is_empty() || resolved.is_empty());
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test]
    fn link_search_without_kind_keeps_absolute_path() {
        let parsed = parse_build_script_output("cargo:rustc-link-search=/out/lib\n").unwrap();
        assert_eq!(parsed.transitive_args, vec!["-L/out/lib".to_string()]);
        assert!(check_link_search_roots(&parsed, &["/out"]).is_ok());
        assert!(check_link_search_roots(&parsed, &["/ou"]).is_err());
    }
}
