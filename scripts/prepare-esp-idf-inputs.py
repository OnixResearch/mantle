#!/usr/bin/env python3
"""Acquisition-only snapshot and content inventory helpers; never called by builds."""
import argparse
import importlib.util
import json
from pathlib import Path
import os
import subprocess

REPO = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("esp_driver", REPO / "crates/crunch-esp-idf/src/driver.py")
DRIVER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DRIVER)
COMMIT = "fcae32885b0296b32044cb99ecbdc50d98dddb83"


def git(executable, source, *args):
    return subprocess.check_output([executable, "-C", str(source), *args], text=True)


def export_repository(executable, source, destination):
    destination.mkdir()
    records = git(executable, source, "ls-tree", "-r", "-z", "--full-tree", "HEAD")
    for record in records.split("\0"):
        if not record:
            continue
        metadata, member = record.split("\t", 1)
        mode, kind, object_id = metadata.split()
        if member.startswith("/") or ".." in Path(member).parts:
            raise RuntimeError("Git tree path escape")
        target = destination / member
        target.parent.mkdir(parents=True, exist_ok=True)
        if mode == "160000" and kind == "commit":
            if git(executable, source / member, "rev-parse", "HEAD").strip() != object_id:
                raise RuntimeError("Git submodule object mismatch: " + member)
            export_repository(executable, source / member, target)
        elif kind == "blob" and mode in ("100644", "100755", "120000"):
            data = subprocess.check_output([executable, "-C", str(source), "cat-file", "blob", object_id])
            if mode == "120000":
                os.symlink(data.decode("utf-8"), target)
            else:
                target.write_bytes(data)
                target.chmod(0o755 if mode == "100755" else 0o644)
        else:
            raise RuntimeError("unsupported Git tree mode: " + mode)


def snapshot(args):
    source = Path(args.source).resolve(strict=True)
    destination = Path(args.destination).absolute()
    if destination.exists():
        raise RuntimeError("refusing to overwrite snapshot")
    if git(args.git, source, "rev-parse", "HEAD").strip() != COMMIT:
        raise RuntimeError("SDK checkout is not the supported exact commit")
    if git(args.git, source, "status", "--porcelain", "--untracked-files=all"):
        raise RuntimeError("SDK checkout is dirty")
    submodules = {}
    for line in git(args.git, source, "submodule", "status", "--recursive").splitlines():
        if not line.startswith(" "):
            raise RuntimeError("uninitialized/modified SDK submodule: " + line)
        commit, member, *_ = line.strip().split()
        if git(args.git, source / member, "status", "--porcelain", "--untracked-files=all"):
            raise RuntimeError("dirty SDK submodule: " + member)
        submodules[member] = commit
    if not submodules:
        raise RuntimeError("recursive SDK submodules were not acquired")
    # Export committed blobs, never ignored venv/build/cache bytes or smudge
    # filters from the working checkout. Gitlinks recurse into verified HEADs.
    export_repository(args.git, source, destination)
    (destination / "version.txt").write_text("v5.5.1\n")
    (destination / "mantle-source.json").write_bytes(DRIVER.json_bytes({
        "release": "5.5.1", "commit": COMMIT, "submodules": submodules,
        "recipe": "recursive committed blobs/gitlinks; omit .git and ignored files; version.txt=v5.5.1"}))


def inventory(args):
    plan = json.loads(Path(args.plan).read_text())
    roots = [Path(role["root"]).resolve(strict=True) for role in plan["roles"].values()]
    pins = {name: {"root": role["root"], "tree_blake3": DRIVER.inventory(Path(role["root"]), roots)}
            for name, role in plan["roles"].items()}
    print(json.dumps(pins, indent=2, sort_keys=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    sdk = commands.add_parser("snapshot-sdk")
    sdk.add_argument("--git", required=True, help="explicit acquisition Git executable")
    sdk.add_argument("source")
    sdk.add_argument("destination")
    sdk.set_defaults(action=snapshot)
    inv = commands.add_parser("inventory")
    inv.add_argument("plan")
    inv.set_defaults(action=inventory)
    args = parser.parse_args()
    args.action(args)


if __name__ == "__main__":
    main()
