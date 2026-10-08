#!/usr/bin/env python3
"""Bounded NAR, Git-commit and Cargo-vendor verification helpers.

The companion vendor-dynamic-assemble.py owns the declared shard/final CLI.
"""

import argparse
import ctypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import struct
import subprocess
import tempfile
import tomllib

MAX_LOCK = 4 * 1024 * 1024
MAX_PLAN = 2 * 1024 * 1024
MAX_ARTIFACTS = 4096
MAX_GIT_NAR_BYTES = 512 * 1024 * 1024
MAX_GIT_NODES = 20_000
MAX_GIT_DB_BYTES = 512 * 1024 * 1024
MAX_INPUT_FILE = 256 * 1024 * 1024
MAX_REGISTRY_BYTES = 512 * 1024 * 1024
MAX_VENDOR_BYTES = 2 * 1024 * 1024 * 1024
MAX_VENDOR_ENTRIES = 120_000
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def nar_sha256(path, *, byte_limit=MAX_GIT_NAR_BYTES, node_limit=MAX_GIT_NODES):
    """SHA-256 of the real recursive Nix NAR stream, without requiring Nix."""
    digest = hashlib.sha256()
    written = 0
    nodes = 0

    def emit(data):
        nonlocal written
        written += len(data)
        require(written <= byte_limit, "NAR exceeds byte bound")
        digest.update(data)

    def atom(value):
        data = os.fsencode(value)
        emit(struct.pack("<Q", len(data)))
        emit(data)
        if padding := (-len(data)) % 8:
            emit(bytes(padding))

    def node(current):
        nonlocal nodes
        nodes += 1
        require(nodes <= node_limit, "NAR exceeds node bound")
        mode = current.lstat().st_mode
        atom("(")
        atom("type")
        if stat.S_ISDIR(mode):
            atom("directory")
            for name in sorted(os.listdir(current), key=os.fsencode):
                atom("entry")
                atom("(")
                atom("name")
                atom(name)
                atom("node")
                node(current / name)
                atom(")")
        elif stat.S_ISREG(mode):
            atom("regular")
            if mode & 0o111:
                atom("executable")
                atom("")
            atom("contents")
            size = current.stat().st_size
            require(size <= MAX_INPUT_FILE, "Git tree file exceeds byte bound")
            emit(struct.pack("<Q", size))
            with current.open("rb") as source:
                while chunk := source.read(1024 * 1024):
                    emit(chunk)
            if padding := (-size) % 8:
                emit(bytes(padding))
        elif stat.S_ISLNK(mode):
            atom("symlink")
            atom("target")
            atom(os.readlink(current))
        else:
            raise ValueError("special file in Git NAR")
        atom(")")

    atom("nix-archive-1")
    node(path)
    return digest.hexdigest(), written


def run(command, *, cwd, environment):
    result = subprocess.run(command, cwd=cwd, env=environment, capture_output=True, text=True, check=False)
    require(result.returncode == 0, f"command denied ({command[0]}): {result.stderr[-1400:]}")
    return result.stdout



def crlf_normalized_blob(path, size, hasher):
    """Compute Git's clean-filter blob for an explicitly checked-in eol=crlf path."""
    def chunks():
        pending = b""
        with path.open("rb") as source:
            while chunk := source.read(1024 * 1024):
                combined = pending + chunk
                pending = b"\r" if combined.endswith(b"\r") else b""
                yield combined[:-1] if pending else combined
        if pending:
            yield pending

    normalized_size = size - sum(chunk.count(b"\r\n") for chunk in chunks())
    require(normalized_size < size, f"no CRLF checkout expansion: {path}")
    digest = hasher(b"blob " + str(normalized_size).encode() + b"\0")
    for chunk in chunks():
        digest.update(chunk.replace(b"\r\n", b"\n"))
    return digest.hexdigest().encode()


def verify_git_revision_tree(db, revision, tree):
    """Bind a reviewed NAR tree to a Git revision's blobs, modes and checked-in EOL attributes."""
    hasher = hashlib.sha1 if len(revision) == 40 else hashlib.sha256
    test = subprocess.run(["git", "--git-dir", str(db), "cat-file", "-e", revision + "^{commit}"], capture_output=True)
    require(test.returncode == 0, f"Git revision unavailable: {revision}")
    listed = subprocess.run(["git", "--git-dir", str(db), "ls-tree", "-rz", "--full-tree", revision], capture_output=True)
    require(listed.returncode == 0 and len(listed.stdout) <= 4 * 1024 * 1024, f"Git tree listing denied: {revision}")
    expected = set()
    normalized = []
    for record in listed.stdout.split(b"\0"):
        if not record:
            continue
        require(b"\t" in record, "invalid Git tree entry")
        descriptor, filename = record.split(b"\t", 1)
        fields = descriptor.split(b" ")
        require(len(fields) == 3 and fields[1] == b"blob", "unsupported Git tree entry")
        name = os.fsdecode(filename)
        relative = PurePosixPath(name)
        require(not relative.is_absolute() and relative.parts and all(part not in ("", ".", "..") for part in relative.parts), "Git tree path escape")
        require(name not in expected, "duplicate Git tree path")
        expected.add(name)
        path = tree.joinpath(*relative.parts)
        require(path.is_file() or path.is_symlink(), f"missing Git tree file: {name}")
        mode = fields[0]
        if mode == b"120000":
            require(path.is_symlink(), f"Git symlink mode mismatch: {name}")
            link_target = os.readlink(path)
            require(not os.path.isabs(link_target), f"absolute Git symlink: {name}")
            resolved_target = os.path.normpath(os.path.join(os.path.dirname(name), link_target))
            require(resolved_target != ".." and not resolved_target.startswith("../"), f"escaping Git symlink: {name}")
            payload = os.fsencode(link_target)
            digest = hasher(b"blob " + str(len(payload)).encode() + b"\0" + payload).hexdigest().encode()
        else:
            require(mode in (b"100644", b"100755") and path.is_file() and not path.is_symlink(), f"Git file mode mismatch: {name}")
            size = path.stat().st_size
            require(size <= MAX_INPUT_FILE, f"Git file bound exceeded: {name}")
            require(bool(path.stat().st_mode & 0o111) == (mode == b"100755"), f"Git executable mode mismatch: {name}")
            hash_state = hasher(b"blob " + str(size).encode() + b"\0")
            with path.open("rb") as source:
                while chunk := source.read(1024 * 1024):
                    hash_state.update(chunk)
            digest = hash_state.hexdigest().encode()
        if digest != fields[2]:
            require(mode != b"120000" and not name.endswith("/.gitattributes")
                    and name != ".gitattributes", f"Git blob differs from pinned revision: {name}")
            normalized.append((name, path, size, fields[2]))
    observed = {str(path.relative_to(tree)) for path in tree.rglob("*") if path.is_file() or path.is_symlink()}
    require(observed == expected, "Git tree has unpinned or missing files")
    for name, path, size, expected_blob in normalized:
        attribute = subprocess.run(
            ["git", "-c", "core.attributesFile=/dev/null", "--git-dir", str(db),
             "--work-tree", str(tree), "check-attr", "-z", "eol", "--", name],
            capture_output=True,
        )
        require(attribute.returncode == 0 and attribute.stdout == os.fsencode(name) + b"\0eol\0crlf\0",
                f"Git blob differs from pinned revision: {name}")
        require(crlf_normalized_blob(path, size, hasher) == expected_blob,
                f"Git blob differs from pinned revision: {name}")


