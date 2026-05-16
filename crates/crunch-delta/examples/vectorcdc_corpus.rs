use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

const DEFAULT_OUTPUT_DIR: &str = "target/vectorcdc-corpus";
const MUTATION_WINDOW: usize = 64;
const MUTATION_INSERT: &[u8] = b"\n# vectorcdc-corpus deterministic insertion\n";
const MUTATION_APPEND_BYTES: usize = 4096;

#[derive(Debug)]
struct SourceCandidate {
    id: &'static str,
    kind: &'static str,
    path: PathBuf,
    required: bool,
}

#[derive(Debug)]
struct CorpusEntry {
    id: String,
    kind: &'static str,
    source_id: String,
    relative_path: String,
    source_path: String,
    mutation: &'static str,
    bytes: usize,
    blake3: String,
}

fn main() -> io::Result<()> {
    let output_dir = env::args().nth(1).unwrap_or_else(|| DEFAULT_OUTPUT_DIR.to_string());
    let repo_root = repo_root();
    let output_dir = repo_root.join(output_dir);

    let mut entries = Vec::new();
    let corpus_dir = output_dir.join("files");
    fs::create_dir_all(&corpus_dir)?;

    for candidate in source_candidates(&repo_root) {
        match fs::read(&candidate.path) {
            Ok(bytes) => {
                add_entry(&mut entries, &corpus_dir, &candidate, "base", bytes.clone())?;
                add_entry(&mut entries, &corpus_dir, &candidate, "middle-64-byte-patch", patch_middle(bytes.clone()))?;
                add_entry(&mut entries, &corpus_dir, &candidate, "prefix-insert", prefix_insert(bytes.clone()))?;
                add_entry(&mut entries, &corpus_dir, &candidate, "tail-append", tail_append(bytes))?;
            }
            Err(error) if candidate.required => return Err(error),
            Err(_) => {}
        }
    }

    if entries.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "no corpus sources were available"));
    }

    let manifest = render_manifest(&repo_root, &output_dir, &entries);
    fs::write(output_dir.join("manifest.json"), manifest)?;

    println!("wrote {} corpus entries to {}", entries.len(), output_dir.display());
    println!("manifest={}", output_dir.join("manifest.json").display());
    Ok(())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn source_candidates(repo_root: &Path) -> Vec<SourceCandidate> {
    let mut candidates = vec![
        SourceCandidate {
            id: "cargo-lock",
            kind: "repo-artifact",
            path: repo_root.join("Cargo.lock"),
            required: true,
        },
        SourceCandidate {
            id: "castore-object-store-rs",
            kind: "repo-artifact",
            path: repo_root.join("vendor/snix-castore/src/blobservice/object_store.rs"),
            required: true,
        },
        SourceCandidate {
            id: "castore-chunker-rs",
            kind: "repo-artifact",
            path: repo_root.join("vendor/snix-castore/src/blobservice/chunker.rs"),
            required: true,
        },
    ];

    if let Some(path) = newest_matching(&repo_root.join("target/debug/deps"), "libsnix_castore", Some("rlib")) {
        candidates.push(SourceCandidate {
            id: "snix-castore-rlib",
            kind: "local-build-artifact",
            path,
            required: false,
        });
    }

    if let Some(path) = newest_matching(&repo_root.join("target/debug/deps"), "snix_castore", Some("d")) {
        candidates.push(SourceCandidate {
            id: "snix-castore-depfile",
            kind: "local-build-artifact",
            path,
            required: false,
        });
    }

    candidates
}

fn newest_matching(dir: &Path, prefix: &str, extension: Option<&str>) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    let entries = fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if !name.starts_with(prefix) {
            continue;
        }
        if let Some(expected) = extension
            && path.extension().and_then(OsStr::to_str) != Some(expected)
        {
            continue;
        }
        let modified = entry.metadata().and_then(|metadata| metadata.modified()).unwrap_or(std::time::UNIX_EPOCH);
        match &best {
            Some((best_modified, _)) if *best_modified >= modified => {}
            _ => best = Some((modified, path)),
        }
    }

    best.map(|(_, path)| path)
}

