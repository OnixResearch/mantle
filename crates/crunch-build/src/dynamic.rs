//! Staged admission for dynamically discovered native derivations.
//!
//! The pure core parses, validates, resolves identity facts, and creates one
//! registry plan. The Worker owns castore reads, registry observations,
//! diagnostics, registry mutation, goal creation, waiters, and dispatch.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::build_text_path_with_store_dir;
use snix_castore::Node;

use crate::Error;

// r[impl dynamic_derivation_admission.staged_core]
// r[impl dynamic_derivation_admission.versioned_forms]
// r[impl dynamic_derivation_admission.complete_parent_identity]
// r[impl dynamic_derivation_admission.compatibility]

const DEFAULT_STORE_PREFIX: &str = "/nix/store";
const TRADITIONAL_PREFIX: &[u8] = b"Derive(";
const VERSIONED_PREFIX: &[u8] = b"DrvWithVersion(";
const SUPPORTED_VERSION_PREFIX: &[u8] = b"DrvWithVersion(\"xp-dyn-drv\",";
const MAX_DRV_SIZE_BYTES: u64 = 4_194_304;
const MAX_FIELD_BYTES: u64 = 1_048_576;
const MAX_COLLECTION_ENTRIES: u32 = 16_384;
const MAX_PARENT_EDGES: u32 = 4_096;
const MAX_DYNAMIC_NODES: u32 = 16_384;
const MAX_DYNAMIC_DEPTH: u32 = 64;
const MAX_PARSER_COLLECTIONS: u32 = 65_536;
const BLAKE3_DIGEST_BYTES: usize = 32;
const VERSIONED_FIELD_COUNT: usize = 7;
const INITIAL_COLLECTION_CAPACITY: usize = 16;
const INITIAL_RENDER_CAPACITY_BYTES: usize = 1_024;
const FRAME_TAG: &[u8] = b"mantle-native-dynamic-derivation-v1\0";
const FULL_IDENTITY_TAG: &[u8] = b"mantle-native-dynamic-admission-identity-v1\0";

/// Fixed admission limits for one dynamic derivation candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DynamicAdmissionLimits {
    pub bytes_max: u64,
    pub field_bytes_max: u64,
    pub collection_entries_max: u32,
    pub parent_edges_max: u32,
    pub dynamic_nodes_max: u32,
    pub dynamic_depth_max: u32,
    pub parser_collections_max: u32,
}

impl Default for DynamicAdmissionLimits {
    fn default() -> Self {
        Self {
            bytes_max: MAX_DRV_SIZE_BYTES,
            field_bytes_max: MAX_FIELD_BYTES,
            collection_entries_max: MAX_COLLECTION_ENTRIES,
            parent_edges_max: MAX_PARENT_EDGES,
            dynamic_nodes_max: MAX_DYNAMIC_NODES,
            dynamic_depth_max: MAX_DYNAMIC_DEPTH,
            parser_collections_max: MAX_PARSER_COLLECTIONS,
        }
    }
}

/// Stable blocker classes for staged native admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DynamicAdmissionError {
    InvalidPolicy(&'static str),
    PlatformLimit(&'static str),
    SizeLimit {
        observed: u64,
        limit: u64,
    },
    FieldLimit {
        field: &'static str,
        observed: u64,
        limit: u64,
    },
    CollectionLimit {
        field: &'static str,
        observed: u32,
        limit: u32,
    },
    DepthLimit {
        observed: u32,
        limit: u32,
    },
    Syntax {
        offset: u64,
        detail: String,
    },
    Semantic(String),
    UnsupportedVersion(String),
    UnsupportedOutput(String),
    EmptyRequest(String),
    MixedPrefix(String),
    MissingParent(String),
    DuplicateParent(String),
    ConflictingParent(String),
    UnexpectedParent(String),
    WrongParentPrefix(String),
    WrongHashDomain(String),
    Identity(String),
    PathCollision(String),
}

impl DynamicAdmissionError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidPolicy(_) => "dynamic-admission-invalid-policy",
            Self::PlatformLimit(_) => "dynamic-admission-platform-limit",
            Self::SizeLimit { .. } | Self::FieldLimit { .. } | Self::CollectionLimit { .. } => {
                "dynamic-admission-limit"
            }
            Self::DepthLimit { .. } => "dynamic-admission-depth-limit",
            Self::Syntax { .. } => "dynamic-admission-syntax",
            Self::Semantic(_) => "dynamic-admission-semantic",
            Self::UnsupportedVersion(_) => "dynamic-admission-unsupported-version",
            Self::UnsupportedOutput(_) => "dynamic-admission-unsupported-output",
            Self::EmptyRequest(_) => "dynamic-admission-empty-request",
            Self::MixedPrefix(_) => "dynamic-admission-mixed-prefix",
            Self::MissingParent(_) => "dynamic-admission-missing-parent",
            Self::DuplicateParent(_) => "dynamic-admission-duplicate-parent",
            Self::ConflictingParent(_) => "dynamic-admission-conflicting-parent",
            Self::UnexpectedParent(_) => "dynamic-admission-unexpected-parent",
            Self::WrongParentPrefix(_) => "dynamic-admission-wrong-parent-prefix",
            Self::WrongHashDomain(_) => "dynamic-admission-wrong-hash-domain",
            Self::Identity(_) => "dynamic-admission-identity",
            Self::PathCollision(_) => "dynamic-admission-path-collision",
        }
    }
}

