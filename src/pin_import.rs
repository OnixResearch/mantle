use std::path::Component;
use std::path::Path;

use serde::Deserialize;

use crate::errors::RunError;

const DEFAULT_PINS_FILE: &str = "nixtamal-pins.json";
const MAX_PIN_IMPORT_FILE_BYTES: u64 = 1_048_576;
const PIN_IMPORT_APPLY_HINT: &str = "Review the plan, resolve blockers, then run `mantle import pins apply`.";

pub const PIN_IMPORTER_NIXTAMAL: &str = crunch_project::PIN_IMPORT_SUPPORTED_IMPORTER;
pub const PIN_IMPORTER_FLAKE: &str = "flake";
pub const PIN_IMPORTER_NPINS: &str = "npins";
pub const PIN_IMPORTER_NIV: &str = "niv";

pub struct PinImportShellOptions<'a> {
    pub root: &'a Path,
    pub importer: &'a str,
    pub pins_file: &'a Path,
    pub project_file: &'a str,
    pub lock_file: &'a str,
    pub inputs_file: &'a str,
    pub apply: bool,
    pub json: bool,
}

impl<'a> PinImportShellOptions<'a> {
    pub fn default_pins_file() -> &'static str {
        DEFAULT_PINS_FILE
    }
}

#[derive(Debug, Deserialize)]
struct NixtamalFixture {
    #[serde(default)]
    unsupported_semantics: Vec<String>,
    #[serde(default)]
    inputs: Vec<NixtamalInput>,
    #[serde(default)]
    patches: Vec<NixtamalPatch>,
}

#[derive(Debug, Deserialize)]
struct NixtamalInput {
    name: String,
    #[serde(alias = "source_kind")]
    kind: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    rev: Option<String>,
    #[serde(default = "default_hash_algo")]
    hash_algo: String,
    #[serde(default, alias = "expected_hash")]
    hash: Option<String>,
    #[serde(default)]
    frozen: bool,
    #[serde(default)]
    mirrors: Vec<String>,
    #[serde(default)]
    patches: Vec<String>,
    #[serde(default)]
    freshness: Option<String>,
    #[serde(default)]
    fetch_policy: Option<String>,
    #[serde(default)]
    trust_policy: Option<String>,
    #[serde(default)]
    composition_semantics: Vec<String>,
    #[serde(default)]
    lock_identity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NixtamalPatch {
    name: String,
    #[serde(alias = "source_kind")]
    kind: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default = "default_hash_algo")]
    hash_algo: String,
    #[serde(default, alias = "expected_hash")]
    hash: Option<String>,
}

pub fn run_pin_import(options: PinImportShellOptions<'_>) -> Result<(), RunError> {
    let import_options = crunch_project::PinImportOptions {
        importer: options.importer.to_string(),
        project_file: options.project_file.to_string(),
        lock_file: options.lock_file.to_string(),
        inputs_file: options.inputs_file.to_string(),
    };
    let pin_set = load_pin_set(options.root, options.pins_file, &import_options)?;
    let plan = crunch_project::build_pin_import_plan(pin_set, import_options);

    if options.apply {
        if !plan.can_apply() {
            emit_pin_import_plan(&plan, false, options.json)?;
            return Err(RunError::Internal(format!(
                "refusing to apply pin import plan with {} blocker(s)",
                plan.blockers.len()
            )));
        }
        apply_pin_import_plan(options.root, &plan)?;
        emit_pin_import_plan(&plan, true, options.json)?;
        return Ok(());
    }

    emit_pin_import_plan(&plan, false, options.json)?;
    if plan.can_apply() {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "pin import plan has {} blocker(s); {PIN_IMPORT_APPLY_HINT}",
        plan.blockers.len()
    )))
}

