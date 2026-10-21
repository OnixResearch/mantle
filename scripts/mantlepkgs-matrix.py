#!/usr/bin/env python3
"""Bounded, opt-in package matrix. Production APIs only; never a package resolver."""
import argparse
import copy
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import stat
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
COHORT = ROOT / "mantlepkgs/live-cohort"
FIXTURES = ROOT / "mantlepkgs/validation/fixtures"
PACKAGES = ("hello", "zlib", "libpng", "jq", "ripgrep", "bison")
PHASES = ("acquisition", "import", "plan", "execution", "behavior", "reuse")
NEGATIVES = ("floating", "deferred", "impure", "text-fixed", "git-fixed", "dynamic", "platform",
             "catalog-tamper", "missing-source-bundle", "source-identity-tamper", "missing-required-source")
REVISION = "dfd9566f82a6e1d55c30f861879186440614696e"
LOG_MAX = 16 * 1024 * 1024
FILE_COUNT_MAX = 262144
JSON_MAX = 1024 * 1024 * 1024


def load(path):
    if path.stat().st_size > JSON_MAX:
        raise ValueError(f"JSON exceeds {JSON_MAX} bytes: {path}")
    return json.loads(path.read_text())


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")
    temporary.replace(path)


def digest(path):
    result = subprocess.run(["b3sum", "--no-names", str(path)], capture_output=True,
                            check=True, timeout=600)
    value = result.stdout.decode().strip()
    if not re.fullmatch("[0-9a-f]{64}", value):
        raise ValueError("invalid b3sum response")
    return value


def in_root(root, logical):
    """Resolve symlinks as the consumer kernel will, including absolute links."""
    pending = list(Path(logical).parts)
    resolved = []
    links = 0
    while pending:
        part = pending.pop(0)
        if part in ("/", ".", ""):
            continue
        if part == "..":
            if resolved:
                resolved.pop()
            continue
        candidate = root.joinpath(*resolved, part)
        if candidate.is_symlink():
            links += 1
            if links > 40:
                raise ValueError("rootfs symlink loop")
            target = os.readlink(candidate)
            if target.startswith("/"):
                resolved.clear()
            pending = list(Path(target).parts) + pending
        else:
            resolved.append(part)
    return root.joinpath(*resolved)


def tree(root, clean=False):
    """Bind files, modes and symlink text. Never follow directory symlinks."""
    entries = []
    for base, dirs, files in os.walk(root, followlinks=False):
        for name in sorted(dirs + files):
            path = Path(base) / name
            relative = path.relative_to(root).as_posix()
            mode = path.lstat().st_mode
            if len(entries) >= FILE_COUNT_MAX:
                raise ValueError("rootfs file-count limit exceeded")
            if clean and ((name in ("nix", "nix-store", "nix-daemon", "nix-instantiate", "nix-build")
                           and not stat.S_ISDIR(mode)) or stat.S_ISSOCK(mode)):
                raise ValueError(f"Nix executable name or socket in consumer rootfs: {relative}")
            if stat.S_ISLNK(mode):
                target = os.readlink(path)
                if clean and not in_root(root, "/" + relative).exists():
                    raise ValueError(f"rootfs symlink has no in-root target: {relative} -> {target}")
                entry = {"path": relative, "kind": "symlink", "target": target}
            elif stat.S_ISREG(mode):
                entry = {"path": relative, "kind": "file", "bytes": path.stat().st_size,
                         "blake3": digest(path), "mode": stat.S_IMODE(mode)}
            elif stat.S_ISDIR(mode):
                entry = {"path": relative, "kind": "directory", "mode": stat.S_IMODE(mode)}
            else:
                raise ValueError(f"special file in input tree: {relative}")
            entries.append(entry)
    return sorted(entries, key=lambda item: item["path"])


