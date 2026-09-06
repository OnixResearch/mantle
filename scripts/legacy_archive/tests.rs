use super::*;
use codec::{Node, PathInfo};
use nix_compat::{
    narinfo,
    nixhash::{CAHash, NixHash},
    store_path::{StorePath, build_ca_path_with_store_dir},
};
use sha2::{Digest, Sha256};
const PREFIX: &str = "/mantle/store";
// Public upstream fixture from vendor/nix-compat/src/narinfo/mod.rs, never an operator key.
const KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

fn token(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
    out.resize(out.len().next_multiple_of(8), 0);
}
fn nar(contents: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for t in [
        "nix-archive-1",
        "(",
        "type",
        "directory",
        "entry",
        "(",
        "name",
        "sub",
        "node",
        "(",
        "type",
        "directory",
        "entry",
        "(",
        "name",
        "file",
        "node",
        "(",
        "type",
        "regular",
        "contents",
    ] {
        token(&mut out, t.as_bytes());
    }
    token(&mut out, contents);
    for _ in 0..5 {
        token(&mut out, b")");
    }
    out
}
fn fixture() -> (Request, Header, PathFrame, Vec<u8>) {
    let bytes = nar(b"fixture bytes");
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    let ca = CAHash::Nar(NixHash::Sha256(hash));
    let path = build_ca_path_with_store_dir("fixture", &ca, Vec::<String>::new(), false, PREFIX).unwrap();
    let (_, old) = codec::observe_nar(&bytes).unwrap();
    let (key, public) = narinfo::parse_keypair(KEY).unwrap();
    let fp = narinfo::fingerprint_with_store_dir(&path.as_ref(), &hash, bytes.len() as u64, [].iter(), PREFIX);
    let name = path.to_string();
    let request = Request {
        schema: "mantle-legacy-archive-migration-request-v1".into(),
        expected_input_blake3: "0".repeat(64),
        store_prefix: PREFIX.into(),
        roots: vec![name.clone()],
        trusted_public_keys: vec![public.to_string()],
    };
    let header = Header {
        format: "mantle-store-archive-v1".into(),
        version: 1,
        store_prefix: PREFIX.into(),
        record_count: 1,
        roots: vec![name],
        compatibility: "mantle-native; nario-v2 byte compatibility unproven".into(),
        payload_len: 0,
    };
    let frame = PathFrame {
        path_info: PathInfo {
            store_path: path,
            node: old,
            references: vec![],
            nar_size: bytes.len() as u64,
            nar_sha256: hash,
            signatures: vec![key.sign(fp.as_bytes()).to_owned()],
            deriver: None,
            ca: Some(ca),
        },
        root: true,
        payload_len: bytes.len() as u64,
        payload_blake3: blake3::hash(&bytes).to_hex().to_string(),
    };
    (request, header, frame, bytes)
}
fn archive(header: Header, path: PathFrame, payload: &[u8]) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    codec::write_frame(&mut out, &Frame::Header(header)).unwrap();
    codec::write_frame(&mut out, &Frame::Path(Box::new(path))).unwrap();
    out.extend_from_slice(payload);
    codec::write_frame(
        &mut out,
        &Frame::End(codec::End {
            record_count: 1,
            total_payload_bytes: payload.len() as u64,
            payload_len: 0,
        }),
    )
    .unwrap();
    out
}
#[test]
fn signed_migration_changes_only_node_and_preserves_payload() {
    let (r, h, f, bytes) = fixture();
    let old = serde_json::to_value(&f.path_info).unwrap();
    let input = archive(h, f, &bytes);
    let (_, plans, evidence) = inspect_archive(&input, &r).unwrap();
    assert_eq!(evidence[0].disposition, "canonicalized-doubled-counts");
    assert_eq!(&input[plans[0].1.clone()], &bytes);
    let mut new = serde_json::to_value(&plans[0].0.path_info).unwrap();
    new["node"] = old["node"].clone();
    assert_eq!(new, old);
    assert!(matches!(plans[0].0.path_info.node, Node::Directory { size: 2, .. }));
    let replay = archive(
        codec::Header {
            roots: r.roots.clone(),
            ..fixture().1
        },
        plans[0].0.clone(),
        &bytes,
    );
    assert_eq!(inspect_archive(&replay, &r).unwrap().2[0].disposition, "unchanged");
}
#[test]
fn denies_poisoned_record_facts() {
    let (r, _, good, payload) = fixture();
    for case in 0..7 {
        let mut f = good.clone();
        match case {
            0 => f.path_info.signatures.clear(),
            1 => f.path_info.nar_sha256[0] ^= 1,
            2 => f.payload_blake3 = "0".repeat(64),
            3 => {
                if let Node::Directory { digest, .. } = &mut f.path_info.node {
                    digest[0] ^= 1;
                }
            }
            4 => {
                if let Node::Directory { size, .. } = &mut f.path_info.node {
                    *size += 1;
                }
            }
            5 => f.path_info.ca = Some(CAHash::Nar(NixHash::Sha256([5; 32]))),
            _ => f.path_info.references.push(StorePath::from_name_and_digest_fixed("missing", [5; 20]).unwrap()),
        }
        assert!(adapter::admit_record(&mut f, &payload, &r).is_err(), "case {case}");
    }
}
#[test]
fn marker_hash_is_checked_not_replaced() {
    let (r, _, mut f, _) = fixture();
    let path = format!("{PREFIX}/{}", f.path_info.store_path);
    let bytes = nar(path.as_bytes());
    let seed = blake3::hash(b"crunch-ca-marker:out");
    let normalized: Vec<_> = seed.as_bytes().iter().copied().cycle().take(path.len()).collect();
    let expected: [u8; 32] = Sha256::digest(nar(&normalized)).into();
    f.path_info.ca = Some(CAHash::Nar(NixHash::Sha256(expected)));
    assert_eq!(
        adapter::observe_ca(&f.path_info, &bytes, &r.store_prefix),
        core::CaEvidence::MarkerNar { replacements: 1 }
    );
    assert_eq!(adapter::observe_ca(&f.path_info, &nar(b"wrong"), &r.store_prefix), core::CaEvidence::Rejected);
    assert_eq!(f.path_info.ca, Some(CAHash::Nar(NixHash::Sha256(expected))));
}
#[test]
fn rejects_bad_framing_and_nar() {
    let (r, h, f, bytes) = fixture();
    let good = archive(h, f, &bytes);
    for len in [0, 5, good.len() - 1] {
        assert!(inspect_archive(&good[..len], &r).is_err());
    }
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(inspect_archive(&trailing, &r).is_err());
    assert!(codec::observe_nar(b"bad").is_err());
    let mut trailing_nar = bytes;
    trailing_nar.push(0);
    assert!(codec::observe_nar(&trailing_nar).is_err());
}
#[test]
fn checks_roots_prefix_closure_and_trust() {
    let (mut r, h, f, bytes) = fixture();
    r.roots.push("extra".into());
    assert!(adapter::validate_header(&h, &r).is_err());
    r.roots.pop();
    r.store_prefix = "/other/store".into();
    assert!(adapter::validate_header(&h, &r).is_err());
    let (r, h, mut f2, _) = fixture();
    f2.root = false;
    assert!(adapter::validate_closure(&h, &[(f2, 0..0)]).is_err());
    let mut missing = f.clone();
    missing
        .path_info
        .references
        .push(StorePath::from_name_and_digest_fixed("missing", [5; 20]).unwrap());
    assert!(adapter::validate_closure(&h, &[(missing, 0..0)]).is_err());
    assert!(adapter::validate_closure(&h, &[(f.clone(), 0..0), (f.clone(), 0..0)]).is_err());
    let mut wrong_key = r;
    wrong_key.trusted_public_keys = vec!["wrong:913onE1x8N2z1n2xFA9bbpmhNCfsWrwUnoV9ESvy+RM=".into()];
    assert!(adapter::admit_record(&mut f.clone(), &bytes, &wrong_key).is_err());
}
#[test]
fn shell_denial_leaves_no_output_and_success_never_clobbers() {
    let dir = tempfile::tempdir().unwrap();
    let (mut r, h, f, bytes) = fixture();
    let data = archive(h, f, &bytes);
    let input = dir.path().join("input");
    std::fs::write(&input, &data).unwrap();
    let rp = dir.path().join("request");
    let dest = dir.path().join("out");
    fn write_request(r: &Request, path: &Path) {
        std::fs::write(
            path,
            serde_json::to_vec(&serde_json::json!({"schema":r.schema, "expected_input_blake3":r.expected_input_blake3,
            "store_prefix":r.store_prefix, "roots":r.roots,"trusted_public_keys":r.trusted_public_keys}))
            .unwrap(),
        )
        .unwrap();
    }
    write_request(&r, &rp);
    assert!(migrate(&rp, &input, &dest).is_err());
    assert!(!dest.exists());
    r.expected_input_blake3 = blake3::hash(&data).to_hex().to_string();
    write_request(&r, &rp);
    let receipt = migrate(&rp, &input, &dest).unwrap();
    assert!(!receipt.package_realization);
    let saved = std::fs::read(dest.join("archive")).unwrap();
    assert_eq!(blake3::hash(&saved).to_hex().as_str(), receipt.output_blake3);
    assert!(migrate(&rp, &input, &dest).is_err());
    assert_eq!(std::fs::read(dest.join("archive")).unwrap(), saved);
    assert_eq!(std::fs::read(&input).unwrap(), data);
    let (_, h, mut poisoned, payload) = fixture();
    poisoned.path_info.signatures.clear();
    let rejected = archive(h, poisoned, &payload);
    std::fs::write(&input, &rejected).unwrap();
    r.expected_input_blake3 = blake3::hash(&rejected).to_hex().to_string();
    write_request(&r, &rp);
    let absent = dir.path().join("denied");
    assert!(migrate(&rp, &input, &absent).is_err());
    assert!(!absent.exists());
}

