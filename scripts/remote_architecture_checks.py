#!/usr/bin/env python3
"""Check the remote hexagon using resolved Cargo graphs and compiler output.

Run inside the Mantle toolchain after the remote source and compiler lanes are
released. The fixture is a separate workspace with its own checked-in lockfile;
no validation command changes the root manifest or lockfile.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import os
import re
import subprocess
import sys
import tomllib
from collections import deque
from pathlib import Path
from no_std_core_checks import BANNED_PURITY_PATTERNS, production_text

HOST_STD_IMPORT = re.compile(r"\b(?:extern\s+crate|use)\s+(?:::)?std\b")


ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "crates/crunch-remote-core/tests/fixtures/architecture/Cargo.toml"
LOCKS = (ROOT / "Cargo.lock", FIXTURE.with_name("Cargo.lock"))
ADOPTED = {
    "crunch-remote-core": ROOT / "crates/crunch-remote-core/Cargo.toml",
    "crunch-remote-app": ROOT / "crates/crunch-remote-app/Cargo.toml",
}
HOST_TARGET = {
    "x86_64": "x86_64-unknown-linux-gnu",
    "aarch64": "aarch64-unknown-linux-gnu",
}.get(platform.machine())
WASM_TARGET = "wasm32-unknown-unknown"

DIRECT = {
    "crunch-remote-core": frozenset(("blake3", "serde")),
    "crunch-remote-app": frozenset(("crunch-remote-core",)),
    "remote-architecture-fixture": frozenset(("crunch-remote-core", "crunch-remote-app")),
}
NEGATIVE = {
    "snix": ("snix", ("nix-compat",)),
    "store": ("store", ("crunch-store",)),
    "async-runtime": ("async-runtime", ("tokio",)),
    "filesystem": ("filesystem", ("cap-std",)),
    "process": ("process", ("async-process",)),
    "environment": ("environment", ("dotenvy",)),
    "clock": ("clock", ("chrono",)),
    "random": ("random", ("rand",)),
    "network": ("network", ("ureq",)),
    "credential": ("credential", ("secrecy",)),
    "cli": ("cli", ("clap",)),
    "rendering": ("rendering", ("serde_json",)),
    "wrapped-filesystem": ("filesystem", ("host-filesystem", "cap-std")),
    "wrapped-process": ("process", ("host-process", "async-process")),
    "wrapped-network": ("network", ("host-network", "ureq")),
}


class BoundaryFailure(Exception):
    pass


def cargo(*args: str) -> str:
    command = (os.environ.get("CARGO", "cargo"), *args)
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if result.returncode:
        raise BoundaryFailure(f"{' '.join(command)} failed:\n{result.stderr}{result.stdout}")
    return result.stdout


def metadata(manifest: Path, *features: str, target: str | None = None) -> dict:
    args = ["metadata", "--format-version", "1", "--locked", "--offline", "--manifest-path", str(manifest)]
    if target is not None:
        args.extend(("--filter-platform", target))
    if features:
        args.extend(("--no-default-features", "--features", ",".join(features)))
    return json.loads(cargo(*args))


def authority_class(package: str) -> str | None:
    if package.startswith("snix-") or package == "nix-compat":
        return "snix"
    if package == "crunch-store":
        return "store"
    for category, names in (
        ("async-runtime", ("tokio",)),
        ("filesystem", ("tempfile", "cap-std")),
        ("process", ("async-process", "duct", "xshell")),
        ("environment", ("dotenvy", "envy")),
        ("clock", ("chrono", "time")),
        ("random", ("rand", "getrandom")),
        ("network", ("reqwest", "ureq")),
        ("credential", ("secrecy", "crunch-credentials")),
        ("cli", ("clap",)),
        ("rendering", ("serde_json", "tera")),
    ):
        if package in names:
            return category
    return None


def pinned_id(graph: dict, name: str, manifest: Path) -> str:
    matching = [
        package["id"]
        for package in graph["packages"]
        if (
            package["name"] == name
            and package["version"] == "0.1.0"
            and Path(package["manifest_path"]).resolve() == manifest.resolve()
        )
    ]
    if len(matching) != 1:
        raise BoundaryFailure(f"expected exactly one pinned package {name} at {manifest}; found {len(matching)}")
    return matching[0]


def resolved_runtime(graph: dict) -> tuple[dict[str, str], dict[str, list[str]]]:
    names = {package["id"]: package["name"] for package in graph["packages"]}
    edges: dict[str, list[str]] = {}
    for node in graph["resolve"]["nodes"]:
        normal = []
        for dependency in node["deps"]:
            if any(kind["kind"] is None for kind in dependency["dep_kinds"]):
                target_id = dependency["pkg"]
                if target_id not in names:
                    raise BoundaryFailure(f"unresolved runtime package ID: {target_id}")
                normal.append(target_id)
        edges[node["id"]] = sorted(set(normal), key=lambda dep: (names[dep], dep))
    return names, edges


def audit(graph: dict, root_id: str) -> None:
    names, edges = resolved_runtime(graph)
    root_name = names[root_id]
    allowed = DIRECT.get(root_name)
    if allowed is None:
        raise BoundaryFailure(f"unrecognized architecture root: {root_name}")
    pending = deque([(root_id, (root_name,))])
    visited: set[str] = set()
    unknown_direct: list[str] = []
    while pending:
        current_id, path = pending.popleft()
        if current_id in visited:
            continue
        visited.add(current_id)
        if current_id not in edges:
            raise BoundaryFailure(f"missing-runtime-dependency-node:{' -> '.join(path)}")
        for dependent_id in edges[current_id]:
            name = names[dependent_id]
            next_path = (*path, name)
            category = authority_class(name)
            if category is not None:
                raise BoundaryFailure(f"{root_name}-forbidden-{category}:{' -> '.join(next_path)}")
            if current_id == root_id and name not in allowed:
                unknown_direct.append(" -> ".join(next_path))
            pending.append((dependent_id, next_path))
    if unknown_direct:
        raise BoundaryFailure(f"{root_name}-unclassified-runtime-authority:{min(unknown_direct)}")


def check_no_std_source() -> None:
    # The generic adopted-core API rule bans even legitimate borrowed facts
    # and public ports, so reuse only its production-aware purity scanner.
    for name, manifest in ADOPTED.items():
        lib = manifest.parent / "src/lib.rs"
        lib_text = lib.read_text()
        if "#![no_std]" not in lib_text or "extern crate alloc;" not in lib_text:
            raise BoundaryFailure(f"{name}-missing-no-std-alloc:{lib}")
        if "#![forbid(unsafe_code)]" not in lib_text:
            raise BoundaryFailure(f"{name}-unsafe-boundary:{lib}")
        features = tomllib.loads(manifest.read_text()).get("features", {})
        if "std" in features or "std" in features.get("default", []):
            raise BoundaryFailure(f"{name}-forbidden-std-feature:{manifest}")
        for source in sorted((manifest.parent / "src").rglob("*.rs")):
            text = production_text(source)
            if HOST_STD_IMPORT.search(text):
                raise BoundaryFailure(f"{name}-forbidden-source-std-import:{source}")
            for label, pattern in BANNED_PURITY_PATTERNS:
                if pattern.search(text):
                    raise BoundaryFailure(f"{name}-forbidden-source-{label}:{source}")


def blake3_resolution(graph: dict) -> dict:
    packages = [package for package in graph["packages"] if package["name"] == "blake3" and package["version"] == "1.8.2"]
    if len(packages) != 1:
        raise BoundaryFailure(f"expected pinned blake3 1.8.2, found {len(packages)}")
    package = packages[0]
    nodes = [node for node in graph["resolve"]["nodes"] if node["id"] == package["id"]]
    if len(nodes) != 1:
        raise BoundaryFailure(f"missing active pinned blake3 node: {len(nodes)}")
    node = nodes[0]
    return {
        "features": sorted(node["features"]),
        "digest_declared_optional": any(dep["name"] == "digest" and dep["optional"] for dep in package["dependencies"]),
        "digest_edges": [dep["dep_kinds"] for dep in node["deps"] if dep["name"] == "digest"],
    }


def check_dependencies() -> dict:
    check_no_std_source()
    if HOST_TARGET is None:
        raise BoundaryFailure(f"unsupported architecture for remote host dependency rail: {platform.machine()}")
    # Cargo unifies features for every selected workspace member. The root
    # build may activate blake3's OPTIONAL digest trait through an unrelated
    # std adapter, which is not a feature requested by either adopted package.
    # Audit the real pinned package IDs in a separate locked workspace instead.
    root = metadata(ROOT / "Cargo.toml", target=HOST_TARGET)
    fixture = metadata(FIXTURE, target=HOST_TARGET)
    print("remote blake3 feature resolution: " + json.dumps({
        "workspace": blake3_resolution(root),
        "isolated": blake3_resolution(fixture),
        "host_target": HOST_TARGET,
    }, sort_keys=True), flush=True)
    for target, graph in ((HOST_TARGET, fixture), (WASM_TARGET, metadata(FIXTURE, target=WASM_TARGET))):
        for name, manifest in ADOPTED.items():
            audit(graph, pinned_id(graph, name, manifest))
        print(f"remote adopted core/app dependency closure: {target} passed")

    fixture_root = pinned_id(fixture, "remote-architecture-fixture", FIXTURE)
    audit(fixture, fixture_root)
    for feature, (category, suffix) in NEGATIVE.items():
        graph = metadata(FIXTURE, feature, target=HOST_TARGET)
        try:
            audit(graph, pinned_id(graph, "remote-architecture-fixture", FIXTURE))
        except BoundaryFailure as error:
            expected = f"remote-architecture-fixture-forbidden-{category}:remote-architecture-fixture -> "
            if str(error) != expected + " -> ".join(suffix):
                raise BoundaryFailure(f"{feature}: expected {expected + ' -> '.join(suffix)!r}; got {error!s}") from error
        else:
            raise BoundaryFailure(f"{feature}: resolved forbidden dependency was accepted")
    print("remote dependency graph: adopted core/app and 16 real locked fixtures passed")
    return root


def public_items(document: dict) -> list[dict]:
    index = document["index"]
    reachable = set()
    root_id = str(document["root"])
    pending = [(root_id, False)]
    while pending:
        item_id, inherits_visibility = pending.pop()
        if item_id in reachable or item_id not in index:
            continue
        item = index[item_id]
        if item_id != root_id and item.get("visibility") != "public" and not inherits_visibility:
            continue
        reachable.add(item_id)
        inner = item.get("inner", {})
        for kind in ("module", "trait", "enum", "struct", "union", "variant"):
            body = inner.get(kind)
            if not isinstance(body, dict):
                continue
            children = [*body.get("items", ()), *body.get("variants", ()), *body.get("fields", ())]
            shape = body.get("kind", {})
            if kind in ("struct", "variant") and isinstance(shape, dict):
                for style in ("plain", "tuple"):
                    fields = shape.get(style)
                    if isinstance(fields, dict):
                        children.extend(fields.get("fields", ()))
                    elif isinstance(fields, list):
                        children.extend(fields)
            for child_id in children:
                if child_id is not None:
                    pending.append((str(child_id), kind in ("trait", "enum", "variant")))
    return [index[item_id] for item_id in reachable]


def referenced_ids(value: object):
    if isinstance(value, dict):
        for key, child in value.items():
            if key == "id" and isinstance(child, (str, int)):
                yield str(child)
            else:
                yield from referenced_ids(child)
    elif isinstance(value, list):
        for child in value:
            yield from referenced_ids(child)


def public_type_class(path: list[str]) -> str | None:
    if not path:
        return None
    crate = path[0].replace("-", "_")
    category = authority_class(crate) or authority_class(crate.replace("_", "-"))
    if category is not None:
        return category
    if crate == "std":
        if len(path) > 1:
            return {
                "fs": "filesystem", "path": "filesystem", "process": "process",
                "env": "environment", "time": "clock", "net": "network",
                "io": "host-io",
            }.get(path[1], "std")
        return "std"
    return None


def forbidden_public_types(document: dict, name: str) -> set[str]:
    leaked: set[str] = set()
    paths = document["paths"]
    for item in public_items(document):
        for item_id in referenced_ids(item.get("inner", {})):
            entry = paths.get(item_id)
            if entry is None:
                continue
            path = entry["path"]
            category = public_type_class(path)
            if category is not None:
                leaked.add(f"{name}-forbidden-public-{category}:{'::'.join(path)}")
    return leaked


def check_api_shape(root: dict) -> None:
    doc_dir = Path(root["target_directory"]) / "doc"
    for name in ADOPTED:
        cargo("rustdoc", "-p", name, "--lib", "--locked", "--offline", "--", "-Z", "unstable-options", "--output-format", "json")
        document = json.loads((doc_dir / f"{name.replace('-', '_')}.json").read_text())
        leaked = forbidden_public_types(document, name)
        if leaked:
            raise BoundaryFailure(min(leaked))

    cargo("rustdoc", "--manifest-path", str(FIXTURE), "--lib", "--features", "host-api", "--locked", "--offline", "--", "-Z", "unstable-options", "--output-format", "json")
    fixture_dir = Path(metadata(FIXTURE)["target_directory"]) / "doc"
    fixture_document = json.loads((fixture_dir / "remote_architecture_fixture.json").read_text())
    found = {item.split("-forbidden-public-", 1)[1].split(":", 1)[0] for item in forbidden_public_types(fixture_document, "fixture")}
    expected = {"filesystem", "process", "network", "environment", "clock"}
    if not expected.issubset(found):
        raise BoundaryFailure(f"compiler public API negative fixture missed: {', '.join(sorted(expected - found))}")
    print("remote public API: both real crates clean; compiler-resolved host type fixture rejected")


def check_compiler() -> None:
    cargo("test", "--doc", "--manifest-path", str(FIXTURE), "--locked", "--offline")
    for name in ADOPTED:
        cargo("check", "-p", name, "--target", "wasm32-unknown-unknown", "--no-default-features", "--locked", "--offline")
    cargo("check", "-p", "crunch-remote-core", "--features", "serde", "--target", "wasm32-unknown-unknown", "--locked", "--offline")
    print("remote API: positive and compile-fail fixtures, no-default wasm and core serde wasm passed")


def lock_digests() -> dict[Path, bytes]:
    digests = {}
    for path in LOCKS:
        with path.open("rb") as source:
            digests[path] = hashlib.file_digest(source, "sha256").digest()
    return digests


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", nargs="?", choices=("all", "deps", "api", "compiler"), default="all")
    parser.add_argument("--locked", action="store_true", help="assert both root and isolated fixture locks remain byte-for-byte unchanged")
    args = parser.parse_args()
    try:
        before = lock_digests() if args.locked else None
        try:
            root = check_dependencies() if args.phase in ("all", "deps", "api") else None
            if args.phase in ("all", "api"):
                assert root is not None
                check_api_shape(root)
            if args.phase in ("all", "compiler"):
                check_compiler()
        finally:
            if before is not None:
                after = lock_digests()
                changed = [str(path) for path, digest in before.items() if after[path] != digest]
                if changed:
                    raise BoundaryFailure(f"Cargo.lock changed during architecture rail: {', '.join(changed)}")
    except BoundaryFailure as error:
        print(f"remote architecture: {error}", file=sys.stderr)
        raise SystemExit(1) from error


if __name__ == "__main__":
    main()
