#!/usr/bin/env python3
"""Read-only e9a58881 source/capture identity audit; no source is executed."""
from __future__ import annotations

import collections
import hashlib
import json
import os
import stat
import subprocess
import sys
from pathlib import Path

RUN = "/home/brittonr/sbfp-20261113-berdKH"
FROZEN = f"{RUN}/runner-source-e9a58881-typed-index"
CAPTURE = f"{RUN}/aws-original-source-02561621.TGISLN"
ORIGINAL = f"{CAPTURE}/original"
GIT_DIR = f"{CAPTURE}/aws-lc.git"
ORIGINAL_COPY = (
    f"{FROZEN}/bootstrap/source-inputs/"
    "aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
)
REVISION = "02561621ffa4cf17c0c4f70bc11a82df36b42ae9"
TREE = "969d75db5c0494de4a26bbb2dcbf6837058054c1"
FROZEN_TREE_B3 = "e62509cd40ab6e4b202dce425a40483d74101210c6322c64f70b196ff7e177f7"
ORIGINAL_TREE_B3 = "8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89"
CODE_TREE_B3 = "1ea2223d9f992ed98f511fe4489cc415002e2fcb33718b0f7181331a0b6db1df"
CODE_ARCHIVE_B3 = "494a677f912de7f25d9efadde55ba6bcc3052bf9193a80022332a1da810ee4a5"
CODE_RAW_B3 = "fca622321dec330f848520f1fc6e9f89561869b4c4453ad7bb14f2f6a092f13f"
ORIGINAL_ARCHIVE_B3 = "282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88"
COMPLETE_ARCHIVE_B3 = FROZEN_TREE_B3
COMPLETE_GZIP_B3 = "02cb82a404b09b3b85aec971aacf3437271f203157e9e7acd296aaeee1624819"
EXPECTED_FILE_COUNT = 10427
B3 = "/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum"
GIT = "/run/current-system/sw/bin/git"
TAR = "/run/current-system/sw/bin/tar"
JQ = "/run/current-system/sw/bin/jq"
GZIP = "/run/current-system/sw/bin/gzip"
SCRATCH = "/tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115"

# Frozen source inputs for generated Mantle helpers/translators and their Rust
# authority. These hashes identify source; generated cohort members are deferred.
SELECTED_SOURCE_FILES = [
    "src/source_built_mantle_programs.rs",
    "src/source_built_mantle_programs_graph.rs",
    "src/source_built_mantle_programs_metadata.rs",
    "src/source_built_mantle_programs_metadata_helper.rs",
    "src/source_built_mantle_programs_native.rs",
    "src/source_built_mantle_programs_native_helper.rs",
    "src/source_built_mantle_programs_compression.rs",
    "src/source_built_mantle_programs_native_request.rs",
    "src/source_built_mantle_programs_aws.rs",
    "src/source_built_mantle_programs_aws_helper.rs",
    "src/source_built_fixed_point.rs",
    "src/source_built_fixed_point_planning.rs",
    "src/source_built_fixed_point_stagex_actions.rs",
    "src/source_built_fixed_point_rust_actions.rs",
    "src/source_built_fixed_point_mantle_actions.rs",
    "src/source_built_fixed_point_native_actions.rs",
    "src/source_built_fixed_point_shell.rs",
    "src/source_built_fixed_point_authoring.rs",
    "src/source_built_fixed_point_action_trust.rs",
    "src/source_built_fixed_point_action_controller.rs",
    "src/source_built_program_authoring.rs",
    "src/source_built_recipe.rs",
]


def run(argv: list[str], *, input_bytes: bytes | None = None) -> bytes:
    return subprocess.run(argv, input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True).stdout


def b3_file(path: str) -> str:
    return run([B3, "--no-names", path]).decode("ascii").strip().split()[0]


