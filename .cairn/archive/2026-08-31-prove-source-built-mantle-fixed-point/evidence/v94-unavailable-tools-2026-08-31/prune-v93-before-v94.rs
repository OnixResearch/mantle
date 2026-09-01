use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const RUN_ROOT: &str = "/home/brittonr/mantle-runs/receipt-fix-v31";
const V91_ROOT: &str = "/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v93-bound-linker-20260831.source-built-fixed-point-staging-3412854";
const RECEIPT: &str = "/home/brittonr/mantle-runs/receipt-fix-v31/cleanup-v93-before-v94.txt";
const RECEIPT_TMP: &str =
    "/home/brittonr/mantle-runs/receipt-fix-v31/.cleanup-v93-before-v94.txt.tmp";
const STAT: &str = "/run/current-system/sw/bin/stat";
const ENTRY_COUNT_MAX: u64 = 3_000_000;
const TREE_DEPTH_MAX: u32 = 256;
const OWNER_RWX_MODE_BITS: u32 = 0o700;
const ROOT_COUNT: usize = 1;

#[derive(Clone, Copy, Default)]
struct TreeStats {
    directories: u64,
    files: u64,
    symlinks: u64,
    bytes: u64,
}

fn main() -> Result<(), String> {
    if std::env::args_os().count() != 1 {
        return Err("cleanup tool takes no arguments".to_string());
    }
    let roots = [PathBuf::from(V91_ROOT)];
    validate_roots(&roots)?;
    let before_bytes = free_bytes(Path::new(RUN_ROOT))?;
    let mut stats = Vec::with_capacity(ROOT_COUNT);
    for root in &roots {
        stats.push(inspect_tree(root)?);
    }
    let mut adjusted_directories = Vec::with_capacity(ROOT_COUNT);
    for root in &roots {
        adjusted_directories.push(remove_tree(root)?);
    }
    for root in &roots {
        if root.exists() || fs::symlink_metadata(root).is_ok() {
            return Err(format!("cleanup root still exists: {}", root.display()));
        }
    }
    let after_bytes = free_bytes(Path::new(RUN_ROOT))?;
    if after_bytes <= before_bytes {
        return Err(format!(
            "cleanup did not increase free bytes: before={before_bytes} after={after_bytes}"
        ));
    }
    publish_receipt(
        &roots,
        &stats,
        &adjusted_directories,
        before_bytes,
        after_bytes,
    )?;
    println!("cleanup=complete before_bytes={before_bytes} after_bytes={after_bytes}");
    Ok(())
}

fn validate_roots(roots: &[PathBuf; ROOT_COUNT]) -> Result<(), String> {
    let run =
        fs::canonicalize(RUN_ROOT).map_err(|error| format!("canonicalize run root: {error}"))?;
    for root in roots {
        if !root.is_absolute() {
            return Err(format!("cleanup root is not absolute: {}", root.display()));
        }
        let parent = root
            .parent()
            .ok_or_else(|| format!("cleanup root has no parent: {}", root.display()))?;
        let canonical_parent =
            fs::canonicalize(parent).map_err(|error| format!("canonicalize parent: {error}"))?;
        if canonical_parent != run {
            return Err(format!("cleanup root escapes run root: {}", root.display()));
        }
        let metadata = fs::symlink_metadata(root)
            .map_err(|error| format!("inspect root {}: {error}", root.display()))?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(format!(
                "cleanup root is not a real directory: {}",
                root.display()
            ));
        }
    }
    for absent in [RECEIPT, RECEIPT_TMP] {
        if fs::symlink_metadata(absent).is_ok() {
            return Err(format!("cleanup receipt path already exists: {absent}"));
        }
    }
    Ok(())
}

fn inspect_tree(root: &Path) -> Result<TreeStats, String> {
    let mut stats = TreeStats::default();
    inspect_entry(root, 0, &mut stats)?;
    let entries = stats
        .directories
        .checked_add(stats.files)
        .and_then(|count| count.checked_add(stats.symlinks))
        .ok_or_else(|| "cleanup entry count overflow".to_string())?;
    if entries == 0 || entries > ENTRY_COUNT_MAX {
        return Err(format!(
            "cleanup entry count outside 1..={ENTRY_COUNT_MAX}: {entries}"
        ));
    }
    Ok(stats)
}

