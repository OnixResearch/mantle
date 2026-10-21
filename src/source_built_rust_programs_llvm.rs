//! Original selected LLVM CMake evaluation. Configuration and generator edges
//! come from the source-built evaluator; each native request is a protected leaf.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;

use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_program_authoring::ContinuationReservationPolicy;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_stagex_programs::controller::ControllerExecutable;
use crate::source_built_stagex_programs::controller::ControllerSlot;
use crate::source_built_stagex_programs::controller::ControllerSpec;
use crate::source_built_stagex_programs::controller::append_controller;
const STAGE: &str = "full-source-rust-provider";
const HELPER_PATH: &str = "mantle-source-authoring/rust-llvm-products.c";
const CONTROLLER_PATH: &str = "mantle-source-authoring/rust-llvm-controller.c";
pub(crate) struct Fragment {
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub exports: BTreeMap<String, ArtifactRef>,
}
fn error(s: impl Into<String>) -> RunError {
    RunError::Build(format!("original LLVM source authoring: {}", s.into()))
}
fn lit(s: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: s.into() }],
    }
}
fn input(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn output(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn produced(id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: name.into(),
    }
}
fn require(r: &ArtifactRef) -> Result<(), RunError> {
    if matches!(r, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("LLVM source/runtime/executable requires exact Produced authority"))
    }
}
fn native_export(request: &AuthoringRequest<'_>, name: &str) -> Result<ArtifactRef, RunError> {
    let r = request
        .predecessor_exports
        .get(&format!("full-source-native-provider:{name}"))
        .ok_or_else(|| error(format!("missing exact original native export {name}")))?
        .clone();
    require(&r)?;
    Ok(r)
}
fn spec(kind: ArtifactKind, disk: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max: disk }
}
fn exec(
    id: String,
    image: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    declared: BTreeMap<String, RecipeOutput>,
) -> RecipeStep {
    inputs.insert("executable".into(), image);
    RecipeStep {
        id,
        inputs,
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "executable".into(),
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: None,
        },
    }
}
/// The native owner packages these under its own authenticated source record.
pub(crate) fn cmake_source_files() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "source-built/native/llvm-cmake-patch.c".into(),
            include_str!("source_built_rust_programs_llvm_cmake_patch.c").into(),
        ),
        (
            "source-built/native/llvm-cmake-phase.h".into(),
            include_str!("source_built_rust_programs_llvm_cmake_phase.h").into(),
        ),
    ])
}
/// Source-only patch before original CMake bootstrap. `source_member` is the
/// authentic archive top component, not a guessed configured/build directory.
pub(crate) fn append_cmake_source(
    request: &AuthoringRequest<'_>,
    recipe: &mut SourceRecipe,
    id: &str,
    original: ArtifactRef,
    source_member: &str,
) -> Result<ArtifactRef, RunError> {
    require(&original)?;
    crate::source_built_recipe::relative(source_member, true)?;
    let helper = crate::source_built_native_programs::append_stagex_c_helper(
        request,
        recipe,
        &format!("{id}.transformer"),
        "source-built/native/llvm-cmake-patch.c",
    )?;
    let header = ArtifactRef::ManifestSource {
        role: crate::source_built_fixed_point::SourceAuthorityRole::NativeSourceBundle,
        record_identity: crate::source_built_native_programs::RECORD_IDENTITY.into(),
        member: "source-built/native/llvm-cmake-phase.h".into(),
    };
    let step = exec(
        id.into(),
        helper,
        BTreeMap::from([("original".into(), original), ("header".into(), header)]),
        vec![
            input("original", source_member),
            input("header", ""),
            output("source", ""),
        ],
        BTreeMap::from([("source".into(), spec(ArtifactKind::Tree, request.proof.resource_bounds.disk_bytes_max))]),
    );
    let result = ArtifactRef::Produced {
        action_id: format!("{}/{id}", recipe.stage_id),
        output_id: "source".into(),
    };
    recipe.steps.push(step);
    Ok(result)
}
/// Native host composition already includes the original make/CMake evaluator.
/// Only the shared option grammar is appended, never a second request protocol.
pub(crate) fn controller_hooks_source() -> String {
    include_str!("source_built_rust_programs_llvm_driver.c").into()
}
pub(crate) fn controller_source() -> Result<String, RunError> {
    crate::source_built_native_programs::host::controller_source(&[])
}

