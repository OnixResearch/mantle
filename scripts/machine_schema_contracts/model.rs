use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub const REGISTRY_SCHEMA: &str = "mantle-machine-artifact-registry-v1";
pub const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
pub const REGISTRY_DATA_BEGIN: &str = "# registry-data-begin";
pub const REGISTRY_DATA_END: &str = "# registry-data-end";
pub const PUBLIC_MARKER_PREFIX: &str = "// machine-artifact-public:";
pub const CONTRACTED_CLASS: &str = "contracted";
pub const PRELUDE_PATH: &str = "schemas/machine-contracts/prelude.ncl";
pub const BLAKE3_HEX_LENGTH_CHARS: u32 = 64;
pub const MAX_REGISTRY_SURFACES: u32 = 512;
pub const MAX_REGISTRY_LIST_ITEMS: u32 = 4_096;
pub const MAX_NEGATIVE_FIXTURE_CASES: u32 = 4_096;
pub const MAX_SCHEMA_DEPTH: u32 = 64;
pub const MAX_RUST_TYPE_DEPTH: u32 = 16;
pub const MAX_VALIDATION_ISSUES: u32 = 256;
pub const MAX_INVARIANTS_PER_SCHEMA: u32 = 32;
pub const MAX_CONTRACT_DEFINITIONS: u32 = 256;
pub const MAX_SCHEMA_PROPERTIES: u32 = 4_096;
pub const MAX_ENUM_VALUES: u32 = 4_096;
pub const MAX_SAFE_REFERENCE_BYTES: u32 = 4_096;
pub const MAX_REDACTION_TEXT_BYTES: u32 = 16_384;
pub const MIN_REQUIRED_FIXTURE_CASES: u32 = 1;
pub const BOUND_KEYWORDS: &[&str] = &[
    "minItems",
    "maxItems",
    "minLength",
    "maxLength",
    "minProperties",
    "maxProperties",
    "minimum",
    "maximum",
];
pub const EXPECTED_ISSUE_CLASSES: &[&str] = &[
    "schema",
    "version",
    "enum",
    "digest",
    "reference",
    "bounds",
    "unknown-field",
    "redaction",
    "cross-field",
];
pub const SUPPORTED_SURFACE_CLASSES: &[&str] = &["contracted", "internal", "debug", "compatibility", "external"];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Registry {
    pub registry_schema: String,
    pub prelude_path: String,
    pub supported_classes: Vec<String>,
    pub initial_cohort: Vec<String>,
    pub surfaces: Vec<Surface>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Surface {
    pub id: String,
    pub class: String,
    pub rust_owner: String,
    pub producer: Producer,
    pub consumers: Vec<String>,
    pub artifacts: Artifacts,
    pub version_policy: VersionPolicy,
    pub validation_commands: Vec<String>,
    pub fixture_coverage: Vec<String>,
    pub freshness_strategy: String,
    pub freshness: Freshness,
    pub non_claims: Vec<String>,
    pub rationale: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Producer {
    pub command_or_api: String,
    pub marker: String,
    pub source_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Artifacts {
    pub schema: String,
    pub generated_contract: String,
    pub positive_fixtures: Vec<String>,
    pub negative_fixture_set: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VersionPolicy {
    pub current: String,
    pub supported: Vec<String>,
    pub unknown_version_behavior: String,
    pub compatibility_converter: String,
    pub compatibility_fixtures: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Freshness {
    pub schema_blake3: String,
    pub contract_blake3: String,
    pub prelude_blake3: String,
    pub fixture_set_blake3: String,
    pub producer_identity_blake3: String,
    pub consumer_policy_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NegativeFixtureSet {
    pub surface_id: String,
    pub cases: Vec<NegativeFixtureCase>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NegativeFixtureCase {
    pub id: String,
    pub issue_class: String,
    pub expected_path: String,
    pub artifact: Value,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Issue {
    pub surface_id: String,
    pub class: String,
    pub path: String,
    pub message: String,
}

impl Issue {
    pub fn registry(class: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            surface_id: "registry".to_string(),
            class: class.to_string(),
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn surface(surface_id: &str, class: &str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.to_string(),
            class: class.to_string(),
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Invariant {
    pub kind: String,
    #[serde(default)]
    pub array: String,
    #[serde(default)]
    pub other_array: String,
    #[serde(default)]
    pub integer: String,
    #[serde(default)]
    pub boolean: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub terms: Vec<String>,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub item_field: String,
    #[serde(default)]
    pub other_item_field: String,
    #[serde(default)]
    pub value: Value,
}

pub fn push_issue(issues: &mut Vec<Issue>, issue: Issue) {
    let issue_count = u32::try_from(issues.len()).unwrap_or(u32::MAX);
    if issue_count < MAX_VALIDATION_ISSUES {
        issues.push(issue);
    }
}