fn inspect_entry(path: &Path, depth: u32, stats: &mut TreeStats) -> Result<(), String> {
    if depth > TREE_DEPTH_MAX {
        return Err(format!(
            "cleanup tree depth exceeds {TREE_DEPTH_MAX}: {}",
            path.display()
        ));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect {}: {error}", path.display()))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        stats.symlinks = stats
            .symlinks
            .checked_add(1)
            .ok_or_else(|| "symlink count overflow".to_string())?;
        return Ok(());
    }
    if file_type.is_file() {
        stats.files = stats
            .files
            .checked_add(1)
            .ok_or_else(|| "file count overflow".to_string())?;
        stats.bytes = stats
            .bytes
            .checked_add(metadata.len())
            .ok_or_else(|| "byte count overflow".to_string())?;
        return Ok(());
    }
    if !file_type.is_dir() {
        return Err(format!(
            "cleanup tree contains unsupported entry: {}",
            path.display()
        ));
    }
    stats.directories = stats
        .directories
        .checked_add(1)
        .ok_or_else(|| "directory count overflow".to_string())?;
    let mut children = fs::read_dir(path)
        .map_err(|error| format!("read directory {}: {error}", path.display()))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    let child_depth = depth
        .checked_add(1)
        .ok_or_else(|| "cleanup depth overflow".to_string())?;
    for child in children {
        inspect_entry(&child, child_depth, stats)?;
    }
    Ok(())
}

fn remove_tree(root: &Path) -> Result<u64, String> {
    let mut adjusted_directories = 0_u64;
    remove_entry(root, 0, &mut adjusted_directories)?;
    Ok(adjusted_directories)
}

fn remove_entry(path: &Path, depth: u32, adjusted_directories: &mut u64) -> Result<(), String> {
    if depth > TREE_DEPTH_MAX {
        return Err(format!(
            "cleanup removal depth exceeds {TREE_DEPTH_MAX}: {}",
            path.display()
        ));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("reinspect {}: {error}", path.display()))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() || file_type.is_file() {
        return fs::remove_file(path)
            .map_err(|error| format!("remove file {}: {error}", path.display()));
    }
    if !file_type.is_dir() {
        return Err(format!(
            "cleanup entry changed to unsupported type: {}",
            path.display()
        ));
    }
    let mode = metadata.permissions().mode();
    if mode & OWNER_RWX_MODE_BITS != OWNER_RWX_MODE_BITS {
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode | OWNER_RWX_MODE_BITS);
        fs::set_permissions(path, permissions)
            .map_err(|error| format!("make directory removable {}: {error}", path.display()))?;
        *adjusted_directories = adjusted_directories
            .checked_add(1)
            .ok_or_else(|| "adjusted-directory count overflow".to_string())?;
    }
    let mut children = fs::read_dir(path)
        .map_err(|error| format!("read removal directory {}: {error}", path.display()))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    let child_depth = depth
        .checked_add(1)
        .ok_or_else(|| "cleanup removal depth overflow".to_string())?;
    for child in children {
        remove_entry(&child, child_depth, adjusted_directories)?;
    }
    fs::remove_dir(path).map_err(|error| format!("remove directory {}: {error}", path.display()))
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
        .ok_or_else(|| "stat omitted free blocks".to_string())?
        .parse::<u64>()
        .map_err(|error| format!("parse free blocks: {error}"))?;
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

fn publish_receipt(
    roots: &[PathBuf; ROOT_COUNT],
    stats: &[TreeStats],
    adjusted_directories: &[u64],
    before_bytes: u64,
    after_bytes: u64,
) -> Result<(), String> {
    if stats.len() != ROOT_COUNT || adjusted_directories.len() != ROOT_COUNT {
        return Err("cleanup receipt vectors have wrong length".to_string());
    }
    let finished_unix_s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock before epoch: {error}"))?
        .as_secs();
    let mut text = format!(
        "schema=mantle-failed-proof-cleanup-v1\nfinished_unix_s={finished_unix_s}\nreason=reclaim failed-proof staging after repository evidence preservation\nno_follow_preflight=true\nregular_file_modes_changed=false\nfree_bytes_before={before_bytes}\nfree_bytes_after={after_bytes}\n"
    );
    for (index, root) in roots.iter().enumerate() {
        let stat = stats[index];
        text.push_str(&format!(
            "root_{index}={}\nroot_{index}_directories={}\nroot_{index}_files={}\nroot_{index}_symlinks={}\nroot_{index}_apparent_bytes={}\nroot_{index}_directory_mode_adjustments={}\nroot_{index}_removed=true\n",
            root.display(), stat.directories, stat.files, stat.symlinks, stat.bytes, adjusted_directories[index]
        ));
    }
    text.push_str("evidence_v93_commit=3cccfc11\n");
    text.push_str("evidence_v93=cairn/changes/prove-source-built-mantle-fixed-point/evidence/v93-bound-linker-2026-08-31\n");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(RECEIPT_TMP)
        .map_err(|error| format!("create cleanup receipt temp: {error}"))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("write cleanup receipt: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync cleanup receipt: {error}"))?;
    fs::hard_link(RECEIPT_TMP, RECEIPT)
        .map_err(|error| format!("publish cleanup receipt without replacement: {error}"))?;
    fs::remove_file(RECEIPT_TMP)
        .map_err(|error| format!("remove cleanup receipt temp: {error}"))?;
    Ok(())
}
