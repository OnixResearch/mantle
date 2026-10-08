use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;

use crate::receipt::ExpectedOutputFacts;
use crate::receipt::OrderedOutputReceipt;
use crate::receipt::OutputReceiptFields;
use crate::transfer::TransferMode;

const BLAKE3_HEX_LENGTH_CHARS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputBlocker {
    NarDigestInvalid,
    NarPathinfoMissing,
    NarSummaryIncomplete,
    TransferBuilderKeyEmpty,
    TransferModeLabelMismatch,
    TransferDeltaHasFallbackReason,
    TransferFullReusedBytesNonzero,
    TransferStreamingHasFallbackReason,
    MetadataCountMismatch,
    MetadataDigestMismatch,
    MetadataNameDuplicate,
    MetadataIdentityMismatch,
    MetadataPathinfoKeyMismatch,
    MetadataDigestInvalid,
    RequestIdentityMismatch,
    StorePrefixMismatch,
    ResultDigestInvalid,
    TransferBuilderKeyMismatch,
    ExecutionRequestMismatch,
    ExecutionPlanDigestMismatch,
    ExecutionOutputDigestInvalid,
    ExecutionOutputDigestMismatch,
    ExecutionOutputSizeOverflow,
    ExecutionOutputSizeMismatch,
    ExecutionOutputCountMismatch,
    ExecutionOutputNameDuplicate,
    ExecutionOutputIdentityMismatch,
    ExecutionOutputMetadataDigestInvalid,
    ExecutionOutputNarPathinfoMissing,
}

impl OutputBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NarDigestInvalid => "remote-output-nar-digest-invalid",
            Self::NarPathinfoMissing => "remote-output-nar-pathinfo-missing",
            Self::NarSummaryIncomplete => "remote-output-nar-summary-incomplete",
            Self::TransferBuilderKeyEmpty => "remote-transfer-builder-key-empty",
            Self::TransferModeLabelMismatch => "remote-transfer-mode-label-mismatch",
            Self::TransferDeltaHasFallbackReason => "remote-transfer-delta-has-fallback-reason",
            Self::TransferFullReusedBytesNonzero => "remote-transfer-full-reused-bytes-nonzero",
            Self::TransferStreamingHasFallbackReason => "remote-transfer-streaming-has-fallback-reason",
            Self::MetadataCountMismatch => "remote-output-metadata-count-mismatch",
            Self::MetadataDigestMismatch => "remote-output-metadata-digest-mismatch",
            Self::MetadataNameDuplicate => "remote-output-metadata-name-duplicate",
            Self::MetadataIdentityMismatch => "remote-output-metadata-identity-mismatch",
            Self::MetadataPathinfoKeyMismatch => "remote-output-pathinfo-key-mismatch",
            Self::MetadataDigestInvalid => "remote-output-metadata-digest-invalid",
            Self::RequestIdentityMismatch => "remote-output-request-id-mismatch",
            Self::StorePrefixMismatch => "remote-output-store-prefix-mismatch",
            Self::ResultDigestInvalid => "remote-output-digest-invalid",
            Self::TransferBuilderKeyMismatch => "remote-transfer-builder-key-mismatch",
            Self::ExecutionRequestMismatch => "remote-execution-request-id-mismatch",
            Self::ExecutionPlanDigestMismatch => "remote-execution-plan-digest-mismatch",
            Self::ExecutionOutputDigestInvalid => "remote-execution-output-digest-invalid",
            Self::ExecutionOutputDigestMismatch => "remote-execution-output-digest-mismatch",
            Self::ExecutionOutputSizeOverflow => "remote-execution-output-size-overflow",
            Self::ExecutionOutputSizeMismatch => "remote-execution-output-size-mismatch",
            Self::ExecutionOutputCountMismatch => "remote-execution-output-count-mismatch",
            Self::ExecutionOutputNameDuplicate => "remote-execution-output-name-duplicate",
            Self::ExecutionOutputIdentityMismatch => "remote-execution-output-identity-mismatch",
            Self::ExecutionOutputMetadataDigestInvalid => "remote-execution-output-metadata-digest-invalid",
            Self::ExecutionOutputNarPathinfoMissing => "remote-execution-output-nar-pathinfo-missing",
        }
    }
}

