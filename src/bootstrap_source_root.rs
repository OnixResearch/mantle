use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const SOURCE_ROOT_MANIFEST_VERSION: u32 = 1;
pub(crate) const DIGEST_ALGORITHM_BLAKE3: &str = "blake3";
pub(crate) const LEGACY_MUSL_CC_URL: &str = "https://musl.cc/x86_64-linux-musl-native.tgz";
pub(crate) const LEGACY_MUSL_CC_HASH: &str = "sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY=";
pub(crate) const PROVIDER_NAME: &str = "musl-seed-toolchain";
pub(crate) const PROVIDER_TARGET: &str = "x86_64-linux-musl";
pub(crate) const PROVIDER_DYNAMIC_LINKER: &str = "ld-musl-x86_64.so.1";
pub(crate) const PROVIDER_METADATA_ROLE: &str = "share/crunch-bootstrap/provider.json";

const MIN_REQUIRED_ITEM_COUNT: usize = 1;
const MAX_MANIFEST_ITEM_COUNT: usize = 4_096;
const DIGEST_HEX_BYTES: usize = 32;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LEN: usize = DIGEST_HEX_BYTES.saturating_mul(HEX_CHARS_PER_BYTE);
const BOOTSTRAP_PROVIDER_MODE_COUNT_MAX: usize = 3;

