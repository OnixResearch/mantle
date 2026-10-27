#!/usr/bin/env python3
"""Exercise pinned Valence consumption of an actual retained gateway publication.
Run only after a real gateway run; never generates a positive fixture bundle.
"""
import argparse
import copy
import json
from pathlib import Path
import subprocess

MAX_EVIDENCE_BYTES = 1_048_576


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checker", type=Path, required=True)
    parser.add_argument("--publication", type=Path, required=True)
    parser.add_argument("--expected-blake3", required=True)
    parser.add_argument("--b3sum", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    commands = []

    def digest(path):
        return subprocess.check_output([str(args.b3sum), "--no-names", str(path)], text=True).strip()

    def save(name, value):
        path = args.output / name
        path.write_text(json.dumps(value, ensure_ascii=False, separators=(",", ":")))
        return path

    def check(label, publication, expected_digest, expected_exit):
        argv = [str(args.checker), str(publication), expected_digest]
        result = subprocess.run(argv, capture_output=True)
        (args.output / (label + ".stdout")).write_bytes(result.stdout)
        (args.output / (label + ".stderr")).write_bytes(result.stderr)
        commands.append({"label": label, "argv": argv, "exit": result.returncode,
                         "expected_exit": expected_exit})
        save("commands.json", commands)
        if result.returncode != expected_exit:
            raise RuntimeError(f"{label}: exit {result.returncode}, expected {expected_exit}")
        return json.loads(result.stdout) if result.stdout else None

    checker_digest = digest(args.checker)
    positive = check("actual-publication", args.publication, args.expected_blake3, 0)
    if not positive["report"]["valid"] or not positive["producer_roles_valid"]:
        raise RuntimeError("positive consumer report did not validate actual publication")
    with args.publication.open("rb") as source:
        original = source.read(MAX_EVIDENCE_BYTES + 1)
    if len(original) > MAX_EVIDENCE_BYTES:
        raise RuntimeError("publication exceeds reviewed bound")
    bundle = json.loads(original)
    if not bundle["resource_observations"]:
        raise RuntimeError("actual resource observation missing; cannot exercise role-promotion negative")
    wrong_digest = ("1" if args.expected_blake3[0] == "0" else "0") + args.expected_blake3[1:]
    check("wrong-independent-digest", args.publication, wrong_digest, 2)
    missing = copy.deepcopy(bundle)
    missing["profile"]["non_claims"] = []
    path = save("missing-boundary.json", missing)
    rejected = check("missing-boundary", path, digest(path), 1)
    if rejected["report"]["valid"]:
        raise RuntimeError("independent Valence validator accepted missing nonclaims")
    promoted = copy.deepcopy(bundle)
    promoted["resource_observations"][0]["role"] = "Verified"
    path = save("promoted-observation.json", promoted)
    rejected = check("promoted-observation", path, digest(path), 1)
    if rejected["producer_roles_valid"]:
        raise RuntimeError("producer profile accepted observation role promotion")
    if digest(args.checker) != checker_digest or digest(args.publication) != args.expected_blake3:
        raise RuntimeError("checker or original publication changed during consumption")
    save("receipt.json", {"schema": "mantle-resource-valence-consumption-v1",
        "publication": str(args.publication), "publication_blake3": args.expected_blake3,
        "checker_blake3": checker_digest, "driver_blake3": digest(Path(__file__).resolve()),
        "commands": commands, "lifecycle_acceptance": False, "worker_authentication": False,
        "scope": "actual retained publication; pinned Valence linkage and producer roles; mutation digests are negative inputs, not provenance"})
    print(args.output / "receipt.json")


if __name__ == "__main__":
    main()
