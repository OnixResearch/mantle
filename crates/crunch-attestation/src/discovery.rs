use std::path::Path;
use std::path::PathBuf;

use crate::Canonicalize;
use crate::Error;
use crate::policy::ReleasePolicy;
use crate::policy::ReleaseRevocations;
use crate::policy::ValidatedWitness;
use crate::release::DetachedSignature;
use crate::release::RELEASE_ATTESTATION_SCHEMA;
use crate::release::ReleaseAttestation;
use crate::release::WITNESS_ATTESTATION_SCHEMA;
use crate::release::WitnessAttestation;

const MAX_WITNESS_FILES: u32 = 1_024;

// ---------------------------------------------------------------------------
// Verification directory layout
// ---------------------------------------------------------------------------

/// File-based discovery of release and witness attestations from a
/// verification directory.
///
/// Expected layout:
/// ```text
/// <dir>/
///   release-attestation.json
///   release-attestation.json.sig
///   witnesses/
///     <witness-identity>.json
///     <witness-identity>.json.sig
///   policy.json
///   revocations.json          (optional)
/// ```
pub struct VerificationDirectory {
    root: PathBuf,
}

/// Loaded verification material before signature/policy evaluation.
#[derive(Debug)]
pub struct VerificationMaterial {
    pub release_attestation: ReleaseAttestation,
    pub release_signature: DetachedSignature,
    pub release_attestation_path: PathBuf,
    pub witnesses: Vec<DiscoveredWitness>,
    pub policy: ReleasePolicy,
    pub revocations: ReleaseRevocations,
}

/// One discovered witness attestation with its parsed signature.
#[derive(Debug)]
pub struct DiscoveredWitness {
    pub attestation: WitnessAttestation,
    pub signature: DetachedSignature,
    pub attestation_path: PathBuf,
}

impl VerificationDirectory {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Discover and load all verification material from the directory.
    ///
    /// Validates:
    /// - Release attestation JSON exists and parses with correct schema tag
    /// - Release `.sig` sidecar exists and parses
    /// - Each witness `.json` has a matching `.json.sig` sidecar
    /// - Schema tags match expected values
    /// - Collection limits are respected
    ///
    /// Does NOT verify cryptographic signatures — that is the caller's job
    /// (the caller holds the trusted key material).
    pub fn discover(&self) -> Result<VerificationMaterial, DiscoveryError> {
        let release_path = self.root.join("release-attestation.json");
        let release_sig_path = self.root.join("release-attestation.json.sig");

        // Load release attestation.
        let release_json = read_file(&release_path)?;
        let release_attestation: ReleaseAttestation = parse_json(&release_json, &release_path)?;
        if release_attestation.schema != RELEASE_ATTESTATION_SCHEMA {
            return Err(DiscoveryError::SchemaTag {
                path: release_path,
                expected: RELEASE_ATTESTATION_SCHEMA,
                actual: release_attestation.schema.clone(),
            });
        }
        // Validate canonical form.
        let _ = release_attestation.canonical_bytes().map_err(|err| DiscoveryError::Validation {
            path: release_path.clone(),
            source: err,
        })?;

        // Load release signature.
        let release_sig_text = read_file_text(&release_sig_path)?;
        let release_signature =
            DetachedSignature::parse(release_sig_text.trim()).map_err(|err| DiscoveryError::Validation {
                path: release_sig_path,
                source: err,
            })?;

        // Load policy.
        let policy_path = self.root.join("policy.json");
        let policy_json = read_file(&policy_path)?;
        let policy: ReleasePolicy = parse_json(&policy_json, &policy_path)?;

        // Load revocations (optional).
        let revocations_path = self.root.join("revocations.json");
        let revocations = if revocations_path.exists() {
            let rev_json = read_file(&revocations_path)?;
            parse_json(&rev_json, &revocations_path)?
        } else {
            ReleaseRevocations::empty()
        };

        // Discover witnesses.
        let witnesses_dir = self.root.join("witnesses");
        let witnesses = if witnesses_dir.is_dir() {
            discover_witnesses(&witnesses_dir)?
        } else {
            Vec::new()
        };

        Ok(VerificationMaterial {
            release_attestation,
            release_signature,
            release_attestation_path: release_path,
            witnesses,
            policy,
            revocations,
        })
    }
}

