use std::collections::BTreeSet;
use std::fs::File;
use std::io;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::errors::RunError;
use crate::self_build::require_checked_vendor_inputs;

const MAX_TRACKED_SOURCE_PATHS: u32 = 200_000;
const MAX_VENDOR_SOURCE_PATHS: u32 = 200_000;
const MAX_PARENT_DIRS: u32 = 200_000;
const TAR_MODE_DIR_DEFAULT: u32 = 0o755;
const TAR_MODE_SYMLINK_DEFAULT: u32 = 0o777;
const RELEASE_SOURCE_SKIP_ANYWHERE: &[&str] = &[".agent", ".git", ".jj", ".pi", "target"];
const RELEASE_SOURCE_SKIP_AT_ROOT: &[&str] = &["cairn"];

pub(crate) fn write_tracked_source_archive(repo_root: &Path, archive_path: &Path) -> Result<(), RunError> {
    assert!(repo_root.is_dir(), "repo root must exist: {}", repo_root.display());
    require_checked_vendor_inputs(repo_root)?;
    let source_paths = source_archive_paths(repo_root)?;
    if source_paths.is_empty() {
        return Err(RunError::Internal(format!("source archive found no files under {}", repo_root.display())));
    }

    let archive_parent = archive_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("archive path has no parent: {}", archive_path.display())))?;
    std::fs::create_dir_all(archive_parent)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", archive_parent.display())))?;
    let archive_file = File::create(archive_path)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", archive_path.display())))?;
    let mut builder = tar::Builder::new(archive_file);
    append_source_paths_to_archive(&mut builder, repo_root, &source_paths)?;
    builder
        .finish()
        .map_err(|err| RunError::Internal(format!("finalizing {}: {err}", archive_path.display())))?;
    Ok(())
}

fn source_archive_paths(repo_root: &Path) -> Result<Vec<PathBuf>, RunError> {
    let mut paths: BTreeSet<PathBuf> = tracked_source_paths(repo_root)?.into_iter().collect();
    for vendor_path in vendored_source_paths(repo_root)? {
        paths.insert(vendor_path);
    }
    Ok(paths.into_iter().collect())
}

fn tracked_source_paths(repo_root: &Path) -> Result<Vec<PathBuf>, RunError> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("-z")
        .current_dir(repo_root)
        .output()
        .map_err(|err| RunError::Internal(format!("running git ls-files in {}: {err}", repo_root.display())))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RunError::Internal(format!("git ls-files failed in {}: {}", repo_root.display(), stderr.trim())));
    }

    let mut tracked = Vec::new();
    for raw_path in output.stdout.split(|byte| *byte == 0) {
        if raw_path.is_empty() {
            continue;
        }
        let relative_path = PathBuf::from(os_string_from_git_bytes(raw_path)?);
        if !is_releasable_tracked_source_path(&relative_path) {
            continue;
        }
        let tracked_count_u32: u32 = tracked
            .len()
            .try_into()
            .map_err(|_| RunError::Internal("tracked source path count overflowed u32".to_string()))?;
        if tracked_count_u32 >= MAX_TRACKED_SOURCE_PATHS {
            return Err(RunError::Internal(format!("tracked source path count exceeded {MAX_TRACKED_SOURCE_PATHS}")));
        }
        tracked.push(relative_path);
    }
    tracked.sort();
    Ok(tracked)
}

