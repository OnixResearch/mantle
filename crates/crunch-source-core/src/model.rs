//! Structural wire values, locator boundary checks, and observation admission.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::digest::Blake3Digest;
use crate::digest::domain_digest;

/// Observation schema identifier.
pub const SOURCE_OBSERVATION_SCHEMA: &str = "mantle-source-observation-v1";

/// Observation encoding version.
pub const SOURCE_OBSERVATION_ENCODING_VERSION: u32 = 1;

/// Domain tag for the observation identity.
pub(crate) const OBSERVATION_DOMAIN: &[u8] = b"mantle-source-observation-v1\0";

/// Maximum locator bytes.
pub const MAX_LOCATOR_BYTES: usize = 4_096;

/// Maximum snapshot-profile name bytes.
pub const MAX_PROFILE_BYTES: usize = 64;

/// Maximum projection path components.
pub const MAX_PROJECTION_COMPONENTS: usize = 64;

/// Maximum admitted locator query fields.
pub const MAX_QUERY_FIELDS: usize = 8;

/// Query-field names that always reject as secret-bearing.
const SECRET_QUERY_FIELDS: [&str; 7] = ["token", "key", "password", "secret", "signature", "sig", "auth"];

/// One bounded diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDiagnostic {
    pub code: String,
    pub subject: String,
    pub message: String,
}

fn diagnostic(spec: (&str, &str, &str)) -> SourceDiagnostic {
    let (code, subject, message) = spec;
    debug_assert!(!code.is_empty() && !subject.is_empty() && !message.is_empty());
    SourceDiagnostic {
        code: String::from(code),
        subject: String::from(subject),
        message: String::from(message),
    }
}

/// Admitted source kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    FixedUrl,
    Git,
    LocalLogical,
    PackageMirror,
    Opaque,
}

/// Git object formats admitted as immutable revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitObjectFormat {
    /// Git SHA-1 object IDs are protocol interoperability, not stack identity.
    Sha1,
    Sha256,
}

impl GitObjectFormat {
    fn revision_hex_length(self) -> usize {
        match self {
            GitObjectFormat::Sha1 => 40,
            GitObjectFormat::Sha256 => 64,
        }
    }
}

/// Structural locator classes. Locators are observations, never authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "class", content = "value")]
pub enum LocatorClass {
    FixedUrl { url: String },
    GitRemote { remote: String },
    LocalLogical { path: String },
    PackageMirror { mirror: String },
    Opaque { label: String },
}

impl LocatorClass {
    /// The structural locator value.
    pub fn value(&self) -> &str {
        match self {
            LocatorClass::FixedUrl { url } => url,
            LocatorClass::GitRemote { remote } => remote,
            LocatorClass::LocalLogical { path } => path,
            LocatorClass::PackageMirror { mirror } => mirror,
            LocatorClass::Opaque { label } => label,
        }
    }

    fn class_name(&self) -> &'static str {
        match self {
            LocatorClass::FixedUrl { .. } => "fixed-url",
            LocatorClass::GitRemote { .. } => "git-remote",
            LocatorClass::LocalLogical { .. } => "local-logical",
            LocatorClass::PackageMirror { .. } => "package-mirror",
            LocatorClass::Opaque { .. } => "opaque",
        }
    }
}

/// A normalized relative projection path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionPath {
    components: Vec<String>,
}

impl ProjectionPath {
    /// Parse a normalized relative projection path.
    pub fn parse(value: &str) -> Result<Self, SourceDiagnostic> {
        if value.is_empty() || value.starts_with('/') || value.contains('\\') {
            return Err(diagnostic((
                "source-projection-unsafe",
                "projection",
                "projection must be a non-empty relative path",
            )));
        }
        let mut components = Vec::with_capacity(MAX_PROJECTION_COMPONENTS);
        for component in value.split('/') {
            if component.is_empty() || component == "." || component == ".." {
                return Err(diagnostic((
                    "source-projection-unsafe",
                    "projection",
                    "projection components must be normalized and non-empty",
                )));
            }
            if components.len() >= MAX_PROJECTION_COMPONENTS {
                return Err(diagnostic((
                    "source-projection-bound",
                    "projection",
                    "projection exceeds the fixed component bound",
                )));
            }
            components.push(String::from(component));
        }
        if components.is_empty() {
            return Err(diagnostic((
                "source-projection-empty",
                "projection",
                "projection must contain at least one component",
            )));
        }
        debug_assert!(!components.is_empty());
        debug_assert!(components.len() <= MAX_PROJECTION_COMPONENTS);
        Ok(Self { components })
    }

    /// The normalized projection value.
    pub fn as_string(&self) -> String {
        self.components.join("/")
    }

