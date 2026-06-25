use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use toml::Value;

use crate::errors::RunError;

pub const CARGO_IMPORT_PLAN_SCHEMA: &str = "mantle-cargo-import-plan-v1";
const DEFAULT_PROJECT_FILE: &str = "mantle-project.ncl";
const DEFAULT_INPUTS_FILE: &str = ".mantle/inputs.ncl";
const DEFAULT_TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";
const DEFAULT_PROFILE: &str = "release";
const SOURCE_FIELD_SUFFIX: &str = "_src";
const RUST_FIELD_NAME: &str = "rust";
const SEED_FIELD_NAME: &str = "musl_seed_toolchain";
const MUSL_FIELD_NAME: &str = "musl";
const RUST_INPUT_NAME: &str = "rust";
const SEED_INPUT_NAME: &str = "musl-seed-toolchain";
const MUSL_INPUT_NAME: &str = "musl";
const MAX_PACKAGES: usize = 64;
const MAX_BINARIES_PER_PACKAGE: usize = 16;
const MAX_DEPENDENCIES_PER_PACKAGE: usize = 128;
const BLAKE3_HEX_BYTES: usize = 64;
const MAX_NAME_BYTES: usize = 128;
const MIN_TARGET_SEGMENTS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoImportOptions {
    pub selected_package: Option<String>,
    pub selected_binary: Option<String>,
    pub project_file: String,
    pub inputs_file: String,
    pub target_triple: String,
    pub profile: String,
}

