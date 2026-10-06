#!/usr/bin/env python3
"""Import and verify exact published Nickel vector and SecretSpec source ports.

Run `import [registry-cache-directory]` only against the original crates.io
archives listed in Cargo.lock. `check [registry-cache-directory]` verifies the
tracked source, original-file receipt, patch replay, and optionally the actual
registry archive. No Cargo-generated vendor source or checksum is ever edited.
"""

import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
RECEIPT = ROOT / "third_party/advisory-source-ports.json"
# Pin the generated receipt after exact archive import; Nix independently pins it.
RECEIPT_SHA256 = "ff3c1885a880ce2c260bfabbada13457c82c72b674dfd8031947e963434a157e"
PACKAGES = (
    ("nickel-lang-vector", "0.2.0", "36f243832286908d8873add24a905d6732ffabd6cfb2bf74cb18d667e892e279",
     "f09fce4517c853a9845db13aa60d2b73405c799a", "MIT", "nickel-vector-safe-chunks.patch"),
    ("secretspec", "0.17.0", "68498f9695bb3662c157b8fd4b4665a594f1157de022ff5b0f891af4c7ec75d2",
     "a8794e46ec9664a0e1a3869cc3105d0853937e48", "Apache-2.0", "secretspec-optional-rsa-generation.patch"),
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def archive_files(path, name, version, checksum):
    archive = path.read_bytes()
    if digest(archive) != checksum:
        raise ValueError(f"{name}: original registry archive checksum mismatch")
    prefix = f"{name}-{version}/"
    files = {}
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as source:
        for member in source:
            if member.isdir():
                continue
            if not member.isfile() or not member.name.startswith(prefix):
                raise ValueError(f"{name}: unsupported archive entry {member.name}")
            relative = member.name.removeprefix(prefix)
            if not relative or Path(relative).is_absolute() or ".." in Path(relative).parts or relative in files:
                raise ValueError(f"{name}: unsafe or duplicate archive path {relative}")
            files[relative] = source.extractfile(member).read()
    return files


def tree_files(directory):
    if not directory.is_dir() or directory.is_symlink():
        raise ValueError(f"missing or unsafe port directory: {directory}")
    files = {}
    for path in directory.rglob("*"):
        if path.is_symlink() or not (path.is_file() or path.is_dir()):
            raise ValueError(f"unsafe port entry: {path}")
        if path.is_file():
            files[path.relative_to(directory).as_posix()] = path.read_bytes()
    return files


def patch_files(files, patch, reverse=False):
    with tempfile.TemporaryDirectory(prefix="mantle-port-") as tmp:
        directory = Path(tmp)
        for name, data in files.items():
            destination = directory / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        command = ["patch", "--batch", "--fuzz=0", "-p1", "-d", tmp, "-i", str(patch)]
        command.insert(2, "--reverse" if reverse else "--forward")
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        if result.returncode:
            raise ValueError(f"{patch.name}: source patch does not apply exactly:\n{result.stdout}{result.stderr}")
        return tree_files(directory)


def pin_and_receipt(spec, original):
    name, version, checksum, revision, license_name, patch_name = spec
    manifest = tomllib.loads(original["Cargo.toml"].decode())
    vcs = json.loads(original[".cargo_vcs_info.json"])
    if (manifest["package"]["name"], manifest["package"]["version"], manifest["package"]["license"], vcs["git"]["sha1"]) != (name, version, license_name, revision):
        raise ValueError(f"{name}: published package identity, license, or revision drift")
    locked = [p for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"] if p["name"] == name]
    if len(locked) != 1 or locked[0]["version"] != version:
        raise ValueError(f"{name}: Cargo.lock exact package version drift")
    if "source" in locked[0]:
        if locked[0]["source"] != "registry+https://github.com/rust-lang/crates.io-index" or locked[0].get("checksum") != checksum:
            raise ValueError(f"{name}: Cargo.lock registry source checksum drift")
    else:
        root = tomllib.loads((ROOT / "Cargo.toml").read_text())
        if "checksum" in locked[0] or root["patch"]["crates-io"].get(name) != {"path": f"third_party/{name}"}:
            raise ValueError(f"{name}: Cargo.lock path patch drift")
    patch = ROOT / "patches" / patch_name
    patched = patch_files(original, patch)
    return patched, {
        "name": name,
        "version": version,
        "original_registry_checksum_sha256": checksum,
        "published_revision": revision,
        "license": license_name,
        "source": "https://github.com/rust-lang/crates.io-index",
        "patch": f"patches/{patch_name}",
        "patch_sha256": digest(patch.read_bytes()),
        "original_files_sha256": {file: digest(data) for file, data in sorted(original.items())},
        "patched_files_sha256": {file: digest(data) for file, data in sorted(patched.items())},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("import", "check"))
    parser.add_argument("cache", nargs="?", type=Path)
    args = parser.parse_args()
    cache = args.cache or Path.home() / ".cargo/registry/cache/index.crates.io-1949cf8c6b5b557f"
    selected = []
    pending = []
    if args.operation == "import" and (
        RECEIPT.exists() or any((ROOT / "third_party" / spec[0]).exists() for spec in PACKAGES)
    ):
        raise ValueError("refusing to clobber an existing source port or receipt")
    for spec in PACKAGES:
        name, version, checksum, revision, license_name, patch_name = spec
        directory = ROOT / "third_party" / name
        archive = cache / f"{name}-{version}.crate"
        if args.operation == "import":
            original = archive_files(archive, name, version, checksum)
            patched, record = pin_and_receipt(spec, original)
            pending.append((directory, patched))
        else:
            recorded = json.loads(RECEIPT.read_text())
            if digest(RECEIPT.read_bytes()) != RECEIPT_SHA256:
                raise ValueError("advisory source port receipt pin drift")
            entry = next((item for item in recorded["packages"] if item["name"] == name), None)
            if entry is None:
                raise ValueError(f"{name}: port receipt missing")
            patched = tree_files(directory)
            original = patch_files(patched, ROOT / "patches" / patch_name, reverse=True)
            replay, record = pin_and_receipt(spec, original)
            if replay != patched:
                raise ValueError(f"{name}: tracked patch replay mismatch")
            if archive.exists() and archive_files(archive, name, version, checksum) != original:
                raise ValueError(f"{name}: port differs from original registry archive")
            if entry != record:
                raise ValueError(f"{name}: original or patched port file receipt drift")
        selected.append(record)
        print(f"{name} {version}: published {revision}; {len(record['patched_files_sha256'])} source files; patch verified")
    if args.operation == "import":
        for directory, files in pending:
            directory.mkdir(parents=True)
            for file, data in files.items():
                destination = directory / file
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(data)
        RECEIPT.write_text(json.dumps({"packages": selected}, indent=2, sort_keys=True) + "\n")
        print(f"Pin receipt SHA256: {digest(RECEIPT.read_bytes())}")
    elif json.loads(RECEIPT.read_text()) != {"packages": selected}:
        raise ValueError("advisory source port receipt package set drift")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, tarfile.TarError) as error:
        print(f"import-advisory-ports: {error}", file=sys.stderr)
        sys.exit(1)
