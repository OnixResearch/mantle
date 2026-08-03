// r[impl mantlepkgs.catalog_manifest]
// r[verify mantlepkgs.catalog_manifest]

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::CoreFailure;
use crate::Diagnostic;

pub const MANIFEST_SCHEMA: &str = "mantlepkgs-manifest-v1";
pub const SUPPORTED_SYSTEM_X86_64_LINUX: &str = "x86_64-linux";
pub const RECOMPUTE_CONVERSION_MODE: &str = "recompute-blake3-v1";
pub const SOURCE_BUNDLE_POLICY_MODE: &str = "source-bundle-required-v1";
pub const NARIO_V2_TRANSPORT: &str = "nario-v2";
pub const PRODUCER_COMMAND_CLASS: &str = "nix-eval-drv-path-derivation-show-recursive-and-build-fixed-output-seeds-v1";
pub const BLAKE3_HEX_LENGTH: usize = 64;
pub const GIT_REVISION_HEX_LENGTH: usize = 40;
pub const MAX_MANIFEST_SELECTORS: u32 = 256;
pub const MAX_MANIFEST_GRAPH_NODES: u32 = 65_536;
pub const MAX_MANIFEST_GRAPH_BYTES: u64 = 1_073_741_824;
pub const MAX_MANIFEST_SOURCE_REQUIREMENTS: u32 = 65_536;
pub const MAX_MANIFEST_ARTIFACT_BYTES: u64 = 2_147_483_648;
const MAX_TEXT_BYTES: usize = 4_096;
const MAX_ALIAS_COUNT: usize = 64;
const MIN_COLLECTION_ITEMS: u32 = 1;
const MANIFEST_DOMAIN: &[u8] = b"mantle.mantlepkgs.manifest.v1";
const DOMAIN_SEPARATOR: u8 = 0;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MantlepkgsManifest {
    pub schema: String,
    pub source: NixpkgsSourceLock,
    pub systems: Vec<String>,
    pub selectors: Vec<PackageSelector>,
    pub conversion_policy: ConversionPolicy,
    pub source_policy: SourcePolicy,
    pub output: OutputLayout,
    pub limits: ManifestLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NixpkgsSourceLock {
    pub reference: String,
    pub revision: String,
    pub lock_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageSelector {
    pub name: String,
    pub attribute: String,
    pub system: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConversionPolicy {
    pub mode: String,
    pub target_store_prefix: String,
    pub translation_policy: ManifestArtifact,
    pub execution_profile: ManifestArtifact,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestArtifact {
    pub path: String,
    pub digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePolicy {
    pub mode: String,
    pub optional_transports: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutputLayout {
    pub generation_directory: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestLimits {
    pub max_selectors: u32,
    pub max_graph_nodes: u32,
    pub max_graph_bytes: u64,
    pub max_source_requirements: u32,
    pub max_artifact_bytes: u64,
}

pub fn normalize_manifest(manifest: &MantlepkgsManifest) -> Result<MantlepkgsManifest, CoreFailure> {
    let mut diagnostics = validate_manifest(manifest);
    if !diagnostics.is_empty() {
        return Err(CoreFailure::from_diagnostics(diagnostics));
    }
    let mut normalized = manifest.clone();
    normalized.systems.sort();
    normalized.systems.dedup();
    for selector in &mut normalized.selectors {
        selector.aliases.sort();
        selector.aliases.dedup();
    }
    normalized.selectors.sort();
    normalized.source_policy.optional_transports.sort();
    normalized.source_policy.optional_transports.dedup();
    diagnostics = validate_manifest(&normalized);
    if !diagnostics.is_empty() {
        return Err(CoreFailure::from_diagnostics(diagnostics));
    }
    debug_assert!(!normalized.selectors.is_empty());
    debug_assert!(!normalized.systems.is_empty());
    Ok(normalized)
}

pub fn manifest_digest_blake3(manifest: &MantlepkgsManifest) -> Result<String, CoreFailure> {
    let normalized = normalize_manifest(manifest)?;
    let bytes = serde_json::to_vec(&normalized).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "manifest-serialization-failed",
            "manifest",
            "the normalized manifest did not serialize",
        ))
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(MANIFEST_DOMAIN);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().as_str().into())
}

fn validate_manifest(manifest: &MantlepkgsManifest) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    require_equal(&manifest.schema, MANIFEST_SCHEMA, "schema", "unsupported-manifest-schema", &mut diagnostics);
    validate_source_lock(&manifest.source, &mut diagnostics);
    validate_systems(manifest, &mut diagnostics);
    validate_selectors(manifest, &mut diagnostics);
    validate_conversion_policy(&manifest.conversion_policy, &mut diagnostics);
    validate_source_policy(&manifest.source_policy, &mut diagnostics);
    validate_relative_path(&manifest.output.generation_directory, "output.generation_directory", &mut diagnostics);
    validate_limits(&manifest.limits, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_source_lock(source: &NixpkgsSourceLock, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    require_text(&source.reference, "source.reference", diagnostics);
    require_text(&source.revision, "source.revision", diagnostics);
    require_hex(&source.lock_digest_blake3, BLAKE3_HEX_LENGTH, "source.lock_digest_blake3", diagnostics);
    if !is_lower_hex_length(&source.revision, GIT_REVISION_HEX_LENGTH)
        && !is_lower_hex_length(&source.revision, BLAKE3_HEX_LENGTH)
    {
        diagnostics.push(Diagnostic::new(
            "floating-source-lock",
            "source.revision",
            "the source revision must be an exact lowercase hexadecimal identity",
        ));
    }
    if !source.reference.contains(&source.revision) {
        diagnostics.push(Diagnostic::new(
            "source-reference-not-locked",
            "source.reference",
            "the source reference must contain the exact revision",
        ));
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.iter().skip(initial_diagnostic_count).all(|item| !item.code.is_empty()));
}

fn validate_systems(manifest: &MantlepkgsManifest, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    if manifest.systems.is_empty() {
        diagnostics.push(Diagnostic::new("missing-system", "systems", "the manifest must select at least one system"));
    }
    let mut systems = BTreeSet::new();
    for (index, system) in manifest.systems.iter().enumerate() {
        let path = format!("systems[{index}]");
        require_text(system, &path, diagnostics);
        if system != SUPPORTED_SYSTEM_X86_64_LINUX {
            diagnostics.push(Diagnostic::new(
                "unsupported-system",
                &path,
                "the selected system is not supported by this Mantlepkgs version",
            ));
        }
        if !systems.insert(system) {
            diagnostics.push(Diagnostic::new("duplicate-system", &path, "the manifest repeats a system"));
        }
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(systems.len() <= manifest.systems.len());
}

fn validate_selectors(manifest: &MantlepkgsManifest, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    if manifest.selectors.is_empty() {
        diagnostics.push(Diagnostic::new(
            "missing-selector",
            "selectors",
            "the manifest must select at least one package",
        ));
    }
    if exceeds_u32_limit(manifest.selectors.len(), manifest.limits.max_selectors)
        || exceeds_u32_limit(manifest.selectors.len(), MAX_MANIFEST_SELECTORS)
    {
        diagnostics.push(Diagnostic::new(
            "selector-limit-exceeded",
            "selectors",
            "the selector count exceeds a named limit",
        ));
    }
    let allowed_systems = manifest.systems.iter().collect::<BTreeSet<_>>();
    let mut selector_keys = BTreeSet::new();
    let mut aliases = BTreeMap::<(&str, &str), &str>::new();
    for (index, selector) in manifest.selectors.iter().enumerate() {
        validate_selector(index, selector, &allowed_systems, &mut selector_keys, &mut aliases, diagnostics);
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(selector_keys.len() <= manifest.selectors.len());
}

#[allow(tigerstyle::too_many_parameters)] // Selector validation threads bounded uniqueness maps and one ordered diagnostic sink.
fn validate_selector<'a>(
    index: usize,
    selector: &'a PackageSelector,
    allowed_systems: &BTreeSet<&String>,
    selector_keys: &mut BTreeSet<(&'a str, &'a str)>,
    aliases: &mut BTreeMap<(&'a str, &'a str), &'a str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let initial_diagnostic_count = diagnostics.len();
    let base = format!("selectors[{index}]");
    require_text(&selector.name, &format!("{base}.name"), diagnostics);
    require_text(&selector.attribute, &format!("{base}.attribute"), diagnostics);
    require_text(&selector.system, &format!("{base}.system"), diagnostics);
    if !allowed_systems.contains(&selector.system) {
        diagnostics.push(Diagnostic::new(
            "selector-system-not-declared",
            &format!("{base}.system"),
            "the selector system is absent from the manifest system set",
        ));
    }
    if !selector_keys.insert((&selector.system, &selector.name)) {
        diagnostics.push(Diagnostic::new("duplicate-selector", &base, "the manifest repeats a package selector"));
    }
    if selector.aliases.len() > MAX_ALIAS_COUNT {
        diagnostics.push(Diagnostic::new(
            "alias-limit-exceeded",
            &format!("{base}.aliases"),
            "the selector alias count exceeds the limit",
        ));
    }
    register_alias(&selector.system, &selector.name, &selector.name, &base, aliases, diagnostics);
    let mut local_aliases = BTreeSet::new();
    for (alias_index, alias) in selector.aliases.iter().enumerate() {
        let path = format!("{base}.aliases[{alias_index}]");
        require_text(alias, &path, diagnostics);
        if !local_aliases.insert(alias) {
            diagnostics.push(Diagnostic::new("duplicate-alias", &path, "the selector repeats an alias"));
        }
        register_alias(&selector.system, alias, &selector.name, &path, aliases, diagnostics);
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(local_aliases.len() <= selector.aliases.len());
}

#[allow(tigerstyle::ambiguous_params)] // System, alias, owner, and path name separate alias-registry roles.
#[allow(tigerstyle::too_many_parameters)] // The helper mutates one alias map and one diagnostic sink for a named alias tuple.
fn register_alias<'a>(
    system: &'a str,
    alias: &'a str,
    owner: &'a str,
    path: &str,
    aliases: &mut BTreeMap<(&'a str, &'a str), &'a str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(previous) = aliases.insert((system, alias), owner)
        && previous != owner
    {
        diagnostics.push(Diagnostic::new(
            "ambiguous-alias",
            path,
            "the alias resolves to more than one selector for the system",
        ));
    }
}

fn validate_conversion_policy(policy: &ConversionPolicy, diagnostics: &mut Vec<Diagnostic>) {
    require_equal(
        &policy.mode,
        RECOMPUTE_CONVERSION_MODE,
        "conversion_policy.mode",
        "unsupported-conversion-mode",
        diagnostics,
    );
    require_absolute_store_prefix(&policy.target_store_prefix, diagnostics);
    validate_manifest_artifact(&policy.translation_policy, "conversion_policy.translation_policy", diagnostics);
    validate_manifest_artifact(&policy.execution_profile, "conversion_policy.execution_profile", diagnostics);
}

fn validate_manifest_artifact(artifact: &ManifestArtifact, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_relative_path(&artifact.path, &format!("{path}.path"), diagnostics);
    require_hex(&artifact.digest_blake3, BLAKE3_HEX_LENGTH, &format!("{path}.digest_blake3"), diagnostics);
}

fn validate_source_policy(policy: &SourcePolicy, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    require_equal(
        &policy.mode,
        SOURCE_BUNDLE_POLICY_MODE,
        "source_policy.mode",
        "unsupported-source-policy",
        diagnostics,
    );
    let mut seen = BTreeSet::new();
    for (index, transport) in policy.optional_transports.iter().enumerate() {
        let path = format!("source_policy.optional_transports[{index}]");
        if transport != NARIO_V2_TRANSPORT {
            diagnostics.push(Diagnostic::new(
                "unsupported-source-transport",
                &path,
                "the optional source transport is not supported",
            ));
        }
        if !seen.insert(transport) {
            diagnostics.push(Diagnostic::new(
                "duplicate-source-transport",
                &path,
                "the optional source transport is repeated",
            ));
        }
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(seen.len() <= policy.optional_transports.len());
}

fn validate_limits(limits: &ManifestLimits, diagnostics: &mut Vec<Diagnostic>) {
    validate_u32_limit(limits.max_selectors, MAX_MANIFEST_SELECTORS, "limits.max_selectors", diagnostics);
    validate_u32_limit(limits.max_graph_nodes, MAX_MANIFEST_GRAPH_NODES, "limits.max_graph_nodes", diagnostics);
    validate_u64_limit(limits.max_graph_bytes, MAX_MANIFEST_GRAPH_BYTES, "limits.max_graph_bytes", diagnostics);
    validate_u32_limit(
        limits.max_source_requirements,
        MAX_MANIFEST_SOURCE_REQUIREMENTS,
        "limits.max_source_requirements",
        diagnostics,
    );
    validate_u64_limit(
        limits.max_artifact_bytes,
        MAX_MANIFEST_ARTIFACT_BYTES,
        "limits.max_artifact_bytes",
        diagnostics,
    );
}

#[allow(tigerstyle::ambiguous_params)] // Observed value and implementation maximum are separate numeric bounds.
fn validate_u32_limit(value: u32, maximum: u32, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value < MIN_COLLECTION_ITEMS || value > maximum {
        diagnostics.push(Diagnostic::new(
            "invalid-named-limit",
            path,
            "the named limit is zero or exceeds the implementation limit",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Observed value and implementation maximum are separate numeric bounds.
fn validate_u64_limit(value: u64, maximum: u64, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value < u64::from(MIN_COLLECTION_ITEMS) || value > maximum {
        diagnostics.push(Diagnostic::new(
            "invalid-named-limit",
            path,
            "the named limit is zero or exceeds the implementation limit",
        ));
    }
}

fn require_absolute_store_prefix(value: &str, diagnostics: &mut Vec<Diagnostic>) {
    require_text(value, "conversion_policy.target_store_prefix", diagnostics);
    let is_absolute = value.starts_with('/');
    let has_unsafe_component = [value.ends_with('/'), value.contains("//"), value.contains("..")]
        .into_iter()
        .any(|is_present| is_present);
    if !is_absolute || has_unsafe_component {
        diagnostics.push(Diagnostic::new(
            "unsafe-target-store-prefix",
            "conversion_policy.target_store_prefix",
            "the target store prefix must be one normalized absolute path",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Path value and diagnostic field path are separate validation inputs.
fn validate_relative_path(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    require_text(value, path, diagnostics);
    let is_unsafe_path = value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.split('/').any(|component| component.is_empty() || component == "." || component == "..");
    if is_unsafe_path {
        diagnostics.push(Diagnostic::new(
            "unsafe-relative-path",
            path,
            "the path must be normalized, relative, and confined",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Value, expected contract, field path, and code are distinct validation roles.
fn require_equal(value: &str, expected: &str, path: &str, code: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value != expected {
        diagnostics.push(Diagnostic::new(code, path, "the value does not match the supported contract"));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Text value and diagnostic field path are distinct validation inputs.
fn require_text(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        diagnostics.push(Diagnostic::new(
            "invalid-text-field",
            path,
            "the text field is empty, too large, or contains a control character",
        ));
    }
}

fn require_hex(value: &str, length: usize, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !is_lower_hex_length(value, length) {
        diagnostics.push(Diagnostic::new(
            "invalid-blake3-digest",
            path,
            "the digest must be lowercase hexadecimal with the required length",
        ));
    }
}

fn is_lower_hex_length(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn exceeds_u32_limit(observed_items: usize, limit_items: u32) -> bool {
    match u32::try_from(observed_items) {
        Ok(observed_items_u32) => observed_items_u32 > limit_items,
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DEFAULT_SELECTOR_LIMIT: u32 = 8;
    const DEFAULT_GRAPH_NODE_LIMIT: u32 = 1_024;
    const DEFAULT_GRAPH_BYTE_LIMIT: u64 = 16_777_216;
    const DEFAULT_SOURCE_LIMIT: u32 = 1_024;
    const DEFAULT_ARTIFACT_BYTE_LIMIT: u64 = 33_554_432;

    fn manifest() -> MantlepkgsManifest {
        MantlepkgsManifest {
            schema: MANIFEST_SCHEMA.into(),
            source: NixpkgsSourceLock {
                reference: format!("github:NixOS/nixpkgs/{REVISION}"),
                revision: REVISION.into(),
                lock_digest_blake3: DIGEST.into(),
            },
            systems: alloc::vec![SUPPORTED_SYSTEM_X86_64_LINUX.into()],
            selectors: alloc::vec![PackageSelector {
                name: "hello".into(),
                attribute: "hello".into(),
                system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                aliases: alloc::vec!["hi".into(), "hello-world".into()],
            }],
            conversion_policy: ConversionPolicy {
                mode: RECOMPUTE_CONVERSION_MODE.into(),
                target_store_prefix: "/mantle/store".into(),
                translation_policy: ManifestArtifact {
                    path: "policy/translation.json".into(),
                    digest_blake3: DIGEST.into(),
                },
                execution_profile: ManifestArtifact {
                    path: "policy/execution.json".into(),
                    digest_blake3: DIGEST.into(),
                },
            },
            source_policy: SourcePolicy {
                mode: SOURCE_BUNDLE_POLICY_MODE.into(),
                optional_transports: alloc::vec![NARIO_V2_TRANSPORT.into()],
            },
            output: OutputLayout {
                generation_directory: "mantlepkgs/generations".into(),
            },
            limits: ManifestLimits {
                max_selectors: DEFAULT_SELECTOR_LIMIT,
                max_graph_nodes: DEFAULT_GRAPH_NODE_LIMIT,
                max_graph_bytes: DEFAULT_GRAPH_BYTE_LIMIT,
                max_source_requirements: DEFAULT_SOURCE_LIMIT,
                max_artifact_bytes: DEFAULT_ARTIFACT_BYTE_LIMIT,
            },
        }
    }

    #[test]
    fn valid_manifest_normalizes_and_hashes_deterministically() {
        let first = manifest();
        let mut second = manifest();
        second.selectors[0].aliases = alloc::vec!["hello-world".into(), "hi".into()];

        let first_normalized = normalize_manifest(&first).expect("valid manifest normalizes");
        let second_normalized = normalize_manifest(&second).expect("reordered aliases normalize");
        assert_eq!(first_normalized, second_normalized);
        assert_eq!(
            manifest_digest_blake3(&first).expect("first manifest hashes"),
            manifest_digest_blake3(&second).expect("second manifest hashes")
        );
    }

    #[test]
    fn invalid_manifest_rejects_floating_source_alias_conflict_and_path_escape() {
        let mut invalid = manifest();
        invalid.source.reference = "nixpkgs".into();
        invalid.source.revision = "main".into();
        invalid.output.generation_directory = "../escape".into();
        invalid.limits.max_graph_nodes = 0;
        invalid.selectors.push(PackageSelector {
            name: "other".into(),
            attribute: "other".into(),
            system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
            aliases: alloc::vec!["hi".into()],
        });

        let failure = normalize_manifest(&invalid).expect_err("invalid manifest fails");
        let codes = failure.diagnostics.iter().map(|item| item.code.as_str()).collect::<BTreeSet<_>>();
        assert!(codes.contains("floating-source-lock"));
        assert!(codes.contains("source-reference-not-locked"));
        assert!(codes.contains("ambiguous-alias"));
        assert!(codes.contains("unsafe-relative-path"));
        assert!(codes.contains("invalid-named-limit"));
    }
}