impl Default for CargoImportOptions {
    fn default() -> Self {
        Self {
            selected_package: None,
            selected_binary: None,
            project_file: DEFAULT_PROJECT_FILE.to_string(),
            inputs_file: DEFAULT_INPUTS_FILE.to_string(),
            target_triple: DEFAULT_TARGET_TRIPLE.to_string(),
            profile: DEFAULT_PROFILE.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoWorkspaceFacts {
    pub workspace_root: String,
    pub lockfile_digest_blake3: Option<String>,
    pub packages: Vec<CargoPackageFact>,
    pub existing_files: Vec<ExistingProjectFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoPackageFact {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub package_root: String,
    pub binaries: Vec<String>,
    pub dependencies: Vec<CargoDependencyFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoDependencyFact {
    pub name: String,
    pub source: CargoDependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CargoDependencySource {
    LocalPath(String),
    Registry,
    Git(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingProjectFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportPlan {
    pub schema: String,
    pub selected_package: Option<String>,
    pub selected_binary: Option<String>,
    pub target_triple: String,
    pub profile: String,
    pub file_operations: Vec<CargoImportFileOperation>,
    pub source_inputs: Vec<CargoImportSourceInput>,
    pub blockers: Vec<CargoImportBlocker>,
    pub non_claims: Vec<String>,
    pub build_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportFileOperation {
    pub path: String,
    pub action: String,
    pub digest_blake3: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportSourceInput {
    pub role: String,
    pub field: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoImportBlocker {
    pub class: String,
    pub message: String,
}

pub struct CargoImportShellOptions<'a> {
    pub root: &'a Path,
    pub selected_package: Option<&'a str>,
    pub selected_binary: Option<&'a str>,
    pub project_file: &'a str,
    pub inputs_file: &'a str,
    pub target_triple: &'a str,
    pub profile: &'a str,
    pub apply: bool,
    pub json: bool,
}

pub fn run_cargo_import(options: CargoImportShellOptions<'_>) -> Result<(), RunError> {
    let facts = load_workspace_facts(options.root, options.project_file, options.inputs_file)?;
    let plan = build_cargo_import_plan(facts, CargoImportOptions {
        selected_package: options.selected_package.map(ToOwned::to_owned),
        selected_binary: options.selected_binary.map(ToOwned::to_owned),
        project_file: options.project_file.to_string(),
        inputs_file: options.inputs_file.to_string(),
        target_triple: options.target_triple.to_string(),
        profile: options.profile.to_string(),
    });

    if options.apply {
        if !plan.blockers.is_empty() {
            emit_cargo_import_plan(&plan, options.apply, options.json)?;
            return Err(RunError::Internal(format!(
                "refusing to apply cargo import plan with {} blocker(s)",
                plan.blockers.len()
            )));
        }
        apply_cargo_import_plan(options.root, &plan)?;
        emit_cargo_import_plan(&plan, options.apply, options.json)?;
        return Ok(());
    }
    emit_cargo_import_plan(&plan, options.apply, options.json)?;
    if plan.blockers.is_empty() {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "cargo import plan has {} blocker(s); rerun after resolving them",
        plan.blockers.len()
    )))
}

pub fn build_cargo_import_plan(facts: CargoWorkspaceFacts, options: CargoImportOptions) -> CargoImportPlan {
    let mut blockers = Vec::new();
    validate_workspace_shape(&facts, &mut blockers);
    validate_import_options(&options, &mut blockers);

    let selected_package = select_package(&facts, options.selected_package.as_deref(), &mut blockers);
    let selected_binary =
        selected_package.and_then(|package| select_binary(package, options.selected_binary.as_deref(), &mut blockers));
    if let Some(package) = selected_package {
        validate_package_fact(package, &mut blockers);
    }
    validate_lockfile(facts.lockfile_digest_blake3.as_deref(), &mut blockers);

    let mut file_operations = Vec::new();
    let mut source_inputs = Vec::new();
    if blockers.is_empty() {
        let package = selected_package.expect("selected package must exist after blocker-free selection");
        let binary = selected_binary.expect("selected binary must exist after blocker-free selection");
        source_inputs = source_inputs_for(package);
        file_operations = generated_file_operations(package, binary, &options, &source_inputs);
        validate_file_conflicts(&mut file_operations, &facts.existing_files, &mut blockers);
    }

    CargoImportPlan {
        schema: CARGO_IMPORT_PLAN_SCHEMA.to_string(),
        selected_package: selected_package.map(|package| package.name.clone()),
        selected_binary: selected_binary.map(ToOwned::to_owned),
        target_triple: options.target_triple,
        profile: options.profile,
        file_operations,
        source_inputs,
        blockers,
        non_claims: vec![
            "not-cargo-free-execution".to_string(),
            "not-module-layer-semantics".to_string(),
            "not-network-vendoring".to_string(),
        ],
        build_hints: vec![
            "Review the plan, materialize declared source/toolchain inputs, then run `mantle import cargo --apply`."
                .to_string(),
        ],
    }
}

fn validate_workspace_shape(facts: &CargoWorkspaceFacts, blockers: &mut Vec<CargoImportBlocker>) {
    if facts.workspace_root.is_empty() {
        blockers.push(blocker("invalid-workspace-root", "workspace root must be valid UTF-8"));
    }
    if facts.packages.is_empty() {
        blockers.push(blocker("missing-package", "Cargo import requires at least one package"));
    }
    if facts.packages.len() > MAX_PACKAGES {
        blockers.push(blocker("too-many-packages", "Cargo import initial surface supports a bounded package count"));
    }
}

fn validate_import_options(options: &CargoImportOptions, blockers: &mut Vec<CargoImportBlocker>) {
    validate_relative_output_path(&options.project_file, "invalid-project-file", blockers);
    validate_relative_output_path(&options.inputs_file, "invalid-inputs-file", blockers);
    if options.target_triple.split('-').filter(|segment| !segment.is_empty()).count() < MIN_TARGET_SEGMENTS {
        blockers.push(blocker("unsupported-target-triple", "target triple must be explicit"));
    }
    if !matches!(options.profile.as_str(), "debug" | "release") {
        blockers.push(blocker("unsupported-profile", "import scaffold supports debug or release profiles"));
    }
}

fn validate_relative_output_path(path: &str, class: &str, blockers: &mut Vec<CargoImportBlocker>) {
    if path.is_empty() || path.starts_with('/') || path.split('/').any(|part| part == ".." || part.is_empty()) {
        blockers.push(blocker(class, "generated file path must be a relative path without parent traversal"));
    }
}

fn select_package<'a>(
    facts: &'a CargoWorkspaceFacts,
    selected_package: Option<&str>,
    blockers: &mut Vec<CargoImportBlocker>,
) -> Option<&'a CargoPackageFact> {
    if let Some(name) = selected_package {
        let matches: Vec<&CargoPackageFact> = facts.packages.iter().filter(|package| package.name == name).collect();
        if matches.len() == 1 {
            return matches.first().copied();
        }
        blockers.push(blocker("unknown-package", &format!("selected package `{name}` is not in the workspace")));
        return None;
    }
    if facts.packages.len() == 1 {
        return facts.packages.first();
    }
    blockers.push(blocker("ambiguous-default-package", "workspace has multiple packages; select one with --package"));
    None
}

fn select_binary<'a>(
    package: &'a CargoPackageFact,
    selected_binary: Option<&'a str>,
    blockers: &mut Vec<CargoImportBlocker>,
) -> Option<&'a str> {
    if let Some(name) = selected_binary {
        if package.binaries.iter().any(|binary| binary == name) {
            return Some(name);
        }
        blockers.push(blocker(
            "missing-selected-binary",
            &format!("selected binary `{name}` is not declared by package `{}`", package.name),
        ));
        return None;
    }
    if package.binaries.len() == 1 {
        return package.binaries.first().map(String::as_str);
    }
    if package.binaries.is_empty() {
        blockers.push(blocker("missing-selected-binary", "package has no supported binary target"));
    } else {
        blockers.push(blocker(
            "ambiguous-binary-selection",
            "package has multiple binary targets; select one with --binary",
        ));
    }
    None
}

fn validate_package_fact(package: &CargoPackageFact, blockers: &mut Vec<CargoImportBlocker>) {
    validate_name("package name", &package.name, "malformed-package-name", blockers);
    for binary in &package.binaries {
        validate_name("binary name", binary, "malformed-binary-name", blockers);
    }
    if package.binaries.len() > MAX_BINARIES_PER_PACKAGE {
        blockers.push(blocker("too-many-binaries", "package has too many binary targets for initial import"));
    }
    if package.dependencies.len() > MAX_DEPENDENCIES_PER_PACKAGE {
        blockers.push(blocker("too-many-dependencies", "package dependency count exceeds initial import bound"));
    }
    for dependency in &package.dependencies {
        validate_dependency_source(dependency, blockers);
    }
}

fn validate_dependency_source(dependency: &CargoDependencyFact, blockers: &mut Vec<CargoImportBlocker>) {
    match &dependency.source {
        CargoDependencySource::LocalPath(_) => {}
        CargoDependencySource::Registry => blockers.push(blocker(
            "unsupported-dependency-source",
            &format!("dependency `{}` requires undeclared registry/vendor material", dependency.name),
        )),
        CargoDependencySource::Git(url) => blockers.push(blocker(
            "unsupported-dependency-source",
            &format!("dependency `{}` requires undeclared git source `{url}`", dependency.name),
        )),
        CargoDependencySource::Unknown => blockers.push(blocker(
            "unsupported-dependency-source",
            &format!("dependency `{}` has an unsupported source declaration", dependency.name),
        )),
    }
}

fn validate_lockfile(digest: Option<&str>, blockers: &mut Vec<CargoImportBlocker>) {
    let Some(digest) = digest else {
        blockers.push(blocker("missing-lockfile", "Cargo.lock is required for offline Cargo import"));
        return;
    };
    if digest.len() == BLAKE3_HEX_BYTES && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return;
    }
    blockers.push(blocker("invalid-lockfile-digest", "Cargo.lock digest must be BLAKE3 hex"));
}

fn source_inputs_for(package: &CargoPackageFact) -> Vec<CargoImportSourceInput> {
    let source_field = source_field_name(&package.name);
    vec![
        CargoImportSourceInput {
            role: "package-source".to_string(),
            field: source_field,
            name: format!("{}-src", package.name),
        },
        CargoImportSourceInput {
            role: "rust-toolchain".to_string(),
            field: RUST_FIELD_NAME.to_string(),
            name: RUST_INPUT_NAME.to_string(),
        },
        CargoImportSourceInput {
            role: "seed-toolchain".to_string(),
            field: SEED_FIELD_NAME.to_string(),
            name: SEED_INPUT_NAME.to_string(),
        },
        CargoImportSourceInput {
            role: "musl-runtime".to_string(),
            field: MUSL_FIELD_NAME.to_string(),
            name: MUSL_INPUT_NAME.to_string(),
        },
    ]
}

fn generated_file_operations(
    package: &CargoPackageFact,
    binary: &str,
    options: &CargoImportOptions,
    source_inputs: &[CargoImportSourceInput],
) -> Vec<CargoImportFileOperation> {
    let project = render_project_ncl(package, binary, options, source_inputs);
    let inputs = render_inputs_ncl(source_inputs);
    vec![
        file_operation(&options.project_file, project),
        file_operation(&options.inputs_file, inputs),
    ]
}

fn file_operation(path: &str, content: String) -> CargoImportFileOperation {
    CargoImportFileOperation {
        path: path.to_string(),
        action: "write".to_string(),
        digest_blake3: blake3_hex(content.as_bytes()),
        content,
    }
}

fn validate_file_conflicts(
    operations: &mut [CargoImportFileOperation],
    existing_files: &[ExistingProjectFile],
    blockers: &mut Vec<CargoImportBlocker>,
) {
    for operation in operations {
        let Some(existing) = existing_files.iter().find(|existing| existing.path == operation.path) else {
            continue;
        };
        if existing.content == operation.content {
            operation.action = "keep-equivalent".to_string();
            continue;
        }
        blockers.push(blocker("existing-file-conflict", &format!("{} exists with different content", operation.path)));
    }
}

fn render_project_ncl(
    package: &CargoPackageFact,
    binary: &str,
    options: &CargoImportOptions,
    source_inputs: &[CargoImportSourceInput],
) -> String {
    let package_field = nickel_field(&package.name);
    let source = source_inputs.iter().find(|input| input.role == "package-source").expect("source input exists");
    format!(
        r#"# Generated by `mantle import cargo --apply`.
# Build-shaped handoff: Cargo runs inside Mantle's sandbox; this is not a module layer.
let mantle = import "lib.ncl" in
let inputs = import {inputs_file} in
{{
  packages.{package_field} = mantle.offlineCargoPackage {{
    name = {package_name},
    src = inputs.{source_field},
    source_name = {source_name},
    rust = inputs.{rust_field},
    rust_name = {rust_name},
    seed_toolchain = inputs.{seed_field},
    seed_toolchain_name = {seed_name},
    musl = inputs.{musl_field},
    musl_name = {musl_name},
    binary = {binary},
    target = {target},
    profile = '{profile},
  }},
  default.package = {package_name},
}} | mantle.Project
"#,
        inputs_file = nickel_string(&options.inputs_file),
        package_field = package_field,
        package_name = nickel_string(&package.name),
        source_field = nickel_field(&source.field),
        source_name = nickel_string(&source.name),
        rust_field = nickel_field(RUST_FIELD_NAME),
        rust_name = nickel_string(RUST_INPUT_NAME),
        seed_field = nickel_field(SEED_FIELD_NAME),
        seed_name = nickel_string(SEED_INPUT_NAME),
        musl_field = nickel_field(MUSL_FIELD_NAME),
        musl_name = nickel_string(MUSL_INPUT_NAME),
        binary = nickel_string(binary),
        target = nickel_string(&options.target_triple),
        profile = options.profile,
    )
}

fn render_inputs_ncl(source_inputs: &[CargoImportSourceInput]) -> String {
    let mut fields = String::new();
    for input in source_inputs {
        fields.push_str(&format!(
            "  {} = placeholder {} {},\n",
            nickel_field(&input.field),
            nickel_string(&input.name),
            nickel_string(&format!("materialize `{}` input `{}` before running mantle build", input.role, input.name))
        ));
    }
    format!(
        r#"# Generated by `mantle import cargo --apply`.
# Replace placeholders with admitted source/toolchain derivations before building.
let mantle = import "lib.ncl" in
let placeholder = fun artifact_name message => {{
  name = artifact_name,
  builder = "/bin/sh",
  args = ["-c", "echo \"$MANTLE_PLACEHOLDER_MESSAGE\" >&2; exit 1"],
  env = {{ MANTLE_PLACEHOLDER_MESSAGE = message }},
  addressing_mode = 'input-addressed,
}} | mantle.Derivation in
{{
{fields}}}
"#
    )
}

fn source_field_name(package_name: &str) -> String {
    let mut field = package_name.replace('-', "_");
    field.push_str(SOURCE_FIELD_SUFFIX);
    field
}

fn validate_name(label: &str, value: &str, class: &str, blockers: &mut Vec<CargoImportBlocker>) {
    if is_valid_derivation_name(value) {
        return;
    }
    blockers.push(blocker(class, &format!("{label} must be a non-empty derivation-compatible name")));
}

fn is_valid_derivation_name(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_NAME_BYTES {
        return false;
    }
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'_' | b'?' | b'=' | b'-'))
}

fn nickel_field(value: &str) -> String {
    if is_nickel_identifier(value) {
        return value.to_string();
    }
    nickel_string(value)
}

fn is_nickel_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn nickel_string(value: &str) -> String {
    serde_json::to_string(value).expect("JSON string serialization cannot fail")
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn blocker(class: &str, message: &str) -> CargoImportBlocker {
    CargoImportBlocker {
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn load_workspace_facts(root: &Path, project_file: &str, inputs_file: &str) -> Result<CargoWorkspaceFacts, RunError> {
    let root_text = root
        .to_str()
        .ok_or_else(|| RunError::Internal("cargo import requires a UTF-8 workspace root path".to_string()))?
        .to_string();
    let manifest_path = root.join("Cargo.toml");
    let manifest = read_toml_value(&manifest_path)?;
    let lock_digest = read_lock_digest(root);
    let package_paths = package_manifest_paths(root, &manifest)?;
    let mut packages = Vec::with_capacity(package_paths.len());
    for package_path in package_paths {
        packages.push(load_package_fact(&package_path)?);
    }
    let existing_files = read_existing_files(root, &[project_file, inputs_file])?;
    Ok(CargoWorkspaceFacts {
        workspace_root: root_text,
        lockfile_digest_blake3: lock_digest,
        packages,
        existing_files,
    })
}

fn read_toml_value(path: &Path) -> Result<Value, RunError> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    text.parse::<Value>()
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))
}

fn read_lock_digest(root: &Path) -> Option<String> {
    std::fs::read(root.join("Cargo.lock")).ok().map(|bytes| blake3_hex(&bytes))
}

fn package_manifest_paths(root: &Path, manifest: &Value) -> Result<Vec<PathBuf>, RunError> {
    if manifest.get("package").is_some() {
        return Ok(vec![root.join("Cargo.toml")]);
    }
    let Some(workspace) = manifest.get("workspace").and_then(Value::as_table) else {
        return Ok(Vec::new());
    };
    let members = workspace.get("members").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut paths = Vec::with_capacity(members.len());
    for member in members {
        let Some(member_path) = member.as_str() else {
            continue;
        };
        paths.push(root.join(member_path).join("Cargo.toml"));
    }
    Ok(paths)
}

fn load_package_fact(manifest_path: &Path) -> Result<CargoPackageFact, RunError> {
    let manifest = read_toml_value(manifest_path)?;
    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .ok_or_else(|| RunError::Internal(format!("{} has no [package] table", manifest_path.display())))?;
    let name = required_string(package.get("name"), "package.name", manifest_path)?;
    let version = optional_string(package.get("version")).unwrap_or_else(|| "0.0.0".to_string());
    let edition = optional_string(package.get("edition")).unwrap_or_else(|| "2021".to_string());
    let package_root = manifest_path
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| RunError::Internal("package root must be UTF-8".to_string()))?
        .to_string();
    let binaries = package_binaries(manifest_path, &manifest, &name);
    let dependencies = package_dependencies(&manifest);
    Ok(CargoPackageFact {
        name,
        version,
        edition,
        package_root,
        binaries,
        dependencies,
    })
}