impl fmt::Display for DynamicAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.code())?;
        match self {
            Self::InvalidPolicy(field) => write!(formatter, "invalid nonzero limit `{field}`"),
            Self::PlatformLimit(field) => write!(formatter, "{field} exceeds the supported integer range"),
            Self::SizeLimit { observed, limit } => {
                write!(formatter, "candidate has {observed} bytes; limit is {limit}")
            }
            Self::FieldLimit { field, observed, limit } => {
                write!(formatter, "{field} has {observed} bytes; limit is {limit}")
            }
            Self::CollectionLimit { field, observed, limit } => {
                write!(formatter, "{field} has {observed} entries; limit is {limit}")
            }
            Self::DepthLimit { observed, limit } => write!(formatter, "depth {observed} exceeds limit {limit}"),
            Self::Syntax { offset, detail } => write!(formatter, "syntax error at byte {offset}: {detail}"),
            Self::Semantic(detail)
            | Self::UnsupportedVersion(detail)
            | Self::UnsupportedOutput(detail)
            | Self::EmptyRequest(detail)
            | Self::MixedPrefix(detail)
            | Self::MissingParent(detail)
            | Self::DuplicateParent(detail)
            | Self::ConflictingParent(detail)
            | Self::UnexpectedParent(detail)
            | Self::WrongParentPrefix(detail)
            | Self::WrongHashDomain(detail)
            | Self::Identity(detail)
            | Self::PathCollision(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for DynamicAdmissionError {}

impl From<DynamicAdmissionError> for Error {
    fn from(error: DynamicAdmissionError) -> Self {
        Error::Store(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateForm {
    Traditional,
    Versioned,
}

/// Digest roles cannot cross the native and Nix compatibility domains.
#[allow(dead_code)] // The rejected Nix role is constructed by compatibility boundaries and negative tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DynamicHashDomain {
    MantleBlake3,
    NixSha256,
}

/// One shell-observed direct-parent identity fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicParentHashFact {
    pub logical_drv_path: String,
    pub digest: [u8; BLAKE3_DIGEST_BYTES],
    pub domain: DynamicHashDomain,
}

impl DynamicParentHashFact {
    pub(crate) fn native(logical_drv_path: String, digest: [u8; BLAKE3_DIGEST_BYTES]) -> Self {
        Self {
            logical_drv_path,
            digest,
            domain: DynamicHashDomain::MantleBlake3,
        }
    }
}

/// One flattened dynamic-output node in deterministic preorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicOutputRequest {
    pub parent_node_index: Option<u32>,
    pub output_name: String,
    pub requested_outputs: BTreeSet<String>,
}

/// Requests tied to one direct parent derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicParentRequest {
    pub drv_path: StorePath<String>,
    pub direct_outputs: BTreeSet<String>,
    pub dynamic_outputs: Vec<DynamicOutputRequest>,
}

/// Detected dynamic derivation returned to the scheduler shell.
#[derive(Debug, Clone)]
pub struct DynamicDrv {
    pub output_name: String,
    pub drv_store_path: StorePath<String>,
    pub derivation: Derivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityProjection {
    AsParsed,
    /// Native output paths were absent when their parent-compatible hash was computed.
    OutputsMasked,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedDynamicDerivation {
    candidate_output_path: StorePath<String>,
    derivation: Derivation,
    form: CandidateForm,
    identity_projection: IdentityProjection,
    name: String,
    source_bytes: Vec<u8>,
    parent_requests: Vec<DynamicParentRequest>,
}

#[derive(Debug, Clone)]
pub(crate) struct ValidatedDynamicDerivation {
    parsed: ParsedDynamicDerivation,
    logical_store_prefix: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IdentityResolvedDynamicDerivation {
    validated: ValidatedDynamicDerivation,
    drv_path: StorePath<String>,
    hash_derivation_modulo: [u8; BLAKE3_DIGEST_BYTES],
    full_identity: [u8; BLAKE3_DIGEST_BYTES],
    content_addressed: bool,
}

impl IdentityResolvedDynamicDerivation {
    pub(crate) fn drv_path(&self) -> &StorePath<String> {
        &self.drv_path
    }

    pub(crate) fn logical_store_prefix(&self) -> &str {
        &self.validated.logical_store_prefix
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DynamicRegistrationDecision {
    Insert,
    AlreadyPresent,
}

/// Existing registry facts observed by the Worker before planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExistingDynamicRegistration {
    pub logical_drv_path: String,
    pub full_identity: Option<[u8; BLAKE3_DIGEST_BYTES]>,
}

/// The only value accepted by dynamic registry mutation.
#[derive(Debug, Clone)]
pub(crate) struct RegistryReadyDynamicDerivation {
    resolved: IdentityResolvedDynamicDerivation,
    decision: DynamicRegistrationDecision,
}

impl RegistryReadyDynamicDerivation {
    pub(crate) fn decision(&self) -> DynamicRegistrationDecision {
        self.decision
    }

    pub(crate) fn drv_path(&self) -> &StorePath<String> {
        &self.resolved.drv_path
    }

    pub(crate) fn hash_derivation_modulo(&self) -> [u8; BLAKE3_DIGEST_BYTES] {
        self.resolved.hash_derivation_modulo
    }

    pub(crate) fn full_identity(&self) -> [u8; BLAKE3_DIGEST_BYTES] {
        self.resolved.full_identity
    }

    pub(crate) fn derivation(&self) -> &Derivation {
        &self.resolved.validated.parsed.derivation
    }

    pub(crate) fn content_addressed(&self) -> bool {
        self.resolved.content_addressed
    }

    pub(crate) fn into_registration(self) -> (StorePath<String>, [u8; BLAKE3_DIGEST_BYTES], Derivation, bool, [u8; BLAKE3_DIGEST_BYTES]) {
        let resolved = self.resolved;
        (
            resolved.drv_path,
            resolved.hash_derivation_modulo,
            resolved.validated.parsed.derivation,
            resolved.content_addressed,
            resolved.full_identity,
        )
    }

    pub(crate) fn into_dynamic_drv(self, output_name: String) -> DynamicDrv {
        DynamicDrv {
            output_name,
            drv_store_path: self.resolved.drv_path,
            derivation: self.resolved.validated.parsed.derivation,
        }
    }
}

/// Check whether one output node is a bounded regular `.drv` candidate.
pub fn is_drv_output(output_path: &StorePath<String>, node: &Node) -> bool {
    if !output_path.name().ends_with(".drv") {
        return false;
    }
    matches!(node, Node::File { size, .. } if *size <= MAX_DRV_SIZE_BYTES)
}

/// Parse candidate bytes into the first private state.
#[allow(tigerstyle::assertion_density)] // Untrusted input failures return stable errors instead of assertions.
pub(crate) fn parse_dynamic_candidate(
    content: &[u8],
    candidate_output_path: &StorePath<String>,
    logical_store_prefix: &str,
    limits: DynamicAdmissionLimits,
) -> Result<Option<ParsedDynamicDerivation>, DynamicAdmissionError> {
    validate_limits(limits)?;
    validate_candidate_size(content, limits)?;
    let Some(form) = classify_candidate(content)? else {
        return Ok(None);
    };
    let (derivation, parent_requests) = match form {
        CandidateForm::Traditional => parse_traditional(content, logical_store_prefix)?,
        CandidateForm::Versioned => parse_versioned(content, logical_store_prefix, limits)?,
    };
    let name = derivation_name(&derivation, limits)?;
    Ok(Some(ParsedDynamicDerivation {
        candidate_output_path: candidate_output_path.clone(),
        derivation,
        form,
        identity_projection: IdentityProjection::AsParsed,
        name,
        source_bytes: content.to_vec(),
        parent_requests,
    }))
}

/// Admit a derivation constructed from the already checked native-plan wire.
/// Content-addressed native units have no output path, so they cannot round-trip
/// through the traditional ATerm parser, which requires a populated output path.
/// They still pass the same collection, parent-fact, identity, and registry gates.
pub(crate) fn admit_native_plan_derivation(
    derivation: Derivation,
    candidate_output_path: StorePath<String>,
    logical_store_prefix: &str,
    limits: DynamicAdmissionLimits,
) -> Result<ValidatedDynamicDerivation, DynamicAdmissionError> {
    validate_limits(limits)?;
    validate_store_prefix(logical_store_prefix)?;
    let name = derivation_name(&derivation, limits)?;
    let parent_requests = derivation
        .input_derivations
        .iter()
        .map(|(drv_path, outputs)| DynamicParentRequest {
            drv_path: drv_path.clone(),
            direct_outputs: outputs.clone(),
            dynamic_outputs: Vec::new(),
        })
        .collect();
    let source_bytes = derivation.to_aterm_bytes_with_store_dir(logical_store_prefix);
    validate_candidate_size(&source_bytes, limits)?;
    validate_dynamic_candidate(
        ParsedDynamicDerivation {
            candidate_output_path,
            derivation,
            form: CandidateForm::Traditional,
            identity_projection: IdentityProjection::OutputsMasked,
            name,
            source_bytes,
            parent_requests,
        },
        logical_store_prefix,
        limits,
    )
}

/// Validate semantic and collection invariants before identity work.
pub(crate) fn validate_dynamic_candidate(
    parsed: ParsedDynamicDerivation,
    logical_store_prefix: &str,
    limits: DynamicAdmissionLimits,
) -> Result<ValidatedDynamicDerivation, DynamicAdmissionError> {
    validate_store_prefix(logical_store_prefix)?;
    validate_candidate_output(&parsed.candidate_output_path)?;
    validate_derivation_collections(&parsed.derivation, limits)?;
    validate_parent_requests(&parsed.parent_requests, parsed.form, limits)?;
    validate_output_semantics(&parsed.derivation, parsed.form)?;
    Ok(ValidatedDynamicDerivation {
        parsed,
        logical_store_prefix: logical_store_prefix.to_string(),
    })
}

/// Return the complete direct-parent set needed from the Worker shell.
pub(crate) fn required_parent_paths(validated: &ValidatedDynamicDerivation) -> Vec<String> {
    validated
        .parsed
        .parent_requests
        .iter()
        .map(|request| request.drv_path.to_absolute_path_with_prefix(&validated.logical_store_prefix))
        .collect()
}

/// Resolve native identity from explicit parent facts.
pub(crate) fn resolve_dynamic_identity(
    mut validated: ValidatedDynamicDerivation,
    parent_facts: &[DynamicParentHashFact],
) -> Result<IdentityResolvedDynamicDerivation, DynamicAdmissionError> {
    let fact_map = validate_parent_facts(&validated, parent_facts)?;
    let hdm = compute_hash_derivation_modulo(&mut validated, &fact_map)?;
    let drv_path = compute_derivation_path(&validated)?;
    let is_content_addressed = validated
        .parsed
        .derivation
        .outputs
        .values()
        .all(|output| output.path.is_none() && output.ca_hash.is_none());
    let full_identity = compute_full_identity(&validated, &drv_path, hdm)?;
    Ok(IdentityResolvedDynamicDerivation {
        validated,
        drv_path,
        hash_derivation_modulo: hdm,
        full_identity,
        content_addressed: is_content_addressed,
    })
}

/// Plan insertion or an exact idempotent duplicate without mutation.
pub(crate) fn plan_dynamic_registration(
    resolved: IdentityResolvedDynamicDerivation,
    existing: Option<&ExistingDynamicRegistration>,
) -> Result<RegistryReadyDynamicDerivation, DynamicAdmissionError> {
    let expected_path = resolved.drv_path.to_absolute_path_with_prefix(&resolved.validated.logical_store_prefix);
    let decision = match existing {
        None => DynamicRegistrationDecision::Insert,
        Some(fact) if fact.logical_drv_path != expected_path => {
            return Err(DynamicAdmissionError::PathCollision(format!(
                "observed registry path `{}` differs from planned path `{expected_path}`",
                fact.logical_drv_path
            )));
        }
        Some(fact) if fact.full_identity == Some(resolved.full_identity) => DynamicRegistrationDecision::AlreadyPresent,
        Some(_) => {
            return Err(DynamicAdmissionError::PathCollision(format!(
                "path `{expected_path}` already has a different admitted identity"
            )));
        }
    };
    Ok(RegistryReadyDynamicDerivation { resolved, decision })
}

/// Reject collisions and select one representative for each exact duplicate.
#[allow(tigerstyle::unbounded_collection_growth)] // BTree collections cannot reserve; `ready` is the explicit batch bound.
pub(crate) fn validate_registry_ready_batch(
    ready: &[RegistryReadyDynamicDerivation],
) -> Result<BTreeSet<u32>, DynamicAdmissionError> {
    let mut identities: BTreeMap<String, [u8; BLAKE3_DIGEST_BYTES]> = BTreeMap::new();
    let mut selected = BTreeSet::new();
    for (index, item) in ready.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| DynamicAdmissionError::InvalidPolicy("registry-ready batch index exceeds u32"))?;
        let path = item.drv_path().to_absolute_path_with_prefix(&item.resolved.validated.logical_store_prefix);
        if let Some(previous) = identities.get(&path) {
            if previous != &item.full_identity() {
                return Err(DynamicAdmissionError::PathCollision(format!(
                    "batch contains conflicting identities for `{path}`"
                )));
            }
            continue;
        }
        identities.insert(path, item.full_identity());
        selected.insert(index);
    }
    debug_assert!(selected.len() <= ready.len(), "selected dynamic batch cannot grow");
    Ok(selected)
}

#[allow(tigerstyle::assertion_density)] // Every invalid limit returns a typed policy error.
fn validate_limits(limits: DynamicAdmissionLimits) -> Result<(), DynamicAdmissionError> {
    let values = [
        ("bytes_max", limits.bytes_max),
        ("field_bytes_max", limits.field_bytes_max),
        ("collection_entries_max", u64::from(limits.collection_entries_max)),
        ("parent_edges_max", u64::from(limits.parent_edges_max)),
        ("dynamic_nodes_max", u64::from(limits.dynamic_nodes_max)),
        ("dynamic_depth_max", u64::from(limits.dynamic_depth_max)),
        ("parser_collections_max", u64::from(limits.parser_collections_max)),
    ];
    for (name, value) in values {
        if value == 0 {
            return Err(DynamicAdmissionError::InvalidPolicy(name));
        }
    }
    if limits.bytes_max > MAX_DRV_SIZE_BYTES {
        return Err(DynamicAdmissionError::InvalidPolicy("bytes_max"));
    }
    if limits.dynamic_depth_max > MAX_DYNAMIC_DEPTH {
        return Err(DynamicAdmissionError::InvalidPolicy("dynamic_depth_max"));
    }
    Ok(())
}

fn validate_candidate_size(content: &[u8], limits: DynamicAdmissionLimits) -> Result<(), DynamicAdmissionError> {
    let observed = usize_to_u64(content.len(), "candidate byte length")?;
    if observed > limits.bytes_max {
        return Err(DynamicAdmissionError::SizeLimit {
            observed,
            limit: limits.bytes_max,
        });
    }
    Ok(())
}

fn classify_candidate(content: &[u8]) -> Result<Option<CandidateForm>, DynamicAdmissionError> {
    if content.starts_with(TRADITIONAL_PREFIX) {
        return Ok(Some(CandidateForm::Traditional));
    }
    if content.starts_with(SUPPORTED_VERSION_PREFIX) {
        return Ok(Some(CandidateForm::Versioned));
    }
    if content.starts_with(VERSIONED_PREFIX) {
        let Some(version) = version_tag(content) else {
            return Err(DynamicAdmissionError::Syntax {
                offset: 0,
                detail: "malformed version tag".to_string(),
            });
        };
        return Err(DynamicAdmissionError::UnsupportedVersion(version));
    }
    Ok(None)
}

fn version_tag(content: &[u8]) -> Option<String> {
    let mut cursor = Cursor::new(content, DynamicAdmissionLimits::default());
    cursor.expect_bytes(VERSIONED_PREFIX).ok()?;
    String::from_utf8(cursor.parse_string().ok()?).ok()
}

fn parse_traditional(
    content: &[u8],
    logical_store_prefix: &str,
) -> Result<(Derivation, Vec<DynamicParentRequest>), DynamicAdmissionError> {
    let mut derivation = parse_legacy_projection(content, logical_store_prefix)?;
    restore_custom_prefix_fields(&mut derivation, logical_store_prefix)?;
    let parent_requests = derivation
        .input_derivations
        .iter()
        .map(|(drv_path, outputs)| DynamicParentRequest {
            drv_path: drv_path.clone(),
            direct_outputs: outputs.clone(),
            dynamic_outputs: Vec::new(),
        })
        .collect();
    Ok((derivation, parent_requests))
}

fn parse_versioned(
    content: &[u8],
    logical_store_prefix: &str,
    limits: DynamicAdmissionLimits,
) -> Result<(Derivation, Vec<DynamicParentRequest>), DynamicAdmissionError> {
    let fields = split_versioned_fields(content, limits)?;
    let parent_requests = parse_parent_requests(fields[1], logical_store_prefix, limits)?;
    let direct_input_field = render_direct_input_field(&parent_requests, logical_store_prefix);
    let traditional = render_traditional_projection(&fields, &direct_input_field);
    let mut derivation = parse_legacy_projection(&traditional, logical_store_prefix)?;
    restore_custom_prefix_fields(&mut derivation, logical_store_prefix)?;
    Ok((derivation, parent_requests))
}

fn split_versioned_fields(
    content: &[u8],
    limits: DynamicAdmissionLimits,
) -> Result<[&[u8]; VERSIONED_FIELD_COUNT], DynamicAdmissionError> {
    let mut cursor = Cursor::new(content, limits);
    cursor.expect_bytes(SUPPORTED_VERSION_PREFIX)?;
    let mut fields: Vec<&[u8]> = Vec::with_capacity(VERSIONED_FIELD_COUNT);
    for index in 0..VERSIONED_FIELD_COUNT {
        fields.push(cursor.take_value()?);
        if index < VERSIONED_FIELD_COUNT.saturating_sub(1) {
            cursor.expect_byte(b',')?;
        }
    }
    cursor.expect_byte(b')')?;
    cursor.expect_end()?;
    fields.try_into().map_err(|_| DynamicAdmissionError::Syntax {
        offset: 0,
        detail: "versioned derivation does not contain seven fields".to_string(),
    })
}

#[allow(tigerstyle::assertion_density)] // Parser checks reject untrusted input without panicking.
fn parse_parent_requests(
    bytes: &[u8],
    logical_store_prefix: &str,
    limits: DynamicAdmissionLimits,
) -> Result<Vec<DynamicParentRequest>, DynamicAdmissionError> {
    let mut cursor = Cursor::new(bytes, limits);
    cursor.expect_byte(b'[')?;
    let mut requests = Vec::with_capacity(INITIAL_COLLECTION_CAPACITY);
    while cursor.peek() != Some(b']') {
        cursor.expect_byte(b'(')?;
        let path_bytes = cursor.parse_string()?;
        cursor.expect_byte(b',')?;
        let drv_path = parse_parent_path(&path_bytes, logical_store_prefix)?;
        let (direct_outputs, dynamic_outputs) = cursor.parse_input_node()?;
        cursor.expect_byte(b')')?;
        if direct_outputs.is_empty() && dynamic_outputs.is_empty() {
            return Err(DynamicAdmissionError::EmptyRequest(format!(
                "parent `{}` requests no direct or dynamic outputs",
                drv_path
            )));
        }
        requests.push(DynamicParentRequest {
            drv_path,
            direct_outputs,
            dynamic_outputs,
        });
        enforce_count("input derivations", requests.len(), limits.collection_entries_max)?;
        if cursor.peek() == Some(b',') {
            cursor.expect_byte(b',')?;
        } else if cursor.peek() != Some(b']') {
            return cursor.syntax("expected comma or closing input list");
        }
    }
    cursor.expect_byte(b']')?;
    cursor.expect_end()?;
    Ok(requests)
}

fn parse_parent_path(bytes: &[u8], store_prefix: &str) -> Result<StorePath<String>, DynamicAdmissionError> {
    StorePath::from_absolute_path_with_prefix(bytes, store_prefix).map_err(|_| {
        DynamicAdmissionError::WrongParentPrefix(format!(
            "parent path `{}` is outside `{store_prefix}` or malformed",
            String::from_utf8_lossy(bytes)
        ))
    })
}

#[allow(tigerstyle::assertion_density)] // Private validated data determines this canonical projection.
fn render_direct_input_field(requests: &[DynamicParentRequest], logical_store_prefix: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(INITIAL_RENDER_CAPACITY_BYTES);
    bytes.push(b'[');
    for (index, request) in requests.iter().enumerate() {
        if index != 0 {
            bytes.push(b',');
        }
        bytes.push(b'(');
        write_aterm_string(&mut bytes, request.drv_path.to_absolute_path_with_prefix(logical_store_prefix).as_bytes());
        bytes.push(b',');
        bytes.push(b'[');
        let outputs = projected_parent_outputs(request);
        for (output_index, output) in outputs.iter().enumerate() {
            if output_index != 0 {
                bytes.push(b',');
            }
            write_aterm_string(&mut bytes, output.as_bytes());
        }
        bytes.extend_from_slice(b"])");
    }
    bytes.push(b']');
    bytes
}

fn projected_parent_outputs(request: &DynamicParentRequest) -> BTreeSet<String> {
    let mut outputs = request.direct_outputs.clone();
    for node in &request.dynamic_outputs {
        if node.parent_node_index.is_none() {
            outputs.insert(node.output_name.clone());
        }
    }
    outputs
}

fn render_traditional_projection(fields: &[&[u8]; VERSIONED_FIELD_COUNT], input_field: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(INITIAL_RENDER_CAPACITY_BYTES);
    bytes.extend_from_slice(TRADITIONAL_PREFIX);
    for (index, field) in fields.iter().enumerate() {
        if index != 0 {
            bytes.push(b',');
        }
        if index == 1 {
            bytes.extend_from_slice(input_field);
        } else {
            bytes.extend_from_slice(field);
        }
    }
    bytes.push(b')');
    bytes
}

fn parse_legacy_projection(content: &[u8], store_prefix: &str) -> Result<Derivation, DynamicAdmissionError> {
    validate_store_prefix(store_prefix)?;
    let normalized = normalize_structural_prefix(content, store_prefix)?;
    Derivation::from_aterm_bytes(&normalized).map_err(|error| DynamicAdmissionError::Syntax {
        offset: 0,
        detail: format!("parsing ATerm projection: {error:?}"),
    })
}

fn normalize_structural_prefix(content: &[u8], store_prefix: &str) -> Result<Vec<u8>, DynamicAdmissionError> {
    if store_prefix == DEFAULT_STORE_PREFIX {
        return Ok(content.to_vec());
    }
    let default_marker = format!("{DEFAULT_STORE_PREFIX}/");
    if contains_bytes(content, default_marker.as_bytes()) {
        return Err(DynamicAdmissionError::MixedPrefix(format!(
            "candidate for `{store_prefix}` contains `{DEFAULT_STORE_PREFIX}` paths"
        )));
    }
    let selected_marker = format!("{store_prefix}/");
    Ok(replace_bytes(content, selected_marker.as_bytes(), default_marker.as_bytes()))
}

struct PrefixRewrite {
    default_marker: String,
    selected_marker: String,
}

fn restore_custom_prefix_fields(derivation: &mut Derivation, store_prefix: &str) -> Result<(), DynamicAdmissionError> {
    if store_prefix == DEFAULT_STORE_PREFIX {
        return Ok(());
    }
    let rewrite = PrefixRewrite {
        default_marker: format!("{DEFAULT_STORE_PREFIX}/"),
        selected_marker: format!("{store_prefix}/"),
    };
    derivation.builder = restore_prefixed_string(&derivation.builder, &rewrite)?;
    for argument in &mut derivation.arguments {
        *argument = restore_prefixed_string(argument, &rewrite)?;
    }
    for value in derivation.environment.values_mut() {
        *value = replace_bytes(value, rewrite.default_marker.as_bytes(), rewrite.selected_marker.as_bytes()).into();
    }
    Ok(())
}

fn restore_prefixed_string(value: &str, rewrite: &PrefixRewrite) -> Result<String, DynamicAdmissionError> {
    String::from_utf8(replace_bytes(
        value.as_bytes(),
        rewrite.default_marker.as_bytes(),
        rewrite.selected_marker.as_bytes(),
    ))
    .map_err(|_| DynamicAdmissionError::Semantic("prefix replacement produced invalid UTF-8".to_string()))
}

fn replace_bytes(input: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    if from.is_empty() {
        return input.to_vec();
    }
    let mut output = Vec::with_capacity(input.len());
    let mut cursor = 0;
    while cursor < input.len() {
        if input[cursor..].starts_with(from) {
            output.extend_from_slice(to);
            cursor = cursor.saturating_add(from.len());
        } else {
            output.push(input[cursor]);
            cursor = cursor.saturating_add(1);
        }
    }
    output
}

fn contains_bytes(input: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && input.windows(needle.len()).any(|window| window == needle)
}

fn derivation_name(derivation: &Derivation, limits: DynamicAdmissionLimits) -> Result<String, DynamicAdmissionError> {
    let bytes = derivation
        .environment
        .get("name")
        .ok_or_else(|| DynamicAdmissionError::Semantic("missing required `name` environment entry".to_string()))?;
    enforce_field("derivation name", bytes.len(), limits.field_bytes_max)?;
    let name = std::str::from_utf8(bytes)
        .map_err(|_| DynamicAdmissionError::Semantic("derivation name is not UTF-8".to_string()))?
        .to_string();
    nix_compat::store_path::validate_name(name.as_bytes())
        .map_err(|_| DynamicAdmissionError::Semantic(format!("invalid derivation name `{name}`")))?;
    Ok(name)
}

fn validate_store_prefix(prefix: &str) -> Result<(), DynamicAdmissionError> {
    if prefix.is_empty() || !prefix.starts_with('/') || prefix.ends_with('/') {
        return Err(DynamicAdmissionError::Semantic(format!(
            "logical store prefix `{prefix}` must be absolute and have no trailing slash"
        )));
    }
    Ok(())
}

fn validate_candidate_output(path: &StorePath<String>) -> Result<(), DynamicAdmissionError> {
    if !path.name().ends_with(".drv") {
        return Err(DynamicAdmissionError::Semantic(format!(
            "candidate output `{}` does not end with .drv",
            path.name()
        )));
    }
    Ok(())
}

fn validate_derivation_collections(
    derivation: &Derivation,
    limits: DynamicAdmissionLimits,
) -> Result<(), DynamicAdmissionError> {
    enforce_count("outputs", derivation.outputs.len(), limits.collection_entries_max)?;
    enforce_count("input derivations", derivation.input_derivations.len(), limits.parent_edges_max)?;
    enforce_count("input sources", derivation.input_sources.len(), limits.collection_entries_max)?;
    enforce_count("arguments", derivation.arguments.len(), limits.collection_entries_max)?;
    enforce_count("environment", derivation.environment.len(), limits.collection_entries_max)?;
    enforce_field("builder", derivation.builder.len(), limits.field_bytes_max)?;
    enforce_field("system", derivation.system.len(), limits.field_bytes_max)?;
    for argument in &derivation.arguments {
        enforce_field("argument", argument.len(), limits.field_bytes_max)?;
    }
    for (key, value) in &derivation.environment {
        enforce_field("environment key", key.len(), limits.field_bytes_max)?;
        enforce_field("environment value", value.len(), limits.field_bytes_max)?;
    }
    Ok(())
}

#[allow(tigerstyle::assertion_density)] // Validation returns stable errors for each rejected request.
fn validate_parent_requests(
    requests: &[DynamicParentRequest],
    form: CandidateForm,
    limits: DynamicAdmissionLimits,
) -> Result<(), DynamicAdmissionError> {
    enforce_count("parent edges", requests.len(), limits.parent_edges_max)?;
    for request in requests {
        let projected = projected_parent_outputs(request);
        if projected.is_empty() {
            return Err(DynamicAdmissionError::EmptyRequest(format!(
                "parent `{}` requests no direct or dynamic outputs",
                request.drv_path
            )));
        }
        enforce_count("direct outputs", request.direct_outputs.len(), limits.collection_entries_max)?;
        enforce_count("dynamic nodes", request.dynamic_outputs.len(), limits.dynamic_nodes_max)?;
        if form == CandidateForm::Traditional && !request.dynamic_outputs.is_empty() {
            return Err(DynamicAdmissionError::UnsupportedVersion(
                "traditional derivation contains dynamic input nodes".to_string(),
            ));
        }
        for node in &request.dynamic_outputs {
            if node.requested_outputs.is_empty() {
                return Err(DynamicAdmissionError::EmptyRequest(format!(
                    "dynamic output `{}` has no requested outputs",
                    node.output_name
                )));
            }
        }
    }
    Ok(())
}

fn validate_output_semantics(derivation: &Derivation, form: CandidateForm) -> Result<(), DynamicAdmissionError> {
    if form == CandidateForm::Versioned {
        for (name, output) in &derivation.outputs {
            if output.path.is_none() || output.ca_hash.is_some() {
                return Err(DynamicAdmissionError::UnsupportedOutput(format!(
                    "versioned output `{name}` is not input-addressed"
                )));
            }
        }
    }
    Ok(())
}

#[allow(tigerstyle::assertion_density)] // Completeness failures remain recoverable admission errors.
#[allow(tigerstyle::unbounded_collection_growth)] // BTree maps cannot reserve; parent_edges_max bounds both maps.
fn validate_parent_facts(
    validated: &ValidatedDynamicDerivation,
    facts: &[DynamicParentHashFact],
) -> Result<BTreeMap<StorePath<String>, [u8; BLAKE3_DIGEST_BYTES]>, DynamicAdmissionError> {
    let required: BTreeSet<String> = required_parent_paths(validated).into_iter().collect();
    let mut observed = BTreeMap::new();
    for fact in facts {
        validate_one_parent_fact(validated, fact, &required, &mut observed)?;
    }
    for path in required {
        if !observed.contains_key(&path) {
            return Err(DynamicAdmissionError::MissingParent(path));
        }
    }
    let mut mapped = BTreeMap::new();
    for request in &validated.parsed.parent_requests {
        let absolute = request.drv_path.to_absolute_path_with_prefix(&validated.logical_store_prefix);
        let digest = observed
            .get(&absolute)
            .copied()
            .ok_or_else(|| DynamicAdmissionError::MissingParent(absolute.clone()))?;
        mapped.insert(request.drv_path.clone(), digest);
    }
    Ok(mapped)
}

#[allow(tigerstyle::assertion_density)] // Supplied facts are untrusted and must fail without panics.
fn validate_one_parent_fact(
    validated: &ValidatedDynamicDerivation,
    fact: &DynamicParentHashFact,
    required: &BTreeSet<String>,
    observed: &mut BTreeMap<String, [u8; BLAKE3_DIGEST_BYTES]>,
) -> Result<(), DynamicAdmissionError> {
    if fact.domain != DynamicHashDomain::MantleBlake3 {
        return Err(DynamicAdmissionError::WrongHashDomain(fact.logical_drv_path.clone()));
    }
    StorePath::<String>::from_absolute_path_with_prefix(
        fact.logical_drv_path.as_bytes(),
        &validated.logical_store_prefix,
    )
    .map_err(|_| DynamicAdmissionError::WrongParentPrefix(fact.logical_drv_path.clone()))?;
    if !required.contains(&fact.logical_drv_path) {
        return Err(DynamicAdmissionError::UnexpectedParent(fact.logical_drv_path.clone()));
    }
    if let Some(previous) = observed.get(&fact.logical_drv_path) {
        if previous == &fact.digest {
            return Err(DynamicAdmissionError::DuplicateParent(fact.logical_drv_path.clone()));
        }
        return Err(DynamicAdmissionError::ConflictingParent(fact.logical_drv_path.clone()));
    }
    observed.insert(fact.logical_drv_path.clone(), fact.digest);
    Ok(())
}

fn compute_hash_derivation_modulo(
    validated: &mut ValidatedDynamicDerivation,
    facts: &BTreeMap<StorePath<String>, [u8; BLAKE3_DIGEST_BYTES]>,
) -> Result<[u8; BLAKE3_DIGEST_BYTES], DynamicAdmissionError> {
    debug_assert!(
        validated.parsed.parent_requests.iter().all(|request| facts.contains_key(&request.drv_path)),
        "validated parent requests must have complete facts"
    );
    match validated.parsed.form {
        CandidateForm::Traditional if validated.parsed.identity_projection == IdentityProjection::OutputsMasked => {
            hash_native_masked_outputs(&mut validated.parsed.derivation, facts, &validated.logical_store_prefix)
        }
        CandidateForm::Traditional => Ok(validated
            .parsed
            .derivation
            .hash_derivation_modulo_with_store_dir(|path| facts[&path.to_owned()], &validated.logical_store_prefix)),
        CandidateForm::Versioned => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(FRAME_TAG);
            hash_frame_bytes(&mut hasher, &validated.parsed.source_bytes)?;
            for (path, digest) in facts {
                hash_frame_bytes(
                    &mut hasher,
                    path.to_absolute_path_with_prefix(&validated.logical_store_prefix).as_bytes(),
                )?;
                hash_frame_bytes(&mut hasher, digest)?;
            }
            Ok(*hasher.finalize().as_bytes())
        }
    }
}

/// Hash the native unit as it existed before output path realization, without
/// copying its derivation or discarding the finished output paths/ATerm bytes.
fn hash_native_masked_outputs(
    derivation: &mut Derivation,
    facts: &BTreeMap<StorePath<String>, [u8; BLAKE3_DIGEST_BYTES]>,
    store_prefix: &str,
) -> Result<[u8; BLAKE3_DIGEST_BYTES], DynamicAdmissionError> {
    if derivation.outputs.keys().any(|name| !derivation.environment.contains_key(name)) {
        return Err(DynamicAdmissionError::Identity("native output is missing its environment binding".to_string()));
    }
    let mut original = Vec::with_capacity(derivation.outputs.len());
    for (name, output) in &mut derivation.outputs {
        let Some(value) = derivation.environment.get_mut(name) else {
            return Err(DynamicAdmissionError::Identity("native output binding vanished before hashing".to_string()));
        };
        original.push((output.path.take(), std::mem::take(value)));
    }
    let hash = derivation.hash_derivation_modulo_with_store_dir(|path| facts[&path.to_owned()], store_prefix);
    for ((name, output), (path, value)) in derivation.outputs.iter_mut().zip(original) {
        output.path = path;
        let Some(binding) = derivation.environment.get_mut(name) else {
            return Err(DynamicAdmissionError::Identity("native output binding vanished after hashing".to_string()));
        };
        *binding = value;
    }
    Ok(hash)
}

#[allow(tigerstyle::assertion_density)] // The private validated state already carries the required invariants.
fn compute_derivation_path(validated: &ValidatedDynamicDerivation) -> Result<StorePath<String>, DynamicAdmissionError> {
    if validated.parsed.form == CandidateForm::Traditional {
        return validated
            .parsed
            .derivation
            .calculate_derivation_path_with_store_dir(&validated.parsed.name, &validated.logical_store_prefix)
            .map_err(|error| DynamicAdmissionError::Identity(error.to_string()));
    }
    let references = validated
        .parsed
        .derivation
        .input_sources
        .iter()
        .chain(validated.parsed.derivation.input_derivations.keys())
        .map(|path| path.to_absolute_path_with_prefix(&validated.logical_store_prefix))
        .collect::<BTreeSet<_>>();
    let name = format!("{}.drv", validated.parsed.name);
    build_text_path_with_store_dir(
        &name,
        &validated.parsed.source_bytes,
        references.iter().map(String::as_str),
        &validated.logical_store_prefix,
    )
    .map_err(|error| DynamicAdmissionError::Identity(error.to_string()))
}

fn compute_full_identity(
    validated: &ValidatedDynamicDerivation,
    drv_path: &StorePath<String>,
    hdm: [u8; BLAKE3_DIGEST_BYTES],
) -> Result<[u8; BLAKE3_DIGEST_BYTES], DynamicAdmissionError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FULL_IDENTITY_TAG);
    hash_frame_bytes(&mut hasher, &validated.parsed.source_bytes)?;
    hash_frame_bytes(&mut hasher, drv_path.to_absolute_path_with_prefix(&validated.logical_store_prefix).as_bytes())?;
    hash_frame_bytes(&mut hasher, &hdm)?;
    Ok(*hasher.finalize().as_bytes())
}

