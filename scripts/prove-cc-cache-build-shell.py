#!/usr/bin/env python3
"""Exercise production Mantle build-shell caching under protected execution.

Consumes a passed prove-cc-cache-slots.py fixture, then forces fresh local build
states with identical declared inputs. This bounded seed-compiler scenario is
not representative-chain, source-built, fixed-point, or release acceptance.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import time

# Full selected compiler/runtime packages are ingested afresh in each case.
COMMAND_ELAPSED_SECS_MAX = 900


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def read_json(path):
    return json.loads(path.read_text())


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def store_root(path):
    selected = path.resolve()
    for candidate in (selected, *selected.parents):
        if candidate.parent == Path("/nix/store"):
            return candidate
    raise RuntimeError(f"selected runtime is outside the explicit Nix seed store: {path}")


class Scenario:
    def __init__(self, arguments):
        self.args = arguments
        self.work = arguments.work_dir.absolute()
        require(not self.work.exists() or not any(self.work.iterdir()), "work directory must be absent or empty")
        require(len(os.fsencode(str(self.work / "daemon.sock"))) < 104, "work directory exceeds Unix socket bound")
        self.work.mkdir(parents=True, exist_ok=True)
        self.fixture = arguments.fixture_dir.resolve()
        previous = read_json(self.fixture / "evidence.json")
        require(previous["status"] == "passed", "native driver fixture did not pass")
        require(previous["protected_inventory"]["status"] == "passed", "protected inventory fixture did not pass")
        self.record = {"schema": "mantle-cc-build-shell-observation-v1", "status": "running",
                       "claim_class": "bounded-production-build-shell-observation", "proof_evidence": False,
                       "fixture": str(self.fixture), "commands": [], "cases": [],
                       "command_elapsed_secs_max": COMMAND_ELAPSED_SECS_MAX,
                       "non_claims": ["not-representative-bootstrap-chain", "not-source-built-provenance",
                                      "not-fixed-point-proof", "not-release-acceptance", "not-performance-proof"]}
        self.daemon = None
        self.daemon_streams = []
        self.environment = dict(os.environ)
        self.environment["CRUNCH_NO_FUSE"] = "1"
        require(not any(key.startswith(("MANTLE_CC_CACHE_", "MANTLE_COMPILE_")) for key in self.environment),
                "native harness must not inherit compiler runtime authority")
        for name in ("mantle", "driver", "daemon", "protected_exec_runner", "sandbox_shell"):
            path = getattr(arguments, name)
            require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK), f"invalid explicit {name}: {path}")
        self.record["executables_blake3"] = {
            name: self.digest(getattr(arguments, name))
            for name in ("mantle", "driver", "daemon", "protected_exec_runner", "sandbox_shell")}

    def save(self):
        write_json(self.work / "evidence.json", self.record)

    def digest(self, path):
        return subprocess.check_output(["b3sum", "--no-names", str(path)], text=True).strip()

    def command(self, label, argv, expected=0):
        argv = [str(item) for item in argv]
        started = time.monotonic_ns()
        result = subprocess.run(argv, cwd=self.work, env=self.environment, capture_output=True,
                                timeout=COMMAND_ELAPSED_SECS_MAX)
        stdout = self.work / f"{label}.stdout"
        stderr = self.work / f"{label}.stderr"
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        self.record["commands"].append({"label": label, "argv": argv, "exit_status": result.returncode,
                                        "expected_exit_status": expected, "elapsed_ns": time.monotonic_ns() - started,
                                        "stdout": str(stdout), "stderr": str(stderr),
                                        "stdout_blake3": self.digest(stdout), "stderr_blake3": self.digest(stderr)})
        self.save()
        require(result.returncode == expected, f"{label}: expected exit {expected}, observed {result.returncode}; see {stderr}")
        return result

    def prepare_inputs(self):
        self.inventory = read_json(self.fixture / "protected.inventory.json")
        entries = {entry["id"]: entry for entry in self.inventory["executable_entries"]}
        require(entries["cc-driver"]["digest"]["hex"] == self.digest(self.args.driver), "fixture driver bytes differ")
        require(entries["sandbox-shell"]["digest"]["hex"] == self.digest(self.args.sandbox_shell), "fixture shell bytes differ")
        self.environment["SNIX_BUILD_BWRAP"] = entries["sandbox-entry"]["executable_path"]
        self.environment["SNIX_BUILD_SANDBOX_SHELL"] = str(self.args.sandbox_shell)
        self.record["sandbox_runtime"] = {key: self.environment[key] for key in
                                          ("SNIX_BUILD_BWRAP", "SNIX_BUILD_SANDBOX_SHELL", "CRUNCH_NO_FUSE")}
        # Match bootstrap's truncated BLAKE3/Nix-base32 seed naming. This names a
        # declared fixture seed, not a claim that its path authenticates bytes.
        number = int.from_bytes(bytes.fromhex(self.digest(self.args.driver))[:20], "little")
        alphabet = "0123456789abcdfghijklmnpqrsvwxyz"
        token = "".join(alphabet[(number >> bit) & 31] for bit in range(155, -1, -5))
        self.seed_path = Path("/nix/store") / (token + "-cc-bootstrap-runtime")
        self.inputs = self.work / "input-store"
        self.inputs.mkdir()
        seed = self.inputs / self.seed_path.name
        for member in ("bin", "source", "tools"):
            (seed / member).mkdir(parents=True)
        shutil.copy2(self.args.driver, seed / "bin/mantle-cc-driver")
        for name in ("a.c", "a.h"):
            shutil.copy2(self.fixture / "source" / name, seed / "source" / name)
        shutil.copy2(self.fixture / "tools/as", seed / "tools/as")
        self.policy = read_json(self.fixture / "protected.policy.json")
        self.policy.update(cache_enabled=True, slots_enabled=True, probe_script=None, receipt_dir="/build/cc-receipts")
        for root in self.policy["roots"]:
            path = Path(root["path"])
            if path.is_relative_to(self.fixture):
                root["path"] = str(self.seed_path / path.relative_to(self.fixture))
        self.policy["environment"]["PATH"] = str(self.seed_path / "tools")
        write_json(seed / "policy.json", self.policy)
        for entry in self.inventory["executable_entries"]:
            path = Path(entry["executable_path"])
            if entry["id"] == "cc-driver":
                entry["executable_path"] = str(self.seed_path / "bin/mantle-cc-driver")
            elif path.is_relative_to(self.fixture):
                entry["executable_path"] = str(self.seed_path / path.relative_to(self.fixture))
        runtime = self.command("driver-runtime-closure", ["ldd", self.args.driver]).stdout.decode()
        require("not found" not in runtime, "driver runtime closure has missing libraries")
        files = [Path(root["path"]) for root in self.policy["roots"] if not root["path"].startswith(str(self.seed_path))]
        files.extend(Path(found) for found in re.findall(r"(?:=>\s*)?(/[^\s()]+)", runtime))
        roots = sorted({store_root(path) for path in files})
        for root in roots:
            shutil.copytree(root, self.inputs / root.name, symlinks=True)
        self.declared_inputs = [str(path) for path in roots] + [str(self.seed_path)]
        self.record.update(declared_inputs=self.declared_inputs, seed_driver_blake3=entries["cc-driver"]["digest"]["hex"],
                           fixture_evidence_blake3=self.digest(self.fixture / "evidence.json"))
        self.write_recipe()

    def write_recipe(self):
        script = f'''set -eu
{json.dumps(str(self.args.sandbox_shell))} mkdir -p "$out" /build/cc-receipts
compile() {{
  {json.dumps(str(self.args.sandbox_shell))} env -i PATH={self.seed_path}/tools LC_ALL=C TZ=UTC "$@" \
    {self.seed_path}/bin/mantle-cc-driver --policy {self.seed_path}/policy.json -- \\
    -x c -c {self.seed_path}/source/a.c -o "$out/a.o" -O2 -g0 -nostdinc \\
    -DOPTION_VALUE=3 -ffile-prefix-map={self.seed_path}/source=src
}}
if [ -n "${{MANTLE_CC_CACHE_SOCKET-}}" ]; then
  compile "MANTLE_CC_CACHE_SOCKET=$MANTLE_CC_CACHE_SOCKET" "MANTLE_COMPILE_BUILD_ID=$MANTLE_COMPILE_BUILD_ID"
else
  compile
fi
for receipt in /build/cc-receipts/*; do
  printf 'MANTLE_CC_RECEIPT_BEGIN\\n'
  {json.dumps(str(self.args.sandbox_shell))} cat "$receipt"
  printf '\\nMANTLE_CC_RECEIPT_END\\n'
done
'''
        self.recipe = self.work / "bootstrap-build-shell.ncl"
        self.recipe.write_text('let mantle = import "lib.ncl" in\n{\n'
                               f' name = "cc-protected-bootstrap-shell", builder = {json.dumps(str(self.args.sandbox_shell))},\n'
                               ' addressing_mode = \'input-addressed,\n'
                               f' args = ["sh", "-c", {json.dumps(script)}],\n'
                               f' inputs = {json.dumps(self.declared_inputs)},\n'
                               '} | mantle.Derivation\n')
        self.record["recipe_blake3"] = self.digest(self.recipe)
        self.record["driver_policy_blake3"] = self.digest(self.inputs / self.seed_path.name / "policy.json")
        self.save()

    def start_daemon(self):
        policy = read_json(self.fixture / "daemon-policy.json")
        policy.update(policy_id="cc-production-build-shell", socket_path=str(self.work / "daemon.sock"),
                      readable_roots=[str(self.work), *self.declared_inputs], writable_roots=[str(self.work)])
        write_json(self.work / "daemon-policy.json", policy)
        shutil.copy2(self.fixture / "compile-slot-policy.json", self.work / "compile-slot-policy.json")
        argv = [str(self.args.daemon), "--policy", str(self.work / "daemon-policy.json"),
                "--state-dir", str(self.work / "daemon-state"), "--store-backend", "snix",
                "--store-output-dir", str(self.work / "daemon-store"), "--receipt-dir", str(self.work / "daemon-receipts")]
        self.daemon_streams = [(self.work / "daemon.stdout").open("wb"), (self.work / "daemon.stderr").open("wb")]
        self.daemon = subprocess.Popen(argv, env=self.environment, stdout=self.daemon_streams[0], stderr=self.daemon_streams[1])
        self.record["daemon"] = {"argv": argv, "pid": self.daemon.pid}
        for _ in range(100):
            require(self.daemon.poll() is None, "daemon exited during startup; see daemon.stderr")
            if (self.work / "daemon.sock").exists():
                require(stat.S_ISSOCK((self.work / "daemon.sock").stat().st_mode), "daemon endpoint is not a socket")
                return
            time.sleep(0.1)
        raise RuntimeError("daemon did not create its endpoint within ten seconds")

    def build(self, label, mode="protected", inventory=None, expected=0):
        case = self.work / label
        case.mkdir()
        physical_store = case / "store"
        shutil.copytree(self.inputs, physical_store, symlinks=True, copy_function=os.link)
        argv = [self.args.mantle, "--store", physical_store, "--state-dir", case / "state",
                "--store-prefix", "/nix/store", "--json", "build", self.recipe, "--no-substitute"]
        selected_inventory = inventory if inventory is not None else self.inventory
        if mode != "plain":
            write_json(case / "inventory.json", selected_inventory)
            runtime = {"schema": "mantle-compile-driver-runtime-v1", "host_socket": str(self.work / "daemon.sock"),
                       "build_id": "production-build-shell", "admitted_builders": [str(self.args.sandbox_shell)],
                       "receipt_path": str(case / "mapping.ndjson"),
                       "protected_exec": {"runner": str(self.args.protected_exec_runner),
                                          "inventory": str(case / "inventory.json"), "audit_dir": str(case / "audits")}}
            if mode == "absent":
                runtime["host_socket"] = str(self.work / "absent.sock")
            if mode == "output-audit":
                runtime["protected_exec"]["audit_dir"] = str(physical_store / "runtime-audits")
            write_json(case / "runtime.json", runtime)
            argv.extend(["--compile-driver-runtime", case / "runtime.json"])
        result = self.command(label, argv, expected)
        if mode == "output-audit":
            require(b"compile-driver-audit-is-build-output" in result.stdout + result.stderr,
                    "physical output audit was not explicitly refused")
            require(not (case / "mapping.ndjson").exists() and not (physical_store / "runtime-audits").exists(),
                    "output-overlapping runtime wrote observation state")
            row = {"label": label, "mode": mode, "admission": "refused-before-runtime-files"}
            self.record["cases"].append(row)
            self.save()
            return row
        report = json.loads(result.stdout)
        row = {"label": label, "mode": mode, "report": str(self.work / f"{label}.stdout"),
               "recipe_blake3": self.digest(self.recipe), "audits": []}
        self.record["cases"].append(row)
        if (case / "audits").exists():
            for path in sorted((case / "audits").iterdir()):
                audit = read_json(path)
                require(audit["schema"] == "mantle-protected-build-observation-v1" and not audit["proof_evidence"],
                        f"{label}: invalid protected audit")
                row["audits"].append({"path": str(path), "observed": audit})
        if expected:
            require(report["counts"]["failed_total"] == 1, f"{label}: expected one failed goal")
            self.save()
            return row
        require(report["counts"]["built_total"] == 1 and report["counts"]["cached_total"] == 0,
                f"{label}: must execute a fresh sandbox, not reuse a derivation result")
        outcome, = report["outcomes"]
        output, = outcome["outputs"]
        object_path = physical_store / Path(output["path"]).name / "a.o"
        log = Path(outcome["log_file"]).read_text()
        captured = re.findall(r"MANTLE_CC_RECEIPT_BEGIN\n(.*?)\nMANTLE_CC_RECEIPT_END", log, re.S)
        require(len(captured) == 1, f"{label}: missing exact driver disposition in real build log")
        receipt = json.loads(captured[0])
        row.update(drv_key=outcome["drv_key"], output_path=output["artifact_attestation"]["logical_path"],
                   exported_path=output["path"], object_blake3=self.digest(object_path),
                   object_mode=stat.S_IMODE(object_path.stat().st_mode), receipt=receipt, log_file=outcome["log_file"])
        if mode != "plain":
            self.check_audit(label, row, selected_inventory)
        self.save()
        return row

    def check_audit(self, label, row, inventory):
        require(len(row["audits"]) == 1, f"{label}: missing sandbox protection audit")
        audit = row["audits"][0]["observed"]
        require(audit["exit_code"] == 0 and audit["launch_error"] is None and audit["audit_error"] is None,
                f"{label}: protected command did not finish successfully")
        entries = {entry["id"]: entry for entry in inventory["executable_entries"]}
        aliases = {alias["inventory_entry_id"]: alias for alias in audit["materialized_aliases"]}
        require("cc-driver" in aliases and "c-compiler" in aliases, f"{label}: selected tools were not materialized")
        require(audit["source_inventory_digest_blake3"] != audit["inventory_digest_blake3"],
                f"{label}: materialized inventory identity did not change")
        for event in audit["events"]:
            require(event["phase"] == "protected" and event["policy_decision"] == "allowed", f"{label}: denied execution")
            entry = entries[event["inventory_entry_id"]]
            allowed_paths = {entry["executable_path"]}
            if entry["id"] in aliases:
                allowed_paths.add(aliases[entry["id"]]["materialized_path"])
            require(event["executable_path"] in allowed_paths and event["digest_hex"] == entry["digest"]["hex"],
                    f"{label}: allowed event lacks an exact declared or verified materialized path and bytes")
        ids = {event["inventory_entry_id"] for event in audit["events"]}
        require({"sandbox-entry", "sandbox-shell", "cc-driver"} <= ids, f"{label}: missing real launcher/driver execution")
        row["executed_inventory_ids"] = sorted(ids)

    def exercise(self):
        self.prepare_inputs()
        self.start_daemon()
        plain = self.build("plain-no-runtime", "plain")
        cold = self.build("protected-cold")
        warm = self.build("protected-warm")
        absent = self.build("protected-absent-daemon", "absent")
        require(cold["receipt"]["cache_disposition"] == "cache-miss", "cold production build did not populate the cache")
        require(warm["receipt"]["cache_disposition"] == "object-hit", "warm production build did not replay an object")
        for row in (plain, absent):
            require(row["receipt"]["cache_disposition"] == "daemon-unavailable", f"{row['label']}: missing daemon changed fallback")
        require("c-compiler" in cold["executed_inventory_ids"] and "c-compiler" in absent["executed_inventory_ids"],
                "cold and absent-daemon cases must execute the declared compiler")
        compiler_ids = {entry["id"] for entry in self.inventory["executable_entries"] if entry["role"] == "bootstrap-toolchain-tool"}
        require(not compiler_ids.intersection(warm["executed_inventory_ids"]), "object hit executed a compiler/helper")
        for row in (cold, warm, absent):
            require((row["drv_key"], row["output_path"], row["object_blake3"], row["object_mode"], row["recipe_blake3"]) ==
                    (plain["drv_key"], plain["output_path"], plain["object_blake3"], plain["object_mode"], plain["recipe_blake3"]),
                    f"{row['label']}: runtime policy changed derivation/output identity, bytes or permissions")
        removed = copy.deepcopy(self.inventory)
        removed["executable_entries"] = [entry for entry in removed["executable_entries"] if entry["id"] != "cc-driver"]
        denied = self.build("undeclared-materialized-driver", "absent", removed, expected=1)
        require(any(event["policy_decision"] == "denied" for audit in denied["audits"] for event in audit["observed"]["events"]),
                "undeclared materialized driver was not explicitly denied")
        altered = copy.deepcopy(self.inventory)
        compiler = next(entry for entry in altered["executable_entries"] if entry["id"] == "c-compiler")
        compiler["digest"]["hex"] = "0" * 64 if compiler["digest"]["hex"] != "0" * 64 else "1" * 64
        rejected = self.build("wrong-materialized-compiler-digest", inventory=altered, expected=1)
        require(not rejected["audits"], "wrong materialized bytes must fail before audit creation and cache lookup")
        diagnostic = "\n".join(path.read_text() for path in
                               (self.work / "wrong-materialized-compiler-digest/state/logs").glob("*.log"))
        require("materialized executable digest mismatch" in diagnostic,
                "wrong compiler digest did not produce the materialization refusal")
        self.build("physical-output-audit-refused", "output-audit", expected=1)
        require(self.record["executables_blake3"] == {name: self.digest(getattr(self.args, name))
                for name in self.record["executables_blake3"]}, "selected binaries changed during the native scenario")
        self.record["status"] = "passed"

    def close(self):
        if self.daemon is not None:
            self.daemon.terminate()
            try:
                self.daemon.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.daemon.kill()
                self.daemon.wait(timeout=10)
        for stream in self.daemon_streams:
            stream.close()
        self.save()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("fixture-dir", "mantle", "driver", "daemon", "protected-exec-runner", "sandbox-shell", "work-dir"):
        parser.add_argument("--" + name, required=True, type=Path)
    scenario = Scenario(parser.parse_args())
    try:
        scenario.exercise()
    except BaseException as error:
        scenario.record.update(status="failed", error=str(error))
        raise
    finally:
        scenario.close()
    print(scenario.work / "evidence.json")


if __name__ == "__main__":
    main()
