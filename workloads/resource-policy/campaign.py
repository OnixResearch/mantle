#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Explicit artifact-bound command generation; run long stages only under guarded Pueue."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

SDK_REV = "a154ddd1320129a133bb84b8e13ad3df7e55a55f"
HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
NEGATIVES = {
    "broken-retry-link": "resource retry has fresh fence and exact predecessor",
    "double-charge": "resource duplicate completion charges once across reopen",
    "lost-reservation": "resource killed accounting leaves complete old or new transaction",
    "cas-bypass": "resource absent CAS denies result reuse",
}

def invoke(argv, *, output=None, env=None, expected=0, cwd=None, data=None):
    result = subprocess.run([str(x) for x in argv], env=env, cwd=cwd, input=data,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if output:
        Path(output).write_bytes(result.stdout + result.stderr)
    if result.returncode != expected:
        text = (result.stdout + result.stderr).decode(errors='replace')[-2000:]
        raise RuntimeError(f"exit {result.returncode}, expected {expected}: {shlex.join(map(str, argv))}\n{text}")
    return result.stdout

def digest(path):
    return "blake3:" + invoke(["b3sum", "--no-names", str(path)]).decode().strip()

def dump(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")

def prepare(args):
    out = Path(args.output).resolve()
    out.mkdir(parents=True, exist_ok=False)
    guest, kernel, tools = (Path(x).resolve(strict=True) for x in (args.guest, args.kernel, args.tools))
    program_headers = invoke(["readelf", "--program-headers", guest])
    if b"INTERP" in program_headers:
        raise RuntimeError("guest must be a static x86_64 Linux ELF (no interpreter)")
    header = invoke(["readelf", "--file-header", guest])
    if b"Advanced Micro Devices X86-64" not in header:
        raise RuntimeError("guest is not the reviewed x86_64 cohort")
    lock = HERE / "Cargo.lock"
    if not lock.is_file() or f"?rev={SDK_REV}#{SDK_REV}" not in lock.read_text():
        raise RuntimeError("generate the dedicated Cargo.lock with the pinned public SDK revision first")
    config_bytes = invoke(["nickel", "export", "--format", "json", HERE / "campaign.ncl"])
    config = json.loads(config_bytes)
    (out / "campaign.json").write_bytes(config_bytes)
    initrd = out / "initrd.gz"
    with tempfile.TemporaryDirectory(prefix="mantle-resource-initrd-") as directory:
        tree = Path(directory)
        for child in ["dev", "proc", "sys/kernel/debug", "tmp"]:
            (tree / child).mkdir(parents=True, exist_ok=True)
        shutil.copyfile(guest, tree / "init")
        (tree / "init").chmod(0o755)
        # Same newc+gzip packaging as ChaosControl mkChaosInitrd; sorted input.
        names = ["."] + sorted(str(p.relative_to(tree)) for p in tree.rglob("*"))
        cpio = invoke(["cpio", "--null", "-o", "-H", "newc", "--quiet"], cwd=tree,
                      data=("\0".join(names) + "\0").encode())
        # invoke returns stdout only: cpio/gzip diagnostics cannot corrupt bytes.
        initrd.write_bytes(invoke(["gzip", "-n", "-9"], data=cpio))
    paths = [guest, kernel, initrd, lock, HERE / "Cargo.toml", HERE / "campaign.ncl", Path(__file__).resolve(),
             ROOT / "crates/crunch-build/src/distributed/resource_quota_store.rs",
             ROOT / "crates/crunch-remote-core/src/resource_policy.rs",
             ROOT / "crates/crunch-remote-core/src/resource_quota.rs"]
    paths.extend(sorted((HERE / "src").glob("*.rs")))
    paths.append(ROOT / "crates/crunch-remote-core/Cargo.toml")
    paths.extend(sorted((ROOT / "crates/crunch-remote-core/src").rglob("*.rs")))
    for name in ["summarize-sdk-local-output", "check-sdk-assertion-quality", "chaoscontrol-explore"]:
        paths.append(tools / "bin" / name)
    source_revision = invoke(["git", "-C", ROOT, "rev-parse", "HEAD"]).decode().strip()
    commands = []
    for seed in config["seeds"]:
        command = [str(tools / "bin/chaoscontrol-explore"), "run", "--kernel", str(kernel), "--initrd", str(initrd),
                   "--seed", str(seed), "--mode", config["mode"], "--frontier-policy", "campaign",
                   "--scheduling", "round-robin", "--memory-mb", str(config["memory_mib"])]
        for flag, field in [("vms", "vms"), ("vcpus", "vcpus"), ("rounds", "rounds"), ("branches", "branches"),
                            ("ticks", "ticks"), ("quantum", "quantum"), ("bootstrap-budget", "bootstrap_budget"),
                            ("max-frontier", "max_frontier"), ("workers", "workers")]:
            command += ["--" + flag, str(config[field])]
        command += ["--output", str(out / f"seed-{seed}"), "--min-assertion-exercise", "1"]
        commands.append(command)
    replay_templates = {}
    for seed in config["seeds"]:
        common = ["--kernel", str(kernel), "--initrd", str(initrd), "--seed", str(seed),
                  "--vms", str(config["vms"]), "--vcpus", str(config["vcpus"]),
                  "--ticks", str(config["ticks"]), "--quantum", str(config["quantum"]),
                  "--bootstrap-budget", str(config["bootstrap_budget"]),
                  "--memory-mb", str(config["memory_mib"]), "--scheduling", "round-robin",
                  "--bug", "OBSERVED_EXPORTED_BUG"]
        replay_templates[str(seed)] = {
            "reproduce": [str(tools / "bin/chaoscontrol-explore"), "reproduce", *common,
                          "--verdict-output", str(out / f"seed-{seed}/replay-verdict.json")],
            "minimize": [str(tools / "bin/chaoscontrol-explore"), "minimize", *common,
                         "--output", str(out / f"seed-{seed}/minimized-schedule.json")]}
    manifest = {"schema": "mantle-resource-chaos-commands-v1", "sdk_public_source": f"https://github.com/brittonr/chaoscontrol?rev={SDK_REV}",
                "mantle_revision": source_revision, "source_identity_note": "file identities bind uncommitted edits as well as revision",
                "identities": {str(path): digest(path) for path in paths}, "guest": str(guest), "kernel": str(kernel),
                "initrd": str(initrd), "tools": str(tools), "config": config, "vm_commands": commands,
                "local_command": ["python3", str(HERE / "campaign.py"), "local", "--manifest", str(out / "commands.json")],
                "vm_command": ["python3", str(HERE / "campaign.py"), "vm", "--manifest", str(out / "commands.json")],
                "conditional_replay_templates_by_seed": replay_templates,
                "replay": {"status": "not-requested-no-observed-bug", "prerequisites": ["selected actual bug with current assertion identity", "valid retained parent snapshot and matching kernel/initrd", "no schedule-only promotion"],
                           "export_command_template": [str(tools / "bin/chaoscontrol-explore"), "export-bugs", "--checkpoint", "OBSERVED_CHECKPOINT", "--assertion-id", "OBSERVED_ASSERTION_ID", "--min-replay-parent-depth", "1", "--max-bugs", "1", "--no-overwrite"]}}
    dump(out / "commands.json", manifest)
    print(out / "commands.json")

def load_manifest(path):
    manifest = json.loads(Path(path).read_text())
    for name, identity in manifest["identities"].items():
        if digest(name) != identity:
            raise RuntimeError(f"artifact drift: {name}; prepare a fresh run, never reuse stale reports")
    return manifest, Path(path).resolve().parent

def local(manifest, out):
    root = out / "local"
    root.mkdir(exist_ok=False)
    tools = Path(manifest["tools"]) / "bin"
    for name in ["positive", *NEGATIVES]:
        target = root / name
        target.mkdir()
        env = dict(os.environ, CHAOSCONTROL_SDK_LOCAL_OUTPUT=str(target / "sdk.jsonl"))
        command = [manifest["guest"]]
        if name != "positive":
            command += ["--negative-control", name]
        invoke(command, output=target / "workload.log", env=env, expected=0 if name == "positive" else 1)
        invoke([tools / "summarize-sdk-local-output", "--input", target / "sdk.jsonl", "--output", target / "report.json"], output=target / "summary.log")
        invoke([tools / "check-sdk-assertion-quality", "--input", target / "report.json"], output=target / "assertion-quality.txt", expected=0 if name == "positive" else 1)
        if name != "positive":
            report = json.loads((target / "report.json").read_text())
            rows = report["assertion_coverage"]
            if not any(row["message"] == NEGATIVES[name] and row["failure_count"] > 0 for row in rows):
                raise RuntimeError(f"negative control failed for wrong reason: {name}")
    for index, args in enumerate([["--negative-control"], ["--negative-control", "unknown"], ["--unknown"], ["--negative-control", "x" * 129]]):
        env = dict(os.environ, CHAOSCONTROL_SDK_LOCAL_OUTPUT=str(root / f"rejected-{index}.jsonl"))
        invoke([manifest["guest"], *args], env=env, expected=2, output=root / f"rejected-{index}.log")
    dump(root / "receipt.json", {"evidence_class": "instrumentation-dry-run", "replay_evidence": False,
        "commands_manifest_identity": digest(out / "commands.json"), "negative_controls": list(NEGATIVES),
        "positive_report_identity": digest(root / "positive/report.json"), "non_claims": manifest["config"]["non_claims"]})

def vm(manifest, out):
    receipt = out / "local/receipt.json"
    if not receipt.is_file() or json.loads(receipt.read_text())["commands_manifest_identity"] != digest(out / "commands.json"):
        raise RuntimeError("current manifest-bound local instrumentation and negative controls required before VM")
    local_receipt = json.loads(receipt.read_text())
    positive = out / "local/positive/report.json"
    if local_receipt["positive_report_identity"] != digest(positive):
        raise RuntimeError("local instrumentation report changed after acceptance")
    expected_messages = {row["message"] for row in json.loads(positive.read_text())["assertion_coverage"]}
    observed_artifacts, results = {}, []
    kvm = os.open("/dev/kvm", os.O_RDWR | os.O_CLOEXEC)
    os.close(kvm)
    # Each explicit seed is independent; no timing/fleet claims.
    for command in manifest["vm_commands"]:
        seed = command[command.index("--seed") + 1]
        result = {"seed": int(seed), "accepted": False, "error": None}
        try:
            invoke(command, output=out / f"seed-{seed}.log")
            summary_path = out / f"seed-{seed}" / "assertions.json"
            summary = json.loads(summary_path.read_text())
            observed_artifacts[str(summary_path)] = digest(summary_path)
            rows = summary["assertions"]
            observed = {row["message"] for row in rows if row["hit_count"] > 0 and row["verdict"] == "passed"}
            result["missing_messages"] = sorted(expected_messages - observed)
            if summary["schema"] != "chaoscontrol.assertion-summary.v2" or not summary["collision_safe_evidence"]:
                raise RuntimeError("non-authoritative assertion catalog")
            if result["missing_messages"] or any(row["verdict"] != "passed" for row in rows):
                raise RuntimeError("missing or failing workload outcome/invariant assertions")
            if list((out / f"seed-{seed}").glob("bug_*.json")):
                raise RuntimeError("retained bug requires triage, not passing execution classification")
            result["accepted"] = True
        except (RuntimeError, OSError, ValueError, KeyError, TypeError) as error:
            result["error"] = str(error)
        results.append(result)
        dump(out / "campaign-results.json", {"manifest_identity": digest(out / "commands.json"),
            "evidence_class": "bounded-vm-execution-attempts", "replay_evidence": False,
            "seeds": results, "non_claims": manifest["config"]["non_claims"]})
    if not all(result["accepted"] for result in results):
        raise RuntimeError("declared campaign incomplete; inspect campaign-results.json and each retained seed log")
    dump(out / "execution-classification.json", {"evidence_class": "bounded-vm-execution", "replay_evidence": False,
        "verdict": "all-recorded-seeds-observed-all-workload-assertions-passing",
        "assertion_artifacts": observed_artifacts,
        "manifest_identity": digest(out / "commands.json"), "non_claims": manifest["config"]["non_claims"]})

parser = argparse.ArgumentParser(description=__doc__)
sub = parser.add_subparsers(dest="stage", required=True)
p = sub.add_parser("prepare")
for field in ["guest", "kernel", "tools", "output"]:
    p.add_argument("--" + field, required=True)
for name in ["local", "vm"]:
    p = sub.add_parser(name)
    p.add_argument("--manifest", required=True)
args = parser.parse_args()
if args.stage == "prepare":
    prepare(args)
else:
    m, directory = load_manifest(args.manifest)
    {"local": local, "vm": vm}[args.stage](m, directory)
