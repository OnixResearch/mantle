#!/usr/bin/env python3
"""Generate or check the checkout-local Cargo vendor closure without modifying user data.

Run from any directory: python3 scripts/vendor-deps.py generate|check.
The check regenerates the closure with Cargo, applies the reviewed exact-pin
Casita patch, compares every file, then resolves locked offline metadata.
"""

import argparse
import ctypes
import filecmp
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
DEST = ROOT / "vendor-deps"
CONFIG = ROOT / ".cargo/vendor-config.toml"
REV = "90404fcb1cfb3d83f2233715448dfefe913f5fd1"
TURSO_REV = "dca55133caa690f90dcdd58d3c4329fb0703659c"
CASITA_PATCH = ROOT / "patches/casita-blake3-finalize.patch"
CASITA_NAR_SHA256 = "bed3012bed878a81b19348a2b927b95ea7e736b6888229d41a6342918a813e09"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def run(*args):
    environment = os.environ.copy()
    if args[0] == "cargo" and environment.get("MANTLE_CARGO_HOME_BEFORE_DEV_SHELL"):
        # The dev shell's default Cargo home points at Crane's patched source.
        # Vendoring must start from the pinned, unpatched git checkout instead.
        environment["CARGO_HOME"] = environment["MANTLE_CARGO_HOME_BEFORE_DEV_SHELL"]
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=False, env=environment)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, args))}:\n{result.stderr}\n{result.stdout}")
    return result.stdout


def check_pins():
    manifest = tomllib.loads((ROOT / "crates/crunch-store/Cargo.toml").read_text())
    dep = manifest["dependencies"].get("casita")
    require(isinstance(dep, dict), "casita: missing crunch-store dependency")
    require(dep.get("git") == "https://github.com/cachix/casita", "casita: git repository drift")
    require(dep.get("rev") == REV and not {"branch", "tag"}.intersection(dep), "casita: revision drift")
    require(dep.get("default-features") is False, "casita: default features must be disabled")
    require(sorted(dep.get("features", [])) == ["experimental", "native"], "casita: feature drift")
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    names = {p["name"]: p for p in packages if p["name"] in {"casita", "turso", "astral-tokio-tar"}}
    require(names.get("casita", {}).get("source") == f"git+https://github.com/cachix/casita?rev={REV}#{REV}", "casita: Cargo.lock source drift")
    require(names.get("turso", {}).get("source") == f"git+https://github.com/cachix/turso.git?rev={TURSO_REV}#{TURSO_REV}", "turso: Cargo.lock source drift")
    require(names.get("astral-tokio-tar", {}).get("version") == "0.6.4", "astral-tokio-tar: Cargo.lock version drift")
    return {(p["name"], p["version"], p.get("source")) for p in packages}


def compare_config(generated):
    source = tomllib.loads(generated)["source"]
    configured = tomllib.loads(CONFIG.read_text())["source"]
    require(source.keys() == configured.keys(), f"vendor-config source drift: generated {sorted(source)}, configured {sorted(configured)}")
    for name, value in source.items():
        actual = configured[name].copy()
        expected = value.copy()
        if name == "vendored-sources":
            require(actual.pop("directory", None) == "vendor-deps", "vendor-config must select checkout-local vendor-deps")
            expected.pop("directory", None)
        require(actual == expected, f"vendor-config source drift: {name}: generated {expected}, configured {actual}")
    require(set(tomllib.loads(CONFIG.read_text())) == {"source"}, "vendor-config contains unexpected entries")


def apply_casita_patch(vendor):
    crate = vendor / "casita"
    manifest = tomllib.loads((crate / "Cargo.toml").read_text())["package"]
    require((manifest["name"], manifest["version"]) == ("casita", "0.1.0"), "casita: unexpected vendored package")
    source = crate / "src/nar.rs"
    original = hashlib.sha256(source.read_bytes()).hexdigest()
    require(original == CASITA_NAR_SHA256, "casita: pinned upstream nar.rs content drift")
    patch = CASITA_PATCH.read_text()
    lines = patch.splitlines()
    require(lines.count("diff --git a/crates/casita/src/nar.rs b/crates/casita/src/nar.rs") == 1, "casita: patch target drift")
    require(sum(line.startswith("diff --git ") for line in lines) == 1, "casita: patch contains extra files")
    require(sum(line.startswith("@@ ") for line in lines) == 1, "casita: patch contains extra hunks")
    hunk = lines[next(index for index, line in enumerate(lines) if line.startswith("@@ ")) + 1:]
    removed = [line for line in hunk if line.startswith("-")]
    added = [line for line in hunk if line.startswith("+")]
    require(removed == ["-            key.extend_from_slice(hash.finalize().as_bytes());"], "casita: patch original line drift")
    require(added == ["+            key.extend_from_slice(blake3::Hasher::finalize(&hash).as_bytes());"], "casita: patch replacement drift")
    run("patch", "--batch", "--forward", "--fuzz=0", "-p3", "-d", str(crate), "-i", str(CASITA_PATCH))
    checksum_path = crate / ".cargo-checksum.json"
    checksum = json.loads(checksum_path.read_text())
    require(checksum["files"]["src/nar.rs"] == original, "casita: generated checksum drift")
    checksum["files"]["src/nar.rs"] = hashlib.sha256(source.read_bytes()).hexdigest()
    checksum_path.write_text(json.dumps(checksum, sort_keys=True, separators=(",", ":")))