fn vendored_source_paths(repo_root: &Path) -> Result<Vec<PathBuf>, RunError> {
    let vendor_dir = repo_root.join("vendor-deps");
    if !vendor_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    collect_vendor_source_paths(repo_root, &vendor_dir, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_vendor_source_paths(repo_root: &Path, current_dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let mut children = Vec::new();
    let entries = std::fs::read_dir(current_dir)
        .map_err(|err| RunError::Internal(format!("read_dir {}: {err}", current_dir.display())))?;
    for entry in entries {
        let entry =
            entry.map_err(|err| RunError::Internal(format!("read_dir entry {}: {err}", current_dir.display())))?;
        children.push(entry.path());
    }
    children.sort();
    for child in &children {
        collect_vendor_source_child(repo_root, child, paths)?;
    }
    Ok(())
}

fn collect_vendor_source_child(repo_root: &Path, child: &Path, paths: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(child)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", child.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(RunError::Internal(format!("vendor-deps source archive path is a symlink: {}", child.display())));
    }
    if metadata.is_dir() {
        return collect_vendor_source_paths(repo_root, child, paths);
    }
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("vendor-deps source archive path is unsupported: {}", child.display())));
    }
    let relative = child.strip_prefix(repo_root).map_err(|err| {
        RunError::Internal(format!("strip repo prefix {} from {}: {err}", repo_root.display(), child.display()))
    })?;
    let path_count_u32: u32 = paths
        .len()
        .try_into()
        .map_err(|_| RunError::Internal("vendor source path count overflowed u32".to_string()))?;
    if path_count_u32 >= MAX_VENDOR_SOURCE_PATHS {
        return Err(RunError::Internal(format!("vendor source path count exceeded {MAX_VENDOR_SOURCE_PATHS}")));
    }
    paths.push(relative.to_path_buf());
    Ok(())
}

fn is_releasable_tracked_source_path(relative_path: &Path) -> bool {
    assert!(!relative_path.is_absolute(), "source path must be relative");
    assert!(!relative_path.as_os_str().is_empty(), "source path must not be empty");

    let mut first_component = None;
    for component in relative_path.components() {
        let std::path::Component::Normal(component_name) = component else {
            return false;
        };
        if first_component.is_none() {
            first_component = Some(component_name);
        }
        let Some(name) = component_name.to_str() else {
            return false;
        };
        if RELEASE_SOURCE_SKIP_ANYWHERE.contains(&name) {
            return false;
        }
    }

    let Some(first_component) = first_component else {
        return false;
    };
    let Some(first_name) = first_component.to_str() else {
        return false;
    };
    !RELEASE_SOURCE_SKIP_AT_ROOT.contains(&first_name)
}

fn append_source_paths_to_archive(
    builder: &mut tar::Builder<File>,
    repo_root: &Path,
    tracked_paths: &[PathBuf],
) -> Result<(), RunError> {
    let mut appended_dirs = BTreeSet::new();
    for relative_path in tracked_paths {
        append_parent_dirs(builder, repo_root, relative_path, &mut appended_dirs)?;
        append_single_path(builder, repo_root, relative_path)?;
    }
    Ok(())
}

fn append_parent_dirs(
    builder: &mut tar::Builder<File>,
    repo_root: &Path,
    relative_path: &Path,
    appended_dirs: &mut BTreeSet<PathBuf>,
) -> Result<(), RunError> {
    let mut parent_paths = Vec::new();
    let mut cursor = relative_path.parent();
    while let Some(parent) = cursor {
        if parent.as_os_str().is_empty() {
            break;
        }
        parent_paths.push(parent.to_path_buf());
        cursor = parent.parent();
    }
    parent_paths.reverse();

    for parent_path in parent_paths {
        let dir_count_u32: u32 = appended_dirs
            .len()
            .try_into()
            .map_err(|_| RunError::Internal("archive parent dir count overflowed u32".to_string()))?;
        if dir_count_u32 >= MAX_PARENT_DIRS {
            return Err(RunError::Internal(format!("archive parent dir count exceeded {MAX_PARENT_DIRS}")));
        }
        if !appended_dirs.insert(parent_path.clone()) {
            continue;
        }
        append_directory_entry(builder, &repo_root.join(&parent_path), &parent_path)?;
    }
    Ok(())
}

fn append_single_path(
    builder: &mut tar::Builder<File>,
    repo_root: &Path,
    relative_path: &Path,
) -> Result<(), RunError> {
    let source_path = repo_root.join(relative_path);
    let metadata = std::fs::symlink_metadata(&source_path)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", source_path.display())))?;

    if metadata.file_type().is_symlink() {
        return append_symlink_entry(builder, &source_path, relative_path, &metadata);
    }
    if metadata.is_file() {
        return append_file_entry(builder, &source_path, relative_path, &metadata);
    }
    if metadata.is_dir() {
        return append_directory_entry(builder, &source_path, relative_path);
    }
    Err(RunError::Internal(format!("unsupported source archive entry type: {}", source_path.display())))
}

