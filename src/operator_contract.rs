// machine-artifact-public: operator.command-contract-reports
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

pub const OPERATOR_INVENTORY_SCHEMA: &str = "mantle-operator-surface-inventory-v1";
pub const OPERATOR_CATALOG_SCHEMA: &str = "mantle-operator-command-catalog-v1";
pub const REMEDIATION_SCHEMA: &str = "mantle-remediation-v1";
pub const COMMAND_SURFACE_COUNT_MAX: usize = 1_024;
pub const COMPATIBILITY_SPELLING_COUNT_MAX: usize = 16;
pub const SUPPORTED_OPERATION_COUNT_MAX: usize = 16;
pub const COMMAND_FLAG_COUNT_MAX: usize = 128;
pub const EXIT_CLASS_COUNT_MAX: usize = 8;
pub const REMEDIATION_ACTION_COUNT_MAX: usize = 8;
pub const REMEDIATION_PRECONDITION_COUNT_MAX: usize = 8;
pub const EVIDENCE_REFERENCE_COUNT_MAX: usize = 16;
pub const OPERATOR_TEXT_BYTES_MAX: usize = 1_024;
pub const SAFE_SUBJECT_BYTES_MAX: usize = 128;
pub const PLATFORM_PROFILE_COUNT_MAX: usize = 128;
const ALLOWED_EXIT_CLASSES: &[&str] = &["policy-rejection", "success", "usage"];

