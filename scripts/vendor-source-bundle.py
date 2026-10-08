#!/usr/bin/env python3
"""Export a bounded, deterministic vendor source archive without generated vendor-deps.

The reviewed full shared hash table is a separate fixed-output input to the
content-addressed selected-table derivation; it must never enter this archive.
"""
import argparse
import gzip
import importlib.util
import io
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile
import tomllib

MAX_FILES = 65_536
MAX_FILE = 256 * 1024 * 1024
MAX_TOTAL = 1024 * 1024 * 1024
MAX_INDEX_BYTES = 128 * 1024 * 1024
INDEX_NAME = "index.crates.io-1949cf8c6b5b557f"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def relative(name):
    require(isinstance(name, str) and len(name) <= 4096 and "\\" not in name, "invalid archive path")
    value = PurePosixPath(name)
    require(name and value.as_posix() == name and not value.is_absolute()
            and all(part not in ("", ".", "..") for part in value.parts),
            f"unsafe or noncanonical archive path: {name!r}")
    return value.as_posix()


def add_tree(files, source, destination, *, allow_symlinks=False):
    require(source.is_dir() and not source.is_symlink(), f"missing declared source tree: {source}")
    for base, dirs, names in os.walk(source, followlinks=False):
        dirs.sort()
        names.sort()
        for item in dirs + names:
            path = Path(base) / item
            if path.is_symlink():
                require(allow_symlinks, f"source symlink forbidden: {path}")
                target = os.readlink(path)
                require(not os.path.isabs(target)
                        and (path.parent / target).resolve().is_relative_to(source.resolve()),
                        f"source symlink escapes declared Git checkout: {path}")
                key = relative(f"{destination}/{path.relative_to(source).as_posix()}")
                require(key not in files, f"duplicate archive path: {key}")
                files[key] = path
            else:
                require(path.is_dir() or path.is_file(), f"special source file forbidden: {path}")
                if path.is_dir():
                    key = relative(f"{destination}/{path.relative_to(source).as_posix()}")
                    require(key not in files, f"duplicate archive path: {key}")
                    files[key] = path
        for name in names:
            path = Path(base) / name
            if path.is_symlink():
                continue
            key = relative(f"{destination}/{path.relative_to(source).as_posix()}")
            require(key not in files, f"duplicate archive path: {key}")
            files[key] = path


def add_index(files, source):
    destination = f"registry/index/{INDEX_NAME}"
    if source.is_dir():
        add_tree(files, source, destination)
    else:
        require(source.is_file() and not source.is_symlink(), "missing declared sparse registry index archive")
        total = 0
        count = 0
        with tarfile.open(source, "r:gz") as archive:
            for member in archive:
                count += 1
                require(count <= MAX_FILES * 2, "sparse index member-count bound exceeded")
                if member.isdir():
                    relative(member.name.rstrip("/"))
                    continue
                require(member.isfile() and 0 <= member.size <= MAX_FILE, "unsafe or oversized registry index member")
                key = relative(f"{destination}/{member.name}")
                require(key not in files and len(files) < MAX_FILES, f"duplicate or excess registry index path: {key}")
                total += member.size
                require(total <= MAX_INDEX_BYTES, "sparse index extracted byte bound exceeded")
                extracted = archive.extractfile(member)
                require(extracted is not None, f"unreadable registry index entry: {key}")
                payload = extracted.read(MAX_FILE + 1)
                require(len(payload) == member.size, f"truncated registry index entry: {key}")
                files[key] = payload
    require(f"{destination}/config.json" in files, "missing sparse registry config.json")