fn discover_witnesses(dir: &Path) -> Result<Vec<DiscoveredWitness>, DiscoveryError> {
    let mut json_files: Vec<PathBuf> = Vec::new();
    assert!(json_files.is_empty(), "json witness list must start empty");
    let entries = std::fs::read_dir(dir).map_err(|err| DiscoveryError::Io {
        path: dir.to_path_buf(),
        source: err,
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| DiscoveryError::Io {
            path: dir.to_path_buf(),
            source: err,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            json_files.push(path);
        }
    }

    let count = u32::try_from(json_files.len()).unwrap_or(u32::MAX);
    if count > MAX_WITNESS_FILES {
        return Err(DiscoveryError::TooManyWitnesses {
            limit: MAX_WITNESS_FILES,
            actual: count,
        });
    }

    json_files.sort();
    let mut witnesses = Vec::with_capacity(json_files.len());
    assert!(witnesses.is_empty(), "witness output list must start empty");

    for json_path in &json_files {
        let sig_path = PathBuf::from(format!("{}.sig", json_path.display()));
        if !sig_path.exists() {
            return Err(DiscoveryError::MissingSigSidecar {
                attestation_path: json_path.clone(),
            });
        }

        let json_bytes = read_file(json_path)?;
        let attestation: WitnessAttestation = parse_json(&json_bytes, json_path)?;
        if attestation.schema != WITNESS_ATTESTATION_SCHEMA {
            return Err(DiscoveryError::SchemaTag {
                path: json_path.clone(),
                expected: WITNESS_ATTESTATION_SCHEMA,
                actual: attestation.schema.clone(),
            });
        }

        let sig_text = read_file_text(&sig_path)?;
        let signature = DetachedSignature::parse(sig_text.trim()).map_err(|err| DiscoveryError::Validation {
            path: sig_path,
            source: err,
        })?;

        witnesses.push(DiscoveredWitness {
            attestation,
            signature,
            attestation_path: json_path.clone(),
        });
    }

    Ok(witnesses)
}

/// Convert discovered witnesses into validated witnesses.
///
/// In the first phase, this creates `ValidatedWitness` entries from
/// the parsed material. Cryptographic signature verification is the
/// caller's responsibility — this function records the signer key name
/// from the parsed `.sig` file and computes the canonical attestation
/// digest.
///
/// Witnesses whose signatures were already validated externally can
/// use this to bridge discovery into policy evaluation.
pub fn to_validated_witnesses(discovered: &[DiscoveredWitness]) -> Result<Vec<ValidatedWitness>, Error> {
    let mut validated = Vec::with_capacity(discovered.len());
    for witness in discovered {
        let attestation_digest = witness.attestation.canonical_digest()?;
        validated.push(ValidatedWitness {
            attestation: witness.attestation.clone(),
            attestation_digest,
            signer_key_name: witness.signature.key_name.clone(),
        });
    }
    Ok(validated)
}

// ---------------------------------------------------------------------------
// Discovery errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum DiscoveryError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    SchemaTag {
        path: PathBuf,
        expected: &'static str,
        actual: String,
    },
    Validation {
        path: PathBuf,
        source: Error,
    },
    MissingSigSidecar {
        attestation_path: PathBuf,
    },
    TooManyWitnesses {
        limit: u32,
        actual: u32,
    },
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "I/O error reading {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(f, "JSON parse error in {}: {source}", path.display())
            }
            Self::SchemaTag { path, expected, actual } => {
                write!(f, "schema tag mismatch in {}: expected {expected}, got {actual}", path.display())
            }
            Self::Validation { path, source } => {
                write!(f, "validation error in {}: {source}", path.display())
            }
            Self::MissingSigSidecar { attestation_path } => {
                write!(f, "missing .sig sidecar for {}", attestation_path.display())
            }
            Self::TooManyWitnesses { limit, actual } => {
                write!(f, "too many witness files: {actual} > {limit}")
            }
        }
    }
}

impl std::error::Error for DiscoveryError {}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_file(path: &Path) -> Result<Vec<u8>, DiscoveryError> {
    std::fs::read(path).map_err(|err| DiscoveryError::Io {
        path: path.to_path_buf(),
        source: err,
    })
}