fn required_string(value: Option<&Value>, label: &str, manifest_path: &Path) -> Result<String, RunError> {
    value
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| RunError::Internal(format!("{} must contain string field {label}", manifest_path.display())))
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Table(table) => table
            .get("workspace")
            .and_then(Value::as_bool)
            .and_then(|workspace| if workspace { Some("2021".to_string()) } else { None }),
        _ => None,
    }
}

fn package_binaries(manifest_path: &Path, manifest: &Value, package_name: &str) -> Vec<String> {
    let mut binaries = Vec::new();
    if let Some(bin_entries) = manifest.get("bin").and_then(Value::as_array) {
        for entry in bin_entries {
            if let Some(name) = entry.get("name").and_then(Value::as_str) {
                binaries.push(name.to_string());
            }
        }
    }
    if binaries.is_empty() {
        let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        if root.join("src/main.rs").is_file() {
            binaries.push(package_name.to_string());
        }
    }
    binaries.sort();
    binaries.dedup();
    binaries
}

fn package_dependencies(manifest: &Value) -> Vec<CargoDependencyFact> {
    let mut dependencies = Vec::new();
    collect_dependency_table(manifest.get("dependencies"), &mut dependencies);
    collect_dependency_table(manifest.get("build-dependencies"), &mut dependencies);
    collect_dependency_table(manifest.get("dev-dependencies"), &mut dependencies);
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    dependencies
}

