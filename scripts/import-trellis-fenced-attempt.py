#!/usr/bin/env python3
"""Import/check exact Trellis oracle source, not proof acceptance or runtime code."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

REVISION = "d91188e495d65bca87551b7a06330efb2c909edf"
REPOSITORY = "https://github.com/OnixResearch/trellis"
VSTD_VERSION = "=0.0.0-2026-01-25-0057"
MODEL_BLAKE3 = "ceb717657f092747a03501e63a0f2a8aeda7af0aba226e21de725334d47c0549"
MODEL = "src/fenced_attempt/verified.rs"
PATHS = (MODEL, "src/fenced_attempt/tests/verified_positive.rs", "src/fenced_attempt/tests/verified_negative.rs", "LICENSE")
ROOT = Path(__file__).resolve().parent.parent / "vendor" / "trellis-fenced-attempt"
BYTES_MAX = 1_048_576


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
    return member.removeprefix("src/fenced_attempt/") if member != "LICENSE" else member


def check(root):
    manifest = json.loads((root / "upstream.json").read_text())
    if manifest["repository"] != REPOSITORY or manifest["revision"] != REVISION or manifest["vstd_version"] != VSTD_VERSION:
        raise ValueError("Trellis oracle source/toolchain cohort differs")
    if set(manifest["files"]) != set(PATHS):
        raise ValueError("Trellis oracle member inventory differs")
    expected = {"upstream.json"}
    for member in PATHS:
        relative = destination(member)
        expected.add(relative)
        path = root / relative
        if not path.is_file() or path.is_symlink() or path.stat().st_size > BYTES_MAX:
            raise ValueError(f"invalid imported oracle member: {relative}")
        data = path.read_bytes()
        if {"path": relative, "bytes": len(data), "blake3": digest(data)} != manifest["files"][member]:
            raise ValueError(f"imported oracle source drift: {relative}")
        if member == MODEL and digest(data) != MODEL_BLAKE3:
            raise ValueError("imported decision kernel differs from immutable producer")
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if not path.is_dir()}
    if actual != expected:
        raise ValueError("unexpected imported oracle member")
    print(json.dumps({"status": "source-identities-match", "revision": REVISION, "model_blake3": MODEL_BLAKE3, "non_claim": "not fresh Verus verification, proof acceptance, runtime authority or release eligibility"}, sort_keys=True))


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
    if cargo["dependencies"]["vstd"] != VSTD_VERSION:
        raise ValueError("upstream Verus standard library cohort differs")
    files = {member: blob(arguments.repository, member) for member in PATHS}
    if digest(files[MODEL]) != MODEL_BLAKE3:
        raise ValueError("upstream source is not the selected immutable decision kernel")
    manifest = {"repository": REPOSITORY, "revision": REVISION, "vstd_version": VSTD_VERSION, "files": {}}
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
