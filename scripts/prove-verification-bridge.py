#!/usr/bin/env python3
"""Exercise real owner-byte handoff through an ordinary Mantle packaging derivation.

Not a proof-tool builder or proof acceptance rail. The pinned-tool producer job
contract required by ADR0034 is deliberately not manufactured here.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import shlex
import subprocess

REVISIONS = {"octet": "4a540dff99aa2040e267e42bbafe09144f276572",
             "trellis": "22d51fb8c1f57155f6c0c4acc4e14902ebb6b4f3"}


def run(argv, log, **kwargs):
    result = subprocess.run([str(arg) for arg in argv], capture_output=True, **kwargs)
    log.with_suffix(".stdout").write_bytes(result.stdout)
    log.with_suffix(".stderr").write_bytes(result.stderr)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}); see {log}.stderr")
    return json.loads(result.stdout)


def recipe(root, b3sum, name, plan):
    files = sorted(path for path in root.rglob("*") if path.is_file())
    bindings, commands, inputs = [], ["set -eu", 'BB=/bin/busybox', '"$BB" mkdir -p "$out"'], []
    for index, path in enumerate(files):
        digest = subprocess.check_output([str(b3sum), str(path)], text=True).split()[0]
        sri = "blake3-" + base64.b64encode(bytes.fromhex(digest)).decode()
        key, source_name = f"s{index}", f"verification-bridge-{name}-{index}"
        bindings.append(f'let {key} = mantle.fetchurl {{ name = {json.dumps(source_name)}, url = {json.dumps(path.as_uri())}, hash = {json.dumps(sri)} }} in')
        inputs.append(key)
        relative = path.relative_to(root).as_posix()
        parent = str(Path(relative).parent)
        commands.extend([f'"$BB" mkdir -p "$out"/{shlex.quote(parent)}',
                         f'for src in "$NIX_STORE"/*-{source_name}; do test -f "$src"; "$BB" cp "$src" "$out"/{shlex.quote(relative)}; done',
                         f'test -f "$out"/{shlex.quote(relative)}'])
    args = json.dumps("\n".join(commands))
    plan.write_text('let mantle = import "lib.ncl" in\n' + "\n".join(bindings) +
                    f'\n{{ name = "verification-bridge-{name}-handoff", builder = "/bin/sh", inputs = [{", ".join(inputs)}], args = ["-c", {args}] }} | mantle.Derivation\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", type=Path, required=True)
    parser.add_argument("--bridge", type=Path, required=True)
    parser.add_argument("--b3sum", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, default=Path("tests/fixtures/verification-bridge"))
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--octet-owner-output", type=Path)
    parser.add_argument("--octet-producer-export", type=Path)
    parser.add_argument("--octet-revision")
    args = parser.parse_args()
    real_octet = (args.octet_owner_output, args.octet_producer_export, args.octet_revision)
    if any(real_octet) and not all(real_octet):
        parser.error("real Octet replay requires --octet-owner-output, --octet-producer-export and --octet-revision")
    args.out = args.out.resolve()
    args.out.mkdir(parents=True, exist_ok=False)
    mantle, bridge, b3sum = (path.resolve(strict=True) for path in (args.mantle, args.bridge, args.b3sum))
    results = {}
    for name, revision in REVISIONS.items():
        source = args.fixtures.resolve() / name
        handoff = args.out / f"{name}-handoff"
        if name == "octet" and all(real_octet):
            source = args.octet_owner_output.resolve(strict=True)
            revision = args.octet_revision
            first = run([bridge, "octet-job", source, args.octet_producer_export.resolve(strict=True), revision, handoff], args.out / f"{name}-consume")
            receipt_path = source / "external-receipt.json"
        else:
            first = run([bridge, name, source, "profile.json", "receipt.json", revision, handoff], args.out / f"{name}-consume")
            receipt_path = source / "receipt.json"
        plan = args.out / f"{name}.ncl"
        recipe(handoff, b3sum, name, plan)
        command = [mantle, "--json", "--store", args.out / "store", "--state-dir", args.out / "state", "build", plan, "--strict-hermetic", "--no-substitute"]
        cold = run(command, args.out / f"{name}-cold")
        warm = run(command, args.out / f"{name}-warm")
        if cold["counts"]["built_total"] < 1 or warm["counts"]["built_total"] != 0 or warm["counts"]["cached_total"] < 1:
            raise RuntimeError("cold construction or warm-cache reuse not observed")
        cold_path = Path(cold["outcomes"][0]["outputs"][0]["path"])
        warm_path = Path(warm["outcomes"][0]["outputs"][0]["path"])
        if cold_path != warm_path:
            raise RuntimeError("warm reuse changed output identity")
        if (warm_path / "external-receipt.json").read_bytes() != receipt_path.read_bytes():
            raise RuntimeError("external receipt bytes changed in Mantle output")
        replay_command = [bridge, "octet-job", warm_path, warm_path, revision, "-"] if name == "octet" and all(real_octet) else [bridge, name, warm_path, "external-profile.json", "external-receipt.json", revision, "-"]
        replay = run(replay_command, args.out / f"{name}-replay", env={"PATH": "/no-ambient-tools", "HOME": str(args.out)})
        if first != replay:
            raise RuntimeError("cache/replay changed evidence association")
        expected_acceptance = "externally-reported-by-octet" if name == "octet" and all(real_octet) else "not-evaluated-by-mantle"
        if replay["proof_acceptance"] != expected_acceptance:
            raise RuntimeError("handoff changed its external proof authority classification")
        results[name] = {"association_identity": replay["association_identity"], "receipt_identity": replay["receipt_identity"],
                         "output": str(warm_path), "cold_counts": cold["counts"], "warm_counts": warm["counts"]}
    scope = "full existing Octet owner report and source-bound packaging/cache/replay; no fresh producer execution or tool construction" if all(real_octet) else "recorded source-bound fixture packaging/cache/replay only; no producer execution, tool construction or external acceptance"
    (args.out / "result.json").write_text(json.dumps({"scope": scope, "results": results}, indent=2) + "\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