fn load_pin_set(
    root: &Path,
    pins_file: &Path,
    options: &crunch_project::PinImportOptions,
) -> Result<crunch_project::ExternalPinSet, RunError> {
    if options.importer != PIN_IMPORTER_NIXTAMAL {
        return Ok(future_adapter_pin_set(pins_file, &options.importer));
    }
    let fixture = load_nixtamal_fixture(root, pins_file)?;
    let existing_files = read_existing_files(root, &[&options.project_file, &options.lock_file, &options.inputs_file])?;
    Ok(nixtamal_fixture_to_pin_set(fixture, pins_file, existing_files))
}

fn future_adapter_pin_set(pins_file: &Path, importer: &str) -> crunch_project::ExternalPinSet {
    crunch_project::ExternalPinSet {
        importer: importer.to_string(),
        source_label: pins_file.display().to_string(),
        pins: Vec::new(),
        patches: Vec::new(),
        existing_files: Vec::new(),
        unsupported_semantics: vec!["future adapter is intentionally blocker-only in this change".to_string()],
    }
}

fn load_nixtamal_fixture(root: &Path, pins_file: &Path) -> Result<NixtamalFixture, RunError> {
    let path = root.join(pins_file);
    let metadata = std::fs::metadata(&path)
        .map_err(|err| RunError::Internal(format!("reading {} metadata: {err}", path.display())))?;
    if metadata.len() > MAX_PIN_IMPORT_FILE_BYTES {
        return Err(RunError::Internal(format!("{} exceeds pin import fixture size limit", path.display())));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|err| RunError::Internal(format!("reading Nixtamal fixture {}: {err}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|err| RunError::Internal(format!("parsing Nixtamal fixture {}: {err}", path.display())))
}

fn read_existing_files(root: &Path, paths: &[&str]) -> Result<Vec<crunch_project::ExternalExistingFile>, RunError> {
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        if !is_safe_relative_path(path) {
            continue;
        }
        let full_path = root.join(path);
        if !full_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&full_path)
            .map_err(|err| RunError::Internal(format!("reading existing file {}: {err}", full_path.display())))?;
        files.push(crunch_project::ExternalExistingFile {
            path: (*path).to_string(),
            content,
        });
    }
    Ok(files)
}

fn is_safe_relative_path(path: &str) -> bool {
    let path = Path::new(path);
    if path.is_absolute() {
        return false;
    }
    path.components().all(|component| matches!(component, Component::Normal(_)))
}

fn nixtamal_fixture_to_pin_set(
    fixture: NixtamalFixture,
    pins_file: &Path,
    existing_files: Vec<crunch_project::ExternalExistingFile>,
) -> crunch_project::ExternalPinSet {
    crunch_project::ExternalPinSet {
        importer: PIN_IMPORTER_NIXTAMAL.to_string(),
        source_label: pins_file.display().to_string(),
        pins: fixture.inputs.into_iter().map(nixtamal_input_to_external_pin).collect(),
        patches: fixture.patches.into_iter().map(nixtamal_patch_to_external_patch).collect(),
        existing_files,
        unsupported_semantics: fixture.unsupported_semantics,
    }
}

fn nixtamal_input_to_external_pin(input: NixtamalInput) -> crunch_project::ExternalPin {
    let kind = nixtamal_input_kind(&input);
    crunch_project::ExternalPin {
        name: input.name,
        kind,
        hash: crunch_project::ExternalHash {
            algo: input.hash_algo,
            value: input.hash,
        },
        frozen: input.frozen,
        mirrors: input.mirrors,
        patches: input.patches,
        metadata: crunch_project::ExternalPinMetadata {
            freshness: input.freshness,
            fetch_policy: input.fetch_policy,
            trust_policy: input.trust_policy,
            composition_semantics: input.composition_semantics,
        },
        lock_identity: input.lock_identity,
    }
}

fn nixtamal_input_kind(input: &NixtamalInput) -> crunch_project::ExternalPinKind {
    match input.kind.as_str() {
        "file" => crunch_project::ExternalPinKind::File {
            url: input.url.clone().unwrap_or_default(),
        },
        "tarball" | "archive" => crunch_project::ExternalPinKind::Tarball {
            url: input.url.clone().unwrap_or_default(),
        },
        "git" => crunch_project::ExternalPinKind::Git {
            repository: input.repository.clone().unwrap_or_default(),
            reference: input.reference.clone(),
            rev: input.rev.clone(),
        },
        other => crunch_project::ExternalPinKind::Unsupported {
            kind: other.to_string(),
        },
    }
}

fn nixtamal_patch_to_external_patch(patch: NixtamalPatch) -> crunch_project::ExternalPatch {
    let source = nixtamal_patch_source(&patch);
    crunch_project::ExternalPatch {
        name: patch.name,
        source,
        hash: crunch_project::ExternalHash {
            algo: patch.hash_algo,
            value: patch.hash,
        },
    }
}

fn nixtamal_patch_source(patch: &NixtamalPatch) -> crunch_project::ExternalPatchSource {
    match patch.kind.as_str() {
        "local" => crunch_project::ExternalPatchSource::Local {
            path: patch.path.clone().unwrap_or_default(),
        },
        "remote" => crunch_project::ExternalPatchSource::Remote {
            url: patch.url.clone().unwrap_or_default(),
        },
        other => crunch_project::ExternalPatchSource::Unsupported {
            kind: other.to_string(),
        },
    }
}

fn apply_pin_import_plan(root: &Path, plan: &crunch_project::PinImportPlan) -> Result<(), RunError> {
    if !plan.can_apply() {
        return Err(RunError::Internal("cannot apply a blocked pin import plan".to_string()));
    }
    for operation in &plan.file_operations {
        if !is_safe_relative_path(&operation.path) {
            return Err(RunError::Internal(format!("refusing unsafe planned path {}", operation.path)));
        }
    }
    for operation in &plan.file_operations {
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

fn emit_pin_import_plan(plan: &crunch_project::PinImportPlan, applied: bool, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(plan)
            .map_err(|err| RunError::Internal(format!("rendering pin import plan: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    let mode = if applied { "applied" } else { "plan" };
    println!("pin import {mode}: {} ({})", plan.importer, plan.source_label);
    println!("planned files:");
    for operation in &plan.file_operations {
        println!("  {} {} {}", operation.action, operation.path, operation.digest_blake3);
    }
    println!("mapped inputs: {}", plan.mapped_inputs.len());
    for input in &plan.mapped_inputs {
        println!("  {} {} {}", input.name, input.kind, input.hash_algo);
    }
    println!("mapped patches: {}", plan.mapped_patches.len());
    for patch in &plan.mapped_patches {
        println!("  {} {} {}", patch.name, patch.source_kind, patch.hash_algo);
    }
    if !plan.blockers.is_empty() {
        println!("blockers:");
        for blocker in &plan.blockers {
            println!("  [{}] {}: {}", blocker.class, blocker.subject, blocker.message);
        }
    }
    println!("non-claims:");
    for non_claim in &plan.non_claims {
        println!("  {non_claim}");
    }
    Ok(())
}

fn default_hash_algo() -> String {
    "sha256".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_adapter_pin_set_is_blocker_only() {
        let options = crunch_project::PinImportOptions {
            importer: PIN_IMPORTER_NPINS.to_string(),
            ..crunch_project::PinImportOptions::default()
        };
        let pin_set = load_pin_set(Path::new("."), Path::new("missing.json"), &options).unwrap();
        let plan = crunch_project::build_pin_import_plan(pin_set, options);

        assert!(!plan.can_apply());
        assert!(plan.blockers.iter().any(|blocker| blocker.class == "future-adapter"));
        assert!(plan.file_operations.is_empty());
    }

    #[test]
    fn relative_path_guard_rejects_absolute_and_parent_traversal() {
        assert!(is_safe_relative_path(".mantle/inputs.ncl"));
        assert!(!is_safe_relative_path("/tmp/out"));
        assert!(!is_safe_relative_path("../mantle.lock"));
    }
}