pub(crate) const REQUIRED_PROVIDER_TOOL_ROLES: &[&str] = &[
    "x86_64-linux-musl-gcc",
    "x86_64-linux-musl-g++",
    "x86_64-linux-musl-c++",
    "x86_64-linux-musl-cpp",
    "x86_64-linux-musl-gcc-ar",
    "x86_64-linux-musl-gcc-nm",
    "x86_64-linux-musl-gcc-ranlib",
    "x86_64-linux-musl-ar",
    "x86_64-linux-musl-as",
    "x86_64-linux-musl-ld",
    "x86_64-linux-musl-nm",
    "x86_64-linux-musl-objcopy",
    "x86_64-linux-musl-objdump",
    "x86_64-linux-musl-ranlib",
    "x86_64-linux-musl-readelf",
    "x86_64-linux-musl-size",
    "x86_64-linux-musl-strings",
    "x86_64-linux-musl-strip",
    "x86_64-linux-musl-include",
    "x86_64-linux-musl-libgcc_s.so.1",
    "x86_64-linux-musl-libc.so",
    "x86_64-linux-musl-cxx-runtime",
    PROVIDER_METADATA_ROLE,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceRootManifest {
    pub version: Option<u32>,
    #[serde(default = "Vec::new")]
    pub artifacts: Vec<SourceArtifact>,
    #[serde(default = "Vec::new")]
    pub patches: Vec<SourcePatch>,
    #[serde(default = "Vec::new")]
    pub network_trust_roots: Vec<NetworkTrustRoot>,
    #[serde(default = "Vec::new")]
    pub trust_notes: Vec<TrustNote>,
    #[serde(default = "Vec::new")]
    pub expected_outputs: Vec<ExpectedProviderOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DigestSpec {
    pub algorithm: String,
    pub value: String,
    #[serde(default = "no_optional_value", skip_serializing_if = "Option::is_none")]
    pub non_blake3_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourceArtifact {
    pub name: String,
    pub source: String,
    pub digest: DigestSpec,
    pub extraction: ExtractionRule,
    pub provenance: String,
    #[serde(default = "Vec::new")]
    pub patches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ExtractionRule {
    pub kind: String,
    #[serde(default = "no_optional_value", skip_serializing_if = "Option::is_none")]
    pub strip_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SourcePatch {
    pub name: String,
    pub digest: DigestSpec,
    pub provenance: String,
    pub apply_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NetworkTrustRoot {
    pub authority: String,
    pub digest: DigestSpec,
    pub provenance: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TrustNote {
    pub name: String,
    pub digest: DigestSpec,
    pub provenance: String,
    pub scope: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ExpectedProviderOutput {
    pub name: String,
    pub kind: String,
    pub digest: DigestSpec,
    pub provenance: String,
    pub contract_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProviderDependencyTrace {
    #[serde(default = "Vec::new")]
    pub urls: Vec<String>,
    #[serde(default = "Vec::new")]
    pub hashes: Vec<String>,
    #[serde(default = "Vec::new")]
    pub emitted_output_roles: Vec<String>,
    #[serde(default = "Vec::new")]
    pub provider_metadata: Vec<String>,
}

fn no_optional_value<T>() -> Option<T> {
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BootstrapProviderMode {
    NixPackages,
    LegacyFetch,
    SourceRoot,
    StagexLineage,
}

impl BootstrapProviderMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NixPackages => "nix-packages",
            Self::LegacyFetch => "legacy-fetch",
            Self::SourceRoot => "source-root",
            Self::StagexLineage => "stagex-lineage",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "nix-packages" => Some(Self::NixPackages),
            "legacy-fetch" => Some(Self::LegacyFetch),
            "source-root" => Some(Self::SourceRoot),
            "stagex-lineage" => Some(Self::StagexLineage),
            _ => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn satisfies_stagex_requirement(self) -> bool {
        matches!(self, Self::StagexLineage)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestValidation {
    pub provider_name: &'static str,
    pub provider_target: &'static str,
    pub provider_dynamic_linker: &'static str,
    pub expected_output_roles: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestDiagnostic {
    pub path: String,
    pub message: String,
}

impl ManifestDiagnostic {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        let path = path.into();
        let message = message.into();
        assert!(!path.trim().is_empty(), "diagnostic path must not be empty");
        assert!(!message.trim().is_empty(), "diagnostic message must not be empty");
        Self { path, message }
    }
}

impl fmt::Display for ManifestDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

pub(crate) fn select_bootstrap_provider(
    fetch: bool,
    source_root: Option<&Path>,
    stagex_lineage: Option<&Path>,
) -> Result<BootstrapProviderMode, ManifestDiagnostic> {
    let selected_mode_count = [fetch, source_root.is_some(), stagex_lineage.is_some()]
        .into_iter()
        .filter(|is_selected| *is_selected)
        .count();
    assert!(BOOTSTRAP_PROVIDER_MODE_COUNT_MAX > 0, "provider mode bound must be positive");
    assert!(
        selected_mode_count <= BOOTSTRAP_PROVIDER_MODE_COUNT_MAX,
        "selected provider mode count must stay bounded"
    );
    if selected_mode_count > 1 {
        return Err(ManifestDiagnostic::new(
            "provider-selection",
            "ambiguous provider selection: only one of --fetch, --source-root, --stagex-lineage may be specified",
        ));
    }
    if stagex_lineage.is_some() {
        Ok(BootstrapProviderMode::StagexLineage)
    } else if source_root.is_some() {
        Ok(BootstrapProviderMode::SourceRoot)
    } else if fetch {
        Ok(BootstrapProviderMode::LegacyFetch)
    } else {
        Ok(BootstrapProviderMode::NixPackages)
    }
}

pub(crate) fn parse_source_root_manifest_bytes(bytes: &[u8]) -> Result<SourceRootManifest, String> {
    assert!(!bytes.is_empty(), "manifest bytes must not be empty");
    serde_json::from_slice(bytes).map_err(|err| format!("parsing source-root manifest JSON: {err}"))
}

pub(crate) fn source_root_manifest_digest(bytes: &[u8]) -> String {
    assert!(!bytes.is_empty(), "manifest bytes must not be empty");
    blake3::hash(bytes).to_hex().to_string()
}

pub(crate) fn format_diagnostics(errors: &[ManifestDiagnostic]) -> String {
    assert!(!errors.is_empty(), "formatting diagnostics requires at least one error");
    errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
}

pub(crate) fn validate_source_root_manifest(
    manifest: &SourceRootManifest,
) -> Result<ManifestValidation, Vec<ManifestDiagnostic>> {
    let mut errors = Vec::new();
    validate_version(manifest.version, &mut errors);
    validate_collection_lengths(manifest, &mut errors);
    validate_artifacts(&manifest.artifacts, &manifest.patches, &mut errors);
    validate_patches(&manifest.patches, &mut errors);
    validate_network_trust_roots(&manifest.network_trust_roots, &mut errors);
    validate_trust_notes(&manifest.trust_notes, &mut errors);
    let expected_output_roles = validate_expected_outputs(&manifest.expected_outputs, &mut errors);

    if errors.is_empty() {
        assert!(!expected_output_roles.is_empty(), "valid manifest must expose expected output roles");
        assert!(
            expected_output_roles.len() >= REQUIRED_PROVIDER_TOOL_ROLES.len(),
            "valid manifest must expose every required provider role"
        );
        Ok(ManifestValidation {
            provider_name: PROVIDER_NAME,
            provider_target: PROVIDER_TARGET,
            provider_dynamic_linker: PROVIDER_DYNAMIC_LINKER,
            expected_output_roles,
        })
    } else {
        assert!(!errors.is_empty(), "invalid manifest must retain diagnostics");
        Err(errors)
    }
}

pub(crate) fn validate_provider_dependency_trace(
    manifest: &SourceRootManifest,
    trace: &ProviderDependencyTrace,
) -> Result<(), Vec<ManifestDiagnostic>> {
    let mut errors = Vec::new();
    let allowed_authorities = allowed_network_authorities(manifest);
    let expected_roles = expected_output_roles(manifest);
    validate_trace_urls(trace, &allowed_authorities, &mut errors);
    validate_trace_hashes_and_metadata(trace, &mut errors);
    validate_emitted_roles(trace, &expected_roles, &mut errors);
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_version(version: Option<u32>, errors: &mut Vec<ManifestDiagnostic>) {
    match version {
        Some(SOURCE_ROOT_MANIFEST_VERSION) => {}
        Some(other) => errors.push(ManifestDiagnostic::new("version", format!("unsupported manifest version {other}"))),
        None => errors.push(ManifestDiagnostic::new("version", "missing manifest version")),
    }
}

fn validate_collection_lengths(manifest: &SourceRootManifest, errors: &mut Vec<ManifestDiagnostic>) {
    bounded_non_empty("artifacts", manifest.artifacts.len(), errors);
    bounded_collection("patches", manifest.patches.len(), errors);
    bounded_collection("network_trust_roots", manifest.network_trust_roots.len(), errors);
    bounded_collection("trust_notes", manifest.trust_notes.len(), errors);
    bounded_non_empty("expected_outputs", manifest.expected_outputs.len(), errors);
}

fn bounded_non_empty(path: &str, len: usize, errors: &mut Vec<ManifestDiagnostic>) {
    if len < MIN_REQUIRED_ITEM_COUNT {
        errors.push(ManifestDiagnostic::new(path, "must contain at least one entry"));
    }
    bounded_collection(path, len, errors);
}

fn bounded_collection(path: &str, len: usize, errors: &mut Vec<ManifestDiagnostic>) {
    if len > MAX_MANIFEST_ITEM_COUNT {
        errors.push(ManifestDiagnostic::new(path, format!("contains more than {MAX_MANIFEST_ITEM_COUNT} entries")));
    }
}

fn validate_artifacts(artifacts: &[SourceArtifact], patches: &[SourcePatch], errors: &mut Vec<ManifestDiagnostic>) {
    let patch_names = patches.iter().map(|patch| patch.name.as_str()).collect::<BTreeSet<_>>();
    for (idx, artifact) in artifacts.iter().enumerate() {
        let path = format!("artifacts[{idx}]");
        require_non_empty(&artifact.name, errors, &format!("{path}.name"));
        require_non_empty(&artifact.source, errors, &format!("{path}.source"));
        require_non_empty(&artifact.provenance, errors, &format!("{path}.provenance"));
        require_non_empty(&artifact.extraction.kind, errors, &format!("{path}.extraction.kind"));
        validate_digest(&artifact.digest, &format!("{path}.digest"), errors);
        validate_patch_references(&artifact.patches, &patch_names, &path, errors);
    }
}

fn validate_patch_references(
    references: &[String],
    patch_names: &BTreeSet<&str>,
    artifact_path: &str,
    errors: &mut Vec<ManifestDiagnostic>,
) {
    for patch_name in references {
        if !patch_names.contains(patch_name.as_str()) {
            errors.push(ManifestDiagnostic::new(
                format!("{artifact_path}.patches"),
                format!("undeclared patch {patch_name}"),
            ));
        }
    }
}

fn validate_patches(patches: &[SourcePatch], errors: &mut Vec<ManifestDiagnostic>) {
    for (idx, patch) in patches.iter().enumerate() {
        let path = format!("patches[{idx}]");
        require_non_empty(&patch.name, errors, &format!("{path}.name"));
        require_non_empty(&patch.provenance, errors, &format!("{path}.provenance"));
        validate_digest(&patch.digest, &format!("{path}.digest"), errors);
    }
}

fn validate_network_trust_roots(roots: &[NetworkTrustRoot], errors: &mut Vec<ManifestDiagnostic>) {
    for (idx, root) in roots.iter().enumerate() {
        let path = format!("network_trust_roots[{idx}]");
        require_non_empty(&root.authority, errors, &format!("{path}.authority"));
        require_non_empty(&root.provenance, errors, &format!("{path}.provenance"));
        require_non_empty(&root.rationale, errors, &format!("{path}.rationale"));
        validate_digest(&root.digest, &format!("{path}.digest"), errors);
    }
}

fn validate_trust_notes(notes: &[TrustNote], errors: &mut Vec<ManifestDiagnostic>) {
    for (idx, note) in notes.iter().enumerate() {
        let path = format!("trust_notes[{idx}]");
        require_non_empty(&note.name, errors, &format!("{path}.name"));
        require_non_empty(&note.provenance, errors, &format!("{path}.provenance"));
        require_non_empty(&note.scope, errors, &format!("{path}.scope"));
        require_non_empty(&note.rationale, errors, &format!("{path}.rationale"));
        validate_digest(&note.digest, &format!("{path}.digest"), errors);
    }
}

fn validate_expected_outputs(
    outputs: &[ExpectedProviderOutput],
    errors: &mut Vec<ManifestDiagnostic>,
) -> BTreeSet<String> {
    let mut roles = BTreeSet::new();
    for (idx, output) in outputs.iter().enumerate() {
        let path = format!("expected_outputs[{idx}]");
        require_non_empty(&output.name, errors, &format!("{path}.name"));
        require_non_empty(&output.kind, errors, &format!("{path}.kind"));
        require_non_empty(&output.provenance, errors, &format!("{path}.provenance"));
        require_non_empty(&output.contract_role, errors, &format!("{path}.contract_role"));
        validate_digest(&output.digest, &format!("{path}.digest"), errors);
        if !output.contract_role.trim().is_empty() {
            roles.insert(output.contract_role.clone());
        }
    }
    require_provider_roles(&roles, errors);
    roles
}

fn require_provider_roles(roles: &BTreeSet<String>, errors: &mut Vec<ManifestDiagnostic>) {
    for role in REQUIRED_PROVIDER_TOOL_ROLES {
        if !roles.contains(*role) {
            errors.push(ManifestDiagnostic::new(
                "expected_outputs",
                format!("missing required provider contract role {role}"),
            ));
        }
    }
}

fn validate_digest(digest: &DigestSpec, path: &str, errors: &mut Vec<ManifestDiagnostic>) {
    require_non_empty(&digest.algorithm, errors, &format!("{path}.algorithm"));
    require_non_empty(&digest.value, errors, &format!("{path}.value"));
    if digest.algorithm == DIGEST_ALGORITHM_BLAKE3 {
        validate_blake3_digest_value(&digest.value, errors, path);
        return;
    }
    if digest.non_blake3_reason.as_deref().unwrap_or_default().trim().is_empty() {
        errors.push(ManifestDiagnostic::new(path, "non-BLAKE3 digest requires an interoperability reason"));
    }
}

fn validate_blake3_digest_value(value: &str, errors: &mut Vec<ManifestDiagnostic>, path: &str) {
    if value.len() != BLAKE3_HEX_LEN {
        errors.push(ManifestDiagnostic::new(path, format!("BLAKE3 digest must be {BLAKE3_HEX_LEN} hex chars")));
    }
    if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        errors.push(ManifestDiagnostic::new(path, "BLAKE3 digest must be lowercase hex"));
    }
}

fn require_non_empty(value: &str, errors: &mut Vec<ManifestDiagnostic>, path: &str) {
    if value.trim().is_empty() {
        errors.push(ManifestDiagnostic::new(path, "must not be empty"));
    }
}

fn allowed_network_authorities(manifest: &SourceRootManifest) -> BTreeSet<&str> {
    let mut authorities = BTreeSet::new();
    for artifact in &manifest.artifacts {
        authorities.insert(artifact.source.as_str());
    }
    for root in &manifest.network_trust_roots {
        authorities.insert(root.authority.as_str());
    }
    authorities
}

fn expected_output_roles(manifest: &SourceRootManifest) -> BTreeSet<&str> {
    manifest.expected_outputs.iter().map(|output| output.contract_role.as_str()).collect()
}

fn validate_trace_urls(
    trace: &ProviderDependencyTrace,
    allowed_authorities: &BTreeSet<&str>,
    errors: &mut Vec<ManifestDiagnostic>,
) {
    for url in &trace.urls {
        if url == LEGACY_MUSL_CC_URL {
            errors.push(ManifestDiagnostic::new("dependency_trace.urls", "legacy musl.cc seed tarball is forbidden"));
            continue;
        }
        if !allowed_authorities.contains(url.as_str()) {
            errors.push(ManifestDiagnostic::new(
                "dependency_trace.urls",
                format!("unmanifested network trust root {url}"),
            ));
        }
    }
}

fn validate_trace_hashes_and_metadata(trace: &ProviderDependencyTrace, errors: &mut Vec<ManifestDiagnostic>) {
    for hash in &trace.hashes {
        if hash == LEGACY_MUSL_CC_HASH {
            errors.push(ManifestDiagnostic::new("dependency_trace.hashes", "legacy musl.cc seed hash is forbidden"));
        }
    }
    for value in &trace.provider_metadata {
        if value.contains(LEGACY_MUSL_CC_URL) || value.contains(LEGACY_MUSL_CC_HASH) {
            errors.push(ManifestDiagnostic::new(
                "dependency_trace.provider_metadata",
                "legacy musl.cc seed metadata is forbidden",
            ));
        }
    }
}

fn validate_emitted_roles(
    trace: &ProviderDependencyTrace,
    expected_roles: &BTreeSet<&str>,
    errors: &mut Vec<ManifestDiagnostic>,
) {
    let emitted_roles: BTreeSet<&str> = trace.emitted_output_roles.iter().map(String::as_str).collect();
    for role in &trace.emitted_output_roles {
        if !expected_roles.contains(role.as_str()) {
            errors.push(ManifestDiagnostic::new(
                "dependency_trace.emitted_output_roles",
                format!("unexpected provider output role {role}"),
            ));
        }
    }
    for role in expected_roles.iter().copied() {
        if !emitted_roles.contains(role) {
            errors.push(ManifestDiagnostic::new(
                "dependency_trace.emitted_output_roles",
                format!("missing provider output role {role}"),
            ));
        }
    }
}

pub(crate) const STAGEX_LINEAGE_PROVIDER_NOT_MATERIALIZED: &str = "StageX-class lineage provider materialization not yet implemented: \
     the hex0-seed to normalized-provider stage0 chain requires a separate integration change";

pub(crate) fn validate_stagex_lineage_manifest(
    manifest_bytes: &[u8],
) -> Result<crunch_bootstrap_core::LineageManifest, Vec<ManifestDiagnostic>> {
    assert!(!manifest_bytes.is_empty(), "lineage manifest bytes must not be empty");
    let manifest: crunch_bootstrap_core::LineageManifest = serde_json::from_slice(manifest_bytes).map_err(|err| {
        vec![ManifestDiagnostic::new(
            "stagex-lineage-manifest",
            format!("parsing StageX lineage manifest: {err}"),
        )]
    })?;

    let result = crunch_bootstrap_core::validate_lineage(&manifest);
    if !result.is_valid() {
        let diagnostics = result
            .errors
            .into_iter()
            .map(|e| ManifestDiagnostic::new("stagex-lineage-validation", e.to_string()))
            .collect();
        return Err(diagnostics);
    }

    Ok(manifest)
}

pub(crate) fn classify_stagex_provider_evidence(
    manifest: &crunch_bootstrap_core::LineageManifest,
) -> StagexProviderEvidence {
    let seed_class = manifest.seed.seed_class.as_str().to_string();
    let seed_digest = manifest.seed.seed_digest.hex_value.clone();
    let environment_assumptions: Vec<String> = manifest
        .environment_assumptions
        .iter()
        .map(|a| format!("{}: {}", a.category, a.description))
        .collect();
    StagexProviderEvidence {
        seed_class,
        seed_digest,
        audit_seed_max_bytes: manifest.seed.audit_seed_max_bytes,
        provider_output_count: manifest.provider_outputs.len() as u32,
        environment_assumptions,
        materialized: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StagexProviderEvidence {
    pub seed_class: String,
    pub seed_digest: String,
    pub audit_seed_max_bytes: u32,
    pub provider_output_count: u32,
    pub environment_assumptions: Vec<String>,
    pub materialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SAMPLE_DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SAMPLE_SOURCE_URL: &str = "https://example.invalid/musl.tar.gz";
    const SAMPLE_NETWORK_ROOT: &str = "https://example.invalid";
    const SAMPLE_PATCH_NAME: &str = "fix-bootstrap.patch";
    const SAMPLE_EXTRA_ROLE: &str = "unexpected-extra";
    const SAMPLE_OUTPUT_KIND: &str = "file";
    const SAMPLE_PROVENANCE: &str = "checked-in source-root manifest fixture";
    const FIRST_PATCH_ORDER: u32 = 1;

    fn sample_digest(value: &str) -> DigestSpec {
        DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            value: value.to_string(),
            non_blake3_reason: None,
        }
    }

    fn sample_manifest() -> SourceRootManifest {
        SourceRootManifest {
            version: Some(SOURCE_ROOT_MANIFEST_VERSION),
            artifacts: vec![sample_artifact()],
            patches: vec![sample_patch()],
            network_trust_roots: vec![sample_network_root()],
            trust_notes: vec![sample_trust_note()],
            expected_outputs: sample_expected_outputs(),
        }
    }

    fn sample_artifact() -> SourceArtifact {
        SourceArtifact {
            name: "musl-source".to_string(),
            source: SAMPLE_SOURCE_URL.to_string(),
            digest: sample_digest(SAMPLE_DIGEST_A),
            extraction: ExtractionRule {
                kind: "tar.gz".to_string(),
                strip_prefix: Some("musl".to_string()),
            },
            provenance: SAMPLE_PROVENANCE.to_string(),
            patches: vec![SAMPLE_PATCH_NAME.to_string()],
        }
    }

    fn sample_patch() -> SourcePatch {
        SourcePatch {
            name: SAMPLE_PATCH_NAME.to_string(),
            digest: sample_digest(SAMPLE_DIGEST_B),
            provenance: SAMPLE_PROVENANCE.to_string(),
            apply_order: FIRST_PATCH_ORDER,
        }
    }

    fn sample_network_root() -> NetworkTrustRoot {
        NetworkTrustRoot {
            authority: SAMPLE_NETWORK_ROOT.to_string(),
            digest: sample_digest(SAMPLE_DIGEST_A),
            provenance: SAMPLE_PROVENANCE.to_string(),
            rationale: "pins test mirror identity".to_string(),
        }
    }

    fn sample_trust_note() -> TrustNote {
        TrustNote {
            name: "host-cc".to_string(),
            digest: sample_digest(SAMPLE_DIGEST_B),
            provenance: SAMPLE_PROVENANCE.to_string(),
            scope: "phase-1 provider construction only".to_string(),
            rationale: "host-tool-free bootstrap is separate change".to_string(),
        }
    }

    fn sample_expected_outputs() -> Vec<ExpectedProviderOutput> {
        REQUIRED_PROVIDER_TOOL_ROLES
            .iter()
            .map(|role| ExpectedProviderOutput {
                name: (*role).to_string(),
                kind: SAMPLE_OUTPUT_KIND.to_string(),
                digest: sample_digest(SAMPLE_DIGEST_A),
                provenance: SAMPLE_PROVENANCE.to_string(),
                contract_role: (*role).to_string(),
            })
            .collect()
    }

    fn assert_error_contains(errors: &[ManifestDiagnostic], expected: &str) {
        assert!(!errors.is_empty(), "negative fixture must produce diagnostics");
        let joined = errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
        assert!(joined.contains(expected), "diagnostics must contain {expected:?}:\n{joined}");
    }

    #[test]
    fn valid_source_root_manifest_is_accepted() {
        let manifest = sample_manifest();
        let validated = validate_source_root_manifest(&manifest).unwrap();
        assert_eq!(validated.provider_name, PROVIDER_NAME);
        assert_eq!(validated.provider_target, PROVIDER_TARGET);
        assert_eq!(validated.provider_dynamic_linker, PROVIDER_DYNAMIC_LINKER);
        assert!(validated.expected_output_roles.contains(PROVIDER_METADATA_ROLE));
        assert_eq!(validated.expected_output_roles.len(), REQUIRED_PROVIDER_TOOL_ROLES.len());
    }

    #[test]
    fn bootstrap_provider_selection_rejects_ambiguous_modes() {
        let source_root = Path::new("source-root.json");
        let err = select_bootstrap_provider(true, Some(source_root), None).unwrap_err();
        assert_eq!(err.path, "provider-selection");

        let lineage = Path::new("lineage.json");
        let err = select_bootstrap_provider(true, None, Some(lineage)).unwrap_err();
        assert_eq!(err.path, "provider-selection");

        let err = select_bootstrap_provider(false, Some(source_root), Some(lineage)).unwrap_err();
        assert_eq!(err.path, "provider-selection");
    }

    #[test]
    fn bootstrap_provider_selection_accepts_each_single_mode() {
        let source_root = Path::new("source-root.json");
        let lineage = Path::new("lineage.json");
        assert_eq!(
            select_bootstrap_provider(false, Some(source_root), None).unwrap(),
            BootstrapProviderMode::SourceRoot
        );
        assert_eq!(select_bootstrap_provider(true, None, None).unwrap(), BootstrapProviderMode::LegacyFetch);
        assert_eq!(select_bootstrap_provider(false, None, None).unwrap(), BootstrapProviderMode::NixPackages);
        assert_eq!(
            select_bootstrap_provider(false, None, Some(lineage)).unwrap(),
            BootstrapProviderMode::StagexLineage
        );
    }

    #[test]
    fn stagex_lineage_manifest_valid_json_passes_validation() {
        let manifest_json = serde_json::json!({
            "seed": {
                "seed_class": "hex0_seed",
                "instruction_set": "x86",
                "entry_point": "0x00",
                "io_contract": "stdin/stdout",
                "host_interface_surface": "linux syscalls",
                "human_readable_source": "bootstrap/hex0.hex0",
                "reproduction_transcript": "hex0 self-hosts",
                "audit_note": "Hand-audited seed",
                "audit_seed_max_bytes": 4096,
                "seed_bytes_len": 357,
                "seed_digest": {"algorithm": "blake3", "hex_value": "a".repeat(64)}
            },
            "source_artifacts": [{
                "id": "hex0-src", "name": "hex0 source",
                "digest": {"algorithm": "blake3", "hex_value": "b".repeat(64)},
                "provenance": "stage0-posix"
            }],
            "generated_artifacts": [{
                "id": "hex0-bin", "name": "hex0 binary",
                "producing_tool_id": "hex0-asm",
                "digest": {"algorithm": "blake3", "hex_value": "c".repeat(64)}
            }],
            "transition_tools": [{
                "id": "hex0-asm", "name": "hex0 assembler",
                "source_artifact_ids": ["hex0-src"],
                "input_artifact_ids": [],
                "output_artifact_ids": ["hex0-bin"],
                "digest": {"algorithm": "blake3", "hex_value": "d".repeat(64)}
            }],
            "patches": [],
            "provider_outputs": [
                {"role": "target_prefixed_tools", "producing_artifact_id": "hex0-bin"}
            ],
            "environment_assumptions": [
                {"id": "kernel", "description": "Linux x86_64", "category": "kernel"}
            ],
            "stage_graph": [{"from_node_id": "seed", "to_node_id": "hex0-src"}]
        });
        let bytes = serde_json::to_vec(&manifest_json).unwrap();
        let manifest = validate_stagex_lineage_manifest(&bytes).unwrap();
        assert_eq!(manifest.seed.seed_class, crunch_bootstrap_core::SeedClass::Hex0Seed);
    }

    #[test]
    fn stagex_lineage_manifest_invalid_json_rejected() {
        let result = validate_stagex_lineage_manifest(b"not json");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags[0].path.contains("stagex-lineage"));
    }

    #[test]
    fn stagex_lineage_manifest_forbidden_root_rejected() {
        let manifest_json = serde_json::json!({
            "seed": {
                "seed_class": "hex0_seed",
                "instruction_set": "x86", "entry_point": "0x00",
                "io_contract": "io", "host_interface_surface": "linux",
                "human_readable_source": "src", "reproduction_transcript": "t",
                "audit_note": "n", "audit_seed_max_bytes": 4096, "seed_bytes_len": 100,
                "seed_digest": {"algorithm": "blake3", "hex_value": "a".repeat(64)}
            },
            "source_artifacts": [],
            "generated_artifacts": [],
            "transition_tools": [{
                "id": "host-gcc", "name": "gcc",
                "source_artifact_ids": [], "input_artifact_ids": [],
                "output_artifact_ids": [],
                "digest": {"algorithm": "blake3", "hex_value": "b".repeat(64)}
            }],
            "patches": [],
            "provider_outputs": [],
            "environment_assumptions": [],
            "stage_graph": []
        });
        let bytes = serde_json::to_vec(&manifest_json).unwrap();
        let result = validate_stagex_lineage_manifest(&bytes);
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags.iter().any(|d| d.message.contains("forbidden")));
    }

    #[test]
    fn stagex_provider_evidence_classification() {
        let manifest_json = serde_json::json!({
            "seed": {
                "seed_class": "hex0_seed",
                "instruction_set": "x86", "entry_point": "0x00",
                "io_contract": "io", "host_interface_surface": "linux",
                "human_readable_source": "src", "reproduction_transcript": "t",
                "audit_note": "n", "audit_seed_max_bytes": 4096, "seed_bytes_len": 100,
                "seed_digest": {"algorithm": "blake3", "hex_value": "a".repeat(64)}
            },
            "source_artifacts": [{
                "id": "s1", "name": "source",
                "digest": {"algorithm": "blake3", "hex_value": "b".repeat(64)},
                "provenance": "stage0"
            }],
            "generated_artifacts": [{"id": "g1", "name": "gen", "producing_tool_id": "t1",
                "digest": {"algorithm": "blake3", "hex_value": "c".repeat(64)}
            }],
            "transition_tools": [{"id": "t1", "name": "tool",
                "source_artifact_ids": ["s1"], "input_artifact_ids": [],
                "output_artifact_ids": ["g1"],
                "digest": {"algorithm": "blake3", "hex_value": "d".repeat(64)}
            }],
            "patches": [],
            "provider_outputs": [{"role": "target_prefixed_tools", "producing_artifact_id": "g1"}],
            "environment_assumptions": [{"id": "k", "description": "Linux", "category": "kernel"}],
            "stage_graph": [{"from_node_id": "seed", "to_node_id": "s1"}]
        });
        let bytes = serde_json::to_vec(&manifest_json).unwrap();
        let manifest = validate_stagex_lineage_manifest(&bytes).unwrap();
        let evidence = classify_stagex_provider_evidence(&manifest);
        assert_eq!(evidence.seed_class, "hex0-seed");
        assert_eq!(evidence.audit_seed_max_bytes, 4096);
        assert_eq!(evidence.provider_output_count, 1);
        assert!(!evidence.materialized);
        assert!(evidence.environment_assumptions.iter().any(|a| a.contains("kernel")));
    }

    #[test]
    fn provider_mode_stagex_satisfies_requirement() {
        assert!(BootstrapProviderMode::StagexLineage.satisfies_stagex_requirement());
    }

    #[test]
    fn provider_mode_legacy_does_not_satisfy_stagex() {
        assert!(!BootstrapProviderMode::LegacyFetch.satisfies_stagex_requirement());
        assert!(!BootstrapProviderMode::SourceRoot.satisfies_stagex_requirement());
        assert!(!BootstrapProviderMode::NixPackages.satisfies_stagex_requirement());
    }

    #[test]
    fn provider_mode_roundtrip() {
        for mode in [
            BootstrapProviderMode::NixPackages,
            BootstrapProviderMode::LegacyFetch,
            BootstrapProviderMode::SourceRoot,
            BootstrapProviderMode::StagexLineage,
        ] {
            let parsed = BootstrapProviderMode::parse(mode.as_str()).unwrap();
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn source_root_manifest_digest_is_blake3_of_bytes() {
        let bytes = br#"{"schema":"source-root"}"#;
        let digest = source_root_manifest_digest(bytes);
        assert_eq!(digest, blake3::hash(bytes).to_hex().to_string());
        assert_eq!(digest.len(), BLAKE3_HEX_LEN);
    }

    #[test]
    fn missing_manifest_version_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.version = None;
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "missing manifest version");
    }

    #[test]
    fn unsupported_manifest_version_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.version = Some(SOURCE_ROOT_MANIFEST_VERSION.saturating_add(FIRST_PATCH_ORDER));
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "unsupported manifest version");
    }

    #[test]
    fn missing_artifact_digest_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.artifacts[0].digest.value.clear();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "artifacts[0].digest.value");
    }

    #[test]
    fn non_blake3_digest_without_reason_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.artifacts[0].digest.algorithm = "sha256".to_string();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "non-BLAKE3 digest requires an interoperability reason");
    }

    #[test]
    fn missing_extraction_rule_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.artifacts[0].extraction.kind.clear();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "artifacts[0].extraction.kind");
    }

    #[test]
    fn undeclared_patch_reference_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.patches.clear();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "undeclared patch");
    }

    #[test]
    fn network_trust_root_missing_rationale_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.network_trust_roots[0].rationale.clear();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "network_trust_roots[0].rationale");
    }

    #[test]
    fn trust_note_missing_scope_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.trust_notes[0].scope.clear();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "trust_notes[0].scope");
    }

    #[test]
    fn expected_output_missing_contract_role_is_rejected() {
        let mut manifest = sample_manifest();
        manifest.expected_outputs[0].contract_role.clear();
        let errors = validate_source_root_manifest(&manifest).unwrap_err();
        assert_error_contains(&errors, "expected_outputs[0].contract_role");
    }

    #[test]
    fn provider_trace_rejects_unmanifested_network_root() {
        let manifest = sample_manifest();
        let trace = ProviderDependencyTrace {
            urls: vec!["https://unlisted.invalid/source.tar.gz".to_string()],
            hashes: Vec::new(),
            emitted_output_roles: Vec::new(),
            provider_metadata: Vec::new(),
        };
        let errors = validate_provider_dependency_trace(&manifest, &trace).unwrap_err();
        assert_error_contains(&errors, "unmanifested network trust root");
    }

    #[test]
    fn provider_trace_rejects_legacy_musl_cc_url_and_hash() {
        let manifest = sample_manifest();
        let trace = ProviderDependencyTrace {
            urls: vec![LEGACY_MUSL_CC_URL.to_string()],
            hashes: vec![LEGACY_MUSL_CC_HASH.to_string()],
            emitted_output_roles: Vec::new(),
            provider_metadata: vec![format!("raw={LEGACY_MUSL_CC_URL}")],
        };
        let errors = validate_provider_dependency_trace(&manifest, &trace).unwrap_err();
        assert_error_contains(&errors, "legacy musl.cc seed tarball is forbidden");
        assert_error_contains(&errors, "legacy musl.cc seed hash is forbidden");
        assert_error_contains(&errors, "legacy musl.cc seed metadata is forbidden");
    }

    #[test]
    fn provider_trace_rejects_unexpected_output_role() {
        let manifest = sample_manifest();
        let trace = ProviderDependencyTrace {
            urls: vec![SAMPLE_SOURCE_URL.to_string()],
            hashes: Vec::new(),
            emitted_output_roles: vec![SAMPLE_EXTRA_ROLE.to_string()],
            provider_metadata: Vec::new(),
        };
        let errors = validate_provider_dependency_trace(&manifest, &trace).unwrap_err();
        assert_error_contains(&errors, "unexpected provider output role");
    }

    #[test]
    fn provider_trace_requires_all_expected_output_roles() {
        let manifest = sample_manifest();
        let trace = ProviderDependencyTrace {
            urls: vec![SAMPLE_SOURCE_URL.to_string()],
            hashes: Vec::new(),
            emitted_output_roles: vec![PROVIDER_METADATA_ROLE.to_string()],
            provider_metadata: Vec::new(),
        };
        let errors = validate_provider_dependency_trace(&manifest, &trace).unwrap_err();
        assert_error_contains(&errors, "missing provider output role");
    }

    #[test]
    fn provider_trace_accepts_exact_expected_output_roles() {
        let manifest = sample_manifest();
        let trace = ProviderDependencyTrace {
            urls: vec![SAMPLE_SOURCE_URL.to_string()],
            hashes: Vec::new(),
            emitted_output_roles: REQUIRED_PROVIDER_TOOL_ROLES.iter().map(|role| (*role).to_string()).collect(),
            provider_metadata: vec!["manifest_digest=test".to_string()],
        };
        validate_provider_dependency_trace(&manifest, &trace).unwrap();
    }
}