class Commands:
    def __init__(self, out, seconds):
        self.out = out
        self.seconds = seconds
        self.sequence = 0

    def run(self, argv, stdin=None):
        self.sequence += 1
        stem = self.out / "commands" / f"{self.sequence:04d}"
        stem.parent.mkdir(parents=True, exist_ok=True)
        argv = list(map(str, argv))
        save(stem.with_suffix(".command.json"), {"argv": argv, "stdin": stdin})
        started = time.monotonic()
        bounded = None
        with stem.with_suffix(".stdout").open("wb") as stdout, stem.with_suffix(".stderr").open("wb") as stderr:
            # stdin comes from a regular temporary file, so a large pipe cannot block supervision.
            with stem.with_suffix(".stdin").open("w+b") as input_file:
                input_file.write((stdin or "").encode())
                input_file.seek(0)
                child = subprocess.Popen(argv, stdin=input_file, stdout=stdout, stderr=stderr,
                                         start_new_session=True, env={**os.environ, "LC_ALL": "C", "TZ": "UTC"})
                while child.poll() is None:
                    if time.monotonic() - started > self.seconds:
                        bounded = "timeout"
                    elif max(stdout.tell(), stderr.tell(), os.fstat(stdout.fileno()).st_size,
                             os.fstat(stderr.fileno()).st_size) > LOG_MAX:
                        bounded = "log-limit"
                    if bounded:
                        os.killpg(child.pid, signal.SIGKILL)
                        break
                    time.sleep(0.1)
                code = child.wait()
        result = {"command": str(stem.with_suffix(".command.json")), "exit_code": code,
                  "bound_failure": bounded, "stdout": str(stem.with_suffix(".stdout")),
                  "stderr": str(stem.with_suffix(".stderr")), "status": "passed" if code == 0 and not bounded else "failed"}
        save(stem.with_suffix(".result.json"), result)
        return result


def require(result):
    if result["status"] != "passed":
        raise ValueError(f"command failed; see {result['stderr']}")
    return result


def bundle_identity(path):
    matches = []
    carry = b""
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            data = carry + chunk
            matches.extend(re.findall(rb'"manifest_blake3"\s*:\s*"([0-9a-f]{64})"', data))
            carry = data[-128:]
            # Preserve only matches not wholly in carry to avoid counting twice.
            matches = list(dict.fromkeys(matches))
    if len(matches) != 1:
        raise ValueError("source bundle must contain one manifest identity")
    return matches[0].decode()


def outcome(status="not-run", **extra):
    return {"status": status, **extra}


def prepare(args):
    args.out.mkdir(parents=True, exist_ok=False)
    commands = Commands(args.out, args.command_seconds)
    report = {"schema": "mantlepkgs-matrix-preparation-v1", "revision": REVISION,
              "system": "x86_64-linux", "mantle_blake3": digest(args.mantle),
              "nix_program": str(args.nix), "nix_blake3": digest(args.nix),
              "driver_blake3": digest(Path(__file__)), "rows": [],
              "second_architecture": {"system": "aarch64-linux", "status": "not-run",
                                      "reason": "no native aarch64 runner admitted by this driver"}}
    save(args.out / "fixture-identities.json", tree(FIXTURES))
    normalized = require(commands.run([args.mantle, "--json", "mantlepkgs", "validate", "--manifest", COHORT / "manifest.ncl"]))
    manifest = load(Path(normalized["stdout"]))
    if manifest["source"]["revision"] != REVISION or manifest["systems"] != ["x86_64-linux"]:
        raise ValueError("cohort source/system changed; review the matrix before updating")
    if sorted(s["name"] for s in manifest["selectors"]) != sorted(PACKAGES):
        raise ValueError("cohort and behavior probe inventory differ")
    shutil.copytree(FIXTURES, args.out / "fixtures")
    for name in PACKAGES:
        row = {"package": name, "system": "x86_64-linux", **{phase: outcome() for phase in PHASES}}
        report["rows"].append(row)
        folder = args.out / name
        folder.mkdir()
        shutil.copytree(COHORT / "policy", folder / "policy")
        selected = copy.deepcopy(manifest)
        selected["selectors"] = [s for s in selected["selectors"] if s["name"] == name]
        save(folder / "manifest.json", selected)
        (folder / "manifest.ncl").write_text('import "manifest.json"\n')
        generated = commands.run([args.mantle, "--json", "mantlepkgs", "generate", "--manifest", folder / "manifest.ncl",
                                  "--nix-program", args.nix, "--output-root", folder / "producer"])
        row["acquisition"] = outcome(generated["status"], generation=generated)
        if generated["status"] == "passed":
            catalog = load(Path(generated["stdout"]))
            generation = folder / "producer" / selected["output"]["generation_directory"] / catalog["catalog_identity_blake3"]
            row["generation"] = str(generation.relative_to(args.out))
            row["catalog_blake3"] = catalog["catalog_identity_blake3"]
            row["producer_receipt"] = load(generation / "producer-receipt.json")
            prepared = commands.run([args.mantle, "mantlepkgs", "prepare-sources", "--generation", generation,
                                     "--package", name, "--system", row["system"], "--out", folder / "sources.json"])
            row["acquisition"]["sources"] = prepared
            row["acquisition"]["status"] = prepared["status"]
            if prepared["status"] == "passed":
                row["source_bundle_manifest_blake3"] = bundle_identity(folder / "sources.json")
                row["source_bundle_file_blake3"] = digest(folder / "sources.json")
        else:
            row["generation_failures"] = [load(path) for path in sorted((folder / "producer/failures").glob("*.json"))]
        save(args.out / "preparation.json", report)
    return all(row["acquisition"]["status"] == "passed" for row in report["rows"])


