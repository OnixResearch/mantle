#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
---

//! Bounded checked-in evidence size guard, reproducible storage inventory, and
//! digest-verified cold retrieval checker for `.cairn/` evidence.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};

const BUDGET_PATH: &str = "config/evidence-size-budget.json";
const EXPECTED_SCHEMA: &str = "mantle-evidence-size-budget-v1";

#[derive(Debug, Clone, Deserialize)]
struct EvidenceBudget {
    schema: String,
    default_max_file_bytes: u64,
    nix_filtered_cairn_max_bytes: u64,
    nix_required_cairn_roots: Vec<String>,
    exceptions: Vec<EvidenceException>,
}

#[derive(Debug, Clone, Deserialize)]
struct EvidenceException {
    path: String,
    size_bytes: u64,
    blake3: String,
    classification: String,
    justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EvidenceInventory {
    schema: &'static str,
    cairn_total_files: u64,
    cairn_total_bytes: u64,
    archive_files: u64,
    archive_bytes: u64,
    changes_files: u64,
    changes_bytes: u64,
    nix_filtered_cairn_files: u64,
    nix_filtered_cairn_bytes: u64,
    git_pack_bytes_note: String,
    exception_count: usize,
    non_claim: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExternalizedPayloadManifest {
    schema: &'static str,
    payloads: Vec<ExternalizedPayloadEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExternalizedPayloadEntry {
    relative_path: String,
    size_bytes: u64,
    blake3: String,
}

fn parse_args() -> Result<(PathBuf, bool, bool, bool), String> {
    let mut root = PathBuf::from(".");
    let mut is_self_test = false;
    let mut show_inventory = false;
    let mut is_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                let value = args.next().ok_or_else(|| "--root requires a path".to_string())?;
                root = PathBuf::from(value);
            }
            "--self-test" => is_self_test = true,
            "--inventory" => show_inventory = true,
            "--json" => is_json = true,
            "--help" | "-h" => {
                return Err("usage: check-evidence-size-budget.rs [--root <dir>] [--inventory [--json]] [--self-test]".to_string());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok((root, is_self_test, show_inventory, is_json))
}

fn load_budget(root: &Path) -> Result<EvidenceBudget, String> {
    let path = root.join(BUDGET_PATH);
    let raw = fs::read(&path).map_err(|err| format!("reading {}: {err}", path.display()))?;
    let budget: EvidenceBudget =
        serde_json::from_slice(&raw).map_err(|err| format!("parsing {}: {err}", path.display()))?;
    if budget.schema != EXPECTED_SCHEMA {
        return Err(format!("unexpected evidence budget schema `{}`", budget.schema));
    }
    if budget.default_max_file_bytes == 0 || budget.nix_filtered_cairn_max_bytes == 0 {
        return Err("evidence budget byte limits must be nonzero".to_string());
    }
    for exception in &budget.exceptions {
        if exception.justification.trim().is_empty() || exception.classification.trim().is_empty() {
            return Err(format!("exception {} must record classification and justification", exception.path));
        }
        if exception.blake3.len() != 64 || !exception.blake3.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("exception {} has invalid blake3 digest", exception.path));
        }
    }
    Ok(budget)
}

fn hash_file_blake3(path: &Path) -> Result<(u64, String), String> {
    let mut file = fs::File::open(path).map_err(|err| format!("opening {}: {err}", path.display()))?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let read = file.read(&mut buf).map_err(|err| format!("reading {}: {err}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
        total = total.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
    }
    Ok((total, hasher.finalize().to_hex().to_string()))
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<(String, u64)>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|err| format!("reading dir {}: {err}", dir.display()))?
        .map(|res| res.map(|e| e.path()).map_err(|err| format!("dir entry {}: {err}", dir.display())))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for path in entries {
        let meta = fs::symlink_metadata(&path).map_err(|err| format!("stat {}: {err}", path.display()))?;
        if meta.file_type().is_dir() {
            walk_files(root, &path, out)?;
        } else if meta.file_type().is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|err| format!("stripping prefix {}: {err}", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, meta.len()));
        }
    }
    Ok(())
}

