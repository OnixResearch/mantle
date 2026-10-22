//! Bounded, effect-free C/C++ driver classification and content identity.
//!
//! Roots are caller-attested identities, not permission to erase arbitrary paths.
//! The shell must establish path-independent compiler bytes (including diagnostics)
//! before setting the corresponding attestations, and must hash complete tool/input
//! roots to cover newly introduced headers and header search-order changes.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::iter::Peekable;
use core::str::Chars;

use serde::Deserialize;
use serde::Serialize;

pub const RECEIPT_SCHEMA: &str = "mantle-cc-cache-receipt-v1";
pub const MAX_ARGUMENTS: usize = 4_096;
pub const MAX_STRING_BYTES: usize = 16_384;
pub const MAX_ARGUMENT_BYTES: usize = 1_048_576;
pub const MAX_DEPFILE_BYTES: usize = 4_194_304;
pub const MAX_DEPENDENCIES: usize = 8_192;
pub const MAX_ROOTS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Bypass {
    BoundsExceeded,
    UnsupportedInvocation,
    UnknownOption,
    SideEffects,
    InvalidDepfile,
    IncompleteManifest,
    InvalidIdentity,
    UnclassifiedPath,
    PathDependentOutput,
    VolatileInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyMode {
    Md,
    Mmd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum SemanticArgument {
    Literal(String),
    Path { option: String, path: String },
    PrefixMap { option: String, from: String, to: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub source: String,
    pub output: String,
    pub depfile: Option<String>,
    pub dependency_mode: Option<DependencyMode>,
    pub phony_targets: bool,
    pub semantic_args: Vec<SemanticArgument>,
}

// r[impl mantle.cc_compile_cache.driver_boundary]
/// Classify arguments *without* argv[0]. Errors mean forward the original argv.
/// This allowlist excludes plugins, modules, coverage, profile output, assembly,
/// response files, debug paths, save-temps, custom dep targets, and link actions.
pub fn classify(args: &[String]) -> Result<Invocation, Bypass> {
    validate_arguments(args)?;
    let mut invocation = Invocation {
        source: String::new(),
        output: String::new(),
        depfile: None,
        dependency_mode: None,
        phony_targets: false,
        semantic_args: Vec::new(),
    };
    let mut is_compile = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-c" if !is_compile => is_compile = true,
            "-o" if invocation.output.is_empty() => {
                invocation.output = take_value(args, &mut index)?.to_string();
            }
            "-MF" if invocation.depfile.is_none() => {
                invocation.depfile = Some(take_value(args, &mut index)?.to_string());
            }
            "-MD" | "-MMD" if invocation.dependency_mode.is_none() => {
                invocation.dependency_mode = Some(if arg == "-MD" {
                    DependencyMode::Md
                } else {
                    DependencyMode::Mmd
                });
            }
            "-MP" if !invocation.phony_targets => invocation.phony_targets = true,
            "-x" if invocation.source.is_empty() => {
                let language = take_value(args, &mut index)?;
                if !matches!(language, "c" | "c++") {
                    return Err(Bypass::UnsupportedInvocation);
                }
                invocation.semantic_args.push(SemanticArgument::Literal(arg.clone()));
                invocation.semantic_args.push(SemanticArgument::Literal(language.to_string()));
            }
            _ if arg.starts_with('-') => {
                invocation.semantic_args.push(classify_option(args, &mut index)?);
            }
            _ if invocation.source.is_empty() && source_path(arg) => {
                invocation.source = arg.clone();
            }
            _ => return Err(Bypass::UnsupportedInvocation),
        }
        index += 1;
    }
    debug_assert!(invocation.source.is_empty() || source_path(&invocation.source));
    debug_assert!(invocation.semantic_args.len() <= args.len());
    finish_invocation(invocation, is_compile)
}

fn validate_arguments(args: &[String]) -> Result<(), Bypass> {
    if args.is_empty() || args.len() > MAX_ARGUMENTS {
        return Err(Bypass::BoundsExceeded);
    }
    let mut bytes = 0_usize;
    for argument in args {
        if !bounded_text(argument) || argument.starts_with('@') {
            return Err(Bypass::UnsupportedInvocation);
        }
        bytes = bytes.checked_add(argument.len()).ok_or(Bypass::BoundsExceeded)?;
        if bytes > MAX_ARGUMENT_BYTES {
            return Err(Bypass::BoundsExceeded);
        }
    }
    Ok(())
}

fn take_value<'a>(args: &'a [String], index: &mut usize) -> Result<&'a str, Bypass> {
    *index += 1;
    let value = args.get(*index).ok_or(Bypass::UnsupportedInvocation)?;
    if value.starts_with('-') || value.starts_with('@') {
        return Err(Bypass::UnsupportedInvocation);
    }
    Ok(value)
}

