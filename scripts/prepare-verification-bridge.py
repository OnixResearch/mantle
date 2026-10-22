#!/usr/bin/env python3
"""Copy exact immutable upstream handoff inputs; never construct proof receipts.

This prepares identity-link fixtures, NOT complete Octet artifact-verify bundles
or an externally authored execution job. No tools/proofs are run or acquired.
"""
import argparse
import json
from pathlib import Path, PurePosixPath
import subprocess

OCTET_REVISION = "4a540dff99aa2040e267e42bbafe09144f276572"
TRELLIS_REVISION = "22d51fb8c1f57155f6c0c4acc4e14902ebb6b4f3"
OCTET_RAIL = "standards/proof-rails/aeneas-lean"


def blob(repository, revision, path):
    parts = PurePosixPath(path).parts
    if path.startswith("/") or any(part in ("..", ".") for part in parts):
        raise ValueError(f"unsafe producer path: {path}")
    data = subprocess.check_output(["git", "-C", str(repository), "show", f"{revision}:{path}"])
    if len(data) > 8_388_608:
        raise ValueError(f"oversized producer artifact: {path}")
    return data


def save(root, path, data):
    destination = root / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("xb") as stream:
        stream.write(data)


def prepare_octet(repository, root):
    revision = OCTET_REVISION
    receipt = blob(repository, revision, f"{OCTET_RAIL}/fixtures/external-consumer/expected/formal-proof-receipt.json")
    save(root, "receipt.json", receipt)
    save(root, "profile.json", blob(repository, revision, f"{OCTET_RAIL}/generated/consumer-profile.json"))
    files = {
        "src/lib.rs": f"{OCTET_RAIL}/fixtures/external-consumer/src/lib.rs",
        "proofs/aeneas/source/src/lib.rs": f"{OCTET_RAIL}/fixtures/external-consumer/src/lib.rs",
        "proofs/aeneas/theorem.lean": f"{OCTET_RAIL}/fixtures/external-consumer/Theorem.lean",
        "proofs/aeneas/provider-profile.json": f"{OCTET_RAIL}/generated/profile.json",
    }
    for name in ["llbc.llbc", "translation.json", "Llbc.lean"]:
        files[f"proofs/aeneas/{name}"] = f"{OCTET_RAIL}/fixtures/external-consumer/expected/{name}"
    for destination, source in files.items():
        save(root, destination, blob(repository, revision, source))
    save(root, "LICENSE", blob(repository, revision, "LICENSE"))
    return {"repository": "https://github.com/OnixResearch/octet", "revision": revision,
            "scope": "selected source/provider/generated-stage links only; NOT full envelope verification",
            "copied_paths": files}


def prepare_trellis(repository, root):
    revision = TRELLIS_REVISION
    receipt = blob(repository, revision, "aeneas-pilot/generated/aeneas-proof-receipt.json")
    parsed = json.loads(receipt)
    save(root, "receipt.json", receipt)
    save(root, "profile.json", blob(repository, revision, "aeneas-pilot/generated/profile.json"))
    paths = {parsed["source_path"], parsed["verus_receipt_path"]}
    paths.update(row["path"] for row in parsed["artifacts"])
    for path in sorted(paths):
        save(root, path, blob(repository, revision, path))
    save(root, "LICENSE", blob(repository, revision, "LICENSE"))
    return {"repository": "https://github.com/OnixResearch/trellis", "revision": revision,
            "scope": "source/profile/artifact and opaque Verus receipt byte links only",
            "copied_paths": sorted(paths)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--octet", type=Path, required=True)
    parser.add_argument("--trellis", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("--out must not exist; fixture preparation never overwrites data")
    metadata = {"octet": prepare_octet(args.octet, args.out / "octet"),
                "trellis": prepare_trellis(args.trellis, args.out / "trellis")}
    save(args.out, "upstream.json", (json.dumps(metadata, indent=2, sort_keys=True) + "\n").encode())
    print(json.dumps(metadata, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