fn check_repo_budget(root: &Path, budget: &EvidenceBudget) -> Result<EvidenceInventory, String> {
    let cairn_dir = root.join(".cairn");
    if !cairn_dir.is_dir() {
        return Err(format!("missing .cairn directory under {}", root.display()));
    }
    let mut files = Vec::new();
    walk_files(root, &cairn_dir, &mut files)?;

    let mut exception_map: BTreeMap<&str, &EvidenceException> = BTreeMap::new();
    for ex in &budget.exceptions {
        if exception_map.insert(ex.path.as_str(), ex).is_some() {
            return Err(format!("duplicate evidence exception {}", ex.path));
        }
    }

    let mut total_bytes = 0u64;
    let mut archive_files = 0u64;
    let mut archive_bytes = 0u64;
    let mut changes_files = 0u64;
    let mut changes_bytes = 0u64;
    let mut matched_exceptions = BTreeMap::new();

    for (rel, size) in &files {
        total_bytes = total_bytes.saturating_add(*size);
        if rel.starts_with(".cairn/archive/") {
            archive_files += 1;
            archive_bytes = archive_bytes.saturating_add(*size);
        } else if rel.starts_with(".cairn/changes/") {
            changes_files += 1;
            changes_bytes = changes_bytes.saturating_add(*size);
        }
        if let Some(ex) = exception_map.get(rel.as_str()) {
            let (observed_size, observed_hash) = hash_file_blake3(&root.join(rel))?;
            if observed_size != ex.size_bytes || observed_hash != ex.blake3 {
                return Err(format!(
                    "historical evidence exception {rel} drifted: expected ({} bytes, {}), observed ({observed_size} bytes, {observed_hash})",
                    ex.size_bytes, ex.blake3
                ));
            }
            matched_exceptions.insert(rel.as_str(), true);
        } else if *size > budget.default_max_file_bytes {
            return Err(format!(
                "evidence file {rel} ({size} bytes) exceeds default budget {} bytes without a justified exception in {BUDGET_PATH}",
                budget.default_max_file_bytes
            ));
        }
    }

    for ex in &budget.exceptions {
        if !matched_exceptions.contains_key(ex.path.as_str()) {
            return Err(format!("missing excepted historical evidence file {}", ex.path));
        }
    }

    let mut nix_filtered_files = 0u64;
    let mut nix_filtered_bytes = 0u64;
    for req_root in &budget.nix_required_cairn_roots {
        let dir = root.join(req_root);
        if !dir.is_dir() {
            return Err(format!("missing Nix-required cairn evidence directory {}", dir.display()));
        }
        let mut scoped = Vec::new();
        walk_files(root, &dir, &mut scoped)?;
        if scoped.is_empty() {
            return Err(format!("Nix-required cairn evidence directory {} has no files", dir.display()));
        }
        for (_, sz) in scoped {
            nix_filtered_files += 1;
            nix_filtered_bytes = nix_filtered_bytes.saturating_add(sz);
        }
    }
    if nix_filtered_bytes > budget.nix_filtered_cairn_max_bytes {
        return Err(format!(
            "Nix-filtered .cairn bytes ({nix_filtered_bytes}) exceed budget {}",
            budget.nix_filtered_cairn_max_bytes
        ));
    }

    Ok(EvidenceInventory {
        schema: "mantle-evidence-inventory-v1",
        cairn_total_files: u64::try_from(files.len()).unwrap_or(u64::MAX),
        cairn_total_bytes: total_bytes,
        archive_files,
        archive_bytes,
        changes_files,
        changes_bytes,
        nix_filtered_cairn_files: nix_filtered_files,
        nix_filtered_cairn_bytes: nix_filtered_bytes,
        git_pack_bytes_note: "measured with `git count-objects -vH` in docs/evidence-retention.md".to_string(),
        exception_count: budget.exceptions.len(),
        non_claim: "evidence inventory measures storage footprint only; historical archive files certify no current build proof",
    })
}

fn retrieve_and_verify_payload(
    manifest: &ExternalizedPayloadManifest,
    cold_store: &Path,
    relative_path: &str,
) -> Result<Vec<u8>, String> {
    let entry = manifest
        .payloads
        .iter()
        .find(|item| item.relative_path == relative_path)
        .ok_or_else(|| format!("manifest has no entry for {relative_path}"))?;
    let artifact = cold_store.join(&entry.blake3);
    if !artifact.is_file() {
        return Err(format!("missing cold evidence payload {} ({})", entry.relative_path, artifact.display()));
    }
    let bytes = fs::read(&artifact).map_err(|err| format!("reading cold payload {}: {err}", artifact.display()))?;
    let observed_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let observed_hash = blake3::hash(&bytes).to_hex().to_string();
    if observed_len != entry.size_bytes || observed_hash != entry.blake3 {
        return Err(format!(
            "corrupted cold payload {} failed digest verification: expected {} bytes {}, got {} bytes {}",
            entry.relative_path, entry.size_bytes, entry.blake3, observed_len, observed_hash
        ));
    }
    Ok(bytes)
}

