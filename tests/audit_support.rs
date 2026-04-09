use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Serialize;

const MAX_AUDIT_DEPTH: u32 = 64;
const MAX_AUDIT_ENTRIES: u32 = 100_000;

pub struct AuditArtifact<'a> {
    pub label: &'a str,
    pub path: &'a Path,
}

#[derive(Debug, Serialize)]
struct AuditBundle {
    schema: &'static str,
    test_name: String,
    step: String,
    cwd: String,
    command: Vec<String>,
    exit_code: Option<i32>,
    started_unix_s: u64,
    finished_unix_s: u64,
    env: BTreeMap<String, String>,
    artifacts: Vec<ArtifactDigest>,
}

#[derive(Debug, Serialize)]
struct ArtifactDigest {
    label: String,
    path: String,
    exists: bool,
    kind: Option<String>,
    digest_blake3: Option<String>,
    size_bytes: Option<u64>,
}

pub fn write_command_audit(
    test_name: &str,
    step: &str,
    cwd: &Path,
    command: &[String],
    output: &Output,
    artifacts: &[AuditArtifact<'_>],
    extra_env: &[(&str, String)],
) -> io::Result<PathBuf> {
    let started_unix_s = now_unix_s();
    let bundle_dir = audit_bundle_dir(test_name, step);
    fs::create_dir_all(&bundle_dir)?;
    fs::write(bundle_dir.join("stdout.txt"), &output.stdout)?;
    fs::write(bundle_dir.join("stderr.txt"), &output.stderr)?;

    let finished_unix_s = now_unix_s();
    let bundle = AuditBundle {
        schema: "crunch-test-audit-v1",
        test_name: test_name.to_string(),
        step: step.to_string(),
        cwd: cwd.display().to_string(),
        command: command.to_vec(),
        exit_code: output.status.code(),
        started_unix_s,
        finished_unix_s,
        env: collect_env(extra_env),
        artifacts: collect_artifacts(artifacts)?,
    };
    let meta_json = serde_json::to_vec_pretty(&bundle).map_err(io::Error::other)?;
    fs::write(bundle_dir.join("meta.json"), meta_json)?;
    Ok(bundle_dir)
}

fn collect_env(extra_env: &[(&str, String)]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "CRUNCH_STATE_DIR",
        "CRUNCH_LOG_DIR",
        "SNIX_BUILD_SANDBOX_SHELL",
        "TMPDIR",
        "CC",
        "PKG_CONFIG_PATH",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.to_string(), value);
        }
    }
    for (key, value) in extra_env {
        env.insert((*key).to_string(), value.clone());
    }
    env
}

fn collect_artifacts(artifacts: &[AuditArtifact<'_>]) -> io::Result<Vec<ArtifactDigest>> {
    let mut collected = Vec::new();
    for artifact in artifacts {
        collected.push(hash_artifact(artifact)?);
    }
    Ok(collected)
}

fn hash_artifact(artifact: &AuditArtifact<'_>) -> io::Result<ArtifactDigest> {
    if !artifact.path.exists() {
        return Ok(ArtifactDigest {
            label: artifact.label.to_string(),
            path: artifact.path.display().to_string(),
            exists: false,
            kind: None,
            digest_blake3: None,
            size_bytes: None,
        });
    }

    let mut hasher = blake3::Hasher::new();
    let mut size_bytes = 0u64;
    let mut entry_count = 0u32;
    hash_path_recursive(artifact.path, Path::new(""), &mut hasher, &mut size_bytes, &mut entry_count, 0)?;
    let kind = if artifact.path.is_dir() {
        Some("dir".to_string())
    } else if fs::symlink_metadata(artifact.path)?.file_type().is_symlink() {
        Some("symlink".to_string())
    } else {
        Some("file".to_string())
    };

    Ok(ArtifactDigest {
        label: artifact.label.to_string(),
        path: artifact.path.display().to_string(),
        exists: true,
        kind,
        digest_blake3: Some(hasher.finalize().to_hex().to_string()),
        size_bytes: Some(size_bytes),
    })
}

fn hash_path_recursive(
    abs_path: &Path,
    rel_path: &Path,
    hasher: &mut blake3::Hasher,
    size_bytes: &mut u64,
    entry_count: &mut u32,
    depth: u32,
) -> io::Result<()> {
    assert!(depth <= MAX_AUDIT_DEPTH, "audit depth exceeded");
    *entry_count = entry_count.saturating_add(1);
    assert!(*entry_count <= MAX_AUDIT_ENTRIES, "audit entry count exceeded");

    let metadata = fs::symlink_metadata(abs_path)?;
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(abs_path)?;
        update_hasher(
            hasher,
            b"symlink\0",
            rel_path.as_os_str().as_encoded_bytes(),
            target.as_os_str().as_encoded_bytes(),
        );
        return Ok(());
    }

    if metadata.is_file() {
        let bytes = fs::read(abs_path)?;
        *size_bytes = size_bytes.saturating_add(bytes.len() as u64);
        update_hasher(hasher, b"file\0", rel_path.as_os_str().as_encoded_bytes(), &bytes);
        return Ok(());
    }

    assert!(metadata.is_dir(), "audit hashing expects file, dir, or symlink");
    update_hasher(hasher, b"dir\0", rel_path.as_os_str().as_encoded_bytes(), &[]);

    let mut entries: Vec<PathBuf> = fs::read_dir(abs_path)?.filter_map(|entry| entry.ok().map(|e| e.path())).collect();
    entries.sort();
    for entry in entries {
        let name = entry.file_name().expect("entry path should have file name");
        let next_rel = if rel_path.as_os_str().is_empty() {
            PathBuf::from(name)
        } else {
            rel_path.join(name)
        };
        hash_path_recursive(&entry, &next_rel, hasher, size_bytes, entry_count, depth.saturating_add(1))?;
    }
    Ok(())
}

fn update_hasher(hasher: &mut blake3::Hasher, prefix: &[u8], rel_bytes: &[u8], payload: &[u8]) {
    hasher.update(prefix);
    hasher.update(&(rel_bytes.len() as u64).to_le_bytes());
    hasher.update(rel_bytes);
    hasher.update(&(payload.len() as u64).to_le_bytes());
    hasher.update(payload);
}

fn audit_bundle_dir(test_name: &str, step: &str) -> PathBuf {
    let unique = format!("{}-{}", std::process::id(), now_unix_ns());
    audit_root().join(sanitize(test_name)).join(format!("{}-{}", sanitize(step), unique))
}

fn audit_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-audit")
}

fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

fn now_unix_s() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0)
}

fn now_unix_ns() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0)
}