def prepare(args):
    repo = Path(__file__).resolve().parent.parent
    fixture = repo / "fixtures/lock-vendor-two-crate"
    root = fixture if args.profile == "two-crate" else repo
    files = {
        "vendor-producer-build.py": repo / "scripts/vendor-producer-build.py",
        "vendor-dynamic-assemble.py": repo / "scripts/vendor-dynamic-assemble.py",
        "vendor-assemble.py": repo / "scripts/vendor-assemble.py",
        "producer-src/core/lib.rs": repo / "crates/mantle-lock-vendor-core/src/lib.rs",
        "producer-src/core/shared_table.rs": repo / "crates/mantle-lock-vendor-core/src/shared_table.rs",
        "producer-src/main.rs": repo / "crates/mantle-lock-vendor-producer/src/main.rs",
        "workspace/Cargo.toml": root / "Cargo.toml",
        "workspace/Cargo.lock": root / "Cargo.lock",
    }
    if args.profile == "two-crate":
        add_tree(files, fixture / "src", "workspace/src")
        files["workspace/.cargo/vendor-config.toml"] = fixture / ".cargo/vendor-config.toml"
        files["git-transports.json"] = fixture / "git-transports.json"
        require(args.git_cache is None and args.git_sidecars is None and args.reviewed_table is None,
                "two-crate profile does not use workspace Git sidecars or table")
    else:
        for directory in ("crates", "src", "vendor"):
            add_tree(files, repo / directory, f"workspace/{directory}")
        for name in ("build.rs", ".cargo/vendor-config.toml", "scripts/vendor-deps.py", "patches/casita-blake3-finalize.patch"):
            path = repo / name
            if name != "build.rs" or path.is_file():
                files[relative(f"workspace/{name}")] = path
        lock = tomllib.loads((root / "Cargo.lock").read_text())
        git_sources = {package["source"] for package in lock["package"] if package.get("source", "").startswith("git+")}
        require(bool(git_sources) == (args.git_cache is not None and args.git_sidecars is not None
                                     and args.reviewed_table is not None),
                "workspace Git lock sources require declared Git cache, sidecars and reviewed hash table")
        sidecars = json.loads(args.git_sidecars.read_text()) if git_sources else {"git": {}}
        require(isinstance(sidecars, dict) and set(sidecars) == {"git"} and isinstance(sidecars["git"], dict),
                "invalid Git sidecar mapping")
        missing = git_sources - sidecars["git"].keys()
        extra = sidecars["git"].keys() - git_sources
        require(not missing, f"missing locked Git sidecar: {sorted(missing)[0] if missing else ''}")
        require(not extra, f"unlocked Git sidecar: {sorted(extra)[0] if extra else ''}")
        mapping = {}
        if git_sources:
            spec = importlib.util.spec_from_file_location("verified_dynamic_vendor", repo / "scripts/vendor-dynamic-assemble.py")
            dynamic = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(dynamic)
            identities = [line.split("\t", 1)[0] for line in args.reviewed_table.read_text().splitlines()[1:]]
            require(identities == sorted(identities), "reviewed Git table identities are not canonically sorted")
            reviewed = dynamic.rows(root / "Cargo.lock", args.reviewed_table)
            groups = {}
            for row in reviewed:
                if row[3] == "recursive":
                    source = row[6].split("/", 2)[2]
                    identity = (row[2], row[4], row[5])
                    require(groups.setdefault(source, identity) == identity, "contradictory reviewed Git source")
            require(set(groups) == git_sources, "reviewed table lacks locked Git sources")
            git_environment = {"PATH": os.environ["PATH"], "HOME": "/nonexistent",
                               "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null",
                               "GIT_TERMINAL_PROMPT": "0"}
            copied = set()
            for source, data in sorted(sidecars["git"].items()):
                require(isinstance(data, dict) and set(data) == {"git_db", "checkout", "git_tree"},
                        "invalid Git sidecar mapping")
                reviewed_hash, fetch_url, revision = groups[source]
                db = args.git_cache / relative(data["git_db"])
                checkout = args.git_cache / relative(data["checkout"])
                fetched = Path(data["git_tree"])
                require(db.resolve().is_relative_to(args.git_cache.resolve())
                        and checkout.resolve().is_relative_to(args.git_cache.resolve())
                        and db.parent.name == "db" and checkout.parent.parent.name == "checkouts"
                        and checkout.parent.name == db.name and checkout.name == revision[:7],
                        "Git cache sidecar identity or path escapes declared cache")
                require(fetched.is_absolute() and fetched.is_dir() and not fetched.is_symlink(),
                        "missing explicitly declared fetched Git tree")
                dynamic.v.bounded_git_metadata(db)
                dynamic.v.bounded_git_metadata(checkout / ".git")
                require((checkout / ".cargo-ok").is_file() and not (checkout / ".cargo-ok").is_symlink(),
                        "missing pinned Git checkout completion")
                head = subprocess.run(["git", "-C", str(checkout), "rev-parse", "HEAD"],
                                      env=git_environment, capture_output=True, text=True, check=True).stdout.strip()
                origin = subprocess.run(["git", "-C", str(checkout), "remote", "get-url", "origin"],
                                        env=git_environment, capture_output=True, text=True, check=True).stdout.strip()
                require(head == revision and origin == source[4:].split("?", 1)[0].split("#", 1)[0],
                        "Git checkout revision or declared source transport drift")
                actual, _ = dynamic.v.nar_sha256(fetched)
                require(actual == reviewed_hash, f"Git NAR differs from reviewed lock table: {source}")
                dynamic.v.verify_git_revision_tree(db, revision, fetched)
                mapped = {}
                for kind in ("git_db", "checkout"):
                    suffix = relative(data[kind])
                    tree = args.git_cache.joinpath(*PurePosixPath(suffix).parts)
                    target = relative(f"git-cache/{suffix}")
                    if target not in copied:
                        add_tree(files, tree, target, allow_symlinks=kind == "checkout")
                        copied.add(target)
                    mapped[kind] = target
                mapping[source] = {**mapped, "nar_sha256": reviewed_hash,
                                   "revision": revision, "fetch_url": fetch_url}
        files["git-transports.json"] = (json.dumps({"git": mapping}, sort_keys=True, separators=(",", ":")) + "\n").encode()
    add_index(files, args.index)
    require(all(key != "shared-lock-hashes-v1" and "cargo-shared-lock-hashes-v1" not in key
                and not key.startswith("workspace/vendor-deps/") for key in files),
            "global pin table or generated vendor tree leaked into source bundle")
    require(len(files) <= MAX_FILES, "source archive file-count bound")
    return files


