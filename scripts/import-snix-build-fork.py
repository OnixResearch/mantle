#!/usr/bin/env python3
"""Reconstruct the Mantle-owned snix-build fork from an immutable Mantle tree.

This is NOT an upstream Snix import: upstream revision and original import
mechanism are unknown. The unmodified source is a pinned Mantle commit/tree;
tracked patch files are applied in order, then the exact result is checked.
Requires a full-history Mantle clone with the pinned commit available plus
the repo dev shell's b3sum (BLAKE3 v1.8.7). Explicit sync accepts only a
pinned unmodified vendor preimage in an existing full-history checkout;
it stages and verifies the entire postimage before five atomic file replaces.
No fetch, network access, or repository writes are implicit.
"""

import argparse
import io
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parent.parent
SOURCE_REV = "da00f58425740adea559ef926c9dfa97b2cb8240"
SOURCE_TREE = "a0979715301705e77c8567cb1bea054dc996000c"
SOURCE_PREFIX = PurePosixPath("vendor/snix-build")
PATCHES = ["patches/snix-build-watch-pidfd.patch"]
# BLAKE3 over sorted (relative path, mode, byte length, bytes) entries.
POSTIMAGE_BLAKE3 = "c6c6578b808adf15551e55f733f7702d3ec59899f22599f58e7a4a48a1a4a897"
PATCHED_FILES = frozenset({
    "Cargo.toml",
    "src/buildservice/bwrap.rs",
    "src/buildservice/mod.rs",
    "src/bwrap/mod.rs",
    "src/bwrap/watch_cancel.rs",
})
MAX_ARCHIVE_BYTES = 16 * 1024 * 1024
MAX_FILES = 1024


def git(*args, cwd=ROOT):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    ).stdout


def pinned_source():
    if git("rev-parse", "--is-shallow-repository").strip() != b"false":
        raise ValueError("shallow clone: fetch complete Mantle history before importing the local fork")
    try:
        tree = git("rev-parse", f"{SOURCE_REV}:vendor/snix-build").strip().decode("ascii")
    except subprocess.CalledProcessError as exc:
        raise ValueError("pinned Mantle source revision unavailable; use a full-history clone") from exc
    if tree != SOURCE_TREE:
        raise ValueError(f"Mantle snix-build source tree mismatch: {tree}")
    archive = git("archive", "--format=tar", SOURCE_REV, "vendor/snix-build")
    if len(archive) > MAX_ARCHIVE_BYTES:
        raise ValueError("pinned fork archive exceeds bounded import")
    return archive


def unpack_source(archive, destination):
    count = 0
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as source:
        for member in source:
            if member.isdir():
                continue
            relative = PurePosixPath(member.name)
            if not member.isfile() or SOURCE_PREFIX not in relative.parents:
                raise ValueError(f"unrecognized pinned fork archive member: {member.name}")
            if any(part in (".", "..", "") for part in relative.parts):
                raise ValueError(f"unsafe pinned fork archive member: {member.name}")
            count += 1
            if count > MAX_FILES or member.size > MAX_ARCHIVE_BYTES:
                raise ValueError("pinned fork archive exceeds file bounds")
            target = destination.joinpath(*relative.parts)
            target.parent.mkdir(parents=True, exist_ok=True)
            handle = source.extractfile(member)
            if handle is None:
                raise ValueError(f"missing pinned fork archive member: {member.name}")
            data = handle.read(member.size + 1)
            if len(data) != member.size:
                raise ValueError(f"truncated pinned fork archive member: {member.name}")
            target.write_bytes(data)
            target.chmod(0o755 if member.mode & 0o111 else 0o644)
    if not count:
        raise ValueError("empty pinned fork archive")


def apply_patches(destination):
    for relative in PATCHES:
        patch = ROOT / relative
        if not patch.is_file():
            raise ValueError(f"missing tracked local fork patch: {relative}")
        subprocess.run(
            ["git", "apply", "--check", "--whitespace=error", str(patch)],
            cwd=destination, check=True, capture_output=True,
        )
        subprocess.run(
            ["git", "apply", "--whitespace=error", str(patch)],
            cwd=destination, check=True, capture_output=True,
        )