fn empty_strings() -> Vec<String> {
    Vec::new()
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceKind {
    Command,
    ProjectFile,
    Environment,
    MachineSchema,
    Identifier,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportTier {
    Daily,
    Advanced,
    Compatibility,
    Internal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutationClass {
    None,
    ProjectFiles,
    StoreState,
    ProjectAndStore,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkClass {
    None,
    Optional,
    Required,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityState {
    Canonical,
    CompatibilityReadWrite,
    CompatibilityReadOnly,
    HistoricalOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OperatorSurface {
    pub kind: SurfaceKind,
    pub canonical: String,
    #[serde(default = "empty_strings")]
    pub compatibility_spellings: Vec<String>,
    pub owner: String,
    pub role: String,
    #[serde(default = "empty_strings")]
    pub supported_operations: Vec<String>,
    pub support_tier: SupportTier,
    pub mutation: MutationClass,
    pub network: NetworkClass,
    pub compatibility_state: CompatibilityState,
    #[serde(default = "empty_strings")]
    pub flags: Vec<String>,
    #[serde(default = "empty_strings")]
    pub exit_classes: Vec<String>,
    pub machine_schema: Option<String>,
    pub migration_reference: Option<String>,
    pub removal_gate: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DarwinSupport {
    Supported,
    RemoteRequired,
    Mixed,
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformProfile {
    pub command_root: String,
    pub role: mantle_portable_client_core::CommandRole,
    pub effects: mantle_portable_client_core::CommandEffects,
    pub darwin_support: DarwinSupport,
    pub blocker: Option<String>,
    pub remote_capability_required: bool,
    pub trusted_builder_key_required: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OperatorInventory {
    pub schema: String,
    pub surfaces: Vec<OperatorSurface>,
    pub platform_profiles: Vec<PlatformProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandDescriptor {
    pub path: String,
    #[serde(default = "empty_strings")]
    pub aliases: Vec<String>,
    #[serde(default = "empty_strings")]
    pub flags: Vec<String>,
    pub help: String,
    pub supports_json: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandCatalogEntry {
    pub descriptor: CommandDescriptor,
    pub policy: OperatorSurface,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandCatalog {
    pub schema: &'static str,
    pub identity_blake3: String,
    pub entries: Vec<CommandCatalogEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractError {
    pub class: &'static str,
    pub context: String,
}

impl ContractError {
    fn new(class: &'static str, context: impl Into<String>) -> Self {
        Self {
            class,
            context: context.into(),
        }
    }
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.class, self.context)
    }
}

impl std::error::Error for ContractError {}

struct TextValidation<'a> {
    field: &'static str,
    value: &'a str,
}

struct SortedUniqueValidation<'a> {
    class: &'static str,
    canonical: &'a str,
    values: &'a [String],
}

struct SpellingRegistration<'a> {
    spelling: &'a str,
    canonical: &'a str,
}

pub fn validate_inventory(inventory: &OperatorInventory) -> Result<(), ContractError> {
    if inventory.schema != OPERATOR_INVENTORY_SCHEMA {
        return Err(ContractError::new("unknown-schema", inventory.schema.clone()));
    }
    if inventory.surfaces.is_empty() {
        return Err(ContractError::new("empty-inventory", "no operator surfaces"));
    }
    if inventory.surfaces.len() > COMMAND_SURFACE_COUNT_MAX {
        return Err(ContractError::new(
            "inventory-limit",
            format!("{} surfaces exceeds {COMMAND_SURFACE_COUNT_MAX}", inventory.surfaces.len()),
        ));
    }

    validate_platform_profiles(&inventory.platform_profiles)?;
    debug_assert_eq!(inventory.schema, OPERATOR_INVENTORY_SCHEMA);
    debug_assert!(!inventory.surfaces.is_empty());
    debug_assert!(inventory.surfaces.len() <= COMMAND_SURFACE_COUNT_MAX);

    let mut spellings = BTreeMap::<String, String>::new();
    for surface in &inventory.surfaces {
        validate_surface(surface)?;
        insert_spelling(&mut spellings, SpellingRegistration {
            spelling: &surface.canonical,
            canonical: &surface.canonical,
        })?;
        for spelling in &surface.compatibility_spellings {
            insert_spelling(&mut spellings, SpellingRegistration {
                spelling,
                canonical: &surface.canonical,
            })?;
        }
    }
    Ok(())
}

fn validate_platform_profiles(profiles: &[PlatformProfile]) -> Result<(), ContractError> {
    if profiles.len() != mantle_portable_client_core::ROOT_COMMAND_COUNT {
        return Err(ContractError::new(
            "platform-profile-count",
            format!(
                "{} profiles does not match {} reviewed roots",
                profiles.len(),
                mantle_portable_client_core::ROOT_COMMAND_COUNT
            ),
        ));
    }
    if profiles.len() > PLATFORM_PROFILE_COUNT_MAX {
        return Err(ContractError::new("platform-profile-limit", profiles.len().to_string()));
    }
    debug_assert_eq!(profiles.len(), mantle_portable_client_core::ROOT_COMMAND_COUNT);
    debug_assert!(profiles.len() <= PLATFORM_PROFILE_COUNT_MAX);

    let mut roots = BTreeMap::<String, ()>::new();
    for profile in profiles {
        validate_text(TextValidation {
            field: "platform-command-root",
            value: &profile.command_root,
        })?;
        if roots.insert(profile.command_root.clone(), ()).is_some() {
            return Err(ContractError::new("duplicate-platform-profile", profile.command_root.clone()));
        }
        let Some(core_profile) = mantle_portable_client_core::find_command_profile(&profile.command_root) else {
            return Err(ContractError::new("unknown-platform-profile", profile.command_root.clone()));
        };
        if profile.role != core_profile.role || profile.effects != core_profile.effects {
            return Err(ContractError::new("platform-profile-drift", profile.command_root.clone()));
        }
        let is_blocker_required = matches!(profile.darwin_support, DarwinSupport::Mixed | DarwinSupport::Unsupported);
        if is_blocker_required != profile.blocker.as_deref().is_some_and(|value| !value.is_empty()) {
            return Err(ContractError::new("platform-blocker-drift", profile.command_root.clone()));
        }
        if let Some(blocker) = &profile.blocker {
            validate_text(TextValidation {
                field: "platform-blocker",
                value: blocker,
            })?;
        }
    }
    debug_assert_eq!(roots.len(), profiles.len());
    Ok(())
}

fn validate_surface_identity(surface: &OperatorSurface) -> Result<(), ContractError> {
    validate_text(TextValidation {
        field: "canonical",
        value: &surface.canonical,
    })?;
    validate_text(TextValidation {
        field: "owner",
        value: &surface.owner,
    })?;
    validate_text(TextValidation {
        field: "role",
        value: &surface.role,
    })?;
    Ok(())
}

fn validate_surface(surface: &OperatorSurface) -> Result<(), ContractError> {
    validate_surface_identity(surface)?;
    if surface.compatibility_spellings.len() > COMPATIBILITY_SPELLING_COUNT_MAX {
        return Err(ContractError::new("compatibility-limit", surface.canonical.clone()));
    }
    if surface.supported_operations.is_empty() {
        return Err(ContractError::new("missing-supported-operation", surface.canonical.clone()));
    }
    if surface.supported_operations.len() > SUPPORTED_OPERATION_COUNT_MAX {
        return Err(ContractError::new("supported-operation-limit", surface.canonical.clone()));
    }
    if surface.flags.len() > COMMAND_FLAG_COUNT_MAX {
        return Err(ContractError::new("flag-limit", surface.canonical.clone()));
    }
    if surface.exit_classes.len() > EXIT_CLASS_COUNT_MAX {
        return Err(ContractError::new("exit-class-limit", surface.canonical.clone()));
    }
    if surface.kind == SurfaceKind::Command && surface.exit_classes.is_empty() {
        return Err(ContractError::new("missing-exit-contract", surface.canonical.clone()));
    }
    if surface.compatibility_state != CompatibilityState::Canonical {
        if surface.migration_reference.as_deref().is_none_or(str::is_empty) {
            return Err(ContractError::new("missing-migration-reference", surface.canonical.clone()));
        }
        if surface.removal_gate.as_deref().is_none_or(str::is_empty) {
            return Err(ContractError::new("missing-removal-gate", surface.canonical.clone()));
        }
    }
    debug_assert!(!surface.supported_operations.is_empty());
    debug_assert!(surface.compatibility_spellings.len() <= COMPATIBILITY_SPELLING_COUNT_MAX);
    debug_assert!(surface.supported_operations.len() <= SUPPORTED_OPERATION_COUNT_MAX);
    debug_assert!(surface.flags.len() <= COMMAND_FLAG_COUNT_MAX);
    debug_assert!(surface.exit_classes.len() <= EXIT_CLASS_COUNT_MAX);

    validate_surface_text_fields(surface)?;
    for validation in [
        SortedUniqueValidation {
            class: "compatibility-spelling",
            canonical: &surface.canonical,
            values: &surface.compatibility_spellings,
        },
        SortedUniqueValidation {
            class: "supported-operation",
            canonical: &surface.canonical,
            values: &surface.supported_operations,
        },
        SortedUniqueValidation {
            class: "flag",
            canonical: &surface.canonical,
            values: &surface.flags,
        },
        SortedUniqueValidation {
            class: "exit-class",
            canonical: &surface.canonical,
            values: &surface.exit_classes,
        },
    ] {
        validate_sorted_unique(validation)?;
    }
    for exit_class in &surface.exit_classes {
        if !ALLOWED_EXIT_CLASSES.contains(&exit_class.as_str()) {
            return Err(ContractError::new("unknown-exit-class", exit_class.clone()));
        }
    }
    Ok(())
}

fn validate_surface_text_fields(surface: &OperatorSurface) -> Result<(), ContractError> {
    debug_assert!(!surface.supported_operations.is_empty());
    debug_assert!(surface.compatibility_spellings.len() <= COMPATIBILITY_SPELLING_COUNT_MAX);
    debug_assert!(surface.supported_operations.len() <= SUPPORTED_OPERATION_COUNT_MAX);
    for (field, values) in [
        ("compatibility-spelling", surface.compatibility_spellings.as_slice()),
        ("supported-operation", surface.supported_operations.as_slice()),
        ("flag", surface.flags.as_slice()),
        ("exit-class", surface.exit_classes.as_slice()),
    ] {
        for value in values {
            validate_text(TextValidation { field, value })?;
        }
    }
    for (field, value) in [
        ("machine-schema", surface.machine_schema.as_deref()),
        ("migration-reference", surface.migration_reference.as_deref()),
        ("removal-gate", surface.removal_gate.as_deref()),
    ] {
        if let Some(value) = value {
            validate_text(TextValidation { field, value })?;
        }
    }
    Ok(())
}

fn validate_text(validation: TextValidation<'_>) -> Result<(), ContractError> {
    if validation.value.trim().is_empty() {
        return Err(ContractError::new("empty-field", validation.field));
    }
    if validation.value.len() > OPERATOR_TEXT_BYTES_MAX {
        return Err(ContractError::new("field-limit", validation.field));
    }
    if validation.value.chars().any(char::is_control) {
        return Err(ContractError::new("control-character", validation.field));
    }
    Ok(())
}

fn validate_sorted_unique(validation: SortedUniqueValidation<'_>) -> Result<(), ContractError> {
    for pair in validation.values.windows(2) {
        if pair[0] >= pair[1] {
            return Err(ContractError::new(validation.class, validation.canonical.to_string()));
        }
    }
    Ok(())
}

fn insert_spelling(
    spellings: &mut BTreeMap<String, String>,
    registration: SpellingRegistration<'_>,
) -> Result<(), ContractError> {
    validate_text(TextValidation {
        field: "spelling",
        value: registration.spelling,
    })?;
    if let Some(first_owner) = spellings.insert(registration.spelling.to_string(), registration.canonical.to_string()) {
        return Err(ContractError::new(
            "duplicate-spelling",
            format!("{} belongs to {first_owner} and {}", registration.spelling, registration.canonical),
        ));
    }
    Ok(())
}

pub fn build_command_catalog(
    descriptors: &[CommandDescriptor],
    inventory: &OperatorInventory,
) -> Result<CommandCatalog, ContractError> {
    validate_inventory(inventory)?;
    if descriptors.is_empty() {
        return Err(ContractError::new("empty-command-graph", "no public commands"));
    }
    if descriptors.len() > COMMAND_SURFACE_COUNT_MAX {
        return Err(ContractError::new("command-graph-limit", descriptors.len().to_string()));
    }
    debug_assert!(!descriptors.is_empty());
    debug_assert!(descriptors.len() <= COMMAND_SURFACE_COUNT_MAX);

    let command_policies = command_policy_map(inventory)?;
    let mut normalized_descriptors = descriptors.to_vec();
    normalized_descriptors.sort_by(|left, right| left.path.cmp(&right.path));
    for pair in normalized_descriptors.windows(2) {
        if pair[0].path == pair[1].path {
            return Err(ContractError::new("duplicate-command-path", pair[0].path.clone()));
        }
    }

    let mut entries = Vec::with_capacity(normalized_descriptors.len());
    for mut descriptor in normalized_descriptors {
        normalize_descriptor(&mut descriptor)?;
        let Some(policy) = command_policies.get(&descriptor.path) else {
            return Err(ContractError::new("undocumented-command", descriptor.path));
        };
        compare_descriptor_policy(&descriptor, policy)?;
        entries.push(CommandCatalogEntry {
            descriptor,
            policy: (*policy).clone(),
        });
    }
    if entries.len() != command_policies.len() {
        let missing = command_policies
            .keys()
            .find(|path| !entries.iter().any(|entry| &entry.descriptor.path == *path))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        return Err(ContractError::new("stale-inventory-command", missing));
    }

    let identity_bytes =
        serde_json::to_vec(&entries).map_err(|error| ContractError::new("catalog-serialization", error.to_string()))?;
    let identity_blake3 = blake3::hash(&identity_bytes).to_hex().to_string();
    Ok(CommandCatalog {
        schema: OPERATOR_CATALOG_SCHEMA,
        identity_blake3,
        entries,
    })
}

fn command_policy_map(inventory: &OperatorInventory) -> Result<BTreeMap<String, &OperatorSurface>, ContractError> {
    let mut policies = BTreeMap::new();
    for surface in &inventory.surfaces {
        if surface.kind != SurfaceKind::Command {
            continue;
        }
        if policies.insert(surface.canonical.clone(), surface).is_some() {
            return Err(ContractError::new("duplicate-command-policy", surface.canonical.clone()));
        }
    }
    if policies.is_empty() {
        return Err(ContractError::new("empty-command-policy", "no command rows"));
    }
    Ok(policies)
}

fn normalize_descriptor(descriptor: &mut CommandDescriptor) -> Result<(), ContractError> {
    validate_text(TextValidation {
        field: "command-path",
        value: &descriptor.path,
    })?;
    validate_text(TextValidation {
        field: "command-help",
        value: &descriptor.help,
    })?;
    descriptor.aliases.sort();
    descriptor.aliases.dedup();
    descriptor.flags.sort();
    descriptor.flags.dedup();
    if descriptor.aliases.len() > COMPATIBILITY_SPELLING_COUNT_MAX {
        return Err(ContractError::new("alias-limit", descriptor.path.clone()));
    }
    if descriptor.flags.len() > COMMAND_FLAG_COUNT_MAX {
        return Err(ContractError::new("flag-limit", descriptor.path.clone()));
    }
    debug_assert!(descriptor.aliases.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(descriptor.flags.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn compare_descriptor_policy(descriptor: &CommandDescriptor, policy: &OperatorSurface) -> Result<(), ContractError> {
    if descriptor.aliases != policy.compatibility_spellings {
        return Err(ContractError::new("alias-drift", descriptor.path.clone()));
    }
    if descriptor.flags != policy.flags {
        return Err(ContractError::new("flag-drift", descriptor.path.clone()));
    }
    if descriptor.supports_json != policy.machine_schema.is_some() {
        return Err(ContractError::new("json-contract-drift", descriptor.path.clone()));
    }
    Ok(())
}

pub fn render_command_reference(catalog: &CommandCatalog) -> String {
    assert_eq!(catalog.schema, OPERATOR_CATALOG_SCHEMA);
    assert!(!catalog.entries.is_empty(), "catalog must have entries");
    let mut output = String::from("# Mantle command reference\n\n");
    output.push_str(&format!("Catalog BLAKE3: `{}`\n\n", catalog.identity_blake3));
    for tier in [SupportTier::Daily, SupportTier::Advanced, SupportTier::Compatibility] {
        let tier_name = support_tier_name(tier);
        output.push_str(&format!("## {tier_name}\n\n"));
        for entry in catalog.entries.iter().filter(|entry| entry.policy.support_tier == tier) {
            output.push_str(&format!("### `mantle {}`\n\n", entry.descriptor.path));
            output.push_str(&entry.descriptor.help);
            output.push_str("\n\n");
            output.push_str(&format!(
                "- Mutation: `{}`\n- Network: `{}`\n- Exit classes: `{}`\n",
                mutation_name(entry.policy.mutation),
                network_name(entry.policy.network),
                entry.policy.exit_classes.join(", ")
            ));
            if let Some(schema) = &entry.policy.machine_schema {
                output.push_str(&format!("- JSON schema: `{schema}`\n"));
            }
            output.push('\n');
        }
    }
    assert!(output.ends_with("\n\n"), "generated reference must end with one blank line before normalization");
    output.pop();
    output
}

pub fn render_canonical_workflow(catalog: &CommandCatalog) -> Result<String, ContractError> {
    require_workflow_command(catalog, "doctor", &[])?;
    require_workflow_command(catalog, "check", &[])?;
    require_workflow_command(catalog, "build", &["--builder", "--plan", "--ticket-fd"])?;
    require_workflow_command(catalog, "attest show", &[])?;
    Ok(String::from(
        "# Canonical Mantle workflow\n\n\
1. Run `mantle doctor`. This command does not mutate state or use the network.\n\
2. Run `mantle check`. This command reads project files and does not mutate them.\n\
3. Run `mantle build --plan <root.ncl>`. This command plans realization without store mutation.\n\
4. Run `mantle build <root.ncl>` for local realization on a supported Linux host. This step can mutate store state and use the network.\n\
5. If local realization is unsupported, run `mantle build --builder <builder-id> --ticket-fd <fd> <root.ncl>` only for an eligible reviewed remote route. This step mutates store state and requires the network.\n\
6. Run `mantle attest show <store-path>`. This command reads evidence and does not mutate state.\n",
    ))
}

fn require_workflow_command(
    catalog: &CommandCatalog,
    path: &'static str,
    required_flags: &[&'static str],
) -> Result<(), ContractError> {
    let Some(entry) = catalog.entries.iter().find(|entry| entry.descriptor.path == path) else {
        return Err(ContractError::new("workflow-command-missing", path));
    };
    for required_flag in required_flags {
        if !entry.descriptor.flags.iter().any(|flag| flag == required_flag) {
            return Err(ContractError::new("workflow-flag-missing", format!("{path} {required_flag}")));
        }
    }
    Ok(())
}

fn support_tier_name(tier: SupportTier) -> &'static str {
    match tier {
        SupportTier::Daily => "Daily commands",
        SupportTier::Advanced => "Advanced commands",
        SupportTier::Compatibility => "Compatibility commands",
        SupportTier::Internal => "Internal commands",
    }
}

fn mutation_name(class: MutationClass) -> &'static str {
    match class {
        MutationClass::None => "none",
        MutationClass::ProjectFiles => "project-files",
        MutationClass::StoreState => "store-state",
        MutationClass::ProjectAndStore => "project-and-store",
    }
}

fn network_name(class: NetworkClass) -> &'static str {
    match class {
        NetworkClass::None => "none",
        NetworkClass::Optional => "optional",
        NetworkClass::Required => "required",
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticPhase {
    Evaluation,
    Preflight,
    Build,
    Store,
    Source,
    Remote,
    Internal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureFacts<'a> {
    pub kind: &'a str,
    pub message: &'a str,
    pub safe_subject: Option<&'a str>,
    pub remote_route_eligible: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemediationAction {
    pub command: String,
    pub mutation: MutationClass,
    pub network: NetworkClass,
    pub preconditions: Vec<String>,
    pub explanation: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemediationRecord {
    pub schema: &'static str,
    pub code: &'static str,
    pub phase: DiagnosticPhase,
    pub safe_subject: String,
    pub message: String,
    pub evidence_references: Vec<String>,
    pub next_actions: Vec<RemediationAction>,
}

struct RemediationContext<'a> {
    kind: &'a str,
    message_lower: String,
    safe_subject: String,
    remote_route_eligible: bool,
}

pub fn classify_remediation(facts: FailureFacts<'_>) -> Option<RemediationRecord> {
    let context = RemediationContext {
        kind: facts.kind,
        message_lower: facts.message.to_ascii_lowercase(),
        safe_subject: safe_subject(facts.safe_subject),
        remote_route_eligible: facts.remote_route_eligible,
    };
    if let Some(record) = classify_source_boundary(&context) {
        return Some(record);
    }
    if let Some(record) = classify_build_boundary(&context) {
        return Some(record);
    }
    if let Some(record) = classify_store_boundary(&context) {
        return Some(record);
    }
    classify_exit_or_internal_boundary(&context)
}

fn classify_source_boundary(context: &RemediationContext<'_>) -> Option<RemediationRecord> {
    debug_assert!(!context.safe_subject.is_empty());
    debug_assert!(context.safe_subject.len() <= SAFE_SUBJECT_BYTES_MAX);
    if context.message_lower.contains("source input not found") || context.message_lower.contains("sourcenotfound") {
        return Some(remediation(
            "mantle.source.missing-input",
            DiagnosticPhase::Source,
            context.safe_subject.clone(),
            "A declared source input is not available in current source or store state.",
            action(
                "mantle source bundle plan --build-root <root.ncl>",
                MutationClass::None,
                NetworkClass::None,
                &["a reviewed root Nickel file"],
                "Inspect the bounded source plan before import or network access.",
            ),
        ));
    }
    if context.message_lower.contains("bwrap") && context.message_lower.contains("can't") {
        return Some(remediation(
            "mantle.build.namespace-unavailable",
            DiagnosticPhase::Preflight,
            context.safe_subject.clone(),
            "Bubblewrap could not create the required user namespace.",
            action(
                "mantle doctor --profile build",
                MutationClass::None,
                NetworkClass::None,
                &["unprivileged user namespaces enabled by host policy"],
                "Check the user-namespace prerequisite before retrying; the host setting is commonly named unprivileged_userns_clone.",
            ),
        ));
    }
    if context.message_lower.contains("fod hash mismatch") {
        return Some(remediation(
            "mantle.build.fixed-output-hash-mismatch",
            DiagnosticPhase::Build,
            context.safe_subject.clone(),
            "The fixed-output result does not match the declared hash.",
            action(
                "mantle build --fix <root.ncl>",
                MutationClass::ProjectFiles,
                NetworkClass::Optional,
                &["review the fetched content", "a writable Nickel source"],
                "Update the hash only after the fetched content and source identity are reviewed.",
            ),
        ));
    }
    None
}

fn classify_build_boundary(context: &RemediationContext<'_>) -> Option<RemediationRecord> {
    debug_assert!(!context.safe_subject.is_empty());
    debug_assert!(context.safe_subject.len() <= SAFE_SUBJECT_BYTES_MAX);
    if context.message_lower.contains("output not produced by build") {
        return Some(remediation(
            "mantle.build.output-missing",
            DiagnosticPhase::Build,
            context.safe_subject.clone(),
            "The builder did not produce every declared output.",
            action(
                "mantle log <derivation>",
                MutationClass::None,
                NetworkClass::None,
                &["the failed derivation identity"],
                "Inspect whether the builder created $out and each other declared output.",
            ),
        ));
    }
    if context.message_lower.contains("only supported on linux")
        || context.message_lower.contains("builds are not supported")
    {
        let selected_action = if context.remote_route_eligible {
            action(
                "mantle build --builder <builder-id> --ticket-fd <fd> <root.ncl>",
                MutationClass::StoreState,
                NetworkClass::Required,
                &["an eligible remote builder", "a caller-owned bearer descriptor"],
                "Use the reviewed remote route instead of installing local Linux execution tools.",
            )
        } else {
            action(
                "mantle doctor --profile build",
                MutationClass::None,
                NetworkClass::None,
                &["a supported local build host"],
                "Inspect local executor prerequisites and platform support.",
            )
        };
        return Some(remediation(
            "mantle.build.unsupported-platform",
            DiagnosticPhase::Preflight,
            context.safe_subject.clone(),
            "The selected local realization route is unsupported on this host.",
            selected_action,
        ));
    }
    None
}

fn classify_store_boundary(context: &RemediationContext<'_>) -> Option<RemediationRecord> {
    debug_assert!(!context.safe_subject.is_empty());
    debug_assert!(context.safe_subject.len() <= SAFE_SUBJECT_BYTES_MAX);
    if context.message_lower.contains("store directory") && context.message_lower.contains("does not exist") {
        return Some(remediation(
            "mantle.store.missing-directory",
            DiagnosticPhase::Store,
            context.safe_subject.clone(),
            "The selected physical store directory does not exist.",
            action(
                "mantle doctor --profile build",
                MutationClass::None,
                NetworkClass::None,
                &["an explicit --store selection when the default is unsuitable"],
                "Check the selected store path before a mutating build.",
            ),
        ));
    }
    if context.message_lower.contains("failed to run nix") || context.message_lower.contains("failed to resolve") {
        return Some(remediation(
            "mantle.source.resolution-unavailable",
            DiagnosticPhase::Source,
            context.safe_subject.clone(),
            "The selected source resolution path is unavailable.",
            action(
                "mantle bootstrap --fetch",
                MutationClass::StoreState,
                NetworkClass::Required,
                &["reviewed bootstrap source policy", "network access"],
                "Use Mantle's canonical fetch path instead of a legacy command spelling.",
            ),
        ));
    }
    None
}

fn classify_exit_or_internal_boundary(context: &RemediationContext<'_>) -> Option<RemediationRecord> {
    debug_assert!(!context.safe_subject.is_empty());
    debug_assert!(context.safe_subject.len() <= SAFE_SUBJECT_BYTES_MAX);
    if context.message_lower.contains("nonzero exit code") {
        return Some(remediation(
            "mantle.build.nonzero-exit",
            DiagnosticPhase::Build,
            context.safe_subject.clone(),
            "The builder exited with a nonzero status.",
            action(
                "mantle log <derivation>",
                MutationClass::None,
                NetworkClass::None,
                &["the failed derivation identity"],
                "Inspect the persisted bounded build log.",
            ),
        ));
    }
    if context.kind == "internal" && context.message_lower.contains("stdlib") {
        return Some(remediation(
            "mantle.internal.stdlib-unavailable",
            DiagnosticPhase::Internal,
            context.safe_subject.clone(),
            "Mantle could not locate its standard library.",
            action(
                "mantle doctor --profile build",
                MutationClass::None,
                NetworkClass::None,
                &["a complete Mantle installation"],
                "Check the installation without embedding private paths in diagnostics.",
            ),
        ));
    }
    None
}

fn remediation(
    code: &'static str,
    phase: DiagnosticPhase,
    safe_subject: String,
    message: &'static str,
    selected_action: RemediationAction,
) -> RemediationRecord {
    let record = RemediationRecord {
        schema: REMEDIATION_SCHEMA,
        code,
        phase,
        safe_subject,
        message: message.to_string(),
        evidence_references: Vec::new(),
        next_actions: vec![selected_action],
    };
    assert!(!record.code.is_empty(), "diagnostic code must not be empty");
    assert!(record.safe_subject.len() <= SAFE_SUBJECT_BYTES_MAX);
    assert!(record.message.len() <= OPERATOR_TEXT_BYTES_MAX);
    assert!(record.next_actions.len() <= REMEDIATION_ACTION_COUNT_MAX);
    record
}

fn action(
    command: &'static str,
    mutation: MutationClass,
    network: NetworkClass,
    preconditions: &[&'static str],
    explanation: &'static str,
) -> RemediationAction {
    assert!(command.starts_with("mantle "), "remediation must use the canonical product command");
    assert!(!command.contains("crunch"), "remediation must not use a legacy product command");
    assert!(command.len() <= OPERATOR_TEXT_BYTES_MAX);
    assert!(!explanation.is_empty(), "remediation explanation must not be empty");
    assert!(explanation.len() <= OPERATOR_TEXT_BYTES_MAX);
    assert!(preconditions.len() <= REMEDIATION_PRECONDITION_COUNT_MAX);
    assert!(preconditions.iter().all(|value| !value.is_empty()));
    assert!(preconditions.iter().all(|value| value.len() <= OPERATOR_TEXT_BYTES_MAX));
    RemediationAction {
        command: command.to_string(),
        mutation,
        network,
        preconditions: preconditions.iter().map(|value| (*value).to_string()).collect(),
        explanation: explanation.to_string(),
    }
}

fn safe_subject(raw: Option<&str>) -> String {
    let Some(raw) = raw else {
        return "unspecified".to_string();
    };
    let lower = raw.to_ascii_lowercase();
    let has_protected_shape = raw.contains('/')
        || raw.contains('=')
        || raw.contains("--")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("private")
        || raw.chars().any(char::is_control);
    if raw.is_empty() || raw.len() > SAFE_SUBJECT_BYTES_MAX || has_protected_shape {
        return "redacted".to_string();
    }
    raw.to_string()
}

pub fn render_remediation_human(record: &RemediationRecord) -> String {
    assert_eq!(record.schema, REMEDIATION_SCHEMA);
    assert!(!record.next_actions.is_empty(), "remediation must include an action");
    assert!(record.next_actions.len() <= REMEDIATION_ACTION_COUNT_MAX);
    assert!(record.evidence_references.len() <= EVIDENCE_REFERENCE_COUNT_MAX);
    let mut output = format!(
        "remediation: {}\nphase: {:?}\nsubject: {}\n{}\n",
        record.code, record.phase, record.safe_subject, record.message
    );
    for selected_action in &record.next_actions {
        output.push_str(&format!(
            "- command: {}\n  mutation: {}\n  network: {}\n  preconditions: {}\n  {}\n",
            selected_action.command,
            mutation_name(selected_action.mutation),
            network_name(selected_action.network),
            selected_action.preconditions.join(", "),
            selected_action.explanation
        ));
    }
    output
}

pub fn render_remediation_json(record: &RemediationRecord) -> Result<String, serde_json::Error> {
    assert_eq!(record.schema, REMEDIATION_SCHEMA);
    assert!(record.next_actions.len() <= REMEDIATION_ACTION_COUNT_MAX);
    assert!(record.evidence_references.len() <= EVIDENCE_REFERENCE_COUNT_MAX);
    serde_json::to_string(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_surface(path: &str, flags: &[&str], machine_schema: Option<&str>) -> OperatorSurface {
        OperatorSurface {
            kind: SurfaceKind::Command,
            canonical: path.to_string(),
            compatibility_spellings: Vec::new(),
            owner: "mantle-cli".to_string(),
            role: "public-command".to_string(),
            supported_operations: vec!["invoke".to_string(), "read-help".to_string()],
            support_tier: SupportTier::Daily,
            mutation: MutationClass::None,
            network: NetworkClass::None,
            compatibility_state: CompatibilityState::Canonical,
            flags: flags.iter().map(|flag| (*flag).to_string()).collect(),
            exit_classes: vec!["success".to_string(), "usage".to_string()],
            machine_schema: machine_schema.map(str::to_string),
            migration_reference: None,
            removal_gate: None,
        }
    }

    fn inventory(surfaces: Vec<OperatorSurface>) -> OperatorInventory {
        let platform_profiles = mantle_portable_client_core::COMMAND_PROFILES
            .iter()
            .map(|profile| PlatformProfile {
                command_root: profile.root.to_string(),
                role: profile.role,
                effects: profile.effects,
                darwin_support: match profile.role {
                    mantle_portable_client_core::CommandRole::PortableClient
                    | mantle_portable_client_core::CommandRole::Internal => DarwinSupport::Supported,
                    mantle_portable_client_core::CommandRole::PortableRemoteBuild => DarwinSupport::RemoteRequired,
                    mantle_portable_client_core::CommandRole::RemoteMixed => DarwinSupport::Mixed,
                    _ => DarwinSupport::Unsupported,
                },
                blocker: match profile.role {
                    mantle_portable_client_core::CommandRole::PortableClient
                    | mantle_portable_client_core::CommandRole::Internal
                    | mantle_portable_client_core::CommandRole::PortableRemoteBuild => None,
                    _ => Some("outside portable client boundary".to_string()),
                },
                remote_capability_required: profile.role
                    == mantle_portable_client_core::CommandRole::PortableRemoteBuild,
                trusted_builder_key_required: profile.role
                    == mantle_portable_client_core::CommandRole::PortableRemoteBuild,
            })
            .collect();
        OperatorInventory {
            schema: OPERATOR_INVENTORY_SCHEMA.to_string(),
            surfaces,
            platform_profiles,
        }
    }

    #[test]
    fn catalog_accepts_matching_sorted_commands() {
        let descriptors = vec![CommandDescriptor {
            path: "doctor".to_string(),
            aliases: Vec::new(),
            flags: vec!["--profile".to_string()],
            help: "Run preflight checks".to_string(),
            supports_json: true,
        }];
        let inventory = inventory(vec![command_surface("doctor", &["--profile"], Some("doctor-v1"))]);

        let catalog = build_command_catalog(&descriptors, &inventory).unwrap();

        assert_eq!(catalog.entries.len(), 1);
        assert_eq!(catalog.entries[0].descriptor.path, "doctor");
        assert_eq!(catalog.identity_blake3.len(), 64);
    }

    #[test]
    fn catalog_rejects_undocumented_command() {
        let descriptors = vec![CommandDescriptor {
            path: "doctor".to_string(),
            aliases: Vec::new(),
            flags: Vec::new(),
            help: "Run preflight checks".to_string(),
            supports_json: false,
        }];
        let inventory = inventory(vec![command_surface("build", &[], None)]);

        let error = build_command_catalog(&descriptors, &inventory).unwrap_err();

        assert_eq!(error.class, "undocumented-command");
        assert_eq!(error.context, "doctor");
    }

    #[test]
    fn inventory_rejects_duplicate_compatibility_spelling() {
        let mut first = command_surface("shell", &[], None);
        first.compatibility_spellings = vec!["develop".to_string()];
        let mut second = command_surface("develop", &[], None);
        second.support_tier = SupportTier::Compatibility;
        second.compatibility_state = CompatibilityState::CompatibilityReadWrite;
        second.migration_reference = Some("docs/mantle-naming.md".to_string());
        second.removal_gate = Some("consumer inventory and migration evidence".to_string());

        let error = validate_inventory(&inventory(vec![first, second])).unwrap_err();

        assert_eq!(error.class, "duplicate-spelling");
        assert!(error.context.contains("develop"));
    }

    #[test]
    fn inventory_rejects_compatibility_without_removal_gate() {
        let mut surface = command_surface("develop", &[], None);
        surface.compatibility_state = CompatibilityState::CompatibilityReadOnly;
        surface.migration_reference = Some("docs/mantle-naming.md".to_string());

        let error = validate_inventory(&inventory(vec![surface])).unwrap_err();

        assert_eq!(error.class, "missing-removal-gate");
    }

    #[test]
    fn inventory_rejects_missing_supported_operation() {
        let mut surface = command_surface("doctor", &[], None);
        surface.supported_operations.clear();

        let error = validate_inventory(&inventory(vec![surface])).unwrap_err();

        assert_eq!(error.class, "missing-supported-operation");
        assert_eq!(error.context, "doctor");
    }

    #[test]
    fn inventory_rejects_unowned_surface() {
        let mut surface = command_surface("doctor", &[], None);
        surface.owner.clear();

        let error = validate_inventory(&inventory(vec![surface])).unwrap_err();

        assert_eq!(error.class, "empty-field");
        assert_eq!(error.context, "owner");
    }

    #[test]
    fn legacy_omitted_collections_decode_to_explicit_empty_values() {
        let surface_json = r#"{
            "kind":"identifier",
            "canonical":"legacy-id",
            "owner":"mantle-cli",
            "role":"compatibility-id",
            "support_tier":"compatibility",
            "mutation":"none",
            "network":"none",
            "compatibility_state":"historical-only",
            "machine_schema":null,
            "migration_reference":"docs/mantle-naming.md",
            "removal_gate":"reviewed consumer migration"
        }"#;
        let descriptor_json = r#"{"path":"doctor","help":"Run checks","supports_json":false}"#;

        let surface: OperatorSurface = serde_json::from_str(surface_json).unwrap();
        let descriptor: CommandDescriptor = serde_json::from_str(descriptor_json).unwrap();

        assert!(surface.compatibility_spellings.is_empty());
        assert!(surface.supported_operations.is_empty());
        assert!(surface.flags.is_empty());
        assert!(surface.exit_classes.is_empty());
        assert!(descriptor.aliases.is_empty());
        assert!(descriptor.flags.is_empty());
    }

    #[test]
    fn inventory_decode_rejects_unknown_support_state() {
        let malformed = r#"{
            "schema":"mantle-operator-surface-inventory-v1",
            "surfaces":[{
                "kind":"command",
                "canonical":"doctor",
                "compatibility_spellings":[],
                "owner":"mantle-cli",
                "role":"public-command",
                "supported_operations":["invoke","read-help"],
                "support_tier":"unknown",
                "mutation":"none",
                "network":"none",
                "compatibility_state":"canonical",
                "flags":[],
                "exit_classes":["success"],
                "machine_schema":null,
                "migration_reference":null,
                "removal_gate":null
            }]
        }"#;

        let error = serde_json::from_str::<OperatorInventory>(malformed).unwrap_err();

        assert!(error.to_string().contains("unknown variant"));
        assert!(error.to_string().contains("daily"));
    }

    #[test]
    fn inventory_rejects_conflicting_surface_roles() {
        let command = command_surface("doctor", &[], None);
        let mut environment = command.clone();
        environment.kind = SurfaceKind::Environment;
        environment.exit_classes.clear();

        let error = validate_inventory(&inventory(vec![command, environment])).unwrap_err();

        assert_eq!(error.class, "duplicate-spelling");
        assert!(error.context.contains("doctor"));
    }

    #[test]
    fn inventory_rejects_oversized_surface_collection() {
        let surface_count = COMMAND_SURFACE_COUNT_MAX.checked_add(1).unwrap();
        let surfaces = vec![command_surface("doctor", &[], None); surface_count];

        let error = validate_inventory(&inventory(surfaces)).unwrap_err();

        assert_eq!(error.class, "inventory-limit");
        assert!(error.context.contains(&surface_count.to_string()));
    }

    #[test]
    fn inventory_decode_rejects_malformed_generated_data() {
        let malformed = r#"{"schema":"mantle-operator-surface-inventory-v1","surfaces":[}"#;

        let error = serde_json::from_str::<OperatorInventory>(malformed).unwrap_err();

        assert!(error.is_syntax());
        assert!(error.line() > 0);
    }

    #[test]
    fn inventory_rejects_unknown_exit_class() {
        let mut surface = command_surface("doctor", &[], None);
        surface.exit_classes = vec!["success".to_string(), "unknown".to_string()];

        let error = validate_inventory(&inventory(vec![surface])).unwrap_err();

        assert_eq!(error.class, "unknown-exit-class");
        assert_eq!(error.context, "unknown");
    }

    #[test]
    fn catalog_rejects_flag_drift() {
        let descriptors = vec![CommandDescriptor {
            path: "doctor".to_string(),
            aliases: Vec::new(),
            flags: vec!["--profile".to_string()],
            help: "Run preflight checks".to_string(),
            supports_json: false,
        }];
        let inventory = inventory(vec![command_surface("doctor", &[], None)]);

        let error = build_command_catalog(&descriptors, &inventory).unwrap_err();

        assert_eq!(error.class, "flag-drift");
        assert_eq!(error.context, "doctor");
    }

    #[test]
    fn catalog_rejects_alias_drift() {
        let descriptors = vec![CommandDescriptor {
            path: "develop".to_string(),
            aliases: vec!["dev".to_string()],
            flags: Vec::new(),
            help: "Open a compatibility development shell".to_string(),
            supports_json: false,
        }];
        let inventory = inventory(vec![command_surface("develop", &[], None)]);

        let error = build_command_catalog(&descriptors, &inventory).unwrap_err();

        assert_eq!(error.class, "alias-drift");
        assert_eq!(error.context, "develop");
    }

    #[test]
    fn catalog_rejects_json_contract_drift() {
        let descriptors = vec![CommandDescriptor {
            path: "doctor".to_string(),
            aliases: Vec::new(),
            flags: Vec::new(),
            help: "Run preflight checks".to_string(),
            supports_json: true,
        }];
        let inventory = inventory(vec![command_surface("doctor", &[], None)]);

        let error = build_command_catalog(&descriptors, &inventory).unwrap_err();

        assert_eq!(error.class, "json-contract-drift");
        assert_eq!(error.context, "doctor");
    }

    #[test]
    fn catalog_rejects_stale_inventory_command() {
        let descriptors = vec![CommandDescriptor {
            path: "doctor".to_string(),
            aliases: Vec::new(),
            flags: Vec::new(),
            help: "Run preflight checks".to_string(),
            supports_json: false,
        }];
        let inventory = inventory(vec![
            command_surface("build", &[], None),
            command_surface("doctor", &[], None),
        ]);

        let error = build_command_catalog(&descriptors, &inventory).unwrap_err();

        assert_eq!(error.class, "stale-inventory-command");
        assert_eq!(error.context, "build");
    }

    #[test]
    fn workflow_rejects_a_documented_flag_missing_from_clap() {
        let paths = ["attest show", "build", "check", "doctor"];
        let descriptors = paths
            .iter()
            .map(|path| {
                let flags = if *path == "build" {
                    vec!["--builder".to_string(), "--ticket-fd".to_string()]
                } else {
                    Vec::new()
                };
                CommandDescriptor {
                    path: (*path).to_string(),
                    aliases: Vec::new(),
                    flags,
                    help: format!("Help for {path}"),
                    supports_json: false,
                }
            })
            .collect::<Vec<_>>();
        let surfaces = paths
            .iter()
            .map(|path| {
                if *path == "build" {
                    command_surface(path, &["--builder", "--ticket-fd"], None)
                } else {
                    command_surface(path, &[], None)
                }
            })
            .collect::<Vec<_>>();
        let catalog = build_command_catalog(&descriptors, &inventory(surfaces)).unwrap();

        let error = render_canonical_workflow(&catalog).unwrap_err();

        assert_eq!(error.class, "workflow-flag-missing");
        assert_eq!(error.context, "build --plan");
    }

    #[test]
    fn remediation_classifier_covers_reviewed_failure_classes() {
        let cases = [
            ("build", "source input not found", "mantle.source.missing-input"),
            ("build", "bwrap can't create namespace", "mantle.build.namespace-unavailable"),
            ("build", "FOD hash mismatch", "mantle.build.fixed-output-hash-mismatch"),
            ("build", "output not produced by build", "mantle.build.output-missing"),
            ("build", "store directory does not exist", "mantle.store.missing-directory"),
            ("build", "failed to resolve source", "mantle.source.resolution-unavailable"),
            ("build", "nonzero exit code", "mantle.build.nonzero-exit"),
            ("internal", "stdlib missing", "mantle.internal.stdlib-unavailable"),
        ];

        for (kind, message, expected_code) in cases {
            let record = classify_remediation(FailureFacts {
                kind,
                message,
                safe_subject: Some("root"),
                remote_route_eligible: false,
            })
            .unwrap();
            assert_eq!(record.code, expected_code);
            assert_eq!(record.next_actions.len(), 1);
            assert!(record.next_actions[0].command.starts_with("mantle "));
            assert!(!record.next_actions[0].command.contains("crunch"));
        }
    }

    #[test]
    fn remediation_classifier_preserves_first_match_priority() {
        let record = classify_remediation(FailureFacts {
            kind: "internal",
            message: "source input not found after nonzero exit code and stdlib failure",
            safe_subject: Some("root"),
            remote_route_eligible: false,
        })
        .unwrap();

        assert_eq!(record.code, "mantle.source.missing-input");
        assert_eq!(record.phase, DiagnosticPhase::Source);
    }

    #[test]
    fn remediation_human_and_json_share_the_same_action() {
        let record = classify_remediation(FailureFacts {
            kind: "build",
            message: "building is only supported on Linux",
            safe_subject: Some("root"),
            remote_route_eligible: true,
        })
        .unwrap();

        let human = render_remediation_human(&record);
        let json = render_remediation_json(&record).unwrap();

        assert!(human.contains("mantle build --builder"));
        assert!(json.contains("mantle build --builder"));
        assert!(human.contains("network: required"));
        assert!(json.contains("\"network\":\"required\""));
    }

    #[test]
    fn remediation_redacts_secret_bearing_subjects() {
        let oversized = "x".repeat(SAFE_SUBJECT_BYTES_MAX.checked_add(1).unwrap());
        let protected_subjects = [
            "--token=super-secret",
            "/private/key/path",
            "PASSWORD=value",
            "root\nforged-line",
            oversized.as_str(),
        ];

        for protected in protected_subjects {
            let record = classify_remediation(FailureFacts {
                kind: "build",
                message: "nonzero exit code",
                safe_subject: Some(protected),
                remote_route_eligible: false,
            })
            .unwrap();
            let human = render_remediation_human(&record);
            let json = render_remediation_json(&record).unwrap();
            assert_eq!(record.safe_subject, "redacted");
            assert!(!human.contains(protected));
            assert!(!json.contains(protected));
        }
    }

    #[test]
    fn generic_failure_has_no_invented_action() {
        let record = classify_remediation(FailureFacts {
            kind: "build",
            message: "unexpected failure",
            safe_subject: None,
            remote_route_eligible: false,
        });

        assert!(record.is_none());
    }
}