class Consumer:
    def __init__(self, args, commands, work, store):
        self.args, self.commands, self.work, self.store = args, commands, work, store
        work.mkdir(parents=True, exist_ok=True)
        store.mkdir(parents=True, exist_ok=True)

    def run(self, argv, stdin=None):
        command = [self.args.bwrap, "--die-with-parent", "--new-session", "--unshare-user", "--unshare-pid",
                   "--unshare-uts", "--unshare-ipc", "--unshare-net", "--ro-bind", self.args.rootfs, "/",
                   "--proc", "/proc", "--dev", "/dev", "--tmpfs", "/tmp", "--tmpfs", "/run",
                   "--ro-bind", self.args.prepared, "/inputs", "--bind", self.work, "/work",
                   "--bind", self.store, "/mantle/store", "--clearenv", "--setenv", "PATH", "/tools/bin",
                   "--setenv", "HOME", "/work/home", "--setenv", "TMPDIR", "/work/tmp",
                   "--setenv", "LC_ALL", "C", "--setenv", "TZ", "UTC", "--setenv", "CRUNCH_NO_FUSE", "1",
                   "--chdir", "/work", "--", *argv]
        (self.work / "tmp").mkdir(exist_ok=True)
        (self.work / "home").mkdir(exist_ok=True)
        return self.commands.run(command, stdin)

    def mantle(self, argv):
        return self.run(["/tools/bin/mantle", "--store", "/mantle/store", "--store-prefix", "/mantle/store",
                         "--state-dir", "/work/state", *argv])


def same_output(result, expected):
    require(result)
    actual = Path(result["stdout"]).read_text()
    if actual != expected:
        raise ValueError(f"behavior output mismatch: {actual!r} != {expected!r}")
    return result


