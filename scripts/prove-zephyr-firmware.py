#!/usr/bin/env python3
"""Real offline Zephyr/Mantle proof. Requires already signed, admitted source/tool closures.

No compiler substitute, west update, pip, SDK setup or network acquisition occurs here.
Run detached on a spacious scratch filesystem after the integrated source is built.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys

ARTIFACTS = ("zephyr.elf", "zephyr.bin", "zephyr.hex", "zephyr.map", ".config", "zephyr.dts", "include/generated/zephyr/devicetree_generated.h")


def invoke(command, *, check=True):
    result = subprocess.run([str(x) for x in command], text=True, capture_output=True, check=False)
    if check and result.returncode:
        raise RuntimeError(f"exit {result.returncode}: {command!r}\n{result.stdout[-12000:]}\n{result.stderr[-12000:]}")
    return result


def write_json(path, value):
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def nickel_json(value):
    return "std.deserialize 'Json " + json.dumps(json.dumps(value, sort_keys=True))


class Proof:
    def __init__(self, args):
        self.args = args
        self.repo = Path(__file__).resolve().parent.parent
        self.root = args.run_dir.resolve()
        self.root.mkdir(parents=True, exist_ok=False)
        self.observations = []
        self.plan = json.loads(args.plan.read_text())
        for executable, expected in [(args.mantle, args.expected_mantle_blake3), (args.runner, args.expected_runner_blake3)]:
            observed = invoke([args.b3sum, executable]).stdout.split()[0]
            if len(expected) != 64 or observed != expected:
                raise RuntimeError(f"proof executable content mismatch: {executable}")
        if self.plan["resolved_west"] != {"role": self.plan["project"], "member": "resolved-west.json"}:
            raise RuntimeError("proof requires resolved-west.json in the fixture role")
        self.runner = {"role": args.runner_role, "member": "bin/mantle-zephyr", "version": "Mantle firmware builder"}
        # Exact checked-in fixture layout, rather than an arbitrary user's project.
        if self.plan["project_subdir"] != "app":
            raise RuntimeError("proof requires the complete tests/fixtures/zephyr root with project_subdir=app")
        self.state = self.root / "first-state"
        self.store = self.root / "first-store"

    def identity(self, plan, name, expected=None):
        path = self.root / f"{name}.plan.json"
        write_json(path, plan)
        result = invoke([self.args.runner, "identity", path], check=expected is None)
        write_json(self.root / f"{name}.identity.json", {"exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        if expected is not None:
            if result.returncode == 0 or expected not in result.stderr:
                raise RuntimeError(f"negative {name} did not reject with {expected}: {result.stderr}")
            self.observations.append({"case": name, "rejection": expected})
        return result.stdout.strip()

    def recipe(self, name, plan=None, derivation=None):
        path = self.root / f"{name}.ncl"
        if plan is not None:
            bindings = {"plan": plan, "runner": self.runner}
            code = f'let bindings = {nickel_json(bindings)} in (import {json.dumps(str(self.repo / "tests/fixtures/zephyr/firmware.ncl"))}) bindings\n'
        else:
            code = f'let d = {nickel_json(derivation)} in {{ name=d.name, builder=d.builder, args=d.args, system=\'x86_64-linux, outputs=["out"], addressing_mode=\'input-addressed, inputs=d.inputs, env=d.env }} | import {json.dumps(str(self.repo / "lib/derivation.ncl"))}\n'
        path.write_text(code)
        return path

    def build(self, name, recipe, *, state=None, store=None, expected=None):
        state = state or self.state
        store = store or self.store
        command = [self.args.mantle, "--json", "--state-dir", state, "--store", store, "--store-prefix", self.plan["store_prefix"]]
        for base in self.args.base_state:
            command += ["--base-store", base]
        command += ["build", recipe, "--no-substitute", "--signing-key", self.args.signing_key]
        for key in self.args.trusted_public_key:
            command += ["--trusted-public-keys", key]
        result = invoke(command, check=expected is None)
        write_json(self.root / f"{name}.command.json", {"command": [str(x) for x in command], "exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        if expected:
            logs = result.stdout + result.stderr
            log_dir = state / "logs"
            if log_dir.exists():
                for file in sorted(log_dir.rglob("*")):
                    if file.is_file() and file.stat().st_size <= 2 * 1024 * 1024:
                        logs += file.read_text(errors="replace")
            if result.returncode == 0 or not any(marker in logs for marker in expected):
                raise RuntimeError(f"negative {name} lacks expected rejection {expected}: {logs[-12000:]}")
            self.observations.append({"case": name, "rejection": expected})
            return None, None
        report = json.loads(result.stdout)
        if report["schema"] != "crunch-build-report-v1" or report["counts"]["failed_total"] != 0:
            raise RuntimeError(f"unexpected aggregate report in {name}")
        outputs = [item["path"] for outcome in report["outcomes"] for item in outcome["outputs"] if item["name"] == "out"]
        if len(outputs) != 1:
            raise RuntimeError(f"expected exactly one concrete root output, got {outputs}")
        path = store / Path(outputs[0]).name
        if not path.is_dir():
            raise RuntimeError(f"output is not physically exported: {path}")
        return path, report

    def firmware(self, name, plan, *, state=None, store=None, expected=None):
        return self.build(name, self.recipe(name, plan=plan), state=state, store=store, expected=expected)

    def mutate(self, name, role, operations):
        root = self.plan["roles"][role]["root"]
        python = self.plan["tools"]["python"]
        program = """import json,os,pathlib,shutil,sys
