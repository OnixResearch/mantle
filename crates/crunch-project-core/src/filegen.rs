use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

pub const FILEGEN_PLAN_SCHEMA: &str = "mantle-project-filegen-plan-v1";
pub const FILEGEN_NON_CLAIM: &str = "file generation evidence proves only declared content/materialization planning; it does not prove deployability, services, frontend correctness, or build success";

const MAX_FILEGEN_DECLARATIONS: u32 = 4096;
const MAX_FILEGEN_TARGET_BYTES: usize = 4096;
const MAX_CONTRACT_FIELDS: u32 = 256;
const BLAKE3_HEX_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedFileDeclaration {
    pub name: String,
    pub target: String,
    pub content: GeneratedFileContent,
    #[serde(default = "default_materialization")]
    pub materialization: GeneratedFileMaterialization,
    pub contract: Option<GeneratedFileContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GeneratedFileContent {
    #[serde(rename = "inline")]
    Inline { text: String },
    #[serde(rename = "nickel-export")]
    NickelExport {
        text: String,
        receipt_digest_blake3: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedFileMaterialization {
    Copy,
    Symlink,
}

fn default_materialization() -> GeneratedFileMaterialization {
    GeneratedFileMaterialization::Copy
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedFileContract {
    pub identity: String,
    pub required_json_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentFileFact {
    pub target: String,
    pub state: CurrentFileState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum CurrentFileState {
    #[serde(rename = "missing")]
    Missing,
    #[serde(rename = "managed")]
    Managed { digest_blake3: String },
    #[serde(rename = "unmanaged")]
    Unmanaged { digest_blake3: String },
    #[serde(rename = "symlink")]
    Symlink { target: String, managed: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilegenPlanRequest {
    pub declarations: Vec<GeneratedFileDeclaration>,
    pub current_files: Vec<CurrentFileFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilegenPlan {
    pub schema: String,
    pub operations: Vec<FilegenOperation>,
    pub blockers: Vec<FilegenBlocker>,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilegenOperation {
    pub name: String,
    pub target: String,
    pub action: FilegenAction,
    pub materialization: GeneratedFileMaterialization,
    pub desired_digest_blake3: String,
    pub content: String,
    pub contract_identity: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilegenAction {
    Create,
    Update,
    Unchanged,
    Stale,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilegenBlocker {
    pub code: String,
    pub target: String,
    pub message: String,
}

pub fn plan_file_generation(request: FilegenPlanRequest) -> FilegenPlan {
    let mut blockers = Vec::new();
    let declaration_count = request.declarations.len() as u64;
    let current_fact_count = request.current_files.len() as u64;
    if declaration_count > u64::from(MAX_FILEGEN_DECLARATIONS) {
        blockers.push(blocker(
            "too-many-generated-files",
            "<manifest>",
            format!("too many generated file declarations: {declaration_count} > {MAX_FILEGEN_DECLARATIONS}"),
        ));
    }
    if current_fact_count > u64::from(MAX_FILEGEN_DECLARATIONS) {
        blockers.push(blocker(
            "too-many-current-file-facts",
            "<state>",
            format!("too many generated file state facts: {current_fact_count} > {MAX_FILEGEN_DECLARATIONS}"),
        ));
    }
    if !blockers.is_empty() {
        return FilegenPlan {
            schema: FILEGEN_PLAN_SCHEMA.to_string(),
            operations: Vec::new(),
            blockers,
            non_claim: FILEGEN_NON_CLAIM.to_string(),
        };
    }

    let mut operations = Vec::with_capacity(request.declarations.len().saturating_add(request.current_files.len()));
    let current_files = normalized_current_files(&request.current_files, &mut blockers);
    let mut declared_targets = BTreeSet::new();

    for declaration in &request.declarations {
        plan_declaration(declaration, &current_files, &mut declared_targets, &mut operations, &mut blockers);
    }
    append_stale_operations(&current_files, &declared_targets, &mut operations);

    let plan = FilegenPlan {
        schema: FILEGEN_PLAN_SCHEMA.to_string(),
        operations,
        blockers,
        non_claim: FILEGEN_NON_CLAIM.to_string(),
    };
    debug_assert_eq!(plan.schema, FILEGEN_PLAN_SCHEMA);
    debug_assert_eq!(plan.non_claim, FILEGEN_NON_CLAIM);
    plan
}

pub fn verify_filegen_apply_plan(reviewed: &FilegenPlan, current: &FilegenPlan) -> Result<(), Vec<FilegenBlocker>> {
    assert_eq!(current.schema, FILEGEN_PLAN_SCHEMA, "current plan schema must be current");
    debug_assert_eq!(current.non_claim, FILEGEN_NON_CLAIM);
    if reviewed.schema != FILEGEN_PLAN_SCHEMA {
        return Err(vec![blocker(
            "unsupported-reviewed-plan-schema",
            "<plan>",
            format!("reviewed plan schema `{}` is unsupported; expected `{FILEGEN_PLAN_SCHEMA}`", reviewed.schema),
        )]);
    }

    if !current.blockers.is_empty() {
        return Err(current.blockers.clone());
    }
    if !reviewed.blockers.is_empty() {
        return Err(reviewed.blockers.clone());
    }
    if reviewed.operations != current.operations {
        return Err(vec![blocker(
            "plan-drift",
            "<plan>",
            "current generated-file facts differ from the reviewed plan".to_string(),
        )]);
    }
    Ok(())
}

impl FilegenPlan {
    pub fn has_blockers(&self) -> bool {
        !self.blockers.is_empty()
    }
}

fn normalized_current_files(
    facts: &[CurrentFileFact],
    blockers: &mut Vec<FilegenBlocker>,
) -> BTreeMap<String, CurrentFileState> {
    let mut map = BTreeMap::new();
    for fact in facts {
        match normalize_target(&fact.target) {
            Ok(target) => {
                debug_assert!(!target.is_empty());
                if map.insert(target.clone(), fact.state.clone()).is_some() {
                    blockers.push(blocker(
                        "duplicate-current-file-fact",
                        &target,
                        format!("duplicate current file fact for `{target}`"),
                    ));
                }
            }
            Err(error) => {
                blockers.push(blocker("unsafe-current-file-target", &fact.target, error.message().to_string()))
            }
        }
    }
    // Duplicate facts collapse into one entry, so the map never exceeds the input.
    debug_assert!(map.len() <= facts.len());
    map
}

fn plan_declaration(
    declaration: &GeneratedFileDeclaration,
    current_files: &BTreeMap<String, CurrentFileState>,
    declared_targets: &mut BTreeSet<String>,
    operations: &mut Vec<FilegenOperation>,
    blockers: &mut Vec<FilegenBlocker>,
) {
    debug_assert!(declared_targets.len() as u64 <= u64::from(MAX_FILEGEN_DECLARATIONS));
    debug_assert!(operations.len() as u64 <= u64::from(MAX_FILEGEN_DECLARATIONS));
    if declaration.name.is_empty() {
        blockers.push(blocker(
            "empty-generated-file-name",
            &declaration.target,
            "generated file name must not be empty".to_string(),
        ));
    }
    let target = match normalize_target(&declaration.target) {
        Ok(target) => target,
        Err(error) => {
            blockers.push(blocker("target-escape", &declaration.target, error.message().to_string()));
            return;
        }
    };
    if !declared_targets.insert(target.clone()) {
        blockers.push(blocker(
            "duplicate-generated-file-target",
            &target,
            format!("multiple generated files target `{target}`"),
        ));
        return;
    }
    if let Err(error) = validate_contract(declaration) {
        blockers.push(blocker("contract-invalid-content", &target, error.message().to_string()));
        return;
    }
    let content = declaration_content(&declaration.content);
    let desired_digest = match desired_digest(declaration) {
        Ok(digest) => digest,
        Err(error) => {
            blockers.push(blocker("invalid-export-receipt-digest", &target, error.message().to_string()));
            return;
        }
    };
    let action = classify_action(current_files.get(&target), &desired_digest, declaration.materialization, &content);
    if action == FilegenAction::Conflict {
        blockers.push(blocker(
            "existing-file-conflict",
            &target,
            format!("generated file target `{target}` collides with unmanaged content"),
        ));
    }
    operations.push(FilegenOperation {
        name: declaration.name.clone(),
        target,
        action,
        materialization: declaration.materialization,
        desired_digest_blake3: desired_digest,
        content,
        contract_identity: declaration.contract.as_ref().map(|contract| contract.identity.clone()),
    });
}

fn append_stale_operations(
    current_files: &BTreeMap<String, CurrentFileState>,
    declared_targets: &BTreeSet<String>,
    operations: &mut Vec<FilegenOperation>,
) {
    for (target, state) in current_files {
        if declared_targets.contains(target) {
            continue;
        }
        if let CurrentFileState::Managed { digest_blake3 } = state {
            operations.push(FilegenOperation {
                name: "<stale>".to_string(),
                target: target.clone(),
                action: FilegenAction::Stale,
                materialization: GeneratedFileMaterialization::Copy,
                desired_digest_blake3: digest_blake3.clone(),
                content: String::new(),
                contract_identity: None,
            });
        }
    }
}

fn normalize_target(target: &str) -> Result<String, Error> {
    if target.is_empty() {
        return Err(Error::Validation("generated file target must not be empty".to_string()));
    }
    if target.len() > MAX_FILEGEN_TARGET_BYTES {
        return Err(Error::Validation(format!("generated file target exceeds {MAX_FILEGEN_TARGET_BYTES} bytes")));
    }
    if target.starts_with('/') {
        return Err(Error::Validation(format!(
            "generated file target `{target}` must be relative to the project root"
        )));
    }
    debug_assert!(!target.is_empty());
    debug_assert!(target.len() <= MAX_FILEGEN_TARGET_BYTES);
    let component_count = target.split('/').count();
    let mut normalized = Vec::with_capacity(component_count);
    for component in target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                return Err(Error::Validation(format!("generated file target `{target}` escapes the project root")));
            }
            other => normalized.push(other),
        }
    }
    if normalized.is_empty() {
        return Err(Error::Validation(format!("generated file target `{target}` does not name a file")));
    }
    Ok(normalized.join("/"))
}

fn validate_contract(declaration: &GeneratedFileDeclaration) -> Result<(), Error> {
    let Some(contract) = &declaration.contract else {
        return Ok(());
    };
    if contract.identity.is_empty() {
        return Err(Error::Validation("generated file contract identity must not be empty".to_string()));
    }
    if contract.required_json_fields.len() as u64 > u64::from(MAX_CONTRACT_FIELDS) {
        return Err(Error::Validation(format!(
            "generated file contract has too many required JSON fields: {} > {MAX_CONTRACT_FIELDS}",
            contract.required_json_fields.len()
        )));
    }
    if declaration.materialization == GeneratedFileMaterialization::Symlink {
        return Err(Error::Validation("typed generated-file contracts apply only to copy materialization".to_string()));
    }
    debug_assert!(!contract.identity.is_empty());
    debug_assert!(contract.required_json_fields.len() as u64 <= u64::from(MAX_CONTRACT_FIELDS));
    let content = declaration_content(&declaration.content);
    let value: serde_json::Value = serde_json::from_str(&content).map_err(|err| {
        Error::Validation(format!("generated content is not valid JSON for contract `{}`: {err}", contract.identity))
    })?;
    let Some(object) = value.as_object() else {
        return Err(Error::Validation(format!(
            "generated content for contract `{}` must be a JSON object",
            contract.identity
        )));
    };
    for field in &contract.required_json_fields {
        if field.is_empty() {
            return Err(Error::Validation("generated file contract required field must not be empty".to_string()));
        }
        if !object.contains_key(field) {
            return Err(Error::Validation(format!(
                "generated content for contract `{}` is missing field `{field}`",
                contract.identity
            )));
        }
    }
    Ok(())
}

fn desired_digest(declaration: &GeneratedFileDeclaration) -> Result<String, Error> {
    if let GeneratedFileContent::NickelExport {
        receipt_digest_blake3, ..
    } = &declaration.content
    {
        validate_blake3_hex(receipt_digest_blake3)?;
    }
    Ok(blake3_hex(declaration_content(&declaration.content).as_bytes()))
}

fn declaration_content(content: &GeneratedFileContent) -> String {
    match content {
        GeneratedFileContent::Inline { text } => text.clone(),
        GeneratedFileContent::NickelExport { text, .. } => text.clone(),
    }
}

fn classify_action(
    current: Option<&CurrentFileState>,
    desired_digest: &str,
    materialization: GeneratedFileMaterialization,
    content: &str,
) -> FilegenAction {
    match current {
        None | Some(CurrentFileState::Missing) => FilegenAction::Create,
        Some(CurrentFileState::Managed { digest_blake3 }) if digest_blake3 == desired_digest => {
            FilegenAction::Unchanged
        }
        Some(CurrentFileState::Managed { .. }) => FilegenAction::Update,
        Some(CurrentFileState::Unmanaged { digest_blake3 }) if digest_blake3 == desired_digest => {
            FilegenAction::Unchanged
        }
        Some(CurrentFileState::Unmanaged { .. }) => FilegenAction::Conflict,
        Some(CurrentFileState::Symlink { target, managed }) => {
            if materialization == GeneratedFileMaterialization::Symlink && target == content {
                FilegenAction::Unchanged
            } else if *managed {
                FilegenAction::Update
            } else {
                FilegenAction::Conflict
            }
        }
    }
}

fn validate_blake3_hex(value: &str) -> Result<(), Error> {
    if value.len() != BLAKE3_HEX_BYTES {
        return Err(Error::Validation(format!("BLAKE3 digest must be {BLAKE3_HEX_BYTES} lowercase hex bytes")));
    }
    if !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(Error::Validation("BLAKE3 digest must be lowercase hexadecimal".to_string()));
    }
    Ok(())
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn blocker(code: &'static str, target: impl AsRef<str>, message: String) -> FilegenBlocker {
    assert!(!code.is_empty(), "filegen blocker code must not be empty");
    assert!(!message.is_empty(), "filegen blocker message must not be empty");
    FilegenBlocker {
        code: code.to_string(),
        target: target.as_ref().to_string(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inline_file(name: &str, target: &str, text: &str) -> GeneratedFileDeclaration {
        GeneratedFileDeclaration {
            name: name.to_string(),
            target: target.to_string(),
            content: GeneratedFileContent::Inline { text: text.to_string() },
            materialization: GeneratedFileMaterialization::Copy,
            contract: None,
        }
    }

    fn current(target: &str, state: CurrentFileState) -> CurrentFileFact {
        CurrentFileFact {
            target: target.to_string(),
            state,
        }
    }

    #[test]
    fn plan_create_update_unchanged_and_stale_generated_files() {
        let create = inline_file("create", "generated/create.json", "{}");
        let update = inline_file("update", "generated/update.json", "new");
        let unchanged = inline_file("same", "generated/same.json", "same");
        let same_digest = blake3_hex(b"same");
        let plan = plan_file_generation(FilegenPlanRequest {
            declarations: vec![create, update, unchanged],
            current_files: vec![
                current("generated/update.json", CurrentFileState::Managed {
                    digest_blake3: blake3_hex(b"old"),
                }),
                current("generated/same.json", CurrentFileState::Managed {
                    digest_blake3: same_digest,
                }),
                current("generated/old.json", CurrentFileState::Managed {
                    digest_blake3: blake3_hex(b"stale"),
                }),
            ],
        });

        assert!(!plan.has_blockers());
        assert_eq!(plan.operations[0].action, FilegenAction::Create);
        assert_eq!(plan.operations[1].action, FilegenAction::Update);
        assert_eq!(plan.operations[2].action, FilegenAction::Unchanged);
        assert_eq!(plan.operations[3].action, FilegenAction::Stale);
        assert_eq!(plan.non_claim, FILEGEN_NON_CLAIM);
    }

    #[test]
    fn symlink_materialization_accepts_matching_managed_symlink() {
        let declaration = GeneratedFileDeclaration {
            name: "link".to_string(),
            target: "generated/tool".to_string(),
            content: GeneratedFileContent::Inline {
                text: "../tools/tool".to_string(),
            },
            materialization: GeneratedFileMaterialization::Symlink,
            contract: None,
        };
        let plan = plan_file_generation(FilegenPlanRequest {
            declarations: vec![declaration],
            current_files: vec![current("generated/tool", CurrentFileState::Symlink {
                target: "../tools/tool".to_string(),
                managed: true,
            })],
        });

        assert!(!plan.has_blockers());
        assert_eq!(plan.operations[0].action, FilegenAction::Unchanged);
    }

    #[test]
    fn typed_valid_content_records_contract_identity() {
        let mut declaration = inline_file("typed", "generated/report.json", r#"{"schema":"demo","ok":true}"#);
        declaration.contract = Some(GeneratedFileContract {
            identity: "demo-contract".to_string(),
            required_json_fields: vec!["schema".to_string(), "ok".to_string()],
        });

        let plan = plan_file_generation(FilegenPlanRequest {
            declarations: vec![declaration],
            current_files: Vec::new(),
        });

        assert!(!plan.has_blockers());
        assert_eq!(plan.operations[0].contract_identity.as_deref(), Some("demo-contract"));
    }

    #[test]
    fn target_escape_existing_conflict_invalid_contract_and_bad_export_digest_fail_closed() {
        let escape = inline_file("escape", "../outside", "x");
        let conflict = inline_file("conflict", "generated/conflict", "desired");
        let mut invalid_contract = inline_file("invalid", "generated/invalid.json", r#"{"schema":"demo"}"#);
        invalid_contract.contract = Some(GeneratedFileContract {
            identity: "demo-contract".to_string(),
            required_json_fields: vec!["missing".to_string()],
        });
        let bad_export = GeneratedFileDeclaration {
            name: "export".to_string(),
            target: "generated/export.json".to_string(),
            content: GeneratedFileContent::NickelExport {
                text: "{}".to_string(),
                receipt_digest_blake3: "not-a-digest".to_string(),
            },
            materialization: GeneratedFileMaterialization::Copy,
            contract: None,
        };

        let plan = plan_file_generation(FilegenPlanRequest {
            declarations: vec![escape, conflict, invalid_contract, bad_export],
            current_files: vec![current("generated/conflict", CurrentFileState::Unmanaged {
                digest_blake3: blake3_hex(b"other"),
            })],
        });
        let codes = plan.blockers.iter().map(|blocker| blocker.code.as_str()).collect::<Vec<_>>();

        assert!(codes.contains(&"target-escape"));
        assert!(codes.contains(&"existing-file-conflict"));
        assert!(codes.contains(&"contract-invalid-content"));
        assert!(codes.contains(&"invalid-export-receipt-digest"));
    }

    #[test]
    fn over_limit_inputs_return_blockers_without_planning() {
        let too_many = usize::try_from(MAX_FILEGEN_DECLARATIONS).unwrap() + 1;
        let declarations = (0..too_many)
            .map(|index| inline_file("file", &format!("generated/{index}"), "{}"))
            .collect::<Vec<_>>();

        let plan = plan_file_generation(FilegenPlanRequest {
            declarations,
            current_files: Vec::new(),
        });

        assert!(plan.operations.is_empty());
        assert_eq!(plan.blockers[0].code, "too-many-generated-files");
    }

    #[test]
    fn apply_plan_detects_plan_drift() {
        let reviewed = plan_file_generation(FilegenPlanRequest {
            declarations: vec![inline_file("file", "generated/file", "old")],
            current_files: Vec::new(),
        });
        let current = plan_file_generation(FilegenPlanRequest {
            declarations: vec![inline_file("file", "generated/file", "new")],
            current_files: Vec::new(),
        });

        let err = verify_filegen_apply_plan(&reviewed, &current).unwrap_err();

        assert_eq!(err[0].code, "plan-drift");
    }
}