def behavior(consumer, name, plan):
    roots = plan["roots"]
    if len(roots) != 1 or roots[0]["package_name"] != name:
        raise ValueError("unexpected root selection")
    outputs = roots[0]["target_outputs"]
    out = outputs["out"]
    results = []
    if name == "hello":
        results.append(same_output(consumer.run([out + "/bin/hello"]), "Hello, world!\n"))
        results.append(same_output(consumer.run([out + "/bin/hello", "--greeting=Mantle matrix"]), "Mantle matrix\n"))
    elif name == "jq":
        results.append(same_output(consumer.run([out + "/bin/jq", "-c", "{sum: ([.items[].n] | add), labels: [.items[].label]}"],
                                               '{"items":[{"n":7,"label":"alpha"},{"n":35,"label":"beta"}]}\n'),
                                   '{"sum":42,"labels":["alpha","beta"]}\n'))
        bad = consumer.run([out + "/bin/jq", "."], "{broken\n")
        if bad["exit_code"] != 5 or "parse error" not in Path(bad["stderr"]).read_text():
            raise ValueError("jq malformed-input rejection differs")
        results.append(bad)
    elif name == "ripgrep":
        results.append(same_output(consumer.run([out + "/bin/rg", "--no-config", "--color=never", "-n", "^alpha:[0-9]+$"],
                                               "alpha:7\nbeta:35\nalpha:42\nalpha:wrong\n"), "1:alpha:7\n3:alpha:42\n"))
        missing = consumer.run([out + "/bin/rg", "--no-config", "absent"], "present\n")
        if missing["exit_code"] != 1 or Path(missing["stdout"]).read_bytes() != b"":
            raise ValueError("ripgrep no-match semantics differ")
        results.append(missing)
    elif name == "bison":
        # BISON_PKGDATADIR and M4 must come from this output / admitted validation tools.
        results.append(require(consumer.run([out + "/bin/bison", "-o", "/work/calculator.c", "/inputs/fixtures/calculator.y"])))
        results.append(require(consumer.run(["/tools/bin/cc", "/work/calculator.c", "-o", "/work/calculator"])))
        results.append(same_output(consumer.run(["/work/calculator"], "6*7\n"), "42\n"))
        bad = consumer.run(["/work/calculator"], "6+*7\n")
        if bad["exit_code"] != 1:
            raise ValueError("generated parser accepted malformed expression")
        results.append(bad)
    else:
        if "dev" not in outputs or outputs["dev"] == out:
            raise ValueError(f"{name}: expected distinct dev/out outputs from pinned recipe")
        # Include every selected library's headers and libraries, but never foreign /nix/store outputs.
        selected = [outputs]
        if name == "libpng":
            selected.append(load(consumer.args.out / "zlib/plan.json")["roots"][0]["target_outputs"])
        flags = []
        for library in selected:
            flags += ["-I" + library["dev"] + "/include", "-L" + library["out"] + "/lib",
                      "-Wl,-rpath," + library["out"] + "/lib"]
        libraries = ["-lpng", "-lz"] if name == "libpng" else ["-lz"]
        results.append(require(consumer.run(["/tools/bin/cc", "/inputs/fixtures/" + name + ".c", *flags,
                                              *libraries, "-o", "/work/library-probe"])))
        expected = {"zlib": "zlib: round-trip and corrupt-stream rejection passed\n",
                    "libpng": "libpng: RGBA round-trip and invalid-signature rejection passed\n"}[name]
        results.append(same_output(consumer.run(["/work/library-probe"]), expected))
    return outcome("passed", commands=results, outputs=outputs,
                   scope="separate semantic consumer; does not alter package recipe identity")


def check_receipt(path, plan, warm=False):
    receipt = load(path)
    if receipt["status"] != "complete" or receipt["failure"] is not None:
        raise ValueError("realization receipt is incomplete")
    if receipt["plan_blake3"] != plan["plan_identity"]["value"]:
        raise ValueError("realization receipt plan identity differs")
    if receipt["cache_policy"]["substitution_enabled"] or not receipt["cache_policy"]["offline"]:
        raise ValueError("realization did not deny substitution/network")
    units = {unit["node_id"]: unit for unit in receipt["units"]}
    for root in plan["roots"]:
        unit = units[root["node_id"]]
        if unit["execution_class"] != ("already-present" if warm else "built"):
            raise ValueError("root was not built cold / reused warm")
        if {o["output_name"]: o["target_path"] for o in unit["outputs"]} != root["target_outputs"]:
            raise ValueError("realized root outputs differ from plan")
    if warm and any(unit["execution_class"] != "already-present" for unit in receipt["units"]):
        raise ValueError("warm run executed or fetched a unit")
    return receipt


def rejected(result, needle, receipt=None):
    text = Path(result["stdout"]).read_text(errors="replace") + Path(result["stderr"]).read_text(errors="replace")
    if result["exit_code"] in (0, None) or result["bound_failure"] or needle not in text:
        return outcome("failed", reason=f"negative did not reject for {needle!r}", command=result)
    if receipt and receipt.exists() and load(receipt).get("status") == "complete":
        return outcome("failed", reason="negative published a complete realization receipt", command=result)
    return outcome("passed", expected_diagnostic=needle, command=result)


