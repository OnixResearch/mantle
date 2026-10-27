#!/usr/bin/env python3
"""Fixed-input real Mantle builds and observe-only production selector replay.
Requires an explicitly trusted, signed inputs-only cache containing static BusyBox.
Metrics stay in persisted build logs, never in deterministic store outputs.
Each independent round creates its private physical store before signed pull.
Output verification selects the store-path basename and requires trusted results.
"""
import argparse
import gzip
import json
import os
from pathlib import Path
import platform
import re
import shlex
import subprocess
import time
from decimal import Decimal


MAX_REPORT_BYTES = 16 * 1024 * 1024
MARKER = "MANTLE_RESOURCE_MEASUREMENT_V1"


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def save(path, value):
    path.write_text(encoded(value) + "\n")


class Driver:
    def __init__(self, args):
        self.args = args
        self.root = args.output.resolve()
        self.root.mkdir(exist_ok=False)
        self.commands = []
        self.repo = Path(__file__).resolve().parents[1]

    def command(self, label, argv, data=None):
        started = time.monotonic_ns()
        with (self.root / (label + ".stdout")).open("xb") as out, (self.root / (label + ".stderr")).open("xb") as err:
            result = subprocess.run([str(x) for x in argv], input=data, stdout=out, stderr=err,
                                    timeout=120, env={**os.environ, "HOME": str(self.root / "empty-home")})
        self.commands.append({"label": label, "argv": [str(x) for x in argv], "exit": result.returncode,
                              "elapsed_ns": time.monotonic_ns() - started})
        save(self.root / "commands.json", self.commands)
        if result.returncode:
            raise RuntimeError(f"{label}: exit {result.returncode}; inspect retained stdout/stderr")
        path = self.root / (label + ".stdout")
        if path.stat().st_size > MAX_REPORT_BYTES:
            raise RuntimeError("benchmark report exceeds bound")
        return path.read_text()

    def digest(self, label, path):
        return self.command(label, [self.args.b3sum, "--no-names", path]).strip()

    def recipe(self, workload):
        bb = self.args.busybox_root + "/bin/busybox"
        count = workload["bytes"] // 1024
        if workload["kind"] not in ("write-zero", "gzip-zero") or not 1 <= count <= 16384:
            raise RuntimeError("unsupported corpus geometry")
        script = 'set -eu\numask 022\nulimit -v 65536\nulimit -f 32768\n'
        body = f'{shlex.quote(bb)} mkdir -p "$out"\n{shlex.quote(bb)} dd if=/dev/zero of="$out/data" bs=1024 count={count}\n'
        if workload["kind"] == "gzip-zero":
            body += f'{shlex.quote(bb)} gzip -n -c "$out/data" > "$out/data.gz"\n'
        fmt = MARKER + " elapsed_s=%e user_s=%U system_s=%S rss_kib=%M input_blocks=%I output_blocks=%O"
        script += f'exec {shlex.quote(bb)} time -f {shlex.quote(fmt)} {shlex.quote(bb)} sh -ec {shlex.quote(body)}'
        drv = {"name": "resource-" + workload["id"], "builder": bb, "args": ["sh", "-ec", script],
               "inputs": [self.args.busybox_root], "env": {"SOURCE_DATE_EPOCH": str(workload["source_date_epoch"])},
               "addressing_mode": "input-addressed"}
        return 'std.deserialize \'Json ' + json.dumps(encoded(drv)) + '\n'

    def run(self):
        version = self.command("nickel-version", [self.args.nickel, "--version"])
        if "1.17.0" not in version or (platform.system(), platform.machine()) != ("Linux", "x86_64"):
            raise RuntimeError("benchmark requires reviewed Nickel 1.17.0 / x86_64-linux cohort")
        corpus = json.loads(self.command("corpus", [self.args.nickel, "export", "--format", "json", self.repo / "config/resource-policy/benchmark.ncl"]))
        identities = {"mantle": self.digest("mantle-blake3", self.args.mantle),
                      "replay": self.digest("replay-blake3", self.args.replay),
                      "driver": self.digest("driver-blake3", Path(__file__).resolve()),
                      "corpus": self.digest("corpus-blake3", self.repo / "config/resource-policy/benchmark.ncl")}
        features = {"architecture": "x86_64", "platform": "linux", "trust_domain": "local-benchmark",
                    "isolation": "bwrap-rlimit-static-busybox", "kvm": False}
        quantities = lambda memory, scratch: {"cpu_units": 1, "memory_bytes": memory, "scratch_bytes": scratch}
        classes = [{"id": name, "features": features, "quantities": quantities(memory, scratch), "order": order,
                    "reservation_units": units, "available": True, "existing_admission_compatible": True}
                   for order, (name, memory, scratch, units) in enumerate([
                       ("small", 2**21, 2**21, 1), ("medium", 2**24, 2**25, 4), ("large", 2**26, 2**26, 16)])]
        declared = quantities(2**20, 2**20)
        records = []
        for workload in corpus["workloads"]:
            history, artifacts = [], []
            recipe = self.root / (workload["id"] + ".ncl")
            recipe.write_text(self.recipe(workload))
            for index in range(corpus["rounds"]):
                label = f'{workload["id"]}-{index}'
                state, store = self.root / (label + "-state"), self.root / (label + "-store")
                state.mkdir(mode=0o700)
                store.mkdir(mode=0o700)
                base = [self.args.mantle, "--nix-compat", "--store", store, "--state-dir", state]
                self.command(label + "-inputs", base + ["store", "pull", "--from", self.args.input_cache,
                    "--trusted-public-keys", self.args.trusted_public_key, "--all"])
                self.command(label + "-source", base + ["source", "bundle", "import-store-path", "--logical",
                    self.args.busybox_root, "--physical", store / Path(self.args.busybox_root).name,
                    "--trusted-public-keys", self.args.trusted_public_key, "--pin"])
                build = base + ["--json", "build", recipe, "--import-path", self.repo / "lib", "--no-substitute",
                                "--strict-hermetic", "--signing-key", self.args.signing_key,
                                "--trusted-public-keys", self.args.trusted_public_key]
                report = json.loads(self.command(label + "-build", build))
                outcome = report["outcomes"][0]
                if outcome["cached"]:
                    raise RuntimeError("clean benchmark unexpectedly reused an existing output")
                output = Path(outcome["outputs"][0]["path"])
                payload = output / "data"
                if payload.stat().st_size != workload["bytes"]:
                    raise RuntimeError("workload produced wrong byte geometry")
                with payload.open("rb") as stream:
                    while chunk := stream.read(65536):
                        if any(chunk):
                            raise RuntimeError("workload changed fixed zero contents")
                if workload["kind"] == "gzip-zero":
                    with gzip.open(output / "data.gz", "rb") as stream:
                        total = 0
                        while chunk := stream.read(65536):
                            total += len(chunk)
                            if any(chunk) or total > workload["bytes"]:
                                raise RuntimeError("gzip payload mismatch")
                        if total != workload["bytes"]:
                            raise RuntimeError("gzip payload truncated")
                artifact = {p.name: self.digest(label + "-" + p.name + "-blake3", p) for p in sorted(output.iterdir())}
                artifacts.append(artifact)
                if any(item != artifacts[0] for item in artifacts):
                    raise RuntimeError("independent clean store output bytes differ")
                logs = list((state / "logs").glob("*resource-" + workload["id"] + ".drv.log"))
                if len(logs) != 1 or logs[0].stat().st_size > MAX_REPORT_BYTES:
                    raise RuntimeError("unique bounded persisted builder log missing")
                matches = re.findall(MARKER + r" elapsed_s=([0-9.]+) user_s=([0-9.]+) system_s=([0-9.]+) rss_kib=([0-9]+) input_blocks=([0-9]+) output_blocks=([0-9]+)", logs[0].read_text())
                if len(matches) != 1:
                    raise RuntimeError("actual process-resource measurements missing or ambiguous")
                elapsed, user, system, rss, reads, writes = matches[0]
                ms = lambda value: int(Decimal(value) * 1000)
                observation = {"schema": "mantle-resource-observation-v2", "attempt_id": label,
                    "action_family": workload["id"], "features": features, "machine_class": "large",
                    "policy_version": 1, "declared": declared, "selected": classes[2]["quantities"],
                    "measured": {"cpu_units": None, "memory_bytes": None, "scratch_bytes": None},
                    "scratch_sampled_peak_bytes": None, "queue_ms": None,
                    "execution_ms": ms(elapsed), "terminal_unix_s": int(time.time()),
                    "cpu_ms": ms(user) + ms(system), "io_bytes": (int(reads) + int(writes)) * 512,
                    "executor_output_bytes": None, "process_peak_rss_bytes": int(rss) * 1024,
                    "transfer_bytes": None, "oom": "Unknown", "outcome": "Success", "predecessor": None,
                    "collector_version": "busybox-time-local-v2", "trusted_collector": False}
                history.append(observation)
                warm = json.loads(self.command(label + "-warm", build))
                if not warm["outcomes"][0]["cached"] or warm["outcomes"][0]["outputs"] != outcome["outputs"]:
                    raise RuntimeError("signed warm output identity/reuse mismatch")
                output_selector = output.name
                verified = self.command(label + "-verify", base + ["store", "verify", output_selector,
                    "--trusted-public-keys", self.args.trusted_public_key])
                if not re.fullmatch(r"OK " + re.escape(output_selector)
                                    + r"  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*\n", verified):
                    raise RuntimeError("signed output verification did not check the requested store path")
            replay_input = {"declared": declared, "features": features, "action_family": workload["id"],
                "classes": classes, "history": history, "now_unix_s": int(time.time()),
                "project_remaining_units": 1000, "account_remaining_units": 1000,
                "policy": {"version": 1, "min_samples": corpus["rounds"], "max_age_s": 3600,
                    "margin_basis_points": 2500, "max_measurement": classes[2]["quantities"],
                    "max_duration_ms": 120000, "max_io_bytes": 2**30}}
            save(self.root / (workload["id"] + "-replay.json"), replay_input)
            wire = (encoded(replay_input) + "\n").encode()
            decision = json.loads(self.command(workload["id"] + "-decision", [self.args.replay], wire))
            repeated = json.loads(self.command(workload["id"] + "-repeated-decision", [self.args.replay], wire))
            if repeated != decision or decision["active"] != decision["static_decision"]:
                raise RuntimeError("production replay is nondeterministic or observe-only changed dispatch")
            historical = decision["historical_decision"].get("Ok")
            expected_exclusions = [[index, "HistoryUntrusted"] for index in range(corpus["rounds"])]
            if (historical is None or historical["reason"] != "HistoryInsufficient"
                    or historical["samples"] != 0 or historical["exclusions"] != expected_exclusions
                    or historical["requested"] != declared):
                raise RuntimeError("process-only local observations gained historical selection authority")
            records.append({"workload": workload, "observations": history, "artifacts": artifacts,
                "decision": decision, "static_reservation_units": classes[decision["static_decision"]["Ok"]["class_index"]]["reservation_units"],
                "historical_reservation_units": classes[historical["class_index"]]["reservation_units"],
                "cost_scope": "reviewed class reservation units only; no billing or measured remote charge"})
        final_identities = {"mantle": self.digest("end-mantle-blake3", self.args.mantle),
                            "replay": self.digest("end-replay-blake3", self.args.replay),
                            "driver": self.digest("end-driver-blake3", Path(__file__).resolve()),
                            "corpus": self.digest("end-corpus-blake3", self.repo / "config/resource-policy/benchmark.ncl")}
        if final_identities != identities:
            save(self.root / "source-drift.json", {"start": identities, "end": final_identities})
            raise RuntimeError("benchmark source/executable bytes changed during execution")
        save(self.root / "evidence.json", {"schema": "mantle-resource-benchmark-v2", "source_identities": identities,
            "corpus": corpus, "records": records, "commands": self.commands,
            "scope": "real signed local builds; process-resource metrics; offline observe-only fail-closed replay",
            "measurement_limits": ["queue, transfer, executor-output bytes and scratch peaks are unknown, not zero",
                "process RSS is not an execution-tree or dedicated-cgroup memory peak",
                "large measurement class uses RLIMIT_AS and per-file RLIMIT_FSIZE, not a delegated worker cgroup",
                "local timer observations are not authenticated worker reports and are excluded from history",
                "this corpus does not measure historical placement benefits or remote accounting"],
            "non_claims": corpus["non_claims"]})
        print(self.root / "evidence.json")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("mantle", "replay", "nickel", "b3sum", "signing-key", "input-cache", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--trusted-public-key", required=True)
    parser.add_argument("--busybox-root", required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"/nix/store/[0-9a-z]{32}-[A-Za-z0-9+._?=-]+", args.busybox_root):
        parser.error("busybox-root must name a single admitted logical store root")
    Driver(args).run()


if __name__ == "__main__":
    main()
