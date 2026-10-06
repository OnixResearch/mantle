#!/usr/bin/env python3
"""Generate, check, or preserve-and-refresh the checkout-local Cargo vendor closure.

Run from any directory: python3 scripts/vendor-deps.py generate|check|refresh|dev-shell-check.
The check regenerates the closure with Cargo, applies the reviewed exact-pin
Casita patch, compares every file, then resolves locked offline metadata.
refresh preserves the existing directory in a separate backup before it
installs a fresh generated closure without clobbering a concurrent creator.
dev-shell-check resolves Cargo metadata without --config and rejects an
unpatched git checkout or a non-Crane source in the default Nix shell.
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
TURSO_PACKAGES = {
    "turso", "turso_core", "turso_ext", "turso_macros", "turso_parser",
    "turso_sdk_kit", "turso_sdk_kit_macros", "turso_sync_engine",
    "turso_sync_sdk_kit",
}
CASITA_PATCH = ROOT / "patches/casita-blake3-finalize.patch"
CASITA_PATCH_SHA256 = "c0def0527dcc56beafa8d3f89c418a071b1d93df418245ebcd6a69aeb580840a"
CASITA_NAR_SHA256 = "bed3012bed878a81b19348a2b927b95ea7e736b6888229d41a6342918a813e09"
CASITA_PATCHED_NAR_SHA256 = "e88c332c3bb0605e5684e303c17e755adc66d2d525b80672dd7e16015f1be0b1"


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
    run(sys.executable, str(ROOT / "scripts/import-bao-tree.py"), "check")
    root_manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    require(root_manifest["patch"]["crates-io"]["bao-tree"] == {"path": "third_party/bao-tree"}, "bao-tree: crates.io path patch drift")
    bao_manifest = tomllib.loads((ROOT / "third_party/bao-tree/Cargo.toml").read_text())
    require(bao_manifest["package"]["version"] == "0.16.1", "bao-tree: imported package version drift")
    require(bao_manifest["features"]["validate"] == ["dep:futures-lite"], "bao-tree: validation feature drift")
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    casita = [p for p in packages if p["name"] == "casita"]
    require(len(casita) == 1 and casita[0].get("version") == "0.1.0", "casita: Cargo.lock package drift")
    casita_source = f"git+https://github.com/cachix/casita?rev={REV}#{REV}"
    require(casita[0].get("source") == casita_source, "casita: Cargo.lock source drift")
    require(
        all(p["name"] == "casita" for p in packages if p.get("source", "").startswith("git+https://github.com/cachix/casita?")),
        "casita: unexpected package from pinned repository",
    )
    bao = [p for p in packages if p["name"] == "bao-tree"]
    require(len(bao) == 1 and bao[0]["version"] == "0.16.1" and "source" not in bao[0], "bao-tree: Cargo.lock did not select local candidate")
    require(not any(p["name"] in {"genawaiter-proc-macro", "proc-macro-error", "proc-macro-error-attr", "proc-macro-hack", "syn-mid"} for p in packages), "bao-tree: obsolete generator proc macro remains locked")
    turso = [p for p in packages if p["name"] in TURSO_PACKAGES]
    require(
        len(turso) == len(TURSO_PACKAGES) and {p["name"] for p in turso} == TURSO_PACKAGES,
        "turso: Cargo.lock package set drift",
    )
    turso_source = f"git+https://github.com/cachix/turso.git?rev={TURSO_REV}#{TURSO_REV}"
    require(
        all(p.get("version") == "0.8.0-pre.7" and p.get("source") == turso_source for p in turso),
        "turso: Cargo.lock version or source drift",
    )
    require(
        {p["name"] for p in packages if p.get("source", "").startswith("git+https://github.com/cachix/turso.git?")}
        == TURSO_PACKAGES,
        "turso: unexpected package from pinned repository",
    )
    tar = [p for p in packages if p["name"] == "astral-tokio-tar"]
    require(len(tar) == 1 and tar[0]["version"] == "0.6.4", "astral-tokio-tar: Cargo.lock version drift")
    blake3 = [p for p in packages if p["name"] == "blake3"]
    require(len(blake3) == 1 and blake3[0]["version"] == "1.8.2", "blake3: Cargo.lock version drift")
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
    require((manifest["name"], manifest["version"], manifest.get("rust-version")) == ("casita", "0.1.0", "1.94.1"), "casita: unexpected vendored package or rust-version")
    source = crate / "src/nar.rs"
    original = hashlib.sha256(source.read_bytes()).hexdigest()
    require(original == CASITA_NAR_SHA256, "casita: pinned upstream nar.rs content drift")
    patch_bytes = CASITA_PATCH.read_bytes()
    require(hashlib.sha256(patch_bytes).hexdigest() == CASITA_PATCH_SHA256, "casita: tracked patch content drift")
    lines = patch_bytes.decode("utf-8").splitlines()
    require(lines.count("diff --git a/crates/casita/src/nar.rs b/crates/casita/src/nar.rs") == 1, "casita: patch target drift")
    require(sum(line.startswith("diff --git ") for line in lines) == 1, "casita: patch contains extra files")
    require(sum(line.startswith("@@ ") for line in lines) == 1, "casita: patch contains extra hunks")
    hunk = lines[next(index for index, line in enumerate(lines) if line.startswith("@@ ")) + 1:]
    removed = [line for line in hunk if line.startswith("-")]
    added = [line for line in hunk if line.startswith("+")]
    require(removed == ["-            key.extend_from_slice(hash.finalize().as_bytes());"], "casita: patch original line drift")
    require(added == ["+            key.extend_from_slice(blake3::Hasher::finalize(&hash).as_bytes());"], "casita: patch replacement drift")
    run("patch", "--batch", "--forward", "--fuzz=0", "-p3", "-d", str(crate), "-i", str(CASITA_PATCH))
    patched = hashlib.sha256(source.read_bytes()).hexdigest()
    require(patched == CASITA_PATCHED_NAR_SHA256, "casita: patched nar.rs content drift")
    checksum_path = crate / ".cargo-checksum.json"
    checksum = json.loads(checksum_path.read_text())
    require(checksum["files"]["src/nar.rs"] == original, "casita: generated checksum drift")
    checksum["files"]["src/nar.rs"] = patched
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

def verify_dev_shell_casita(packages, lock_packages):
    casita = [package for package in packages if package["name"] == "casita"]
    require(len(casita) == 1, "default dev-shell source map did not resolve exactly one casita")
    package = casita[0]
    require((package["name"], package["version"], package["source"]) in lock_packages, "default dev-shell Casita pin drift")
    manifest = Path(package["manifest_path"]).resolve(strict=True)
    require(manifest.is_relative_to("/nix/store"), "default dev shell resolved a mutable git checkout")
    source_manifest = tomllib.loads(manifest.read_text())["package"]
    require(source_manifest.get("rust-version") == "1.94.1", "default dev-shell Casita rust-version drift")
    source = manifest.parent / "src/nar.rs"
    require(hashlib.sha256(source.read_bytes()).hexdigest() == CASITA_PATCHED_NAR_SHA256, "default dev shell resolved unpatched Casita")
    print(f"default dev-shell Cargo resolved patched pinned Casita from {manifest.parent}")


def check_dev_shell(lock_packages):
    home = os.environ.get("CARGO_HOME")
    require(home and os.environ.get("MANTLE_CARGO_HOME_BEFORE_DEV_SHELL"), "enter the default nix develop shell first")
    config = Path(home) / "config.toml"
    require(config.is_symlink(), "default dev-shell Cargo source replacement missing")
    replacement = config.resolve(strict=True)
    require(replacement.name == "config.toml" and replacement.is_relative_to("/nix/store"), "dev-shell Cargo config is not Crane's immutable source map")
    result = subprocess.run(
        ("cargo", "metadata", "--locked", "--offline", "--format-version", "1"),
        cwd=ROOT, text=True, capture_output=True, check=False,
    )
    require(result.returncode == 0, f"default dev-shell Cargo metadata failed:\n{result.stderr}")
    packages = json.loads(result.stdout)["packages"]
    verify_dev_shell_casita(packages, lock_packages)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("generate", "check", "refresh", "dev-shell-check"))
    args = parser.parse_args()
    lock_packages = check_pins()
    if args.operation == "dev-shell-check":
        check_dev_shell(lock_packages)
        return
    require(not DEST.is_symlink(), "vendor-deps is a symlink; refusing to follow")
    if args.operation == "generate":
        require(not DEST.exists(), "vendor-deps exists; refusing to clobber user data (run check, or move it aside yourself)")
    else:
        require(DEST.is_dir(), "vendor-deps missing; run generate")
    if args.operation == "refresh":
        entries(DEST)
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
        elif args.operation == "refresh":
            backup_root = Path(tempfile.mkdtemp(prefix="mantle-vendor-backup-", dir=ROOT.parent))
            backup = backup_root / "vendor-deps"
            install_without_clobber(DEST, backup)
            try:
                install_without_clobber(vendor, DEST)
            except OSError:
                install_without_clobber(backup, DEST)
                raise
            offline_metadata(CONFIG, DEST, lock_packages)
            print(f"refreshed vendor-deps; prior checkout-local closure retained at {backup}")
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