def negatives(args, commands, prepared):
    # Negative admission uses a fresh store, never the successful package stores.
    work = args.out / "negatives"
    consumer = Consumer(args, commands, work, work / "store")
    results = {}
    root = "/nix/store/00000000000000000000000000000000-matrix.drv"
    output = "/nix/store/11111111111111111111111111111111-matrix"
    forms = {"floating": ('"","r:sha256",""', "unsupported-nix-derivation-output"),
             "deferred": ('"","",""', "unsupported-nix-derivation-output"),
             "impure": ('"","text:sha256","impure"', "unsupported-nix-derivation-output"),
             "text-fixed": (f'"{output}","text:sha256","' + "0" * 64 + '"', "unsupported-nix-derivation-output"),
             "git-fixed": (f'"{output}","git:sha1","' + "0" * 40 + '"', "unsupported-nix-derivation-output")}
    for name, (fields, code) in forms.items():
        (work / (name + ".drv")).write_text(f'Derive([("out",{fields})],[],[],"x86_64-linux","/bin/sh",[],[])')
        command = consumer.mantle(["foreign-import", "produce-nix", "--drv", root + "=/work/" + name + ".drv",
                                   "--root-derivation", root, "--producer-identity", "matrix-negative-v1",
                                   "--out-dir", "/work/" + name + "-artifacts"])
        results[name] = rejected(command, code)
        if any((work / (name + "-artifacts") / artifact).exists()
               for artifact in ("nixpkgs.graph.json", "nixpkgs.index.json")):
            results[name] = outcome("failed", reason="unsupported form published graph", command=command)
    # Recursive dynamic-input fixture is a parser fact, never flattened into a static graph.
    (work / "dynamic.drv").write_text('DrvWithVersion("xp-dyn-drv",[],[("' + root + '",(["out"],[("generated",["out"])]))],[],"x86_64-linux","/bin/sh",[],[])')
    results["dynamic"] = rejected(consumer.mantle(["foreign-import", "produce-nix", "--drv", root + "=/work/dynamic.drv",
        "--root-derivation", root, "--producer-identity", "matrix-negative-v1", "--out-dir", "/work/dynamic-artifacts"]),
        "unsupported-nix-dynamic-input")
    ready = next((row for row in prepared["rows"] if row["acquisition"]["status"] == "passed"), None)
    if ready is None:
        results["catalog-and-source-negatives"] = outcome("not-run", reason="no acquired package")
        return results
    name = ready["package"]
    generation = "/inputs/" + ready["generation"]
    common = ["--generation", generation, "--package", name, "--system", "x86_64-linux"]
    plan_args = ["--plan-out", "/work/negative.plan.json", "--import-receipt-out", "/work/negative.import.json"]
    results["platform"] = rejected(consumer.mantle(["mantlepkgs", "plan", "--generation", generation, "--package", name,
        "--system", "aarch64-linux", *plan_args]), "catalog-package-missing")
    # Copy only a small catalog tree, never the large source bundles / producer seed roots.
    shutil.copytree(args.prepared / ready["generation"], work / "tampered")
    with (work / "tampered/artifacts/shared.graph.json").open("ab") as stream:
        stream.write(b"\n")
    results["catalog-tamper"] = rejected(consumer.mantle(["mantlepkgs", "verify", "--generation", "/work/tampered"]),
                                             "catalog-artifact-mismatch")
    build = ["mantlepkgs", "build", *common, *plan_args, "--receipt-out", "/work/negative.realization.json", "--offline",
             "--source-bundle-blake3", ready["source_bundle_manifest_blake3"]]
    results["missing-source-bundle"] = rejected(consumer.mantle([*build, "--source-bundle", "/work/absent.sources.json"]),
        "absent.sources.json", work / "negative.realization.json")
    wrong = list(build)
    wrong[wrong.index("--source-bundle-blake3") + 1] = "0" * 64
    results["source-identity-tamper"] = rejected(consumer.mantle([*wrong, "--source-bundle", "/inputs/" + name + "/sources.json"]),
        "does not match the admitted digest", work / "negative.realization.json")
    exported = require(consumer.mantle(["source", "bundle", "export", "--source",
        "local-path:unrelated:/inputs/fixtures/zlib.c", "--to", "/work/unrelated.sources.json"]))
    incomplete = list(build)
    incomplete[incomplete.index("--source-bundle-blake3") + 1] = bundle_identity(work / "unrelated.sources.json")
    results["missing-required-source"] = rejected(consumer.mantle([*incomplete, "--source-bundle", "/work/unrelated.sources.json"]),
        "absent from the admitted source bundle", work / "negative.realization.json")
    results["missing-required-source"]["preparation"] = exported
    return results