/// Compare worker result identities before inspecting host-owned NAR and PathInfo bytes.
/// The adapter supplies its computed receipt only after reading its concrete outputs.
pub fn validate_execution_outcome_scope(
    expected_request_id: &str,
    actual_request_id: &str,
    expected_plan_digest_blake3: &str,
    actual_plan_digest_blake3: &str,
    reported_output_digest_blake3: &str,
) -> Result<(), OutputBlocker> {
    if actual_request_id != expected_request_id {
        return Err(OutputBlocker::ExecutionRequestMismatch);
    }
    if actual_plan_digest_blake3 != expected_plan_digest_blake3 {
        return Err(OutputBlocker::ExecutionPlanDigestMismatch);
    }
    if !is_blake3_hex_digest(reported_output_digest_blake3) {
        return Err(OutputBlocker::ExecutionOutputDigestInvalid);
    }
    Ok(())
}

pub fn validate_execution_output_digest(reported: &str, observed: &str) -> Result<(), OutputBlocker> {
    if reported != observed {
        return Err(OutputBlocker::ExecutionOutputDigestMismatch);
    }
    Ok(())
}

pub fn validate_execution_output_size(reported: u64, observed: u64) -> Result<(), OutputBlocker> {
    if reported != observed {
        return Err(OutputBlocker::ExecutionOutputSizeMismatch);
    }
    Ok(())
}

pub fn sum_execution_output_sizes(mut sizes: impl Iterator<Item = u64>) -> Result<u64, OutputBlocker> {
    sizes.try_fold(0_u64, |sum, size| {
        sum.checked_add(size).ok_or(OutputBlocker::ExecutionOutputSizeOverflow)
    })
}

#[derive(Debug, Clone, Copy)]
pub struct ExecutionOutputFacts<'a> {
    pub name: &'a str,
    pub logical_path: &'a str,
    pub content_digest_blake3: &'a str,
    pub artifact_attestation_digest_blake3: &'a str,
    pub nar_payload_present: bool,
    pub pathinfo_present: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionOutputValidationError<E> {
    Decision(OutputBlocker),
    Inspect(E),
}

/// Preserve the accepted per-output error order: duplicate/name/path/digest/
/// NAR association, then actual NAR inspection, before looking at the next output.
pub fn validate_execution_output_facts<'a, T, E>(
    expected_outputs: impl ExactSizeIterator<Item = ExpectedOutputFacts<'a>>,
    outputs: impl ExactSizeIterator<Item = (ExecutionOutputFacts<'a>, T)>,
    store_prefix: &str,
    mut inspect_nar: impl FnMut(T) -> Result<(), E>,
) -> Result<(), ExecutionOutputValidationError<E>> {
    if outputs.len() != expected_outputs.len() {
        return Err(ExecutionOutputValidationError::Decision(OutputBlocker::ExecutionOutputCountMismatch));
    }
    let expected = expected_outputs
        .map(|output| (output.name, output.logical_path))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for (output, host_output) in outputs {
        if !seen.insert(output.name) {
            return Err(ExecutionOutputValidationError::Decision(OutputBlocker::ExecutionOutputNameDuplicate));
        }
        match expected.get(output.name) {
            Some(Some(path)) if *path == output.logical_path => {}
            Some(None) if !output.logical_path.is_empty() && output.logical_path.starts_with(store_prefix) => {}
            _ => return Err(ExecutionOutputValidationError::Decision(OutputBlocker::ExecutionOutputIdentityMismatch)),
        }
        if !is_blake3_hex_digest(output.content_digest_blake3)
            || !is_blake3_hex_digest(output.artifact_attestation_digest_blake3)
        {
            return Err(ExecutionOutputValidationError::Decision(
                OutputBlocker::ExecutionOutputMetadataDigestInvalid,
            ));
        }
        if output.nar_payload_present && !output.pathinfo_present {
            return Err(ExecutionOutputValidationError::Decision(OutputBlocker::ExecutionOutputNarPathinfoMissing));
        }
        inspect_nar(host_output).map_err(ExecutionOutputValidationError::Inspect)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputTransferArtifactKind {
    PathInfoJson,
    Nar,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputTransferArtifactFacts<'a> {
    pub request_id: &'a str,
    pub output_name: &'a str,
    pub logical_path: &'a str,
    pub kind: OutputTransferArtifactKind,
    pub digest_blake3: &'a str,
    pub size_bytes: u64,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub struct ExpectedOutputTransferArtifactFacts<'a> {
    pub output_name: &'a str,
    pub logical_path: &'a str,
    pub kind: OutputTransferArtifactKind,
    pub digest_blake3: &'a str,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTransferArtifactBlocker {
    RequestIdEmpty,
    ArtifactCountExceeded,
    RequestIdMismatch,
    ArtifactIdentityEmpty,
    DigestInvalid,
    PayloadSizeOverflow,
    SizeMismatch,
    DigestMismatch,
    Unexpected,
    OutputMismatch,
    KindMismatch,
    ExpectedDigestMismatch,
    ExpectedSizeMismatch,
    Duplicate,
    TotalBytesOverflow,
    TotalBytesExceeded,
    Missing,
}

impl OutputTransferArtifactBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "remote-output-transfer-request-id-empty",
            Self::ArtifactCountExceeded => "remote-output-transfer-artifact-count-exceeded",
            Self::RequestIdMismatch => "remote-output-transfer-artifact-request-id-mismatch",
            Self::ArtifactIdentityEmpty => "remote-output-transfer-artifact-identity-empty",
            Self::DigestInvalid => "remote-output-transfer-artifact-digest-invalid",
            Self::PayloadSizeOverflow => "remote-output-transfer-payload-size-overflow",
            Self::SizeMismatch => "remote-output-transfer-artifact-size-mismatch",
            Self::DigestMismatch => "remote-output-transfer-artifact-digest-mismatch",
            Self::Unexpected => "remote-output-transfer-artifact-unexpected",
            Self::OutputMismatch => "remote-output-transfer-artifact-output-mismatch",
            Self::KindMismatch => "remote-output-transfer-artifact-kind-mismatch",
            Self::ExpectedDigestMismatch => "remote-output-transfer-artifact-expected-digest-mismatch",
            Self::ExpectedSizeMismatch => "remote-output-transfer-artifact-expected-size-mismatch",
            Self::Duplicate => "remote-output-transfer-artifact-duplicate",
            Self::TotalBytesOverflow => "remote-output-transfer-total-bytes-overflow",
            Self::TotalBytesExceeded => "remote-output-transfer-total-bytes-exceeded",
            Self::Missing => "remote-output-transfer-artifact-missing",
        }
    }
}