struct Original {
    policy: BTreeMap<String, String>,
    native_files: u32,
}
fn original(request: &AuthoringRequest<'_>, version: &str) -> Result<Original, RunError> {
    let archive = super::source_archive(request.source_roots, &format!("rust-{version}"))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(
        std::fs::File::open(archive).map_err(|e| error(e.to_string()))?,
    ));
    let prefix = format!("rustc-{version}-src/");
    let mut result = Original {
        policy: BTreeMap::new(),
        native_files: 0,
    };
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(|e| error(e.to_string()))?.into_owned();
        let Some(path) = path.to_str().and_then(|path| path.strip_prefix(&prefix)) else {
            continue;
        };
        if path.starts_with("src/llvm-project/llvm/")
            && [".cpp", ".c", ".S", ".td"].iter().any(|extension| path.ends_with(extension))
        {
            result.native_files =
                result.native_files.checked_add(1).ok_or_else(|| error("LLVM source count overflow"))?;
        }
        if (path.starts_with("src/bootstrap/") && (path.ends_with(".rs") || path.ends_with(".toml")))
            || path == "src/ci/channel"
        {
            if entry.size() > 4 * 1024 * 1024 {
                return Err(error("original bootstrap member exceeds bound"));
            }
            let mut text = String::new();
            entry.read_to_string(&mut text).map_err(|e| error(e.to_string()))?;
            result.policy.insert(path.into(), text);
        }
    }
    Ok(result)
}
fn archive_member(request: &AuthoringRequest<'_>, identity: &str, member: &str) -> Result<String, RunError> {
    let archive = super::source_archive(request.source_roots, identity)?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(
        std::fs::File::open(archive).map_err(|e| error(e.to_string()))?,
    ));
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        if entry.path().map_err(|e| error(e.to_string()))?.as_ref() == std::path::Path::new(member) {
            if !entry.header().entry_type().is_file() || entry.size() > 4 * 1024 * 1024 {
                return Err(error("original make source member kind/bound"));
            }
            let mut text = String::new();
            entry.read_to_string(&mut text).map_err(|e| error(e.to_string()))?;
            return Ok(text);
        }
    }
    Err(error(format!("original source lacks {member}")))
}
/// Decode only Rust's ordinary quoted source literals, including continuation
/// whitespace. This reads owner policy strings, never a generated build file.
fn rust_string(text: &str) -> Result<(String, usize), RunError> {
    let Some(text) = text.strip_prefix('"') else {
        return Err(error("expected original Rust string literal"));
    };
    let mut value = String::new();
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Ok((value, i + 2)),
            '\\' => {
                let (_, c) = chars.next().ok_or_else(|| error("truncated original Rust escape"))?;
                match c {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\n' => {
                        while chars.clone().next().is_some_and(|(_, c)| c.is_whitespace()) {
                            chars.next();
                        }
                    }
                    _ => return Err(error("unclassified original Rust policy escape")),
                }
            }
            _ => value.push(c),
        }
    }
    Err(error("unterminated original Rust policy literal"))
}
fn source_constant(source: &str, name: &str) -> Result<String, RunError> {
    let marker = format!("const {name}:");
    let after = source.split_once(&marker).ok_or_else(|| error(format!("original Mantle policy lacks {name}")))?.1;
    let after = after.split_once('=').ok_or_else(|| error("original constant lacks initializer"))?.1.trim_start();
    rust_string(after).map(|(value, _)| value)
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn default_bool(source: &Original, field: &str) -> Result<bool, RunError> {
    let marker = format!("{field}:");
    let mut selected = None;
    for text in source.policy.values() {
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with(&marker) || !line.contains(".unwrap_or(") {
                continue;
            }
            let value = line.split_once(".unwrap_or(").unwrap().1.split(')').next().unwrap();
            let value = match value.trim() {
                "true" => true,
                "false" => false,
                _ => continue,
            };
            if selected.replace(value).is_some() {
                return Err(error(format!("ambiguous original LLVM default {field}")));
            }
        }
    }
    selected.ok_or_else(|| error(format!("original bootstrap lacks literal default for {field}")))
}
fn selected_profile(source: &Original) -> Result<toml::Value, RunError> {
    let mut profiles = source
        .policy
        .iter()
        .filter(|(path, _)| path.starts_with("src/bootstrap/defaults/") && path.ends_with(".compiler.toml"));
    let (path, text) = profiles.next().ok_or_else(|| error("original compiler profile missing"))?;
    if profiles.next().is_some() {
        return Err(error("ambiguous original compiler profile"));
    }
    text.parse().map_err(|e| error(format!("original compiler profile {path}: {e}")))
}
fn llvm_bool(source: &Original, profile: &toml::Value, key: &str, field: &str) -> Result<bool, RunError> {
    match profile.get("llvm").and_then(|v| v.get(key)) {
        Some(v) => v.as_bool().ok_or_else(|| error(format!("nonboolean original llvm.{key}"))),
        None => default_bool(source, field),
    }
}
fn default_targets(source: &str, begin: &str, end: &str) -> Result<String, RunError> {
    let block = source
        .split_once(begin)
        .ok_or_else(|| error("original LLVM target selection missing"))?
        .1
        .split_once(end)
        .ok_or_else(|| error("original LLVM target selection boundary changed"))?
        .0;
    let block = block.split_once("None =>").ok_or_else(|| error("original LLVM default targets missing"))?.1;
    let literal = block.find('"').ok_or_else(|| error("original LLVM default target string missing"))?;
    rust_string(&block[literal..]).map(|(value, _)| value)
}
fn first_configuration(
    request: &AuthoringRequest<'_>,
    mantle: &str,
    cc: &str,
    cxx: &str,
) -> Result<(BTreeMap<String, String>, Vec<String>), RunError> {
    let original = archive_member(request, "mrustc-0.12.0", "mrustc-0.12.0/minicargo.mk")?;
    let before = source_constant(mantle, "FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_ORIGINAL_LINE")?;
    let after = source_constant(mantle, "FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_PATCHED_LINE")?;
    if original.matches(&before).count() != 1 {
        return Err(error("original minicargo LLVM option policy anchor changed"));
    }
    let original = original.replacen(&before, &after, 1);
    let targets = original
        .lines()
        .find_map(|line| line.strip_prefix("LLVM_TARGETS ?= "))
        .ok_or_else(|| error("original minicargo LLVM target selection missing"))?
        .split('#')
        .next()
        .unwrap()
        .trim();
    let mut options = BTreeMap::new();
    for line in original.lines().filter(|line| line.starts_with("LLVM_CMAKE_OPTS ")) {
        let (_, value) = line.split_once('=').ok_or_else(|| error("original LLVM Make assignment"))?;
        let value = value
            .replace("$(firstword $(subst -, ,$(RUSTC_TARGET)))", "x86_64")
            .replace("$(RUSTC_TARGET)", "x86_64-unknown-linux-musl")
            .replace("$(LLVM_TARGETS)", targets)
            .replace("$(CXX)", cxx)
            .replace("$(CC)", cc)
            .replace("$(LLVM_CMAKE_OPTS_EXTRA)", "");
        for word in value.split_whitespace() {
            let Some((key, value)) = word.split_once('=') else {
                return Err(error(format!("unresolved original LLVM Make word {word}")));
            };
            if key.contains('$') || value.contains('$') {
                return Err(error("unresolved original LLVM Make expression"));
            }
            options.insert(key.into(), value.trim_matches('"').into());
        }
    }
    let original_build = source_constant(mantle, "FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_ORIGINAL_LINE")?;
    if original.lines().filter(|line| *line == original_build).count() != 1 {
        return Err(error("original minicargo LLVM build rule anchor changed"));
    }
    let patched_build = source_constant(mantle, "FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_PATCHED_PREFIX")?;
    let requested = patched_build
        .strip_prefix(&original_build)
        .ok_or_else(|| error("selected original LLVM build prefix changed"))?;
    let mut selected = requested.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
    let targets = source_constant(mantle, "FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_STATIC_ARCHIVE_TARGETS")?;
    selected.extend(targets.split_whitespace().map(str::to_owned));
    Ok((options, selected))
}
fn successor_configuration(source: &Original, version: &str) -> Result<BTreeMap<String, String>, RunError> {
    let profile = selected_profile(source)?;
    let llvm = source
        .policy
        .iter()
        .find(|(path, _)| path.ends_with("/build_steps/llvm.rs"))
        .ok_or_else(|| error("original bootstrap LLVM owner missing"))?
        .1;
    let optimize = llvm_bool(source, &profile, "optimize", "llvm_optimize")?;
    let debuginfo = llvm_bool(source, &profile, "release-debuginfo", "llvm_release_debuginfo")?;
    let enabled = |value: bool| if value { "ON" } else { "OFF" }.to_owned();
    let mut options = BTreeMap::from([
        (
            "CMAKE_BUILD_TYPE".into(),
            if !optimize {
                "Debug"
            } else if debuginfo {
                "RelWithDebInfo"
            } else {
                "Release"
            }
            .into(),
        ),
        ("LLVM_TARGET_ARCH".into(), "x86_64".into()),
        ("LLVM_DEFAULT_TARGET_TRIPLE".into(), "x86_64-unknown-linux-musl".into()),
        ("LLVM_UNREACHABLE_OPTIMIZE".into(), "OFF".into()),
        (
            "LLVM_ENABLE_ASSERTIONS".into(),
            enabled(llvm_bool(source, &profile, "assertions", "llvm_assertions")?),
        ),
        ("LLVM_ENABLE_PLUGINS".into(), enabled(llvm_bool(source, &profile, "plugins", "llvm_plugins")?)),
        ("LLVM_INCLUDE_TESTS".into(), enabled(llvm_bool(source, &profile, "tests", "llvm_tests")?)),
        (
            "LLVM_ENABLE_WARNINGS".into(),
            enabled(llvm_bool(source, &profile, "enable-warnings", "llvm_enable_warnings")?),
        ),
    ]);
    for (key, value) in [
        ("LLVM_TARGETS_TO_BUILD", default_targets(llvm, "let llvm_targets =", "let llvm_exp_targets =")?),
        (
            "LLVM_EXPERIMENTAL_TARGETS_TO_BUILD",
            default_targets(llvm, "let llvm_exp_targets =", "let assertions =")?,
        ),
    ] {
        options.insert(
            key.into(),
            profile
                .get("llvm")
                .and_then(|v| {
                    v.get(if key == "LLVM_TARGETS_TO_BUILD" {
                        "targets"
                    } else {
                        "experimental-targets"
                    })
                })
                .and_then(toml::Value::as_str)
                .unwrap_or(&value)
                .into(),
        );
    }
    for (key, value) in [
        ("LLVM_INCLUDE_EXAMPLES", "OFF"),
        ("LLVM_INCLUDE_DOCS", "OFF"),
        ("LLVM_INCLUDE_BENCHMARKS", "OFF"),
        ("LLVM_ENABLE_LIBEDIT", "OFF"),
        ("LLVM_ENABLE_BINDINGS", "OFF"),
        ("LLVM_ENABLE_Z3_SOLVER", "OFF"),
        ("LLVM_INSTALL_UTILS", "ON"),
        ("LLVM_ENABLE_LIBXML2", "OFF"),
        ("CMAKE_INSTALL_MESSAGE", "LAZY"),
    ] {
        if !llvm.contains(&format!("\"{key}\"")) {
            return Err(error(format!("original LLVM owner changed option {key}")));
        }
        options.insert(key.into(), value.into());
    }
    options.insert("LLVM_PARALLEL_COMPILE_JOBS".into(), "1".into());
    let channel = source.policy.get("src/ci/channel").ok_or_else(|| error("original release channel missing"))?.trim();
    options.insert(
        "LLVM_VERSION_SUFFIX".into(),
        if channel == "dev" {
            "-rust-dev".into()
        } else {
            format!("-rust-{version}-{channel}")
        },
    );
    let zstd = llvm_bool(source, &profile, "libzstd", "llvm_libzstd")?;
    options.insert("LLVM_ENABLE_ZSTD".into(), if zstd { "FORCE_ON" } else { "OFF" }.into());
    if zstd {
        options.insert("LLVM_USE_STATIC_ZSTD".into(), "TRUE".into());
    }
    Ok(options)
}
fn text(recipe: &mut SourceRecipe, id: String, value: String) -> ArtifactRef {
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("text".into(), spec(ArtifactKind::Text, 64 * 1024 * 1024))]),
        operation: RecipeOperation::WriteText {
            output: "text".into(),
            text: value,
        },
    });
    produced(&id, "text")
}
fn assemble(recipe: &mut SourceRecipe, id: String, values: BTreeMap<String, ArtifactRef>, disk: u64) -> ArtifactRef {
    let mut inputs = BTreeMap::new();
    let mut entries = BTreeMap::new();
    for (index, (path, value)) in values.into_iter().enumerate() {
        let name = format!("member-{index}");
        inputs.insert(name.clone(), value);
        entries.insert(path, name);
    }
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree, disk))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    });
    produced(&id, "tree")
}
fn netstring(to: &mut String, value: &str) {
    to.push_str(&format!("{}:{value},", value.len()));
}
fn render(word: &RecipeWord, roots: &BTreeMap<String, String>) -> Result<String, RunError> {
    let mut result = String::new();
    for part in &word.parts {
        match part {
            WordPart::Literal { text } => result.push_str(text),
            WordPart::Input { name, path } => {
                result.push_str(
                    roots.get(name).ok_or_else(|| error(format!("missing original native data root {name}")))?,
                );
                if !path.is_empty() {
                    result.push('/');
                    result.push_str(path);
                }
            }
            WordPart::Runtime { mount, path } => {
                result.push_str(mount);
                if !path.is_empty() {
                    result.push('/');
                    result.push_str(path);
                }
            }
            _ => return Err(error("unclassified native compiler option source")),
        }
    }
    Ok(result)
}
fn configured_overrides(source: &str) -> Result<BTreeMap<String, String>, RunError> {
    let source = source
        .split_once("const RUSTC_SOURCE_LLVM_BUILD_CONFIG:")
        .ok_or_else(|| error("selected LLVM build-config policy missing"))?
        .1
        .split_once("concat!(")
        .ok_or_else(|| error("original LLVM build-config initializer changed"))?
        .1
        .split_once(");")
        .ok_or_else(|| error("original LLVM build-config boundary changed"))?
        .0;
    let mut rest = source;
    let mut value = String::new();
    while let Some(index) = rest.find('"') {
        let (part, used) = rust_string(&rest[index..])?;
        value.push_str(&part);
        rest = &rest[index + used..];
    }
    let value = value.parse::<toml::Value>().map_err(|e| error(format!("selected LLVM build-config source: {e}")))?;
    value
        .get("build-config")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| error("selected LLVM build-config is not a table"))?
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|value| (key.clone(), value.into()))
                .ok_or_else(|| error("nonstring selected LLVM build-config"))
        })
        .collect()
}

