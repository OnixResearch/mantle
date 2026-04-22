use crunch_attestation::ArtifactReference;
use crunch_attestation::ProjectAttestation;
use crunch_project_core::ProjectAttestationRequest;

use crate::Error;
use crate::Lockfile;
use crate::ProjectManifest;

pub struct ProjectAttestationInput<'a> {
    pub manifest_text: &'a str,
    pub lock_text: &'a str,
    pub manifest: &'a ProjectManifest,
    pub lock: &'a Lockfile,
    pub selected_roots: &'a [ArtifactReference],
}

pub fn synthesize_project_attestation(input: ProjectAttestationInput<'_>) -> Result<ProjectAttestation, Error> {
    crunch_project_core::synthesize_project_attestation(ProjectAttestationRequest {
        manifest_text: input.manifest_text.to_string(),
        lock_text: input.lock_text.to_string(),
        manifest: input.manifest.clone(),
        lock: input.lock.clone(),
        selected_roots: input.selected_roots.to_vec(),
    })
    .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crunch_attestation::AttestationDigest;

    use super::*;
    use crate::HashAlgo;
    use crate::LockedHash;
    use crate::LockedKind;
    use crate::SchemaVersion;

    #[test]
    fn project_attestation_adapter_preserves_borrowed_input_api() {
        let manifest = ProjectManifest {
            version: "1.0.0".to_string(),
            inputs: vec![crate::ManifestInput {
                name: "root".to_string(),
                kind: crate::InputKind::File {
                    url: "https://example.com/root".to_string(),
                },
                hash: Default::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec![],
            }],
            patches: vec![],
        };
        let lock = Lockfile {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::from([("root".to_string(), crate::LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/root".to_string(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-root=".to_string(),
                },
                patches: vec![],
                mirrors: vec![],
            })]),
            patches: BTreeMap::new(),
        };
        let roots = vec![ArtifactReference {
            node_id: "artifact:/nix/store/root".to_string(),
            logical_path: "/nix/store/root".to_string(),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"root"),
        }];
        let attestation = synthesize_project_attestation(ProjectAttestationInput {
            manifest_text: "manifest",
            lock_text: &lock.to_json().unwrap(),
            manifest: &manifest,
            lock: &lock,
            selected_roots: &roots,
        })
        .unwrap();
        assert_eq!(attestation.facts.selected_roots.len(), 1);
    }
}