def export(files, output):
    require(output.parent.is_dir() and not output.exists(), "source archive output already exists or parent missing")
    require(len(files) <= MAX_FILES, "source archive entry-count bound")
    descriptor, temporary = tempfile.mkstemp(prefix=".vendor-source-", dir=output.parent)
    try:
        total = 0
        with os.fdopen(descriptor, "wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w|", format=tarfile.PAX_FORMAT) as archive:
                for name, source in sorted(files.items()):
                    relative(name)
                    if isinstance(source, Path) and source.is_symlink():
                        require(name.startswith("git-cache/checkouts/"), f"source symlink outside Git checkout: {name}")
                    info = tarfile.TarInfo(name)
                    if isinstance(source, Path) and source.is_symlink():
                        info.type = tarfile.SYMTYPE
                        info.linkname = os.readlink(source)
                        info.size = 0
                        info.mode = 0o777
                        payload = b""
                    elif isinstance(source, Path) and source.is_dir():
                        info.type = tarfile.DIRTYPE
                        info.size = 0
                        info.mode = 0o755
                        payload = b""
                    else:
                        if isinstance(source, Path):
                            require(source.is_file(), f"missing or unsafe declared source: {source}")
                            require(source.stat().st_size <= MAX_FILE, f"source file exceeds bound: {source}")
                            payload = source.read_bytes()
                        else:
                            payload = source
                        info.size = len(payload)
                        info.mode = (0o755 if isinstance(source, Path) and source.stat().st_mode & 0o111
                                     else 0o644)
                    total += len(payload)
                    require(len(payload) <= MAX_FILE and total <= MAX_TOTAL, "source archive byte bound exceeded")
                    info.uid = info.gid = info.mtime = 0
                    info.uname = info.gname = ""
                    archive.addfile(info, io.BytesIO(payload))
        require(not output.exists(), "source archive output created concurrently")
        os.link(temporary, output)
        print(json.dumps({"archive": str(output), "files": len(files), "payload_bytes": total}, sort_keys=True))
    finally:
        os.unlink(temporary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("two-crate", "workspace"), required=True)
    parser.add_argument("--index", type=Path, required=True, help="declared sparse index directory or tar.gz")
    parser.add_argument("--git-cache", type=Path, help="root containing reviewed Cargo Git db and checkouts")
    parser.add_argument("--git-sidecars", type=Path, help="JSON mapping each Cargo.lock Git source to relative Git metadata")
    parser.add_argument("--reviewed-table", type=Path, help="explicit reviewed full shared hash table; verified but never archived")
    parser.add_argument("--to", type=Path, required=True)
    args = parser.parse_args()
    export(prepare(args), args.to)


if __name__ == "__main__":
    main()