/// A compatibility artifact is accepted only after its payload and its
/// builder-declared identity match. One result record may contain multiple
/// artifacts, but it cannot silently omit, duplicate, or overrun a bound.
pub struct OutputTransferArtifactSequence<'a> {
    request_id: &'a str,
    seen: BTreeSet<(alloc::string::String, OutputTransferArtifactKind)>,
    transferred_bytes: u64,
    total_bytes_max: u64,
}

impl<'a> OutputTransferArtifactSequence<'a> {
    pub fn new(
        request_id: &'a str,
        artifact_count: usize,
        artifact_count_max: usize,
        total_bytes_max: u64,
    ) -> Result<Self, OutputTransferArtifactBlocker> {
        if request_id.is_empty() {
            return Err(OutputTransferArtifactBlocker::RequestIdEmpty);
        }
        if artifact_count > artifact_count_max {
            return Err(OutputTransferArtifactBlocker::ArtifactCountExceeded);
        }
        Ok(Self {
            request_id,
            seen: BTreeSet::new(),
            transferred_bytes: 0,
            total_bytes_max,
        })
    }

    pub fn observe(
        &mut self,
        artifact: OutputTransferArtifactFacts<'_>,
        expected: Option<ExpectedOutputTransferArtifactFacts<'_>>,
    ) -> Result<(), OutputTransferArtifactBlocker> {
        if artifact.request_id != self.request_id {
            return Err(OutputTransferArtifactBlocker::RequestIdMismatch);
        }
        if artifact.output_name.is_empty() || artifact.logical_path.is_empty() {
            return Err(OutputTransferArtifactBlocker::ArtifactIdentityEmpty);
        }
        if !is_blake3_hex_digest(artifact.digest_blake3) {
            return Err(OutputTransferArtifactBlocker::DigestInvalid);
        }
        let payload_size_bytes =
            u64::try_from(artifact.payload.len()).map_err(|_| OutputTransferArtifactBlocker::PayloadSizeOverflow)?;
        if payload_size_bytes != artifact.size_bytes {
            return Err(OutputTransferArtifactBlocker::SizeMismatch);
        }
        if blake3::hash(artifact.payload).to_hex().as_str() != artifact.digest_blake3 {
            return Err(OutputTransferArtifactBlocker::DigestMismatch);
        }
        let Some(expected) = expected else {
            return Err(OutputTransferArtifactBlocker::Unexpected);
        };
        if expected.output_name != artifact.output_name || expected.logical_path != artifact.logical_path {
            return Err(OutputTransferArtifactBlocker::OutputMismatch);
        }
        if expected.kind != artifact.kind {
            return Err(OutputTransferArtifactBlocker::KindMismatch);
        }
        if expected.digest_blake3 != artifact.digest_blake3 {
            return Err(OutputTransferArtifactBlocker::ExpectedDigestMismatch);
        }
        if expected.size_bytes != artifact.size_bytes {
            return Err(OutputTransferArtifactBlocker::ExpectedSizeMismatch);
        }
        if !self.seen.insert((alloc::string::String::from(artifact.output_name), artifact.kind)) {
            return Err(OutputTransferArtifactBlocker::Duplicate);
        }
        self.transferred_bytes = self
            .transferred_bytes
            .checked_add(artifact.size_bytes)
            .ok_or(OutputTransferArtifactBlocker::TotalBytesOverflow)?;
        if self.transferred_bytes > self.total_bytes_max {
            return Err(OutputTransferArtifactBlocker::TotalBytesExceeded);
        }
        Ok(())
    }