fn usize_to_u64(value: usize, field: &'static str) -> Result<u64, DynamicAdmissionError> {
    u64::try_from(value).map_err(|_| DynamicAdmissionError::PlatformLimit(field))
}

fn usize_to_u32(value: usize, field: &'static str) -> Result<u32, DynamicAdmissionError> {
    u32::try_from(value).map_err(|_| DynamicAdmissionError::PlatformLimit(field))
}

fn hash_frame_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), DynamicAdmissionError> {
    let length_bytes = usize_to_u64(bytes.len(), "hash frame byte length")?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn enforce_count(field: &'static str, observed: usize, limit: u32) -> Result<(), DynamicAdmissionError> {
    let observed = usize_to_u32(observed, field)?;
    if observed > limit {
        return Err(DynamicAdmissionError::CollectionLimit { field, observed, limit });
    }
    Ok(())
}

fn enforce_field(field: &'static str, observed: usize, limit: u64) -> Result<(), DynamicAdmissionError> {
    let observed = usize_to_u64(observed, field)?;
    if observed > limit {
        return Err(DynamicAdmissionError::FieldLimit { field, observed, limit });
    }
    Ok(())
}

fn write_aterm_string(output: &mut Vec<u8>, value: &[u8]) {
    output.push(b'"');
    for byte in value {
        match byte {
            b'"' => output.extend_from_slice(b"\\\""),
            b'\\' => output.extend_from_slice(b"\\\\"),
            b'\n' => output.extend_from_slice(b"\\n"),
            b'\r' => output.extend_from_slice(b"\\r"),
            b'\t' => output.extend_from_slice(b"\\t"),
            other => output.push(*other),
        }
    }
    output.push(b'"');
}

struct InputNodeFrame {
    node_index: Option<u32>,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
    limits: DynamicAdmissionLimits,
    collections: u32,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8], limits: DynamicAdmissionLimits) -> Self {
        Self {
            bytes,
            position: 0,
            limits,
            collections: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), DynamicAdmissionError> {
        if self.peek() != Some(expected) {
            return self.syntax(format!("expected byte `{}`", char::from(expected)));
        }
        self.position = self.position.saturating_add(1);
        Ok(())
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), DynamicAdmissionError> {
        if !self.bytes[self.position..].starts_with(expected) {
            return self.syntax(format!("expected `{}`", String::from_utf8_lossy(expected)));
        }
        self.position = self.position.saturating_add(expected.len());
        Ok(())
    }

    fn expect_end(&self) -> Result<(), DynamicAdmissionError> {
        if self.position != self.bytes.len() {
            return self.syntax("trailing bytes");
        }
        Ok(())
    }

    fn syntax<T>(&self, detail: impl Into<String>) -> Result<T, DynamicAdmissionError> {
        let offset_bytes = usize_to_u64(self.position, "parser byte offset")?;
        Err(DynamicAdmissionError::Syntax {
            offset: offset_bytes,
            detail: detail.into(),
        })
    }

    fn parse_string(&mut self) -> Result<Vec<u8>, DynamicAdmissionError> {
        self.expect_byte(b'"')?;
        let remaining_bytes = self.bytes.len().saturating_sub(self.position);
        let mut output = Vec::with_capacity(remaining_bytes);
        let mut iterations = 0_usize;
        while iterations < remaining_bytes {
            iterations = iterations.saturating_add(1);
            let Some(byte) = self.peek() else {
                return self.syntax("unterminated string");
            };
            self.position = self.position.saturating_add(1);
            match byte {
                b'"' => return Ok(output),
                b'\\' => {
                    let Some(escaped) = self.peek() else {
                        return self.syntax("unterminated string escape");
                    };
                    self.position = self.position.saturating_add(1);
                    output.push(match escaped {
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b't' => b'\t',
                        other => other,
                    });
                }
                other => output.push(other),
            }
            enforce_field("ATerm string", output.len(), self.limits.field_bytes_max)?;
        }
        self.syntax("unterminated string")
    }

    fn take_value(&mut self) -> Result<&'a [u8], DynamicAdmissionError> {
        let start = self.position;
        let Some(first) = self.peek() else {
            return self.syntax("missing value");
        };
        if first == b'"' {
            self.parse_string()?;
            return Ok(&self.bytes[start..self.position]);
        }
        if first != b'[' && first != b'(' {
            return self.syntax("ATerm value must be a string, list, or tuple");
        }
        self.scan_balanced_value()?;
        Ok(&self.bytes[start..self.position])
    }

    fn scan_balanced_value(&mut self) -> Result<(), DynamicAdmissionError> {
        let stack_entries_max = usize::try_from(self.limits.dynamic_depth_max)
            .map_err(|_| DynamicAdmissionError::PlatformLimit("parser stack capacity"))?;
        let mut stack: Vec<u8> = Vec::with_capacity(stack_entries_max);
        let iterations_max = self.bytes.len().saturating_sub(self.position);
        let mut iterations = 0_usize;
        while iterations < iterations_max {
            iterations = iterations.saturating_add(1);
            let Some(byte) = self.peek() else {
                return self.syntax("unterminated ATerm value");
            };
            match byte {
                b'"' => {
                    self.parse_string()?;
                    continue;
                }
                b'[' => self.open_collection(&mut stack, b']')?,
                b'(' => self.open_collection(&mut stack, b')')?,
                b']' | b')' => {
                    let offset_bytes = usize_to_u64(self.position, "parser byte offset")?;
                    let expected = stack.pop().ok_or_else(|| DynamicAdmissionError::Syntax {
                        offset: offset_bytes,
                        detail: "unmatched closing delimiter".to_string(),
                    })?;
                    if byte != expected {
                        return self.syntax("mismatched closing delimiter");
                    }
                    self.position = self.position.saturating_add(1);
                    if stack.is_empty() {
                        return Ok(());
                    }
                }
                _ => self.position = self.position.saturating_add(1),
            }
        }
        self.syntax("unterminated ATerm value")
    }

    fn open_collection(&mut self, stack: &mut Vec<u8>, closing: u8) -> Result<(), DynamicAdmissionError> {
        self.collections = self.collections.saturating_add(1);
        if self.collections > self.limits.parser_collections_max {
            return Err(DynamicAdmissionError::CollectionLimit {
                field: "ATerm collections",
                observed: self.collections,
                limit: self.limits.parser_collections_max,
            });
        }
        stack.push(closing);
        let depth = usize_to_u32(stack.len(), "parser collection depth")?;
        if depth > self.limits.dynamic_depth_max {
            return Err(DynamicAdmissionError::DepthLimit {
                observed: depth,
                limit: self.limits.dynamic_depth_max,
            });
        }
        self.position = self.position.saturating_add(1);
        Ok(())
    }

    fn parse_string_set(&mut self) -> Result<BTreeSet<String>, DynamicAdmissionError> {
        self.expect_byte(b'[')?;
        let mut outputs = BTreeSet::new();
        while self.peek() != Some(b']') {
            let value = self.parse_string()?;
            let value = String::from_utf8(value)
                .map_err(|_| DynamicAdmissionError::Semantic("output request is not UTF-8".to_string()))?;
            nix_compat::derivation::validate_output_name(&value)
                .map_err(|_| DynamicAdmissionError::Semantic(format!("invalid output request `{value}`")))?;
            if !outputs.insert(value.clone()) {
                return Err(DynamicAdmissionError::Semantic(format!("duplicate output request `{value}`")));
            }
            enforce_count("requested outputs", outputs.len(), self.limits.collection_entries_max)?;
            if self.peek() == Some(b',') {
                self.expect_byte(b',')?;
            } else if self.peek() != Some(b']') {
                return self.syntax("expected comma or closing output list");
            }
        }
        self.expect_byte(b']')?;
        Ok(outputs)
    }

    fn parse_input_node(&mut self) -> Result<(BTreeSet<String>, Vec<DynamicOutputRequest>), DynamicAdmissionError> {
        if self.peek() == Some(b'[') {
            return Ok((self.parse_string_set()?, Vec::new()));
        }
        self.expect_byte(b'(')?;
        let root_outputs = self.parse_string_set()?;
        self.expect_byte(b',')?;
        self.expect_byte(b'[')?;
        let mut nodes = Vec::with_capacity(INITIAL_COLLECTION_CAPACITY);
        let mut frames = Vec::with_capacity(INITIAL_COLLECTION_CAPACITY);
        frames.push(InputNodeFrame { node_index: None });
        self.parse_dynamic_frames(&mut frames, &mut nodes)?;
        Ok((root_outputs, nodes))
    }

    fn parse_dynamic_frames(
        &mut self,
        frames: &mut Vec<InputNodeFrame>,
        nodes: &mut Vec<DynamicOutputRequest>,
    ) -> Result<(), DynamicAdmissionError> {
        while !frames.is_empty() {
            if self.peek() == Some(b']') {
                self.close_dynamic_frame(frames)?;
                continue;
            }
            let parent = frames.last().and_then(|frame| frame.node_index);
            self.expect_byte(b'(')?;
            let name = String::from_utf8(self.parse_string()?)
                .map_err(|_| DynamicAdmissionError::Semantic("dynamic output name is not UTF-8".to_string()))?;
            nix_compat::derivation::validate_output_name(&name)
                .map_err(|_| DynamicAdmissionError::Semantic(format!("invalid dynamic output name `{name}`")))?;
            self.expect_byte(b',')?;
            if self.peek() == Some(b'[') {
                let outputs = self.parse_string_set()?;
                self.expect_byte(b')')?;
                nodes.push(DynamicOutputRequest {
                    parent_node_index: parent,
                    output_name: name,
                    requested_outputs: outputs,
                });
                self.after_dynamic_child()?;
                enforce_count("dynamic nodes", nodes.len(), self.limits.dynamic_nodes_max)?;
                continue;
            }
            self.expect_byte(b'(')?;
            let outputs = self.parse_string_set()?;
            self.expect_byte(b',')?;
            self.expect_byte(b'[')?;
            let node_index = usize_to_u32(nodes.len(), "dynamic node index")?;
            nodes.push(DynamicOutputRequest {
                parent_node_index: parent,
                output_name: name,
                requested_outputs: outputs,
            });
            enforce_count("dynamic nodes", nodes.len(), self.limits.dynamic_nodes_max)?;
            frames.push(InputNodeFrame {
                node_index: Some(node_index),
            });
            let depth = usize_to_u32(frames.len(), "dynamic input depth")?;
            if depth > self.limits.dynamic_depth_max {
                return Err(DynamicAdmissionError::DepthLimit {
                    observed: depth,
                    limit: self.limits.dynamic_depth_max,
                });
            }
        }
        Ok(())
    }

    fn close_dynamic_frame(&mut self, frames: &mut Vec<InputNodeFrame>) -> Result<(), DynamicAdmissionError> {
        self.expect_byte(b']')?;
        self.expect_byte(b')')?;
        let closed = frames
            .pop()
            .ok_or_else(|| DynamicAdmissionError::Semantic("dynamic frame stack is empty".to_string()))?;
        if closed.node_index.is_some() {
            self.expect_byte(b')')?;
            self.after_dynamic_child()?;
        }
        Ok(())
    }

    fn after_dynamic_child(&mut self) -> Result<(), DynamicAdmissionError> {
        if self.peek() == Some(b',') {
            self.expect_byte(b',')?;
        } else if self.peek() != Some(b']') {
            return self.syntax("expected comma or closing dynamic-output list");
        }
        Ok(())
    }
}

