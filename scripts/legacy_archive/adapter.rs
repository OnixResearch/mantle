//! Archive facts, signature/CA protocol translation, and closed-closure checks.
use super::{
    Plan, Request, Result,
    codec::{self, Header, Node, PathFrame, PathInfo},
    core::*,
};
use nix_compat::{
    narinfo::VerifyingKey,
    nixhash::{CAHash, NixHash},
    store_path::build_ca_path_with_store_dir,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Serialize)]
pub struct RecordEvidence {
    pub store_path: String,
    pub previous_node: Node,
    pub current_node: Node,
    pub disposition: &'static str,
    pub ca_evidence: &'static str,
    pub marker_replacements: u32,
    pub signed_nar_sha256: String,
    pub payload_blake3: String,
}

pub fn validate_request(r: &Request) -> Result<()> {
    if r.schema != "mantle-legacy-archive-migration-request-v1"
        || r.expected_input_blake3.len() != 64
        || !r.expected_input_blake3.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        || r.roots.is_empty()
        || r.roots.len() > 64
        || r.trusted_public_keys.is_empty()
        || r.trusted_public_keys.len() > 16
    {
        return Err("invalid request schema, identity, roots, or trust bounds".into());
    }
    if !r.store_prefix.starts_with('/')
        || r.store_prefix.len() > 1024
        || r.store_prefix[1..]
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == ".." || c.chars().any(char::is_control))
    {
        return Err("invalid logical store prefix".into());
    }
    for key in &r.trusted_public_keys {
        VerifyingKey::parse(key)?;
    }
    if r.roots.iter().collect::<BTreeSet<_>>().len() != r.roots.len() {
        return Err("duplicate requested roots".into());
    }
    Ok(())
}
pub fn validate_header(h: &Header, r: &Request) -> Result<()> {
    if h.format != "mantle-store-archive-v1"
        || h.version != 1
        || h.payload_len != 0
        || h.store_prefix != r.store_prefix
        || h.record_count == 0
        || h.record_count > 64
        || h.compatibility != "mantle-native; nario-v2 byte compatibility unproven"
        || h.roots.len() != r.roots.len()
        || h.roots.iter().collect::<BTreeSet<_>>() != r.roots.iter().collect::<BTreeSet<_>>()
    {
        return Err("header binding mismatch".into());
    }
    Ok(())
}
fn trusted(pi: &PathInfo, r: &Request) -> Result<bool> {
    if pi.signatures.len() > 64 || pi.references.len() > 64 {
        return Err("signature/reference bound".into());
    }
    let refs: Vec<_> = pi.references.iter().map(|p| p.as_ref()).collect();
    let fp = nix_compat::narinfo::fingerprint_with_store_dir(
        &pi.store_path.as_ref(),
        &pi.nar_sha256,
        pi.nar_size,
        refs.iter(),
        &r.store_prefix,
    );
    let keys: Vec<_> = r
        .trusted_public_keys
        .iter()
        .map(|k| VerifyingKey::parse(k))
        .collect::<std::result::Result<_, _>>()?;
    Ok(pi.signatures.iter().any(|s| keys.iter().any(|k| k.verify(&fp, &s.as_ref()))))
}

fn ca_path_matches(pi: &PathInfo, prefix: &str) -> Result<bool> {
    let Some(ca) = &pi.ca else {
        return Ok(true);
    };
    let marker = build_ca_path_with_store_dir(pi.store_path.name(), ca, Vec::<String>::new(), false, prefix)?;
    if marker == pi.store_path {
        return Ok(true);
    }
    let refs: Vec<_> = pi.references.iter().filter(|p| **p != pi.store_path).map(ToString::to_string).collect();
    let candidate =
        build_ca_path_with_store_dir(pi.store_path.name(), ca, refs, pi.references.contains(&pi.store_path), prefix);
    Ok(candidate.is_ok_and(|p| p == pi.store_path))
}