    pub fn finish(self, expected_artifact_count: usize) -> Result<(), OutputTransferArtifactBlocker> {
        if self.seen.len() != expected_artifact_count {
            return Err(OutputTransferArtifactBlocker::Missing);
        }
        Ok(())
    }
}

pub struct NarSummaryFacts<'a> {
    pub payload_digest_blake3: Option<&'a str>,
    pub payload_size_bytes: Option<u64>,
    /// Presence is structural; actual signature and content admission belong
    /// to the store adapter and cannot be inferred from this boolean.
    pub pathinfo_present: bool,
}

pub fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// Admit the summary's *shape* only; no output is trusted until its PathInfo,
/// signatures, store prefix, payload digest, and content have been checked.
// r[impl remote_builds.hexagonal_core]
pub fn admit_nar_summary(facts: NarSummaryFacts<'_>) -> Result<(), OutputBlocker> {
    match (facts.payload_digest_blake3, facts.payload_size_bytes) {
        (Some(digest), Some(_)) => {
            if !is_blake3_hex_digest(digest) {
                return Err(OutputBlocker::NarDigestInvalid);
            }
            if !facts.pathinfo_present {
                return Err(OutputBlocker::NarPathinfoMissing);
            }
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(OutputBlocker::NarSummaryIncomplete),
    }
}
#[derive(Debug, Clone, Copy)]
pub struct TransferReportFacts<'a> {
    pub verified_builder_key: &'a str,
    pub mode: TransferMode,
    pub mode_label: &'a str,
    pub reused_bytes: u64,
    pub fallback_reason: Option<&'a str>,
}