def normalized_tar_b3(root: str, *, exclude_code_inputs: bool = False) -> str:
    args = [
        TAR,
        "--sort=name",
        "--mtime=@0",
        "--owner=0",
        "--group=0",
        "--numeric-owner",
        "--format=gnu",
    ]
    if exclude_code_inputs:
        # The original source was added after the code-only tree hash, and its
        # newly-created parent was bootstrap/source-inputs.
        args.append("--exclude=./bootstrap/source-inputs")
    args.extend(["--create", "--file", "-", "--directory", root, "."])
    tar = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert tar.stdout is not None
    hasher = subprocess.Popen([B3, "--no-names"], stdin=tar.stdout, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    tar.stdout.close()
    hash_out, hash_err = hasher.communicate()
    tar_err = tar.stderr.read() if tar.stderr is not None else b""
    tar_status = tar.wait()
    if tar_status != 0 or hasher.returncode != 0:
        raise RuntimeError(f"normalized tar failed: tar={tar_status} {tar_err!r}; b3={hasher.returncode} {hash_err!r}")
    return hash_out.decode("ascii").strip().split()[0]


def b3_gzip_payload(path: str) -> str:
    decompressor = subprocess.Popen([GZIP, "--decompress", "--stdout", path], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert decompressor.stdout is not None
    hasher = subprocess.Popen([B3, "--no-names"], stdin=decompressor.stdout, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    decompressor.stdout.close()
    hash_out, hash_err = hasher.communicate()
    gzip_err = decompressor.stderr.read() if decompressor.stderr is not None else b""
    gzip_status = decompressor.wait()
    if gzip_status != 0 or hasher.returncode != 0:
        raise RuntimeError(f"gzip payload hash failed: gzip={gzip_status} {gzip_err!r}; b3={hasher.returncode} {hash_err!r}")
    return hash_out.decode("ascii").strip().split()[0]


def capture_metadata() -> dict:
    with open(f"{CAPTURE}/original-source-capture.json", "rb") as file:
        return json.load(file)


def assembly_metadata() -> dict:
    with open(f"{RUN}/complete-source-e9a58881-typed-index.json", "rb") as file:
        return json.load(file)


def parse_git_tree() -> list[dict]:
    raw = run([GIT, f"--git-dir={GIT_DIR}", "ls-tree", "-r", "-z", "--full-tree", REVISION])
    result = []
    for record in raw.split(b"\0"):
        if not record:
            continue
        meta, path = record.split(b"\t", 1)
        mode, obj_type, oid = meta.split(b" ", 2)
        if not path or path.startswith(b"/") or any(part in (b"", b".", b"..") for part in path.split(b"/")):
            raise RuntimeError(f"unsafe Git tree path {path!r}")
        result.append({"mode": mode.decode("ascii"), "type": obj_type.decode("ascii"), "oid": oid.decode("ascii"), "path": path})
    if len({entry["path"] for entry in result}) != len(result):
        raise RuntimeError("duplicate path in Git tree")
    return result


def actual_inventory(root: bytes) -> tuple[dict[bytes, os.stat_result], set[bytes]]:
    root_stat = os.lstat(root)
    if not stat.S_ISDIR(root_stat.st_mode) or stat.S_ISLNK(root_stat.st_mode):
        raise RuntimeError(f"root is not a nonsymlink directory: {os.fsdecode(root)}")
    leaves: dict[bytes, os.stat_result] = {}
    directories: set[bytes] = set()
    for directory, names, files in os.walk(root, topdown=True, followlinks=False):
        for name in list(names):
            full = os.path.join(directory, name)
            rel = os.path.relpath(full, root)
            item = os.lstat(full)
            if stat.S_ISDIR(item.st_mode):
                directories.add(rel)
            else:
                leaves[rel] = item
                names.remove(name)
        for name in files:
            full = os.path.join(directory, name)
            rel = os.path.relpath(full, root)
            leaves[rel] = os.lstat(full)
    return leaves, directories


def expected_directories(entries: list[dict]) -> set[bytes]:
    result: set[bytes] = set()
    for entry in entries:
        parent = entry["path"].rpartition(b"/")[0]
        while parent:
            result.add(parent)
            parent = parent.rpartition(b"/")[0]
    return result


def path_sample(values: set[bytes], count: int = 12) -> list[str]:
    return [os.fsdecode(path) for path in sorted(values)[:count]]


def audit_copy(root_text: str, entries: list[dict]) -> dict:
    root = os.fsencode(root_text)
    leaves, dirs = actual_inventory(root)
    expected_paths = {entry["path"] for entry in entries}
    expected_dirs = expected_directories(entries)
    missing = expected_paths - set(leaves)
    extra = set(leaves) - expected_paths
    missing_dirs = expected_dirs - dirs
    extra_dirs = dirs - expected_dirs
    mode_counts = collections.Counter()
    mode_errors: list[str] = []
    actual_file_bytes = 0
    for entry in entries:
        path = entry["path"]
        item = leaves.get(path)
        if item is None:
            continue
        mode_counts[f"{stat.S_IMODE(item.st_mode):04o}"] += 1
        if entry["mode"] == "100644":
            expected_mode = 0o644
        elif entry["mode"] == "100755":
            expected_mode = 0o755
        else:
            mode_errors.append(f"Git mode {entry['mode']} at {os.fsdecode(path)}")
            continue
        if not stat.S_ISREG(item.st_mode) or stat.S_ISLNK(item.st_mode):
            mode_errors.append(f"not regular file at {os.fsdecode(path)}")
        elif stat.S_IMODE(item.st_mode) != expected_mode:
            mode_errors.append(f"mode {stat.S_IMODE(item.st_mode):04o} != {expected_mode:04o} at {os.fsdecode(path)}")
        actual_file_bytes += item.st_size
    # Stream every actual member against its exact Git blob object. The batch
    # reader also recomputes the SHA-1 blob object ID from the received bytes.
    cat = subprocess.Popen([GIT, f"--git-dir={GIT_DIR}", "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    assert cat.stdin is not None and cat.stdout is not None
    compared = 0
    blob_bytes = 0
    content_mismatches: list[str] = []
    object_errors: list[str] = []
    expected_mode_counts = collections.Counter(entry["mode"] for entry in entries)
    if any(entry["type"] != "blob" or entry["mode"] not in ("100644", "100755") for entry in entries):
        object_errors.extend(f"unexpected {entry['type']} mode={entry['mode']} path={os.fsdecode(entry['path'])}" for entry in entries if entry["type"] != "blob" or entry["mode"] not in ("100644", "100755"))
    for entry in entries:
        oid = entry["oid"].encode("ascii")
        cat.stdin.write(oid + b"\n")
        cat.stdin.flush()
        header = cat.stdout.readline()
        parts = header.rstrip(b"\n").split(b" ")
        if len(parts) != 3 or parts[0] != oid or parts[1] != b"blob":
            object_errors.append(f"invalid/missing Git blob response for {entry['oid']}: {header!r}")
            continue
        try:
            size = int(parts[2])
        except ValueError:
            object_errors.append(f"invalid Git blob size response for {entry['oid']}: {header!r}")
            continue
        blob_bytes += size
        git_hash = hashlib.sha1()
        git_hash.update(b"blob " + str(size).encode("ascii") + b"\0")
        path = entry["path"]
        item = leaves.get(path)
        compare_file = item is not None and stat.S_ISREG(item.st_mode)
        file = open(os.path.join(root, path), "rb") if compare_file else None
        if compare_file:
            compared += 1
            if item.st_size != size:
                content_mismatches.append(f"size {item.st_size} != {size} at {os.fsdecode(path)}")
        offset = 0
        while offset < size:
            chunk = cat.stdout.read(min(1024 * 1024, size - offset))
            if not chunk:
                object_errors.append(f"truncated Git blob {entry['oid']}")
                break
            git_hash.update(chunk)
            if file is not None:
                local = file.read(len(chunk))
                if local != chunk:
                    content_mismatches.append(f"content differs at {os.fsdecode(path)}")
                    # Avoid listing the same path more than once.
                    content_mismatches = list(dict.fromkeys(content_mismatches))
            offset += len(chunk)
        if file is not None:
            if file.read(1):
                content_mismatches.append(f"extra trailing bytes at {os.fsdecode(path)}")
            file.close()
        if cat.stdout.read(1) != b"\n":
            object_errors.append(f"missing Git blob record delimiter for {entry['oid']}")
        if git_hash.hexdigest() != entry["oid"]:
            object_errors.append(f"Git blob object hash mismatch for {entry['oid']}")
    cat.stdin.close()
    if cat.stdout.read(1):
        object_errors.append("unexpected trailing Git cat-file output")
    cat_status = cat.wait()
    if cat_status:
        object_errors.append(f"git cat-file --batch exited {cat_status}")
    gitlinks = sum(1 for entry in entries if entry["mode"] == "160000" or entry["type"] == "commit")
    symlinks = sum(1 for entry in entries if entry["mode"] == "120000")
    return {
        "root": root_text,
        "git_entries": len(entries),
        "actual_leaf_entries": len(leaves),
        "expected_directories": len(expected_dirs),
        "actual_directories": len(dirs),
        "missing_files": len(missing),
        "extra_files": len(extra),
        "missing_directories": len(missing_dirs),
        "extra_directories": len(extra_dirs),
        "missing_file_sample": path_sample(missing),
        "extra_file_sample": path_sample(extra),
        "missing_directory_sample": path_sample(missing_dirs),
        "extra_directory_sample": path_sample(extra_dirs),
        "git_modes": dict(sorted(expected_mode_counts.items())),
        "actual_mode_counts": dict(sorted(mode_counts.items())),
        "mode_errors": mode_errors[:12],
        "gitlinks": gitlinks,
        "git_symlinks": symlinks,
        "actual_symlinks": sum(1 for item in leaves.values() if stat.S_ISLNK(item.st_mode)),
        "actual_special_files": sum(1 for item in leaves.values() if not stat.S_ISREG(item.st_mode) and not stat.S_ISLNK(item.st_mode)),
        "git_blob_bytes": blob_bytes,
        "actual_regular_file_bytes": actual_file_bytes,
        "blobs_content_compared": compared,
        "content_mismatches": list(dict.fromkeys(content_mismatches))[:12],
        "object_errors": object_errors[:12],
        "inventory_exact": not (missing or extra or missing_dirs or extra_dirs),
        "blob_content_exact": compared == len(entries) and not content_mismatches and not object_errors,
        "mode_exact": not mode_errors,
    }


def source_identities() -> list[dict]:
    result = []
    for relative in SELECTED_SOURCE_FILES:
        path = os.path.join(FROZEN, relative)
        item = os.stat(path, follow_symlinks=False)
        if not stat.S_ISREG(item.st_mode):
            raise RuntimeError(f"selected source is not a regular file: {relative}")
        result.append({"path": relative, "bytes": item.st_size, "mode": f"{stat.S_IMODE(item.st_mode):04o}", "blake3": b3_file(path)})
    return result


def main() -> int:
    errors: list[str] = []
    checks: dict[str, dict] = {}
    capture = capture_metadata()
    assembly = assembly_metadata()
    expected_capture = {
        "revision": REVISION,
        "git_tree_sha1": TREE,
        "archive_blake3": ORIGINAL_ARCHIVE_B3,
        "normalized_tar_blake3": ORIGINAL_TREE_B3,
        "tracked_file_count": EXPECTED_FILE_COUNT,
        "git_object_modes": "100644\n100755",
        "gitlinks_absent": True,
    }
    for field, expected in expected_capture.items():
        checks[f"capture_metadata_{field}"] = {"observed": capture.get(field), "expected": expected, "pass": capture.get(field) == expected}
        if capture.get(field) != expected:
            errors.append(f"original capture metadata {field} mismatch")
    expected_assembly = {
        "schema": "mantle-complete-source-snapshot-v1",
        "source_code_revision": "e9a58881b9ff491f47864632367c9b8ea4af76e3",
        "code_archive_blake3": CODE_ARCHIVE_B3,
        "code_tree_blake3": CODE_TREE_B3,
        "original_git_commit": REVISION,
        "original_git_tree_sha1": TREE,
        "original_source_archive_blake3": ORIGINAL_ARCHIVE_B3,
        "original_source_normalized_tar_blake3": ORIGINAL_TREE_B3,
        "complete_archive_blake3": COMPLETE_ARCHIVE_B3,
        "compressed_archive_blake3": COMPLETE_GZIP_B3,
    }
    for field, expected in expected_assembly.items():
        checks[f"assembly_metadata_{field}"] = {"observed": assembly.get(field), "expected": expected, "pass": assembly.get(field) == expected}
        if assembly.get(field) != expected:
            errors.append(f"complete source assembly metadata {field} mismatch")
    git_show = run([GIT, f"--git-dir={GIT_DIR}", "show", "-s", "--format=%H %T", REVISION]).decode("ascii").strip()
    git_type = run([GIT, f"--git-dir={GIT_DIR}", "cat-file", "-t", REVISION]).decode("ascii").strip()
    git_tree = run([GIT, f"--git-dir={GIT_DIR}", "rev-parse", f"{REVISION}^{{tree}}"]).decode("ascii").strip()
    checks["git_commit"] = {"observed": git_show, "expected": f"{REVISION} {TREE}", "pass": git_show == f"{REVISION} {TREE}"}
    checks["git_commit_type"] = {"observed": git_type, "expected": "commit", "pass": git_type == "commit"}
    checks["git_tree"] = {"observed": git_tree, "expected": TREE, "pass": git_tree == TREE}
    if git_show != f"{REVISION} {TREE}" or git_type != "commit" or git_tree != TREE:
        errors.append("Git commit/tree binding mismatch")

    source_code_archive = f"{RUN}/incoming/source-e9a58881b9ff491f47864632367c9b8ea4af76e3.tar.gz"
    original_archive = f"{CAPTURE}/aws-lc-original.tar"
    complete_archive = f"{RUN}/incoming/complete-source-e9a58881-typed-index.tar"
    complete_gzip = complete_archive + ".gz"
    archive_identities = {
        "source_code_archive_gzip": (b3_file(source_code_archive), CODE_ARCHIVE_B3),
        "source_code_archive_raw_payload": (b3_gzip_payload(source_code_archive), CODE_RAW_B3),
        "original_source_archive": (b3_file(original_archive), ORIGINAL_ARCHIVE_B3),
        "complete_source_archive": (b3_file(complete_archive), COMPLETE_ARCHIVE_B3),
        "complete_source_archive_gzip": (b3_file(complete_gzip), COMPLETE_GZIP_B3),
    }
    for name, (observed, expected) in archive_identities.items():
        checks[f"archive_{name}"] = {"observed": observed, "expected": expected, "pass": observed == expected}
        if observed != expected:
            errors.append(f"archive identity mismatch for {name}")

    tree_hashes = {
        "frozen_complete_tree": (normalized_tar_b3(FROZEN), FROZEN_TREE_B3),
        "frozen_code_tree_without_original_source_inputs": (normalized_tar_b3(FROZEN, exclude_code_inputs=True), CODE_TREE_B3),
        "captured_original_tree": (normalized_tar_b3(ORIGINAL), ORIGINAL_TREE_B3),
        "frozen_original_copy_tree": (normalized_tar_b3(ORIGINAL_COPY), ORIGINAL_TREE_B3),
    }
    for name, (observed, expected) in tree_hashes.items():
        checks[f"tree_{name}"] = {"observed": observed, "expected": expected, "pass": observed == expected}
        if observed != expected:
            errors.append(f"normalized tree identity mismatch for {name}")

    entries = parse_git_tree()
    if len(entries) != EXPECTED_FILE_COUNT:
        errors.append(f"Git tree file count {len(entries)} != {EXPECTED_FILE_COUNT}")
    mode_counts = collections.Counter(entry["mode"] for entry in entries)
    type_counts = collections.Counter(entry["type"] for entry in entries)
    if set(mode_counts) != {"100644", "100755"} or type_counts != {"blob": EXPECTED_FILE_COUNT}:
        errors.append("Git tree contains an unexpected mode or non-blob entry")
    copy_audits = [audit_copy(root, entries) for root in (ORIGINAL, ORIGINAL_COPY)]
    for audit in copy_audits:
        if not (audit["inventory_exact"] and audit["blob_content_exact"] and audit["mode_exact"]):
            errors.append(f"Git content/mode/member inventory mismatch for {audit['root']}")
        if audit["gitlinks"] or audit["git_symlinks"] or audit["actual_symlinks"] or audit["actual_special_files"]:
            errors.append(f"unexpected gitlink/symlink/special member in {audit['root']}")
        if audit["missing_files"] or audit["extra_files"] or audit["missing_directories"] or audit["extra_directories"]:
            errors.append(f"member inventory mismatch for {audit['root']}")

    identities = source_identities()
    result = {
        "schema": "luna-frozen-originals-audit-v1",
        "scope": "immutable frozen complete source and original AWS-LC capture only; no cohort read",
        "host": "leviathan.cymric-daggertooth.ts.net",
        "frozen_root": FROZEN,
        "frozen_tree_blake3": tree_hashes["frozen_complete_tree"][0],
        "original_capture_root": ORIGINAL,
        "frozen_original_copy": ORIGINAL_COPY,
        "original_git_commit": REVISION,
        "original_git_tree": TREE,
        "git_entry_count": len(entries),
        "git_mode_counts": dict(sorted(mode_counts.items())),
        "git_object_type_counts": dict(sorted(type_counts.items())),
        "git_blob_bytes": copy_audits[0]["git_blob_bytes"] if copy_audits else 0,
        "copy_audits": copy_audits,
        "archive_identities": {name: {"blake3": observed, "expected": expected, "pass": observed == expected} for name, (observed, expected) in archive_identities.items()},
        "tree_hashes": {name: {"blake3": observed, "expected": expected, "pass": observed == expected} for name, (observed, expected) in tree_hashes.items()},
        "selected_frozen_source_identities": identities,
        "checks": checks,
        "errors": errors,
        "limitations": [
            "No generated program/helper archive, cohort manifest/profile/plan, role directories, records, NARs, payloads, recipe, provider, or controller was read or executed.",
            "Selected hashes identify frozen generator/helper/translator/Rust-authority source files only; generated cohort members and source-to-cohort copy parity remain deferred.",
            "No prepared-profile, launch, fixed-point, compiler-correctness, reproducibility, or release claim is made.",
        ],
    }
    os.makedirs(SCRATCH, mode=0o700, exist_ok=True)
    report_path = os.path.join(SCRATCH, "luna-frozen-originals-result.json")
    with open(report_path, "w", encoding="utf-8") as output:
        json.dump(result, output, indent=2, sort_keys=True)
        output.write("\n")
    print(json.dumps(result, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as failure:
        print(f"AUDIT ERROR: {type(failure).__name__}: {failure}", file=sys.stderr)
        raise