    /// The projection components.
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

/// Snapshot profile name and version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotProfile {
    pub name: String,
    pub version: u32,
}

impl SnapshotProfile {
    fn is_admissible(&self) -> bool {
        !self.name.is_empty() && self.name.len() <= MAX_PROFILE_BYTES && self.version >= 1
    }
}

/// Locator boundary policy: which query fields are approved at all.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocatorBoundaryPolicy {
    pub allowed_query_fields: Vec<String>,
}

/// Structural observation request before admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceObservationRequest {
    pub kind: SourceKind,
    pub locator: LocatorClass,
    /// Mutable ref recorded as a hint; never canonical identity.
    pub mutable_ref_hint: Option<String>,
    pub git_object_format: Option<GitObjectFormat>,
    pub git_revision: Option<String>,
    pub projection: ProjectionPath,
    pub snapshot_profile: SnapshotProfile,
    pub payload_blake3: Blake3Digest,
}

/// Admitted observation with its domain-separated identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceObservation {
    pub schema: String,
    pub kind: SourceKind,
    pub locator: LocatorClass,
    pub mutable_ref_hint: Option<String>,
    pub git_object_format: Option<GitObjectFormat>,
    pub git_revision: Option<String>,
    pub projection: ProjectionPath,
    pub snapshot_profile: SnapshotProfile,
    pub payload_blake3: Blake3Digest,
    pub encoding_version: u32,
    pub observation_blake3: Blake3Digest,
}

/// Admission outcome: one observation or ordered diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceObservationResult {
    pub observation: Option<SourceObservation>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

/// Identity input: canonical fields only. Mutable hints stay outside identity.
#[derive(Serialize)]
struct ObservationIdentityInput<'a> {
    schema: &'a str,
    kind: SourceKind,
    locator_class: &'a str,
    git_object_format: Option<GitObjectFormat>,
    git_revision: Option<&'a str>,
    projection: &'a ProjectionPath,
    snapshot_profile: &'a SnapshotProfile,
    payload_blake3: &'a Blake3Digest,
    encoding_version: u32,
}

/// Admit a structural request into a checked source observation.
pub fn admit_source_observation(
    request: &SourceObservationRequest,
    policy: &LocatorBoundaryPolicy,
) -> SourceObservationResult {
    let mut diagnostics = Vec::new();
    check_locator(request.locator.value(), policy, &mut diagnostics);
    check_kind_agreement(request, &mut diagnostics);
    check_git_facts(request, &mut diagnostics);
    check_profile_and_projection(request, &mut diagnostics);
    if !diagnostics.is_empty() {
        return rejected(ordered(diagnostics));
    }
    let identity_input = ObservationIdentityInput {
        schema: SOURCE_OBSERVATION_SCHEMA,
        kind: request.kind,
        locator_class: request.locator.class_name(),
        git_object_format: request.git_object_format,
        git_revision: request.git_revision.as_deref(),
        projection: &request.projection,
        snapshot_profile: &request.snapshot_profile,
        payload_blake3: &request.payload_blake3,
        encoding_version: SOURCE_OBSERVATION_ENCODING_VERSION,
    };
    let identity = match domain_digest(OBSERVATION_DOMAIN, &identity_input) {
        Ok(identity) => identity,
        Err(_) => {
            return rejected(ordered(vec![diagnostic((
                "source-observation-identity-failed",
                "observation",
                "source observation could not be canonically identified",
            ))]));
        }
    };
    let observation = SourceObservation {
        schema: String::from(SOURCE_OBSERVATION_SCHEMA),
        kind: request.kind,
        locator: request.locator.clone(),
        mutable_ref_hint: request.mutable_ref_hint.clone(),
        git_object_format: request.git_object_format,
        git_revision: request.git_revision.clone(),
        projection: request.projection.clone(),
        snapshot_profile: request.snapshot_profile.clone(),
        payload_blake3: request.payload_blake3.clone(),
        encoding_version: SOURCE_OBSERVATION_ENCODING_VERSION,
        observation_blake3: identity,
    };
    debug_assert!(observation.mutable_ref_hint.is_none() || observation.observation_blake3.as_str().len() == 64);
    SourceObservationResult {
        observation: Some(observation),
        diagnostics: Vec::new(),
    }
}