fn source_path(path: &str) -> bool {
    [".c", ".cc", ".cpp", ".cxx", ".C"].iter().any(|suffix| path.ends_with(suffix))
}

fn finish_invocation(mut value: Invocation, is_compile: bool) -> Result<Invocation, Bypass> {
    if !is_compile || value.source.is_empty() || value.output.is_empty() {
        return Err(Bypass::UnsupportedInvocation);
    }
    if value.output == "-" || value.output == value.source {
        return Err(Bypass::SideEffects);
    }
    if value.dependency_mode.is_none() && (value.depfile.is_some() || value.phony_targets) {
        return Err(Bypass::UnsupportedInvocation);
    }
    if value.dependency_mode.is_some() && value.depfile.is_none() {
        let basename = value.output.rsplit('/').next().unwrap_or(&value.output);
        let stem_bytes = basename.rfind('.').map_or(value.output.len(), |offset| {
            value.output.len().saturating_sub(basename.len()).saturating_add(offset)
        });
        value.depfile = Some(alloc::format!("{}.d", &value.output[..stem_bytes]));
    }
    if let Some(depfile) = &value.depfile
        && (depfile == "-" || depfile == &value.output || depfile == &value.source)
    {
        return Err(Bypass::SideEffects);
    }
    debug_assert!(!value.source.is_empty());
    debug_assert!(value.depfile.is_some() == value.dependency_mode.is_some());
    Ok(value)
}

fn classify_option(args: &[String], index: &mut usize) -> Result<SemanticArgument, Bypass> {
    let arg = &args[*index];
    for option in [
        "-I",
        "-isystem",
        "-iquote",
        "-idirafter",
        "-include",
        "-imacros",
        "--sysroot",
        "-isysroot",
    ] {
        if arg == option {
            let path = take_value(args, index)?.to_string();
            debug_assert!(bounded_text(&path));
            return Ok(SemanticArgument::Path {
                option: option.to_string(),
                path,
            });
        }
    }
    if let Some(path) = arg.strip_prefix("-I").filter(|value| !value.is_empty()) {
        debug_assert!(!path.is_empty());
        if path == "-" {
            return Err(Bypass::UnknownOption);
        }
        return Ok(SemanticArgument::Path {
            option: "-I".to_string(),
            path: path.to_string(),
        });
    }
    if let Some(path) = arg.strip_prefix("--sysroot=") {
        if path.is_empty() {
            return Err(Bypass::UnknownOption);
        }
        return Ok(SemanticArgument::Path {
            option: "--sysroot".to_string(),
            path: path.to_string(),
        });
    }
    if let Some(argument) = prefix_map_option(arg)? {
        return Ok(argument);
    }
    if arg == "-D" || arg == "-U" {
        let value = take_value(args, index)?;
        if !safe_macro(value) {
            return Err(Bypass::UnknownOption);
        }
        debug_assert!(!value.is_empty());
        return Ok(SemanticArgument::Literal(alloc::format!("{arg}{value}")));
    }
    if (arg.starts_with("-D") || arg.starts_with("-U")) && safe_macro(&arg[2..]) {
        return Ok(SemanticArgument::Literal(arg.clone()));
    }
    if literal_option(arg) {
        return Ok(SemanticArgument::Literal(arg.clone()));
    }
    Err(Bypass::UnknownOption)
}

/// Prefix-map options are order sensitive, so only the first matching option
/// form is classified; the mapped target must stay a relative, `..`-free path.
fn prefix_map_option(arg: &str) -> Result<Option<SemanticArgument>, Bypass> {
    for option in ["-ffile-prefix-map", "-fmacro-prefix-map", "-fdebug-prefix-map"] {
        let Some(mapping) = arg.strip_prefix(option).and_then(|text| text.strip_prefix('=')) else {
            continue;
        };
        let Some((from, to)) = mapping.split_once('=') else {
            return Err(Bypass::UnknownOption);
        };
        if from.is_empty() || !is_classifiable_prefix_target(to) {
            return Err(Bypass::UnclassifiedPath);
        }
        return Ok(Some(SemanticArgument::PrefixMap {
            option: option.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        }));
    }
    Ok(None)
}

fn is_classifiable_prefix_target(to: &str) -> bool {
    if to.contains('=') {
        return false;
    }
    if to.starts_with('/') {
        return false;
    }
    if to.contains('\\') {
        return false;
    }
    if to.split('/').any(|part| part == "..") {
        return false;
    }
    true
}

/// Content-only replay cannot represent clocks or file mtimes. Be conservative:
/// token pasting and alternate spellings can synthesize a volatile builtin, so
/// they bypass too. This scans bytes, including inactive code and comments;
/// false positives compile normally, never weakening replay admission.
pub fn volatile_input(bytes: &[u8]) -> bool {
    [
        b"__TIME__".as_slice(),
        b"__DATE__",
        b"__TIMESTAMP__",
        b"##",
        b"%:%:",
        b"??",
        b"\\\n",
        b"\\\r",
        b"\\u",
        b"\\U",
    ]
    .iter()
    .any(|needle| bytes.windows(needle.len()).any(|part| part == *needle))
}

