#!/usr/bin/env python3
"""Compare the real original PerlAsm pipe with its source-selected split."""
import argparse
import collections
import json
import os
import pathlib
import subprocess

parser = argparse.ArgumentParser()
for name in ("bindings", "wrapper", "helper", "aws", "ring", "perl", "cc", "b3sum", "out"):
    parser.add_argument("--" + name, required=True)
args = parser.parse_args()
root = pathlib.Path(args.out)
root.mkdir(parents=True, exist_ok=False)
environment = dict(os.environ)
environment.update(HOME=str(root / "home"), LANG="C", LC_ALL="C", TZ="UTC", CC=args.cc)
(root / "home").mkdir()
results = []

def run(name, command, cwd, *, stdin=None, env=None, allow_failure=False, merged=False):
    completed = subprocess.run(command, cwd=cwd, env=env or environment, stdin=stdin,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT if merged else subprocess.PIPE, timeout=120)
    (cwd / (name + ".stdout")).write_bytes(completed.stdout)
    if not merged:
        (cwd / (name + ".stderr")).write_bytes(completed.stderr)
    if completed.returncode != 0 and not allow_failure:
        raise RuntimeError(f"{name} failed with {completed.returncode}: {command}; {completed.stderr!r}")
    return completed

def digest(path):
    return subprocess.run([args.b3sum, "--no-names", str(path)], check=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=30).stdout.strip()

try:
    bindings = json.loads(pathlib.Path(args.bindings).read_text())
    families = collections.Counter(item["family"] for item in bindings)
    if families != {"ring": 12, "aws": 26}:
        raise RuntimeError(f"incomplete selected source coverage: {families}")
    probe = run("assembler-probe", [args.cc, "-Wa,-v", "-c", "-o", "/dev/null", "-x", "assembler", "/dev/null"],
        root, allow_failure=True, merged=True)
    probe_path = root / "assembler-probe.stdout"
    environment["MANTLE_AWS_AS_PROBE"] = str(probe_path)
    transformed = root / "aws-transformed"
    run("source-helper", [args.helper, "perlasm-tree", args.aws, str(transformed)], root)
    for ordinal, binding in enumerate(bindings):
        relative = pathlib.PurePosixPath(binding["relative"])
        filename = binding["filename"]
        if relative.is_absolute() or ".." in relative.parts or not str(relative).endswith(".pl"):
            raise RuntimeError(f"invalid source member: {binding}")
        if pathlib.PurePosixPath(filename).name != filename or not filename.endswith(".S"):
            raise RuntimeError(f"invalid original output basename: {binding}")
        original_root = pathlib.Path(args.ring if binding["family"] == "ring" else args.aws)
        candidate_root = pathlib.Path(args.ring) if binding["family"] == "ring" else transformed
        original = original_root / relative
        authenticated = candidate_root / relative
        translator = candidate_root / "crypto/perlasm/x86_64-xlate.pl"
        case = root / f"case-{ordinal:02d}"
        reference = case / "reference"
        split = case / "split"
        reference.mkdir(parents=True)
        split.mkdir()
        run("original-pipe", [args.perl, str(original), "elf", filename], reference)
        raw = split / "raw"
        # The exact original logical pathname is separate from the authenticated
        # transformed bytes; this preserves source-path-dependent computation.
        run("split-generator", [args.perl, args.wrapper, str(authenticated), str(original), str(raw), filename], split)
        assembly = split / "assembly"
        with raw.open("rb") as stream:
            run("split-translator", [args.perl, str(translator), "elf", str(assembly)], split, stdin=stream)
        reference_file = reference / filename
        expected = reference_file.read_bytes()
        observed = assembly.read_bytes()
        if expected != observed:
            common = min(len(expected), len(observed))
            difference = next((i for i in range(common) if expected[i] != observed[i]), common)
            raise RuntimeError(f"split changes original assembly for {binding}: byte {difference}, sizes {len(expected)}/{len(observed)}")
        results.append({"family": binding["family"], "id": binding["id"], "source": str(relative),
            "filename": filename, "assembly_bytes": len(observed),
            "original_blake3": digest(reference_file), "split_blake3": digest(assembly), "equal": True})
    report = {"schema": "mantle-original-perlasm-split-equivalence-v1", "status": "passed",
        "counts": dict(families), "assembler_probe_exit": probe.returncode, "cases": results,
        "non_claim": "Original/split source-helper equivalence for the named inputs and observed host-tool probe inside a fixture bwrap. Host Perl/compiler tooling is not a source-built provider; no root action reconciliation, kernel isolation, bootstrap fixed point or release acceptance."}
    (root / "equivalence.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": "passed", "counts": dict(families), "report": str(root / "equivalence.json")}))
except Exception as error:
    report = {"schema": "mantle-original-perlasm-split-equivalence-v1", "status": "failed",
        "error": str(error), "completed_cases": results,
        "non_claim": "Failed helper fixture; no provider or cold fixed-point proof."}
    (root / "equivalence.json").write_text(json.dumps(report, indent=2) + "\n")
    raise