fn collect_dependency_table(value: Option<&Value>, dependencies: &mut Vec<CargoDependencyFact>) {
    let Some(table) = value.and_then(Value::as_table) else {
        return;
    };
    for (name, value) in table {
        dependencies.push(CargoDependencyFact {
            name: name.clone(),
            source: dependency_source(value),
        });
    }
}

fn dependency_source(value: &Value) -> CargoDependencySource {
    match value {
        Value::String(_) => CargoDependencySource::Registry,
        Value::Table(table) => {
            if let Some(path) = table.get("path").and_then(Value::as_str) {
                return CargoDependencySource::LocalPath(path.to_string());
            }
            if let Some(git) = table.get("git").and_then(Value::as_str) {
                return CargoDependencySource::Git(git.to_string());
            }
            if table.get("version").is_some() {
                return CargoDependencySource::Registry;
            }
            CargoDependencySource::Unknown
        }
        _ => CargoDependencySource::Unknown,
    }
}

fn read_existing_files(root: &Path, paths: &[&str]) -> Result<Vec<ExistingProjectFile>, RunError> {
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let absolute = root.join(path);
        if !absolute.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&absolute)
            .map_err(|err| RunError::Internal(format!("reading {}: {err}", absolute.display())))?;
        files.push(ExistingProjectFile {
            path: (*path).to_string(),
            content,
        });
    }
    Ok(files)
}