#[test]
fn closed_graph_preserves_cycles_but_rejects_extras() {
    use super::core::{ClosureRecord, ClosureRejection, validate_closed_archive};
    let record = |name: &str, root, refs: &[&str]| ClosureRecord {
        identity: name.into(),
        root,
        references: refs.iter().map(|s| (*s).into()).collect(),
    };
    assert!(
        validate_closed_archive(vec!["a".into()], vec![record("a", true, &["b"]), record("b", false, &["a"])]).is_ok()
    );
    assert_eq!(
        validate_closed_archive(vec!["a".into()], vec![record("a", true, &[]), record("b", false, &[])]),
        Err(ClosureRejection::UnreachableMember)
    );
    assert_eq!(
        validate_closed_archive(vec!["a".into()], vec![record("a", true, &["a", "a"])]),
        Err(ClosureRejection::DuplicateReference)
    );
}

#[test]
#[cfg(target_os = "linux")]
fn publication_is_atomic_and_has_one_concurrent_winner() {
    let parent = tempfile::tempdir().unwrap();
    let output = parent.path().join("out");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut threads = Vec::new();
    for value in [b"one", b"two"] {
        let stage = tempfile::tempdir_in(parent.path()).unwrap();
        std::fs::write(stage.path().join("archive"), value).unwrap();
        std::fs::write(stage.path().join("receipt.json"), value).unwrap();
        let output = output.clone();
        let barrier = barrier.clone();
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            publish_directory(stage.path(), &output).is_ok()
        }));
    }
    barrier.wait();
    assert_eq!(threads.into_iter().filter_map(|t| t.join().ok()).filter(|won| *won).count(), 1);
    assert_eq!(std::fs::read(output.join("archive")).unwrap(), std::fs::read(output.join("receipt.json")).unwrap());
    let empty = parent.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    let stage = tempfile::tempdir_in(parent.path()).unwrap();
    assert!(publish_directory(stage.path(), &empty).is_err());
    assert!(empty.is_dir());
}

#[test]
fn rejects_lossy_metadata_and_resource_violations() {
    let (mut r, h, frame, _) = fixture();
    let mut json = serde_json::to_value(Frame::Path(Box::new(frame))).unwrap();
    json["path_info"]["ca"]["undeclared"] = serde_json::json!(true);
    let encoded = serde_json::to_vec(&json).unwrap();
    let mut wire = (encoded.len() as u32).to_le_bytes().to_vec();
    wire.extend(encoded);
    assert!(codec::read_frame(&mut std::io::Cursor::new(wire)).is_err());
    assert!(codec::read_frame(&mut std::io::Cursor::new(u32::MAX.to_le_bytes())).is_err());
    let oversized = Header { record_count: 65, ..h };
    assert!(adapter::validate_header(&oversized, &r).is_err());
    r.roots.push(r.roots[0].clone());
    assert!(adapter::validate_request(&r).is_err());
    r.roots.pop();
    r.trusted_public_keys.clear();
    assert!(adapter::validate_request(&r).is_err());
}