src,out,ops=sys.argv[1],os.environ['out'],json.loads(sys.argv[2])
shutil.copytree(src,out,symlinks=True)
os.chmod(out,0o755)
for op in ops:
 p=pathlib.Path(out)/op['path']
 if op['kind']=='replace':
  text=p.read_text()
  if op['old'] not in text: raise SystemExit('fixture mutation anchor absent: '+str(p))
  p.chmod(0o644);p.write_text(text.replace(op['old'],op['new']))
 elif op['kind']=='write':
  p.parent.chmod(0o755);p.write_text(op['text'])
 elif op['kind']=='symlink':
  p.parent.chmod(0o755);p.unlink();p.symlink_to(op['target'])
 else: raise SystemExit('unsupported mutation')
"""
        derivation = {"name": "zephyr-input-" + name, "builder": self.plan["roles"][python["role"]]["root"] + "/" + python["member"],
                      "args": ["-c", program, root, json.dumps(operations)], "inputs": [x["root"] for x in self.plan["roles"].values()],
                      "env": {"PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1", "LC_ALL": "C"}}
        physical, _ = self.build(name + "-input", self.recipe(name + "-input", derivation=derivation))
        changed = copy.deepcopy(self.plan)
        changed["name"] = "zephyr-" + name
        changed["roles"][role] = {"root": self.plan["store_prefix"] + "/" + physical.name,
                                  "tree_blake3": invoke([self.args.runner, "seal-tree", physical]).stdout.strip()}
        return changed

    def observe(self, name, path, plan):
        manifest = json.loads((path / "manifest.json").read_text())
        if manifest["schema"] != "mantle-zephyr-provenance-v1" or manifest["plan_identity_blake3"] != self.identity(plan, name + "-observed"):
            raise RuntimeError("firmware manifest does not bind the exact plan")
        if manifest["plan"] != plan or set(manifest["artifacts_blake3"]) != set(ARTIFACTS):
            raise RuntimeError("firmware manifest inputs or artifact set differ")
        for artifact in ARTIFACTS:
            # Compare actual payloads with the runner's BLAKE3 observation, not a name/version claim.
            result = invoke([self.args.b3sum, path / artifact]).stdout.split()[0]
            if result != manifest["artifacts_blake3"][artifact]:
                raise RuntimeError("artifact digest differs: " + artifact)
        elf = (path / "zephyr.elf").read_bytes()[:20]
        if elf[:6] != b"\x7fELF\x01\x01" or elf[16:20] != b"\x02\x00\x28\x00":
            raise RuntimeError("not an ARM ELF32 executable")
        if "CONFIG_MAIN_STACK_SIZE=1024" not in (path / ".config").read_text() and name == "first":
            raise RuntimeError("fixture Kconfig was not resolved")
        self.observations.append({"case": name, "plan_identity_blake3": manifest["plan_identity_blake3"], "artifacts_blake3": manifest["artifacts_blake3"]})
        return manifest

    def execute(self):
        baseline_identity = self.identity(self.plan, "baseline")
        first, report = self.firmware("first", self.plan)
        if report["counts"]["built_total"] != 1:
            raise RuntimeError("first execution was not a real compiler build; base state must contain inputs only")
        first_manifest = self.observe("first", first, self.plan)
        cached, report = self.firmware("cached", self.plan)
        if report["counts"]["cached_total"] != 1 or report["counts"]["built_total"] != 0 or cached != first:
            raise RuntimeError("unchanged signed cache admission did not retain the exact firmware root")
        self.observations.append({"case": "cached", "counts": report["counts"]})
        clean, report = self.firmware("clean", self.plan, state=self.root / "clean-state", store=self.root / "clean-store")
        if report["counts"]["built_total"] != 1:
            raise RuntimeError("clean comparison did not execute the compiler independently")
        clean_manifest = self.observe("clean", clean, self.plan)
        if clean_manifest["artifacts_blake3"] != first_manifest["artifacts_blake3"]:
            raise RuntimeError("clean generated/firmware artifact hashes differ")
        for artifact in ARTIFACTS:
            if (first / artifact).read_bytes() != (clean / artifact).read_bytes():
                raise RuntimeError("clean artifact byte mismatch: " + artifact)
        project = self.plan["project"]
        positive = [
            ("source", project, [{"kind": "replace", "path": "app/src/main.c", "old": "Mantle Zephyr fixture", "new": "Mantle Zephyr changed source"}]),
            ("configuration", project, [{"kind": "replace", "path": "app/prj.conf", "old": "CONFIG_MAIN_STACK_SIZE=1024", "new": "CONFIG_MAIN_STACK_SIZE=1536"}]),
            ("overlay", project, [{"kind": "replace", "path": "app/uart.overlay", "old": "115200", "new": "57600"}]),
            ("module-content", self.plan["west"]["modules"]["cmsis"]["role"], [{"kind": "write", "path": "mantle-input-variation", "text": "content-only identity control\n"}]),
            ("toolchain-content", self.plan["sdk"], [{"kind": "write", "path": "mantle-input-variation", "text": "content-only identity control\n"}]),
        ]
        identities = {baseline_identity}
        for name, role, operations in positive:
            changed = self.mutate(name, role, operations)
            # Keep the derivation name fixed: content/config changes, not label changes, must alter identity.
            changed["name"] = self.plan["name"]
            changed_identity = self.identity(changed, name)
            if changed_identity in identities:
                raise RuntimeError("changed input did not change exact plan identity: " + name)
            identities.add(changed_identity)
            output, report = self.firmware(name, changed)
            if report["counts"]["built_total"] != 1:
                raise RuntimeError("changed input incorrectly reused original output: " + name)
            if output.name == first.name:
                raise RuntimeError("changed input retained the original generic derivation output identity: " + name)
            observed = self.observe(name, output, changed)
            if name in {"source", "overlay", "configuration"} and observed["artifacts_blake3"]["zephyr.bin"] == first_manifest["artifacts_blake3"]["zephyr.bin"]:
                raise RuntimeError("changed fixture did not affect firmware bytes: " + name)
        for name, mutation, rejection in [
            ("missing-module", lambda p: p["roles"].pop(p["west"]["modules"]["cmsis"]["role"]), "zephyr-missing-role"),
            ("manifest-revision", lambda p: p["west"]["modules"]["cmsis"].update(revision="0" * 40), "zephyr-manifest-mismatch"),
            ("unsupported-board", lambda p: p.update(board="nrf52840dk/nrf52840"), "zephyr-board-unsupported"),
            ("unsupported-qualifier", lambda p: p.update(qualifiers=["cpu0"]), "zephyr-board-unsupported"),
            ("undeclared-path", lambda p: p["kconfig"][0].update(member="../prj.conf"), "zephyr-path-escape"),
        ]:
            bad = copy.deepcopy(self.plan); mutation(bad)
            self.identity(bad, name, rejection)
        bad = copy.deepcopy(self.plan)
        bad["roles"][project]["tree_blake3"] = "0" * 64
        self.firmware("content-mismatch", bad, expected=["zephyr-content-mismatch"])
        bad = self.mutate("resolved-manifest", project, [{"kind": "replace", "path": "resolved-west.json", "old": self.plan["west"]["zephyr_revision"], "new": "0" * 40}])
        self.firmware("resolved-manifest", bad, expected=["zephyr-manifest-mismatch"])
        bad = self.mutate("symlink-escape", project, [{"kind": "symlink", "path": "app/src/main.c", "target": "/etc/mantle-undeclared-source"}])
        self.firmware("symlink-escape", bad, expected=["zephyr-path-escape", "zephyr-missing-input"])
        bad = self.mutate("network", project, [{"kind": "replace", "path": "app/CMakeLists.txt", "old": "target_sources(app PRIVATE src/main.c)",
                                               "new": 'target_sources(app PRIVATE src/main.c)\nfile(DOWNLOAD "https://example.invalid/forbidden" "${CMAKE_BINARY_DIR}/forbidden" TIMEOUT 2)'}])
        self.firmware("network", bad, expected=["cmake-undeclared-download"])
        write_json(self.root / "proof.json", {"schema": "mantle-zephyr-real-build-proof-v1", "observations": self.observations,
                                              "non_claims": ["hardware execution", "flashing", "signing", "general board support", "different tool cohorts/store prefixes"]})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", type=Path, required=True)
    parser.add_argument("--runner", type=Path, required=True, help="built mantle-zephyr, same content as the admitted runner role")
    parser.add_argument("--expected-mantle-blake3", required=True)
    parser.add_argument("--expected-runner-blake3", required=True)
    parser.add_argument("--runner-role", default="runner")
    parser.add_argument("--plan", type=Path, required=True, help="complete independently pinned plan, not a template")
    parser.add_argument("--run-dir", type=Path, required=True, help="fresh scratch directory")
    parser.add_argument("--base-state", type=Path, action="append", required=True, help="signed admitted inputs only; no firmware output")
    parser.add_argument("--trusted-public-key", action="append", required=True, help="name:base64 Ed25519 key; never trust unsigned")
    parser.add_argument("--signing-key", type=Path, required=True)
    parser.add_argument("--b3sum", type=Path, required=True, help="independently pinned observer")
    args = parser.parse_args()
    try:
        Proof(args).execute()
    except (OSError, RuntimeError, ValueError, KeyError) as error:
        print(f"Zephyr proof failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
