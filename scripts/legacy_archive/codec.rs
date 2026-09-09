//! Protocol adapter: reuse the repository's NAR decoder and postcard wire shape.
use super::Result;
use nix_compat::{narinfo::Signature, nixhash::CAHash, store_path::StorePath};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub const MAGIC: &[u8] = b"mantle-store-archive-v1\n";
const MAX_METADATA_BYTES: usize = 1_048_576;

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "frame", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Frame {
    Header(Header),
    Path(Box<PathFrame>),
    End(End),
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub format: String,
    pub version: u32,
    pub store_prefix: String,
    pub record_count: u32,
    pub roots: Vec<String>,
    pub compatibility: String,
    pub payload_len: u64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathFrame {
    pub path_info: PathInfo,
    pub root: bool,
    pub payload_len: u64,
    pub payload_blake3: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct End {
    pub record_count: u32,
    pub total_payload_bytes: u64,
    pub payload_len: u64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathInfo {
    pub store_path: StorePath<String>,
    pub node: Node,
    pub references: Vec<StorePath<String>>,
    pub nar_size: u64,
    pub nar_sha256: [u8; 32],
    pub signatures: Vec<Signature<String>>,
    pub deriver: Option<StorePath<String>>,
    pub ca: Option<CAHash>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Node {
    Directory {
        digest: [u8; 32],
        size: u64,
    },
    File {
        digest: [u8; 32],
        size: u64,
        executable: bool,
    },
    Symlink {
        target: Vec<u8>,
    },
}

pub fn read_frame(reader: &mut impl Read) -> Result<Frame> {
    let mut word = [0u8; 4];
    reader.read_exact(&mut word)?;
    let len = u32::from_le_bytes(word) as usize;
    if len == 0 || len > MAX_METADATA_BYTES {
        return Err("metadata length outside bounds".into());
    }
    let mut bytes = vec![0; len];
    reader.read_exact(&mut bytes)?;
    let frame: Frame = serde_json::from_slice(&bytes)?;
    let original: serde_json::Value = serde_json::from_slice(&bytes)?;
    if serde_json::to_value(&frame)? != original {
        return Err("metadata would lose or normalize non-node fields".into());
    }
    Ok(frame)
}
pub fn write_frame(writer: &mut impl Write, frame: &Frame) -> Result<()> {
    let bytes = serde_json::to_vec(frame)?;
    if bytes.len() > MAX_METADATA_BYTES {
        return Err("metadata output bound".into());
    }
    writer.write_all(&u32::try_from(bytes.len())?.to_le_bytes())?;
    writer.write_all(&bytes)?;
    Ok(())
}

// This exact field order is the current snix.castore.v1 postcard representation.
#[derive(Default, Serialize)]
struct Directory {
    directories: Vec<DirEntry>,
    files: Vec<FileEntry>,
    symlinks: Vec<SymEntry>,
}
#[derive(Serialize)]
struct DirEntry {
    name: Vec<u8>,
    digest: Vec<u8>,
    size: u64,
}
#[derive(Serialize)]
struct FileEntry {
    name: Vec<u8>,
    digest: Vec<u8>,
    size: u64,
    executable: bool,
}
#[derive(Serialize)]
struct SymEntry {
    name: Vec<u8>,
    target: Vec<u8>,
}

impl Directory {
    fn push(&mut self, name: Vec<u8>, node: Node) {
        match node {
            Node::Directory { digest, size } => self.directories.push(DirEntry {
                name,
                digest: digest.to_vec(),
                size,
            }),
            Node::File {
                digest,
                size,
                executable,
            } => self.files.push(FileEntry {
                name,
                digest: digest.to_vec(),
                size,
                executable,
            }),
            Node::Symlink { target } => self.symlinks.push(SymEntry { name, target }),
        }
    }
    fn identity(&self, size: u64) -> Result<Node> {
        Ok(Node::Directory {
            digest: *blake3::hash(&postcard::to_stdvec(self)?).as_bytes(),
            size,
        })
    }
}

pub fn observe_nar(bytes: &[u8]) -> Result<(Node, Node)> {
    let mut reader = std::io::Cursor::new(bytes);
    let node = nix_compat::nar::reader::open(&mut reader)?;
    let mut remaining = 131_072u32;
    let result = observe_node(node, 0, &mut remaining)?;
    if reader.position() != bytes.len() as u64 {
        return Err("trailing NAR bytes".into());
    }
    Ok(result)
}
fn observe_node(node: nix_compat::nar::reader::Node<'_, '_>, depth: u32, remaining: &mut u32) -> Result<(Node, Node)> {
    if depth > 64 || *remaining == 0 {
        return Err("NAR tree bound".into());
    }
    *remaining -= 1;
    match node {
        nix_compat::nar::reader::Node::Symlink { target } => {
            let n = Node::Symlink { target };
            Ok((n.clone(), n))
        }
        nix_compat::nar::reader::Node::File { executable, mut reader } => {
            let size = reader.len();
            let mut hash = blake3::Hasher::new();
            reader.copy(&mut hash)?;
            let n = Node::File {
                digest: *hash.finalize().as_bytes(),
                size,
                executable,
            };
            Ok((n.clone(), n))
        }
        nix_compat::nar::reader::Node::Directory(mut reader) => {
            let mut current = Directory::default();
            let mut legacy = Directory::default();
            let mut size = 0u64;
            while let Some(entry) = reader.next()? {
                let (new, old) = observe_node(entry.node, depth + 1, remaining)?;
                let child_size = match &new {
                    Node::Directory { size, .. } => *size,
                    _ => 0,
                };
                size = size.checked_add(1 + child_size).ok_or("directory size overflow")?;
                current.push(entry.name.to_vec(), new);
                legacy.push(entry.name.to_vec(), old);
            }
            // Legacy bug counted all child entries twice, not only directories.
            Ok((current.identity(size)?, legacy.identity(size.checked_mul(2).ok_or("legacy size overflow")?)?))
        }
    }
}