def entries(directory):
    found = {}
    for base, dirs, files in os.walk(directory, followlinks=False):
        for name in dirs + files:
            path = Path(base) / name
            relative = path.relative_to(directory).as_posix()
            require(not path.is_symlink(), f"symlink in vendor closure: {relative}")
            require(path.is_dir() or path.is_file(), f"special file in vendor closure: {relative}")
            found[relative] = path
    return found


def install_without_clobber(source, destination):
    # Linux renameat2(NO_REPLACE) checks the destination atomically; a second
    # process creating vendor-deps between our preflight and install is refused.
    libc = ctypes.CDLL(None, use_errno=True)
    result = libc.renameat2(-100, os.fsencode(source), -100, os.fsencode(destination), 1)
    if result != 0:
        error = ctypes.get_errno()
        raise OSError(error, f"vendor-deps appeared during generation; refusing to clobber: {destination}")


def compare_trees(generated, existing):
    require(existing.is_dir() and not existing.is_symlink(), f"missing or unsafe {existing}")
    lhs, rhs = entries(generated), entries(existing)
    for name in sorted(lhs.keys() | rhs.keys()):
        require(name in lhs, f"extra vendor-deps entry: {name}")
        require(name in rhs, f"missing vendor-deps entry: {name}")
        require(lhs[name].is_file() == rhs[name].is_file(), f"vendor-deps type drift: {name}")
        if lhs[name].is_file():
            require(filecmp.cmp(lhs[name], rhs[name], shallow=False), f"vendor-deps content drift: {name}")
    return len(lhs)


def offline_metadata(config, vendor, lock_packages):
    data = json.loads(run("cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--config", str(config)))
    found = set()
    vendor_resolved = vendor.resolve()
    for package in data["packages"]:
        source = package.get("source")
        if not source:
            continue
        name = package["name"]
        entry = (name, package["version"], source)
        require(entry in lock_packages, f"{name}: offline metadata source absent from Cargo.lock: {source}")
        path = Path(package["manifest_path"]).resolve()
        require(path.is_relative_to(vendor_resolved), f"{name}: offline metadata did not resolve via {vendor}: {path}")
        found.add(name)
    for name in ("casita", "turso", "astral-tokio-tar"):
        require(name in found, f"{name}: absent from offline locked metadata")
    print(f"locked offline metadata: {len(found)} external package names resolved from {vendor}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("generate", "check"))
    args = parser.parse_args()
    lock_packages = check_pins()
    require(not DEST.is_symlink(), "vendor-deps is a symlink; refusing to follow")
    if args.operation == "generate":
        require(not DEST.exists(), "vendor-deps exists; refusing to clobber user data (run check, or move it aside yourself)")
    else:
        require(DEST.is_dir(), "vendor-deps missing; run generate")
    lock_before = (ROOT / "Cargo.lock").read_bytes()
    with tempfile.TemporaryDirectory(prefix="mantle-vendor-", dir=ROOT.parent) as staging:
        vendor = Path(staging) / "vendor-deps"
        generated = run("cargo", "vendor", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), str(vendor))
        require((ROOT / "Cargo.lock").read_bytes() == lock_before, "Cargo.lock changed during generation")
        compare_config(generated)
        apply_casita_patch(vendor)
        # Cargo's generated source map contains the staging path; the committed
        # checkout-local map is used for the final metadata proof.
        config = Path(staging) / "vendor-config.toml"
        config.write_text(generated)
        offline_metadata(config, vendor, lock_packages)
        if args.operation == "check":
            count = compare_trees(vendor, DEST)
            offline_metadata(CONFIG, DEST, lock_packages)
            print(f"vendor-deps matches fresh Cargo generation ({count} entries)")
        else:
            install_without_clobber(vendor, DEST)
            offline_metadata(CONFIG, DEST, lock_packages)
            print("generated vendor-deps without replacing existing data")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print(f"vendor-deps: {error}", file=sys.stderr)
        sys.exit(1)
