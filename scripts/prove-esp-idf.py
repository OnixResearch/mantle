#!/usr/bin/env python3
"""Real Mantle/signed-cache ESP-IDF proof. No tool mocks, fetching, or host SDK fallback."""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import struct
import subprocess

REPO = Path(__file__).resolve().parents[1]
RAW_ARTIFACTS = ["app.elf", "app.bin", "bootloader.bin", "partition-table.bin"]
MUTATIONS = {"source", "configuration", "partition", "sdk", "toolchain", "component"}
LOCK_NEGATIVES = {"lock-target-mismatch", "stale-manifest", "missing-offline-component"}
EXPECTED_ERRORS = {
    "digest-drift": "input digest drift", "path-escape": "nonescaping ESP-IDF member",
    "undeclared-role": "undeclared role", "unsupported-target": "unsupported cohort",
    "toolchain-mismatch": "TargetToolchainMismatch", "missing-dependency": "mantle_message",
    "lock-target-mismatch": "frozen lock target/SDK mismatch",
    "stale-manifest": "frozen manifest/lock mismatch",
    "missing-offline-component": "missing frozen dependency",
}


def require(value, message):
    if not value:
        raise RuntimeError(message)


def command(args, directory, label, *, negative=False, env=None):
    process = subprocess.run([str(arg) for arg in args], capture_output=True, text=True, env=env)
    (directory / (label + ".stdout")).write_text(process.stdout)
    (directory / (label + ".stderr")).write_text(process.stderr)
    (directory / (label + ".command.json")).write_text(json.dumps([str(arg) for arg in args]))
    require((process.returncode != 0) if negative else (process.returncode == 0),
            f"unexpected exit {process.returncode}: {label}")
    return process.stdout


def scenario_inputs(plan, scenario):
    def bound(member):
        return [member, plan["roles"][member["role"]]]
    if scenario == "source":
        return bound(plan["project"])
    if scenario == "configuration":
        return [bound(plan["project"]), plan["sdkconfig"], plan["common_defaults"], plan["target_defaults"]]
    if scenario == "partition":
        return [bound(plan["project"]), plan["partition_csv"]]
    if scenario == "sdk":
        return bound(plan["sdk"])
    if scenario == "toolchain":
        return [bound(plan["compiler"]), [bound(member) for member in plan["path"]]]
    return [[entry, bound(entry["source"])] for entry in plan["components"]]