fn add_entry(
    entries: &mut Vec<CorpusEntry>,
    corpus_dir: &Path,
    source: &SourceCandidate,
    mutation: &'static str,
    bytes: Vec<u8>,
) -> io::Result<()> {
    let id = format!("{}--{}", source.id, mutation);
    let relative_path = format!("files/{}", sanitize_file_name(&id));
    let output_path = corpus_dir.join(sanitize_file_name(&id));
    fs::write(output_path, &bytes)?;

    entries.push(CorpusEntry {
        id,
        kind: source.kind,
        source_id: source.id.to_string(),
        relative_path,
        source_path: source.path.display().to_string(),
        mutation,
        bytes: bytes.len(),
        blake3: blake3::hash(&bytes).to_hex().to_string(),
    });

    Ok(())
}

fn patch_middle(mut bytes: Vec<u8>) -> Vec<u8> {
    if bytes.is_empty() {
        return b"vectorcdc-corpus-empty-patch".to_vec();
    }
    let start = bytes.len() / 2;
    let end = start.saturating_add(MUTATION_WINDOW).min(bytes.len());
    for (index, byte) in bytes[start..end].iter_mut().enumerate() {
        *byte ^= (index as u8).wrapping_mul(31).wrapping_add(0x5a);
    }
    bytes
}

fn prefix_insert(bytes: Vec<u8>) -> Vec<u8> {
    let mut mutated = Vec::with_capacity(bytes.len().saturating_add(MUTATION_INSERT.len()));
    let split = bytes.len().min(4096);
    mutated.extend_from_slice(&bytes[..split]);
    mutated.extend_from_slice(MUTATION_INSERT);
    mutated.extend_from_slice(&bytes[split..]);
    mutated
}

fn tail_append(mut bytes: Vec<u8>) -> Vec<u8> {
    bytes.reserve(MUTATION_APPEND_BYTES);
    for index in 0..MUTATION_APPEND_BYTES {
        bytes.push((index as u8).wrapping_mul(17).wrapping_add(0xa5));
    }
    bytes
}

fn sanitize_file_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn render_manifest(repo_root: &Path, output_dir: &Path, entries: &[CorpusEntry]) -> String {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"schema\": \"crunch.vectorcdc-corpus.v1\",\n");
    json.push_str("  \"purpose\": \"Representative CDC benchmark corpus: repo artifacts, local build artifacts, and deterministic small-delta mutations.\",\n");
    json.push_str(&format!("  \"repo_root\": \"{}\",\n", escape_json(&repo_root.display().to_string())));
    json.push_str(&format!("  \"output_dir\": \"{}\",\n", escape_json(&output_dir.display().to_string())));
    json.push_str("  \"identity_invariant\": \"Corpus files are benchmark inputs only; blob identity remains BLAKE3(raw bytes).\",\n");
    json.push_str("  \"mutation_policy\": {\n");
    json.push_str(
        "    \"middle-64-byte-patch\": \"XOR up to 64 bytes at the midpoint with a deterministic pattern.\",\n",
    );
    json.push_str(
        "    \"prefix-insert\": \"Insert a fixed marker after the first 4096 bytes or at EOF for smaller files.\",\n",
    );
    json.push_str("    \"tail-append\": \"Append 4096 deterministic bytes.\"\n");
    json.push_str("  },\n");
    json.push_str("  \"entries\": [\n");
    for (index, entry) in entries.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"id\": \"{}\",\n", escape_json(&entry.id)));
        json.push_str(&format!("      \"kind\": \"{}\",\n", escape_json(entry.kind)));
        json.push_str(&format!("      \"source_id\": \"{}\",\n", escape_json(&entry.source_id)));
        json.push_str(&format!("      \"relative_path\": \"{}\",\n", escape_json(&entry.relative_path)));
        json.push_str(&format!("      \"source_path\": \"{}\",\n", escape_json(&entry.source_path)));
        json.push_str(&format!("      \"mutation\": \"{}\",\n", escape_json(entry.mutation)));
        json.push_str(&format!("      \"bytes\": {},\n", entry.bytes));
        json.push_str(&format!("      \"blake3\": \"{}\"\n", entry.blake3));
        if index + 1 == entries.len() {
            json.push_str("    }\n");
        } else {
            json.push_str("    },\n");
        }
    }
    json.push_str("  ]\n");
    json.push_str("}\n");
    json
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}