fn read_file_text(path: &Path) -> Result<String, DiscoveryError> {
    std::fs::read_to_string(path).map_err(|err| DiscoveryError::Io {
        path: path.to_path_buf(),
        source: err,
    })
}

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8], path: &Path) -> Result<T, DiscoveryError> {
    serde_json::from_slice(bytes).map_err(|err| DiscoveryError::Parse {
        path: path.to_path_buf(),
        source: err,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::AttestationDigest;
    use crate::Canonicalize;
    use crate::release::BinaryDigest;
    use crate::release::RebuildEnvironmentSummary;
    use crate::release::Workflow;

    // -- Discovery from verification directory -----------------------------

    #[test]
    fn discovers_release_and_witnesses_from_directory() {
        let dir = tempfile::tempdir().unwrap();
        let layout = write_full_layout(dir.path(), 2);

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let material = vdir.discover().unwrap();

        assert_eq!(material.release_attestation.release_id, "crunch-0.1.0");
        assert_eq!(material.release_signature.key_name, "release-signer-1");
        assert_eq!(material.witnesses.len(), 2);
        assert_eq!(material.policy.min_matching_witnesses, 2);

        let validated = to_validated_witnesses(&material.witnesses).unwrap();
        assert_eq!(validated.len(), 2);
        assert_eq!(validated[0].signer_key_name, layout.witness_keys[0]);
        assert_eq!(validated[1].signer_key_name, layout.witness_keys[1]);
    }

    #[test]
    fn discovers_with_zero_witnesses() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 0);

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let material = vdir.discover().unwrap();

        assert!(material.witnesses.is_empty());
    }

    #[test]
    fn discovers_without_witnesses_directory() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 0);
        // Remove witnesses dir entirely.
        let _ = fs::remove_dir_all(dir.path().join("witnesses"));

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let material = vdir.discover().unwrap();

        assert!(material.witnesses.is_empty());
    }

    #[test]
    fn discovers_without_revocations_file() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 0);
        fs::remove_file(dir.path().join("revocations.json")).unwrap();

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let material = vdir.discover().unwrap();

        assert!(material.revocations.revoked_witness_keys.is_empty());
    }

    // -- Missing .sig sidecar rejection ------------------------------------

    #[test]
    fn rejects_witness_missing_sig_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 1);
        // Remove the .sig file for the witness.
        let sig_path = dir.path().join("witnesses").join("witness-a.json.sig");
        fs::remove_file(&sig_path).unwrap();

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let err = vdir.discover().unwrap_err();

        assert!(matches!(err, DiscoveryError::MissingSigSidecar { .. }));
    }

    #[test]
    fn rejects_release_missing_sig_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 0);
        fs::remove_file(dir.path().join("release-attestation.json.sig")).unwrap();

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let err = vdir.discover().unwrap_err();

        assert!(matches!(err, DiscoveryError::Io { .. }));
    }

    // -- Schema tag validation ---------------------------------------------

    #[test]
    fn rejects_release_wrong_schema_tag() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 0);
        // Overwrite release attestation with wrong schema.
        let mut release = sample_release();
        release.schema = "wrong".to_string();
        let json = serde_json::to_vec_pretty(&release).unwrap();
        fs::write(dir.path().join("release-attestation.json"), json).unwrap();

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let err = vdir.discover().unwrap_err();

        assert!(matches!(err, DiscoveryError::SchemaTag { .. }));
    }

    // -- End-to-end with policy evaluation ---------------------------------

    #[test]
    fn end_to_end_discovery_and_policy_evaluation() {
        let dir = tempfile::tempdir().unwrap();
        write_full_layout(dir.path(), 2);

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let material = vdir.discover().unwrap();
        let validated = to_validated_witnesses(&material.witnesses).unwrap();

        let result = crate::policy::evaluate_policy(
            &material.release_attestation,
            &validated,
            &material.policy,
            &material.revocations,
        )
        .unwrap();

        assert_eq!(result.trust_tier.technical_class, crate::release::TechnicalClass::ExternalWitnessMatch);
        assert_eq!(result.trust_tier.policy_status, crate::release::PolicyStatus::Satisfied);
        assert_eq!(result.trust_tier.final_class, crate::release::FinalClass::QuorumSatisfied);
    }

    #[test]
    fn discovery_applies_revocations_before_quorum() {
        let dir = tempfile::tempdir().unwrap();
        let layout = write_full_layout(dir.path(), 2);

        // Revoke witness-a's key.
        let revocations = ReleaseRevocations::new(vec![layout.witness_keys[0].clone()], Vec::new());
        let rev_json = serde_json::to_vec_pretty(&revocations).unwrap();
        fs::write(dir.path().join("revocations.json"), rev_json).unwrap();

        let vdir = VerificationDirectory::new(dir.path().to_path_buf());
        let material = vdir.discover().unwrap();
        let validated = to_validated_witnesses(&material.witnesses).unwrap();

        let result = crate::policy::evaluate_policy(
            &material.release_attestation,
            &validated,
            &material.policy,
            &material.revocations,
        )
        .unwrap();

        assert_eq!(result.revoked_witness_count, 1);
        assert_eq!(result.matching_witness_count, 1);
        assert_eq!(result.trust_tier.policy_status, crate::release::PolicyStatus::Insufficient);
    }

    // -- Helpers -----------------------------------------------------------

    struct WrittenLayout {
        witness_keys: Vec<String>,
    }

    fn write_full_layout(dir: &Path, witness_count: u32) -> WrittenLayout {
        let release = sample_release();
        let release_bytes = release.canonical_bytes().unwrap();
        let release_json = serde_json::to_vec_pretty(&release).unwrap();
        fs::write(dir.join("release-attestation.json"), &release_json).unwrap();

        let release_sig = DetachedSignature {
            key_name: "release-signer-1".to_string(),
            signature_bytes: [0xab; 64],
        };
        fs::write(dir.join("release-attestation.json.sig"), release_sig.encode()).unwrap();

        let release_digest = AttestationDigest::from_canonical_bytes(&release_bytes);

        let witnesses_dir = dir.join("witnesses");
        fs::create_dir_all(&witnesses_dir).unwrap();

        let identities = ["witness-a", "witness-b", "witness-c", "witness-d"];
        let mut witness_keys = Vec::new();
        for i in 0..witness_count {
            let idx = i as usize;
            let identity = identities[idx];
            let key_name = format!("{identity}-key");
            let witness = WitnessAttestation::new(
                release_digest,
                identity.to_string(),
                release.binary_digests.clone(),
                RebuildEnvironmentSummary {
                    system: "x86_64-linux".to_string(),
                    toolchain: "rust-1.91.1".to_string(),
                    host_class: "nixos-25.05".to_string(),
                },
            );
            let witness_json = serde_json::to_vec_pretty(&witness).unwrap();
            fs::write(witnesses_dir.join(format!("{identity}.json")), witness_json).unwrap();

            let witness_sig = DetachedSignature {
                key_name: key_name.clone(),
                signature_bytes: [0xcd; 64],
            };
            fs::write(witnesses_dir.join(format!("{identity}.json.sig")), witness_sig.encode()).unwrap();
            witness_keys.push(key_name);
        }

        let policy = ReleasePolicy::new(
            witness_count,
            "witness_identity".to_string(),
            vec!["release-signer-1".to_string()],
            identities[..witness_count as usize].iter().map(|s| s.to_string()).collect(),
        );
        let policy_json = serde_json::to_vec_pretty(&policy).unwrap();
        fs::write(dir.join("policy.json"), policy_json).unwrap();

        let revocations = ReleaseRevocations::empty();
        let rev_json = serde_json::to_vec_pretty(&revocations).unwrap();
        fs::write(dir.join("revocations.json"), rev_json).unwrap();

        WrittenLayout { witness_keys }
    }

    fn sample_release() -> ReleaseAttestation {
        ReleaseAttestation::new(
            "crunch-0.1.0".to_string(),
            AttestationDigest::from_canonical_bytes(b"manifest"),
            AttestationDigest::from_canonical_bytes(b"proof"),
            "fixed-point".to_string(),
            Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            vec![BinaryDigest {
                name: "crunch".to_string(),
                algorithm: "blake3".to_string(),
                digest: "aa".repeat(32),
            }],
        )
    }
}