fn append_directory_entry(
    builder: &mut tar::Builder<File>,
    source_path: &Path,
    relative_path: &Path,
) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(source_path)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", source_path.display())))?;
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Directory);
    header.set_size(0);
    header.set_mode(mode_bits_or_default(&metadata, TAR_MODE_DIR_DEFAULT));
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    let tar_path = tar_directory_path(relative_path)?;
    builder
        .append_data(&mut header, tar_path, io::empty())
        .map_err(|err| RunError::Internal(format!("appending dir {}: {err}", source_path.display())))?;
    Ok(())
}

fn append_file_entry(
    builder: &mut tar::Builder<File>,
    source_path: &Path,
    relative_path: &Path,
    metadata: &std::fs::Metadata,
) -> Result<(), RunError> {
    let mut file = File::open(source_path)
        .map_err(|err| RunError::Internal(format!("opening {}: {err}", source_path.display())))?;
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_size(metadata.len());
    header.set_mode(mode_bits_or_default(metadata, 0o644));
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    let tar_path = tar_path_string(relative_path)?;
    builder
        .append_data(&mut header, tar_path, &mut file)
        .map_err(|err| RunError::Internal(format!("appending file {}: {err}", source_path.display())))?;
    Ok(())
}

fn append_symlink_entry(
    builder: &mut tar::Builder<File>,
    source_path: &Path,
    relative_path: &Path,
    metadata: &std::fs::Metadata,
) -> Result<(), RunError> {
    let link_target = std::fs::read_link(source_path)
        .map_err(|err| RunError::Internal(format!("read_link {}: {err}", source_path.display())))?;
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    header.set_mode(mode_bits_or_default(metadata, TAR_MODE_SYMLINK_DEFAULT));
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header
        .set_link_name(&link_target)
        .map_err(|err| RunError::Internal(format!("setting link name for {}: {err}", source_path.display())))?;
    header.set_cksum();
    let tar_path = tar_path_string(relative_path)?;
    builder
        .append_data(&mut header, tar_path, io::empty())
        .map_err(|err| RunError::Internal(format!("appending symlink {}: {err}", source_path.display())))?;
    Ok(())
}

#[cfg(unix)]
fn os_string_from_git_bytes(raw_path: &[u8]) -> Result<std::ffi::OsString, RunError> {
    Ok(std::ffi::OsString::from_vec(raw_path.to_vec()))
}

#[cfg(not(unix))]
fn os_string_from_git_bytes(raw_path: &[u8]) -> Result<std::ffi::OsString, RunError> {
    let text = String::from_utf8(raw_path.to_vec())
        .map_err(|err| RunError::Internal(format!("git ls-files emitted non-utf8 path: {err}")))?;
    Ok(std::ffi::OsString::from(text))
}

fn tar_directory_path(relative_path: &Path) -> Result<String, RunError> {
    let mut tar_path = tar_path_string(relative_path)?;
    if !tar_path.ends_with('/') {
        tar_path.push('/');
    }
    Ok(tar_path)
}

fn tar_path_string(relative_path: &Path) -> Result<String, RunError> {
    let Some(path_str) = relative_path.to_str() else {
        return Err(RunError::Internal(format!("source archive path is not valid utf-8: {}", relative_path.display())));
    };
    Ok(path_str.replace('\\', "/"))
}

#[cfg(unix)]
fn mode_bits_or_default(metadata: &std::fs::Metadata, default_mode: u32) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode() & 0o7777;
    if mode == 0 { default_mode } else { mode }
}