/// Complete original first/successor LLVM graph. `remaining_steps` is the
/// caller's actual remaining immutable recipe capacity, not a per-stage reset.
/// `cmake_phase` and its matching installed root are genuine NativeHost products.
#[allow(clippy::too_many_arguments)]
pub(crate) fn author(
    request: &AuthoringRequest<'_>,
    version: &str,
    source_tree: ArtifactRef,
    original_mrustc: ArtifactRef,
    runtime_members: &BTreeMap<String, ArtifactRef>,
    reservations: &ContinuationReservationPolicy,
    cmake_phase: ArtifactRef,
    cmake_phase_root: ArtifactRef,
    remaining_steps: usize,
) -> Result<Fragment, RunError> {
    for reference in [&source_tree, &original_mrustc, &cmake_phase, &cmake_phase_root] {
        require(reference)?;
    }
    let route = super::source_route(request.source_roots)?;
    if !route.stages.iter().any(|stage| stage.rust_version == version) {
        return Err(error("LLVM release is outside the original selected route"));
    }
    let source = original(request, version)?;
    let mantle_root = request
        .source_roots
        .get(&crate::source_built_fixed_point::SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("original Mantle policy missing"))?;
    let mantle =
        std::fs::read_to_string(mantle_root.join("src/rust_source_provider.rs")).map_err(|e| error(e.to_string()))?;
    let id = format!("llvm-{version}");
    let disk = request.proof.resource_bounds.disk_bytes_max;
    let file_limit = disk.min(64 * 1024 * 1024);
    let mut recipe = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "original LLVM CMake source continuation".into(),
            provenance: "bootstrap/rust-source-musl-host-plan.ncl".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let files = BTreeMap::from([
        (HELPER_PATH.into(), include_str!("source_built_rust_programs_llvm_source.c").into()),
        (CONTROLLER_PATH.into(), controller_source()?),
    ]);
    let collector = super::translation::append_c_helper(
        request,
        &mut recipe,
        &format!("{id}.products-helper"),
        super::helper_source_ref(HELPER_PATH),
        BTreeMap::new(),
    )?;
    let controller = super::translation::append_c_helper(
        request,
        &mut recipe,
        &format!("{id}.controller-helper"),
        super::helper_source_ref(CONTROLLER_PATH),
        BTreeMap::new(),
    )?;
    let native_runtime = runtime_members
        .get("tree")
        .ok_or_else(|| error("original normalized compiler runtime missing"))?
        .clone();
    require(&native_runtime)?;
    let mut native = super::translation::Native::new(
        request,
        &mut recipe,
        &format!("{id}.native"),
        BTreeMap::new(),
        Some(native_runtime.clone()),
    )?;
    let mut tool_members = BTreeMap::new();
    let mut inventory = Vec::new();
    for (name, role) in [
        ("gcc", "native.gcc"),
        ("g++", "native.gxx"),
        ("cc1", "native.cc1"),
        ("cc1plus", "native.cc1plus"),
        ("as", "native.as"),
        ("collect2", "native.collect2-query"),
        ("ld", "native.ld"),
        ("ar", "native.ar"),
        ("ranlib", "native.ranlib"),
        ("nm", "native.nm"),
        ("objcopy", "native.objcopy"),
        ("strip", "native.strip"),
        ("readelf", "native.readelf"),
    ] {
        let image = native_export(request, role)?;
        tool_members.insert(name.into(), image.clone());
        inventory.push((name.into(), image));
    }
    let tools = assemble(&mut recipe, format!("{id}.tools"), tool_members, disk);
    native.inputs.insert("phase-tools".into(), tools);
    let source_id = format!("{id}.original-project");
    recipe.steps.push(exec(
        source_id.clone(),
        collector.clone(),
        BTreeMap::from([
            ("rust-source".into(), source_tree),
            ("mrustc-source".into(), original_mrustc),
        ]),
        vec![
            lit("source"),
            input("rust-source", &format!("rustc-{version}-src/src/llvm-project")),
            output("source", ""),
        ],
        BTreeMap::from([("source".into(), spec(ArtifactKind::Tree, disk))]),
    ));
    let llvm_source = produced(&source_id, "source");
    let store = request.proof.logical_store_prefix.trim_end_matches('/');
    let mut logical = BTreeMap::new();
    let mut command_inputs = BTreeMap::new();
    let mut runtime_mounts = BTreeMap::new();
    for (name, reference) in &native.inputs {
        let root = if name == "rust-runtime" {
            "/mantle-rust-runtime".to_string()
        } else {
            format!("{store}/mantle-{id}-{name}")
        };
        logical.insert(name.clone(), root.clone());
        runtime_mounts.insert(root, name.clone());
        command_inputs.insert(name.clone(), reference.clone());
    }
    for (name, reference) in [
        ("llvm-source", llvm_source.clone()),
        ("cmake-root", cmake_phase_root),
        ("make-root", native_export(request, "host.make-phase-root")?),
    ] {
        let root = format!("{store}/mantle-{id}-{name}");
        logical.insert(name.into(), root.clone());
        runtime_mounts.insert(root, name.into());
        command_inputs.insert(name.into(), reference);
    }
    let busybox = native_export(request, "host.busybox")?;
    let busybox_root = native_export(request, "host.busybox-root")?;
    let busybox_path = format!("{store}/mantle-{id}-busybox");
    command_inputs.insert("busybox-root".into(), busybox_root);
    runtime_mounts.insert(busybox_path.clone(), "busybox-root".into());
    // A finite alias catalog grants only the one actual, byte-equal multicall
    // image. Reachability and exit results still come from original source calls.
    for name in [
        "busybox",
        "mkdir",
        "uname",
        "cat",
        "cp",
        "mv",
        "rm",
        "rmdir",
        "touch",
        "chmod",
        "ln",
        "readlink",
        "realpath",
        "install",
        "test",
        "[",
        "cmp",
        "diff",
        "head",
        "tail",
        "cut",
        "sort",
        "uniq",
        "wc",
        "tee",
        "od",
        "cksum",
        "sha256sum",
        "md5sum",
        "sed",
        "awk",
        "grep",
        "tr",
        "expr",
        "printf",
        "echo",
        "basename",
        "dirname",
        "arch",
        "nproc",
        "true",
        "false",
        "yes",
        "sleep",
        "seq",
        "find",
    ] {
        inventory.push((name.into(), busybox.clone()));
    }
    inventory.push(("cmake".into(), cmake_phase));
    inventory.push(("make".into(), native_export(request, "host.make-phase")?));
    let mut interpreter_paths = Vec::new();
    let mut interpreter_environment = BTreeMap::new();
    for (role, names) in [
        ("host.python", &["python3", "python"][..]),
        ("host.perl-phase", &["perl"][..]),
    ] {
        let image = native_export(request, role)?;
        let root = native_export(request, &format!("{role}-root"))?;
        for name in names {
            inventory.push(((*name).into(), image.clone()));
        }
        let alias = role.replace('.', "-");
        let mount = format!("{store}/mantle-{id}-{alias}");
        interpreter_paths.push(format!("{mount}/bin"));
        if role == "host.python" {
            interpreter_environment.insert("PYTHONHOME".into(), mount.clone());
        } else {
            let library = source_constant(&mantle, "FULL_SOURCE_PERL_LIBRARY_RELATIVE_PATH")?;
            interpreter_environment.insert("PERL5LIB".into(), format!("{mount}/{library}"));
        }
        command_inputs.insert(alias.clone(), root);
        runtime_mounts.insert(mount, alias);
    }
    let cc = format!("{}/gcc", logical["phase-tools"]);
    let cxx = format!("{}/g++", logical["phase-tools"]);
    let cmake = format!("{}/bin/cmake", logical["cmake-root"]);
    let make = format!("{}/bin/make", logical["make-root"]);
    let headers = format!("-I{}/include", logical["linux-headers"]);
    let c_base = native
        .arguments()
        .iter()
        .map(|word| render(word, &logical))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let cxx_base = native
        .cxx_arguments()
        .iter()
        .map(|word| render(word, &logical))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let (first, mut options, targets) = if version == route.stages[0].rust_version {
        let (options, targets) = first_configuration(request, &mantle, &cc, &cxx)?;
        (true, options, targets)
    } else {
        (false, successor_configuration(&source, version)?, vec!["install".into()])
    };
    if !first {
        options.extend(configured_overrides(&mantle)?);
    }
    for (key, value) in [
        ("CMAKE_C_COMPILER", cc.clone()),
        ("CMAKE_CXX_COMPILER", cxx.clone()),
        ("CMAKE_ASM_COMPILER", cc.clone()),
        ("CMAKE_AR", format!("{}/ar", logical["phase-tools"])),
        ("CMAKE_RANLIB", format!("{}/ranlib", logical["phase-tools"])),
        ("CMAKE_C_COMPILER_AR", format!("{}/ar", logical["phase-tools"])),
        ("CMAKE_CXX_COMPILER_AR", format!("{}/ar", logical["phase-tools"])),
        ("CMAKE_C_COMPILER_RANLIB", format!("{}/ranlib", logical["phase-tools"])),
        ("CMAKE_CXX_COMPILER_RANLIB", format!("{}/ranlib", logical["phase-tools"])),
        ("CMAKE_MAKE_PROGRAM", make.clone()),
        ("CMAKE_INSTALL_PREFIX", "/llvm/install".into()),
    ] {
        options.insert(key.into(), value);
    }
    let mut script = String::from("set -eu\nmkdir -p /llvm/build /llvm/install /llvm/probe\ncd /llvm/probe\n");
    if !first {
        script.push_str("printf '%s' 'int main(void) { return 0; }' > flag_check.c\nprintf '%s' 'int main(void) { return 0; }' > flag_check.cpp\nC_GZ=\nCXX_GZ=\n");
        for (compiler, extension, flag) in [(&cc, "c", "C_GZ"), (&cxx, "cpp", "CXX_GZ")] {
            script.push_str(&format!("if {} -O0 -ffunction-sections -fdata-sections -fPIC -m64 -gz {} {} -o flag_check -c flag_check.{} 2> probe.err; then\n  if test ! -s probe.err; then {}=-gz; fi\nfi\n",quote(compiler),quote(&headers),if extension=="cpp"{&cxx_base}else{&c_base},extension,flag));
        }
    }
    let c_flags = if first {
        format!("{c_base} {headers}")
    } else {
        format!("-ffunction-sections -fdata-sections -fPIC -m64 $C_GZ {c_base} {headers}")
    };
    let cxx_flags = if first {
        format!(
            "{} -static-libstdc++ -static-libgcc {cxx_base} {headers}",
            source_constant(&mantle, "FIRST_STAGE_CXXFLAGS")?
        )
    } else {
        format!(
            "-ffunction-sections -fdata-sections -fPIC -m64 $CXX_GZ -static-libstdc++ -static-libgcc {cxx_base} {headers}"
        )
    };
    script.push_str("cd /llvm/build\n");
    script.push_str(&quote(&cmake));
    script.push_str(" -G 'Unix Makefiles'");
    for (key, value) in &options {
        script.push(' ');
        script.push_str(&quote(&format!("-D{key}={value}")));
    }
    script.push_str(&format!(
        " \"-DCMAKE_C_FLAGS={}\" \"-DCMAKE_CXX_FLAGS={}\" {}\n{} -f Makefile",
        c_flags,
        cxx_flags,
        quote(&format!("{}/llvm", logical["llvm-source"])),
        quote(&make)
    ));
    for target in targets {
        script.push(' ');
        script.push_str(&quote(&target));
    }
    script.push('\n');
    let script = text(&mut recipe, format!("{id}.script"), script);
    let scripts = assemble(&mut recipe, format!("{id}.scripts"), BTreeMap::from([("llvm.sh".into(), script)]), disk);
    let seed = text(&mut recipe, format!("{id}.seed"), String::new());
    let initial = assemble(
        &mut recipe,
        format!("{id}.initial"),
        BTreeMap::from([
            ("project/.source-phase".into(), seed.clone()),
            ("tmp/.source-phase".into(), seed),
        ]),
        disk,
    );
    let mut roots = String::new();
    netstring(&mut roots, "MANTLE-CONTROLLER-ROOTS-1");
    for value in ["mutable", "/llvm", "project", "mutable", "/tmp", "tmp"] {
        netstring(&mut roots, value);
    }
    for root in runtime_mounts.keys() {
        for value in ["runtime", root.as_str(), root.as_str()] {
            netstring(&mut roots, value);
        }
    }
    let roots = text(&mut recipe, format!("{id}.roots"), roots);
    command_inputs.insert("command-roots".into(), roots.clone());
    let cwd = text(&mut recipe, format!("{id}.cwd"), "/llvm".into());
    let zero = text(&mut recipe, format!("{id}.zero"), "llvm-source-controller".into());
    let mut environment = BTreeMap::from([
        (
            "PATH".into(),
            format!(
                "{busybox_path}/bin:{}/bin:{}/bin:{}:{}",
                logical["make-root"],
                logical["cmake-root"],
                interpreter_paths.join(":"),
                logical["phase-tools"]
            ),
        ),
        ("TMPDIR".into(), "/tmp".into()),
        ("MANTLE_LLVM_CMAKE".into(), cmake),
        ("MANTLE_CMAKE_PROJECT".into(), "/llvm".into()),
        ("LC_ALL".into(), "C".into()),
        ("MANTLE_CONTROLLER_ROOTS".into(), "source-owned-manifest".into()),
    ]);
    environment.extend(interpreter_environment);
    for (key, value) in native.environment() {
        environment.insert(key, render(&value, &logical)?);
    }
    environment.insert("MANTLE_NATIVE_COLLECT2_LINKER".into(), format!("{}/ld", logical["phase-tools"]));
    let mut frame = String::new();
    for (key, value) in environment {
        netstring(&mut frame, &key);
        netstring(&mut frame, &value);
    }
    let environment = text(&mut recipe, format!("{id}.environment"), frame);
    let inputs = BTreeMap::from([
        ("scripts".into(), scripts),
        ("initial-environment".into(), environment),
        ("initial-workspace".into(), initial),
        ("controller-roots".into(), roots),
        ("initial-cwd".into(), cwd),
        ("original-zero".into(), zero),
    ]);
    let start_arguments = vec![
        lit("--mantle-source-shell"),
        lit("run"),
        input("scripts", ""),
        lit("llvm.sh"),
        input("initial-environment", ""),
        lit("-"),
        lit("-"),
        input("initial-workspace", ""),
        output("checkpoint", ""),
    ];
    let policy = RecipeExitPolicy::Normal;
    let slots = vec![
        ControllerSlot {
            id: "llvm-native".into(),
            executable: ControllerExecutable::Inventory(inventory),
            inputs: command_inputs.clone(),
            runtime_mounts: runtime_mounts.clone(),
            exit_policy: policy.clone(),
            merge_stderr: false,
            generated_outputs: true,
        },
        ControllerSlot {
            id: "llvm-generated".into(),
            executable: ControllerExecutable::RequestOutput {
                producer_slots: vec!["llvm-native".into()],
            },
            inputs: command_inputs,
            runtime_mounts,
            exit_policy: policy,
            merge_stderr: false,
            generated_outputs: true,
        },
    ];
    // Reserve the final projection separately; the shared compact controller owns
    // its checked declaration footprint and full source-authored command bound.
    let controller_steps = remaining_steps
        .checked_sub(recipe.steps.len())
        .and_then(|remaining| remaining.checked_sub(1))
        .ok_or_else(|| error("LLVM fixed leaves exceed remaining recipe capacity"))?;
    let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
        reservations,
        &format!("{id}.native"),
        source.native_files,
        controller_steps,
    )?;
    let result = append_controller(&mut recipe, ControllerSpec {
        id: format!("{id}.original"),
        controller: controller.clone(),
        inputs,
        controller_runtime_mounts: BTreeMap::new(),
        start_arguments,
        environment: BTreeMap::from([
            ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
            ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("initial-cwd", "")),
            ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("original-zero", "")),
        ]),
        restart_indices: [5, 6, 8],
        slots,
        commands_max,
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: file_limit,
        tree_bytes_max: disk,
    })?;
    let finish = format!("{id}.products");
    recipe.steps.push(exec(
        finish.clone(),
        collector.clone(),
        BTreeMap::from([
            ("workspace".into(), result.workspace),
            ("protected-products".into(), result.generated),
            ("llvm-source".into(), llvm_source),
        ]),
        vec![
            lit("collect"),
            input("workspace", if first { "project/build" } else { "project/install" }),
            input("llvm-source", "llvm/include"),
            input("protected-products", "project/build/bin/llvm-config"),
            output("root", ""),
            output("llvm-config", ""),
        ],
        BTreeMap::from([
            ("root".into(), spec(ArtifactKind::Tree, disk)),
            ("llvm-config".into(), spec(ArtifactKind::Executable, file_limit)),
        ]),
    ));
    Ok(Fragment {
        steps: recipe.steps,
        files,
        exports: BTreeMap::from([
            ("llvm-root".into(), produced(&finish, "root")),
            ("llvm-config".into(), produced(&finish, "llvm-config")),
            ("llvm-native-controller".into(), controller),
            ("llvm-native-helper".into(), collector),
        ]),
    })
}

