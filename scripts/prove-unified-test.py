#!/usr/bin/env python3
"""Real Mantle issue-20 scenarios. Requires admitted immutable inputs; never compiles or mocks a runner."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", required=True, type=Path)
    parser.add_argument("--target", required=True, type=Path, help="complete target JSON for admitted real fixture and coverage tools")
    parser.add_argument("--state-dir", required=True, type=Path)
    parser.add_argument("--store", required=True, type=Path)
    parser.add_argument("--store-prefix", default="/mantle/store", help="logical admitted store prefix; use /nix/store for signed Nix tool closures")
    parser.add_argument("--scratch", required=True, type=Path, help="new scenario directory; must not exist")
    parser.add_argument("--baseline-receipt", required=True, type=Path, help="#15 accepted baseline evidence; recorded, not reinterpreted")
    args = parser.parse_args()
    args.scratch.mkdir(parents=True, exist_ok=False)
    target = json.loads(args.target.read_text())
    if target.get("coverage") is None:
        parser.error("the immutable fixture must be instrumented and declare matching llvm-tools coverage")
    # Presence on the host is essential: absence alone would not demonstrate isolation.
    sentinel = args.scratch.resolve() / "host-only-sentinel"
    sentinel.write_text("must not enter the sandbox\n")
    target.setdefault("environment", {})["TEST_UNDECLARED_PATH"] = str(sentinel)
    target["environment"]["TEST_EXPECT"] = "reject"
    records = []
    prefix = [str(args.mantle.resolve()), "--json", "--log-level", "info", "--state-dir", str(args.state_dir.resolve()), "--store", str(args.store.resolve()), "--store-prefix", args.store_prefix]

    def scenario(name, options, expected, code, changed=None):
        path = args.scratch / f"{name}.json"
        path.write_text(json.dumps(target if changed is None else changed, sort_keys=True))
        command = prefix + ["test", str(path.resolve())] + options
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        try:
            stdout, stderr = process.communicate(timeout=120)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
            raise AssertionError(f"{name}: Mantle exceeded proof-driver guard; not a test deadline observation")
        (args.scratch / f"{name}.stdout").write_bytes(stdout)
        (args.scratch / f"{name}.stderr").write_bytes(stderr)
        report = json.loads(stdout)
        assert process.returncode == code, (name, process.returncode, stderr.decode(errors="replace"))
        assert report["schema"] == "mantle-test-report-v1" and report["outcome"] == expected, (name, report)
        records.append({"scenario": name, "command": command, "exit": process.returncode, "report": report})
        return report

    passed = scenario("pass", ["--filter", "pass_"], "pass", 0)
    assert passed["selected"] == ["pass_alpha", "pass_beta"]
    cached = scenario("cache", ["--filter", "pass_"], "pass", 0)
    assert cached == passed, "identical signed result must have stable machine content"
    forced = scenario("force", ["--filter", "pass_", "--force-rerun", "issue20-proof-forced-1"], "pass", 0)
    assert forced["identity"] != passed["identity"]
    changed = copy.deepcopy(target)
    changed["environment"]["TEST_EXPECT"] = "changed"
    invalidated = scenario("invalidation", ["--filter", "pass_"], "pass", 0, changed)
    assert invalidated["identity"] != passed["identity"]
    failure = scenario("assertion", ["--filter", "assertion_failure", "--exact"], "assertion-failure", 1)
    assert failure["attempts"][0]["outcome"] == "assertion-failure"
    timeout = scenario("timeout", ["--filter", "timeout_owned_child", "--exact", "--timeout-ms", "100", "--force-rerun", "issue20-timeout-1"], "timeout", 124)
    assert timeout["attempts"][0]["reason"] == "deadline"
    for name in ["crash", "partial_output"]:
        scenario(name, ["--filter", name, "--exact", "--force-rerun", "issue20-crash-1"], "infrastructure-failure", 2)
    shard0 = scenario("shard0", ["--filter", "pass_", "--shard-count", "2", "--shard-index", "0"], "pass", 0)
    shard1 = scenario("shard1", ["--filter", "pass_", "--shard-count", "2", "--shard-index", "1"], "pass", 0)
    assert set(shard0["selected"]).isdisjoint(shard1["selected"])
    assert sorted(shard0["selected"] + shard1["selected"]) == passed["selected"]
    repeated = scenario("seed-repeat", ["--filter", "pass_alpha", "--exact", "--seed", "17", "--repeat", "2"], "pass", 0)
    assert [a["repeat"] for a in repeated["attempts"]] == [0, 1]
    coverage = scenario("coverage", ["--filter", "pass_", "--coverage"], "pass", 0)
    assert 0 < coverage["coverage"]["covered_lines"] <= coverage["coverage"]["total_lines"]
    assert coverage["coverage"]["source_blake3"] == target["coverage"]["source_blake3"]
    scenario("denied-input", ["--filter", "undeclared_input_denied", "--exact"], "pass", 0)
    evidence = {
        "schema": "mantle-unified-test-proof-v1",
        "target_sha256": hashlib.sha256(args.target.read_bytes()).hexdigest(),
        "baseline_receipt_sha256": hashlib.sha256(args.baseline_receipt.read_bytes()).hexdigest(),
        "records": records,
        "non_claim": "Local bounded libtest observations only; no universal framework support, formal correctness, release parity, or external proof authority.",
    }
    (args.scratch / "evidence.json").write_text(json.dumps(evidence, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"schema": evidence["schema"], "scenarios": len(records), "evidence": str(args.scratch / "evidence.json")}, sort_keys=True))


if __name__ == "__main__":
    main()