fn check_locator(locator: &str, policy: &LocatorBoundaryPolicy, diagnostics: &mut Vec<SourceDiagnostic>) {
    let blocker_count_before = diagnostics.len();
    if locator.is_empty() || locator.len() > MAX_LOCATOR_BYTES {
        diagnostics.push(diagnostic((
            "source-locator-bound",
            "locator",
            "locator must be non-empty and within the named byte bound",
        )));
        return;
    }
    if locator.contains('@') {
        diagnostics.push(diagnostic((
            "source-locator-userinfo",
            "locator",
            "locator must not carry userinfo or display credentials",
        )));
    }
    if locator.contains('#') {
        diagnostics.push(diagnostic(("source-locator-fragment", "locator", "locator fragments are not admitted")));
    }
    let Some((_, query)) = locator.split_once('?') else {
        return;
    };
    let fields: Vec<&str> = query.split('&').filter(|field| !field.is_empty()).collect();
    if fields.len() > MAX_QUERY_FIELDS {
        diagnostics.push(diagnostic((
            "source-locator-query-bound",
            "locator",
            "locator query exceeds the fixed field bound",
        )));
        return;
    }
    for field in fields {
        let name = field.split('=').next().unwrap_or(field);
        let is_secret = SECRET_QUERY_FIELDS.iter().any(|secret| secret.eq_ignore_ascii_case(name));
        if is_secret {
            diagnostics.push(diagnostic((
                "source-locator-secret",
                "locator",
                "locator must not carry secret-bearing query fields",
            )));
            continue;
        }
        let is_approved = policy.allowed_query_fields.iter().any(|allowed| allowed == name);
        if !is_approved {
            diagnostics.push(diagnostic((
                "source-locator-unapproved-query",
                "locator",
                "locator query fields must be approved by policy",
            )));
        }
    }
    debug_assert!(diagnostics.len() >= blocker_count_before);
    debug_assert!(policy.allowed_query_fields.iter().all(|field| !field.is_empty()));
}

fn check_kind_agreement(request: &SourceObservationRequest, diagnostics: &mut Vec<SourceDiagnostic>) {
    let is_kind_and_locator_agreeing = matches!(
        (request.kind, &request.locator),
        (SourceKind::FixedUrl, LocatorClass::FixedUrl { .. })
            | (SourceKind::Git, LocatorClass::GitRemote { .. })
            | (SourceKind::LocalLogical, LocatorClass::LocalLogical { .. })
            | (SourceKind::PackageMirror, LocatorClass::PackageMirror { .. })
            | (SourceKind::Opaque, LocatorClass::Opaque { .. })
    );
    if !is_kind_and_locator_agreeing {
        diagnostics.push(diagnostic((
            "source-kind-locator-mismatch",
            "locator",
            "source kind and locator class must agree",
        )));
    }
}

fn check_git_facts(request: &SourceObservationRequest, diagnostics: &mut Vec<SourceDiagnostic>) {
    let blocker_count_before = diagnostics.len();
    let is_git = request.kind == SourceKind::Git;
    if is_git {
        let Some(revision) = request.git_revision.as_deref() else {
            diagnostics.push(diagnostic((
                "source-git-revision-required",
                "git-revision",
                "git observations require an immutable revision; a mutable ref is not identity",
            )));
            return;
        };
        let Some(format) = request.git_object_format else {
            diagnostics.push(diagnostic((
                "source-git-format-required",
                "git-object-format",
                "git observations require an explicit object format",
            )));
            return;
        };
        let is_hex_revision = revision.len() == format.revision_hex_length()
            && revision.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !is_hex_revision {
            diagnostics.push(diagnostic((
                "source-git-revision-format",
                "git-revision",
                "immutable revision must be lowercase hex for the declared object format",
            )));
        }
        debug_assert!(diagnostics.len() >= blocker_count_before);
        debug_assert!(!revision.is_empty());
        return;
    }
    if request.git_revision.is_some() || request.git_object_format.is_some() {
        diagnostics.push(diagnostic((
            "source-git-facts-without-git-kind",
            "git-facts",
            "git revision and object format are admitted only for git sources",
        )));
    }
}

fn check_profile_and_projection(request: &SourceObservationRequest, diagnostics: &mut Vec<SourceDiagnostic>) {
    if !request.snapshot_profile.is_admissible() {
        diagnostics.push(diagnostic((
            "source-profile-inadmissible",
            "snapshot-profile",
            "snapshot profile name and version must be non-empty, bounded, and at least version one",
        )));
    }
    if request.projection.components().is_empty() {
        diagnostics.push(diagnostic((
            "source-projection-empty",
            "projection",
            "projection must contain at least one component",
        )));
    }
}

fn ordered(mut diagnostics: Vec<SourceDiagnostic>) -> Vec<SourceDiagnostic> {
    diagnostics.sort_by(|left, right| (&left.code, &left.subject).cmp(&(&right.code, &right.subject)));
    diagnostics.dedup();
    debug_assert!(!diagnostics.is_empty());
    diagnostics
}

fn rejected(diagnostics: Vec<SourceDiagnostic>) -> SourceObservationResult {
    debug_assert!(!diagnostics.is_empty());
    SourceObservationResult {
        observation: None,
        diagnostics,
    }
}
