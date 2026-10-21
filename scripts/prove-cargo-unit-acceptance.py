#!/usr/bin/env python3
"""Execute a pinned Cargo-unit acceptance campaign, not a receipt-only checker.

Usage: python3 scripts/prove-cargo-unit-acceptance.py MANIFEST.json NEW_DIRECTORY

MANIFEST schema mantle-cargo-unit-acceptance-request-v1:
  mantle: absolute executable; mantle_blake3: exact b3sum digest;
  source_revision: full immutable Mantle commit used to build those tools;
  input_cache: signed file cache; trusted_public_keys: nonempty array;
  signing_key/public_key: explicit publisher key file and matching trusted key;
  store_prefix: /nix/store or /mantle/store;
  import_paths: Nickel import directories; timeout_seconds: finite command bound;
  cases: cold, source_edit, unrelated_edit, native_change, offline. Each contains
    goal: immutable .ncl file; goal_blake3; config: logical producer-config path;
    app_crate: rustc crate name (offline uses output_relative, e.g. bin/app).
    The offline case additionally has contract: the exact cold config JSON.
  expected: {stdout: string, stderr: string, exit: integer, argv: [], env: {}};
  affected_sources: {source_edit: [source IDs], native_change: [source IDs]};
  negatives: map of unknown-directive, error-directive, nonzero-exit,
    duplicate-links, undeclared-role, removed-native, host-path, network to
    {goal, goal_blake3, error: stable failure/log substring}.
  controls: array of {argv: [absolute real executable, ...], stdout: string}.
    Use the same admitted probe executable for outside/inside host/network cases.

All goals, configs, fixture/vendor/lock/tool/runtime roots must be prepared from
one immutable source revision and included in input_cache. Preparation may use
Nix to export signed inputs; no Nix process is used by this runtime workflow.
Goals must invoke the existing cargo_unit_plan/offlineCargoPackage contracts;
this runner does not replace their planner, scheduler, sandbox, or trust policy.
An existing output directory is rejected, including interrupted campaigns.
"""
import collections
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest(path):
    return subprocess.check_output(["b3sum", "--", str(path)], text=True).split()[0]


def load(path):
    return json.loads(Path(path).read_bytes())


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")

def output_bytes(path):
    path = Path(path)
    if path.is_symlink():
        return len(os.readlink(path).encode())
    if path.is_file():
        return path.stat().st_size
    return sum(output_bytes(item) for item in path.iterdir())