pub fn observe_ca(pi: &PathInfo, bytes: &[u8], prefix: &str) -> CaEvidence {
    let Some(ca) = &pi.ca else {
        return CaEvidence::Absent;
    };
    // Closed first subset: recursive SHA-256 only. No stripping unknown CA modes.
    let CAHash::Nar(NixHash::Sha256(expected)) = ca else {
        return CaEvidence::Rejected;
    };
    if Sha256::digest(bytes).as_slice() == expected {
        return CaEvidence::FinalNar;
    }
    let path = format!("{prefix}/{}", pi.store_path);
    let target = path.as_bytes();
    let seed = blake3::hash(b"crunch-ca-marker:out");
    let marker: Vec<u8> = seed.as_bytes().iter().copied().cycle().take(target.len()).collect();
    let mut hash = Sha256::new();
    let mut count = 0u32;
    let mut start = 0usize;
    // Stream the substituted hash. Never modify or publish normalized NAR bytes.
    for (index, window) in bytes.windows(target.len()).enumerate() {
        if index >= start && window == target {
            hash.update(&bytes[start..index]);
            hash.update(&marker);
            start = index + target.len();
            count += 1;
            if count > 131_072 {
                return CaEvidence::Rejected;
            }
        }
    }
    hash.update(&bytes[start..]);
    if count > 0 && hash.finalize().as_slice() == expected {
        CaEvidence::MarkerNar { replacements: count }
    } else {
        CaEvidence::Rejected
    }
}

pub fn admit_record(frame: &mut PathFrame, bytes: &[u8], r: &Request) -> Result<RecordEvidence> {
    let pi = &frame.path_info;
    let previous_node = pi.node.clone();
    let actual_sha: [u8; 32] = Sha256::digest(bytes).into();
    let actual_b3 = blake3::hash(bytes).to_hex().to_string();
    let is_trusted = trusted(pi, r)?;
    let ca = observe_ca(pi, bytes, &r.store_prefix);
    let path_matches = ca_path_matches(pi, &r.store_prefix)?;
    let (current, legacy) = codec::observe_nar(bytes)?;
    let observed = match (&previous_node, &current, &legacy) {
        (
            Node::Directory { digest, size },
            Node::Directory { digest: nd, size: ns },
            Node::Directory { digest: od, size: os },
        ) => (
            DirectoryIdentity {
                digest: *digest,
                descendants: *size,
            },
            DirectoryIdentity {
                digest: *nd,
                descendants: *ns,
            },
            DirectoryIdentity {
                digest: *od,
                descendants: *os,
            },
        ),
        _ if previous_node == current => {
            let identity = DirectoryIdentity {
                digest: [0; 32],
                descendants: 0,
            };
            (identity, identity, identity)
        }
        _ => return Err("non-directory node mismatch".into()),
    };
    let disposition = plan_directory_migration(MigrationFacts {
        trusted_signature: is_trusted,
        final_nar_matches: actual_sha == pi.nar_sha256 && bytes.len() as u64 == pi.nar_size,
        payload_blake3_matches: actual_b3 == frame.payload_blake3,
        ca_path_matches: path_matches,
        ca,
        recorded: observed.0,
        current: observed.1,
        legacy: observed.2,
    })
    .map_err(|error| format!("{}: {error:?}", pi.store_path))?;
    let (ca_evidence, marker_replacements) = match ca {
        CaEvidence::Absent => ("absent", 0),
        CaEvidence::FinalNar => ("final-nar", 0),
        CaEvidence::MarkerNar { replacements } => ("mantle-out-marker-nar-v1", replacements),
        CaEvidence::Rejected => unreachable!("core rejected CA"),
    };
    let evidence = RecordEvidence {
        store_path: pi.store_path.to_string(),
        previous_node,
        current_node: current.clone(),
        disposition: match disposition {
            Disposition::Unchanged => "unchanged",
            Disposition::CanonicalizeDoubledDirectoryCounts => "canonicalized-doubled-counts",
        },
        ca_evidence,
        marker_replacements,
        signed_nar_sha256: format!("{:x}", Sha256::digest(bytes)),
        payload_blake3: actual_b3,
    };
    // This is the sole permitted metadata difference. All signed fields and CA remain.
    frame.path_info.node = current;
    Ok(evidence)
}

pub fn validate_closure(header: &Header, plans: &[Plan]) -> Result<()> {
    let records = plans
        .iter()
        .map(|(p, _)| ClosureRecord {
            identity: p.path_info.store_path.to_string(),
            root: p.root,
            references: p.path_info.references.iter().map(ToString::to_string).collect(),
        })
        .collect();
    validate_closed_archive(header.roots.clone(), records).map_err(|e| format!("closure rejected: {e:?}").into())
}
