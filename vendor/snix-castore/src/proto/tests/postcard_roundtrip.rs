use super::super::*;
use bytes::Bytes;

#[test]
fn directory_roundtrip() {
    let dir = Directory {
        directories: vec![DirectoryEntry {
            name: Bytes::from_static(b"subdir"),
            digest: Bytes::from_static(&[0xab; 32]),
            size: 42,
        }],
        files: vec![FileEntry {
            name: Bytes::from_static(b"hello.txt"),
            digest: Bytes::from_static(&[0xcd; 32]),
            size: 13,
            executable: false,
        }],
        symlinks: vec![SymlinkEntry {
            name: Bytes::from_static(b"link"),
            target: Bytes::from_static(b"/nix/store/foo"),
        }],
    };
    let encoded = postcard::to_stdvec(&dir).unwrap();
    let decoded: Directory = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(dir, decoded);
}

#[test]
fn entry_roundtrip() {
    let entry = Entry {
        entry: Some(entry::Entry::File(FileEntry {
            name: Bytes::from_static(b"test"),
            digest: Bytes::from_static(&[0xff; 32]),
            size: 100,
            executable: true,
        })),
    };
    let encoded = postcard::to_stdvec(&entry).unwrap();
    let decoded: Entry = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(entry, decoded);
}

#[test]
fn stat_blob_response_roundtrip() {
    let resp = StatBlobResponse {
        chunks: vec![stat_blob_response::ChunkMeta {
            digest: Bytes::from_static(&[0x11; 32]),
            size: 65536,
        }],
        bao: Bytes::new(),
    };
    let encoded = postcard::to_stdvec(&resp).unwrap();
    let decoded: StatBlobResponse = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn empty_directory_roundtrip() {
    let dir = Directory::default();
    let encoded = postcard::to_stdvec(&dir).unwrap();
    let decoded: Directory = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(dir, decoded);
}