def author(plan, runner, prefix):
    # Plan is data, never interpolated executable Nickel syntax.
    encoded = json.dumps(json.dumps(plan, sort_keys=True))
    return ('let esp = import "esp_idf.ncl" in\n'
            + 'let plan = std.deserialize \'Json ' + encoded + ' in\n'
            + '{ firmware = esp.mkFirmware { plan, runner = { role = '
            + json.dumps(runner["role"]) + ', member = ' + json.dumps(runner["member"])
            + ' }, store_prefix = ' + json.dumps(prefix) + ' } }\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", required=True)
    parser.add_argument("--cohort", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    cohort = json.loads(args.cohort.read_text())
    require(cohort["schema"] == "mantle-esp-idf-proof-cohort-v1", "wrong cohort schema")
    require(set(cohort["targets"]) == {"esp32s3", "esp32c3"}, "both exact targets required")
    require(cohort["store_prefix"] == "/nix/store", "proof uses explicit Nix-compatible signed input cache")
    work = args.work.resolve()
    work.mkdir(parents=True, exist_ok=False)
    base = args.cohort.resolve().parent
    env = dict(os.environ, IDF_PATH="/forbidden/ambient-sdk", IDF_TOOLS_PATH="/forbidden/ambient-tools",
               VIRTUAL_ENV="/forbidden/ambient-venv", PYTHONPATH="/forbidden/ambient-python")
    summary = {"schema": "mantle-esp-idf-proof-v1", "targets": {},
               "raw_equality": RAW_ARTIFACTS, "diagnostic_normalization": "none applied to raw comparison"}

    def read_plan(name):
        return json.loads((base / name).read_text())

    def initialize(directory):
        directory.mkdir()
        state, store = directory / "state", directory / "store"
        command([args.mantle, "--nix-compat", "--state-dir", state, "--store", store,
                 "store", "pull", "--from", cohort["input_cache"], "--all",
                 "--trusted-public-keys", cohort["trusted_key"]], directory, "admit-inputs", env=env)
        return state, store

    def build(plan, directory, state, store, label, negative=False, expected_error=None):
        root = directory / (label + ".ncl")
        root.write_text(author(plan, cohort["runner"], cohort["store_prefix"]))
        logs = directory / (label + "-builder-logs")
        build_env = dict(env, CRUNCH_LOG_DIR=str(logs))
        text = command([args.mantle, "--json", "--nix-compat", "--state-dir", state, "--store", store,
                        "build", "--no-substitute", "--signing-key", cohort["signing_key"],
                        "--trusted-public-keys", cohort["trusted_key"], "-I", REPO / "lib", root],
                       directory, label, negative=negative, env=build_env)
        if negative:
            messages = text + (directory / (label + ".stderr")).read_text()
            for path in logs.rglob("*"):
                if path.is_file() and path.stat().st_size <= 16 * 1024 * 1024:
                    messages += path.read_text(errors="replace")
            require(expected_error and expected_error.lower() in messages.lower(),
                    "negative failed for an unproven/unrelated reason: " + label)
            return None
        report = json.loads(text)
        rows = report["outcomes"]
        require(len(rows) == 1, "expected exactly one generic firmware derivation")
        paths = [item["path"] for item in rows[0]["outputs"] if item["name"] == "out"]
        require(len(paths) == 1, "missing unique firmware output")
        output = store / Path(paths[0]).name
        require(output.is_dir(), "output not exported in isolated physical store")
        verified = command([args.mantle, "--nix-compat", "--state-dir", state, "--store", store,
                            "store", "verify", output.name, "--trusted-public-keys", cohort["trusted_key"]],
                           directory, label + "-verify", env=env)
        require(re.search(r"^OK " + re.escape(output.name) + r"  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*$",
                          verified, re.MULTILINE), "output did not pass trusted signed store verification")
        return output, rows[0]

    for target, inputs in cohort["targets"].items():
        require(set(inputs["changed_plans"]) == MUTATIONS, "all six invalidation scenarios required")
        require(set(inputs["negative_plans"]) == LOCK_NEGATIVES, "real prepared lock/component negatives required")
        plan = read_plan(inputs["plan"])
        require(plan["target"] == target, "target plan mismatch")
        first, second = work / (target + "-directory-a"), work / (target + "-different-directory-b")
        state_a, store_a = initialize(first)
        state_b, store_b = initialize(second)
        output_a, row_a = build(plan, first, state_a, store_a, "clean")
        output_b, row_b = build(plan, second, state_b, store_b, "clean")
        require(not row_a["cached"] and not row_b["cached"], "clean builds must actually execute")
        equality = {}
        for name in RAW_ARTIFACTS:
            raw_a = (output_a / "firmware" / name).read_bytes()
            raw_b = (output_b / "firmware" / name).read_bytes()
            require(raw_a == raw_b, f"raw reproducibility failed: {target}/{name}")
            equality[name] = True
        elf = (output_a / "firmware/app.elf").read_bytes()
        require(elf[:4] == b"\x7fELF" and elf[4:6] == b"\x01\x01", "not a real little-endian ELF32")
        require(struct.unpack_from("<H", elf, 18)[0] == (94 if target == "esp32s3" else 243),
                "ELF architecture mismatch")
        layout = json.loads((output_a / "flash-layout.json").read_text())
        require(layout["target"] == target, "flash layout target mismatch")
        for path in layout["flash_files"].values():
            require(path.startswith("firmware/") and ".." not in Path(path).parts
                    and (output_a / path).is_file(), "flash artifact path escapes output")
        _, cached = build(plan, first, state_a, store_a, "cached")
        require(cached["cached"], "unchanged firmware did not reuse signed result")
        managed = read_plan(inputs["managed_plan"])
        require(managed["target"] == target and any(entry["managed"] for entry in managed["components"]),
                "managed positive must contain a real managed component for this target")
        managed_a, managed_row_a = build(managed, first, state_a, store_a, "managed-clean")
        managed_b, managed_row_b = build(managed, second, state_b, store_b, "managed-clean")
        require(not managed_row_a["cached"] and not managed_row_b["cached"], "managed positives must execute")
        for name in RAW_ARTIFACTS:
            require((managed_a / "firmware" / name).read_bytes() == (managed_b / "firmware" / name).read_bytes(),
                    "managed raw reproducibility failed: " + name)
        identities = {}
        for name, path in inputs["changed_plans"].items():
            changed = read_plan(path)
            require(changed["target"] == target and changed["name"] == plan["name"],
                    "invalidation scenario changed target/name instead of its input")
            require(scenario_inputs(changed, name) != scenario_inputs(plan, name),
                    "scenario did not change its named input: " + name)
            output, row = build(changed, first, state_a, store_a, "changed-" + name)
            require(not row["cached"] and output.name != output_a.name, "input change failed to invalidate: " + name)
            identities[name] = output.name
        negatives = {}
        for name, change in {
            "digest-drift": lambda p: p["roles"][p["project"]["role"]].update(tree_blake3="f" * 64),
            "path-escape": lambda p: p["project"].update(member="../undeclared"),
            "undeclared-role": lambda p: p["sdk"].update(role="ambient"),
            "unsupported-target": lambda p: p.update(target="esp8266"),
            "toolchain-mismatch": lambda p: p.update(target="esp32c3" if target == "esp32s3" else "esp32s3"),
            "missing-dependency": lambda p: p.update(components=[]),
        }.items():
            negative = copy.deepcopy(plan)
            change(negative)
            build(negative, first, state_a, store_a, "negative-" + name, negative=True,
                  expected_error=EXPECTED_ERRORS[name])
            negatives[name] = "rejected"
        for name, path in inputs["negative_plans"].items():
            build(read_plan(path), first, state_a, store_a, "negative-" + name, negative=True,
                  expected_error=EXPECTED_ERRORS[name])
            negatives[name] = "rejected"
        summary["targets"][target] = {"clean_outputs": [output_a.name, output_b.name],
                                       "managed_raw_byte_equal": True,
                                       "raw_byte_equal": equality, "cache_reused": True,
                                       "changed_outputs": identities, "negatives": negatives}
    (work / "receipt.json").write_text(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