/// Validate evidence report consistency without converting the report itself
/// into evidence that output was imported or its signer authenticated.
// r[impl remote_builds.hexagonal_core]
pub fn admit_transfer_report(report: TransferReportFacts<'_>) -> Result<(), OutputBlocker> {
    if report.verified_builder_key.is_empty() {
        return Err(OutputBlocker::TransferBuilderKeyEmpty);
    }
    let expected_mode_label = match report.mode {
        TransferMode::Delta => "delta",
        TransferMode::Full => "full",
        TransferMode::Streaming => "streaming",
    };
    if report.mode_label != expected_mode_label {
        return Err(OutputBlocker::TransferModeLabelMismatch);
    }
    match report.mode {
        TransferMode::Delta if report.fallback_reason.is_some() => Err(OutputBlocker::TransferDeltaHasFallbackReason),
        TransferMode::Full if report.reused_bytes != 0 => Err(OutputBlocker::TransferFullReusedBytesNonzero),
        TransferMode::Streaming if report.fallback_reason.is_some() => {
            Err(OutputBlocker::TransferStreamingHasFallbackReason)
        }
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OutputScopeFacts<'a> {
    pub expected_request_id: &'a str,
    pub request_id: &'a str,
    pub expected_store_prefix: &'a str,
    pub store_prefix: &'a str,
    pub output_digest_blake3: &'a str,
}

/// Scope and digest shape, before metadata, artifacts, or transfer facts.
pub fn validate_output_scope(facts: OutputScopeFacts<'_>) -> Result<(), OutputBlocker> {
    if facts.request_id != facts.expected_request_id {
        return Err(OutputBlocker::RequestIdentityMismatch);
    }
    if facts.store_prefix != facts.expected_store_prefix {
        return Err(OutputBlocker::StorePrefixMismatch);
    }
    if !is_blake3_hex_digest(facts.output_digest_blake3) {
        return Err(OutputBlocker::ResultDigestInvalid);
    }
    Ok(())
}

/// Binding a transfer report to the builder does not authenticate its output.
pub fn validate_transfer_builder_key(report_key: &str, result_key: &str) -> Result<(), OutputBlocker> {
    if report_key != result_key {
        return Err(OutputBlocker::TransferBuilderKeyMismatch);
    }
    Ok(())
}

/// Builder-produced metadata, not proof that a PathInfo, signature, or NAR
/// has been authenticated. The std adapter retains those authorities.
#[derive(Debug, Clone, Copy)]
pub struct ProducedOutputFacts<'a> {
    pub receipt: OutputReceiptFields<'a>,
    pub path_info_signing_key_id: &'a str,
    pub pathinfo_present: bool,
}

/// Check the ordered metadata receipt before individual identities, preserving
/// the existing first-error precedence. Nothing here admits content to a store.
// r[impl remote_builds.hexagonal_core]
pub fn validate_produced_output_metadata<'a, E, O>(
    expected_outputs: E,
    outputs: O,
    reported_digest_blake3: &str,
    builder_signing_key_id: &str,
    store_prefix: &str,
) -> Result<(), OutputBlocker>
where
    E: ExactSizeIterator<Item = ExpectedOutputFacts<'a>>,
    O: Clone + ExactSizeIterator<Item = ProducedOutputFacts<'a>>,
{
    if outputs.len() != expected_outputs.len() {
        return Err(OutputBlocker::MetadataCountMismatch);
    }
    let mut receipt = OrderedOutputReceipt::new();
    for output in outputs.clone() {
        receipt.append(output.receipt);
    }
    if blake3::Hash::from_bytes(receipt.finish()).to_hex().as_str() != reported_digest_blake3 {
        return Err(OutputBlocker::MetadataDigestMismatch);
    }
    let mut expected = expected_outputs
        .map(|output| (output.name, (output.logical_path, false)))
        .collect::<BTreeMap<_, _>>();
    for output in outputs {
        let fields = output.receipt;
        let Some((expected_path, seen)) = expected.get_mut(fields.name) else {
            return Err(OutputBlocker::MetadataIdentityMismatch);
        };
        if *seen {
            return Err(OutputBlocker::MetadataNameDuplicate);
        }
        *seen = true;
        match expected_path {
            Some(path) if *path == fields.logical_path => {}
            None if !fields.logical_path.is_empty() && fields.logical_path.starts_with(store_prefix) => {}
            _ => return Err(OutputBlocker::MetadataIdentityMismatch),
        }
        if output.path_info_signing_key_id != builder_signing_key_id {
            return Err(OutputBlocker::MetadataPathinfoKeyMismatch);
        }
        if !is_blake3_hex_digest(fields.content_digest_blake3)
            || !is_blake3_hex_digest(fields.artifact_attestation_digest_blake3)
        {
            return Err(OutputBlocker::MetadataDigestInvalid);
        }
        admit_nar_summary(NarSummaryFacts {
            payload_digest_blake3: fields.nar_payload_digest_blake3,
            payload_size_bytes: fields.nar_payload_size_bytes,
            pathinfo_present: output.pathinfo_present,
        })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTrustBlocker {
    StorePrefixMismatch,
    KeyMaterialMissing,
    SameNameDifferentMaterial,
    UntrustedKey,
}

impl OutputTrustBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StorePrefixMismatch => "store-prefix-mismatch",
            Self::KeyMaterialMissing => "output-key-material-missing",
            Self::SameNameDifferentMaterial => "same-name-different-output-key",
            Self::UntrustedKey => "untrusted-output-key",
        }
    }
}

