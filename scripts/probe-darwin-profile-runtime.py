#!/usr/bin/env python3
"""Diagnose deny-default Darwin runtime permissions; no capability admission.

Broad file metadata/Mach variants are diagnostic only and never fallback.
Each case must launch the real fixture before any denial is interpreted.
"""
import argparse
import importlib.util
import json
import pathlib
import platform
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-blake3", required=True)
    parser.add_argument("--report", required=True, type=pathlib.Path)
    args = parser.parse_args()
    if len(args.source_blake3) != 64 or any(c not in "0123456789abcdef" for c in args.source_blake3):
        parser.error("operator source binding must be 64 lowercase hexadecimal BLAKE3")
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise SystemExit("requires native Darwin arm64")
    repo = pathlib.Path(__file__).resolve().parent.parent
    spec = importlib.util.spec_from_file_location("darwin_probe", repo / "scripts/probe-darwin-isolation.py")
    probe = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(probe)
    source = repo / "tests/fixtures/darwin-native/probe.c"
    report = {"schema": "mantle-darwin-profile-runtime-probe-v1",
              "operator_supplied_source_blake3": args.source_blake3,
              "os_build": probe.invoke(["/usr/bin/sw_vers", "-buildVersion"]),
              "non_claims": ["not Mantle acceptance", "broad diagnostic permissions are not supported policy"],
              "cases": {}}
    with tempfile.TemporaryDirectory(prefix="mantle-darwin-runtime-") as temporary:
        root = pathlib.Path(temporary).resolve()
        binary = root / "probe"
        allowed = root / "allowed"
        allowed.mkdir()
        report["compile"] = probe.invoke(["/Library/Developer/CommandLineTools/usr/bin/clang",
                                          "-Wall", "-Wextra", "-Werror", "-arch", "arm64", str(source), "-o", str(binary)])
        if report["compile"]["status"] == 0:
            filters = '(subpath "/usr/lib") (subpath "/System/Library") (literal "/bin/sh") (literal ' + probe.quote(binary) + ')'
            base = '(version 1)\n(deny default)\n(allow process-exec)\n(allow sysctl-read)\n'
            base += '(allow file-read* ' + filters + ' (literal "/dev/null") (subpath ' + probe.quote(allowed) + '))\n'
            base += '(allow file-write* (literal "/dev/null") (subpath ' + probe.quote(allowed) + '))\n'
            mapping = '(allow file-map-executable ' + filters + ')\n'
            ancestors = {parent for path in [binary, allowed, pathlib.Path("/usr/lib"), pathlib.Path("/System/Library"), pathlib.Path("/bin/sh")]
                         for parent in path.parents}
            metadata = '(allow file-read-metadata ' + ' '.join('(literal ' + probe.quote(path) + ')' for path in sorted(ancestors)) + ')\n'
            self_info = '(allow process-info* (target self))\n'
            mach = '(allow mach-lookup)\n'
            variants = {"base": "", "map": mapping, "metadata": metadata,
                        "map-metadata": mapping + metadata,
                        "map-metadata-self-info": mapping + metadata + self_info,
                        "diagnostic-broad-mach": mach,
                        "diagnostic-map-metadata-broad-mach": mapping + metadata + mach,
                        "diagnostic-map-metadata-self-info-broad-mach": mapping + metadata + self_info + mach,
                        "diagnostic-broad-map-read": '(allow file-read* file-map-executable)\n'}
            env = {"PATH": "/path-not-set", "HOME": "/homeless-shelter", "LANG": "C", "LC_ALL": "C"}
            for name, extra in variants.items():
                profile = base + extra
                path = root / (name + ".sb")
                path.write_text(profile)
                cases = {"profile": profile, "diagnostic_only": name.startswith("diagnostic-")}
                for command_name, command in [("fixture", [str(binary), "hello"]),
                                               ("shell", ["/bin/sh", "-c", "printf mantle-runtime-shell"])]:
                    cases[command_name] = probe.invoke(["/usr/bin/sandbox-exec", "-f", str(path)] + command,
                                                       cwd=allowed, env=env, preexec_fn=probe.limits)
                report["cases"][name] = cases
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    raise SystemExit(0 if report["compile"]["status"] == 0 else 1)


if __name__ == "__main__":
    main()