/// Original host tools and runtime roots for commands captured by Rust Std.
/// The source caller extends PATH with path_prefix; its compiler/wrapper PATH
/// and all other original environment entries retain their own precedence.
pub(crate) struct NativeCommandWorld {
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub inventory: Vec<(String, ArtifactRef)>,
    pub path_prefix: String,
    pub environment: BTreeMap<String, String>,
}
pub(crate) fn native_command_world(request: &AuthoringRequest<'_>) -> Result<NativeCommandWorld, RunError> {
    let mut world = NativeCommandWorld {
        inputs: BTreeMap::new(),
        runtime_mounts: BTreeMap::new(),
        inventory: Vec::new(),
        path_prefix: String::new(),
        environment: BTreeMap::new(),
    };
    for (name, role) in [
        ("cc", "native.gcc"),
        ("gcc", "native.gcc"),
        ("c++", "native.gxx"),
        ("g++", "native.gxx"),
        ("cc1", "native.cc1"),
        ("cc1plus", "native.cc1plus"),
        ("as", "native.as"),
        ("collect2", "native.collect2-query"),
        ("ld", "native.ld"),
        ("ar", "native.ar"),
        ("ranlib", "native.ranlib"),
        ("nm", "native.nm"),
        ("objcopy", "native.objcopy"),
        ("strip", "native.strip"),
        ("readelf", "native.readelf"),
    ] {
        world.inventory.push((name.into(), native_export(request, role)?));
    }
    let store = request.proof.logical_store_prefix.trim_end_matches('/');
    let mut paths = Vec::new();
    for (role, names) in [
        (
            "host.busybox",
            &[
                "busybox",
                "mkdir",
                "uname",
                "cat",
                "cp",
                "mv",
                "rm",
                "rmdir",
                "touch",
                "chmod",
                "ln",
                "readlink",
                "realpath",
                "install",
                "test",
                "[",
                "cmp",
                "diff",
                "head",
                "tail",
                "cut",
                "sort",
                "uniq",
                "wc",
                "tee",
                "od",
                "cksum",
                "sha256sum",
                "md5sum",
                "sed",
                "awk",
                "grep",
                "tr",
                "expr",
                "printf",
                "echo",
                "basename",
                "dirname",
                "arch",
                "nproc",
                "true",
                "false",
                "yes",
                "sleep",
                "seq",
                "find",
            ][..],
        ),
        ("host.make-phase", &["make"][..]),
        ("host.cmake-phase", &["cmake"][..]),
        ("host.python", &["python3", "python"][..]),
        ("host.perl-phase", &["perl"][..]),
    ] {
        let image = native_export(request, role)?;
        for name in names {
            world.inventory.push(((*name).into(), image.clone()));
        }
        let name = format!("native-command-{}", role.replace('.', "-"));
        let root = format!("{store}/mantle-rust-{name}");
        world.inputs.insert(name.clone(), native_export(request, &format!("{role}-root"))?);
        world.runtime_mounts.insert(root.clone(), name);
        paths.push(format!("{root}/bin"));
        if role == "host.python" {
            world.environment.insert("PYTHONHOME".into(), root);
        } else if role == "host.perl-phase" {
            let mantle = request
                .source_roots
                .get(&crate::source_built_fixed_point::SourceAuthorityRole::MantleSource)
                .ok_or_else(|| error("original Perl policy source missing"))?;
            let source = std::fs::read_to_string(mantle.join("src/rust_source_provider.rs"))
                .map_err(|e| error(e.to_string()))?;
            world.environment.insert(
                "PERL5LIB".into(),
                format!("{root}/{}", source_constant(&source, "FULL_SOURCE_PERL_LIBRARY_RELATIVE_PATH")?),
            );
        }
    }
    world.path_prefix = paths.join(":");
    Ok(world)
}