def contents(directory):
    result = {}
    base = directory / "vendor" / "snix-build"
    if not base.is_dir():
        raise ValueError(f"local fork tree is missing: {base}")
    for entry in base.rglob("*"):
        if entry.is_symlink() or (not entry.is_file() and not entry.is_dir()):
            raise ValueError(f"unsupported local fork entry: {entry}")
        if entry.is_file():
            result[entry.relative_to(base).as_posix()] = (
                0o755 if os.access(entry, os.X_OK) else 0o644,
                entry.read_bytes(),
            )
            if len(result) > MAX_FILES:
                raise ValueError("local fork postimage exceeds file bound")
    if not result:
        raise ValueError("local fork tree is empty")
    return result


def manifest_digest(entries):
    framing = bytearray()
    for path, (mode, data) in sorted(entries.items()):
        framing.extend(path.encode("utf-8") + b"\0")
        framing.extend(f"{mode:o}".encode("ascii") + b"\0")
        framing.extend(len(data).to_bytes(8, "big"))
        framing.extend(data)
    return subprocess.run(["b3sum", "--no-names"], input=framing, check=True, capture_output=True).stdout.decode("ascii").strip()


def require_full_checkout(target):
    if not target.is_dir():
        raise ValueError("sync/check destination must be an existing full-history Git checkout")
    try:
        checkout = Path(git("rev-parse", "--show-toplevel", cwd=target).decode("utf-8").strip()).resolve()
        shallow = git("rev-parse", "--is-shallow-repository", cwd=target).strip()
        source_tree = git("rev-parse", f"{SOURCE_REV}:vendor/snix-build", cwd=target).strip().decode("ascii")
    except subprocess.CalledProcessError as exc:
        raise ValueError("destination cannot prove pinned Mantle source history") from exc
    if checkout != target or shallow != b"false" or source_tree != SOURCE_TREE:
        raise ValueError("destination is not the exact full-history Mantle checkout")