def bounded_git_metadata(root):
    require(root.is_dir() and not root.is_symlink(), "missing unsafe Cargo Git metadata")
    total = 0
    entries = 0
    for parent, dirs, files in os.walk(root, followlinks=False):
        for name in dirs + files:
            path = Path(parent) / name
            entries += 1
            require(entries <= MAX_GIT_NODES and not path.is_symlink(), "Git metadata entry bound or symlink")
            if path.is_file():
                total += path.stat().st_size
                require(total <= MAX_GIT_DB_BYTES, "Git object metadata byte bound")



def add_git_sources(groups, transports, home):
    require(set(transports) == set(groups), "Git transports must match exact selected lock sources")
    for source, spec in groups.items():
        transport = transports[source]
        db = Path(transport["git_db"])
        checkout = Path(transport["checkout"])
        tree = Path(transport["git_tree"])
        require(tree.is_dir() and not tree.is_symlink(), "missing unsafe fetched Git NAR tree")
        require(db.is_dir() and checkout.is_dir() and not db.is_symlink() and not checkout.is_symlink(), "missing unsafe Git object input")
        require(checkout.parent.name == db.name and checkout.name == spec["revision"][:7], "Git Cargo cache identity drift")
        actual, size = nar_sha256(tree)
        require(actual == spec["sha256"], f"Git NAR mismatch for {source}: {actual} != {spec['sha256']}")
        verify_git_revision_tree(db, spec["revision"], tree)
        require((checkout / ".git").is_dir() and (checkout / ".cargo-ok").is_file(), "missing Cargo Git checkout metadata")
        observed_head = subprocess.run(["git", "-C", str(checkout), "rev-parse", "HEAD"], capture_output=True, text=True)
        require(observed_head.returncode == 0 and observed_head.stdout.strip() == spec["revision"], "Cargo checkout Git revision drift")
        bounded_git_metadata(db)
        bounded_git_metadata(checkout / ".git")
        target_db = home / "git/db" / db.name
        target_db.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(db, target_db, symlinks=False)
        target_checkout = home / "git/checkouts" / checkout.parent.name / checkout.name
        target_checkout.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(tree, target_checkout, symlinks=True)
        target_checkout.chmod(target_checkout.stat().st_mode | stat.S_IWUSR)
        shutil.copytree(checkout / ".git", target_checkout / ".git", symlinks=False)
        shutil.copy2(checkout / ".cargo-ok", target_checkout / ".cargo-ok")
        print(f"authenticated Git {spec['revision'][:12]} NAR sha256={actual} bytes={size}", flush=True)




def checked_vendor(vendor, artifacts):
    expected = {row[2] for row in artifacts}
    actual = {path.name for path in vendor.iterdir() if path.is_dir() and not path.is_symlink()}
    require(actual == expected, "assembled vendor package layout differs from lock plan")
    total = 0
    entries = 0
    for parent, dirs, names in os.walk(vendor, followlinks=False):
        for name in dirs + names:
            path = Path(parent) / name
            entries += 1
            require(entries <= MAX_VENDOR_ENTRIES, "vendor entry bound exceeded")
            require(not path.is_symlink(), f"symlink in assembled vendor: {path}")
            if path.is_file():
                total += path.stat().st_size
                require(total <= MAX_VENDOR_BYTES, "vendor payload byte bound exceeded")
    for name, version, dest, sha, mode, _, _, _ in artifacts:
        checksum = json.loads((vendor / dest / ".cargo-checksum.json").read_text())
        require(checksum["package"] == (sha if mode == "flat" else None), f"{name}@{version}: Cargo package checksum mismatch")
    return entries, total




def publish_noreplace(source, destination):
    libc = ctypes.CDLL(None, use_errno=True)
    status = libc.renameat2(-100, os.fsencode(source), -100, os.fsencode(destination), 1)
    require(status == 0, f"refusing to replace existing vendor destination (errno={ctypes.get_errno()})")