fn run_self_tests() -> Result<(), String> {
    let temp = temp_dir("evidence-budget-self-test")?;
    let root = temp.path();

    // 1. Setup isolated budget & required root + oversized guard checks.
    fs::create_dir_all(root.join("config")).map_err(|e| e.to_string())?;
    let req_dir = root.join(".cairn/changes/separate-remote-build-hexagon/evidence");
    fs::create_dir_all(&req_dir).map_err(|e| e.to_string())?;
    fs::write(req_dir.join("receipt.json"), br#"{"ok":true}"#).map_err(|e| e.to_string())?;

    let archive_dir = root.join(".cairn/archive/2026-07-30-sample/evidence");
    fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;
    let excepted_bytes = vec![b'x'; 2_048];
    let excepted_hash = blake3::hash(&excepted_bytes).to_hex().to_string();
    fs::write(archive_dir.join("excepted.json"), &excepted_bytes).map_err(|e| e.to_string())?;

    let budget_json = serde_json::json!({
        "schema": EXPECTED_SCHEMA,
        "default_max_file_bytes": 1024,
        "nix_filtered_cairn_max_bytes": 4096,
        "nix_required_cairn_roots": [".cairn/changes/separate-remote-build-hexagon/evidence"],
        "exceptions": [{
            "path": ".cairn/archive/2026-07-30-sample/evidence/excepted.json",
            "size_bytes": 2048,
            "blake3": excepted_hash,
            "classification": "fixture-exception",
            "justification": "Isolated self-test justified fixture exception."
        }]
    });
    fs::write(root.join(BUDGET_PATH), serde_json::to_vec_pretty(&budget_json).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let budget = load_budget(root)?;

    // Justified exception + small required file passes and yields exact inventory.
    let inv = check_repo_budget(root, &budget)?;
    if inv.cairn_total_files != 2 || inv.nix_filtered_cairn_bytes != 11 {
        return Err(format!("unexpected self-test inventory: {inv:?}"));
    }

    // Newly added oversized file fails closed.
    let rogue = root.join(".cairn/changes/new-rogue.json");
    fs::write(&rogue, vec![b'y'; 1_500]).map_err(|e| e.to_string())?;
    let err = check_repo_budget(root, &budget).expect_err("oversized unapproved evidence must fail");
    if !err.contains("exceeds default budget") {
        return Err(format!("unexpected oversized error: {err}"));
    }
    fs::remove_file(&rogue).map_err(|e| e.to_string())?;

    // 2. Cold retrieval + digest verification + corrupted/missing fail-closed tests.
    let cold_store = root.join("cold-archive-store");
    fs::create_dir_all(&cold_store).map_err(|e| e.to_string())?;
    let payload = b"immutable-archived-transition-report";
    let digest = blake3::hash(payload).to_hex().to_string();
    fs::write(cold_store.join(&digest), payload).map_err(|e| e.to_string())?;
    let manifest = ExternalizedPayloadManifest {
        schema: "mantle-externalized-evidence-v1",
        payloads: vec![ExternalizedPayloadEntry {
            relative_path: ".cairn/archive/sample/report.json".to_string(),
            size_bytes: u64::try_from(payload.len()).unwrap(),
            blake3: digest.clone(),
        }],
    };
    let retrieved = retrieve_and_verify_payload(&manifest, &cold_store, ".cairn/archive/sample/report.json")?;
    if retrieved.as_slice() != payload {
        return Err("retrieved cold payload differed".to_string());
    }
    fs::write(cold_store.join(&digest), b"corrupted").map_err(|e| e.to_string())?;
    let corrupt_err =
        retrieve_and_verify_payload(&manifest, &cold_store, ".cairn/archive/sample/report.json").unwrap_err();
    if !corrupt_err.contains("failed digest verification") {
        return Err(format!("unexpected corrupt error: {corrupt_err}"));
    }
    fs::remove_file(cold_store.join(&digest)).map_err(|e| e.to_string())?;
    let missing_err =
        retrieve_and_verify_payload(&manifest, &cold_store, ".cairn/archive/sample/report.json").unwrap_err();
    if !missing_err.contains("missing cold evidence payload") {
        return Err(format!("unexpected missing error: {missing_err}"));
    }
    Ok(())
}

struct TempDirGuard(PathBuf);
impl TempDirGuard {
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(prefix: &str) -> Result<TempDirGuard, String> {
    let base = env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
    if base.exists() {
        fs::remove_dir_all(&base).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    Ok(TempDirGuard(base))
}

fn run() -> Result<(), String> {
    let (root, is_self_test, show_inventory, is_json) = parse_args()?;
    let budget = load_budget(&root)?;
    let inventory = check_repo_budget(&root, &budget)?;
    if is_self_test {
        run_self_tests()?;
    }
    if show_inventory && is_json {
        println!("{}", serde_json::to_string_pretty(&inventory).map_err(|e| e.to_string())?);
        return Ok(());
    }
    println!(
        "evidence size budget: PASS (cairn_files={}, cairn_bytes={}, nix_filtered_cairn_bytes={}, exceptions={})",
        inventory.cairn_total_files,
        inventory.cairn_total_bytes,
        inventory.nix_filtered_cairn_bytes,
        inventory.exception_count
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("evidence size budget check failed: {error}");
            ExitCode::FAILURE
        }
    }
}
