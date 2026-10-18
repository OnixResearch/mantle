#!/usr/bin/env python3
"""Observe native Darwin process-tree and memory enforcement gaps.

A mechanism experiment only: never enables a Mantle command or admits output.
The bounded C child always exits or alarms after five seconds and is reaped.
"""
import argparse
import importlib.util
import json
import pathlib
import platform
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True, type=pathlib.Path)
    parser.add_argument("--source-blake3", required=True,
                        help="operator-computed BLAKE3 binding of process-boundaries.c")
    args = parser.parse_args()
    if len(args.source_blake3) != 64 or any(c not in "0123456789abcdef" for c in args.source_blake3):
        parser.error("source BLAKE3 must be 64 lowercase hexadecimal characters")
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise SystemExit("requires real native Darwin arm64")
    root = pathlib.Path(__file__).resolve().parent.parent
    spec = importlib.util.spec_from_file_location("darwin_probe", root / "scripts/probe-darwin-isolation.py")
    probe = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(probe)
    source = root / "tests/fixtures/darwin-native/process-boundaries.c"
    report = {"schema": "mantle-darwin-process-boundaries-probe-v1",
              "os_build": probe.invoke(["/usr/bin/sw_vers", "-buildVersion"]),
              "operator_supplied_source_blake3": args.source_blake3,
              "non_claims": ["not Mantle acceptance", "not a cleanup implementation",
                             "successful polling is not race-free tree containment"]}
    with tempfile.TemporaryDirectory(prefix="mantle-darwin-process-") as temporary:
        work = pathlib.Path(temporary).resolve()
        binary = work / "boundaries"
        report["compile"] = probe.invoke(["/Library/Developer/CommandLineTools/usr/bin/clang",
                                           "-Wall", "-Wextra", "-Werror", "-arch", "arm64", str(source), "-o", str(binary)])
        if report["compile"]["status"] == 0:
            profile = '(version 1)\n(deny default)\n(allow process-exec)\n(allow process-fork)\n(allow sysctl-read)\n'
            profile += '(allow file-read* (subpath "/usr/lib") (subpath "/System/Library") (literal "/dev/null") (literal '
            profile += probe.quote(binary) + '))\n'
            profile += '(allow file-write* (literal "/dev/null"))\n'
            profile_path = work / "subprocess.sb"
            profile_path.write_text(profile)
            report["subprocess_profile"] = profile
            env = {"PATH": "/path-not-set", "HOME": "/homeless-shelter", "LANG": "C", "LC_ALL": "C"}
            report["sandboxed_setsid_escape"] = probe.invoke(
                ["/usr/bin/sandbox-exec", "-f", str(profile_path), str(binary), "escape"],
                cwd=work, env=env, preexec_fn=probe.limits)
            # Kqueue tracking and memory limits are inspected directly on the
            # host, not silently used instead of a failed sandbox execution.
            report["native_note_track"] = probe.invoke([str(binary), "track"], cwd=work, env=env, preexec_fn=probe.limits)
            report["native_rlimit_as"] = probe.invoke([str(binary), "memory"], cwd=work, env=env, preexec_fn=probe.limits)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    raise SystemExit(0 if report["compile"]["status"] == 0 else 1)


if __name__ == "__main__":
    main()
