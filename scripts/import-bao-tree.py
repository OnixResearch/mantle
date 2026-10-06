#!/usr/bin/env python3
"""Import or compare the exact Bao candidate Git tree without replacing local data.

Usage: python3 scripts/import-bao-tree.py import /path/to/bao-checkout
       python3 scripts/import-bao-tree.py check [/path/to/bao-checkout]
The resulting tracked snapshot is sufficient for later offline builds; the
checkout is needed only when importing a new candidate or auditing its origin.
"""

import argparse
import json
import io
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parent.parent
DEST = ROOT / "third_party/bao-tree"
REVISION = "eecfbbb458cc684fd85e056881580d307a1d1868"
UPSTREAM_BASE = "2be9abd144783455606424424c29bd3a57f926f8"


def git(checkout, *args):
    return subprocess.run(
        ["git", "-C", str(checkout), *args], capture_output=True, check=True
    ).stdout


def source_files(checkout):
    revision = git(checkout, "rev-parse", "HEAD").decode().strip()
    if revision != REVISION:
        raise ValueError(f"Bao revision drift: {revision}")
    base = git(checkout, "rev-parse", f"{REVISION}^").decode().strip()
    if base != UPSTREAM_BASE:
        raise ValueError(f"Bao upstream base drift: {base}")
    if git(checkout, "status", "--porcelain"):
        raise ValueError("Bao checkout has uncommitted changes")
    archive = git(checkout, "archive", "--format=tar", REVISION)
    files = {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as tree:
        for member in tree:
            if member.isdir():
                continue
            if not member.isfile() or member.name.startswith("/") or ".." in Path(member.name).parts:
                raise ValueError(f"Unexpected Bao archive entry: {member.name}")
            files[member.name] = tree.extractfile(member).read()
    if "Cargo.toml" not in files or "src/io/validate.rs" not in files:
        raise ValueError("Bao candidate has no manifest or validation implementation")
    return files


def digest(data):
    return subprocess.run(["b3sum", "--no-names"], input=data, capture_output=True, check=True).stdout.decode().strip()


def receipt(files):
    return {
        "source": "https://github.com/n0-computer/bao-tree",
        "upstream_base": UPSTREAM_BASE,
        "candidate": REVISION,
        "files_blake3": {name: digest(data) for name, data in sorted(files.items())},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["import", "check"])
    parser.add_argument("checkout", type=Path, nargs="?")
    args = parser.parse_args()
    receipt_path = ROOT / "third_party/bao-tree-source.json"
    if args.operation == "import":
        if args.checkout is None:
            raise ValueError("Import requires the exact Bao candidate checkout")
        files = source_files(args.checkout)
        selected = receipt(files)
        if DEST.exists() or DEST.is_symlink() or receipt_path.exists():
            raise ValueError("Refusing to replace existing Bao snapshot or receipt")
        DEST.mkdir(parents=True)
        for name, data in files.items():
            destination = DEST / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        receipt_path.write_text(json.dumps(selected, indent=2, sort_keys=True) + "\n")
    else:
        selected = json.loads(receipt_path.read_text())
        if selected != {**receipt({}), "files_blake3": selected["files_blake3"]}:
            raise ValueError("Bao receipt provenance metadata drift")
        if not DEST.is_dir() or DEST.is_symlink():
            raise ValueError("Missing or unsafe Bao snapshot")
        entries = list(DEST.rglob("*"))
        if any(path.is_symlink() for path in entries):
            raise ValueError("Symlink in Bao snapshot")
        existing = {path.relative_to(DEST).as_posix() for path in entries if path.is_file()}
        expected = set(selected["files_blake3"])
        if any(name.startswith("/") or ".." in Path(name).parts for name in expected):
            raise ValueError("Invalid Bao receipt path")
        if existing != expected:
            raise ValueError(f"Bao snapshot file-set drift: missing={sorted(expected-existing)}, extra={sorted(existing-expected)}")
        for name, expected_digest in selected["files_blake3"].items():
            if digest((DEST / name).read_bytes()) != expected_digest:
                raise ValueError(f"Bao snapshot content drift: {name}")
        if args.checkout is not None and receipt(source_files(args.checkout)) != selected:
            raise ValueError("Bao candidate provenance does not match snapshot receipt")
        files = {name: (DEST / name).read_bytes() for name in expected}
    print(f"Bao {args.operation}: {len(files)} tracked files at {REVISION}; BLAKE3 file receipt {receipt_path}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError, tarfile.TarError) as error:
        print(f"import-bao-tree: {error}", file=sys.stderr)
        sys.exit(1)
