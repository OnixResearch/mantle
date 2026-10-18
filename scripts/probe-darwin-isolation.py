#!/usr/bin/env python3
"""Native mechanism experiment, not Mantle store/executor acceptance evidence.

Run on an operator-owned native arm64 Darwin runner. No downloads, SDK license
acceptance, store publication, or supported-command changes are performed.
"""
import argparse
import json
import os
import pathlib
import platform
import resource
import signal
import subprocess
import tempfile
import time


def quote(value):
    return '"' + str(value).replace('\\', '\\\\').replace('"', '\\"') + '"'


def invoke(command, **kwargs):
    result = subprocess.run(command, capture_output=True, timeout=30, **kwargs)
    return {"status": result.returncode,
            "stdout": result.stdout.decode(errors="replace")[:8192],
            "stderr": result.stderr.decode(errors="replace")[:8192]}


def limits():
    os.setsid()
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_CPU, (2, 2))
    resource.setrlimit(resource.RLIMIT_FSIZE, (1048576, 1048576))
    resource.setrlimit(resource.RLIMIT_NOFILE, (64, 64))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True, type=pathlib.Path)
    parser.add_argument("--source-blake3", required=True,
                        help="operator-computed BLAKE3 of the copied probe.c; not locally verified")
    parser.add_argument("--sdk-settings-blake3", required=True,
                        help="operator-computed BLAKE3 of native SDKSettings.json; not SDK closure identity")
    args = parser.parse_args()
    for digest in [args.source_blake3, args.sdk_settings_blake3]:
        if len(digest) != 64 or any(byte not in "0123456789abcdef" for byte in digest):
            parser.error("BLAKE3 bindings must be 64 lowercase hexadecimal characters")
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise SystemExit("requires real native Darwin arm64; no emulation/fallback")
    sdk = pathlib.Path("/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk").resolve(strict=True)
    clang = pathlib.Path("/Library/Developer/CommandLineTools/usr/bin/clang")
    source = pathlib.Path(__file__).resolve().parent.parent / "tests/fixtures/darwin-native/probe.c"
    report = {"schema": "mantle-darwin-mechanism-probe-v1", "host": platform.platform(),
              "host_arch": platform.machine(), "target": "aarch64-darwin",
              "os_build": invoke(["/usr/bin/sw_vers", "-buildVersion"]),
              "sdk_path": str(sdk), "operator_supplied_sdk_settings_blake3": args.sdk_settings_blake3,
              "operator_supplied_source_blake3": args.source_blake3,
              "compiler_version": invoke([str(clang), "--version"]), "cases": {},
              "non_claims": ["not Mantle build/shell/run acceptance", "not SDK redistribution authorization",
                             "not content-authoritative SDK closure digest", "not a process-tree or memory quota proof"]}
    with tempfile.TemporaryDirectory(prefix="mantle-darwin-probe-") as temporary:
        root = pathlib.Path(temporary).resolve()
        binary = root / "probe"
        report["fixture_compile"] = invoke([str(clang), "-Wall", "-Wextra", "-Werror", "-isysroot", str(sdk),
                                            "-arch", "arm64", str(source), "-o", str(binary)])
        if report["fixture_compile"]["status"] != 0:
            args.report.write_text(json.dumps(report, indent=2) + "\n")
            raise SystemExit("native fixture compilation failed; see report")
        allowed = root / "allowed"
        allowed.mkdir()
        secret = root / "secret"
        secret.write_text("undeclared-input\n")
        denied_write = root / "denied-write"
        # No broad /usr, /Library, home, /private/tmp or network allowances.
        profile = '(version 1)\n(deny default)\n(allow process-exec)\n(allow sysctl-read)\n'
        profile += '(allow file-read* (subpath "/usr/lib") (subpath "/System/Library") (literal "/dev/null")'
        profile += ' (literal "/bin/sh") (literal ' + quote(binary) + ') (subpath ' + quote(allowed) + '))\n'
        profile += '(allow file-write* (subpath ' + quote(allowed) + ') (literal "/dev/null"))\n'
        profile_path = root / "isolation.sb"
        profile_path.write_text(profile)
        report["profile"] = profile
        env = {"PATH": "/path-not-set", "HOME": "/homeless-shelter", "LANG": "C", "LC_ALL": "C"}
        def sandbox(command):
            return invoke(["/usr/bin/sandbox-exec", "-f", str(profile_path)] + command,
                          cwd=allowed, env=env, preexec_fn=limits)
        cases = report["cases"]
        cases["native_run"] = sandbox([str(binary), "hello"])
        cases["shell_builtins"] = sandbox(["/bin/sh", "-c", "printf 'mantle-shell-native\\n'"])
        cases["shell_exec"] = sandbox(["/bin/sh", "-c", 'exec "$1" hello', "probe", str(binary)])
        cases["undeclared_read"] = sandbox([str(binary), "read", str(secret)])
        cases["undeclared_write"] = sandbox([str(binary), "write", str(denied_write)])
        cases["network"] = sandbox([str(binary), "network"])
        cases["process_fork"] = sandbox([str(binary), "fork"])
        cases["declared_write"] = sandbox([str(binary), "write", str(allowed / "output")])
        cases["failure"] = sandbox([str(binary), "fail"])
        cases["recovery"] = sandbox([str(binary), "hello"])
        cases["cpu_limit"] = sandbox([str(binary), "cpu"])
        child = subprocess.Popen(["/usr/bin/sandbox-exec", "-f", str(profile_path), str(binary), "sleep"],
                                 cwd=allowed, env=env, preexec_fn=limits,
                                 stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        time.sleep(0.25)
        alive_before_cancel = child.poll() is None
        if alive_before_cancel:
            child.kill()
        child.wait(timeout=5)
        try:
            os.kill(child.pid, 0)
            reaped = False
        except ProcessLookupError:
            reaped = True
        cases["cancellation"] = {"alive_before_cancel": alive_before_cancel, "status": child.returncode, "reaped": reaped}
        report["denied_write_absent"] = not denied_write.exists()
        report["declared_output_bytes"] = (allowed / "output").read_text() if (allowed / "output").exists() else None
        report["mechanism_pass"] = all(cases[name]["status"] == 77 for name in
                                      ["undeclared_read", "undeclared_write", "network", "process_fork"])
        report["mechanism_pass"] &= all(cases[name]["status"] == 0 for name in
                                       ["native_run", "shell_builtins", "shell_exec", "declared_write", "recovery"])
        report["mechanism_pass"] &= cases["failure"]["status"] == 42
        report["mechanism_pass"] &= cases["cpu_limit"]["status"] in [-signal.SIGKILL, -signal.SIGXCPU]
        report["mechanism_pass"] &= alive_before_cancel and reaped and child.returncode == -signal.SIGKILL
        report["mechanism_pass"] &= report["denied_write_absent"] and report["declared_output_bytes"] == "native-output\n"
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    raise SystemExit(0 if report["mechanism_pass"] else 1)


if __name__ == "__main__":
    main()
