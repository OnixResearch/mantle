#!/usr/bin/env python3
"""Copy the complete reviewed working-tree source into a read-only APK proof snapshot.

The index, unstaged edits, and untracked non-ignored source files are all included.
The source list and every file identity must remain unchanged through the copy.
"""

import argparse
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import tomllib


def command(arguments, cwd):
    return subprocess.run(arguments, cwd=cwd, check=True, capture_output=True).stdout


def source_paths(repo):
    names = command(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], repo).split(b"\0")
    paths = sorted({os.fsdecode(name) for name in names if name})
    if not paths:
        raise RuntimeError("Git reported no source files")
    sources = []
    for name in paths:
        parsed = PurePosixPath(name)
        if parsed.is_absolute() or not parsed.parts or ".." in parsed.parts or ".git" in parsed.parts:
            raise RuntimeError(f"unsafe source member: {name!r}")
        if "__pycache__" not in parsed.parts and parsed.suffix != ".pyc":
            sources.append(name)
    return sources


def blake3_bytes(data):
    result = subprocess.run(["b3sum", "--no-names"], input=data, check=True, capture_output=True).stdout.decode().strip()
    if len(result) != 64:
        raise RuntimeError("BLAKE3 failed to identify canonical source bytes")
    return result


def blake3_file(path):
    result = command(["b3sum", "--no-names", str(path)], path.parent).decode().strip()
    if len(result) != 64:
        raise RuntimeError(f"BLAKE3 failed to identify {path}")
    return result


def identity(repo, name):
    path = repo / name
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        # A tracked deletion is intentionally absent from the working-tree snapshot.
        return None
    mode = stat.S_IMODE(metadata.st_mode)
    if stat.S_ISLNK(metadata.st_mode):
        link = os.readlink(path)
        resolved = path.resolve()
        if not resolved.is_relative_to(repo) or not resolved.exists():
            raise RuntimeError(f"source symlink escapes or loses its dependency: {name} -> {link}")
        return {"path": name, "kind": "symlink", "mode": mode, "target": link, "blake3": blake3_bytes(b"symlink\0" + os.fsencode(link))}
    if stat.S_ISREG(metadata.st_mode):
        return {"path": name, "kind": "file", "mode": mode, "size": metadata.st_size, "blake3": blake3_file(path)}
    raise RuntimeError(f"unsupported source file type: {name}")


def listed_identities(repo, names):
    return [record for name in names if (record := identity(repo, name)) is not None]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    arguments = parser.parse_args()
    repo = arguments.source.resolve()
    snapshot = arguments.snapshot.resolve()
    manifest_path = arguments.manifest.resolve()
    if snapshot.exists() or manifest_path.exists():
        raise RuntimeError("source snapshot and manifest paths must both be fresh")
    if snapshot.is_relative_to(repo) or manifest_path.is_relative_to(repo) or manifest_path.is_relative_to(snapshot):
        raise RuntimeError("snapshot and manifest must be outside source and each other")
    cargo_config = repo / ".cargo/config.toml"
    if cargo_config.is_file() and tomllib.loads(cargo_config.read_text(encoding="utf-8")).get("source"):
        raise RuntimeError("active Cargo source replacement may require an ignored vendor tree; review it before snapshot capture")
    ignored_vendor_deps = (repo / "vendor-deps").exists()
    names = source_paths(repo)
    required = {
        "Cargo.toml", "Cargo.lock", ".cargo/config.toml",
        "crates/crunch-build/src/lib.rs", "crates/crunch-store/src/lib.rs",
        "crates/crunch-android-core/src/lib.rs", "crates/crunch-android/src/lib.rs",
        "scripts/snapshot-android-apk-source.py", "scripts/prove-android-apk.py",
    }
    git_head = command(["git", "rev-parse", "HEAD"], repo).decode().strip()
    before = listed_identities(repo, names)
    if not required <= {record["path"] for record in before}:
        raise RuntimeError("source contents omitted Cargo lock, an owner crate, or the proof rail")
    snapshot.mkdir(parents=True)
    for record in before:
        original = repo / record["path"]
        destination = snapshot / record["path"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        if record["kind"] == "symlink":
            os.symlink(record["target"], destination)
        else:
            shutil.copy2(original, destination)
            if identity(snapshot, record["path"]) != record:
                raise RuntimeError(f"source changed during controlled copy: {record['path']}")
    if source_paths(repo) != names or listed_identities(repo, names) != before:
        raise RuntimeError("live source paths or contents changed during snapshot capture")
    if listed_identities(snapshot, names) != before:
        raise RuntimeError("snapshot contents do not match all original source identities")
    for root, directories, files in os.walk(snapshot, topdown=False):
        for name in files:
            path = Path(root) / name
            if not path.is_symlink():
                path.chmod(stat.S_IMODE(path.stat().st_mode) & ~0o222)
        for name in directories:
            path = Path(root) / name
            if not path.is_symlink():
                path.chmod(stat.S_IMODE(path.stat().st_mode) & ~0o222)
    snapshot.chmod(stat.S_IMODE(snapshot.stat().st_mode) & ~0o222)
    nix_interop_nar_sha256 = command(["nix", "hash", "path", "--type", "sha256", "--base16", str(snapshot)], repo).decode().strip()
    if len(nix_interop_nar_sha256) != 64:
        raise RuntimeError("Nix did not return a SHA-256 NAR identity for the immutable source tree")
    if (
        command(["git", "rev-parse", "HEAD"], repo).decode().strip() != git_head
        or source_paths(repo) != names
        or listed_identities(repo, names) != before
    ):
        raise RuntimeError("live HEAD, source paths, or file BLAKE3 changed before snapshot manifest sealing")
    profile = {"entries": before, "git_head": git_head}
    source_profile_blake3 = blake3_bytes(json.dumps(profile, sort_keys=True, separators=(",", ":")).encode("utf-8"))
    manifest = {
        "schema": "mantle-apk-immutable-source-snapshot-v1",
        "source_root": str(repo),
        "snapshot_root": str(snapshot),
        "source_profile_blake3": source_profile_blake3,
        "nix_interop_nar_sha256": nix_interop_nar_sha256,
        "git_head": git_head,
        "file_count": len(before),
        "excluded_ignored_vendor_deps": {
            "present": ignored_vendor_deps,
            "reason": "inactive: pinned Nix devshell CARGO_HOME uses Crane-vendored Nix-store sources; proof Cargo uses --offline without opt-in .cargo/vendor-config.toml; active .cargo/config.toml has no source replacement",
        },
        "entries": before,
    }
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    manifest_blake3 = blake3_file(manifest_path)
    print(
        f"APK_SOURCE_SNAPSHOT root={snapshot} profile_blake3={source_profile_blake3} "
        f"nix_interop_nar_sha256={nix_interop_nar_sha256} files={len(before)} "
        f"manifest={manifest_path} manifest_blake3={manifest_blake3}"
    )


if __name__ == "__main__":
    main()
