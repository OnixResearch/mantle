#!/usr/bin/env python3
"""Source-bound CMake proof. Requires explicit already-admitted inputs; no fetch/cache/tool fallback."""
import argparse
import copy
import json
from pathlib import Path
import struct
import subprocess


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    m = json.loads(args.manifest.read_text())
    require(m["schema"] == "mantle-cmake-proof-inputs-v1", "wrong proof manifest schema")
    args.work.mkdir(parents=True, exist_ok=False)
    work = args.work.resolve()
    for directory in ("home", "tmp"):
        (work / directory).mkdir()
    # Every process gets an explicit PATH/tool runtime contract, not os.environ.
    env = {"PATH": ":".join(m["tool_path"]), "HOME": str(work / "home"),
           "TMPDIR": str(work / "tmp"), "LC_ALL": "C", "TZ": "UTC",
           "SNIX_BUILD_SANDBOX_SHELL": m["sandbox_shell"]}
    for name in ("mantle", "adapter", "b3sum", "readelf", "sandbox_shell", "signing_key", "trusted_public_key", "nickel_module"):
        require(Path(m[name]).is_absolute() and Path(m[name]).is_file(), f"missing explicit input {name}")
    public_key_token = Path(m["trusted_public_key"]).read_text().strip()
    logs = []

    def run(label, argv, process_env=None, success=True):
        result = subprocess.run([str(item) for item in argv], env=process_env or env,
                                cwd=work, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        (work / f"{label}.stdout").write_text(result.stdout)
        (work / f"{label}.stderr").write_text(result.stderr)
        logs.append({"label": label, "argv": [str(v) for v in argv], "status": result.returncode})
        require((result.returncode == 0) == success, f"{label}: unexpected exit {result.returncode}; see captured output")
        return result

    def digest(label, path):
        return run(label, [m["b3sum"], path]).stdout.split()[0]

    plans = {name: json.loads(Path(m[f"{name}_plan"]).read_text()) for name in ("native", "cross")}
    for name, plan in plans.items():
        require(plan["source_subdir"] == "project" and plan["emulator"] is None, f"{name}: wrong fixture/emulator")
        require(plan["store_prefix"] == m["store_prefix"], "store prefix drift")
        require(plan["tools"]["cmake"]["version"] == "3.31.8", "proof pins CMake 3.31.8")
        require(plan["tools"]["ninja"]["version"] == "1.12.1", "proof pins Ninja 1.12.1")
        path = work / f"{name}.plan.json"
        path.write_text(json.dumps(plan))
        identity = run(f"{name}-pin", [m["adapter"], "validate", path]).stdout.strip()
        require(identity == m[f"{name}_plan_blake3"], f"{name}: plan identity drift")
        physical_source = Path(m["input_store"]) / Path(plan["roles"][plan["source"]]).name
        for member, expected in json.loads(Path(m["fixture_lock"]).read_text())["files"].items():
            require(digest(f"{name}-source-{member.replace('/', '-')}", physical_source / member) == expected,
                    f"{name}: fixture bytes drift: {member}")
    require(plans["native"]["build_platform"] == "x86_64-linux" and plans["native"]["target"]["processor"] == "x86_64", "native proof is x86_64 Linux")
    require(plans["cross"]["target"]["triple"] == "aarch64-unknown-linux-gnu", "cross proof is pinned AArch64 Linux")
    require(plans["cross"]["build_platform"] == "x86_64-linux", "cross generator platform must remain x86_64")
    require(plans["native"]["roles"][plans["native"]["source"]] == plans["cross"]["roles"][plans["cross"]["source"]], "native/cross sources differ")
    require(Path(m["state_dir"]).resolve() != Path(m["clean_state_dir"]).resolve(), "clean comparison needs an independent admitted state")

    def build(label, plan, clean=False, negative=None):
        recipe = work / f"{label}.ncl"
        plan_json = json.dumps(json.dumps(plan, separators=(",", ":")))
        runner_json = json.dumps(json.dumps(m["runner"], separators=(",", ":")))
        # Same Nickel authoring path used by callers; adapter CLI lowering is not a fake builder.
        recipe.write_text(f'let cmake = import {json.dumps(m["nickel_module"])} in\n'
                          f'cmake.mkProject {{ plan = std.deserialize \'Json {plan_json}, runner = std.deserialize \'Json {runner_json} }}\n')
        invocation_env = dict(env)
        if clean:
            invocation_env.update({"CPATH": "/usr/include", "CPLUS_INCLUDE_PATH": "/usr/include",
                                   "CFLAGS": "-I/undeclared-host", "CMAKE_PREFIX_PATH": "/usr",
                                   "PKG_CONFIG_PATH": "/usr/lib/pkgconfig"})
        result = run(label, [m["mantle"], "--json", "--store-prefix", m["store_prefix"],
                            "--store", m["clean_store"] if clean else m["store"],
                            "--state-dir", m["clean_state_dir"] if clean else m["state_dir"],
                            "--base-store", m["input_store"], "build", recipe, "--no-substitute",
                            "--signing-key", m["signing_key"], "--trusted-public-keys", public_key_token],
                     invocation_env, success=negative is None)
        if negative is not None:
            # Logs may be reported as structured build failure paths. Read only files
            # explicitly named by this report; never scan arbitrary store/state trees.
            text = result.stdout + result.stderr
            try:
                report = json.loads(result.stdout)
            except json.JSONDecodeError:
                report = {}
            def selected_logs(value):
                if isinstance(value, dict):
                    for key, item in value.items():
                        if key in ("log_path", "build_log_path", "saved_log_path") and isinstance(item, str) and Path(item).is_file():
                            yield Path(item).read_text(errors="replace")
                        else:
                            yield from selected_logs(item)
                elif isinstance(value, list):
                    for item in value:
                        yield from selected_logs(item)
            text += "\n".join(selected_logs(report))
            require(negative in text, f"{label}: missing expected diagnostic {negative!r}")
            return None
        report = json.loads(result.stdout)
        require(report["counts"]["failed_total"] == 0, f"{label}: failed outcomes")
        rows = [row for row in report["outcomes"] if row["label"] == plan["name"]]
        require(len(rows) == 1, f"{label}: ambiguous root outcome")
        outputs = [out["path"] for out in rows[0]["outputs"] if out["name"] == "out"]
        require(len(outputs) == 1, f"{label}: missing output")
        return rows[0], Path(outputs[0])

    first, native = build("native-first", plans["native"])
    require(not first["cached"], "native proof state was not clean")
    cached, replay = build("native-replay", plans["native"])
    require(cached["cached"] and replay == native, "unchanged input did not reuse output")
    clean, rebuilt = build("native-clean-poisoned-environment", plans["native"], clean=True)
    require(not clean["cached"], "independent state reused a cached root")
    native_manifest = json.loads((native / "share/mantle/cmake-provenance.json").read_text())
    rebuilt_manifest = json.loads((rebuilt / "share/mantle/cmake-provenance.json").read_text())
    require(native_manifest == rebuilt_manifest, "clean builds differ, including configuration/tool provenance")
    for artifact in native_manifest["artifacts"]:
        if artifact["symlink_target"] is None:
            require(digest("native-installed-" + artifact["path"].replace("/", "-"), native / artifact["path"]) == artifact["blake3"], "installed digest mismatch")
            require(digest("clean-installed-" + artifact["path"].replace("/", "-"), rebuilt / artifact["path"]) == artifact["blake3"], "clean artifact mismatch")
    run("native-explicit-execution", [native / "bin/answer-bin"])
    cross_row, cross = build("cross-first", plans["cross"])
    require(not cross_row["cached"], "cross proof state was not clean")
    for label, root, machine, readelf_machine in (("native", native, 62, "Advanced Micro Devices X86-64"), ("cross", cross, 183, "AArch64")):
        data = (root / "bin/answer-bin").read_bytes()
        require(data[:6] == b"\x7fELF\x02\x01" and struct.unpack_from("<H", data, 18)[0] == machine, f"{label}: wrong ELF machine")
        for artifact in ("bin/answer-bin", "lib/libanswer.a"):
            inspection = run(f"{label}-readelf-{artifact.replace('/', '-')}", [m["readelf"], "-h", root / artifact]).stdout
            require(readelf_machine in inspection, f"{label}: independent architecture inspector disagrees")
    changed = copy.deepcopy(plans["native"])
    changed["definitions"]["FIXTURE_OFFSET"] = "1"
    changed_row, changed_root = build("configuration-invalidation", changed)
    require(not changed_row["cached"] and changed_root != native, "configuration change did not invalidate identity")
    require(digest("native-library", native / "lib/libanswer.a") != digest("changed-library", changed_root / "lib/libanswer.a"), "changed C definition did not change native library")
    negatives = {"download": "cmake-undeclared-download", "external": "cmake-undeclared-download",
                 "fetch": "FETCHCONTENT_FULLY_DISCONNECTED", "missing": "MantleUndeclaredDependency",
                 "host-leak": "Could not find HOST_LEAK", "target-run": "cross-compiling mode",
                 "install-escape": "cmake-install-symlink-escape"}
    for fixture, diagnostic in negatives.items():
        bad = copy.deepcopy(plans["native"])
        bad["name"] += "-" + fixture
        bad["source_subdir"] = fixture
        build("negative-" + fixture, bad, negative=diagnostic)
    bad = copy.deepcopy(plans["native"])
    bad["tools"]["cc"]["version"] = "deliberately-incompatible-tool-version"
    build("negative-incompatible-tool", bad, negative="cmake-incompatible-tool")
    bad = copy.deepcopy(plans["native"])
    bad["target"] = {"system": "Linux", "processor": "aarch64", "triple": "aarch64-unknown-linux-gnu"}
    bad["compiler"] = "gcc"
    build("negative-incompatible-target", bad, negative="cmake-incompatible-toolchain")
    bad = copy.deepcopy(plans["native"])
    bad["tools"]["cc"]["member"] = "../../usr/bin/cc"
    build("negative-member-escape", bad, negative="cmake-path-escape")
    evidence = {"schema": "mantle-cmake-proof-v1", "native": str(native), "cross": str(cross),
                "inputs": m, "commands": logs, "observed": ["native-c-and-cxx-library-and-executable", "aarch64-inspected-not-executed",
                "cache-hit", "configuration-invalidation", "independent-clean-rebuild-with-poisoned-ambient-environment", "negative-policy-cases"],
                "non_claims": ["not-global-reproducibility", "not-compiler-correctness", "not-universal-cmake-compatibility"]}
    (work / "evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print(work / "evidence.json")


if __name__ == "__main__":
    main()