pub(crate) struct NativeCommandSpec {
    pub id: String,
    /// Shared MANTLE-SHELL-2 directory, including actual cwd/zero and timestamps.
    pub captured_request: ArtifactRef,
    pub captured_request_path: String,
    pub whole_tmp: ArtifactRef,
    /// Data references, including authenticated fixed sources. Recipe admission
    /// validates their paths/digests/kinds; executable inventory stays Produced.
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub readonly_inputs: BTreeSet<String>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub inventory: Vec<(String, ArtifactRef)>,
    pub predicates: Vec<(ArtifactRef, Vec<i32>)>,
    pub commands_max: usize,
}
fn native_predicates(
    inputs: &mut BTreeMap<String, ArtifactRef>,
    predicates: &[(ArtifactRef, Vec<i32>)],
) -> Result<crate::source_built_recipe::RecipeGuard, RunError> {
    let mut conditions = Vec::new();
    for (index, (reference, codes)) in predicates.iter().enumerate() {
        require(reference)?;
        let name = format!("native-command-predicate-{index}");
        if inputs.insert(name.clone(), reference.clone()).is_some() {
            return Err(error("native command predicate input collision"));
        }
        conditions.push(crate::source_built_recipe::RecipeGuardCondition {
            status_input: name,
            accepted_codes: codes.clone(),
        });
    }
    if conditions.is_empty() {
        return Err(error("captured command requires its genuine source classification predicate"));
    }
    Ok(crate::source_built_recipe::RecipeGuard { conditions })
}
/// Return the actual ObservePhase envelope. outputs/native-files is the entire
/// resulting /tmp; outputs/native-request is the shared write_result directory
/// with every descriptor snapshot/alias/offset and genuine normal exit status.
pub(crate) fn append_native_command(
    request: &AuthoringRequest<'_>,
    recipe: &mut SourceRecipe,
    controller: ArtifactRef,
    helper: ArtifactRef,
    specification: NativeCommandSpec,
) -> Result<ArtifactRef, RunError> {
    let NativeCommandSpec {
        id,
        captured_request,
        captured_request_path,
        whole_tmp,
        inputs: mut command_inputs,
        readonly_inputs,
        runtime_mounts,
        inventory,
        predicates,
        commands_max,
    } = specification;
    for reference in [&controller, &helper, &captured_request, &whole_tmp] {
        require(reference)?;
    }
    for (_, reference) in &inventory {
        require(reference)?;
    }
    crate::source_built_recipe::relative(&captured_request_path, true)?;
    let disk = request.proof.resource_bounds.disk_bytes_max;
    let limit = disk.min(64 * 1024 * 1024);
    let roots = native_command_roots(&command_inputs, &readonly_inputs, &runtime_mounts)?;
    let roots = text(recipe, format!("{id}.roots"), roots);
    if command_inputs.insert("command-roots".into(), roots.clone()).is_some() {
        return Err(error("native command roots input collision"));
    }
    let prepare = format!("{id}.entry");
    let mut prepare_inputs = BTreeMap::from([
        ("captured-request".into(), captured_request),
        ("whole-tmp".into(), whole_tmp),
    ]);
    let guard = native_predicates(&mut prepare_inputs, &predicates)?;
    let mut step = exec(
        prepare.clone(),
        helper.clone(),
        prepare_inputs,
        vec![
            lit("native-seed"),
            input("captured-request", &captured_request_path),
            input("whole-tmp", ""),
            output("entry", ""),
        ],
        BTreeMap::from([("entry".into(), spec(ArtifactKind::Tree, disk))]),
    );
    if let RecipeOperation::Exec { guard: binding, .. } = &mut step.operation {
        *binding = Some(guard);
    }
    recipe.steps.push(step);
    let entry = produced(&prepare, "entry");
    let controller_id = format!("{id}.original");
    let inputs = BTreeMap::from([("entry".into(), entry), ("controller-roots".into(), roots)]);
    let start_arguments = vec![
        lit("--mantle-source-shell"),
        lit("run"),
        input("entry", ""),
        lit("native-command"),
        input("entry", "environment"),
        lit("-"),
        lit("-"),
        input("entry", "initial"),
        output("checkpoint", ""),
    ];
    let slots = vec![
        ControllerSlot {
            id: "native-command".into(),
            executable: ControllerExecutable::Inventory(inventory),
            inputs: command_inputs.clone(),
            runtime_mounts: runtime_mounts.clone(),
            exit_policy: RecipeExitPolicy::Normal,
            merge_stderr: false,
            generated_outputs: true,
        },
        ControllerSlot {
            id: "native-generated".into(),
            executable: ControllerExecutable::RequestOutput {
                producer_slots: vec!["native-command".into()],
            },
            inputs: command_inputs,
            runtime_mounts,
            exit_policy: RecipeExitPolicy::Normal,
            merge_stderr: false,
            generated_outputs: true,
        },
    ];
    let result = crate::source_built_stagex_programs::controller::append_guarded_controller(
        recipe,
        ControllerSpec {
            id: controller_id,
            controller,
            inputs,
            controller_runtime_mounts: BTreeMap::new(),
            start_arguments,
            environment: BTreeMap::from([
                ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
                ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("entry", "cwd")),
                ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("entry", "zero")),
                ("MANTLE_NATIVE_CAPTURED_REQUEST".into(), input("entry", "request")),
            ]),
            restart_indices: [5, 6, 8],
            slots,
            commands_max,
            exit_policy: RecipeExitPolicy::Normal,
            file_bytes_max: limit,
            tree_bytes_max: disk,
        },
        predicates.clone(),
    )?;
    let projection = format!("{id}.result");
    let mut inputs = BTreeMap::from([("checkpoint".into(), result.checkpoint)]);
    let guard = native_predicates(&mut inputs, &predicates)?;
    let mut step = exec(
        projection.clone(),
        helper,
        inputs,
        vec![
            lit("native-result"),
            input("checkpoint", ""),
            output("native-files", ""),
            output("native-request", ""),
        ],
        BTreeMap::from([
            ("native-files".into(), spec(ArtifactKind::Tree, disk)),
            ("native-request".into(), spec(ArtifactKind::Tree, disk)),
            ("stdout".into(), spec(ArtifactKind::Object, limit)),
            ("stderr".into(), spec(ArtifactKind::Text, limit)),
            ("exit-status".into(), spec(ArtifactKind::Text, 4)),
        ]),
    );
    if let RecipeOperation::Exec {
        guard: binding,
        exit_policy,
        ..
    } = &mut step.operation
    {
        *binding = Some(guard);
        *exit_policy = RecipeExitPolicy::Normal;
    }
    recipe.steps.push(step);
    let observation = format!("{id}.observe");
    recipe.steps.push(RecipeStep {
        id: observation.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("observation".into(), spec(ArtifactKind::Tree, disk))]),
        operation: RecipeOperation::ObservePhase {
            phase: projection,
            output: "observation".into(),
            state: None,
        },
    });
    Ok(produced(&observation, "observation"))
}