def consume(args):
    args.out.mkdir(parents=True, exist_ok=False)
    if platform.system() != "Linux" or platform.machine() not in ("x86_64", "amd64"):
        raise ValueError("this matrix requires a native x86_64 Linux runner; no architecture fallback")
    prepared = load(args.prepared / "preparation.json")
    report = copy.deepcopy(prepared)
    report["schema"] = "mantlepkgs-behavior-matrix-v1"
    report["consumer_rootfs"] = tree(args.rootfs, clean=True)
    report["bwrap_blake3"] = digest(args.bwrap)
    report["validation_tools"] = {}
    if digest(Path(__file__)) != prepared["driver_blake3"]:
        raise ValueError("consumer driver differs from acquisition driver")
    for entry in report["consumer_rootfs"]:
        if entry.get("blake3") == prepared["nix_blake3"]:
            raise ValueError("producer Nix executable appears in consumer rootfs")
    for name in ("mantle", "bwrap", "cc", "sh", "m4"):
        path = in_root(args.rootfs, "/tools/bin/" + name)
        if not path.is_file():
            raise ValueError(f"missing admitted consumer tool: {name}")
        report["validation_tools"][name] = digest(path)
    if report["validation_tools"]["mantle"] != prepared["mantle_blake3"]:
        raise ValueError("consumer-toolchain-identity-mismatch: consumer Mantle differs from producer")
    if tree(args.prepared / "fixtures") != load(args.prepared / "fixture-identities.json"):
        raise ValueError("behavior fixtures differ from prepared identities")
    commands = Commands(args.out, args.command_seconds)
    preflight = Consumer(args, commands, args.out / "preflight", args.out / "preflight/store")
    report["clean_consumer"] = same_output(preflight.run(["/tools/bin/sh", "-c",
        "! command -v nix && ! command -v nix-store && ! command -v nix-daemon && "
        "test ! -e /nix/var/nix/daemon-socket/socket && printf 'consumer: no Nix commands or daemon socket\\n'"]),
        "consumer: no Nix commands or daemon socket\n")
    report["tool_versions"] = [require(preflight.run(["/tools/bin/" + tool, "--version"]))
                               for tool in ("mantle", "bwrap", "cc", "m4")]
    report["nested_sandbox"] = same_output(preflight.run(["/tools/bin/bwrap", "--ro-bind", "/", "/", "--",
        "/tools/bin/sh", "-c", "printf 'nested sandbox: admitted\\n'"]), "nested sandbox: admitted\n")
    save(args.out / "matrix.json", report)
    for row in report["rows"]:
        name = row["package"]
        if row["acquisition"]["status"] != "passed":
            continue
        work = args.out / name
        # A private cold store per package proves each root built, not inherited from another row.
        consumer = Consumer(args, commands, work, work / "store")
        if digest(args.prepared / name / "sources.json") != row["source_bundle_file_blake3"]:
            row["import"] = outcome("failed", reason="prepared source bundle file identity differs")
            continue
        generation = "/inputs/" + row["generation"]
        imported = consumer.mantle(["mantlepkgs", "verify", "--generation", generation])
        row["import"] = imported
        if imported["status"] != "passed":
            continue
        row["import"] = outcome("not-run", verification=imported)
        common = ["--generation", generation, "--package", name, "--system", row["system"]]
        plans = ["--plan-out", "/work/plan.json", "--import-receipt-out", "/work/import.json"]
        planned = consumer.mantle(["mantlepkgs", "plan", *common, *plans])
        row["plan"] = planned
        if planned["status"] != "passed":
            row["import"] = outcome("failed", verification=imported, command=planned,
                                    reason="combined import/plan admission rejected; inspect diagnostics")
            continue
        plan = load(work / "plan.json")
        row["import"] = outcome("passed", verification=imported, accepted_import=plan["accepted_import"],
                                receipt_file_blake3=digest(work / "import.json"))
        repeated = consumer.mantle(["mantlepkgs", "plan", *common, "--plan-out", "/work/repeat.plan.json",
                                   "--import-receipt-out", "/work/repeat.import.json"])
        row["plan"]["repeat"] = repeated
        if repeated["status"] != "passed" or digest(work / "plan.json") != digest(work / "repeat.plan.json") or digest(work / "import.json") != digest(work / "repeat.import.json"):
            row["plan"]["status"] = "failed"
            row["plan"]["reason"] = "identical admitted inputs did not reproduce exact plan/import bytes"
            continue
        row["plan"]["identity"] = plan["plan_identity"]
        row["plan"]["root_outputs"] = plan["roots"]
        build = ["mantlepkgs", "build", *common, *plans, "--source-bundle", "/inputs/" + name + "/sources.json",
                 "--source-bundle-blake3", row["source_bundle_manifest_blake3"], "--offline", "--jobs", str(args.jobs)]
        executed = consumer.mantle([*build, "--receipt-out", "/work/cold.realization.json"])
        row["execution"] = executed
        if executed["status"] != "passed":
            save(args.out / "matrix.json", report)
            continue
        try:
            cold = check_receipt(work / "cold.realization.json", plan)
            row["execution"]["receipt_blake3"] = cold["receipt_blake3"]
        except (ValueError, KeyError, OSError) as error:
            row["execution"] = outcome("failed", reason=str(error), command=executed)
            save(args.out / "matrix.json", report)
            continue
        try:
            row["behavior"] = behavior(consumer, name, plan)
        except (ValueError, KeyError, OSError) as error:
            row["behavior"] = outcome("failed", reason=str(error))
        reused = consumer.mantle([*build, "--receipt-out", "/work/warm.realization.json"])
        row["reuse"] = reused
        if reused["status"] == "passed":
            try:
                warm = check_receipt(work / "warm.realization.json", plan, warm=True)
                cold_outputs = sorted((o["target_path"], o["nar_sha256"], o["nar_size"]) for u in cold["units"] for o in u["outputs"])
                warm_outputs = sorted((o["target_path"], o["nar_sha256"], o["nar_size"]) for u in warm["units"] for o in u["outputs"])
                if cold_outputs != warm_outputs:
                    raise ValueError("warm output contents differ from cold receipt")
                row["reuse"]["receipt_blake3"] = warm["receipt_blake3"]
                row["reuse"]["behavior"] = behavior(consumer, name, plan)
            except (ValueError, KeyError, OSError) as error:
                row["reuse"] = outcome("failed", reason=str(error), command=reused)
        save(args.out / "matrix.json", report)
    try:
        report["negatives"] = negatives(args, commands, prepared)
    except (ValueError, KeyError, OSError) as error:
        report["negatives"] = {"driver-failure": outcome("failed", reason=str(error))}
    passed = all(row[phase]["status"] == "passed" for row in report["rows"] for phase in PHASES)
    passed = passed and all(report["negatives"].get(name, {}).get("status") == "passed" for name in NEGATIVES)
    report["status"] = "passed" if passed else "failed"
    report["non_claims"] = ["No Nix evaluator parity, full Nixpkgs coverage, bootstrap parity, package correctness or release authority.",
                            "Warm local reuse is not an independent reproducibility rebuild.",
                            "No second Linux architecture was exercised."]
    save(args.out / "matrix.json", report)
    return passed


