#!/usr/bin/env python3
"""Exercise issue #3 using admitted immutable tools, never an ambient Cargo cache.

Run AFTER root CLI integration. --acquisition-inputs contains exact signed
CargoAcquisitionInputs roots already admitted into --state-dir/--store through
existing fetch/producer/selected-table recipes. --tools contains signed roots:
seed_toolchain, musl, unit_producer (executable), planner (executable), linker
(executable), helper (executable), runtime (roots), host_triple, target_triple.
The target is x86_64-unknown-linux-musl for both lanes. host_triple must match
the observed rustc host; cross-target tools additionally declare target_linker.
No compiler/tool download or Nix evaluation occurs in this driver.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

REPO = Path(__file__).resolve().parent.parent
FIXTURE = REPO / "fixtures/lock-vendor-two-crate"
EXPECTED = "offline two-crate consumer: verified\n"
PREFIX = "/nix/store"


def require(ok, detail):
    if not ok:
        raise RuntimeError(detail)


def q(value):
    return json.dumps(str(value))


def logical(path):
    return PREFIX + "/" + Path(path).name


def source_root(path):
    require(str(path).startswith(PREFIX + "/"), f"not an immutable store input: {path}")
    return "/".join(str(path).split("/")[:4])


class Proof:
    def __init__(self, args):
        self.a = args
        require(not args.scratch.exists(), "scratch must be absent; no old evidence reuse")
        args.scratch.mkdir(parents=True)
        self.workspace = args.scratch / "workspace"
        shutil.copytree(FIXTURE, self.workspace, ignore=shutil.ignore_patterns("vendor-deps", "vendor", "target"))
        self.home = args.scratch / "empty-home"
        self.home.mkdir()
        self.environment = {**os.environ, "HOME": str(self.home), "CARGO_HOME": str(self.home / "cargo"),
                            "CARGO_NET_OFFLINE": "true", "TMPDIR": str(args.scratch)}
        self.inputs = json.loads(args.acquisition_inputs.read_text())
        self.tools = json.loads(args.tools.read_text())
        require("compiler" in self.tools, "tools manifest needs an independently admitted compiler-only rustc path")
        compiler_root = source_root(self.tools["compiler"])
        require(compiler_root != self.inputs["rust"], "execution compiler must not share the Cargo/oracle store root")
        self.tools["runtime"] = sorted({compiler_root, *self.tools["runtime"]})
        require(self.tools["target_triple"] == "x86_64-unknown-linux-musl",
                "matched proof needs a real musl Rust target; no gnu substitution")
        require(self.tools["host_triple"] == self.tools["target_triple"] or self.tools.get("target_linker"),
                "cross-target unit compilation requires a declared target_linker")
        self.records = []

    def run(self, label, command, *, cwd=None, denied=False, negative=False):
        command = [str(item) for item in command]
        if denied:
            command = [self.a.bwrap, "--unshare-net", "--bind", "/", "/", "--proc", "/proc", "--dev-bind", "/dev", "/dev", "--", *command]
        result = subprocess.run(command, cwd=cwd or REPO, env=self.environment,
                                capture_output=True, text=True)
        (self.a.scratch / f"{label}.stdout").write_text(result.stdout)
        (self.a.scratch / f"{label}.stderr").write_text(result.stderr)
        self.records.append({"label": label, "argv": command, "network_denied": denied, "exit_code": result.returncode})
        require(result.returncode != 0 if negative else result.returncode == 0,
                f"{label}: exit {result.returncode}; {result.stderr[-4000:]}")
        return result

    def cli(self, label, command, *, state=None, store=None, cwd=None, denied=False, negative=False):
        command = [self.a.cli, "--json", "--nix-compat", "--state-dir", state or self.a.state_dir,
                   "--store", store or self.a.store, *command]
        return self.run(label, command, cwd=cwd, denied=denied, negative=negative)

    def build(self, label, file, *, state=None, store=None, denied=True, negative=False):
        result = self.cli(label, ["build", "--strict-hermetic", "--no-substitute", "--signing-key", self.a.signing_key,
                                 "--trusted-public-keys", self.a.trusted_key, "-I", REPO / "lib", file],
                          state=state, store=store, denied=denied, negative=negative)
        report = json.loads(result.stdout)
        require(not report.get("hermeticity_audit_events"), f"{label}: fallback audit")
        if not negative:
            require(not report.get("failed"), f"{label}: failed roots")
        return report

    def recipe(self, label, text):
        file = self.a.scratch / f"{label}.ncl"
        file.write_text(text)
        return file

    def output(self, report, label, output="out"):
        paths = [item["path"] for row in report["outcomes"] if row["label"] == label
                 for item in row["outputs"] if item["name"] == output]
        require(len(paths) == 1, f"expected one {label}/{output}: {paths}")
        return logical(paths[0])

    def negative_planning(self):
        table = FIXTURE / "shared-lock-hashes-v1"
        original = (self.workspace / "Cargo.lock").read_text()
        cases = {
            "missing-hash": "\n".join(line for line in original.splitlines() if not line.startswith("checksum =")) + "\n",
            "unreviewed-git": 'version = 4\n[[package]]\nname = "unreviewed"\nversion = "1.0.0"\nsource = "git+https://example.org/repo#1111111111111111111111111111111111111111"\n',
        }
        for label, lock in cases.items():
            (self.workspace / "Cargo.lock").write_text(lock)
            result = self.cli(label, ["import", "cargo", "--acquisition-plan", table],
                              cwd=self.workspace, denied=True, negative=True)
            blocker = json.loads(result.stdout or result.stderr)
            require(blocker["stage"] == "acquisition-planning" and blocker["subject"] in ("crc64", "unreviewed"),
                    f"wrong planning boundary: {blocker}")
        (self.workspace / "Cargo.lock").write_text(original)

    def acquire(self, label, *, state=None, store=None, denied=False):
        report = self.a.scratch / f"{label}-acquisition.json"
        result = self.cli(label, ["import", "cargo", "--plan", "--acquire", self.a.acquisition_inputs,
                                 "--acquisition-report", report, "--strict-hermetic", "--signing-key", self.a.signing_key,
                                 "--trusted-public-keys", self.a.trusted_key],
                          cwd=self.workspace, state=state, store=store, denied=denied)
        imported = json.loads(result.stdout)
        acquisition = json.loads(report.read_text())
        require(acquisition["stage"] == "acquisition-realized" and not imported["blockers"], "acquisition/import failed")
        vendor = acquisition["output"]["logical_store_path"]
        require(imported["vendor_source"]["root"] == vendor, "ordinary import did not bind the same verified vendor root")
        return acquisition, imported

    def require_failure_log(self, report, expected):
        logs = [Path(row["saved_log_path"]) for row in report.get("failed", []) if row.get("saved_log_path")]
        require(any(path.is_file() and expected in path.read_text(errors="replace") for path in logs),
                f"failure did not reach its own boundary: {expected}")

    def acquisition_negatives(self, acquisition):
        artifact = acquisition["plan_output"]["logical_store_path"]
        python = self.inputs["python"] + "/bin/python3"
        # This is a separate NEGATIVE ACQUISITION, not offline planning or
        # compilation: the declared fetch may obtain its original HTTPS URL,
        # but it must reject bytes against the intentionally changed lock hash.
        code = "import json,os,sys; p=json.load(open(sys.argv[1])); p['units'][0]['derivation']['fixed_output']['hash']='0'*64; json.dump(p,open(os.environ['plan'],'w'),separators=(',',':'))"
        forged = self.recipe("changed-lock-hash", f'''let mantle = import "lib.ncl" in
let producer = {{ name = "changed-lock-hash-producer", builder = {q(python)}, system = 'x86_64-linux,
 args = ["-c", {q(code)}, {q(artifact)}], outputs = ["plan"], dynamic_plan_outputs = ["plan"],
 inputs = [{q(artifact)}, {q(self.inputs['python'])}], addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation in {{ name = "changed-lock-hash-consumer", builder = {q(python)}, system = 'x86_64-linux,
 args = ["-c", "raise RuntimeError('a bad fetch must never reach the consumer')"],
 inputs = [{{ name = "vendor", producer = producer, plan_output = "plan", root = "vendor.complete", unit_output = "out" }}, {q(self.inputs['python'])}],
 env = {{ VENDOR = "{{{{mantle-plan-output:vendor}}}}" }}, addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation''')
        failed = self.build("changed-lock-hash", forged, denied=False, negative=True)
        self.require_failure_log(failed, "FOD hash mismatch for lock-fetch-")
        plan = json.loads((self.a.store / Path(artifact).name).read_text())
        fetches = {}
        for unit in plan["units"]:
            if not unit["id"].startswith("fetch."):
                continue
            name = unit["derivation"]["name"]
            matches = [path for path in self.a.store.glob("*-" + name) if path.is_file()]
            require(len(matches) == 1 and hashlib.sha256(matches[0].read_bytes()).hexdigest() == unit["derivation"]["fixed_output"]["hash"],
                    f"no exact declared signed fetch output for {name}")
            fetches[unit["id"]] = logical(matches[0])
        final = next(unit["derivation"] for unit in plan["units"] if unit["id"] == "vendor.complete")
        replacements = {"{{mantle-source:src.bundle}}": self.inputs["bundle"],
                        "{{mantle-source:src.table}}": self.inputs["selected_table"],
                        "{{mantle-source:src.rust}}": self.inputs["rust"]}
        replacements.update({"{{mantle-unit-output:" + unit + ":out}}": path for unit, path in fetches.items()})
        args = final["args"].copy()
        for position, value in enumerate(args):
            for marker, replacement in replacements.items():
                value = value.replace(marker, replacement)
            args[position] = value
        paths = sorted(fetches.values())
        require(len(paths) == 2, "negative fixture requires the two pinned archives")
        target = next(index for index, value in enumerate(args) if value in paths)
        for label, replacement in (("missing-artifact", PREFIX + "/" + "0" * 32 + "-missing.crate"),
                                   ("corrupt-archive", next(path for path in paths if path != args[target]))):
            broken = args.copy()
            broken[target] = replacement
            roots = [self.inputs[key] for key in ("bundle", "selected_table", "python", "rust")] + paths + self.inputs["runtime"]
            file = self.recipe(label, f'''let mantle = import "lib.ncl" in {{
 name = {q(label)}, builder = {q(python)}, system = 'x86_64-linux,
 args = {json.dumps(broken)}, inputs = {json.dumps(roots)},
 addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation''')
            failed = self.build(label, file, negative=True)
            require(not any(row["label"] == label for row in failed["outcomes"]), "partial assembly was published")
            self.require_failure_log(failed, "archive mismatch")

    def run_proof(self):
        self.negative_planning()
        inspected = self.cli("inspection", ["import", "cargo", "--acquisition-plan", FIXTURE / "shared-lock-hashes-v1"],
                             cwd=self.workspace, denied=True)
        inspection = json.loads(inspected.stdout)
        require({(row["name"], row["version"], row["sha256"]) for row in inspection["artifacts"]} == {
            ("crc64", "2.0.0", "2707e3afba5e19b75d582d88bc79237418f2a2a2d673d01cf9b03633b46e98f3"),
            ("memchr", "2.8.0", "f8ca58f447f06ed17d5fc4043ce1b10dd205e060fb3ce5b979b8ed8e59ff3f79")}, "unpinned fixture closure")
        require(not (self.workspace / "vendor-deps").exists() and not (self.workspace / "vendor").exists(),
                "prepopulated vendor fixture forbidden")
        acquisition, imported = self.acquire("acquire")
        vendor = acquisition["output"]["logical_store_path"]
        self.acquisition_negatives(acquisition)
        # All remaining commands are in an outer network namespace, including
        # the scheduler/fetch service itself, not just each compiler sandbox.
        self.run("network-denial", [sys.executable, "-c", "import socket; s=socket.socket(); s.settimeout(1); s.connect(('1.1.1.1',443))"], denied=True, negative=True)
        python = self.inputs["python"] + "/bin/python3"
        bundle = self.inputs["bundle"]
        observe_code = "import os,pathlib,subprocess,sys; rust=sys.argv[1]; assert pathlib.Path(rust+'/lib/rustlib/x86_64-unknown-linux-musl/lib').is_dir(); v=subprocess.run([rust+'/bin/rustc','-vV'],env={'HOME':'/nonexistent','PATH':rust+'/bin'},capture_output=True,text=True,check=True).stdout; pathlib.Path(os.environ['out']).write_text(v)"
        observed_file = self.recipe("toolchain-observation", f'''let mantle = import "lib.ncl" in {{
 name = "cargo-acquisition-toolchain-observation", builder = {q(python)}, system = 'x86_64-linux,
 args = ["-c", {q(observe_code)}, {q(self.inputs['rust'])}],
 inputs = {json.dumps([self.inputs['python'], self.inputs['rust'], *self.inputs['runtime']])},
 addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation''')
        observation = self.output(self.build("toolchain-observation", observed_file), "cargo-acquisition-toolchain-observation")
        observed_host = (self.a.store / Path(observation).name).read_text().split("host: ", 1)[1].splitlines()[0]
        require(observed_host == self.tools["host_triple"], "declared host triple differs from signed toolchain observation")
        source_file = self.recipe("package-source", f'''let mantle = import "lib.ncl" in {{
 name = "lock-vendor-two-crate-src", builder = {q(python)}, system = 'x86_64-linux,
 args = ["-c", "import os,shutil,sys; shutil.copytree(sys.argv[1]+'/workspace',os.environ['out'])", {q(bundle)}],
 inputs = [{q(self.inputs['python'])}, {q(bundle)}], addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation''')
        source = self.output(self.build("package-source", source_file), "lock-vendor-two-crate-src")
        # Apply the already inspected ordinary import file operations exactly.
        for operation in imported["file_operations"]:
            target = self.workspace / operation["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(operation["content"])
        pins = {"lock-vendor-two-crate_src": source, "rust": self.inputs["rust"],
                "musl_seed_toolchain": self.tools["seed_toolchain"], "musl": self.tools["musl"], "vendor_src": vendor}
        (self.workspace / ".mantle/inputs.ncl").write_text("{\n" + "".join(f" {q(key)} = {q(value)},\n" for key, value in pins.items()) + "}\n")
        ordinary = self.recipe("ordinary", f'(import {q(self.workspace / "mantle-project.ncl")}).packages."lock-vendor-two-crate"\n')
        ordinary_report = self.build("ordinary-build", ordinary)
        ordinary_output = self.output(ordinary_report, "lock-vendor-two-crate")
        ordinary_physical = self.a.store / Path(ordinary_output).name
        executed = self.run("ordinary-run", [ordinary_physical / "bin/lock-vendor-two-crate"], denied=True)
        require(executed.stdout == EXPECTED, "ordinary lane fixture assertions/output differ")
        evidence = json.loads((ordinary_physical / "share/mantle/offline-cargo-build.json").read_text())
        require(any(row["role"] == "vendored-dependencies" and row["path"] == vendor for row in evidence["source_closure"]),
                "ordinary build evidence lost vendor identity")
        config = {"workspace": source, "vendor": vendor, "planner": self.tools["planner"],
                  "cargo": self.inputs["rust"] + "/bin/cargo", "rustc": self.tools["compiler"],
                  "linker": self.tools["linker"], "toolchain": self.inputs["rust"], "helper": self.tools["helper"],
                  "runtime_inputs": self.tools["runtime"], "store_prefix": PREFIX, "system": "x86_64-linux",
                  "host_triple": self.tools["host_triple"], "target_triple": self.tools["target_triple"], "profile": "release"}
        target_linker = self.tools.get("target_linker")
        target_field = ""
        target_roots = []
        if target_linker:
            target_linker = {"executable": target_linker["executable"], "arguments": target_linker.get("arguments", []), "inputs": target_linker.get("inputs", [])}
            config["target_linker"] = target_linker
            target_roots = [source_root(target_linker["executable"]), *target_linker["inputs"]]
            target_field = "target_linker = { executable = " + q(target_linker["executable"]) + ", arguments = " + json.dumps(target_linker["arguments"]) + ", inputs = " + json.dumps(target_linker["inputs"]) + " },"
        config_bytes = json.dumps(config, sort_keys=True, separators=(",", ":"))
        config_inputs = sorted({source, vendor, self.inputs["rust"], self.inputs["python"], *self.tools["runtime"],
                                *target_roots, *(source_root(self.tools[key]) for key in ("planner", "linker", "helper"))})
        config_file = self.recipe("unit-config", f'''let mantle = import "lib.ncl" in {{
 name = "cargo-acquisition-unit-config", builder = {q(python)}, system = 'x86_64-linux,
 args = ["-c", "import os,pathlib,sys; pathlib.Path(os.environ['out']).write_text(sys.argv[1])", {q(config_bytes)}],
 inputs = {json.dumps(config_inputs)}, addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation''')
        config_path = self.output(self.build("unit-config", config_file), "cargo-acquisition-unit-config")
        unit_expr = f'''(import "cargo_unit_plan.ncl") {{ goal_name = "cargo-acquisition-unit-plan",
 producer = {q(self.tools['unit_producer'])}, config = {q(config_path)}, workspace = {q(source)},
 planner = {q(self.tools['planner'])}, toolchain = {q(self.inputs['rust'])}, linker = {q(self.tools['linker'])},
 helper = {q(self.tools['helper'])}, vendor_sources = [{q(vendor)}], runtime_inputs = {json.dumps(self.tools['runtime'])}, {target_field} }}'''
        producer_file = self.recipe("unit-producer", unit_expr)
        unit_report = self.build("unit-plan-and-compilation", producer_file)
        plan_path = self.output(unit_report, "cargo-acquisition-unit-plan", "plan")
        plan = json.loads((self.a.store / Path(plan_path).name).read_text())
        require(len(plan["roots"]) == 1 and re.fullmatch(r"u\.[0-9a-f]{64}", plan["roots"][0]), "ambiguous application root")
        root = plan["roots"][0]
        run_code = "import os,pathlib,subprocess; p=subprocess.run([os.environ['APP']+'/lib/lock_vendor_two_crate'],capture_output=True,text=True,check=True); assert p.stdout==" + repr(EXPECTED) + "; pathlib.Path(os.environ['out']).write_text(p.stdout)"
        consumer_file = self.recipe("unit-consumer", f'''let mantle = import "lib.ncl" in let producer = {unit_expr} in {{
 name = "cargo-acquisition-unit-run", builder = {q(python)}, system = 'x86_64-linux,
 args = ["-c", {q(run_code)}], env = {{ APP = "{{{{mantle-plan-output:app}}}}" }},
 inputs = [{q(self.inputs['python'])}, {{ name = "app", producer = producer, plan_output = "plan", root = {q(root)}, unit_output = "out" }}],
 addressing_mode = 'input-addressed, sandbox = 'native,
}} | mantle.Derivation''')
        run_report = self.build("unit-run", consumer_file)
        run_output = self.output(run_report, "cargo-acquisition-unit-run")
        require((self.a.store / Path(run_output).name).read_text() == EXPECTED, "unit lane fixture assertions/output differ")
        cache = self.a.scratch / "signed-cache"
        self.cli("push-signed", ["store", "push", "--to", cache, "--all"], denied=True)
        client_store, client_state = self.a.scratch / "client-store", self.a.scratch / "client-state"
        self.cli("pull-signed", ["store", "pull", "--from", cache, "--all", "--trusted-public-keys", self.a.trusted_key], state=client_state, store=client_store, denied=True)
        reused, _ = self.acquire("clean-client-acquire", state=client_state, store=client_store, denied=True)
        require(reused["output"] == acquisition["output"] and reused["inspection"] == acquisition["inspection"]
                and reused["canonical_plan_blake3"] == acquisition["canonical_plan_blake3"], "clean-client identity or signed policy drift")
        self.build("clean-client-ordinary", ordinary, state=client_state, store=client_store)
        self.build("clean-client-unit", consumer_file, state=client_state, store=client_store)
        signed_capture = ["source", "bundle", "import-store-path", "--logical", vendor,
                          "--physical", client_store / Path(vendor).name]
        untrusted = self.cli("untrusted-key", [*signed_capture, "--trusted-public-keys", self.a.wrong_key],
                             state=client_state, store=client_store, denied=True, negative=True)
        require("untrusted or unsigned" in untrusted.stdout + untrusted.stderr,
                "wrong key failed before the signed PathInfo trust boundary")
        # Corrupt only an isolated clean-client export, never publisher inputs.
        corrupt = next((client_store / Path(vendor).name).glob("*/src/lib.rs"))
        corrupt.chmod(corrupt.stat().st_mode | 0o200)
        corrupt.write_bytes(corrupt.read_bytes() + b"\n// corruption negative\n")
        result = self.cli("corrupt-signed-vendor", [*signed_capture, "--trusted-public-keys", self.a.trusted_key],
                          state=client_state, store=client_store, denied=True, negative=True)
        require("differ" in result.stdout + result.stderr or "mismatch" in result.stdout + result.stderr,
                "corruption did not fail signed physical output byte boundary")
        inputs_blake3 = self.run("inputs-identity", [self.a.b3sum, "--no-names", self.a.acquisition_inputs], denied=True).stdout.strip()
        tools_blake3 = self.run("tools-identity", [self.a.b3sum, "--no-names", self.a.tools], denied=True).stdout.strip()
        require(re.fullmatch(r"[0-9a-f]{64}", inputs_blake3) and re.fullmatch(r"[0-9a-f]{64}", tools_blake3), "invalid native proof manifest identity")
        receipt = {"schema": "mantle-cargo-acquisition-proof-v1", "acquisition": acquisition,
                   "same_verified_vendor": vendor, "ordinary_output": ordinary_output, "unit_plan_output": plan_path,
                   "ordinary_stdout": executed.stdout, "records": self.records,
                   "inputs_blake3": inputs_blake3,
                   "tools_blake3": tools_blake3,
                   "non_claims": ["not-full-workspace-acceptance", "not-Git-compilation-coverage", "not-release-parity"]}
        (self.a.scratch / "proof.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("cli", "scratch", "state-dir", "store", "acquisition-inputs", "tools", "signing-key"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--trusted-key", required=True)
    parser.add_argument("--wrong-key", required=True, help="valid public key that did not sign any admitted input")
    parser.add_argument("--bwrap", required=True, help="explicit pinned bwrap executable")
    parser.add_argument("--b3sum", required=True, help="explicit pinned BLAKE3 proof tool executable")
    args = parser.parse_args()
    for name in ("cli", "scratch", "state_dir", "store", "acquisition_inputs", "tools", "signing_key"):
        setattr(args, name, getattr(args, name).absolute())
    require(args.wrong_key != args.trusted_key, "negative trust key must differ")
    source_root(args.bwrap)
    source_root(args.b3sum)
    Proof(args).run_proof()


if __name__ == "__main__":
    main()
