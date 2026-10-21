#!/usr/bin/env python3
"""Real offline Cargo-bundle campaign. No planner/build output is synthesized.

Inputs are an explicit campaign JSON described in docs/cargo-build-bundles.md.
Run under the admitted Linux proof host; all bundles/transports are local.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import mmap
import subprocess


def read(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    Path(path).write_text(json.dumps(value, separators=(",", ":")) + "\n")


def run(argv, output, ok=True):
    result = subprocess.run([str(x) for x in argv], stdin=subprocess.DEVNULL,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    Path(str(output) + ".stdout").write_bytes(result.stdout)
    Path(str(output) + ".stderr").write_bytes(result.stderr)
    Path(str(output) + ".exit").write_text(str(result.returncode) + "\n")
    if (result.returncode == 0) != ok:
        raise RuntimeError(f"unexpected command result {result.returncode}: {argv}; evidence {output}")
    return json.loads(result.stdout) if ok else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--campaign", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    campaign = read(args.campaign)
    if campaign["schema"] != "mantle-cargo-build-bundle-campaign-v1":
        raise RuntimeError("unsupported campaign schema")
    out = args.output.resolve()
    out.mkdir()  # Never merge evidence with an earlier run.
    mantle = Path(campaign["mantle"]).resolve(strict=True)
    if not os.access(mantle, os.X_OK):
        raise RuntimeError("declared immutable Mantle is not executable")
    shutil.copyfile(Path(campaign["compatibility"]).resolve(strict=True), out / "compatibility.json")
    common = ["--compatibility", str(out / "compatibility.json"),
              "--policy-hash", campaign["policy_hash"], "--valid-at-unix-s", str(campaign["valid_at_unix_s"])]
    for key in campaign["trusted_public_keys"]:
        common.extend(["--trusted-public-key", key])
    if campaign.get("revocation_ref"):
        common.extend(["--revocation-ref", campaign["revocation_ref"]])
    for key in campaign.get("revoked_public_key_digests", []):
        common.extend(["--revoked-public-key-digest", key])
    command = [mantle, "--json", "cargo-build-bundle"]
    variants = campaign["variants"]
    for name in ["baseline", "app-edit", "outside-edit", "tool-edit", "native-edit"]:
        if name not in variants:
            raise RuntimeError(f"mandatory real-input variant missing: {name}")
    exported = {}
    for name, variant in variants.items():
        manifest = Path(variant["manifest"]).resolve(strict=True)
        inputs = Path(variant["inputs"]).resolve(strict=True)
        report = run(command + ["export", "--manifest", manifest, "--inputs", inputs,
                               "--to", out / name] + common, out / f"{name}-export")
        exported[name] = report
        publisher = read(variant["producer_build_report"])
        shutil.copyfile(variant["producer_build_report"], out / f"{name}-publisher-build.json")
        publisher_units = [row for row in publisher["outcomes"] if row["label"].startswith(("rust-unit-", "rust-target-facts-"))]
        executed = sum(not row["cached"] for row in publisher_units)
        reused = sum(row["cached"] for row in publisher_units)
        if (executed, reused) != (variant["expected_units_executed"], variant["expected_units_reused"]):
            raise RuntimeError(f"{name} observed unit work differs from declared expectation")
        if publisher["counts"]["failed_total"] != 0:
            raise RuntimeError(f"{name} publisher build failed")
        if name == "app-edit" and (executed != 1 or reused == 0):
            raise RuntimeError("app-only edit did not rebuild exactly its unit and retain dependencies")
        if name == "outside-edit" and executed != 0:
            raise RuntimeError("outside-package edit executed Cargo units")
        if name in ["tool-edit", "native-edit"] and executed == 0:
            raise RuntimeError(f"{name} did not invalidate actual unit execution")
    baseline = exported["baseline"]
    for same in ["outside-edit"]:
        if exported[same]["canonical_plan_blake3"] != baseline["canonical_plan_blake3"]:
            raise RuntimeError(f"outside-package edit changed native plan: {same}")
    for changed in ["app-edit", "tool-edit", "native-edit"]:
        if exported[changed]["bundle_blake3"] == baseline["bundle_blake3"]:
            raise RuntimeError(f"changed declared input retained bundle identity: {changed}")
    # Export exactly the same transports from a second unrelated filesystem location.
    relocation = out / "relocated-producer"
    relocation.mkdir()
    input_map = read(variants["baseline"]["inputs"])
    relocated = {}
    for index, (name, path) in enumerate(input_map.items()):
        target = relocation / str(index)
        shutil.copyfile(path, target)
        relocated[name] = str(target)
    write(out / "relocated-inputs.json", relocated)
    repeated = run(command + ["export", "--manifest", variants["baseline"]["manifest"],
                   "--inputs", out / "relocated-inputs.json", "--to", out / "relocated-bundle"] + common,
                   out / "relocated-export")
    if repeated["bundle_blake3"] != baseline["bundle_blake3"]:
        raise RuntimeError("producer path changed canonical identity")
    if (out / "baseline/cargo-build-bundle.json").read_bytes() != (out / "relocated-bundle/cargo-build-bundle.json").read_bytes():
        raise RuntimeError("producer path changed canonical manifest bytes")
    identity = ["--expected-bundle-blake3", baseline["bundle_blake3"]]
    run(command + ["verify", "--from", out / "baseline"] + common + identity, out / "baseline-verify")
    manifest = read(out / "baseline/cargo-build-bundle.json")
    negatives = {
        "unknown-schema": lambda m: m.update(schema="unknown-mandatory-v2"),
        "unknown-version": lambda m: m.update(version=2),
        "unknown-plan-schema": lambda m: m["compatibility"].update(plan_schema="mantle-plan-v3"),
        "unknown-source-schema": lambda m: m["compatibility"].update(source_schema="unknown"),
        "unknown-native-schema": lambda m: next(iter(m["native_inputs"].values())).update(schema_version=2),
        "unknown-planning-version": lambda m: m["provenance"].update(cargo_unit_graph_version=2),
        "target-mismatch": lambda m: m["compatibility"].update(target_triple="aarch64-unknown-linux-gnu"),
        "prefix-mismatch": lambda m: m["compatibility"].update(store_prefix="/wrong/store"),
        "policy-mismatch": lambda m: m["compatibility"].update(network="allowed"),
        "admission-limit-mismatch": lambda m: m["compatibility"].update(max_direct_inputs=257),
    }
    # Native declarations are mandatory in this campaign, even for the baseline.
    if not manifest["native_inputs"]:
        raise RuntimeError("campaign must include a real declared native-input role")
    negative_dir = out / "negative"
    negative_dir.mkdir()
    for name, mutate in negatives.items():
        candidate = negative_dir / name
        shutil.copytree(out / "baseline", candidate)
        changed = copy.deepcopy(manifest)
        mutate(changed)
        write(candidate / "cargo-build-bundle.json", changed)
        client = negative_dir / f"{name}-client"
        run(command + ["replay", "--from", candidate, "--client-dir", client] + common + identity,
            negative_dir / f"{name}-result", ok=False)
        if client.exists():
            raise RuntimeError(f"{name} mutated client before rejection")
    for kind in ["source", "store", "plan", "config", "lock"]:
        for operation in ["missing", "tampered"]:
            name = f"{operation}-{kind}-payload"
            candidate = negative_dir / name
            shutil.copytree(out / "baseline", candidate)
            payload = candidate / manifest["artifacts"][kind]["file"]
            if operation == "missing":
                payload.unlink()
            else:
                with payload.open("r+b") as stream:
                    first = stream.read(1)
                    stream.seek(0)
                    stream.write(bytes([first[0] ^ 1]))
            client = negative_dir / f"{name}-client"
            run(command + ["replay", "--from", candidate, "--client-dir", client] + common + identity,
                negative_dir / f"{name}-result", ok=False)
            if client.exists():
                raise RuntimeError(f"{name} admitted state")
    # A real incompatible valid public key tests signature authority, not malformed JSON.
    unrelated = campaign["unrelated_public_key"]
    untrusted_common = [value for value in common]
    for index, value in enumerate(untrusted_common):
        if value == "--trusted-public-key":
            untrusted_common[index + 1] = unrelated
    rejected_client = negative_dir / "untrusted-signature-client"
    run(command + ["replay", "--from", out / "baseline", "--client-dir", rejected_client]
        + untrusted_common + identity, negative_dir / "untrusted-signature-result", ok=False)
    if rejected_client.exists():
        raise RuntimeError("untrusted signature admitted state")
    # Reseal only the outer byte digest; existing inner checks must still reject.
    b3sum = Path(campaign["blake3_program"]).resolve(strict=True)
    archive_listing = run([mantle, "--json", "store", "archive", "list", "--from",
                           out / "baseline" / manifest["artifacts"]["store"]["file"]], negative_dir / "archive-list")
    signature = next(sig for path in archive_listing["paths"] for sig in path["signatures"])
    for name, kind in [("inner-source-version", "source"), ("inner-receipt-version", "receipts"),
                       ("invalid-signature", "store"), ("native-nar-tamper", "store")]:
        case = negative_dir / name
        case.mkdir()
        changed = copy.deepcopy(manifest)
        inputs = {key: str(out / "baseline" / item["file"]) for key, item in manifest["artifacts"].items()}
        inputs.update({key: str(out / "baseline" / item["file"]) for key, item in manifest["provenance"]["evidence"].items()})
        payload = case / "changed-payload"
        shutil.copyfile(inputs[kind], payload)
        if kind in ["source", "receipts"]:
            inner = read(payload)
            inner["version"] = 2
            write(payload, inner)
        else:
            with payload.open("r+b") as stream, mmap.mmap(stream.fileno(), 0) as mapped:
                if name == "invalid-signature":
                    token = signature.encode()
                    offset = mapped.find(token)
                    if offset < 0:
                        raise RuntimeError("existing archive signature token not found")
                    offset += token.index(b":") + 1
                    mapped[offset] = ord("B") if mapped[offset] == ord("A") else ord("A")
                else:
                    marker = campaign["native_payload_marker"].encode()
                    if not marker:
                        raise RuntimeError("native payload marker must be explicitly declared")
                    offset = 0
                    changes = 0
                    while True:
                        offset = mapped.find(marker, offset)
                        if offset < 0:
                            break
                        mapped[offset] ^= 1
                        changes += 1
                        offset += len(marker)
                    if changes == 0:
                        raise RuntimeError("declared native payload marker not present in archive")
                mapped.flush()
        hashed = subprocess.run([str(b3sum), str(payload)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        (case / "hash.stdout").write_bytes(hashed.stdout)
        (case / "hash.stderr").write_bytes(hashed.stderr)
        digest = hashed.stdout.decode().split()[0]
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise RuntimeError("declared BLAKE3 program returned an unsupported digest")
        changed["artifacts"][kind].update(blake3=digest, bytes=payload.stat().st_size)
        inputs[kind] = str(payload)
        write(case / "manifest.json", changed)
        write(case / "inputs.json", inputs)
        destination = case / "rejected-handoff"
        run(command + ["export", "--manifest", case / "manifest.json", "--inputs", case / "inputs.json",
                       "--to", destination] + common, case / "export-result", ok=False)
        if destination.exists():
            raise RuntimeError(f"{name} published an invalid handoff")
    # Hide the actual producer paths, not just environment variable aliases.
    # All required executable/library inputs remain independently declared transports.
    bwrap = Path(campaign["bwrap"]).resolve(strict=True)
    network = [str(bwrap), "--die-with-parent", "--unshare-user", "--unshare-net", "--ro-bind", "/", "/", "--tmpfs", "/tmp",
               "--bind", str(out), str(out)]
    forbidden = campaign["forbidden_client_paths"]
    hidden_paths = set()
    for role in ["checkout", "cargo-home", "target", "resultdb"]:
        hidden = Path(forbidden[role]).resolve(strict=True)
        if hidden in out.parents or hidden == out or hidden in mantle.parents or hidden == mantle:
            raise RuntimeError(f"proof inputs overlap hidden producer state: {role}")
        hidden_paths.add(hidden)
    for hidden in sorted(hidden_paths, key=lambda path: len(path.parts)):
        if any(parent in hidden_paths for parent in hidden.parents):
            continue  # An empty parent mount already hides this declared path.
        if hidden.is_dir():
            network.extend(["--tmpfs", str(hidden)])
        else:
            network.extend(["--ro-bind", "/dev/null", str(hidden)])
    network.extend(["--chdir", str(out), "--"])
    run(network + command + ["replay", "--from", out / "baseline", "--client-dir", out / "clean-client"] + common + identity,
        out / "clean-replay")
    build = read(out / "clean-client/build-report.json")
    units = [row for row in build["outcomes"] if row["label"].startswith(("rust-unit-", "rust-target-facts-"))]
    if len(units) != len(manifest["units"]) or not all(row["cached"] for row in units):
        raise RuntimeError("clean-client native unit reuse was not observed")
    app = next(row for row in build["outcomes"] if row["label"] == campaign["observation_consumer_label"])
    if app["cached"]:
        raise RuntimeError("app observation consumer was cached; real replay app execution is required")
    observed = Path(next(item["path"] for item in app["outputs"] if item["name"] == "out"))
    baseline_observation = Path(campaign["offline_cargo_observation"]).resolve(strict=True)
    for name in ["normal.stdout", "normal.stderr", "normal.exit", "nonzero.stdout", "nonzero.stderr", "nonzero.exit"]:
        if (observed / name).read_bytes() != (baseline_observation / name).read_bytes():
            raise RuntimeError(f"offline Cargo app parity failed: {name}")
    rerun = run(network + [mantle, "--json", "--store-prefix", manifest["compatibility"]["store_prefix"],
                "--store", out / "clean-client/store", "--state-dir", out / "clean-client/state", "build",
                out / "clean-client/replay.ncl", "--no-substitute", "--trusted-public-keys", ",".join(campaign["trusted_public_keys"]),
                "--offline-source-preflight"], out / "unchanged-rerun")
    if rerun["counts"]["failed_total"] != 0 or rerun["counts"]["built_total"] != 0:
        raise RuntimeError("unchanged exported graph executed build work")
    summary = {"schema": "mantle-cargo-build-bundle-campaign-report-v1", "bundle_blake3": baseline["bundle_blake3"],
               "canonical_plan_blake3": baseline["canonical_plan_blake3"], "observed_units_reused": len(units),
               "network_denied_launcher": network, "negative_cases": len(negatives) + 15,
               "non_claim": manifest["non_claim"]}
    write(out / "campaign-report.json", summary)
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
