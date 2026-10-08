#!/usr/bin/env python3
"""Assemble Mantle fixed-output results in deterministic <=240-entry shards.

A declared source bundle supplies workspace/, git-transports.json (relative
git_db and checkout paths), and registry/index/. A separate fixed-output
selected hash table supplies only the reviewed lock sources. Cargo and rustc
are from an independently declared toolchain store source.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tempfile
import tomllib

spec = importlib.util.spec_from_file_location("verified_vendor", Path(__file__).with_name("vendor-assemble.py"))
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)


def rows(lock, table):
    v.require(lock.is_file() and lock.stat().st_size <= v.MAX_LOCK, "missing or oversized Cargo.lock")
    v.require(table.is_file() and table.stat().st_size <= v.MAX_LOCK, "missing or oversized shared hash table")
    lines = table.read_text().splitlines()
    v.require(lines and lines.pop(0) == "mantle-shared-lock-hashes-v1", "wrong shared hash table version")
    hashes = {}
    for line in lines:
        parts = line.split("\t")
        v.require(len(parts) in (3, 4) and parts[0] not in hashes and parts[1] == "recursive-sha256"
                  and len(parts[2]) == 64 and all(ch in "0123456789abcdef" for ch in parts[2]),
                  "malformed shared hash")
        if len(parts) == 4:
            v.require(parts[3].startswith("https://") and not any(ch in parts[3] for ch in "@?#\\"),
                      "invalid reviewed Git HTTPS transport")
        hashes[parts[0]] = (parts[2], parts[3] if len(parts) == 4 else None)
    artifacts = []
    for package in tomllib.loads(lock.read_text())["package"]:
        source = package.get("source")
        if source is None:
            continue
        name, version = package["name"], package["version"]
        v.require(name and version and all(ch.isascii() and (ch.isalnum() or ch in ".-_+") for ch in name + version),
                  "unsafe package identity")
        if source == v.REGISTRY_SOURCE:
            sha = package["checksum"]
            artifact = (name, version, sha, "flat", f"https://static.crates.io/crates/{name}/{name}-{version}.crate", "-", "-")
        else:
            v.require(source.startswith("git+") and "checksum" not in package, "unsupported source")
            identity = f"cargo/{name}@{version}/{source}"
            revision = source.rsplit("#", 1)[-1]
            v.require(identity in hashes and len(revision) == 40 and all(ch in "0123456789abcdef" for ch in revision),
                      "unpinned Git commit or recursive hash")
            sha, reviewed_route = hashes[identity]
            original_route = source[4:].split("?", 1)[0].split("#", 1)[0]
            route = reviewed_route or original_route
            v.require(route.startswith("https://"), "missing reviewed HTTPS Git fetch route")
            artifact = (name, version, sha, "recursive", route, revision, identity)
        v.require(len(artifact[2]) == 64 and all(ch in "0123456789abcdef" for ch in artifact[2]), "invalid fixed output hash")
        artifacts.append(artifact)
    artifacts.sort(key=lambda row: (row[0], row[1]))
    v.require(0 < len(artifacts) <= v.MAX_ARTIFACTS and len({r[:2] for r in artifacts}) == len(artifacts),
              "empty, oversized, or duplicate lock inventory")
    return artifacts


def checked_fetch(row, path):
    v.require(not path.is_symlink(), "fetched output is a symlink")
    if row[3] == "flat":
        v.require(path.is_file() and path.stat().st_size <= v.MAX_INPUT_FILE and v.sha256_file(path) == row[2],
                  f"archive mismatch {row[0]}@{row[1]}")
    else:
        v.require(path.is_dir() and v.nar_sha256(path)[0] == row[2], f"Git NAR mismatch {row[0]}@{row[1]}")


def shard(options):
    artifacts = rows(options.lock, options.table)
    start = options.start
    v.require(start >= 0 and start % 240 == 0, "noncanonical shard offset")
    selected = artifacts[start:start + 240]
    v.require(selected and len(selected) == len(options.artifact), "incomplete shard")
    output = Path(options.output or os.environ["out"])
    v.require(not output.exists() and output.parent.is_dir(), "shard output already exists")
    with tempfile.TemporaryDirectory(prefix="vendor-shard-", dir=output.parent) as tmp:
        stage = Path(tmp) / "result"
        stage.mkdir()
        for row, (identity, fetch_path) in zip(selected, options.artifact):
            v.require(identity == f"{row[0]}@{row[1]}", "shard order or identity drift")
            fetched = Path(fetch_path)
            checked_fetch(row, fetched)
            target = stage / ("archives" if row[3] == "flat" else "git") / f"{row[0]}-{row[1]}"
            target.parent.mkdir(exist_ok=True)
            if row[3] == "flat":
                archive = target.parent / f"{row[0]}-{row[1]}.crate"
                shutil.copyfile(fetched, archive)
                checked_fetch(row, archive)
            else:
                shutil.copytree(fetched, target, symlinks=True)
                checked_fetch(row, target)
        (stage / "selection.json").write_text(json.dumps({"start": start, "names": [f"{r[0]}@{r[1]}" for r in selected]}, separators=(",", ":")))
        v.publish_noreplace(stage, output)


def path_under(bundle, relative):
    v.require(isinstance(relative, str) and relative and len(relative) < 1024, "invalid sidecar path")
    parsed = PurePosixPath(relative)
    v.require(not parsed.is_absolute() and all(c not in ("", ".", "..") for c in parsed.parts), "sidecar path escape")
    return bundle.joinpath(*parsed.parts)

def declared_environment(stage, home, rustc, git, patch):
    tools = [rustc.parent]
    for executable in (git, patch):
        if executable is not None:
            v.require(executable.is_file(), f"missing declared tool: {executable}")
            tools.append(executable.parent)
    return {
        "PATH": os.pathsep.join(map(str, tools)),
        "HOME": str(stage),
        "TMPDIR": str(stage),
        "CARGO_HOME": str(home),
        "CARGO_NET_OFFLINE": "true",
        "CARGO_INCREMENTAL": "0",
        "RUSTC": str(rustc),
        "LANG": "C",
        "LC_ALL": "C",
        "TZ": "UTC",
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": "/dev/null",
        "GIT_TERMINAL_PROMPT": "0",
    }



def complete(options):
    bundle = options.bundle
    v.require(bundle.is_dir() and not bundle.is_symlink(), "missing declared source bundle")
    workspace = bundle / "workspace"
    lock = workspace / "Cargo.lock"
    v.require(lock.is_file() and v.sha256_file(lock) == v.sha256_file(options.lock), "workspace lock differs from plan")
    artifacts = rows(options.lock, options.table)
    v.require(not options.artifact or not options.shard, "mixed direct and shard fetches")
    flat, trees = {}, {}
    if options.artifact:
        v.require(len(artifacts) <= 240 and len(options.artifact) == len(artifacts), "direct fetch inventory mismatch")
        for row, (identity, entry) in zip(artifacts, options.artifact):
            v.require(identity == f"{row[0]}@{row[1]}", f"direct fetch identity mismatch: {identity}")
            tree = Path(entry)
            checked_fetch(row, tree)
            (flat if row[3] == "flat" else trees)[row[:2]] = tree
    else:
        v.require(len(artifacts) > 240, "small lock must use direct verified fetches")
        v.require(len(options.shard) == (len(artifacts) + 239) // 240, "missing shard")
        for n, entry in enumerate(options.shard):
            source = Path(entry)
            v.require(source.is_dir() and not source.is_symlink(), "invalid shard")
            selected = artifacts[n * 240:(n + 1) * 240]
            manifest = json.loads((source / "selection.json").read_text())
            v.require(manifest == {"start": n * 240, "names": [f"{r[0]}@{r[1]}" for r in selected]}, "shard inventory mismatch")
            for row in selected:
                tree = source / ("archives" if row[3] == "flat" else "git") / f"{row[0]}-{row[1]}"
                tree = tree.parent / f"{row[0]}-{row[1]}.crate" if row[3] == "flat" else tree
                checked_fetch(row, tree)
                (flat if row[3] == "flat" else trees)[row[:2]] = tree
    lock_packages = tomllib.loads(lock.read_text())["package"]
    lock_by_id = {(p["name"], p["version"]): p for p in lock_packages if p.get("source")}
    v.require(len(lock_by_id) == len(artifacts), "Cargo.lock inventory changed")
    groups = {}
    for row in artifacts:
        source = lock_by_id[row[:2]]["source"]
        if row[3] == "recursive":
            record = {"sha256": row[2], "url": row[4], "revision": row[5]}
            v.require(groups.setdefault(source, record) == record, "contradictory Git source")
    mapping = json.loads((bundle / "git-transports.json").read_text())["git"]
    v.require(set(mapping) == set(groups), "Git sidecars must match selected revisions exactly")
    transports = {}
    for source, record in groups.items():
        sidecar = mapping[source]
        v.require(set(sidecar) == {"git_db", "checkout", "nar_sha256", "revision", "fetch_url"},
                  "unexpected Git sidecar fields")
        v.require((sidecar["nar_sha256"], sidecar["revision"], sidecar["fetch_url"])
                  == (record["sha256"], record["revision"], record["url"]),
                  "portable Git sidecar differs from reviewed selected lock table")
        matching = next(r for r in artifacts if r[6].endswith("/" + source))
        transports[source] = {"git_db": str(path_under(bundle, sidecar["git_db"])),
                              "checkout": str(path_under(bundle, sidecar["checkout"])),
                              "git_tree": str(trees[matching[:2]])}
    output = Path(options.output or os.environ["out"])
    v.require(not output.exists() and output.parent.is_dir(), "output exists")
    with tempfile.TemporaryDirectory(prefix="lock-vendor-", dir=output.parent) as tmp:
        stage = Path(tmp)
        home = stage / "cargo-home"
        home.mkdir()
        v.require(options.rust.is_dir() and not options.rust.is_symlink(), "missing declared Rust toolchain")
        cargo = options.rust / "bin/cargo"
        rustc = options.rust / "bin/rustc"
        v.require(cargo.is_file() and rustc.is_file(), "missing declared Cargo/Rustc executables")
        v.require((options.git is not None) == bool(groups), "Git sources require exactly one declared Git tool")
        v.require((options.patch is not None) == any(row[0] == "casita" for row in artifacts),
                  "Casita patching requires exactly one declared patch tool")
        environment = declared_environment(stage, home, rustc, options.git, options.patch)
        os.environ.clear()
        os.environ.update(environment)
        v.add_git_sources(groups, transports, home)
        cache_name = options.registry_cache_name
        index_name = options.registry_index_name
        v.require(cache_name == "index.crates.io-1949cf8c6b5b557f" and index_name == cache_name, "unreviewed registry identity")
        cache = home / "registry/cache" / cache_name
        cache.mkdir(parents=True)
        for row in artifacts:
            if row[3] == "flat":
                target = cache / f"{row[0]}-{row[1]}.crate"
                shutil.copyfile(flat[row[:2]], target)
                checked_fetch(row, target)
        index = bundle / "registry/index" / index_name
        v.require(index.is_dir() and (index / "config.json").is_file(), "missing declared sparse index")
        link = home / "registry/index" / index_name
        link.parent.mkdir(parents=True)
        link.symlink_to(index, target_is_directory=True)
        vendor = stage / "vendor-deps"
        generated = v.run([str(cargo), "vendor", "--locked", "--offline", "--manifest-path", str(workspace / "Cargo.toml"), str(vendor)], cwd=stage, environment=environment)
        generator = workspace / "scripts/vendor-deps.py"
        if generator.is_file():
            dep_spec = importlib.util.spec_from_file_location("checked_vendor_generator", generator)
            module = importlib.util.module_from_spec(dep_spec)
            dep_spec.loader.exec_module(module)
            module.apply_casita_patch(vendor)
        else:
            v.require(not (vendor / "casita").exists(), "casita needs reviewed patch")
        found = {}
        for path in vendor.iterdir():
            v.require(path.is_dir() and not path.is_symlink(), "unexpected vendor package entry")
            manifest = tomllib.loads((path / "Cargo.toml").read_text())["package"]
            identity = (manifest["name"], manifest["version"])
            v.require(identity not in found, "duplicate vendored package identity")
            found[identity] = path.name
        v.require(set(found) == {row[:2] for row in artifacts}, "vendored package inventory differs from lock")
        checked = [(row[0], row[1], found[row[:2]], row[2], row[3], row[4], row[5], row[6]) for row in artifacts]
        entries, payload_bytes = v.checked_vendor(vendor, checked)
        source_config = tomllib.loads(generated)
        v.require(set(source_config) == {"source"}, "generated vendor config contains unexpected sections")
        sources = source_config["source"]
        v.require(sources.get("crates-io") == {"replace-with": "vendored-sources"},
                  "generated vendor config has an unexpected registry replacement")
        v.require(sources.get("vendored-sources") == {"directory": str(vendor)}
                  and len(sources) == 2 + len(groups), "generated vendor config source map drift")
        config = stage / "vendor-config.toml"
        config.write_text(generated)
        with tempfile.TemporaryDirectory(prefix="empty-cargo-", dir=stage) as empty_home:
            text = v.run([str(cargo), "metadata", "--locked", "--offline", "--format-version", "1", "--config", str(config), "--manifest-path", str(workspace / "Cargo.toml")], cwd=stage, environment={**environment, "CARGO_HOME": empty_home})
        for package in json.loads(text)["packages"]:
            if package.get("source"):
                entry = lock_by_id.get((package["name"], package["version"]))
                v.require(entry is not None and entry["source"] == package["source"], "offline Cargo source drift")
                v.require(Path(package["manifest_path"]).resolve().is_relative_to(vendor), "offline Cargo escaped vendor")
        v.publish_noreplace(vendor, output)
        print(json.dumps({"artifacts": len(artifacts), "git_repositories": len(groups), "entries": entries,
                          "payload_bytes": payload_bytes, "published": str(output)}, sort_keys=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    one = sub.add_parser("shard")
    one.add_argument("--lock", type=Path, required=True)
    one.add_argument("--table", type=Path, required=True)
    one.add_argument("--start", type=int, required=True)
    one.add_argument("--artifact", nargs=2, action="append", default=[])
    one.add_argument("--output")
    all_shards = sub.add_parser("final")
    all_shards.add_argument("--lock", type=Path, required=True)
    all_shards.add_argument("--table", type=Path, required=True)
    all_shards.add_argument("--bundle", type=Path, required=True)
    all_shards.add_argument("--rust", type=Path, required=True)
    all_shards.add_argument("--git", type=Path)
    all_shards.add_argument("--patch", type=Path)
    all_shards.add_argument("--shard", action="append", default=[])
    all_shards.add_argument("--artifact", nargs=2, action="append", default=[])
    all_shards.add_argument("--output")
    all_shards.add_argument("--registry-cache-name", default="index.crates.io-1949cf8c6b5b557f")
    all_shards.add_argument("--registry-index-name", default="index.crates.io-1949cf8c6b5b557f")
    options = parser.parse_args()
    (shard if options.operation == "shard" else complete)(options)


if __name__ == "__main__":
    main()
