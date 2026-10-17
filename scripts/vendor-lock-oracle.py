#!/usr/bin/env python3
"""Build a same-lock offline Cargo vendor oracle, independent of Mantle's assembler.

Only the explicitly supplied registry cache, sparse index, Git cache, and
single separately signed missing archive enter the isolated Cargo home.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    parser.add_argument("--registry-cache", type=Path, required=True)
    parser.add_argument("--index", type=Path, required=True)
    parser.add_argument("--git-cache", type=Path, required=True)
    parser.add_argument("--signed-missing-archive", type=Path, required=True)
    parser.add_argument("--cargo", type=Path, required=True)
    args = parser.parse_args()
    workspace = args.workspace.resolve()
    scratch = args.scratch
    if scratch.exists() or not scratch.parent.is_dir():
        raise ValueError("independent oracle requires a fresh existing-parent scratch")
    packages = tomllib.loads((workspace / "Cargo.lock").read_text())["package"]
    registry = [row for row in packages if row.get("source", "").startswith("registry+")]
    missing = []
    for row in registry:
        archive = args.registry_cache / f"{row['name']}-{row['version']}.crate"
        if not archive.is_file() or archive.is_symlink():
            missing.append(row)
        elif sha256(archive) != row["checksum"]:
            raise ValueError(f"cached registry checksum drift: {row['name']}@{row['version']}")
    if len(missing) != 1:
        raise ValueError(
            f"expected exactly one separately signed missing archive: {[(p['name'], p['version']) for p in missing]}"
        )
    if sha256(args.signed_missing_archive) != missing[0]["checksum"]:
        raise ValueError("separately signed missing archive differs from Cargo.lock")
    if not args.cargo.is_file() or not args.index.is_dir() or not args.git_cache.is_dir():
        raise ValueError("missing explicitly declared offline Cargo inputs")
    scratch.mkdir()
    home = scratch / "cargo-home"
    cache = home / "registry/cache/index.crates.io-1949cf8c6b5b557f"
    cache.parent.mkdir(parents=True)
    shutil.copytree(args.registry_cache, cache, symlinks=True)
    shutil.copyfile(args.signed_missing_archive, cache / f"{missing[0]['name']}-{missing[0]['version']}.crate")
    index = home / "registry/index/index.crates.io-1949cf8c6b5b557f"
    index.parent.mkdir(parents=True)
    shutil.copytree(args.index, index, symlinks=True)
    shutil.copytree(args.git_cache, home / "git", symlinks=True)
    output = scratch / "vendor-deps"
    environment = {**os.environ, "CARGO_HOME": str(home), "CARGO_NET_OFFLINE": "true"}
    result = subprocess.run([str(args.cargo), "vendor", "--locked", "--offline",
                             "--manifest-path", str(workspace / "Cargo.toml"), str(output)],
                            cwd=workspace, env=environment, text=True, capture_output=True)
    (scratch / "cargo-vendor.stdout").write_text(result.stdout)
    (scratch / "cargo-vendor.stderr").write_text(result.stderr)
    if result.returncode != 0:
        raise RuntimeError(f"independent offline Cargo vendor failed: {result.stderr[-2000:]}")
    import importlib.util
    helper_path = workspace / "scripts/vendor-deps.py"
    spec = importlib.util.spec_from_file_location("checked_vendor_patch", helper_path)
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    helper.apply_casita_patch(output)
    found = set()
    for package in output.iterdir():
        if not package.is_dir() or package.is_symlink():
            raise ValueError(f"unexpected Cargo vendor entry: {package}")
        manifest = tomllib.loads((package / "Cargo.toml").read_text())["package"]
        key = (manifest["name"], manifest["version"])
        if key in found:
            raise ValueError(f"duplicate Cargo vendor package: {key}")
        found.add(key)
    expected = {(p["name"], p["version"]) for p in packages if p.get("source")}
    if found != expected:
        raise ValueError(f"Cargo vendor inventory mismatch: missing={sorted(expected - found)}, extra={sorted(found - expected)}")
    print(json.dumps({"oracle": str(output), "packages": len(found),
                      "lock_sha256": sha256(workspace / "Cargo.lock"),
                      "signed_missing_sha256": missing[0]["checksum"]}, sort_keys=True))


if __name__ == "__main__":
    main()