fn apply_cargo_import_plan(root: &Path, plan: &CargoImportPlan) -> Result<(), RunError> {
    if !plan.blockers.is_empty() {
        return Err(RunError::Internal(format!(
            "refusing to apply cargo import plan with {} blocker(s)",
            plan.blockers.len()
        )));
    }
    for operation in &plan.file_operations {
        if operation.action == "keep-equivalent" {
            continue;
        }
        let path = root.join(&operation.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
        }
        std::fs::write(&path, &operation.content)
            .map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))?;
    }
    Ok(())
}

fn emit_cargo_import_plan(plan: &CargoImportPlan, applied: bool, json: bool) -> Result<(), RunError> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(plan)
                .map_err(|err| RunError::Internal(format!("serializing cargo import plan: {err}")))?
        );
        return Ok(());
    }
    if applied {
        println!("Applied Cargo import scaffold for {:?}", plan.selected_package);
    } else {
        println!("Cargo import plan for {:?}", plan.selected_package);
    }
    println!("  blockers: {}", plan.blockers.len());
    for blocker in &plan.blockers {
        println!("  - {}: {}", blocker.class, blocker.message);
    }
    for operation in &plan.file_operations {
        println!("  {} {} ({})", operation.action, operation.path, operation.digest_blake3);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn package(name: &str, binaries: &[&str], dependencies: Vec<CargoDependencyFact>) -> CargoPackageFact {
        CargoPackageFact {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            package_root: "/tmp/demo".to_string(),
            binaries: binaries.iter().map(|binary| (*binary).to_string()).collect(),
            dependencies,
        }
    }

    fn local_dependency(name: &str) -> CargoDependencyFact {
        CargoDependencyFact {
            name: name.to_string(),
            source: CargoDependencySource::LocalPath("../dep".to_string()),
        }
    }

    fn registry_dependency(name: &str) -> CargoDependencyFact {
        CargoDependencyFact {
            name: name.to_string(),
            source: CargoDependencySource::Registry,
        }
    }

    fn facts(packages: Vec<CargoPackageFact>) -> CargoWorkspaceFacts {
        CargoWorkspaceFacts {
            workspace_root: "/tmp/workspace".to_string(),
            lockfile_digest_blake3: Some(LOCK_DIGEST.to_string()),
            packages,
            existing_files: Vec::new(),
        }
    }

    #[test]
    fn cargo_import_plan_generates_build_shaped_files_for_local_binary_workspace() {
        let plan = build_cargo_import_plan(
            facts(vec![package("demo", &["demo"], vec![local_dependency("dep")])]),
            CargoImportOptions::default(),
        );

        assert!(plan.blockers.is_empty(), "blockers: {:#?}", plan.blockers);
        assert_eq!(plan.selected_package.as_deref(), Some("demo"));
        assert_eq!(plan.selected_binary.as_deref(), Some("demo"));
        assert_eq!(plan.file_operations.len(), 2);
        assert!(plan.file_operations[0].content.contains("mantle.offlineCargoPackage"));
        assert!(plan.file_operations[0].content.contains("not a module layer"));
        assert_eq!(plan.file_operations[0].digest_blake3.len(), BLAKE3_HEX_BYTES);
        assert!(plan.non_claims.contains(&"not-cargo-free-execution".to_string()));
    }

    #[test]
    fn cargo_import_plan_blocks_unsupported_and_ambiguous_surfaces() {
        let plan = build_cargo_import_plan(
            facts(vec![
                package("demo", &["demo", "other"], vec![registry_dependency("serde")]),
                package("second", &["second"], Vec::new()),
            ]),
            CargoImportOptions::default(),
        );
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("ambiguous-default-package"), "classes: {classes:?}");
        assert!(plan.file_operations.is_empty());
        assert!(plan.source_inputs.is_empty());
    }

    #[test]
    fn cargo_import_plan_blocks_missing_lockfile_malformed_name_and_registry_dependency() {
        let mut workspace = facts(vec![package("bad/name", &["bad/name"], vec![registry_dependency("serde")])]);
        workspace.lockfile_digest_blake3 = None;
        let plan = build_cargo_import_plan(workspace, CargoImportOptions {
            selected_package: Some("bad/name".to_string()),
            selected_binary: Some("bad/name".to_string()),
            ..CargoImportOptions::default()
        });
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("missing-lockfile"), "classes: {classes:?}");
        assert!(classes.contains("malformed-package-name"), "classes: {classes:?}");
        assert!(classes.contains("malformed-binary-name"), "classes: {classes:?}");
        assert!(classes.contains("unsupported-dependency-source"), "classes: {classes:?}");
    }

    #[test]
    fn cargo_import_plan_reports_existing_file_conflicts_without_applying() {
        let mut workspace = facts(vec![package("demo", &["demo"], Vec::new())]);
        workspace.existing_files.push(ExistingProjectFile {
            path: DEFAULT_PROJECT_FILE.to_string(),
            content: "# user content\n".to_string(),
        });
        let plan = build_cargo_import_plan(workspace, CargoImportOptions::default());
        let classes = plan.blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("existing-file-conflict"), "classes: {classes:?}");
        assert_eq!(plan.file_operations.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn cargo_import_shell_rejects_non_utf8_workspace_roots() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let invalid_root = PathBuf::from(OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xff]));
        let err = run_cargo_import(CargoImportShellOptions {
            root: &invalid_root,
            selected_package: None,
            selected_binary: None,
            project_file: DEFAULT_PROJECT_FILE,
            inputs_file: DEFAULT_INPUTS_FILE,
            target_triple: DEFAULT_TARGET_TRIPLE,
            profile: DEFAULT_PROFILE,
            apply: false,
            json: true,
        })
        .unwrap_err();

        assert!(err.to_string().contains("UTF-8 workspace root"));
    }
}