/// A policy-key match only: never signature verification or content trust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchedOutputKey {
    pub key_material_digest_blake3: Option<[u8; 32]>,
}

// r[impl remote_builds.hexagonal_core]
pub fn match_output_key<'a>(
    signing_key_id: &str,
    trusted_key_ids: impl IntoIterator<Item = &'a str>,
    store_prefix_matches: bool,
) -> Result<MatchedOutputKey, OutputTrustBlocker> {
    if !store_prefix_matches {
        return Err(OutputTrustBlocker::StorePrefixMismatch);
    }
    let (signer_name, signer_material) = split_output_key(signing_key_id);
    let signer_digest = signer_material.map(|material| *blake3::hash(material.as_bytes()).as_bytes());
    let mut material_required = false;
    let mut different_material = false;
    for trusted_key in trusted_key_ids {
        let (trusted_name, trusted_material) = split_output_key(trusted_key);
        if trusted_name != signer_name {
            continue;
        }
        match (signer_digest, trusted_material) {
            (Some(signer), Some(trusted)) if signer == *blake3::hash(trusted.as_bytes()).as_bytes() => {
                return Ok(MatchedOutputKey {
                    key_material_digest_blake3: signer_digest,
                });
            }
            (Some(_), Some(_)) => different_material = true,
            (None, Some(_)) => material_required = true,
            (_, None) => {
                return Ok(MatchedOutputKey {
                    key_material_digest_blake3: signer_digest,
                });
            }
        }
    }
    if material_required {
        return Err(OutputTrustBlocker::KeyMaterialMissing);
    }
    if different_material {
        return Err(OutputTrustBlocker::SameNameDifferentMaterial);
    }
    Err(OutputTrustBlocker::UntrustedKey)
}

