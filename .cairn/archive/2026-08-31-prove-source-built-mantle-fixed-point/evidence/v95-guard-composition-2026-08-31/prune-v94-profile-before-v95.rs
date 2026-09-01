use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const AUTHORITY_ROOT: &str = "/home/brittonr/mantle-runs/receipt-fix-v31/authority";
const OLD_PROFILE: &str = "/home/brittonr/mantle-runs/receipt-fix-v31/authority/source-built-fixed-point-sources-v94-3cccfc11.json";
const NEW_PROFILE: &str = "/home/brittonr/mantle-runs/receipt-fix-v31/authority/source-built-fixed-point-sources-v95-433b9fb3.json";
const NEW_VERIFY: &str =
    "/home/brittonr/mantle-runs/receipt-fix-v31/authority/source-profile-v95-verification.log";
const RECEIPT: &str =
    "/home/brittonr/mantle-runs/receipt-fix-v31/cleanup-v94-profile-before-v95.txt";
const STAT: &str = "/run/current-system/sw/bin/stat";
const EXPECTED_PROFILE_DIGEST: &str =
    "d1853c44dabafc474686632d300ed7b79e2c9f670b4f21ebbdf0976428c002b3";

fn main() -> Result<(), String> {
    if std::env::args_os().count() != 1 {
        return Err("cleanup tool takes no arguments".to_string());
    }
    validate_regular_file(Path::new(OLD_PROFILE))?;
    validate_regular_file(Path::new(NEW_PROFILE))?;
    let verify = fs::read_to_string(NEW_VERIFY)
        .map_err(|error| format!("read V95 verification: {error}"))?;
    if !verify.contains(EXPECTED_PROFILE_DIGEST) || !verify.contains("readiness=Ready") {
        return Err("V93 replacement profile is not verified Ready".to_string());
    }
    if fs::symlink_metadata(RECEIPT).is_ok() {
        return Err("cleanup receipt already exists".to_string());
    }
    let old_size = fs::symlink_metadata(OLD_PROFILE)
        .map_err(|error| format!("inspect old profile: {error}"))?
        .len();
    if old_size == 0 {
        return Err("old profile is empty".to_string());
    }
    let before_bytes = free_bytes(Path::new(AUTHORITY_ROOT))?;
    fs::remove_file(OLD_PROFILE).map_err(|error| format!("remove old profile: {error}"))?;
    if fs::symlink_metadata(OLD_PROFILE).is_ok() {
        return Err("old profile still exists".to_string());
    }
    let after_bytes = free_bytes(Path::new(AUTHORITY_ROOT))?;
    if after_bytes <= before_bytes {
        return Err("cleanup did not increase free bytes".to_string());
    }
    publish_receipt(old_size, before_bytes, after_bytes)?;
    println!("cleanup=complete before_bytes={before_bytes} after_bytes={after_bytes}");
    Ok(())
}

fn validate_regular_file(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("profile path is not absolute: {}", path.display()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| "profile path has no parent".to_string())?;
    let canonical_parent =
        fs::canonicalize(parent).map_err(|error| format!("canonicalize parent: {error}"))?;
    let canonical_authority = fs::canonicalize(AUTHORITY_ROOT)
        .map_err(|error| format!("canonicalize authority root: {error}"))?;
    if canonical_parent != canonical_authority {
        return Err(format!(
            "profile escapes authority root: {}",
            path.display()
        ));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "profile is not a regular no-follow file: {}",
            path.display()
        ));
    }
    Ok(())
}

fn free_bytes(path: &Path) -> Result<u64, String> {
    let output = Command::new(STAT)
        .args(["-f", "-c", "%a %S"])
        .arg(path)
        .output()
        .map_err(|error| format!("run stat: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "stat failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|error| format!("stat output is not UTF-8: {error}"))?;
    let mut fields = text.split_whitespace();
    let blocks = fields
        .next()
        .ok_or_else(|| "stat omitted blocks".to_string())?
        .parse::<u64>()
        .map_err(|error| format!("parse blocks: {error}"))?;
    let block_size = fields
        .next()
        .ok_or_else(|| "stat omitted block size".to_string())?
        .parse::<u64>()
        .map_err(|error| format!("parse block size: {error}"))?;
    if fields.next().is_some() {
        return Err("stat returned extra fields".to_string());
    }
    blocks
        .checked_mul(block_size)
        .ok_or_else(|| "free-byte count overflow".to_string())
}

fn publish_receipt(old_size: u64, before_bytes: u64, after_bytes: u64) -> Result<(), String> {
    let finished_unix_s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock before epoch: {error}"))?
        .as_secs();
    let text = format!(
        "schema=mantle-source-profile-cleanup-v1\nfinished_unix_s={finished_unix_s}\nold_profile={OLD_PROFILE}\nold_profile_size={old_size}\nold_profile_regular_no_follow=true\nreplacement_profile={NEW_PROFILE}\nreplacement_manifest_blake3={EXPECTED_PROFILE_DIGEST}\nreplacement_readiness=Ready\nfree_bytes_before={before_bytes}\nfree_bytes_after={after_bytes}\nregular_file_modes_changed=false\nremoved=true\n"
    );
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(RECEIPT)
        .map_err(|error| format!("create cleanup receipt: {error}"))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("write cleanup receipt: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync cleanup receipt: {error}"))?;
    Ok(())
}