def assemble_rootfs(args):
    """Copy only explicitly acquired runtime paths; never inspect an ambient closure."""
    args.out.mkdir(parents=True, exist_ok=False)
    paths = load(args.closure)
    if not isinstance(paths, list) or not paths or len(paths) > 2048:
        raise ValueError("closure must be an explicit JSON array of 1..2048 store directories")
    for value in sorted(set(paths)):
        path = Path(value)
        if path.parent != Path("/nix/store") or not path.is_dir() or path.is_symlink():
            raise ValueError(f"runtime closure entry is not a concrete store directory: {value}")
        shutil.copytree(path, args.out / value.lstrip("/"), symlinks=True)
    tools = {}
    for spec in args.tool:
        name, separator, value = spec.partition("=")
        if not separator or not re.fullmatch("[a-z][a-z0-9-]{0,31}", name) or name in tools:
            raise ValueError("tool must be one unique NAME=ABSOLUTE_FILE")
        path = Path(value)
        if not path.is_absolute() or not path.is_file() or not os.access(path, os.X_OK):
            raise ValueError(f"tool is not an executable file: {value}")
        destination = args.out / "tools/bin" / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination, follow_symlinks=True)
        destination.chmod(0o755)
        tools[name] = {"producer_path": str(path), "blake3": digest(destination)}
    for directory in ("proc", "dev", "tmp", "run", "work", "inputs", "mantle/store", "bin", "etc"):
        (args.out / directory).mkdir(parents=True, exist_ok=True)
    (args.out / "bin/sh").symlink_to("/tools/bin/sh")
    inventory = tree(args.out, clean=True)
    save(args.out.parent / (args.out.name + ".identity.json"),
         {"schema": "mantlepkgs-consumer-root-v1", "closure_input_blake3": digest(args.closure),
          "tools": tools, "entries": inventory})
    return True