fn split_output_key(value: &str) -> (&str, Option<&str>) {
    match value.split_once(':') {
        Some((name, material)) if !name.is_empty() && !material.is_empty() => (name, Some(material)),
        _ => (value, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_artifact_sequence_requires_payload_identity_and_every_expected_artifact() {
        let payload = b"payload";
        let digest = blake3::hash(payload).to_hex();
        let artifact = OutputTransferArtifactFacts {
            request_id: "request",
            output_name: "out",
            logical_path: "/store/out",
            kind: OutputTransferArtifactKind::Nar,
            digest_blake3: digest.as_str(),
            size_bytes: payload.len() as u64,
            payload,
        };
        let expected = ExpectedOutputTransferArtifactFacts {
            output_name: artifact.output_name,
            logical_path: artifact.logical_path,
            kind: artifact.kind,
            digest_blake3: artifact.digest_blake3,
            size_bytes: artifact.size_bytes,
        };
        let mut sequence = OutputTransferArtifactSequence::new("request", 2, 2, 14).unwrap();
        assert_eq!(sequence.observe(artifact, Some(expected)), Ok(()));
        assert_eq!(
            sequence.observe(artifact, Some(expected)),
            Err(OutputTransferArtifactBlocker::Duplicate)
        );
        assert_eq!(sequence.finish(2), Err(OutputTransferArtifactBlocker::Missing));
        let mut tampered = OutputTransferArtifactSequence::new("request", 1, 2, 14).unwrap();
        assert_eq!(
            tampered.observe(OutputTransferArtifactFacts { payload: b"PAYLOAD", ..artifact }, Some(expected)),
            Err(OutputTransferArtifactBlocker::DigestMismatch)
        );
        let mut correct = OutputTransferArtifactSequence::new("request", 1, 2, 14).unwrap();
        assert_eq!(correct.observe(artifact, Some(expected)), Ok(()));
        assert_eq!(correct.finish(1), Ok(()));
    }

    #[test]
    fn worker_output_facts_keep_nar_failure_before_later_metadata_errors() {
        const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let expected = [
            ExpectedOutputFacts {
                name: "out",
                logical_path: Some("/store/out"),
            },
            ExpectedOutputFacts {
                name: "dev",
                logical_path: Some("/store/dev"),
            },
        ];
        let out = ExecutionOutputFacts {
            name: "out",
            logical_path: "/store/out",
            content_digest_blake3: DIGEST,
            artifact_attestation_digest_blake3: DIGEST,
            nar_payload_present: true,
            pathinfo_present: true,
        };
        let dev = ExecutionOutputFacts {
            name: "dev",
            logical_path: "/store/dev",
            ..out
        };
        let checked = core::cell::Cell::new(0_u32);
        assert_eq!(
            validate_execution_output_facts(expected.iter().copied(), [(out, ()), (dev, ())].into_iter(), "/store", |_| {
                checked.set(checked.get() + 1);
                Ok::<(), &'static str>(())
            }),
            Ok(())
        );
        assert_eq!(checked.get(), 2);
        assert_eq!(
            validate_execution_output_facts(
                expected.iter().copied(),
                [(ExecutionOutputFacts { pathinfo_present: false, ..out }, ()), (dev, ())].into_iter(),
                "/store",
                |_| Ok::<(), &'static str>(()),
            ),
            Err(ExecutionOutputValidationError::Decision(OutputBlocker::ExecutionOutputNarPathinfoMissing)),
        );
        assert_eq!(
            validate_execution_output_facts(
                expected.iter().copied(),
                [(out, ()), (ExecutionOutputFacts { name: "out", ..dev }, ())].into_iter(),
                "/store",
                |_| Err::<(), &'static str>("nar-digest-mismatch"),
            ),
            Err(ExecutionOutputValidationError::Inspect("nar-digest-mismatch")),
        );
    }

    #[test]
    fn nar_summary_rejects_bad_digest_and_unbound_pathinfo() {
        let valid = "a".repeat(BLAKE3_HEX_LENGTH_CHARS);
        assert_eq!(
            admit_nar_summary(NarSummaryFacts {
                payload_digest_blake3: Some(&valid),
                payload_size_bytes: Some(0),
                pathinfo_present: true,
            }),
            Ok(())
        );
        assert_eq!(
            admit_nar_summary(NarSummaryFacts {
                payload_digest_blake3: Some("not-blake3"),
                payload_size_bytes: Some(0),
                pathinfo_present: false,
            }),
            Err(OutputBlocker::NarDigestInvalid)
        );
        assert_eq!(
            admit_nar_summary(NarSummaryFacts {
                payload_digest_blake3: Some(&valid),
                payload_size_bytes: Some(0),
                pathinfo_present: false,
            }),
            Err(OutputBlocker::NarPathinfoMissing)
        );
        assert_eq!(
            admit_nar_summary(NarSummaryFacts {
                payload_digest_blake3: Some(&valid),
                payload_size_bytes: None,
                pathinfo_present: true,
            }),
            Err(OutputBlocker::NarSummaryIncomplete)
        );
    }

    #[test]
    fn transfer_report_distinguishes_fallback_and_actual_streaming() {
        let full = TransferReportFacts {
            verified_builder_key: "builder-key",
            mode: TransferMode::Full,
            mode_label: "full",
            reused_bytes: 0,
            fallback_reason: Some("streaming-runtime-not-bound"),
        };
        assert_eq!(admit_transfer_report(full), Ok(()));
        let claimed_streaming = TransferReportFacts {
            mode: TransferMode::Streaming,
            mode_label: "streaming",
            ..full
        };
        assert_eq!(admit_transfer_report(claimed_streaming), Err(OutputBlocker::TransferStreamingHasFallbackReason));
        let reused_full = TransferReportFacts {
            reused_bytes: 1,
            ..full
        };
        assert_eq!(admit_transfer_report(reused_full), Err(OutputBlocker::TransferFullReusedBytesNonzero));
        let unbound_key = TransferReportFacts {
            verified_builder_key: "",
            ..full
        };
        assert_eq!(admit_transfer_report(unbound_key), Err(OutputBlocker::TransferBuilderKeyEmpty));
    }

    #[test]
    fn metadata_admission_checks_ordered_receipt_then_identity_and_nar_shape() {
        let valid = "a".repeat(BLAKE3_HEX_LENGTH_CHARS);
        let expected = [ExpectedOutputFacts {
            name: "out",
            logical_path: Some("/store/out"),
        }];
        let output = ProducedOutputFacts {
            receipt: OutputReceiptFields {
                name: "out",
                logical_path: "/store/out",
                content_digest_blake3: &valid,
                artifact_attestation_digest_blake3: &valid,
                size_bytes: u64::MAX,
                nar_payload_digest_blake3: None,
                nar_payload_size_bytes: None,
            },
            path_info_signing_key_id: "builder-key",
            pathinfo_present: false,
        };
        let digest = |facts: &[ProducedOutputFacts<'_>]| {
            let mut receipt = OrderedOutputReceipt::new();
            for fact in facts {
                receipt.append(fact.receipt);
            }
            blake3::Hash::from_bytes(receipt.finish()).to_hex()
        };
        let receipt = digest(&[output]);
        let validate = |outputs: &[ProducedOutputFacts<'_>], expected: &[ExpectedOutputFacts<'_>], claimed: &str| {
            validate_produced_output_metadata(
                expected.iter().copied(),
                outputs.iter().copied(),
                claimed,
                "builder-key",
                "/store",
            )
        };
        assert_eq!(validate(&[output], &expected, receipt.as_str()), Ok(()));
        assert_eq!(
            validate(&[output], &[], "wrong"),
            Err(OutputBlocker::MetadataCountMismatch),
            "count precedes even a forged digest"
        );
        assert_eq!(validate(&[output], &expected, "wrong"), Err(OutputBlocker::MetadataDigestMismatch));
        let wrong_key = ProducedOutputFacts {
            path_info_signing_key_id: "other",
            ..output
        };
        assert_eq!(
            validate(&[wrong_key], &expected, receipt.as_str()),
            Err(OutputBlocker::MetadataPathinfoKeyMismatch)
        );
        let incomplete_nar = ProducedOutputFacts {
            receipt: OutputReceiptFields {
                nar_payload_digest_blake3: Some(&valid),
                nar_payload_size_bytes: None,
                ..output.receipt
            },
            ..output
        };
        let nar_digest = digest(&[incomplete_nar]);
        assert_eq!(
            validate(&[incomplete_nar], &expected, nar_digest.as_str()),
            Err(OutputBlocker::NarSummaryIncomplete)
        );
        let duplicated = [output, output];
        let duplicate_expected = [expected[0], ExpectedOutputFacts {
            name: "next",
            logical_path: Some("/store/next"),
        }];
        let duplicate_digest = digest(&duplicated);
        assert_eq!(
            validate(&duplicated, &duplicate_expected, duplicate_digest.as_str()),
            Err(OutputBlocker::MetadataNameDuplicate)
        );
    }

    #[test]
    fn policy_key_material_matching_does_not_claim_content_trust() {
        let material = match_output_key("builder:secret-a", ["builder:secret-a"], true).unwrap();
        assert_eq!(material.key_material_digest_blake3, Some(*blake3::hash(b"secret-a").as_bytes()));
        assert_eq!(
            match_output_key("builder:secret-b", ["builder:secret-a"], true),
            Err(OutputTrustBlocker::SameNameDifferentMaterial)
        );
        assert_eq!(
            match_output_key("builder", ["builder:secret-a"], true),
            Err(OutputTrustBlocker::KeyMaterialMissing)
        );
        assert_eq!(match_output_key("builder", ["builder"], false), Err(OutputTrustBlocker::StorePrefixMismatch));
        assert_eq!(
            match_output_key("builder:secret-a", ["builder"], true),
            Ok(material),
            "legacy unqualified policy still selects the same key; std validates content"
        );
    }

    #[test]
    fn scope_and_transfer_key_preserve_first_failure_precedence() {
        let valid = "a".repeat(BLAKE3_HEX_LENGTH_CHARS);
        let facts = OutputScopeFacts {
            expected_request_id: "request",
            request_id: "other",
            expected_store_prefix: "/store",
            store_prefix: "/wrong",
            output_digest_blake3: "invalid",
        };
        assert_eq!(validate_output_scope(facts), Err(OutputBlocker::RequestIdentityMismatch));
        assert_eq!(
            validate_output_scope(OutputScopeFacts {
                request_id: "request",
                ..facts
            }),
            Err(OutputBlocker::StorePrefixMismatch)
        );
        assert_eq!(
            validate_output_scope(OutputScopeFacts {
                request_id: "request",
                store_prefix: "/store",
                ..facts
            }),
            Err(OutputBlocker::ResultDigestInvalid)
        );
        assert_eq!(
            validate_output_scope(OutputScopeFacts {
                request_id: "request",
                store_prefix: "/store",
                output_digest_blake3: &valid,
                ..facts
            }),
            Ok(())
        );
        assert_eq!(
            validate_transfer_builder_key("unrelated", "builder"),
            Err(OutputBlocker::TransferBuilderKeyMismatch)
        );
    }
}