fn safe_macro(value: &str) -> bool {
    !value.is_empty() && !value.contains(['/', '\\', '@']) && !volatile_input(value.as_bytes())
}

fn literal_option(value: &str) -> bool {
    literal_codegen_option(value)
        || literal_diagnostic_option(value)
        || literal_runtime_option(value)
        || literal_target_option(value)
        || literal_standard_option(value)
}

fn literal_codegen_option(value: &str) -> bool {
    matches!(
        value,
        "-O0"
            | "-O1"
            | "-O2"
            | "-O3"
            | "-Os"
            | "-Oz"
            | "-Og"
            | "-Ofast"
            | "-g0"
            | "-pipe"
            | "-pthread"
            | "-fPIC"
            | "-fpic"
            | "-fPIE"
            | "-fpie"
            | "-fno-pic"
            | "-fno-pie"
            | "-fno-strict-aliasing"
            | "-fstrict-aliasing"
            | "-fno-omit-frame-pointer"
            | "-fomit-frame-pointer"
            | "-ffunction-sections"
            | "-fdata-sections"
    )
}

fn literal_diagnostic_option(value: &str) -> bool {
    matches!(
        value,
        "-pedantic"
            | "-pedantic-errors"
            | "-Wall"
            | "-Wextra"
            | "-Werror"
            | "-Wno-error"
            | "-Wno-unused-parameter"
            | "-Wno-unused-function"
            | "-Wno-unused-variable"
            | "-Wno-sign-compare"
    )
}

fn literal_runtime_option(value: &str) -> bool {
    matches!(
        value,
        "-fno-builtin"
            | "-ffreestanding"
            | "-fno-common"
            | "-fcommon"
            | "-fno-exceptions"
            | "-fexceptions"
            | "-fno-rtti"
            | "-frtti"
            | "-funsigned-char"
            | "-fsigned-char"
    )
}

fn literal_target_option(value: &str) -> bool {
    matches!(value, "-m32" | "-m64" | "-nostdinc" | "-nostdinc++")
}