def sync_existing_checkout(target, archive, pristine, expected, digest):
    require_full_checkout(target)
    changed = {path for path in set(pristine) | set(expected) if pristine.get(path) != expected.get(path)}
    if changed != PATCHED_FILES:
        raise ValueError(f"tracked fork patch changed unexpected files: {sorted(changed)}")
    current = contents(target)
    if current == expected:
        print(f"local snix-build fork already synchronized tree={SOURCE_TREE} patched_blake3={digest}")
        return
    if current != pristine:
        differing = sorted(set(current) ^ set(pristine) | {
            path for path in current.keys() & pristine.keys() if current[path] != pristine[path]
        })
        raise ValueError(f"sync target vendor preimage differs from pinned source: {differing[:16]}")

    # Stage the complete postimage on the target filesystem and compare it
    # before replacing *any* of the five reviewed files. Never remove or
    # overwrite the vendored directory as a unit.
    with tempfile.TemporaryDirectory(prefix=".mantle-snix-sync-", dir=target) as stage_name:
        stage = Path(stage_name)
        # Keep git-apply inside this disposable nested repository: the stage
        # lives under the target checkout to guarantee same-filesystem renames.
        subprocess.run(["git", "init", "--quiet"], cwd=stage, check=True, capture_output=True)
        unpack_source(archive, stage)
        apply_patches(stage)
        staged = contents(stage)
        if staged != expected or manifest_digest(staged) != digest:
            raise ValueError("staged local fork does not match the pinned full postimage")
        base = target / "vendor" / "snix-build"
        rollback = stage / "rollback"
        for path in sorted(changed):
            destination = base / path
            if not destination.parent.is_dir():
                raise ValueError(f"sync destination lacks pinned parent directory: {path}")
            if path in pristine:
                saved = rollback / path
                saved.parent.mkdir(parents=True, exist_ok=True)
                mode, data = pristine[path]
                saved.write_bytes(data)
                saved.chmod(mode)
            elif destination.exists() or destination.is_symlink():
                raise ValueError(f"sync new destination is already occupied: {path}")
        if contents(target) != pristine:
            raise ValueError("sync target vendor changed while postimage was staged")

        applied = []
        try:
            for path in sorted(changed):
                destination = base / path
                before = pristine.get(path)
                if before is None:
                    if destination.exists() or destination.is_symlink():
                        raise ValueError(f"sync new destination changed during staging: {path}")
                elif (
                    destination.is_symlink() or not destination.is_file()
                    or (0o755 if os.access(destination, os.X_OK) else 0o644, destination.read_bytes()) != before
                ):
                    raise ValueError(f"sync existing destination changed during staging: {path}")
                os.replace(stage / "vendor" / "snix-build" / path, destination)
                applied.append(path)
            final = contents(target)
            if final != expected or manifest_digest(final) != digest:
                raise ValueError("sync postimage differs from pinned local fork")
        except BaseException as failure:
            rollback_errors = []
            for path in reversed(applied):
                destination = base / path
                try:
                    if path in pristine:
                        os.replace(rollback / path, destination)
                    else:
                        destination.unlink()
                except OSError as error:
                    rollback_errors.append(f"{path}: {error}")
            if rollback_errors:
                raise RuntimeError(f"sync rollback failed; freeze vendor for incident review: {rollback_errors}") from failure
            raise
    print(f"synchronized local snix-build fork to {target} tree={SOURCE_TREE} patched_blake3={digest}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("check", "import", "sync", "print-digest"))
    parser.add_argument("--destination", type=Path, help="fresh import root, or existing full-history checkout for check/sync")
    args = parser.parse_args()
    if args.mode in ("import", "sync") and args.destination is None:
        parser.error(f"{args.mode} requires --destination")
    if args.mode == "print-digest" and args.destination is not None:
        parser.error("--destination is only accepted in check/import/sync mode")

    archive = pinned_source()
    with tempfile.TemporaryDirectory(prefix="mantle-snix-fork-") as name:
        reconstructed = Path(name)
        unpack_source(archive, reconstructed)
        pristine = contents(reconstructed)
        apply_patches(reconstructed)
        expected = contents(reconstructed)
        digest = manifest_digest(expected)
        if POSTIMAGE_BLAKE3 and digest != POSTIMAGE_BLAKE3:
            raise ValueError(f"patched local fork BLAKE3 mismatch: {digest}")
        if args.mode == "print-digest":
            print(digest)
            return
        if not POSTIMAGE_BLAKE3:
            raise ValueError("unreviewed local fork patch has no pinned postimage digest")
        if args.mode == "check":
            target = args.destination.resolve() if args.destination is not None else ROOT
            require_full_checkout(target)
            current = contents(target)
            if current != expected:
                differing = sorted(set(current) ^ set(expected) | {
                    path for path in current.keys() & expected.keys() if current[path] != expected[path]
                })
                raise ValueError(f"vendored local fork drift: {differing[:16]}")
            print(f"local snix-build fork OK checkout={target} tree={SOURCE_TREE} patched_blake3={digest}")
            return
        if args.mode == "sync":
            sync_existing_checkout(args.destination.resolve(), archive, pristine, expected, digest)
            return
        target = args.destination.resolve()
        if target.exists():
            raise ValueError("import destination must not already exist")
        target.mkdir(parents=True)
        for path, (mode, data) in expected.items():
            result = target / "vendor" / "snix-build" / path
            result.parent.mkdir(parents=True, exist_ok=True)
            result.write_bytes(data)
            result.chmod(mode)
        if manifest_digest(contents(target)) != digest:
            raise ValueError("imported local fork did not match pinned postimage")
        print(f"imported local snix-build fork to {target} patched_blake3={digest}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError, tarfile.TarError) as exc:
        sys.exit(f"local snix-build fork import rejected: {exc}")
