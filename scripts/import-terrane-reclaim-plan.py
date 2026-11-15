#!/usr/bin/env python3
"""Import/check exact Terrane reclaim-plan kernel source, not proof acceptance or runtime code."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

REVISION = "bbf83d95402370b6ba29e82e01f16b1059c55a83"
REPOSITORY = "https://github.com/OnixResearch/terrane"
# The producer verifies against its own Verus standard library; Mantle only
# compiles the executable kernel for test-only parity under its own pin.
PRODUCER_VSTD_VERSION = "=0.0.0-2026-10-18-0147"
CONSUMER_VSTD_VERSION = "=0.0.0-2026-01-25-0057"
MODEL_BLAKE3 = "ee279c734a91e86d6f154835a57e8e9eb5fc5bbadf349657114acabee8cd5aee"
MODEL = "src/reclaim_plan/verified.rs"
PATHS = (
    MODEL,
    "src/reclaim_plan/tests/verified_positive.rs",
    "src/reclaim_plan/tests/verified_negative.rs",
    "src/reclaim_plan/tests/verified_exhaustive.rs",
    "LICENSE",
)
ROOT = Path(__file__).resolve().parent.parent / "vendor" / "terrane-reclaim-plan"
BYTES_MAX = 1_048_576
NON_CLAIM = (
    "source identity only: not fresh Verus verification, not proof acceptance, "
    "not runtime authority, and not equivalence of Mantle source with the kernel"
)


def digest(data):
    result = subprocess.run(["b3sum", "--no-names"], input=data, stdout=subprocess.PIPE, check=True)
    value = result.stdout.decode("ascii").strip()
    if len(value) != 64 or any(character not in "0123456789abcdef" for character in value):
        raise ValueError("b3sum did not return canonical BLAKE3")
    return value


def blob(repository, member):
    data = subprocess.check_output(["git", "-C", str(repository), "show", f"{REVISION}:{member}"])
    if not data or len(data) > BYTES_MAX:
        raise ValueError(f"upstream member outside import bound: {member}")
    return data


def destination(member):
    return member.removeprefix("src/reclaim_plan/") if member != "LICENSE" else member


def consumer_vstd_version():
    manifest = Path(__file__).resolve().parent.parent / "crates" / "crunch-gc-check" / "Cargo.toml"
    return tomllib.loads(manifest.read_text())["dependencies"]["vstd"]


def check(root):
    manifest = json.loads((root / "upstream.json").read_text())
    if manifest["repository"] != REPOSITORY or manifest["revision"] != REVISION:
        raise ValueError("Terrane reclaim kernel source identity differs")
    if manifest["producer_vstd_version"] != PRODUCER_VSTD_VERSION:
        raise ValueError("Terrane producer Verus standard library cohort differs")
    if manifest["consumer_vstd_version"] != CONSUMER_VSTD_VERSION or consumer_vstd_version() != CONSUMER_VSTD_VERSION:
        raise ValueError("Mantle consumer Verus standard library cohort differs")
    if set(manifest["files"]) != set(PATHS):
        raise ValueError("Terrane kernel member inventory differs")
    expected = {"upstream.json"}
    for member in PATHS:
        relative = destination(member)
        expected.add(relative)
        path = root / relative
        if not path.is_file() or path.is_symlink() or path.stat().st_size > BYTES_MAX:
            raise ValueError(f"invalid imported kernel member: {relative}")
        data = path.read_bytes()
        if {"path": relative, "bytes": len(data), "blake3": digest(data)} != manifest["files"][member]:
            raise ValueError(f"imported kernel source drift: {relative}")
        if member == MODEL and digest(data) != MODEL_BLAKE3:
            raise ValueError("imported decision kernel differs from immutable producer")
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if not path.is_dir()}
    if actual != expected:
        raise ValueError("unexpected imported kernel member")
    print(json.dumps({"status": "source-identities-match", "revision": REVISION, "model_blake3": MODEL_BLAKE3, "non_claim": NON_CLAIM}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path)
    parser.add_argument("--out", type=Path, default=ROOT)
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    if arguments.check:
        if arguments.repository is not None:
            parser.error("--check is an offline identity check; do not supply --repository")
        check(arguments.out)
        return
    if arguments.repository is None:
        parser.error("--repository must explicitly name the upstream Git object store")
    if arguments.out.exists():
        parser.error("--out must not exist; imports never overwrite a vendored snapshot")
    cargo = tomllib.loads(blob(arguments.repository, "Cargo.toml").decode("utf-8"))
    if cargo["dependencies"]["vstd"] != PRODUCER_VSTD_VERSION:
        raise ValueError("upstream Verus standard library cohort differs")
    files = {member: blob(arguments.repository, member) for member in PATHS}
    if digest(files[MODEL]) != MODEL_BLAKE3:
        raise ValueError("upstream source is not the selected immutable decision kernel")
    manifest = {
        "repository": REPOSITORY,
        "revision": REVISION,
        "producer_vstd_version": PRODUCER_VSTD_VERSION,
        "consumer_vstd_version": CONSUMER_VSTD_VERSION,
        "files": {},
    }
    for member, data in files.items():
        relative = destination(member)
        path = arguments.out / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as stream:
            stream.write(data)
        manifest["files"][member] = {"path": relative, "bytes": len(data), "blake3": digest(data)}
    with (arguments.out / "upstream.json").open("x") as stream:
        json.dump(manifest, stream, indent=2, sort_keys=True)
        stream.write("\n")
    check(arguments.out)


if __name__ == "__main__":
    main()