#[cfg(test)]
// r[verify dynamic_derivation_admission.staged_core]
// r[verify dynamic_derivation_admission.versioned_forms]
// r[verify dynamic_derivation_admission.complete_parent_identity]
// r[verify dynamic_derivation_admission.compatibility]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use nix_compat::derivation::Output;
    use snix_castore::B3Digest;

    use super::*;

    const NIX_STORE: &str = "/nix/store";
    const CUSTOM_STORE: &str = "/mantle/store";
    const PARENT_DIGEST: [u8; BLAKE3_DIGEST_BYTES] = [0x42; BLAKE3_DIGEST_BYTES];

    fn simple_drv() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "hello".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        Derivation {
            arguments: vec!["-c".into(), "echo hello > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    fn fake_store_path(name: &str) -> StorePath<String> {
        let mut digest = [0_u8; 20];
        for (index, byte) in name.bytes().enumerate() {
            digest[index % digest.len()] ^= byte;
        }
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    fn candidate_path() -> StorePath<String> {
        fake_store_path("candidate.drv")
    }

    fn traditional_bytes_with_parent(store_prefix: &str) -> (Vec<u8>, String) {
        let parent = fake_store_path("parent.drv");
        let parent_absolute = parent.to_absolute_path_with_prefix(store_prefix);
        let mut derivation = simple_drv();
        derivation.input_derivations.insert(parent, BTreeSet::from(["out".to_string()]));
        let output_hdm = derivation.hash_derivation_modulo_with_store_dir(|_| PARENT_DIGEST, store_prefix);
        derivation.calculate_output_paths("hello", &output_hdm).unwrap();
        if store_prefix != NIX_STORE {
            let output_path =
                derivation.outputs["out"].path.as_ref().unwrap().to_absolute_path_with_prefix(store_prefix);
            derivation.environment.insert("out".to_string(), output_path.into());
        }
        (derivation.to_aterm_bytes_with_store_dir(store_prefix), parent_absolute)
    }

    fn resolve_traditional(
        bytes: &[u8],
        store_prefix: &str,
        facts: &[DynamicParentHashFact],
    ) -> Result<IdentityResolvedDynamicDerivation, DynamicAdmissionError> {
        let parsed =
            parse_dynamic_candidate(bytes, &candidate_path(), store_prefix, DynamicAdmissionLimits::default())?
                .unwrap();
        let validated = validate_dynamic_candidate(parsed, store_prefix, DynamicAdmissionLimits::default())?;
        resolve_dynamic_identity(validated, facts)
    }

    #[test]
    fn native_output_mask_keeps_parent_hash_and_finished_bytes_for_both_store_prefixes() {
        for store_prefix in [NIX_STORE, CUSTOM_STORE] {
            for mode in ["input-addressed", "content-addressed", "fixed-output"] {
                let mut derivation = simple_drv();
                let parent = fake_store_path("parent.drv");
                derivation.input_derivations.insert(parent.clone(), BTreeSet::from(["out".to_string()]));
                if mode == "fixed-output" {
                    derivation.outputs.get_mut("out").unwrap().ca_hash = Some(
                        nix_compat::nixhash::CAHash::Flat(
                            nix_compat::nixhash::NixHash::Sha256([0x42; BLAKE3_DIGEST_BYTES]),
                        ),
                    );
                }
                let original_hash = derivation.hash_derivation_modulo_with_store_dir(|_| PARENT_DIGEST, store_prefix);
                derivation.calculate_output_paths_with_store_dir("hello", &original_hash, store_prefix).unwrap();
                let provisional = derivation.outputs["out"].path.as_ref().unwrap().to_absolute_path_with_prefix(store_prefix);
                derivation.environment.insert("out".to_string(), provisional.into());
                if mode == "content-addressed" {
                    derivation.outputs.get_mut("out").unwrap().path = None;
                }
                let candidate = derivation.calculate_derivation_path_with_store_dir("hello", store_prefix).unwrap();
                let finished = derivation.to_aterm_bytes_with_store_dir(store_prefix);
                let validated = admit_native_plan_derivation(
                    derivation,
                    candidate.clone(),
                    store_prefix,
                    DynamicAdmissionLimits::default(),
                )
                .unwrap();
                let fact = DynamicParentHashFact::native(parent.to_absolute_path_with_prefix(store_prefix), PARENT_DIGEST);
                let ready = resolve_dynamic_identity(validated, &[fact]).unwrap();
                assert_eq!(ready.hash_derivation_modulo, original_hash, "mode={mode} store={store_prefix}");
                assert_eq!(ready.drv_path, candidate, "mode={mode} store={store_prefix}");
                assert_eq!(ready.validated.parsed.source_bytes, finished, "mode={mode} store={store_prefix}");
                assert_eq!(ready.validated.parsed.derivation.to_aterm_bytes_with_store_dir(store_prefix), finished);
                assert_eq!(ready.validated.parsed.derivation.outputs["out"].path.is_none(), mode == "content-addressed");
            }
        }
    }

    fn versioned_bytes(store_prefix: &str) -> Vec<u8> {
        let parent = fake_store_path("parent.drv").to_absolute_path_with_prefix(store_prefix);
        let output = fake_store_path("hello").to_absolute_path_with_prefix(store_prefix);
        format!(
            "DrvWithVersion(\"xp-dyn-drv\",[(\"out\",\"{output}\",\"\",\"\")],[(\"{parent}\",([\"out\"],[(\"generated\",([\"dev\"],[(\"nested\",[\"out\"])]))]))],[],\"x86_64-linux\",\"/bin/sh\",[],[(\"name\",\"hello\"),(\"out\",\"{output}\")])"
        )
        .into_bytes()
    }

    fn versioned_resolved() -> IdentityResolvedDynamicDerivation {
        let bytes = versioned_bytes(NIX_STORE);
        let parsed = parse_dynamic_candidate(&bytes, &candidate_path(), NIX_STORE, DynamicAdmissionLimits::default())
            .unwrap()
            .unwrap();
        let validated = validate_dynamic_candidate(parsed, NIX_STORE, DynamicAdmissionLimits::default()).unwrap();
        let parent = required_parent_paths(&validated).pop().unwrap();
        resolve_dynamic_identity(validated, &[DynamicParentHashFact::native(parent, PARENT_DIGEST)]).unwrap()
    }

    #[test]
    fn drv_output_detection_is_bounded_and_shape_aware() {
        let path = fake_store_path("hello.drv");
        let file = Node::File {
            digest: B3Digest::from(&[0_u8; BLAKE3_DIGEST_BYTES]),
            size: MAX_DRV_SIZE_BYTES,
            executable: false,
        };
        let oversized = Node::File {
            digest: B3Digest::from(&[0_u8; BLAKE3_DIGEST_BYTES]),
            size: MAX_DRV_SIZE_BYTES.saturating_add(1),
            executable: false,
        };
        let directory = Node::Directory {
            digest: B3Digest::from(&[0_u8; BLAKE3_DIGEST_BYTES]),
            size: 1,
        };
        assert!(is_drv_output(&path, &file));
        assert!(!is_drv_output(&path, &oversized));
        assert!(!is_drv_output(&path, &directory));
        assert!(!is_drv_output(&fake_store_path("hello"), &file));
    }

    #[test]
    fn traditional_candidate_reaches_registry_ready_state() {
        let (bytes, _) = traditional_bytes_with_parent(NIX_STORE);
        let parent = fake_store_path("parent.drv").to_absolute_path();
        let resolved =
            resolve_traditional(&bytes, NIX_STORE, &[DynamicParentHashFact::native(parent, PARENT_DIGEST)]).unwrap();
        let ready = plan_dynamic_registration(resolved, None).unwrap();
        assert_eq!(ready.decision(), DynamicRegistrationDecision::Insert);
        assert_eq!(ready.resolved.validated.parsed.parent_requests.len(), 1);
        assert!(ready.drv_path().name().ends_with("hello.drv"));
    }

    #[test]
    fn missing_parent_fails_before_registry_ready_state() {
        let (bytes, parent) = traditional_bytes_with_parent(NIX_STORE);
        let error = resolve_traditional(&bytes, NIX_STORE, &[]).unwrap_err();
        assert_eq!(error, DynamicAdmissionError::MissingParent(parent));
        assert_eq!(error.code(), "dynamic-admission-missing-parent");
    }

    #[test]
    fn duplicate_conflicting_and_unexpected_parent_facts_fail_closed() {
        let (bytes, parent) = traditional_bytes_with_parent(NIX_STORE);
        let duplicate = vec![
            DynamicParentHashFact::native(parent.clone(), PARENT_DIGEST),
            DynamicParentHashFact::native(parent.clone(), PARENT_DIGEST),
        ];
        assert!(matches!(
            resolve_traditional(&bytes, NIX_STORE, &duplicate),
            Err(DynamicAdmissionError::DuplicateParent(_))
        ));
        let conflicting = vec![
            DynamicParentHashFact::native(parent.clone(), PARENT_DIGEST),
            DynamicParentHashFact::native(parent.clone(), [0x24; BLAKE3_DIGEST_BYTES]),
        ];
        assert!(matches!(
            resolve_traditional(&bytes, NIX_STORE, &conflicting),
            Err(DynamicAdmissionError::ConflictingParent(_))
        ));
        let unexpected = DynamicParentHashFact::native(fake_store_path("extra.drv").to_absolute_path(), PARENT_DIGEST);
        assert!(matches!(
            resolve_traditional(&bytes, NIX_STORE, &[unexpected]),
            Err(DynamicAdmissionError::UnexpectedParent(_))
        ));
    }

    #[test]
    fn wrong_prefix_and_hash_domain_fail_before_identity() {
        let (bytes, parent) = traditional_bytes_with_parent(NIX_STORE);
        let wrong_prefix = DynamicParentHashFact::native(parent.replacen(NIX_STORE, CUSTOM_STORE, 1), PARENT_DIGEST);
        assert!(matches!(
            resolve_traditional(&bytes, NIX_STORE, &[wrong_prefix]),
            Err(DynamicAdmissionError::WrongParentPrefix(_))
        ));
        let wrong_domain = DynamicParentHashFact {
            logical_drv_path: parent,
            digest: PARENT_DIGEST,
            domain: DynamicHashDomain::NixSha256,
        };
        assert!(matches!(
            resolve_traditional(&bytes, NIX_STORE, &[wrong_domain]),
            Err(DynamicAdmissionError::WrongHashDomain(_))
        ));
    }

    #[test]
    fn traditional_identity_matches_covered_native_algorithm() {
        let (bytes, parent) = traditional_bytes_with_parent(NIX_STORE);
        let resolved =
            resolve_traditional(&bytes, NIX_STORE, &[DynamicParentHashFact::native(parent, PARENT_DIGEST)]).unwrap();
        let derivation = &resolved.validated.parsed.derivation;
        let expected_hdm = derivation.hash_derivation_modulo(|_| PARENT_DIGEST);
        let expected_path = derivation.calculate_derivation_path("hello").unwrap();
        assert_eq!(resolved.hash_derivation_modulo, expected_hdm);
        assert_eq!(resolved.drv_path, expected_path);
    }

    #[test]
    fn custom_prefix_changes_identity_without_mixing_prefixes() {
        let (bytes, parent) = traditional_bytes_with_parent(CUSTOM_STORE);
        let resolved =
            resolve_traditional(&bytes, CUSTOM_STORE, &[DynamicParentHashFact::native(parent, PARENT_DIGEST)]).unwrap();
        let logical = resolved.drv_path.to_absolute_path_with_prefix(CUSTOM_STORE);
        assert!(logical.starts_with(CUSTOM_STORE));
        assert!(!logical.starts_with(NIX_STORE));
        assert_ne!(resolved.hash_derivation_modulo, [0_u8; BLAKE3_DIGEST_BYTES]);
    }

    #[test]
    fn versioned_form_accepts_one_explicit_custom_prefix() {
        let bytes = versioned_bytes(CUSTOM_STORE);
        let parsed =
            parse_dynamic_candidate(&bytes, &candidate_path(), CUSTOM_STORE, DynamicAdmissionLimits::default())
                .unwrap()
                .unwrap();
        let validated = validate_dynamic_candidate(parsed, CUSTOM_STORE, DynamicAdmissionLimits::default()).unwrap();
        let parent = required_parent_paths(&validated).pop().unwrap();
        let resolved =
            resolve_dynamic_identity(validated, &[DynamicParentHashFact::native(parent, PARENT_DIGEST)]).unwrap();
        let logical = resolved.drv_path.to_absolute_path_with_prefix(CUSTOM_STORE);

        assert!(logical.starts_with(CUSTOM_STORE));
        assert!(!logical.starts_with(NIX_STORE));
    }

    #[test]
    fn mixed_prefix_candidate_is_rejected() {
        let (mut bytes, _) = traditional_bytes_with_parent(CUSTOM_STORE);
        bytes.extend_from_slice(format!("{NIX_STORE}/foreign").as_bytes());
        let error = parse_dynamic_candidate(&bytes, &candidate_path(), CUSTOM_STORE, DynamicAdmissionLimits::default())
            .unwrap_err();
        assert!(matches!(error, DynamicAdmissionError::MixedPrefix(_)));
    }

    #[test]
    fn supported_versioned_tree_is_flattened_in_preorder() {
        let resolved = versioned_resolved();
        let request = &resolved.validated.parsed.parent_requests[0];
        assert_eq!(request.direct_outputs, BTreeSet::from(["out".to_string()]));
        assert_eq!(request.dynamic_outputs.len(), 2);
        assert_eq!(request.dynamic_outputs[0].output_name, "generated");
        assert_eq!(request.dynamic_outputs[0].parent_node_index, None);
        assert_eq!(request.dynamic_outputs[1].output_name, "nested");
        assert_eq!(request.dynamic_outputs[1].parent_node_index, Some(0));
    }

    #[test]
    fn unknown_version_and_truncated_forms_fail_closed() {
        let unknown = b"DrvWithVersion(\"future\",[],[],[],\"x\",\"b\",[],[])";
        assert!(matches!(
            parse_dynamic_candidate(unknown, &candidate_path(), NIX_STORE, DynamicAdmissionLimits::default()),
            Err(DynamicAdmissionError::UnsupportedVersion(_))
        ));
        let truncated = b"DrvWithVersion(\"xp-dyn-drv\",[]";
        assert!(matches!(
            parse_dynamic_candidate(truncated, &candidate_path(), NIX_STORE, DynamicAdmissionLimits::default()),
            Err(DynamicAdmissionError::Syntax { .. })
        ));
    }

    #[test]
    fn byte_and_policy_limits_fail_closed() {
        let small = DynamicAdmissionLimits {
            bytes_max: 1,
            ..DynamicAdmissionLimits::default()
        };
        let error = parse_dynamic_candidate(b"Derive(", &candidate_path(), NIX_STORE, small).unwrap_err();
        assert!(matches!(error, DynamicAdmissionError::SizeLimit { .. }));

        let invalid = DynamicAdmissionLimits {
            bytes_max: MAX_DRV_SIZE_BYTES.saturating_add(1),
            ..DynamicAdmissionLimits::default()
        };
        assert!(matches!(
            parse_dynamic_candidate(b"", &candidate_path(), NIX_STORE, invalid),
            Err(DynamicAdmissionError::InvalidPolicy("bytes_max"))
        ));
    }

    #[test]
    fn over_depth_versioned_tree_fails_without_recursive_traversal() {
        let mut node = "[\"out\"]".to_string();
        for level in 0..=MAX_DYNAMIC_DEPTH {
            node = format!("([\"out\"],[(\"level-{level}\",{node})])");
        }
        let parent = fake_store_path("parent.drv").to_absolute_path();
        let bytes = format!(
            "DrvWithVersion(\"xp-dyn-drv\",[(\"out\",\"{}\",\"\",\"\")],[(\"{parent}\",{node})],[],\"x\",\"b\",[],[(\"name\",\"hello\")])",
            fake_store_path("hello").to_absolute_path()
        );
        assert!(matches!(
            parse_dynamic_candidate(bytes.as_bytes(), &candidate_path(), NIX_STORE, DynamicAdmissionLimits::default()),
            Err(DynamicAdmissionError::DepthLimit { .. })
        ));
    }

    #[test]
    fn versioned_empty_request_and_unsupported_output_fail_closed() {
        let parent = fake_store_path("parent.drv").to_absolute_path();
        let output = fake_store_path("hello").to_absolute_path();
        let empty = format!(
            "DrvWithVersion(\"xp-dyn-drv\",[(\"out\",\"{output}\",\"\",\"\")],[(\"{parent}\",([],[]))],[],\"x\",\"b\",[],[(\"name\",\"hello\")])"
        );
        assert!(matches!(
            parse_dynamic_candidate(empty.as_bytes(), &candidate_path(), NIX_STORE, DynamicAdmissionLimits::default(),),
            Err(DynamicAdmissionError::EmptyRequest(_))
        ));
        let mut fixed = simple_drv();
        fixed.outputs.get_mut("out").unwrap().ca_hash =
            Some(nix_compat::nixhash::CAHash::Flat(nix_compat::nixhash::NixHash::Sha256([0x42; BLAKE3_DIGEST_BYTES])));
        let fixed_hdm = fixed.hash_derivation_modulo(|_| panic!("no parents"));
        fixed.calculate_output_paths("hello", &fixed_hdm).unwrap();
        let traditional = String::from_utf8(fixed.to_aterm_bytes()).unwrap();
        let unsupported = traditional.replacen("Derive(", "DrvWithVersion(\"xp-dyn-drv\",", 1);
        let parsed = parse_dynamic_candidate(
            unsupported.as_bytes(),
            &candidate_path(),
            NIX_STORE,
            DynamicAdmissionLimits::default(),
        )
        .unwrap()
        .unwrap();
        assert!(matches!(
            validate_dynamic_candidate(parsed, NIX_STORE, DynamicAdmissionLimits::default()),
            Err(DynamicAdmissionError::UnsupportedOutput(_))
        ));
    }

    #[test]
    fn exact_duplicate_is_idempotent_and_collision_fails() {
        let resolved = versioned_resolved();
        let path = resolved.drv_path.to_absolute_path();
        let identity = resolved.full_identity;
        let existing = ExistingDynamicRegistration {
            logical_drv_path: path.clone(),
            full_identity: Some(identity),
        };
        let ready = plan_dynamic_registration(resolved.clone(), Some(&existing)).unwrap();
        assert_eq!(ready.decision(), DynamicRegistrationDecision::AlreadyPresent);
        let collision = ExistingDynamicRegistration {
            logical_drv_path: path,
            full_identity: Some([0x24; BLAKE3_DIGEST_BYTES]),
        };
        assert!(matches!(
            plan_dynamic_registration(resolved, Some(&collision)),
            Err(DynamicAdmissionError::PathCollision(_))
        ));
    }

    #[test]
    fn batch_deduplicates_exact_identity_and_rejects_collision() {
        let first = plan_dynamic_registration(versioned_resolved(), None).unwrap();
        let mut second = first.clone();
        let selected = validate_registry_ready_batch(&[first.clone(), second.clone()]).unwrap();
        assert_eq!(selected, BTreeSet::from([0]));
        assert_eq!(selected.len(), 1);

        second.resolved.full_identity = [0x24; BLAKE3_DIGEST_BYTES];
        assert!(matches!(
            validate_registry_ready_batch(&[first, second]),
            Err(DynamicAdmissionError::PathCollision(_))
        ));
    }

    #[test]
    fn malformed_name_and_non_candidate_are_distinct() {
        let mut derivation = simple_drv();
        let hdm = derivation.hash_derivation_modulo(|_| panic!("no parents"));
        derivation.calculate_output_paths("hello", &hdm).unwrap();
        derivation.environment.remove("name");
        let bytes = derivation.to_aterm_bytes();
        let error = parse_dynamic_candidate(&bytes, &candidate_path(), NIX_STORE, DynamicAdmissionLimits::default())
            .unwrap_err();
        assert!(matches!(error, DynamicAdmissionError::Semantic(_)));
        assert!(
            parse_dynamic_candidate(
                b"ordinary output",
                &candidate_path(),
                NIX_STORE,
                DynamicAdmissionLimits::default(),
            )
            .unwrap()
            .is_none()
        );
    }
}
