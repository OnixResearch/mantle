// r[impl foreign_derivation_import.nix_derivation_projection_boundary]
// r[impl foreign_derivation_import.reviewed_nix_derivation_adapter]

use std::collections::BTreeMap;

use nix_derivation::CAHash;
use nix_derivation::ContentAddressMethod;
use nix_derivation::Derivation;
use nix_derivation::Output;
use nix_derivation::StorePath;

const NIX_STORE_PREFIX: &str = "/nix/store/";
const DERIVATION_SUFFIX: &str = ".drv";
const TRADITIONAL_PREFIX: &[u8] = b"Derive(";
const VERSIONED_PREFIX: &[u8] = b"DrvWithVersion(";
const NIX_HASH_BYTES: usize = 32;
const HEX_ALPHABET_LEN: usize = 16;
const HEX_CHARS_PER_BYTE: usize = 2;
const NIBBLE_BITS: u32 = 4;
const LOW_NIBBLE_MASK: u8 = 0x0f;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AdapterLimits {
    pub(crate) derivation_bytes_max: usize,
    pub(crate) collection_items_max: usize,
    pub(crate) input_edges_max: usize,
    pub(crate) field_bytes_max: usize,
    pub(crate) structured_attrs_bytes_max: usize,
    pub(crate) dynamic_depth_max: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterError {
    pub(crate) class: &'static str,
    pub(crate) message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdapterSyntax {
    Traditional,
    Versioned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdapterValidation {
    Validated,
    RequiresInputHashes,
    Rejected(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterDerivation {
    pub(crate) logical_path: String,
    pub(crate) name: String,
    pub(crate) syntax: AdapterSyntax,
    pub(crate) outputs: BTreeMap<String, AdapterOutput>,
    pub(crate) input_derivations: BTreeMap<String, AdapterInputDerivation>,
    pub(crate) input_sources: Vec<String>,
    pub(crate) system: String,
    pub(crate) builder: String,
    pub(crate) arguments: Vec<String>,
    pub(crate) environment: BTreeMap<String, Vec<u8>>,
    pub(crate) structured_attrs_json: Option<Vec<u8>>,
    pub(crate) canonical_aterm: Vec<u8>,
    pub(crate) computed_drv_path: String,
    pub(crate) modulo_hash_without_inputs: Option<[u8; NIX_HASH_BYTES]>,
    pub(crate) validation: AdapterValidation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterOutput {
    pub(crate) path: Option<String>,
    pub(crate) kind: AdapterOutputKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdapterOutputKind {
    InputAddressed,
    Fixed(AdapterFixedOutput),
    Floating {
        method: &'static str,
        algorithm: &'static str,
    },
    Deferred,
    Impure {
        method: &'static str,
        algorithm: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterFixedOutput {
    pub(crate) method: &'static str,
    pub(crate) algorithm: &'static str,
    pub(crate) digest_hex: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterInputDerivation {
    pub(crate) outputs: Vec<String>,
    pub(crate) dynamic_requests: Vec<AdapterDynamicRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterDynamicRequest {
    pub(crate) depth: usize,
    pub(crate) output: String,
    pub(crate) requested_outputs: Vec<String>,
}

pub(crate) fn parse_derivation(
    logical_path: &str,
    bytes: &[u8],
    limits: AdapterLimits,
) -> Result<AdapterDerivation, AdapterError> {
    validate_limit_configuration(limits)?;
    validate_input_bytes(bytes, limits)?;
    let (store_path, name) = parse_logical_path(logical_path)?;
    let derivation =
        Derivation::from_aterm_bytes(bytes, &name).map_err(|error| parse_error(logical_path, bytes, error))?;
    let syntax = classify_syntax(bytes);
    validate_collections(&derivation, limits)?;
    validate_fields(&derivation, limits)?;
    validate_dynamic_inputs(&derivation, limits)?;
    project_derivation(logical_path, store_path, syntax, derivation, limits)
}

fn validate_limit_configuration(limits: AdapterLimits) -> Result<(), AdapterError> {
    let limits_are_positive = limits.derivation_bytes_max > 0
        && limits.collection_items_max > 0
        && limits.input_edges_max > 0
        && limits.field_bytes_max > 0
        && limits.structured_attrs_bytes_max > 0
        && limits.dynamic_depth_max > 0;
    if !limits_are_positive {
        return Err(adapter_error(
            "invalid-nix-derivation-adapter-limits",
            "all Nix derivation adapter limits must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_input_bytes(bytes: &[u8], limits: AdapterLimits) -> Result<(), AdapterError> {
    if bytes.is_empty() || bytes.len() > limits.derivation_bytes_max {
        return Err(adapter_error(
            "foreign-aterm-bytes-out-of-range",
            "Nix derivation bytes are outside the configured limit".to_string(),
        ));
    }
    Ok(())
}

fn parse_logical_path(logical_path: &str) -> Result<(StorePath, String), AdapterError> {
    if !logical_path.starts_with(NIX_STORE_PREFIX) {
        return Err(invalid_logical_path(logical_path));
    }
    let store_path = logical_path.parse::<StorePath>().map_err(|_| invalid_logical_path(logical_path))?;
    if !store_path.is_derivation() {
        return Err(invalid_logical_path(logical_path));
    }
    let name = store_path
        .name()
        .strip_suffix(DERIVATION_SUFFIX)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| invalid_logical_path(logical_path))?
        .to_string();
    debug_assert_eq!(store_path.to_absolute_path(), logical_path);
    debug_assert!(!name.is_empty());
    Ok((store_path, name))
}

fn classify_syntax(bytes: &[u8]) -> AdapterSyntax {
    if bytes.starts_with(VERSIONED_PREFIX) {
        AdapterSyntax::Versioned
    } else {
        debug_assert!(bytes.starts_with(TRADITIONAL_PREFIX));
        AdapterSyntax::Traditional
    }
}

fn validate_collections(derivation: &Derivation, limits: AdapterLimits) -> Result<(), AdapterError> {
    let collection_sizes = [
        derivation.outputs().len(),
        derivation.input_derivations().len(),
        derivation.input_sources().len(),
        derivation.arguments().len(),
        derivation.environment().len(),
    ];
    if collection_sizes.iter().any(|count| *count > limits.collection_items_max) {
        return Err(adapter_error(
            "nix-derivation-collection-limit-exceeded",
            "Nix derivation collection exceeds the configured limit".to_string(),
        ));
    }
    let edge_count = count_input_edges(derivation)?;
    if edge_count > limits.input_edges_max {
        return Err(adapter_error(
            "nix-derivation-input-edge-limit-exceeded",
            "Nix derivation input edges exceed the configured limit".to_string(),
        ));
    }
    debug_assert!(collection_sizes.iter().all(|count| *count <= limits.collection_items_max));
    debug_assert!(edge_count <= limits.input_edges_max);
    Ok(())
}

fn count_input_edges(derivation: &Derivation) -> Result<usize, AdapterError> {
    derivation.input_derivations().values().try_fold(0usize, |total, input| {
        input.walk().try_fold(total, |subtotal, node| {
            let requested = node
                .input()
                .outputs()
                .len()
                .checked_add(node.input().dynamic_outputs().len())
                .ok_or_else(input_edge_overflow)?;
            subtotal.checked_add(requested).ok_or_else(input_edge_overflow)
        })
    })
}

fn validate_fields(derivation: &Derivation, limits: AdapterLimits) -> Result<(), AdapterError> {
    validate_text_field("name", derivation.name(), limits.field_bytes_max)?;
    validate_text_field("system", derivation.system(), limits.field_bytes_max)?;
    validate_text_field("builder", derivation.builder(), limits.field_bytes_max)?;
    for argument in derivation.arguments() {
        validate_text_field("argument", argument, limits.field_bytes_max)?;
    }
    for (key, value) in derivation.environment() {
        validate_text_field("environment key", key, limits.field_bytes_max)?;
        let value_bytes_max = if key == "__json" {
            limits.structured_attrs_bytes_max
        } else {
            limits.field_bytes_max
        };
        validate_byte_field("environment value", value, value_bytes_max)?;
    }
    for output_name in derivation.outputs().keys() {
        validate_text_field("output name", output_name, limits.field_bytes_max)?;
    }
    Ok(())
}

fn validate_dynamic_inputs(derivation: &Derivation, limits: AdapterLimits) -> Result<(), AdapterError> {
    for (path, input) in derivation.input_derivations() {
        validate_text_field("input derivation path", &path.to_absolute_path(), limits.field_bytes_max)?;
        if input.max_depth() > limits.dynamic_depth_max {
            return Err(adapter_error(
                "nix-derivation-dynamic-depth-exceeded",
                format!("Nix derivation dynamic input exceeds depth limit: {path}"),
            ));
        }
        for node in input.walk() {
            if let Some(output) = node.dynamic_output() {
                validate_text_field("dynamic output name", output, limits.field_bytes_max)?;
            }
            for output in node.input().outputs() {
                validate_text_field("requested output name", output, limits.field_bytes_max)?;
            }
        }
    }
    Ok(())
}

fn project_derivation(
    logical_path: &str,
    store_path: StorePath,
    syntax: AdapterSyntax,
    derivation: Derivation,
    limits: AdapterLimits,
) -> Result<AdapterDerivation, AdapterError> {
    let validation = validation_status(&derivation);
    let outputs = project_outputs(&derivation)?;
    let input_derivations = project_inputs(&derivation)?;
    let input_sources = derivation.input_sources().iter().map(StorePath::to_absolute_path).collect::<Vec<_>>();
    let structured_attrs_json = derivation.structured_attrs().map(|attrs| attrs.raw_json().to_vec());
    if structured_attrs_json.as_ref().is_some_and(|json| json.len() > limits.structured_attrs_bytes_max) {
        return Err(adapter_error(
            "nix-derivation-structured-attrs-limit-exceeded",
            "Nix structured attributes exceed the configured limit".to_string(),
        ));
    }
    let canonical_aterm = derivation.to_aterm_bytes();
    let computed_drv_path = derivation
        .drv_path()
        .map_err(|error| adapter_error("invalid-nix-derivation-identity", error.to_string()))?
        .to_absolute_path();
    let modulo_hash_without_inputs = modulo_hash_without_inputs(&derivation)?;
    let projected = AdapterDerivation {
        logical_path: logical_path.to_string(),
        name: derivation.name().to_string(),
        syntax,
        outputs,
        input_derivations,
        input_sources,
        system: derivation.system().to_string(),
        builder: derivation.builder().to_string(),
        arguments: derivation.arguments().to_vec(),
        environment: derivation.environment().clone(),
        structured_attrs_json,
        canonical_aterm,
        computed_drv_path,
        modulo_hash_without_inputs,
        validation,
    };
    debug_assert_eq!(store_path.to_absolute_path(), projected.logical_path);
    debug_assert!(!projected.name.is_empty());
    Ok(projected)
}

fn validation_status(derivation: &Derivation) -> AdapterValidation {
    match derivation.validate() {
        Ok(()) => AdapterValidation::Validated,
        Err(error) if error.to_string().contains("requires input derivation modulo hashes") => {
            AdapterValidation::RequiresInputHashes
        }
        Err(error) => AdapterValidation::Rejected(error.to_string()),
    }
}

fn project_outputs(derivation: &Derivation) -> Result<BTreeMap<String, AdapterOutput>, AdapterError> {
    let output_count = derivation.outputs().len();
    let mut outputs = BTreeMap::new();
    for (name, output) in derivation.outputs() {
        let path = output
            .path(derivation.name(), name)
            .map_err(|error| adapter_error("invalid-nix-derivation-output", error.to_string()))?
            .map(|path| path.to_absolute_path());
        let kind = project_output_kind(output);
        let replaced = outputs.insert(name.clone(), AdapterOutput { path, kind });
        debug_assert!(replaced.is_none());
        debug_assert!(outputs.len() <= output_count);
    }
    debug_assert_eq!(outputs.len(), output_count);
    Ok(outputs)
}

fn project_output_kind(output: &Output) -> AdapterOutputKind {
    match output {
        Output::InputAddressed { .. } => AdapterOutputKind::InputAddressed,
        Output::Fixed { ca } => AdapterOutputKind::Fixed(project_fixed_output(ca)),
        Output::Floating { method, hash_algorithm } => AdapterOutputKind::Floating {
            method: content_method(*method),
            algorithm: hash_algorithm.as_str(),
        },
        Output::Deferred => AdapterOutputKind::Deferred,
        Output::Impure { method, hash_algorithm } => AdapterOutputKind::Impure {
            method: content_method(*method),
            algorithm: hash_algorithm.as_str(),
        },
    }
}

fn project_fixed_output(ca: &CAHash) -> AdapterFixedOutput {
    let hash = ca.hash();
    AdapterFixedOutput {
        method: ca.method(),
        algorithm: hash.algo(),
        digest_hex: encode_hex(hash.digest_as_bytes()),
    }
}

fn project_inputs(derivation: &Derivation) -> Result<BTreeMap<String, AdapterInputDerivation>, AdapterError> {
    derivation
        .input_derivations()
        .iter()
        .map(|(path, input)| {
            let outputs = input.outputs().iter().cloned().collect();
            let dynamic_requests = input
                .walk()
                .skip(1)
                .map(|node| {
                    let output = node.dynamic_output().ok_or_else(|| {
                        adapter_error(
                            "invalid-nix-dynamic-input",
                            "dynamic input node is missing its parent output".to_string(),
                        )
                    })?;
                    Ok(AdapterDynamicRequest {
                        depth: node.depth(),
                        output: output.to_string(),
                        requested_outputs: node.input().outputs().iter().cloned().collect(),
                    })
                })
                .collect::<Result<Vec<_>, AdapterError>>()?;
            Ok((path.to_absolute_path(), AdapterInputDerivation {
                outputs,
                dynamic_requests,
            }))
        })
        .collect()
}

fn modulo_hash_without_inputs(derivation: &Derivation) -> Result<Option<[u8; NIX_HASH_BYTES]>, AdapterError> {
    if !derivation.input_derivations().is_empty() {
        return Ok(None);
    }
    derivation
        .hash_derivation_modulo(false, |_| unreachable!("a derivation without inputs cannot request an input hash"))
        .map(Some)
        .map_err(|error| adapter_error("invalid-nix-derivation-hash", error.to_string()))
}

fn content_method(method: ContentAddressMethod) -> &'static str {
    match method {
        ContentAddressMethod::Flat => "flat",
        ContentAddressMethod::Nar => "nar",
        ContentAddressMethod::Text => "text",
        ContentAddressMethod::Git => "git",
    }
}

fn validate_text_field(field: &str, value: &str, bytes_max: usize) -> Result<(), AdapterError> {
    validate_byte_field(field, value.as_bytes(), bytes_max)
}

fn validate_byte_field(field: &str, value: &[u8], bytes_max: usize) -> Result<(), AdapterError> {
    if value.len() > bytes_max {
        return Err(adapter_error(
            "nix-derivation-field-limit-exceeded",
            format!("Nix derivation {field} exceeds the configured byte limit"),
        ));
    }
    Ok(())
}

fn parse_error(logical_path: &str, bytes: &[u8], error: nix_derivation::Error) -> AdapterError {
    let class = if std::str::from_utf8(bytes).is_err() || matches!(error, nix_derivation::Error::InvalidUtf8 { .. }) {
        "non-utf8-foreign-aterm"
    } else {
        "malformed-foreign-aterm"
    };
    adapter_error(class, format!("{logical_path}: {error}"))
}

fn invalid_logical_path(logical_path: &str) -> AdapterError {
    adapter_error("invalid-nix-derivation-path", format!("Nix derivation logical path is invalid: {logical_path}"))
}

fn input_edge_overflow() -> AdapterError {
    adapter_error("nix-derivation-input-edge-limit-exceeded", "Nix derivation input edge count overflowed".to_string())
}

fn adapter_error(class: &'static str, message: String) -> AdapterError {
    AdapterError { class, message }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; HEX_ALPHABET_LEN] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(HEX_CHARS_PER_BYTE));
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> NIBBLE_BITS)] as char);
        encoded.push(HEX[usize::from(byte & LOW_NIBBLE_MASK)] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_PATH: &str = "/nix/store/44444444444444444444444444444444-hello-source.drv";
    const VARIANT_PATH: &str = "/nix/store/00000000000000000000000000000000-variant.drv";
    const DYNAMIC_INPUT_PATH: &str = "/nix/store/00000000000000000000000000000000-input.drv";
    const EXPECTED_DRV_PATH: &str = "/nix/store/lim8wd2bgsm46w9mqpklv24ihrzmh537-hello-source.drv";
    const EXPECTED_MODULO_HASH_HEX: &str = "81cc8bbb5ac05ea9cbd55c0e7517b65b2a182f23334ff939d3f415cce8951e74";
    const STRUCTURED_NAME: &str = "bench-fixed-output";
    const STRUCTURED_DRV_PATH: &str = "/nix/store/00000000000000000000000000000000-bench-fixed-output.drv";
    const STRUCTURED_OUTPUT_PATH: &str = "/nix/store/lhxfg7yb424j1mm6g6aac6cnvdl2kpwa-bench-fixed-output";
    const ZERO_SHA256_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const STRUCTURED_JSON: &str = "{\"name\":\"bench-fixed-output\",\"system\":\"x86_64-linux\"}";
    const INVALID_UTF8_BYTE: u8 = 0xff;
    const KIBIBYTE_BYTES: usize = 1_024;
    const FIELD_KIBIBYTES_MAX: usize = 32;
    const STRUCTURED_KIBIBYTES_MAX: usize = 1_024;
    const DERIVATION_MEBIBYTES_MAX: usize = 16;
    const COLLECTION_ITEMS_MAX: usize = 256;
    const INPUT_EDGES_MAX: usize = 256;
    const DYNAMIC_DEPTH_MAX: usize = 256;
    const FIELD_BYTES_MAX: usize = FIELD_KIBIBYTES_MAX * KIBIBYTE_BYTES;
    const STRUCTURED_BYTES_MAX: usize = STRUCTURED_KIBIBYTES_MAX * KIBIBYTE_BYTES;
    const DERIVATION_BYTES_MAX: usize = DERIVATION_MEBIBYTES_MAX * KIBIBYTE_BYTES * KIBIBYTE_BYTES;

    fn limits() -> AdapterLimits {
        AdapterLimits {
            derivation_bytes_max: DERIVATION_BYTES_MAX,
            collection_items_max: COLLECTION_ITEMS_MAX,
            input_edges_max: INPUT_EDGES_MAX,
            field_bytes_max: FIELD_BYTES_MAX,
            structured_attrs_bytes_max: STRUCTURED_BYTES_MAX,
            dynamic_depth_max: DYNAMIC_DEPTH_MAX,
        }
    }

    #[test]
    fn parses_traditional_derivation_with_nix_identity_facts() {
        let bytes = include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv");
        let parsed = parse_derivation(FIXTURE_PATH, bytes, limits()).unwrap();

        assert_eq!(parsed.name, "hello-source");
        assert_eq!(parsed.syntax, AdapterSyntax::Traditional);
        assert_eq!(parsed.canonical_aterm, bytes);
        assert_eq!(parsed.computed_drv_path, EXPECTED_DRV_PATH);
        assert_eq!(
            encode_hex(&parsed.modulo_hash_without_inputs.expect("source has no inputs")),
            EXPECTED_MODULO_HASH_HEX,
        );
        assert!(parsed.computed_drv_path.starts_with(NIX_STORE_PREFIX));
        assert!(parsed.modulo_hash_without_inputs.is_some());
        assert_eq!(parsed.outputs.len(), 1);
        assert!(parsed.input_derivations.is_empty());
    }

    #[test]
    fn rejects_invalid_path_and_byte_bounds() {
        let bytes = include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv");
        let invalid_path = parse_derivation("/mantle/store/not-nix.drv", bytes, limits()).unwrap_err();
        assert_eq!(invalid_path.class, "invalid-nix-derivation-path");

        let mut bounded = limits();
        bounded.derivation_bytes_max = bytes.len().saturating_sub(1);
        let oversized = parse_derivation(FIXTURE_PATH, bytes, bounded).unwrap_err();
        assert_eq!(oversized.class, "foreign-aterm-bytes-out-of-range");
    }

    #[test]
    fn rejects_zero_limits_before_parsing() {
        let bytes = include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv");
        let mut invalid = limits();
        invalid.input_edges_max = 0;
        let error = parse_derivation(FIXTURE_PATH, bytes, invalid).unwrap_err();
        assert_eq!(error.class, "invalid-nix-derivation-adapter-limits");
        assert!(!error.message.is_empty());
    }

    #[test]
    fn classifies_every_non_concrete_output_variant() {
        let floating = parse_derivation(
            VARIANT_PATH,
            b"Derive([(\"out\",\"\",\"r:sha256\",\"\")],[],[],\"x86_64-linux\",\"/bin/sh\",[],[])",
            limits(),
        )
        .unwrap();
        let deferred = parse_derivation(
            VARIANT_PATH,
            b"Derive([(\"out\",\"\",\"\",\"\")],[],[],\"x86_64-linux\",\"/bin/sh\",[],[])",
            limits(),
        )
        .unwrap();
        let impure = parse_derivation(
            VARIANT_PATH,
            b"Derive([(\"out\",\"\",\"text:sha256\",\"impure\")],[],[],\"x86_64-linux\",\"/bin/sh\",[],[])",
            limits(),
        )
        .unwrap();

        assert!(matches!(floating.outputs["out"].kind, AdapterOutputKind::Floating { .. }));
        assert!(matches!(deferred.outputs["out"].kind, AdapterOutputKind::Deferred));
        assert!(matches!(impure.outputs["out"].kind, AdapterOutputKind::Impure { .. }));
    }

    #[test]
    fn preserves_structured_attributes_outside_the_environment_map() {
        let bytes = format!(
            "Derive([(\"out\",\"{STRUCTURED_OUTPUT_PATH}\",\"r:sha256\",\"{ZERO_SHA256_HEX}\")],[],[],\"x86_64-linux\",\"/bin/sh\",[],[(\"__json\",\"{{\\\"name\\\":\\\"{STRUCTURED_NAME}\\\",\\\"system\\\":\\\"x86_64-linux\\\"}}\"),(\"name\",\"{STRUCTURED_NAME}\"),(\"out\",\"{STRUCTURED_OUTPUT_PATH}\")])"
        );
        let parsed = parse_derivation(STRUCTURED_DRV_PATH, bytes.as_bytes(), limits()).unwrap();

        assert_eq!(parsed.structured_attrs_json.as_deref(), Some(STRUCTURED_JSON.as_bytes()));
        assert!(!parsed.environment.contains_key("__json"));
        assert_eq!(parsed.validation, AdapterValidation::Validated);
    }

    #[test]
    fn preserves_dynamic_input_facts_for_explicit_rejection() {
        let bytes = format!(
            "DrvWithVersion(\"xp-dyn-drv\",[],[(\"{DYNAMIC_INPUT_PATH}\",([\"out\"],[(\"generated\",[\"out\"])]))],[],\"x86_64-linux\",\"/bin/sh\",[],[])"
        );
        let parsed = parse_derivation(VARIANT_PATH, bytes.as_bytes(), limits()).unwrap();
        let input = &parsed.input_derivations[DYNAMIC_INPUT_PATH];

        assert_eq!(parsed.syntax, AdapterSyntax::Versioned);
        assert_eq!(input.outputs, ["out"]);
        assert_eq!(input.dynamic_requests.len(), 1);
        assert_eq!(input.dynamic_requests[0].output, "generated");
    }

    #[test]
    fn preserves_arbitrary_environment_bytes_until_projection() {
        let mut bytes =
            b"Derive([(\"out\",\"\",\"r:sha256\",\"\")],[],[],\"x86_64-linux\",\"/bin/sh\",[],[(\"raw\",\"".to_vec();
        bytes.push(INVALID_UTF8_BYTE);
        bytes.extend_from_slice(b"\")])");
        let parsed = parse_derivation(VARIANT_PATH, &bytes, limits()).unwrap();

        assert_eq!(parsed.environment["raw"], [INVALID_UTF8_BYTE]);
        assert_eq!(parsed.canonical_aterm, bytes);
    }
}