class Campaign:
    def __init__(self, manifest, destination):
        self.request = load(manifest)
        self.root = Path(destination).absolute()
        self.root.mkdir()  # no partial-state continuation or accidental warm cold run
        shutil.copyfile(manifest, self.root / "request.json")
        self.m = self.request
        require(self.m["schema"] == "mantle-cargo-unit-acceptance-request-v1", "request schema")
        require(len(self.m["source_revision"]) == 40, "full immutable source revision required")
        require(digest(self.m["mantle"]) == self.m["mantle_blake3"], "Mantle binary identity mismatch")
        require(self.m["trusted_public_keys"] and self.m["public_key"], "explicit trust required")
        require(0 < self.m["timeout_seconds"] <= 86400, "finite command deadline required")
        self.trust = ",".join(self.m["trusted_public_keys"] + [self.m["public_key"]])
        self.sequence = 0
        self.summary = {}
        self.transfers = []

    def command(self, name, argv, success=True, env=None, cwd=None):
        self.sequence += 1
        directory = self.root / f"{self.sequence:03}-{name}"
        directory.mkdir()
        write(directory / "command.json", {"argv": list(map(str, argv)), "cwd": str(cwd) if cwd else None, "environment": env})
        with (directory / "stdout").open("wb") as stdout, (directory / "stderr").open("wb") as stderr:
            result = subprocess.run(list(map(str, argv)), stdout=stdout, stderr=stderr,
                                    env=env, cwd=cwd, timeout=self.m["timeout_seconds"], check=False)
        write(directory / "exit.json", result.returncode)
        if success:
            require(result.returncode == 0, f"{name} exited {result.returncode}; see {directory}")
        return directory, result.returncode

    def args(self, client):
        base = self.root / client
        (base / "store").mkdir(parents=True, exist_ok=True)
        (base / "state").mkdir(parents=True, exist_ok=True)
        return [self.m["mantle"], "--store", str(base / "store"), "--state-dir", str(base / "state"), "--store-prefix", self.m["store_prefix"]]

    def physical(self, logical, client):
        prefix = self.m["store_prefix"] + "/"
        require(logical.startswith(prefix) and "/" not in logical[len(prefix):], f"not a store root: {logical}")
        return self.root / client / "store" / logical[len(prefix):]

    def pull(self, client, cache):
        directory, _ = self.command(f"{client}-pull", self.args(client) + ["store", "pull", "--from", cache, "--trusted-public-keys", self.trust, "--all"])
        text = (directory / "stderr").read_text()
        counters = dict(__import__("re").findall(r"(imported|nar_bytes|skipped_[a-z_]+)=(\d+)", text))
        require(int(counters.get("imported", "0")) > 0, "empty signed import is not cache evidence")
        for key in ("skipped_untrusted", "skipped_hash_mismatch", "skipped_missing_nar", "skipped_parse_error"):
            require(counters.get(key) == "0", f"pull rejection or missing counter {key}: {text}")
        self.transfers.append({"client": client, "cache": str(cache), "observed_counters": {key: int(value) for key, value in counters.items()}})

    def build(self, name, case, client="publisher", negative=False):
        require(digest(case["goal"]) == case["goal_blake3"], f"mutable goal: {name}")
        argv = self.args(client) + ["--json", "build", "--no-substitute", "--signing-key", self.m["signing_key"], "--trusted-public-keys", self.trust]
        for path in self.m.get("import_paths", []):
            argv += ["-I", path]
        directory, status = self.command(name, argv + [case["goal"]], success=False)
        try:
            report = load(directory / "stdout")
        except json.JSONDecodeError:
            require(negative and status != 0, f"missing structured success report: {name}")
            require(case["error"] in (directory / "stderr").read_text(), f"early failure missing expected diagnostic: {name}")
            return None
        write(directory / "report.json", report)
        if negative:
            require(status != 0 and report["counts"]["failed_total"] > 0, f"negative unexpectedly succeeded: {name}")
            evidence = "\n".join(failure["message"] for failure in report.get("failed", []))
            for index, failure in enumerate(report.get("failed", [])):
                path = failure.get("saved_log_path")
                if path and Path(path).is_file():
                    shutil.copyfile(path, directory / f"failure-{index}.log")
                    evidence += Path(path).read_text(errors="replace")
            require(case["error"] in evidence, f"negative missing expected diagnostic: {name}")
            return report
        require(status == 0 and report["counts"]["failed_total"] == 0, f"build failed: {name}")
        return self.observe(name, case, report, directory, client)

    def observe(self, name, case, report, directory, client):
        if name == "offline":
            require(not report.get("native_dynamic_plans"), "offline comparison must use offlineCargoPackage")
            roots = [output["path"] for row in report["outcomes"] for output in row["outputs"] if output["name"] == "out"]
            require(len(roots) == 1, "offline goal must expose only its package root")
            executable = Path(roots[0]) / case["output_relative"]
            evidence = load(Path(roots[0]) / "share/mantle/offline-cargo-build.json")
            contract = self.summary["cold"]["config"]
            closure = {row["role"]: row["path"] for row in evidence["source_closure"]}
            require(closure["package-source"] == contract["workspace"] and closure["rust-toolchain"] == contract["toolchain"], "offline realized source/toolchain differs")
            require(evidence["target"] == contract["target_triple"] and evidence["profile"] == ("debug" if contract["profile"] == "dev" else "release"), "offline realized target/profile differs")
            require(not contract.get("features") and not contract.get("no_default_features"), "offline helper comparison currently supports default selected features only")
            if contract.get("vendor"):
                require(closure["vendored-dependencies"] == contract["vendor"], "offline realized vendor differs")
            shutil.copyfile(Path(roots[0]) / "share/mantle/offline-cargo-build.json", directory / "offline-cargo-build.json")
            return self.app(name, executable)
        plans = report["native_dynamic_plans"]
        require(len(plans) == 1 and plans[0]["rejection_reason"] is None, "one admitted Cargo plan required")
        admission = plans[0]
        plan = load(self.physical(admission["plan_artifact_path"], client))
        require(set(admission["accepted_unit_ids"]) == {unit["id"] for unit in plan["units"]}, "admitted unit set mismatch")
        write(directory / "plan.json", plan)
        units = {}
        counts = collections.defaultdict(lambda: {"executed": 0, "reused": 0})
        app = None
        for unit in plan["units"]:
            derivation = unit["derivation"]
            matches = [row for row in report["outcomes"] if row["drv_key"].endswith("-" + derivation["name"] + ".drv")]
            require(len(matches) == 1, f"missing/duplicate observed unit {unit['id']}")
            outcome = matches[0]
            args, env = derivation["args"], derivation["env"]
            mode = args[0] if args and args[0] in ("target-facts", "run-build-script") else "compile"
            counts[mode]["reused" if outcome["cached"] else "executed"] += 1
            units[unit["id"]] = {"kind": mode, "source": env.get("MANTLE_UNIT_SOURCE"), "drv_key": outcome["drv_key"], "cached": outcome["cached"], "outputs": outcome["outputs"], "dependencies": [item["unit"] for item in derivation["inputs"] if item["kind"] == "unit_output"]}
            units[unit["id"]]["output_bytes"] = sum(output_bytes(item["path"]) for item in outcome["outputs"])
            crate = next((args[i + 1] for i, arg in enumerate(args[:-1]) if arg == "--crate-name"), None)
            crate = crate or next((arg.split("=", 1)[1] for arg in args if arg.startswith("--crate-name=")), None)
            if mode == "compile" and crate == case["app_crate"]:
                require(app is None, "ambiguous app compilation")
                output = next(item["path"] for item in outcome["outputs"] if item["name"] == "out")
                manifest = load(Path(output) / "share/mantle/unit-dependencies-v1.json")
                executable = [Path(output) / item for item in manifest["artifacts"] if os.access(Path(output) / item, os.X_OK) and not item.endswith((".so", ".dylib", ".dll"))]
                require(len(executable) == 1, "one executable app artifact required")
                app = executable[0]
        require(app is not None, "app unit not observed")
        producer_rows = [row for row in report["outcomes"] if row["drv_key"] == admission["producer_key"]]
        require(len(producer_rows) == 1, "producer observation missing")
        producer = producer_rows[0]
        producer_out = next(item["path"] for item in producer["outputs"] if item["name"] == "out")
        for receipt in ("cargo-unit-plan-evidence.json", "cargo-unit-graph.json", "cargo-metadata.json", "rust-plan-receipt.json", "cargo-unit-bindings.json"):
            shutil.copyfile(Path(producer_out) / receipt, directory / receipt)
        provenance = load(directory / "cargo-unit-bindings.json")
        require(provenance["schema"] == "mantle-cargo-unit-bindings-v1", "unit binding schema mismatch")
        require(set(provenance["units"]) == {key for key, row in units.items() if row["kind"] != "target-facts"}, "incomplete package/unit identity receipt")
        for key, identity in provenance["units"].items():
            units[key]["identity"] = identity
        config = load(self.physical(case["config"], client))
        lock = self.physical(config["workspace"], client) / "Cargo.lock"
        require(digest(lock) == load(directory / "cargo-unit-plan-evidence.json")["lock_blake3"], "source lock mismatch")
        self.app(name, app)
        for kind, row in counts.items():
            row["requested"] = row["executed"] + row["reused"]
            row["invalidated"] = 0
            row["reused_output_bytes"] = sum(unit["output_bytes"] for unit in units.values() if unit["kind"] == kind and unit["cached"])
            row["executed_output_bytes"] = sum(unit["output_bytes"] for unit in units.values() if unit["kind"] == kind and not unit["cached"])
        result = {"plan_digest": admission["canonical_plan_digest"], "slices": admission["source_slices"], "units": units, "counts_by_kind": dict(counts), "producer_cached": producer["cached"], "producer_drv_key": producer["drv_key"], "planning": {"executed": int(not producer["cached"]), "reused": int(producer["cached"])}, "config": config}
        self.summary[name] = result
        write(directory / "observations.json", result)
        return result

    def app(self, name, executable):
        expected = self.m["expected"]
        directory, status = self.command(name + "-app", [executable] + expected["argv"], success=False, env=expected["env"], cwd="/")
        require((directory / "stdout").read_bytes() == expected["stdout"].encode() and (directory / "stderr").read_bytes() == expected["stderr"].encode() and status == expected["exit"], f"observable app parity failed: {name}")
        return directory

    def unchanged(self, baseline, result):
        require(result["plan_digest"] == baseline["plan_digest"], "unchanged plan digest differs")
        require({key: row["drv_key"] for key, row in result["units"].items()} == {key: row["drv_key"] for key, row in baseline["units"].items()}, "unchanged unit derivation identities differ")
        require(all(row["cached"] for row in result["units"].values()), "unchanged unit executed")
        require([(s["source_id"], s["admitted_store_path"]) for s in result["slices"]] == [(s["source_id"], s["admitted_store_path"]) for s in baseline["slices"]], "unaffected slice identities differ")

    def edited(self, baseline, result, name):
        require(result["plan_digest"] != baseline["plan_digest"], "edited plan digest did not change")
        affected = set(self.m["affected_sources"][name])
        require(affected, "explicit affected package source IDs required")
        before = {row["source_id"]: row["admitted_store_path"] for row in baseline["slices"]}
        after = {row["source_id"]: row["admitted_store_path"] for row in result["slices"]}
        require(before.keys() == after.keys(), "edit changed package set")
        changed = {key for key in before if before[key] != after[key]}
        require(changed == (affected if name == "source_edit" else set()), "unexpected package source invalidation")
        before_drvs = {row["drv_key"] for row in baseline["units"].values()}
        executed = [row for row in result["units"].values() if not row["cached"]]
        require(executed, "affected units did not execute")
        require(all((row["drv_key"] in before_drvs) == row["cached"] for row in result["units"].values()), "unchanged unit rebuilt or changed unit reused")
        require(all(row["kind"] != "target-facts" for row in executed), "edit rebuilt shared target facts")
        invalidated = {key for key, row in result["units"].items() if row["source"] in {"{{mantle-source:" + source + "}}" for source in affected} and (name == "source_edit" or row["kind"] == "run-build-script")}
        while True:
            closure = invalidated | {key for key, row in result["units"].items() if set(row["dependencies"]) & invalidated}
            if closure == invalidated:
                break
            invalidated = closure
        require(invalidated == {key for key, row in result["units"].items() if not row["cached"]}, "execution set differs from affected units and reverse dependencies")
        for key in invalidated:
            result["counts_by_kind"][result["units"][key]["kind"]]["invalidated"] += 1
        require(any(row["cached"] for row in result["units"].values()), "no independent reused units")

    def run(self):
        self.pull("publisher", self.m["input_cache"])
        cases = self.m["cases"]
        cold = self.build("cold", cases["cold"])
        require(all(not row["cached"] for row in cold["units"].values()) and not cold["producer_cached"], "cold run used previous results")
        warm = self.build("warm", cases["cold"])
        self.unchanged(cold, warm)
        edited = self.build("source_edit", cases["source_edit"])
        self.edited(cold, edited, "source_edit")
        unrelated = self.build("unrelated_edit", cases["unrelated_edit"])
        self.unchanged(cold, unrelated)
        native = self.build("native_change", cases["native_change"])
        self.edited(cold, native, "native_change")
        require(cases["offline"]["contract"] == cold["config"], "offline Cargo inputs/profile/target/features differ")
        self.build("offline", cases["offline"])
        required = {"unknown-directive", "error-directive", "nonzero-exit", "duplicate-links", "undeclared-role", "removed-native", "host-path", "network"}
        require(set(self.m["negatives"]) == required, "incomplete negative campaign")
        require(len(self.m["controls"]) >= 2, "outside host and network controls required")
        for index, control in enumerate(self.m["controls"]):
            directory, _ = self.command(f"outside-control-{index}", control["argv"], env={})
            require((directory / "stdout").read_text() == control["stdout"], "outside control output mismatch")
        for name, case in self.m["negatives"].items():
            self.build("negative-" + name, case, negative=True)
        self.command("sign", self.args("publisher") + ["store", "sign", "--all", "--signing-key", self.m["signing_key"]])
        cache = self.root / "results-cache"
        selectors = sorted({Path(output["path"]).name for row in cold["units"].values() for output in row["outputs"]})
        # Include producer plan/sources/evidence as well as units. Store push follows references.
        selectors += [Path(output["path"]).name for row in load(next(self.root.glob("*-cold")) / "report.json")["outcomes"] for output in row["outputs"]]
        self.command("push", self.args("publisher") + ["store", "push", "--to", cache] + sorted(set(selectors)))
        self.pull("clean", self.m["input_cache"])
        self.pull("clean", cache)
        self.command("clean-verify", self.args("clean") + ["store", "verify", "--trusted-public-keys", self.trust])
        clean = self.build("clean", cases["cold"], client="clean")
        self.unchanged(cold, clean)
        write(self.root / "summary.json", {"schema": "mantle-cargo-unit-acceptance-observation-v1", "source_revision": self.m["source_revision"], "mantle_blake3": self.m["mantle_blake3"], "runs": self.summary, "transfers": self.transfers, "non_claims": ["not-full-cargo-parity", "not-release-reproducibility", "not-compiler-correctness"]})
        print(self.root / "summary.json")


if __name__ == "__main__":
    require(len(sys.argv) == 3, __doc__)
    Campaign(sys.argv[1], sys.argv[2]).run()