def comparable_row(row):
    if row is None:
        return None
    facts = {key: row[key] for key in ("package", "system", "catalog_blake3", "source_bundle_manifest_blake3",
                                      "source_bundle_file_blake3", "producer_receipt") if key in row}
    for phase in PHASES:
        facts[phase] = {key: value for key, value in row[phase].items()
                        if key in ("status", "identity", "root_outputs", "receipt_blake3", "outputs",
                                   "accepted_import", "receipt_file_blake3")}
    return facts


def compare(args):
    base, head = load(args.base), load(args.head)
    if any(report.get("schema") != "mantlepkgs-behavior-matrix-v1" for report in (base, head)):
        raise ValueError("compare requires real behavior-matrix reports, not preparation/checklists")
    index = lambda report: {(row["package"], row["system"]): row for row in report["rows"]}
    old, new = index(base), index(head)
    changes, regressions = [], []
    for key in sorted(old.keys() | new.keys()):
        before, after = comparable_row(old.get(key)), comparable_row(new.get(key))
        if before != after:
            changes.append({"package": key[0], "system": key[1], "base": before, "head": after})
        if before and (not after or any(before[p]["status"] == "passed" and after[p]["status"] != "passed" for p in PHASES)):
            regressions.append(list(key))
    for name, result in base.get("negatives", {}).items():
        if result["status"] == "passed" and head.get("negatives", {}).get(name, {}).get("status") != "passed":
            regressions.append(["negative", name])
    identity_changes = [{"field": key, "base": base.get(key), "head": head.get(key)}
                        for key in ("revision", "driver_blake3", "mantle_blake3", "nix_blake3", "validation_tools", "consumer_rootfs")
                        if base.get(key) != head.get(key)]
    complete = len(new) == len(head["rows"]) and set(new) == {(name, "x86_64-linux") for name in PACKAGES}
    complete = complete and all(row[phase]["status"] == "passed" for row in new.values() for phase in PHASES)
    complete = complete and all(head.get("negatives", {}).get(name, {}).get("status") == "passed" for name in NEGATIVES)
    passed = complete and not regressions and head.get("status") == "passed"
    save(args.out, {"schema": "mantlepkgs-matrix-change-v1", "base_blake3": digest(args.base),
                    "head_blake3": digest(args.head), "changes": changes, "identity_changes": identity_changes,
                    "regressions": regressions, "status": "passed" if passed else "failed"})
    return passed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    production = sub.add_parser("prepare", help="explicit Nix producer/acquisition; no Mantle package execution")
    production.add_argument("--mantle", type=Path, required=True)
    production.add_argument("--nix", type=Path, required=True)
    production.add_argument("--out", type=Path, required=True)
    assembly = sub.add_parser("rootfs", help="copy explicitly acquired runtime closure; no Nix invocation")
    assembly.add_argument("--closure", type=Path, required=True, help="JSON array of exact /nix/store directories")
    assembly.add_argument("--tool", action="append", default=[], help="NAME=ABSOLUTE_EXECUTABLE; repeat per tool")
    assembly.add_argument("--out", type=Path, required=True)
    consumption = sub.add_parser("consume", help="isolated cold build, semantics, warm reuse and negatives")
    consumption.add_argument("--prepared", type=Path, required=True)
    consumption.add_argument("--rootfs", type=Path, required=True)
    consumption.add_argument("--bwrap", type=Path, required=True)
    consumption.add_argument("--out", type=Path, required=True)
    consumption.add_argument("--jobs", type=int, default=4)
    for command in (production, consumption):
        command.add_argument("--command-seconds", type=int, default=21600)
    comparison = sub.add_parser("compare", help="deterministic changes; regressions return nonzero")
    comparison.add_argument("--base", type=Path, required=True)
    comparison.add_argument("--head", type=Path, required=True)
    comparison.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    for key, value in vars(args).items():
        if isinstance(value, Path):
            setattr(args, key, value.absolute())
    if hasattr(args, "command_seconds") and not 1 <= args.command_seconds <= 86400:
        parser.error("--command-seconds must be between 1 and 86400")
    if hasattr(args, "jobs") and not 1 <= args.jobs <= 64:
        parser.error("--jobs must be between 1 and 64")
    try:
        return 0 if {"prepare": prepare, "rootfs": assemble_rootfs, "consume": consume, "compare": compare}[args.mode](args) else 1
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"matrix blocked: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
