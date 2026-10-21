#!/usr/bin/env python3
"""Real HIP compilation proof. Requires already-authorized signed input closures.

No downloading, Nix realization, compilation of Mantle, driver/device access,
license acceptance, or unsigned store admission is performed by this driver.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import shutil
import socket
import stat
import subprocess


def run(command, directory, *, success=True, log_dir=None):
    environment = os.environ.copy()
    if log_dir is not None:
        # Exclusive creation proves these logs cannot have come from the seed or
        # another invocation, even when the same case is reused for a cache hit.
        log_dir.mkdir(parents=True)
        environment["CRUNCH_LOG_DIR"] = str(log_dir)
    result = subprocess.run([str(arg) for arg in command], capture_output=True, text=True, check=False, env=environment)
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "command.json").write_text(json.dumps([str(arg) for arg in command], indent=2))
    (directory / "stdout").write_text(result.stdout)
    (directory / "stderr").write_text(result.stderr)
    if success and result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}), see {directory}")
    return result


def digest(tool, path):
    result = subprocess.run([str(tool), "--no-names", str(path)], capture_output=True, text=True, check=True)
    value = result.stdout.strip()
    if len(value) != 64 or any(char not in "0123456789abcdef" for char in value):
        raise RuntimeError("invalid BLAKE3 observer result")
    return value


def expression(plan):
    # Double JSON encoding is a safe Nickel string literal, not code interpolation.
    return 'let gpu = import "gpu.ncl" in gpu.mkGpu (std.deserialize \'Json ' + json.dumps(json.dumps(plan)) + ')\n'


MAX_FAILURE_BYTES = 8 * 1024 * 1024
FAILURE_DIAGNOSTICS = {
    "missing-library": r"^gpu-device-library-missing-or-escaping: ",
    "undeclared-host-path": r"fatal error: '/tmp/mantle-gpu-undeclared-secret\.h' file not found[\s\S]*\n^gpu-compile-failed: ",
    "network": r"^gpu-network-denied: Connection refused \(os error 111\)$",
}


def command_report(result, schema, entry, case, prefix):
    if len(result.stdout.encode()) > MAX_FAILURE_BYTES:
        raise RuntimeError("oversized command report")
    report = json.loads(result.stdout)
    if (not isinstance(report, dict) or report.get("schema") != schema
            or report.get("file") != str(entry) or report.get("state_dir") != str(case / "state")
            or report.get("output_dir") != str(case / "store") or report.get("store_dir") != prefix):
        raise RuntimeError("report does not describe the current build command")
    return report


def planned_derivation(result, entry, case, prefix, label):
    report = command_report(result, "crunch-build-plan-v1", entry, case, prefix)
    entries = report.get("entries")
    if (result.returncode or not isinstance(entries, list) or len(entries) != 1
            or entries[0].get("label") != label or entries[0].get("action") != "build"):
        raise RuntimeError("negative requires one current, uncached, executable derivation")
    key = entries[0].get("drv_key")
    if (not isinstance(key, str) or str(Path(key).parent) != prefix
            or not re.fullmatch(r"[0123456789abcdfghijklmnpqrsvwxyz]{32}-[^/]+\.drv", Path(key).name)):
        raise RuntimeError("negative plan has no bounded derivation identity")
    return key


def diagnosed_failure(result, entry, case, prefix, label, drv_key, log_dir, diagnostic, *, forbidden=()):
    """Classify only this command's typed failure and its exact saved builder log.

    Shapes and headers are produced by src/build_failure.rs, build_report.rs,
    and build_log.rs. A build-phase envelope alone is insufficient: planning and
    prerequisite failures can also be classified as build/builder there.
    """
    report = command_report(result, "crunch-build-report-v1", entry, case, prefix)
    failures = report.get("failed")
    counts = report.get("counts", {})
    if (not result.returncode or not isinstance(failures, list) or len(failures) != 1
            or counts.get("failed_total") != 1 or counts.get("succeeded_total") != 0
            or report.get("outcomes") != [] or report.get("diagnostic_persistence_failures") != []):
        raise RuntimeError("negative requires one unambiguous current execution failure")
    failure = failures[0]
    if (failure.get("root") != label or failure.get("drv_key") != drv_key
            or failure.get("phase") != "build" or failure.get("error_class") != "builder"
            or not isinstance(failure.get("message"), str)
            or not failure["message"].startswith("store error: build: nonzero exit code:")):
        raise RuntimeError("negative did not fail execution of the intended derivation")
    expected_log = log_dir / (Path(drv_key).name + ".log")
    if (failure.get("saved_log_path") != str(expected_log) or log_dir.is_symlink()
            or log_dir.resolve() != log_dir or expected_log.resolve().parent != log_dir):
        raise RuntimeError("failure log is not confined to this command's fresh log directory")
    descriptor = os.open(expected_log, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(descriptor, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1 or metadata.st_size > MAX_FAILURE_BYTES:
            raise RuntimeError("failure log must be a bounded, unshared regular file")
        raw = stream.read(MAX_FAILURE_BYTES + 1)
    if len(raw) > MAX_FAILURE_BYTES:
        raise RuntimeError("oversized failure log")
    log = raw.decode("utf-8", errors="strict")
    header = (f"# crunch build log\n# derivation: {label}\n# drv_path: {Path(drv_key).name}\n"
              "# status: failure\n# timestamp: ")
    if not log.startswith(header):
        raise RuntimeError("saved log identity/status disagrees with current failure")
    timestamp, separator, body = log[len(header):].partition("\n\n")
    if not separator or not timestamp.isdecimal() or not body.startswith(failure["message"]):
        raise RuntimeError("saved log does not contain the current reported failure")
    if not re.search(diagnostic, body, re.MULTILINE) or any(text in body for text in forbidden):
        raise RuntimeError("current execution did not diagnose the intended negative boundary")
    return {"rejected": True, "exit_code": result.returncode, "drv_key": drv_key,
            "phase": failure["phase"], "error_class": failure["error_class"],
            "saved_log_path": str(expected_log), "diagnostic": diagnostic}




def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["mantle", "runner", "b3sum", "plan", "seed-store", "seed-state", "signing-key", "run-dir"]:
        parser.add_argument("--" + name, type=Path, required=True)
    for name in ["plan", "mantle", "runner"]:
        parser.add_argument("--expected-" + name + "-blake3", required=True)
    parser.add_argument("--trusted-public-key", action="append", required=True)
    parser.add_argument("--lib", type=Path, default=Path(__file__).resolve().parent.parent / "lib")
    args = parser.parse_args()
    for name in ["mantle", "runner", "b3sum", "plan", "seed_store", "seed_state", "signing_key", "run_dir", "lib"]:
        setattr(args, name, getattr(args, name).resolve())
    if args.run_dir.exists():
        parser.error("--run-dir must be fresh")
    if any(args.run_dir.is_relative_to(seed) for seed in [args.seed_store, args.seed_state]):
        parser.error("proof scratch must be outside both immutable seed trees")
    args.run_dir.mkdir()
    receipt = {"schema": "mantle-hip-real-proof-v1", "backend": "hip-clang", "compilation_only": True,
               "cases": {}, "non_claims": ["CUDA", "device execution", "JIT", "GPU correctness", "performance", "universal reproducibility"]}
    try:
        if any(path.is_symlink() for path in args.seed_state.rglob("*")):
            raise RuntimeError("seed state must be a self-contained copy without shared mutable symlinks")
        for name in ["plan", "mantle", "runner"]:
            actual = digest(args.b3sum, getattr(args, name))
            if actual != getattr(args, "expected_" + name + "_blake3"):
                raise RuntimeError(f"out-of-band {name} identity mismatch")
            receipt[name + "_blake3"] = actual
        receipt["nickel_helper_blake3"] = digest(args.b3sum, args.lib / "gpu.ncl")
        plan = json.loads(args.plan.read_text())
        prefix = plan["store_prefix"]
        original = copy.deepcopy(plan)
        root = plan["roles"][plan["runner"]["role"]]
        admitted_runner = args.seed_store / Path(root).name / plan["runner"]["member"]
        if digest(args.b3sum, admitted_runner) != receipt["runner_blake3"]:
            raise RuntimeError("host planner and admitted runner differ")
        # Validate the entire pure contract before making any sandbox requests.
        run([args.runner, "lower", args.plan], args.run_dir / "lower")

        def fresh(name):
            case = args.run_dir / name
            case.mkdir()
            shutil.copytree(args.seed_store, case / "store", symlinks=True)
            shutil.copytree(args.seed_state, case / "state", symlinks=True)
            return case

        def base(case):
            return [args.mantle, "--json", "--store", case / "store", "--state-dir", case / "state", "--store-prefix", prefix]

        def admit(case):
            for index, logical in enumerate(sorted(set(original["roles"].values()))):
                run(base(case) + ["source", "bundle", "import-store-path", "--logical", logical,
                    "--physical", case / "store" / Path(logical).name,
                    "--trusted-public-keys", ",".join(args.trusted_public_key), "--pin"], case / f"admit-{index}")

        def build(name, selected, *, case=None, expect=True):
            case = case or fresh(name)
            admit(case)
            entry = case / (name + ".ncl")
            entry.write_text(expression(selected))
            command = base(case) + ["build", entry, "-I", args.lib, "--strict-hermetic", "--no-substitute",
                "--offline-source-preflight", "--signing-key", args.signing_key,
                "--trusted-public-keys", ",".join(args.trusted_public_key)]
            directory = case / ("build-" + name)
            if not expect:
                planned = run(command + ["--plan"], case / ("plan-" + name),
                    log_dir=case / ("plan-" + name) / "logs")
                drv_key = planned_derivation(planned, entry, case, prefix, selected["name"])
            result = run(command, directory, success=expect, log_dir=directory / "logs")
            if not expect:
                receipt["cases"][name] = diagnosed_failure(result, entry, case, prefix, selected["name"],
                    drv_key, directory / "logs", FAILURE_DIAGNOSTICS[name])
                return None
            report = json.loads(result.stdout)
            if report["schema"] != "crunch-build-report-v1" or report["counts"]["failed_total"]:
                raise RuntimeError(f"invalid build report in {name}")
            outcomes = [item for item in report["outcomes"] if item["label"] == selected["name"]]
            if len(outcomes) != 1:
                raise RuntimeError(f"missing unique root outcome in {name}")
            outcome = outcomes[0]
            output = next(item["path"] for item in outcome["outputs"] if item["name"] == "out")
            physical = case / "store" / Path(output).name
            provenance = json.loads((physical / "share/mantle/gpu-provenance.json").read_text())
            if provenance["inspection"]["architecture"] != selected["architecture"] or provenance["inspection"]["entry_points"] != selected["entry_points"]:
                raise RuntimeError("architecture or entry-point observation mismatch")
            object_hash = digest(args.b3sum, physical / "lib/kernel.o")
            if object_hash != provenance["artifact_blake3"] or not provenance["compilation_only"]:
                raise RuntimeError("artifact observation mismatch")
            observation = {"drv_key": outcome["drv_key"], "output": output, "cached": outcome["cached"], "artifact_blake3": object_hash,
                "files_blake3": {str(path.relative_to(physical)): digest(args.b3sum, path) for path in sorted(physical.rglob("*")) if path.is_file()},
                "plan_blake3": provenance["plan_blake3"]}
            receipt["cases"][name] = observation
            return observation

        first = build("clean-a", plan)
        second = build("clean-b", plan)
        if first["cached"] or second["cached"]:
            raise RuntimeError("seed state contained the GPU output; repeated-clean proof is invalid")
        if first["drv_key"] != second["drv_key"] or first["files_blake3"] != second["files_blake3"]:
            raise RuntimeError("repeated clean builds diverged; inspect retained artifacts")
        cached = build("cache-hit", plan, case=args.run_dir / "clean-a")
        if not cached["cached"] or cached["artifact_blake3"] != first["artifact_blake3"]:
            raise RuntimeError("signed output cache did not preserve the artifact")
        changed = copy.deepcopy(plan)
        changed["optimization"] = 0 if plan["optimization"] else 2
        option = build("changed-option", changed)
        if option["cached"] or option["drv_key"] == first["drv_key"] or option["plan_blake3"] == first["plan_blake3"]:
            raise RuntimeError("compiler option did not invalidate the admitted identity")
        source = copy.deepcopy(plan)
        source["kernel"]["member"] = str(Path(source["kernel"]["member"]).parent / "changed.hip")
        source_change = build("changed-source", source)
        generated = copy.deepcopy(plan)
        if not generated["generated"]:
            raise RuntimeError("proof requires the admitted generated_config.h fixture input")
        generated["generated"][0]["member"] = str(Path(generated["generated"][0]["member"]).parent / "generated_changed.h")
        generated_change = build("changed-generated-input", generated)
        for observation in [source_change, generated_change]:
            if observation["cached"] or observation["drv_key"] == first["drv_key"] or observation["artifact_blake3"] == first["artifact_blake3"]:
                raise RuntimeError("changed kernel/generated input failed to invalidate identity and code")
        # All contract negatives are observed through the real planner, not mocks.
        for name, mutate, expected in [
            ("unsupported-architecture", lambda p: p.update(architecture="native"), "gpu-combination-unsupported"),
            ("unsupported-toolkit", lambda p: p.update(toolkit_revision="6.0.2"), "gpu-combination-unsupported"),
            ("unsupported-host-compiler", lambda p: p["host_compiler"].update(version="19.1.0"), "gpu-compiler-unsupported"),
            ("path-escape", lambda p: p["kernel"].update(member="../kernel.hip"), "gpu-path-escape"),
            ("undeclared-role", lambda p: p["kernel"].update(role="unadmitted"), "gpu-role-undeclared"),
        ]:
            rejected = copy.deepcopy(plan)
            mutate(rejected)
            path = args.run_dir / (name + ".json")
            path.write_text(json.dumps(rejected))
            result = run([args.runner, "lower", path], args.run_dir / name, success=False)
            if result.returncode == 0 or expected not in result.stderr:
                raise RuntimeError(f"{name} did not produce its bounded rejection")
            receipt["cases"][name] = {"rejected": True, "diagnostic": expected}
        missing = copy.deepcopy(plan)
        missing["device_libraries"]["member"] = str(Path(missing["device_libraries"]["member"]).parent)
        build("missing-library", missing, expect=False)
        # Both positive and negative sources are part of the separately admitted fixture.
        escape = copy.deepcopy(plan)
        kernel_dir = Path(escape["kernel"]["member"]).parent
        escape["kernel"]["member"] = str(kernel_dir / "escape.hip")
        secret = Path("/tmp/mantle-gpu-undeclared-secret.h")
        descriptor = os.open(secret, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            os.write(descriptor, b"#define HOST_SECRET_SHOULD_NOT_BE_VISIBLE 1\n")
            os.close(descriptor)
            descriptor = None
            build("undeclared-host-path", escape, expect=False)
        finally:
            if descriptor is not None:
                os.close(descriptor)
            secret.unlink()
        # Same admitted runner, generic network-denied derivation, a live host
        # loopback listener as a positive control; no Internet dependency.
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            listener.listen(8)
            port = listener.getsockname()[1]
            with socket.create_connection(("127.0.0.1", port), timeout=2):
                pass
            case = fresh("network")
            admit(case)
            runner_logical = root + "/" + plan["runner"]["member"]
            entry = case / "network.ncl"
            entry.write_text('let m = import "lib.ncl" in { name = "hip-network-denial", builder = ' + json.dumps(runner_logical)
                + ', args = ["probe-network", ' + json.dumps(str(port)) + '], system = \'x86_64-linux, inputs = '
                + json.dumps(list(sorted(set(plan["roles"].values())))) + ', addressing_mode = \'input-addressed } | m.Derivation\n')
            command = base(case) + ["build", entry, "-I", args.lib, "--strict-hermetic", "--no-substitute",
                "--offline-source-preflight", "--signing-key", args.signing_key, "--trusted-public-keys", ",".join(args.trusted_public_key)]
            planned = run(command + ["--plan"], case / "plan", log_dir=case / "plan" / "logs")
            drv_key = planned_derivation(planned, entry, case, prefix, "hip-network-denial")
            result = run(command, case / "build", success=False, log_dir=case / "build" / "logs")
            receipt["cases"]["network"] = diagnosed_failure(result, entry, case, prefix, "hip-network-denial",
                drv_key, case / "build" / "logs", FAILURE_DIAGNOSTICS["network"],
                forbidden=("gpu-network-escape:", "gpu-network-probe-inconclusive:"))
            receipt["cases"]["network"]["host_listener_positive_control"] = True
        tamper = fresh("input-tampering")
        logical = plan["roles"][plan["kernel"]["role"]]
        physical = tamper / "store" / Path(logical).name
        target = physical / plan["kernel"]["member"]
        # Refuse to follow symlinks into the original immutable source closure.
        if physical.is_symlink() or not target.resolve().is_relative_to(physical) or target.is_symlink():
            raise RuntimeError("tamper target is not an isolated copied regular source file")
        target.chmod(target.stat().st_mode | 0o200)
        target.write_bytes(target.read_bytes() + b"\n// intentional admission tamper\n")
        result = run(base(tamper) + ["source", "bundle", "import-store-path", "--logical", logical,
            "--physical", physical, "--trusted-public-keys", ",".join(args.trusted_public_key), "--pin"], tamper / "admission", success=False)
        if result.returncode == 0:
            raise RuntimeError("tampered source was admitted")
        receipt["cases"]["input-tampering"] = {"rejected": True, "boundary": "signed-PathInfo-actual-NAR"}
        receipt["status"] = "passed"
        (args.run_dir / "proof.json").write_text(json.dumps(receipt, indent=2) + "\n")
    except Exception as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error)
        (args.run_dir / "proof.json").write_text(json.dumps(receipt, indent=2) + "\n")
        raise


if __name__ == "__main__":
    main()