#[cfg(not(unix))]
fn mode_bits_or_default(_metadata: &std::fs::Metadata, default_mode: u32) -> u32 {
    default_mode
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::io::Read;
    use std::process::Output;

    use sha2::Sha256;

    use super::*;

    const TEST_SHA256_HEX_LEN: usize = 64;

    fn write_file(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn find_git_binary() -> PathBuf {
        if let Ok(git) = std::env::var("GIT") {
            let path = PathBuf::from(git);
            if path.exists() {
                return path;
            }
        }
        for candidate in [
            "/usr/bin/git",
            "/bin/git",
            "/usr/local/bin/git",
            "/run/current-system/sw/bin/git",
        ] {
            let path = Path::new(candidate);
            if path.exists() {
                return path.to_path_buf();
            }
        }
        if let Ok(user) = std::env::var("USER") {
            let profile = PathBuf::from(format!("/etc/profiles/per-user/{user}/bin/git"));
            if profile.exists() {
                return profile;
            }
        }
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                let candidate = Path::new(dir).join("git");
                if candidate.exists() {
                    return candidate;
                }
            }
        }
        panic!("git not found for release_source tests");
    }

    fn run_git(repo_root: &Path, args: &[&str]) -> Output {
        Command::new(find_git_binary()).args(args).current_dir(repo_root).output().unwrap()
    }

    fn assert_git_ok(repo_root: &Path, args: &[&str]) {
        let output = run_git(repo_root, args);
        assert!(
            output.status.success(),
            "git {:?} failed: stdout={} stderr={}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    fn cargo_sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = <Sha256 as sha2::Digest>::new();
        <Sha256 as sha2::Digest>::update(&mut hasher, bytes);
        let digest = <Sha256 as sha2::Digest>::finalize(hasher);
        let encoded = data_encoding::HEXLOWER.encode(&digest);
        assert_eq!(encoded.len(), TEST_SHA256_HEX_LEN);
        encoded
    }

    fn write_test_vendor_package(repo_root: &Path) {
        let manifest = b"[package]\nname=\"dep\"\nversion=\"0.1.0\"\n";
        let lib = b"pub fn dep() {}\n";
        write_file(&repo_root.join("vendor-deps/dep/Cargo.toml"), manifest);
        write_file(&repo_root.join("vendor-deps/dep/lib.rs"), lib);
        let manifest_digest = cargo_sha256_hex(manifest);
        let lib_digest = cargo_sha256_hex(lib);
        let checksum_manifest = format!(
            "{{\"files\":{{\"Cargo.toml\":\"{manifest_digest}\",\"lib.rs\":\"{lib_digest}\"}},\"package\":null}}",
        );
        write_file(&repo_root.join("vendor-deps/dep/.cargo-checksum.json"), checksum_manifest.as_bytes());
    }

    fn create_minimal_repo(repo_root: &Path) {
        write_file(&repo_root.join(".gitignore"), b"vendor-deps/\ntarget/\n.pi/\n");
        write_file(&repo_root.join(".cargo/vendor-config.toml"), b"directory = \"vendor-deps\"\n");
        write_test_vendor_package(repo_root);
        write_file(&repo_root.join("Cargo.toml"), b"[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
        write_file(
            &repo_root.join("Cargo.lock"),
            b"[[package]]\nname = \"dep\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/dep.git#0123456789abcdef\"\n",
        );
        write_file(&repo_root.join("README.md"), b"# demo\n");
        write_file(&repo_root.join("bootstrap/seed.ncl"), b"{}\n");
        write_file(&repo_root.join("builders/default.ncl"), b"{}\n");
        write_file(&repo_root.join("docs/bootstrap-stage0-inventory.md"), b"# inventory\n");
        write_file(&repo_root.join("scripts/prove-self-hosting.sh"), b"#!/usr/bin/env bash\n");
        write_file(&repo_root.join("scripts/unrelated-helper.sh"), b"#!/usr/bin/env bash\n");
        write_file(&repo_root.join("tests/audit_support.rs"), b"pub fn audit() {}\n");
        write_file(&repo_root.join("tests/self_hosting.rs"), b"#[test]\nfn proof() {}\n");
        write_file(&repo_root.join("tests/unrelated.rs"), b"#[test]\nfn unrelated() {}\n");
        write_file(&repo_root.join("crates/demo/src/lib.rs"), b"pub fn demo() {}\n");
        write_file(&repo_root.join("lib/lib.ncl"), b"{}\n");
        write_file(&repo_root.join("rust-toolchain.toml"), b"[toolchain]\nchannel=\"nightly\"\n");
        write_file(&repo_root.join("src/main.rs"), b"fn main() {}\n");
        write_file(&repo_root.join("vendor/README"), b"vendor\n");
        write_file(&repo_root.join("cairn/specs/demo/spec.md"), b"# skipped evidence\n");
        write_file(&repo_root.join("target/release-signing/test/signing-key"), b"secret\n");
        write_file(&repo_root.join(".pi/private-note"), b"private\n");
    }

    #[test]
    fn source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files() {
        let repo = tempfile::tempdir().unwrap();
        create_minimal_repo(repo.path());
        assert_git_ok(repo.path(), &["init"]);
        assert_git_ok(repo.path(), &["config", "user.email", "pi@example.com"]);
        assert_git_ok(repo.path(), &["config", "user.name", "Pi"]);
        assert_git_ok(repo.path(), &["add", "."]);
        assert_git_ok(repo.path(), &[
            "add",
            "-f",
            "target/release-signing/test/signing-key",
            ".pi/private-note",
        ]);
        assert_git_ok(repo.path(), &["commit", "-m", "initial"]);

        write_file(&repo.path().join("src/main.rs"), b"fn main() { println!(\"worktree\"); }\n");
        write_file(&repo.path().join("src/untracked.txt"), b"ignore me\n");

        let archive = tempfile::NamedTempFile::new().unwrap();
        write_tracked_source_archive(repo.path(), archive.path()).unwrap();

        let archive_bytes = std::fs::read(archive.path()).unwrap();
        let mut tar = tar::Archive::new(Cursor::new(archive_bytes));
        let mut found_main = false;
        let mut found_untracked = false;
        let mut found_ignored_vendor = false;
        let mut found_readme = false;
        let mut found_proof_driver = false;
        let mut found_inventory_doc = false;
        let mut found_unrelated_script = false;
        let mut found_self_hosting_test = false;
        let mut found_audit_support = false;
        let mut found_unrelated_test = false;
        let mut found_cairn_spec = false;
        let mut found_target_secret = false;
        let mut found_pi_private_note = false;
        for entry_result in tar.entries().unwrap() {
            let mut entry = entry_result.unwrap();
            let path = entry.path().unwrap().into_owned();
            if path == std::path::Path::new("src/main.rs") {
                let mut text = String::new();
                entry.read_to_string(&mut text).unwrap();
                assert!(text.contains("worktree"));
                found_main = true;
            }
            if path == std::path::Path::new("src/untracked.txt") {
                found_untracked = true;
            }
            if path == std::path::Path::new("vendor-deps/dep/Cargo.toml") {
                found_ignored_vendor = true;
            }
            if path == std::path::Path::new("README.md") {
                found_readme = true;
            }
            if path == std::path::Path::new("scripts/prove-self-hosting.sh") {
                found_proof_driver = true;
            }
            if path == std::path::Path::new("docs/bootstrap-stage0-inventory.md") {
                found_inventory_doc = true;
            }
            if path == std::path::Path::new("scripts/unrelated-helper.sh") {
                found_unrelated_script = true;
            }
            if path == std::path::Path::new("tests/self_hosting.rs") {
                found_self_hosting_test = true;
            }
            if path == std::path::Path::new("tests/audit_support.rs") {
                found_audit_support = true;
            }
            if path == std::path::Path::new("tests/unrelated.rs") {
                found_unrelated_test = true;
            }
            if path == std::path::Path::new("cairn/specs/demo/spec.md") {
                found_cairn_spec = true;
            }
            if path == std::path::Path::new("target/release-signing/test/signing-key") {
                found_target_secret = true;
            }
            if path == std::path::Path::new(".pi/private-note") {
                found_pi_private_note = true;
            }
        }

        assert!(found_main, "archive must contain tracked modified file");
        assert!(found_ignored_vendor, "archive must include verified ignored vendor-deps");
        assert!(found_readme, "archive must include tracked package-root files seen by native source hashing");
        assert!(found_proof_driver, "archive must include witness rebuild workflow driver");
        assert!(found_inventory_doc, "archive must include proof inventory consumed by workflow driver");
        assert!(found_unrelated_script, "archive must include tracked helper scripts seen by native source hashing");
        assert!(found_self_hosting_test, "archive must include self-hosting test target used by workflow driver");
        assert!(found_audit_support, "archive must include self-hosting test support module");
        assert!(found_unrelated_test, "archive must include tracked tests seen by native source hashing");
        assert!(!found_untracked, "archive must skip untracked file");
        assert!(!found_cairn_spec, "archive must skip root Cairn evidence skipped by native source hashing");
        assert!(!found_target_secret, "archive must skip target secrets even if force-tracked");
        assert!(!found_pi_private_note, "archive must skip private .pi files even if force-tracked");
    }
}
