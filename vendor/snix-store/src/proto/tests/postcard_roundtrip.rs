use super::super::*;
use bytes::Bytes;

#[test]
fn pathinfo_roundtrip() {
    let pi = PathInfo {
        entry: Some(snix_castore::proto::Entry {
            entry: Some(snix_castore::proto::entry::Entry::File(
                snix_castore::proto::FileEntry {
                    name: Bytes::from_static(b"test-path"),
                    digest: Bytes::from_static(&[0xaa; 32]),
                    size: 256,
                    executable: true,
                },
            )),
        }),
        references: vec![Bytes::from_static(&[0xbb; 20])],
        narinfo: Some(NarInfo {
            nar_size: 1024,
            nar_sha256: Bytes::from_static(&[0xcc; 32]),
            signatures: vec![],
            reference_names: vec!["test-ref".to_string()],
            deriver: None,
            ca: None,
        }),
    };
    let encoded = postcard::to_stdvec(&pi).unwrap();
    let decoded: PathInfo = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(pi, decoded);
}

#[test]
fn ca_hash_enum_roundtrip() {
    let ca = nar_info::Ca {
        r#type: nar_info::ca::Hash::NarBlake3 as i32,
        digest: Bytes::from_static(&[0xdd; 32]),
    };
    let encoded = postcard::to_stdvec(&ca).unwrap();
    let decoded: nar_info::Ca = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(ca, decoded);
}
