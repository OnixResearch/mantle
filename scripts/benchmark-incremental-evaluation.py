#!/usr/bin/env python3
"""Reproducible issue #19 baseline. Never claims an incremental implementation."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time


def root_source(count, leaf_name="leaf-0.ncl"):
    fields = []
    for index in range(count):
        leaf = leaf_name if index == 0 else f"leaf-{index}.ncl"
        fields.append(f'r{index} = import "{leaf}",')
    return "{\n" + "\n".join(fields) + "\n}\n"


def leaf_source(index, work, version=0):
    # Explicit bounded input rather than version-sensitive std.array.generate.
    values = ",".join(str(value) for value in range(work))
    return (f'let shared = import "shared.ncl" in '
            f'let sum = std.array.fold_left (fun a b => a + b) 0 [{values}] in '
            f'{{ name = "eval-root-{index}", builder = "/bin/sh", '
            f'args = ["-c", "echo %{{shared}}:%{{std.serialize \'Json sum}}:{version} > $out"], }}\n')


def transact(child, request):
    started = time.perf_counter_ns()
    child.stdin.write(json.dumps(request) + "\n")
    child.stdin.flush()
    line = child.stdout.readline()
    if not line:
        raise RuntimeError(f"evaluation runner exited unexpectedly: {child.poll()}")
    response = json.loads(line)
    response["request_wall_ns"] = time.perf_counter_ns() - started
    return response


def execute_build(mantle, directory, root, paths):
    started = time.perf_counter_ns()
    command = [mantle, "--json", "--store", str(directory / "store"),
               "--state-dir", str(directory / "state"), "build", str(root), "--no-substitute"]
    for path in paths:
        command.extend(["--import-path", path])
    result = subprocess.run(command, text=True, capture_output=True, timeout=180)
    if result.returncode:
        raise RuntimeError(f"build failed ({result.returncode}): {result.stderr}")
    report = json.loads(result.stdout)
    outcomes = report["outcomes"]
    return {"wall_ns": time.perf_counter_ns() - started,
            "builder_execution_count": sum(not row["cached"] for row in outcomes),
            "cached_outcome_count": sum(row["cached"] for row in outcomes),
            "compile_count": 0,
            "scope": "one-shot-shell-builds-same-store-not-live-watch-scheduler"}


def run_round(args, round_index, output):
    with tempfile.TemporaryDirectory(prefix="mantle-evaluation-baseline-") as scratch:
        directory = Path(scratch)
        root = directory / "main.ncl"
        shared = directory / "shared.ncl"
        shared.write_text('"initial"\n')
        root.write_text(root_source(args.roots))
        for index in range(args.roots + 1):
            (directory / f"leaf-{index}.ncl").write_text(leaf_source(index, args.work))
        extra = directory / "alternate"
        extra.mkdir()
        (extra / "alternate.ncl").write_text(leaf_source(0, args.work, 7))
        paths = []
        prior = None
        configuration = "baseline-v1"
        child = subprocess.Popen([args.runner], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        try:
            def observe(case, expected_failure=False, expected_changes=None):
                nonlocal prior
                response = transact(child, {"root": str(root), "import_paths": paths,
                                            "mode": args.mode, "configuration": configuration})
                failed = "error" in response
                if failed != expected_failure:
                    raise RuntimeError(f"{case}: unexpected evaluation response: {response}")
                response.update({"round": round_index, "case": case, "roots_requested": args.roots,
                                 "work_items_per_leaf": args.work,
                                 "configuration": {"import_paths": list(paths),
                                     "NICKEL_IMPORT_PATH": os.environ.get("NICKEL_IMPORT_PATH")}})
                if not failed:
                    if args.mode == "independent-import-roots":
                        if response["mode"] != "independent-import-roots":
                            raise RuntimeError(f"{case}: supported fixture unexpectedly fell back: {response}")
                        reused = {"unchanged": args.roots, "leaf-edit": args.roots - 1,
                                  "goal-add": args.roots, "goal-remove": args.roots,
                                  "unrelated-edit": args.roots,
                                  "rename-import-change": args.roots - 1}.get(case, 0)
                        if response["reused_root_computations"] != reused:
                            raise RuntimeError(f"{case}: wrong root reuse count: {response}")
                    current = response["roots"]
                    if prior is not None:
                        response["changed_labels"] = sorted(key for key in set(prior) | set(current)
                                                             if prior.get(key) != current.get(key))
                        if response["changed_labels"] != sorted(expected_changes):
                            raise RuntimeError(f"{case}: wrong changed roots: {response['changed_labels']}")
                    prior = current
                    if args.mantle:
                        response["build_observation"] = execute_build(args.mantle, directory, root, paths)
                output.write(json.dumps(response, sort_keys=True) + "\n")
                output.flush()

            observe("cold")
            observe("unchanged", expected_changes=[])
            (directory / "leaf-0.ncl").write_text(leaf_source(0, args.work, 1))
            observe("leaf-edit", expected_changes=["r0"])
            shared.write_text('"changed"\n')
            observe("shared-import-edit", expected_changes=[f"r{i}" for i in range(args.roots)])
            root.write_text(root_source(args.roots + 1))
            observe("goal-add", expected_changes=[f"r{args.roots}"])
            root.write_text(root_source(args.roots))
            observe("goal-remove", expected_changes=[f"r{args.roots}"])
            (directory / "unrelated.ncl").write_text("42\n")
            observe("unrelated-edit", expected_changes=[])
            (directory / "leaf-0.ncl").unlink()
            observe("delete", True)
            (directory / "leaf-0.ncl").write_text(leaf_source(0, args.work, 2))
            observe("delete-recovery", expected_changes=["r0"])
            (directory / "leaf-0.ncl").rename(directory / "renamed.ncl")
            root.write_text(root_source(args.roots, "renamed.ncl"))
            observe("rename-import-change", expected_changes=[])
            root.write_text(root_source(args.roots, "alternate.ncl"))
            observe("unresolved-import", True)
            paths.append(str(extra))
            observe("configuration-import-path-change", expected_changes=["r0"])
            shared.write_text("invalid : ???\n")
            observe("failed-evaluation", True)
            shared.write_text('"recovered"\n')
            observe("failure-recovery", expected_changes=[f"r{i}" for i in range(args.roots)])
            configuration = "baseline-v2"
            observe("configuration-policy-change", expected_changes=[])
        finally:
            child.stdin.close()
            try:
                code = child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
                raise
            if code:
                raise RuntimeError(f"runner exit status {code}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", required=True, help="immutable prebuilt evaluation_baseline executable")
    parser.add_argument("--output", required=True)
    parser.add_argument("--rounds", type=int, default=5)
    parser.add_argument("--roots", type=int, default=32)
    parser.add_argument("--work", type=int, default=512)
    parser.add_argument("--mantle", help="optional immutable CLI; also exercise actual shell builds")
    parser.add_argument("--mode", choices=["fresh", "independent-import-roots"], default="fresh")
    args = parser.parse_args()
    if not 1 <= args.rounds <= 100 or not 2 <= args.roots <= 64 or not 1 <= args.work <= 4096:
        parser.error("bounds: rounds 1..100, roots 2..64, work 1..4096")
    args.runner = str(Path(args.runner).resolve(strict=True))
    if args.mantle:
        args.mantle = str(Path(args.mantle).resolve(strict=True))
    # Environment identity is explicit. A separate invocation with a poisoned
    # NICKEL_IMPORT_PATH exercises the facade's documented non-consumption.
    with open(args.output, "x", encoding="utf-8") as output:
        for round_index in range(args.rounds):
            run_round(args, round_index, output)


if __name__ == "__main__":
    main()
