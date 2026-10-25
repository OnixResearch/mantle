#!/usr/bin/env python3
"""Exercise the real C/C++ cache daemon and driver; receipts are not rebuild proof.

Requires prebuilt binaries, GCC-compatible native compilers, bwrap, b3sum and
ldd. Fails rather than substituting mocks or skipped checks. Covers complete
compile-mode probe failures, volatile headers, advisory receipt loss, and real
FIFO-gated concurrency/fairness, saturated Unix backlogs and authority death
during compilation. Link/preprocess probe caching is not supported;
preprocessing in the contention fixture is forwarded unchanged, with slots only.
Process-tree ownership is cooperative: descendants must retain their fresh PGID.
"""
import argparse
import copy
import errno
import json
import os
from pathlib import Path
import platform
import re
import shutil
import socket
import subprocess
import sys
import time


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def executable(value, name):
    path = Path(value)
    require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK),
            f"{name} must name an absolute executable file: {value}")
    return path.resolve()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--driver", required=True)
    parser.add_argument("--daemon", required=True)
    parser.add_argument("--compiler", required=True)
    parser.add_argument("--cxx", required=True)
    parser.add_argument("--work-dir", required=True, type=Path)
    parser.add_argument("--toolchain-root", action="append", type=Path, default=[],
                        help="Additional complete compiler/runtime input roots")
    args = parser.parse_args()
    driver = executable(args.driver, "driver")
    daemon_binary = executable(args.daemon, "daemon")
    compiler = executable(args.compiler, "compiler")
    cxx = executable(args.cxx, "cxx")
    utilities = {}
    for name in ("b3sum", "bwrap", "ldd"):
        located = shutil.which(name)
        require(located is not None, f"missing required executable: {name}")
        utilities[name] = executable(located, name)
    work = args.work_dir.absolute()
    require(not work.exists() or (work.is_dir() and not any(work.iterdir())),
            "--work-dir must be absent or empty; evidence may not overwrite a prior run")
    work.mkdir(parents=True, exist_ok=True)
    require(len(os.fsencode(str(work / "daemon.sock"))) < 104,
            "work-dir is too long for a Unix socket; choose a short absolute path")
    for directory in ("source", "outputs", "receipts", "daemon-receipts", "tools"):
        (work / directory).mkdir()
    commands = []
    evidence = {"schema": "mantle-cc-cache-slots-proof-v1", "status": "running",
                "claim_class": "bounded-machine-local-observation",
                "cache_receipts_are_proof": False, "commands": commands, "cases": []}
    daemon = None
    daemon_stdout = None
    daemon_stderr = None

    def run(label, argv, env=None, success=True):
        started = time.monotonic_ns()
        result = subprocess.run([str(item) for item in argv], cwd=work, env=env,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                timeout=180, check=False)
        elapsed_ns = time.monotonic_ns() - started
        (work / f"{label}.stdout").write_bytes(result.stdout)
        (work / f"{label}.stderr").write_bytes(result.stderr)
        commands.append({"label": label, "argv": [str(item) for item in argv],
                         "exit_status": result.returncode, "elapsed_ns": elapsed_ns})
        if success:
            require(result.returncode == 0,
                    f"{label}: exit {result.returncode}; see {work / (label + '.stderr')}")
        return result, elapsed_ns

    def digest(path, label):
        result, _ = run("digest-" + label, [utilities["b3sum"], path])
        value = result.stdout.decode().split()[0]
        require(re.fullmatch("[0-9a-f]{64}", value), f"invalid BLAKE3 from b3sum: {path}")
        return value

    try:
        # GCC helpers are observed from the selected compiler, never guessed.
        tool_files = {compiler, cxx}
        for index, selected in enumerate((compiler, cxx)):
            helper_name = "cc1" if index == 0 else "cc1plus"
            helper, _ = run(f"query-{helper_name}", [selected, f"-print-prog-name={helper_name}"])
            tool_files.add(executable(helper.stdout.decode().strip(), helper_name))
            assembler, _ = run(f"query-as-{index}", [selected, "-print-prog-name=as"])
            assembler_value = assembler.stdout.decode().strip()
            assembler_path = assembler_value if Path(assembler_value).is_absolute() else shutil.which(assembler_value)
            require(assembler_path is not None, "compiler assembler is unavailable")
            assembler_path = executable(assembler_path, "assembler")
            destination = work / "tools" / "as"
            if destination.exists():
                require(destination.read_bytes() == assembler_path.read_bytes(),
                        "C and C++ compilers require different assemblers; use a matched toolchain")
            else:
                shutil.copy2(assembler_path, destination)
            tool_files.add(assembler_path)
        # Bind dynamic runtimes as content inputs too. Missing runtimes fail in
        # the real process; ldd's static-executable nonzero status is accepted
        # only when its diagnostic explicitly identifies a static executable.
        runtimes = set()
        for index, tool in enumerate(sorted(tool_files)):
            result, _ = run(f"runtime-{index}", [utilities["ldd"], tool], success=False)
            text = (result.stdout + result.stderr).decode(errors="replace")
            require("not found" not in text, f"missing dynamic runtime for {tool}")
            require(result.returncode == 0 or "not a dynamic executable" in text or "statically linked" in text,
                    f"cannot establish dynamic runtime closure for {tool}")
            for found in re.findall(r"(?:=>\s*)?(/[^\s()]+)", text):
                runtime = Path(found).resolve()
                require(runtime.is_file(), f"unreadable compiler runtime: {runtime}")
                runtimes.add(runtime)
        tool_files.update(runtimes)
        roots = [{"alias": "source", "path": str(work / "source"), "path_independent": True},
                 {"alias": "assembler-path", "path": str(work / "tools"), "path_independent": True}]
        for index, path in enumerate(sorted(tool_files)):
            roots.append({"alias": f"tool-{index}", "path": str(path), "path_independent": True})
        for index, path in enumerate(args.toolchain_root):
            require(path.is_absolute() and path.exists(), f"missing absolute toolchain root: {path}")
            roots.append({"alias": f"extra-{index}", "path": str(path.resolve()), "path_independent": True})
        require(len(roots) <= 64, "toolchain closure exceeds driver root limit")
        # No host PATH/HOME/TMPDIR is inherited by compilation. The runtime-only
        # socket and build ID are excluded from the exact admitted environment.
        admitted_env = {"PATH": str(work / "tools"), "LC_ALL": "C", "TZ": "UTC"}
        process_env = dict(admitted_env)
        process_env.update({"MANTLE_CC_CACHE_SOCKET": str(work / "daemon.sock"),
                            "MANTLE_COMPILE_BUILD_ID": "cc-cache-proof"})
        source = work / "source"
        (source / "a.h").write_text("#define HEADER_VALUE 17\n")
        (source / "a.c").write_text('#include "a.h"\nint answer(void) { return HEADER_VALUE + OPTION_VALUE; }\nconst char *source_name = __FILE__;\n')
        (source / "a.cpp").write_text('#include "a.h"\ntemplate<int N> struct Value { static int get() { return N; } };\nint cpp_answer() { return Value<HEADER_VALUE>::get(); }\n')
        probe = work / "probe.sh"
        probe.write_text("configure probe identity one\n")
        policy = {"schema": "mantle-cc-driver-policy-v1", "compiler": str(compiler),
                  "roots": roots, "platform": f"{platform.machine()}-{sys.platform}",
                  "environment": admitted_env, "probe_script": str(probe),
                  "cache_enabled": True, "slots_enabled": True,
                  "output_path_independent": True, "elapsed_ms_max": 120000,
                  "slot_wait_ms_max": 30000, "receipt_dir": str(work / "receipts")}
        wrapper_policy = {
            "schema": "mantle-rustc-wrapper-policy-v1", "policy_id": "cc-cache-proof",
            "cache_mode": "local-read-write", "failure_mode": "fail-open",
            "socket_path": str(work / "daemon.sock"), "allowed_peer_uids": [os.getuid()],
            "readable_roots": sorted({str(work)} | {str(path) for path in tool_files}),
            "writable_roots": [str(work / "outputs")], "allowed_environment": sorted(admitted_env),
            "effects": {"filesystem": "declared-roots", "network": "deny", "clock": "allow", "randomness": "allow"},
            "limits": {"elapsed_millis": 120000, "stdout_bytes": 1048576,
                       "stderr_bytes": 1048576, "artifact_bytes": 8589934592},
            "max_request_bytes": 8388608, "max_response_bytes": 8388608,
            "max_concurrency": 8, "sandbox_program": str(utilities["bwrap"]),
            "sandbox_program_blake3": digest(utilities["bwrap"], "bwrap"),
            "result_sources": [], "shared_trust_policy": None,
            "publication": {"enabled": False, "source_id": None, "signer_name": None, "signing_key_path": None},
            "local_reads_enabled": True, "local_writes_enabled": True,
            "shared_reads_enabled": False, "shared_writes_enabled": False,
            "redact_environment_values": True,
        }
        write_json(work / "daemon-policy.json", wrapper_policy)
        write_json(work / "compile-slot-policy.json", {
            "schema": "mantle-compile-slot-policy-v1", "total_slots": 2,
            "per_build_max": 1, "total_queue": 16, "per_build_queue": 8,
            "wait_timeout_ms": 30000, "fairness": "round_robin"})
        daemon_argv = [daemon_binary, "--policy", work / "daemon-policy.json",
                       "--state-dir", work / "state", "--store-backend", "snix",
                       "--store-output-dir", work / "store", "--receipt-dir", work / "daemon-receipts"]
        daemon_stdout = (work / "daemon.stdout").open("wb")
        daemon_stderr = (work / "daemon.stderr").open("wb")
        daemon = subprocess.Popen([str(item) for item in daemon_argv], cwd=work,
                                  stdout=daemon_stdout, stderr=daemon_stderr)
        commands.append({"label": "daemon", "argv": [str(item) for item in daemon_argv], "pid": daemon.pid})
        deadline = time.monotonic() + 30
        while not (work / "daemon.sock").exists():
            require(daemon.poll() is None, "daemon exited before readiness; see daemon.stderr")
            require(time.monotonic() < deadline, "daemon socket readiness timed out")
            time.sleep(0.02)
        # A successful connect verifies the socket rather than a stale path.
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.connect(str(work / "daemon.sock"))

        def compile_case(label, selected_policy=None, root=None, extra=None,
                         expected="cache-miss", cpp=False):
            selected_policy = copy.deepcopy(selected_policy or policy)
            root = root or source
            output = work / "outputs" / f"{label}.o"
            source_path = root / ("a.cpp" if cpp else "a.c")
            invocation = ["-x", "c++" if cpp else "c", "-c", str(source_path), "-o", str(output), "-O2", "-g0",
                          "-nostdinc", "-DOPTION_VALUE=3", f"-ffile-prefix-map={root}=src"]
            if cpp:
                invocation.append("-nostdinc++")
            invocation.extend(extra or [])
            policy_path = work / f"{label}.policy.json"
            write_json(policy_path, selected_policy)
            before = set((work / "receipts").iterdir())
            result, elapsed = run(label, [driver, "--policy", policy_path, "--", *invocation], process_env)
            added = set((work / "receipts").iterdir()) - before
            require(len(added) == 1, f"{label}: expected exactly one real driver receipt, found {len(added)}")
            receipt_path = added.pop()
            receipt = json.loads(receipt_path.read_text())
            require(receipt["cache_disposition"] == expected,
                    f"{label}: expected {expected}, observed {receipt['cache_disposition']}")
            require(output.is_file(), f"{label}: compiler produced no object")
            row = {"label": label, "object_blake3": digest(output, label),
                   "elapsed_ns": elapsed, "receipt": str(receipt_path), "observed": receipt,
                   "stdout_blake3": digest(work / f"{label}.stdout", label + "-stdout"),
                   "stderr_blake3": digest(work / f"{label}.stderr", label + "-stderr")}
            evidence["cases"].append(row)
            return row, output

        cold, cold_path = compile_case("c-cold")
        warm, warm_path = compile_case("c-warm", expected="object-hit")
        off_policy = copy.deepcopy(policy)
        off_policy["cache_enabled"] = False
        off, off_path = compile_case("c-off", selected_policy=off_policy, expected="cache-off")
        require(cold_path.read_bytes() == warm_path.read_bytes() == off_path.read_bytes(),
                "cold/warm/cache-off objects differ")
        require((cold_path.stat().st_mode & 0o777) == (warm_path.stat().st_mode & 0o777) == (off_path.stat().st_mode & 0o777),
                "cold/warm/cache-off output permissions differ")
        require(cold["stdout_blake3"] == warm["stdout_blake3"] == off["stdout_blake3"] and
                cold["stderr_blake3"] == warm["stderr_blake3"] == off["stderr_blake3"],
                "cold/warm/cache-off compiler streams differ")
        evidence["timing_ns"] = {"cold_cached_compile": cold["elapsed_ns"],
                                  "warm_cached_lookup": warm["elapsed_ns"],
                                  "cache_off_compile": off["elapsed_ns"]}

        # Copying source inputs to a different physical root preserves their
        # content identity; root aliases and prefix-map destinations stay fixed.
        relocated = work / "relocated"
        shutil.copytree(source, relocated)
        relocated_policy = copy.deepcopy(policy)
        relocated_policy["roots"][0]["path"] = str(relocated)
        moved, moved_path = compile_case("relocated-identical", relocated_policy, relocated, expected="object-hit")
        require(moved_path.read_bytes() == cold_path.read_bytes(), "relocated hit bytes differ")
        require(moved["observed"]["key"] == warm["observed"]["key"], "relocated hit changed semantic key")

        source_bytes = (source / "a.c").read_bytes()
        (source / "a.c").write_bytes(source_bytes.replace(b"HEADER_VALUE + OPTION_VALUE", b"HEADER_VALUE + OPTION_VALUE + 1"))
        changed_source, changed_path = compile_case("changed-source")
        require(changed_path.read_bytes() != cold_path.read_bytes(), "source negative control did not change object")
        (source / "a.c").write_bytes(source_bytes)
        header_bytes = (source / "a.h").read_bytes()
        (source / "a.h").write_text("#define HEADER_VALUE 23\n")
        _, changed_header_path = compile_case("changed-header")
        require(changed_header_path.read_bytes() != cold_path.read_bytes(), "header negative control did not change object")
        (source / "a.h").write_bytes(header_bytes)
        _, changed_option_path = compile_case("changed-options", extra=["-UOPTION_VALUE", "-DOPTION_VALUE=9"])
        require(changed_option_path.read_bytes() != cold_path.read_bytes(), "option negative control did not change object")
        # Different real compiler bytes are a tool-identity negative control,
        # not a source change; force C language for g++ on the same .c fixture.
        require(compiler.read_bytes() != cxx.read_bytes(), "tool negative control needs distinct compiler and C++ driver bytes")
        tool_policy = copy.deepcopy(policy)
        tool_policy["compiler"] = str(cxx)
        compile_case("changed-tool", tool_policy)
        probe.write_text("configure probe identity two\n")
        compile_case("changed-probe")
        compile_case("changed-probe-warm", expected="object-hit")
        probe.write_text("configure probe identity one\n")

        cpp_policy = copy.deepcopy(policy)
        cpp_policy["compiler"] = str(cxx)
        _, cpp_cold = compile_case("cpp-cold", cpp_policy, cpp=True)
        _, cpp_warm = compile_case("cpp-warm", cpp_policy, cpp=True, expected="object-hit")
        cpp_off = copy.deepcopy(cpp_policy)
        cpp_off["cache_enabled"] = False
        _, cpp_plain = compile_case("cpp-off", cpp_off, cpp=True, expected="cache-off")
        require(cpp_cold.read_bytes() == cpp_warm.read_bytes() == cpp_plain.read_bytes(),
                "C++ cold/warm/cache-off objects differ")

        # Unknown preprocess-only action must preserve argv and output exactly.
        forward_policy_path = work / "forward.policy.json"
        write_json(forward_policy_path, policy)
        preprocess = ["-E", "-nostdinc", "-DOPTION_VALUE=3", str(source / "a.c")]
        before = set((work / "receipts").iterdir())
        forwarded, _ = run("unknown-forward", [driver, "--policy", forward_policy_path, "--", *preprocess], process_env)
        direct, _ = run("unknown-direct", [compiler, *preprocess], admitted_env)
        require((forwarded.stdout, forwarded.stderr, forwarded.returncode) ==
                (direct.stdout, direct.stderr, direct.returncode), "unknown invocation was not forwarded unchanged")
        added = set((work / "receipts").iterdir()) - before
        require(len(added) == 1, "forward invocation lacks exactly one receipt")
        forward_receipt = json.loads(added.pop().read_text())
        require(forward_receipt["cache_disposition"] == "forward-unchanged", "unknown invocation did not record forward")
        evidence["forward_receipt"] = forward_receipt

        # The real build sandbox has a private PID namespace. Its child PID
        # must not be mistaken for a host PID by the slot daemon.
        namespace_receipts = work / "namespace-receipts"
        namespace_receipts.mkdir()
        namespace_policy = copy.deepcopy(off_policy)
        namespace_policy["receipt_dir"] = str(namespace_receipts)
        namespace_policy_path = work / "namespace.policy.json"
        write_json(namespace_policy_path, namespace_policy)
        namespaced, _ = run("pid-namespace-slot", [utilities["bwrap"], "--die-with-parent", "--unshare-pid", "--new-session",
                               "--ro-bind", "/", "/", "--dev-bind", "/dev", "/dev", "--proc", "/proc",
                               "--bind", work, work, "--", driver, "--policy", namespace_policy_path, "--", *preprocess], process_env)
        require((namespaced.returncode, namespaced.stdout, namespaced.stderr) ==
                (direct.returncode, direct.stdout, direct.stderr), "PID namespace changed actual compiler output")
        namespace_rows = list(namespace_receipts.glob("*.json"))
        require(len(namespace_rows) == 1, "PID namespace driver did not emit exactly one receipt")
        namespace_receipt = json.loads(namespace_rows[0].read_text())
        require(namespace_receipt["slot_disposition"] == "granted", "PID namespace lease degraded instead of owning real child")
        evidence["pid_namespace_slot"] = namespace_receipt

        # A compile-mode configure probe with a syntax error still emits a
        # complete depfile. Missing-header/preprocess/link probes are deliberately
        # not promoted to this supported failure-replay mode.
        (source / "failure.c").write_text('#include "a.h"\nint broken = ;\n')
        failure_output = work / "outputs" / "failure.o"
        failure_args = ["-x", "c", "-c", str(source / "failure.c"), "-o", str(failure_output),
                        "-nostdinc", "-g0", f"-ffile-prefix-map={source}=src"]
        failure_policy = work / "failure.policy.json"
        write_json(failure_policy, policy)

        def failure_case(label, expected):
            before = set((work / "receipts").iterdir())
            result, _ = run(label, [driver, "--policy", failure_policy, "--", *failure_args], process_env, success=False)
            require(result.returncode != 0 and not failure_output.exists(), f"{label}: expected a no-output compiler failure")
            added = set((work / "receipts").iterdir()) - before
            require(len(added) == 1, f"{label}: missing failure disposition")
            receipt = json.loads(added.pop().read_text())
            require(receipt["cache_disposition"] == expected, f"{label}: {receipt}")
            evidence["cases"].append({"label": label, "observed": receipt})
            return result

        failed_cold = failure_case("failure-cold", "cache-miss")
        failed_warm = failure_case("failure-warm", "failure-hit")
        failed_direct, _ = run("failure-direct", [compiler, *failure_args], admitted_env, success=False)
        require((failed_cold.returncode, failed_cold.stdout, failed_cold.stderr) ==
                (failed_warm.returncode, failed_warm.stdout, failed_warm.stderr) ==
                (failed_direct.returncode, failed_direct.stdout, failed_direct.stderr),
                "failure replay changed compiler status or streams")
        probe.write_text("configure probe identity changed for failure\n")
        failure_case("failure-changed-probe", "cache-miss")
        probe.write_text("configure probe identity one\n")
        (source / "failure.c").unlink()

        # Volatile builtins in learned headers must never publish reusable
        # objects. Advancing the wall clock is not the assertion: every real
        # invocation must compile instead of returning an object-hit.
        (source / "a.h").write_bytes(header_bytes + b'const char *compile_time = __TIME__;\n')
        compile_case("volatile-header-cold", expected="publish-unavailable")
        compile_case("volatile-header-repeat", expected="publish-unavailable")
        (source / "a.h").write_bytes(header_bytes)

        # Receipt storage is advisory even after a real compiler exits.
        bad_receipt_path = work / "not-a-receipt-directory"
        bad_receipt_path.write_text("occupied\n")
        bad_receipt_policy = copy.deepcopy(policy)
        bad_receipt_policy["receipt_dir"] = str(bad_receipt_path)
        bad_receipt_policy_path = work / "bad-receipt.policy.json"
        write_json(bad_receipt_policy_path, bad_receipt_policy)
        result, _ = run("receipt-storage-failure", [driver, "--policy", bad_receipt_policy_path, "--",
                         "-x", "c", "-c", source / "a.c", "-o", work / "outputs" / "bad-receipt.o",
                         "-O2", "-g0", "-nostdinc", "-DOPTION_VALUE=3", "-DADVISORY_PROOF=1", f"-ffile-prefix-map={source}=src"], process_env)
        require((work / "outputs" / "bad-receipt.o").read_bytes() == cold_path.read_bytes(),
                "advisory receipt failure changed real compiler output")

        def scheduling_receipts():
            rows = []
            for line in (work / "daemon.stderr").read_text(errors="replace").splitlines():
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if isinstance(row, dict) and row.get("schema") == "mantle-compile-slot-receipt-v1":
                    rows.append(row)
            return rows

        # Real GCC/Clang preprocessing is held on FIFOs, not a fake compiler or
        # duration guess. Opening the writer proves that the actual compiler
        # reached its input. Held readers expose both total and per-build bounds.
        gates = work / "contention"
        gates.mkdir()
        contenders = {}
        observations = []
        active = set()

        def launch(name, build):
            directory = gates / name
            directory.mkdir()
            fifo = directory / "input.c"
            os.mkfifo(fifo)
            selected = copy.deepcopy(off_policy)
            selected["receipt_dir"] = str(directory / "receipts")
            selected_path = directory / "policy.json"
            write_json(selected_path, selected)
            environment = dict(process_env, MANTLE_COMPILE_BUILD_ID=build)
            argv = [str(driver), "--policy", str(selected_path), "--", "-E", "-P", "-x", "c", str(fifo)]
            process = subprocess.Popen(argv, cwd=work, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            contenders[name] = {"process": process, "build": build, "fifo": fifo, "writer": None, "directory": directory}
            commands.append({"label": "contention-" + name, "argv": argv, "pid": process.pid})

        def try_gate(name):
            contender = contenders[name]
            if contender["writer"] is not None:
                return True
            try:
                contender["writer"] = os.open(contender["fifo"], os.O_WRONLY | os.O_NONBLOCK)
                return True
            except OSError as error:
                if error.errno != errno.ENXIO:
                    raise
                return False

        def await_gate(name):
            deadline = time.monotonic() + 15
            while not try_gate(name):
                require(contenders[name]["process"].poll() is None, f"{name}: driver exited before real compiler input")
                require(time.monotonic() < deadline, f"{name}: fair grant did not arrive within bound")
                time.sleep(0.01)
            active.add(name)
            require(len(active) <= 2, "actual compiler concurrency exceeded total_slots")
            builds = [contenders[item]["build"] for item in active]
            require(len(builds) == len(set(builds)), "actual compilers exceeded per_build_max")
            observations.append({"event": "compiler-input-open", "name": name, "active": sorted(active), "at_ns": time.monotonic_ns()})

        def await_queued(build, count):
            deadline = time.monotonic() + 15
            while sum(row["build_id"] == build and row["decision"] == "queued" for row in scheduling_receipts()) < count:
                require(time.monotonic() < deadline, f"{build}: request did not reach bounded authority queue")
                time.sleep(0.01)

        def blocked(*names):
            for name in names:
                require(not try_gate(name), f"{name}: compiler started while scheduling bound forbade it")

        def finish(name):
            contender = contenders[name]
            expected = f"int slot_{name} = 17;\n".encode()
            os.write(contender["writer"], expected)
            os.close(contender["writer"])
            contender["writer"] = None
            stdout, stderr = contender["process"].communicate(timeout=15)
            (contender["directory"] / "stdout").write_bytes(stdout)
            (contender["directory"] / "stderr").write_bytes(stderr)
            require(contender["process"].returncode == 0 and stdout == expected and stderr == b"",
                    f"{name}: real preprocessor output/status changed")
            receipts = list((contender["directory"] / "receipts").glob("*.json"))
            require(len(receipts) == 1, f"{name}: missing driver scheduling disposition")
            observed = json.loads(receipts[0].read_text())
            require(observed["slot_disposition"] == "granted", f"{name}: degraded run cannot prove concurrency bounds")
            active.remove(name)
            observations.append({"event": "compiler-exit", "name": name, "active": sorted(active), "at_ns": time.monotonic_ns()})

        try:
            launch("a0", "contention-large")
            await_gate("a0")
            launch("a1", "contention-large")
            await_queued("contention-large", 2)
            blocked("a1")  # One slot is free, but this build is at its bound.
            launch("a2", "contention-large")
            await_queued("contention-large", 3)
            launch("b0", "contention-other")
            await_gate("b0")
            launch("c0", "contention-small")
            await_queued("contention-small", 1)
            blocked("a1", "a2", "c0")
            finish("b0")
            await_gate("c0")  # Backlogged large build must not starve this one.
            blocked("a1", "a2")
            finish("c0")
            blocked("a1", "a2")
            finish("a0")
            await_gate("a1")
            blocked("a2")
            finish("a1")
            await_gate("a2")
            finish("a2")
        finally:
            for contender in contenders.values():
                if contender["writer"] is not None:
                    os.close(contender["writer"])
                if contender["process"].poll() is None:
                    contender["process"].terminate()
                    contender["process"].communicate(timeout=15)
        concurrent_receipts = [row for row in scheduling_receipts() if row["build_id"].startswith("contention-")]
        grants = [row for row in concurrent_receipts if row["decision"] == "granted"]
        releases = [row for row in concurrent_receipts if row["decision"] == "released"]
        require(len(grants) == len(releases) == 5, "contention leases were not all granted and released")
        require({row["request_id"] for row in grants} == {row["request_id"] for row in releases}, "contention lease leak")
        require(all(row["waited_ms"] < row["wait_timeout_ms"] and row["fairness"] == "round_robin" for row in grants),
                "contention grant exceeded declared fairness/wait bounds")
        evidence["contention"] = {"observations": list(observations), "receipts": concurrent_receipts,
                                   "observed_peak_live_compilers": 2, "per_build_peak": 1,
                                   "scope": "cooperative compiler process groups; real FIFO-gated preprocessor commands"}

        # A full real Unix accept backlog must not strand an otherwise valid
        # compile in connect(). No daemon responses or compiler mocks are used.
        backlog_path = work / "backlog.sock"
        pending_connections = []
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as backlog:
            backlog.bind(str(backlog_path))
            backlog.listen(1)
            try:
                saturated = False
                for _ in range(32):
                    pending = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                    pending.setblocking(False)
                    outcome = pending.connect_ex(str(backlog_path))
                    pending_connections.append(pending)
                    if outcome != 0:
                        require(outcome in (errno.EAGAIN, errno.EWOULDBLOCK),
                                f"unexpected backlog saturation result: {outcome}")
                        saturated = True
                        break
                require(saturated, "could not establish a saturated Unix backlog")
                process_env["MANTLE_CC_CACHE_SOCKET"] = str(backlog_path)
                backlog_case, backlog_output = compile_case("saturated-backlog", expected="daemon-unavailable")
                require(backlog_output.read_bytes() == cold_path.read_bytes(),
                        "backlog degradation changed actual compiler bytes")
                require(backlog_case["observed"]["slot_disposition"] == "degraded-no-grant",
                        "backlogged authority did not report advisory slot degradation")
                evidence["backlog_degradation"] = backlog_case
            finally:
                process_env["MANTLE_CC_CACHE_SOCKET"] = str(work / "daemon.sock")
                for pending in pending_connections:
                    pending.close()

        # Kill the authority while a real compiler owns a grant and is blocked
        # on its source FIFO. The compiler must complete normally after death;
        # subsequent invocations must compile without an authority.
        launch("authoritydeath", "authority-death")
        try:
            await_gate("authoritydeath")
            daemon.kill()
            daemon_status = daemon.wait(timeout=30)
            daemon = None
            finish("authoritydeath")
            evidence["authority_death"] = {
                "daemon_exit_status": daemon_status,
                "compiler_completed_after_authority_exit": True,
                "observation": observations[-1],
                "scope": "one real cooperative compiler group; no restart reclamation claim"}
        finally:
            contender = contenders["authoritydeath"]
            if contender["writer"] is not None:
                os.close(contender["writer"])
            if contender["process"].poll() is None:
                contender["process"].terminate()
                contender["process"].communicate(timeout=15)
        _, degraded_path = compile_case("daemon-loss", expected="daemon-unavailable")
        require(degraded_path.read_bytes() == cold_path.read_bytes(), "daemon-loss plain compilation changed bytes")
        # Scheduling receipts are observations, not independent compiler proofs.
        daemon_stderr.flush()
        scheduling = []
        for line in (work / "daemon.stderr").read_text(errors="replace").splitlines():
            try:
                value = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(value, dict) and value.get("schema") == "mantle-compile-slot-receipt-v1":
                scheduling.append(value)
        for path in sorted((work / "daemon-receipts").glob("*.json")):
            value = json.loads(path.read_text())
            if value.get("schema") == "mantle-compile-slot-receipt-v1":
                scheduling.append(value)
        evidence["scheduling"] = {
            "receipts": scheduling,
            "observed_peak_active_slots": max((row["active_slots"] for row in scheduling), default=None),
            "concurrency_fairness_claim": True,
            "scope": "bounded FIFO-gated real compiler fixture; not build/provenance evidence"}
        require(any(row["observed"]["slot_disposition"] == "granted" for row in evidence["cases"]),
                "no actual compiler invocation acquired a machine-wide slot")
        evidence["status"] = "passed"
    except Exception as error:
        evidence["status"] = "failed"
        evidence["error"] = str(error)
        raise
    finally:
        if daemon is not None:
            daemon.terminate()
            try:
                daemon.wait(timeout=10)
            except subprocess.TimeoutExpired:
                daemon.kill()
                daemon.wait(timeout=10)
        if daemon_stdout is not None:
            daemon_stdout.close()
        if daemon_stderr is not None:
            daemon_stderr.close()
        write_json(work / "evidence.json", evidence)
    print(work / "evidence.json")


if __name__ == "__main__":
    main()