fn literal_standard_option(value: &str) -> bool {
    matches!(
        value,
        "-std=c89"
            | "-std=c90"
            | "-std=c99"
            | "-std=c11"
            | "-std=c17"
            | "-std=c23"
            | "-std=gnu89"
            | "-std=gnu90"
            | "-std=gnu99"
            | "-std=gnu11"
            | "-std=gnu17"
            | "-std=gnu23"
            | "-std=c++98"
            | "-std=c++11"
            | "-std=c++14"
            | "-std=c++17"
            | "-std=c++20"
            | "-std=c++23"
            | "-std=gnu++98"
            | "-std=gnu++11"
            | "-std=gnu++14"
            | "-std=gnu++17"
            | "-std=gnu++20"
            | "-std=gnu++23"
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityRoot {
    pub path: String,
    pub digest_blake3: String,
    /// Explicit caller attestation, including __FILE__, diagnostics, and debug data.
    pub path_independent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyIdentity {
    /// Logical alias-relative name; never a physical/store path.
    pub name: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyManifest {
    /// Includes source, implicit headers, and negative/header-search dependencies
    /// through complete root identities. A depfile alone cannot attest this.
    pub complete: bool,
    pub dependencies: Vec<DependencyIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyInputs {
    pub compiler_blake3: String,
    pub source_blake3: String,
    pub platform_blake3: String,
    pub environment_blake3: String,
    pub toolchain_blake3: String,
    pub probe_blake3: Option<String>,
    pub roots: Vec<IdentityRoot>,
    pub manifest: DependencyManifest,
    /// Object/stream bytes must not depend on -o; depfiles must be re-rendered
    /// for the current destination by the shell, not replayed verbatim.
    pub output_path_independent: bool,
}

fn bounded_text(text: &str) -> bool {
    !text.is_empty() && text.len() <= MAX_STRING_BYTES && !text.chars().any(char::is_control)
}

fn valid_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn clean_path(path: &str) -> bool {
    bounded_text(path)
        && !path.contains('\\')
        && !path.contains("//")
        && !path.split('/').any(|part| part == ".." || (part == "." && path != "."))
        && (path == "/" || !path.ends_with('/'))
}

fn validate_manifest(manifest: &DependencyManifest) -> Result<(), Bypass> {
    if !manifest.complete {
        return Err(Bypass::IncompleteManifest);
    }
    if manifest.dependencies.is_empty() || manifest.dependencies.len() > MAX_DEPENDENCIES {
        return Err(Bypass::IncompleteManifest);
    }
    let mut names = BTreeSet::new();
    for item in &manifest.dependencies {
        if !valid_dependency(item) {
            return Err(Bypass::InvalidIdentity);
        }
        if !names.insert(&item.name) {
            return Err(Bypass::InvalidIdentity);
        }
    }
    debug_assert!(names.len() == manifest.dependencies.len());
    debug_assert!(manifest.dependencies.len() <= MAX_DEPENDENCIES);
    Ok(())
}

/// A dependency identity is only admissible when its alias-relative name is a
/// clean relative path and its digest is a full BLAKE3 hex digest.
fn valid_dependency(item: &DependencyIdentity) -> bool {
    if !clean_path(&item.name) {
        return false;
    }
    if item.name.starts_with('/') {
        return false;
    }
    if item.name == "." {
        return false;
    }
    valid_digest(&item.digest_blake3)
}

fn validate_key_inputs(inputs: &KeyInputs) -> Result<(), Bypass> {
    if !inputs.output_path_independent {
        return Err(Bypass::PathDependentOutput);
    }
    for digest in [
        &inputs.compiler_blake3,
        &inputs.source_blake3,
        &inputs.platform_blake3,
        &inputs.environment_blake3,
        &inputs.toolchain_blake3,
    ] {
        if !valid_digest(digest) {
            return Err(Bypass::InvalidIdentity);
        }
    }
    if inputs.probe_blake3.as_deref().is_some_and(|value| !valid_digest(value)) {
        return Err(Bypass::InvalidIdentity);
    }
    if inputs.roots.is_empty() || inputs.roots.len() > MAX_ROOTS {
        return Err(Bypass::BoundsExceeded);
    }
    let mut paths = BTreeSet::new();
    for root in &inputs.roots {
        if !clean_path(&root.path) || !valid_digest(&root.digest_blake3) || !paths.insert(&root.path) {
            return Err(Bypass::InvalidIdentity);
        }
    }
    debug_assert!(paths.len() == inputs.roots.len());
    debug_assert!(inputs.roots.len() <= MAX_ROOTS);
    validate_manifest(&inputs.manifest)
}

/// Resolve `path` against an attested identity root, returning the
/// root-relative remainder. `None` means the path is outside the root.
fn relative_to<'a>(path: &'a str, root: &IdentityRoot) -> Option<&'a str> {
    let root_path = root.path.as_str();
    if path == root_path {
        return Some("");
    }
    if root_path == "." && !path.starts_with('/') {
        return Some(path);
    }
    if root_path == "/" {
        return path.strip_prefix('/');
    }
    path.strip_prefix(root_path).and_then(|suffix| suffix.strip_prefix('/'))
}

fn hash_field(hash: &mut blake3::Hasher, bytes: &[u8]) {
    hash.update(&(bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn hash_path(hash: &mut blake3::Hasher, path: &str, roots: &[IdentityRoot]) -> Result<(), Bypass> {
    if !clean_path(path) {
        return Err(Bypass::UnclassifiedPath);
    }
    let root = roots
        .iter()
        .filter(|root| relative_to(path, root).is_some())
        .max_by_key(|root| root.path.len())
        .ok_or(Bypass::UnclassifiedPath)?;
    if !root.path_independent {
        return Err(Bypass::PathDependentOutput);
    }
    let relative = relative_to(path, root).ok_or(Bypass::UnclassifiedPath)?;
    hash_field(hash, root.digest_blake3.as_bytes());
    hash_field(hash, relative.as_bytes());
    Ok(())
}

// r[impl mantle.cc_compile_cache.content_keyed_identity]
// r[impl mantle.cc_compile_cache.probe_and_failure_cache]
/// Domain-separated BLAKE3 over complete content facts and ordered options.
/// Physical root paths, output paths, and depfile destinations are never hashed.
pub fn semantic_key(invocation: &Invocation, inputs: &KeyInputs) -> Result<String, Bypass> {
    validate_key_inputs(inputs)?;
    if invocation.semantic_args.len() > MAX_ARGUMENTS {
        return Err(Bypass::BoundsExceeded);
    }
    let mut hash = blake3::Hasher::new_derive_key("mantle.cc.compile.content.v1");
    for digest in [
        &inputs.compiler_blake3,
        &inputs.source_blake3,
        &inputs.platform_blake3,
        &inputs.environment_blake3,
        &inputs.toolchain_blake3,
    ] {
        hash_field(&mut hash, digest.as_bytes());
    }
    hash_field(&mut hash, inputs.probe_blake3.as_deref().unwrap_or("").as_bytes());
    hash_path(&mut hash, &invocation.source, &inputs.roots)?;
    hash_field(&mut hash, match invocation.dependency_mode {
        None => b"none",
        Some(DependencyMode::Md) => b"md",
        Some(DependencyMode::Mmd) => b"mmd",
    });
    hash_field(&mut hash, &[u8::from(invocation.phony_targets)]);
    hash_arguments(&mut hash, &invocation.semantic_args, &inputs.roots)?;
    let mut roots: Vec<&str> = inputs.roots.iter().map(|root| root.digest_blake3.as_str()).collect();
    roots.sort_unstable();
    debug_assert!(roots.len() == inputs.roots.len());
    hash_field(&mut hash, &(roots.len() as u64).to_le_bytes());
    for root in roots {
        hash_field(&mut hash, root.as_bytes());
    }
    let mut dependencies: Vec<&DependencyIdentity> = inputs.manifest.dependencies.iter().collect();
    dependencies.sort_unstable();
    debug_assert!(dependencies.len() == inputs.manifest.dependencies.len());
    hash_field(&mut hash, &(dependencies.len() as u64).to_le_bytes());
    for dependency in dependencies {
        hash_field(&mut hash, dependency.name.as_bytes());
        hash_field(&mut hash, dependency.digest_blake3.as_bytes());
    }
    let key = hash.finalize().to_hex().to_string();
    debug_assert!(key.len() == 64);
    Ok(key)
}

fn hash_arguments(hash: &mut blake3::Hasher, args: &[SemanticArgument], roots: &[IdentityRoot]) -> Result<(), Bypass> {
    hash_field(hash, &(args.len() as u64).to_le_bytes());
    for arg in args {
        match arg {
            SemanticArgument::Literal(value) => {
                if !bounded_text(value) || value.contains(['/', '\\']) {
                    return Err(Bypass::UnclassifiedPath);
                }
                debug_assert!(!value.is_empty());
                hash_field(hash, b"literal");
                hash_field(hash, value.as_bytes());
            }
            SemanticArgument::Path { option, path } => {
                if !bounded_text(option) {
                    return Err(Bypass::InvalidIdentity);
                }
                debug_assert!(!option.is_empty());
                hash_field(hash, b"path");
                hash_field(hash, option.as_bytes());
                hash_path(hash, path, roots)?;
            }
            SemanticArgument::PrefixMap { option, from, to } => {
                if !bounded_text(option) || !is_hashable_prefix_target(to) {
                    return Err(Bypass::UnclassifiedPath);
                }
                debug_assert!(!option.is_empty());
                hash_field(hash, b"prefix-map");
                hash_field(hash, option.as_bytes());
                hash_path(hash, from, roots)?;
                hash_field(hash, to.as_bytes());
            }
        }
    }
    Ok(())
}

/// A hashed prefix-map target must stay relative, bounded and free of
/// control characters, `=` separators and `..` segments.
fn is_hashable_prefix_target(to: &str) -> bool {
    if to.len() > MAX_STRING_BYTES {
        return false;
    }
    if to.starts_with('/') {
        return false;
    }
    if to.contains(['\\', '=']) {
        return false;
    }
    if to.chars().any(char::is_control) {
        return false;
    }
    if to.split('/').any(|part| part == "..") {
        return false;
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CachedResultKind {
    Object,
    Failure,
    Probe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    Hit,
    Compiled,
    Forwarded,
    Degraded,
    FailureHit,
    ProbeHit,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema: String,
    pub disposition: Disposition,
    pub key: Option<String>,
    pub bypass: Option<Bypass>,
    pub probe: bool,
}

/// Caller must additionally compare the stored key with semantic_key's result.
/// Failures and probes have no argument-only or incomplete-manifest fallback.
pub fn replay_admission(kind: CachedResultKind, invocation: &Invocation, inputs: &KeyInputs) -> Result<String, Bypass> {
    if kind == CachedResultKind::Probe && inputs.probe_blake3.is_none() {
        return Err(Bypass::InvalidIdentity);
    }
    semantic_key(invocation, inputs)
}

/// The main rule identity a depfile must declare.
struct DepfileExpectation<'a> {
    target: &'a str,
    source: &'a str,
}

/// Parse the GCC/Clang make depfile subset. One main target is required; -MP
/// stubs must name known prerequisites. Rejects make expansion, multiple main
/// rules, malformed escaping, absent source, truncation, and unknown syntax.
/// The resulting paths are *not* content identities: the shell must hash them.
pub fn parse_depfile<T, U>(text: &str, expected_target: T, expected_source: U) -> Result<Vec<String>, Bypass>
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    let expectation = DepfileExpectation {
        target: expected_target.as_ref(),
        source: expected_source.as_ref(),
    };
    if text.is_empty() || text.len() > MAX_DEPFILE_BYTES {
        return Err(Bypass::BoundsExceeded);
    }
    if !text.ends_with('\n') || !bounded_text(expectation.target) || !bounded_text(expectation.source) {
        return Err(Bypass::InvalidDepfile);
    }
    let lines = depfile_lines(text)?;
    let mut dependencies = Vec::new();
    let mut stubs = BTreeSet::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (targets, inputs) = depfile_rule(&line)?;
        if dependencies.is_empty() {
            if targets.len() != 1 {
                return Err(Bypass::InvalidDepfile);
            }
            if targets[0] != expectation.target {
                return Err(Bypass::InvalidDepfile);
            }
            dependencies = main_rule_dependencies(inputs, &expectation)?;
        } else {
            if targets.len() != 1 {
                return Err(Bypass::InvalidDepfile);
            }
            if !inputs.is_empty() {
                return Err(Bypass::InvalidDepfile);
            }
            if !dependencies.contains(&targets[0]) {
                return Err(Bypass::InvalidDepfile);
            }
            if !stubs.insert(targets[0].clone()) {
                return Err(Bypass::InvalidDepfile);
            }
        }
    }
    if dependencies.is_empty() {
        return Err(Bypass::InvalidDepfile);
    }
    debug_assert!(!dependencies.is_empty());
    debug_assert!(dependencies.len() <= MAX_DEPENDENCIES);
    Ok(dependencies)
}

/// The main rule must name the expected source; prerequisites are returned in
/// first-seen order with duplicates removed.
fn main_rule_dependencies(inputs: Vec<String>, expectation: &DepfileExpectation<'_>) -> Result<Vec<String>, Bypass> {
    if inputs.is_empty() {
        return Err(Bypass::InvalidDepfile);
    }
    if !inputs.iter().any(|path| path == expectation.source) {
        return Err(Bypass::InvalidDepfile);
    }
    let mut unique = BTreeSet::new();
    let mut dependencies = Vec::with_capacity(inputs.len());
    for input in inputs {
        if unique.insert(input.clone()) {
            dependencies.push(input);
        }
    }
    debug_assert!(dependencies.len() == unique.len());
    Ok(dependencies)
}

// Keep escaped characters encoded until rule tokenization. Continuations become
// whitespace (not an empty string, which could join two dependency names).
fn depfile_lines(text: &str) -> Result<Vec<String>, Bypass> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while chars.peek().is_some() {
        let character = chars.next().ok_or(Bypass::InvalidDepfile)?;
        match character {
            '\\' => push_escape(&mut current, &mut chars)?,
            '#' => {
                while chars.peek().is_some_and(|value| *value != '\n') {
                    chars.next();
                }
            }
            '\n' => {
                if lines.len() >= MAX_DEPENDENCIES.saturating_add(1) {
                    return Err(Bypass::BoundsExceeded);
                }
                lines.push(core::mem::take(&mut current));
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '$' | ';' | '|' | '=' | '\0' => return Err(Bypass::InvalidDepfile),
            value if value.is_control() && value != '\t' => return Err(Bypass::InvalidDepfile),
            value => current.push(value),
        }
    }
    if !current.is_empty() {
        return Err(Bypass::InvalidDepfile);
    }
    debug_assert!(current.is_empty());
    debug_assert!(lines.len() <= MAX_DEPENDENCIES.saturating_add(1));
    Ok(lines)
}

/// An escape keeps its backslash unless it continues a line; continuations
/// collapse to a single space so two dependency names never join.
fn push_escape(current: &mut String, chars: &mut Peekable<Chars<'_>>) -> Result<(), Bypass> {
    let next = chars.next().ok_or(Bypass::InvalidDepfile)?;
    if next == '\n' {
        current.push(' ');
        debug_assert!(current.ends_with(' '));
        return Ok(());
    }
    if next == '\r' {
        if chars.peek() != Some(&'\n') {
            return Err(Bypass::InvalidDepfile);
        }
        chars.next();
        current.push(' ');
        debug_assert!(current.ends_with(' '));
        return Ok(());
    }
    if matches!(next, ' ' | '\t' | '\\' | '#' | ':') {
        current.push('\\');
        current.push(next);
        debug_assert!(current.ends_with(next));
        return Ok(());
    }
    Err(Bypass::InvalidDepfile)
}

fn depfile_rule(line: &str) -> Result<(Vec<String>, Vec<String>), Bypass> {
    let mut targets = Vec::new();
    let mut inputs = Vec::new();
    let mut word = String::new();
    let mut has_colon = false;
    let mut chars = line.chars().peekable();
    while chars.peek().is_some() {
        let character = chars.next().ok_or(Bypass::InvalidDepfile)?;
        match character {
            '\\' => word.push(chars.next().ok_or(Bypass::InvalidDepfile)?),
            ':' => {
                if has_colon {
                    return Err(Bypass::InvalidDepfile);
                }
                finish_word(&mut word, &mut targets)?;
                has_colon = true;
            }
            ' ' | '\t' => {
                finish_word(&mut word, if has_colon { &mut inputs } else { &mut targets })?;
            }
            value => word.push(value),
        }
        if word.len() > MAX_STRING_BYTES {
            return Err(Bypass::BoundsExceeded);
        }
    }
    finish_word(&mut word, if has_colon { &mut inputs } else { &mut targets })?;
    if !has_colon || targets.is_empty() {
        return Err(Bypass::InvalidDepfile);
    }
    debug_assert!(has_colon);
    debug_assert!(targets.len() <= MAX_DEPENDENCIES);
    debug_assert!(inputs.len() <= MAX_DEPENDENCIES);
    Ok((targets, inputs))
}

fn finish_word(word: &mut String, words: &mut Vec<String>) -> Result<(), Bypass> {
    if word.is_empty() {
        return Ok(());
    }
    if words.len() >= MAX_DEPENDENCIES {
        return Err(Bypass::BoundsExceeded);
    }
    words.push(core::mem::take(word));
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }
    fn digest(value: &str) -> String {
        blake3::hash(value.as_bytes()).to_hex().to_string()
    }
    fn inputs(root: &str) -> KeyInputs {
        KeyInputs {
            compiler_blake3: digest("compiler"),
            source_blake3: digest("source"),
            platform_blake3: digest("platform"),
            environment_blake3: digest("environment"),
            toolchain_blake3: digest("toolchain"),
            probe_blake3: Some(digest("probe")),
            roots: vec![IdentityRoot {
                path: root.to_string(),
                digest_blake3: digest("tree"),
                path_independent: true,
            }],
            manifest: DependencyManifest {
                complete: true,
                dependencies: vec![
                    DependencyIdentity {
                        name: "source/a.c".to_string(),
                        digest_blake3: digest("source"),
                    },
                    DependencyIdentity {
                        name: "source/a.h".to_string(),
                        digest_blake3: digest("header"),
                    },
                ],
            },
            output_path_independent: true,
        }
    }
    fn invocation(root: &str) -> Invocation {
        classify(&args(&[
            "-c",
            &alloc::format!("{root}/a.c"),
            "-I",
            root,
            "-DVALUE=1",
            "-o",
            "result.o",
        ]))
        .unwrap()
    }

    #[test]
    fn unknown_and_side_effects_forward_instead_of_guessing() {
        for flag in [
            "-fplugin=plugin.so",
            "--coverage",
            "-save-temps",
            "-MJ",
            "-S",
            "-flto",
            "-g",
            "-Wl,surprise",
            "-I-",
        ] {
            assert!(classify(&args(&["-c", "a.c", flag, "-o", "a.o"])).is_err(), "{flag}");
        }
        assert!(classify(&args(&["-c", "a.c", "b.c", "-o", "a.o"])).is_err());
        assert!(classify(&args(&["-c", "a.c", "-o", "a.c"])).is_err());
        assert!(classify(&args(&["-c", "@response", "-o", "a.o"])).is_err());
        assert!(classify(&args(&["-c", "a.c", "-MF", "a.d", "-o", "a.o"])).is_err());
        assert!(classify(&args(&["-c", "a.c", "-x", "c++", "-o", "a.o"])).is_err());
        assert!(classify(&args(&["-x", "c++", "-c", "a.c", "-o", "a.o"])).is_ok());
        assert!(classify(&args(&["-c", "a.c", "-DT=__TIME__", "-o", "a.o"])).is_err());
    }

    #[test]
    fn ordered_macros_and_destination_independence() {
        let first = classify(&args(&["-c", "/one/a.c", "-DVALUE=1", "-UVALUE", "-o", "a.o", "-MD", "-MP"])).unwrap();
        assert_eq!(first.depfile.as_deref(), Some("a.d"));
        let mut moved = first.clone();
        moved.output = "other.o".to_string();
        moved.depfile = Some("other.d".to_string());
        assert_eq!(semantic_key(&first, &inputs("/one")), semantic_key(&moved, &inputs("/one")));
        moved.semantic_args.swap(0, 1);
        assert_ne!(semantic_key(&first, &inputs("/one")), semantic_key(&moved, &inputs("/one")));
        let mut unsafe_inputs = inputs("/one");
        unsafe_inputs.output_path_independent = false;
        assert_eq!(semantic_key(&first, &unsafe_inputs), Err(Bypass::PathDependentOutput));
    }

    #[test]
    fn relocated_content_roots_match_but_new_inputs_do_not() {
        let original = invocation("/store/old");
        let relocated = invocation("/store/new");
        let facts = inputs("/store/old");
        let key = semantic_key(&original, &facts).unwrap();
        assert_eq!(key, semantic_key(&relocated, &inputs("/store/new")).unwrap());
        for field in 0..9 {
            let mut changed = facts.clone();
            match field {
                0 => changed.compiler_blake3 = digest("changed"),
                1 => changed.source_blake3 = digest("changed"),
                2 => changed.platform_blake3 = digest("changed"),
                3 => changed.environment_blake3 = digest("changed"),
                4 => changed.toolchain_blake3 = digest("changed"),
                5 => changed.probe_blake3 = Some(digest("changed")),
                6 => changed.manifest.dependencies[1].digest_blake3 = digest("changed"),
                7 => changed.roots[0].digest_blake3 = digest("new header added"),
                _ => changed.manifest.dependencies[1].name = "source/other.h".to_string(),
            }
            assert_ne!(key, semantic_key(&original, &changed).unwrap(), "field {field}");
        }
        let mut changed = original.clone();
        changed.semantic_args.push(SemanticArgument::Literal("-O2".to_string()));
        assert_ne!(key, semantic_key(&changed, &facts).unwrap());
    }

    #[test]
    fn path_matching_is_component_bounded_and_never_ambient() {
        assert_eq!(semantic_key(&invocation("/store/other"), &inputs("/store/old")), Err(Bypass::UnclassifiedPath));
        assert_eq!(semantic_key(&invocation("/store/oldish"), &inputs("/store/old")), Err(Bypass::UnclassifiedPath));
        let mut facts = inputs("/store/old");
        facts.roots[0].path_independent = false;
        assert_eq!(semantic_key(&invocation("/store/old"), &facts), Err(Bypass::PathDependentOutput));
        facts.roots[0].path_independent = true;
        let mut action = invocation("/store/old");
        action.source = "/store/old/../hidden.c".to_string();
        assert_eq!(semantic_key(&action, &facts), Err(Bypass::UnclassifiedPath));
    }

    #[test]
    fn prefix_maps_mask_only_attested_roots() {
        let one = classify(&args(&["-c", "/old/a.c", "-ffile-prefix-map=/old=source", "-o", "a.o"])).unwrap();
        let two = classify(&args(&["-c", "/new/a.c", "-ffile-prefix-map=/new=source", "-o", "b.o"])).unwrap();
        assert_eq!(semantic_key(&one, &inputs("/old")).unwrap(), semantic_key(&two, &inputs("/new")).unwrap());
        assert!(classify(&args(&["-c", "/old/a.c", "-ffile-prefix-map=/old=/other", "-o", "a.o"])).is_err());
    }

    #[test]
    fn depfile_escapes_continuations_and_phony_stubs() {
        let parsed = parse_depfile("obj\\ file.o: source.c header\\ name.h dir\\\\file.h hash\\#name.h \\\n more.h\nheader\\ name.h:\ndir\\\\file.h:\nhash\\#name.h:\nmore.h:\n", "obj file.o", "source.c").unwrap();
        assert_eq!(parsed, args(&["source.c", "header name.h", "dir\\file.h", "hash#name.h", "more.h"]));
        assert_eq!(parse_depfile("x.o: x.c \\\r\n x.h\r\nx.h:\r\n", "x.o", "x.c").unwrap(), args(&["x.c", "x.h"]));
    }

    #[test]
    fn malformed_incomplete_and_make_programs_are_rejected() {
        for text in [
            "x.o: x.c",
            "x.o: x.h\n",
            "y.o: x.c\n",
            "x.o: x.c \\\n",
            "x.o: x.c\\q\n",
            "x.o: x.c\nunknown.h:\n",
            "x.o: x.c\ny.o: y.c\n",
            "x.o: $(shell echo x.c)\n",
            "x.o x.d: x.c\n",
            "x.o: x.c: x.h\n",
            "x.o: x.c\nx.c:\nx.c:\n",
        ] {
            assert!(parse_depfile(text, "x.o", "x.c").is_err(), "{text:?}");
        }
    }
    #[test]
    fn volatile_builtins_and_synthesized_spellings_never_replay() {
        for bytes in [
            b"const char *t = __TIME__;".as_slice(),
            b"__DATE__",
            b"__TIMESTAMP__",
            b"#define JOIN(a,b) a ## b",
            b"a %:%: b",
            b"__TI\\\nME__",
            b"__TI??/\nME__",
            b"\\u005f_TIME__",
        ] {
            assert!(volatile_input(bytes), "{bytes:?}");
        }
        assert!(!volatile_input(b"#define VALUE 17\nint x = VALUE;"));
    }

    #[test]
    fn failures_and_probes_require_every_identity() {
        let action = invocation("/one");
        let mut facts = inputs("/one");
        facts.manifest.complete = false;
        for kind in [
            CachedResultKind::Object,
            CachedResultKind::Failure,
            CachedResultKind::Probe,
        ] {
            assert_eq!(replay_admission(kind, &action, &facts), Err(Bypass::IncompleteManifest));
        }
        facts.manifest.complete = true;
        facts.probe_blake3 = None;
        assert_eq!(replay_admission(CachedResultKind::Probe, &action, &facts), Err(Bypass::InvalidIdentity));
        facts.probe_blake3 = Some(digest("probe"));
        assert_eq!(
            replay_admission(CachedResultKind::Failure, &action, &facts).unwrap(),
            semantic_key(&action, &facts).unwrap()
        );
    }
}