fn native_command_roots(
    inputs: &BTreeMap<String, ArtifactRef>,
    readonly: &BTreeSet<String>,
    runtime: &BTreeMap<String, String>,
) -> Result<String, RunError> {
    let mut roots = String::new();
    netstring(&mut roots, "MANTLE-CONTROLLER-ROOTS-1");
    for value in ["mutable", "/tmp", "tmp"] {
        netstring(&mut roots, value);
    }
    for name in readonly {
        if !inputs.contains_key(name) {
            return Err(error(format!("native command readonly root lacks input {name}")));
        }
        for value in ["input".to_owned(), format!("/inputs/{name}"), name.clone()] {
            netstring(&mut roots, &value);
        }
    }
    for (root, name) in runtime {
        if !inputs.contains_key(name) {
            return Err(error(format!("native command runtime root lacks input {name}")));
        }
        for value in ["runtime", root.as_str(), root.as_str()] {
            netstring(&mut roots, value);
        }
    }
    Ok(roots)
}

/// A source-owned shell program, such as the unchanged original Rust wrapper
/// generator. The shared evaluator interprets it; a shell is never a leaf.
pub(crate) struct NativeSourceScript {
    pub id: String,
    pub script: String,
    pub initial_workspace: ArtifactRef,
    pub initial_cwd: String,
    pub initial_cwd_input: Option<(ArtifactRef, String)>,
    pub original_zero: String,
    pub environment: BTreeMap<String, String>,
    pub environment_input: Option<(ArtifactRef, String)>,
    /// Fixed source data is admitted by the same recipe validation as Produced
    /// data; this map never grants executable inventory authority.
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub readonly_inputs: BTreeSet<String>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub inventory: Vec<(String, ArtifactRef)>,
    pub commands_max: usize,
}
pub(crate) fn append_native_source_script(
    request: &AuthoringRequest<'_>,
    recipe: &mut SourceRecipe,
    controller: ArtifactRef,
    specification: NativeSourceScript,
) -> Result<ArtifactRef, RunError> {
    let NativeSourceScript {
        id,
        script,
        initial_workspace,
        initial_cwd,
        initial_cwd_input,
        original_zero,
        environment,
        environment_input,
        inputs: mut command_inputs,
        readonly_inputs,
        runtime_mounts,
        inventory,
        commands_max,
    } = specification;
    require(&controller)?;
    require(&initial_workspace)?;
    for (_, reference) in &inventory {
        require(reference)?;
    }
    let disk = request.proof.resource_bounds.disk_bytes_max;
    let program = text(recipe, format!("{id}.program"), script);
    let scripts = assemble(recipe, format!("{id}.scripts"), BTreeMap::from([("source.sh".into(), program)]), disk);
    let (environment, environment_path) = if let Some((reference, path)) = environment_input {
        if !environment.is_empty() {
            return Err(error("actual native environment cannot be overridden by static values"));
        }
        require(&reference)?;
        crate::source_built_recipe::relative(&path, true)?;
        (reference, path)
    } else {
        let mut frame = String::new();
        for (key, value) in environment {
            netstring(&mut frame, &key);
            netstring(&mut frame, &value);
        }
        (text(recipe, format!("{id}.environment"), frame), String::new())
    };
    let roots = native_command_roots(&command_inputs, &readonly_inputs, &runtime_mounts)?;
    let roots = text(recipe, format!("{id}.roots"), roots);
    if command_inputs.insert("command-roots".into(), roots.clone()).is_some() {
        return Err(error("native source script roots input collision"));
    }
    let (cwd, cwd_path) = if let Some((reference, path)) = initial_cwd_input {
        if !initial_cwd.is_empty() {
            return Err(error("actual native cwd cannot be overridden"));
        }
        require(&reference)?;
        crate::source_built_recipe::relative(&path, true)?;
        (reference, path)
    } else {
        (text(recipe, format!("{id}.cwd"), initial_cwd), String::new())
    };
    let zero = text(recipe, format!("{id}.zero"), original_zero);
    let inputs = BTreeMap::from([
        ("scripts".into(), scripts),
        ("initial-environment".into(), environment),
        ("initial-workspace".into(), initial_workspace),
        ("controller-roots".into(), roots),
        ("initial-cwd".into(), cwd),
        ("original-zero".into(), zero),
    ]);
    let start_arguments = vec![
        lit("--mantle-source-shell"),
        lit("run"),
        input("scripts", ""),
        lit("source.sh"),
        input("initial-environment", &environment_path),
        lit("-"),
        lit("-"),
        input("initial-workspace", ""),
        output("checkpoint", ""),
    ];
    let slots = vec![
        ControllerSlot {
            id: "native-source".into(),
            executable: ControllerExecutable::Inventory(inventory),
            inputs: command_inputs.clone(),
            runtime_mounts: runtime_mounts.clone(),
            exit_policy: RecipeExitPolicy::Normal,
            merge_stderr: false,
            generated_outputs: true,
        },
        ControllerSlot {
            id: "native-generated".into(),
            executable: ControllerExecutable::RequestOutput {
                producer_slots: vec!["native-source".into()],
            },
            inputs: command_inputs,
            runtime_mounts,
            exit_policy: RecipeExitPolicy::Normal,
            merge_stderr: false,
            generated_outputs: true,
        },
    ];
    let result = append_controller(recipe, ControllerSpec {
        id: format!("{id}.original"),
        controller,
        inputs,
        controller_runtime_mounts: BTreeMap::new(),
        start_arguments,
        environment: BTreeMap::from([
            ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
            ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("initial-cwd", &cwd_path)),
            ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("original-zero", "")),
        ]),
        restart_indices: [5, 6, 8],
        slots,
        commands_max,
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: disk.min(64 * 1024 * 1024),
        tree_bytes_max: disk,
    })?;
    Ok(result.workspace)
}
