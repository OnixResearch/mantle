#!/usr/bin/env python3
"""Real iCE40 proof. No fetching, PATH discovery, mock tools, or cache fallback.

pin consumes operator-selected exact executable/closure declarations and records
BLAKE3 observations. prove consumes those pins unchanged. See docs/fpga.md.
"""
import argparse
import copy
import json
from pathlib import Path
import shutil
import subprocess
import sys

MAX_JSON = 1_048_576
ROLES = ("yosys", "nextpnr", "packer", "shell", "bwrap")
SIM_ROLES = ("verilator", "cxx", "make", "ar")
PINNED_ROOTS = {
    "yosys": ("0.68", "d4q316mks6inhm54c19lqgw2c7snm5hk-yosys-0.68", "yosys"),
    "nextpnr": ("0.10", "zhalr5ayq69gq0bia0nj53pdnadxz86f-nextpnr-0.10", "nextpnr-ice40"),
    "packer": ("0-unstable-2025-06-03", "n7hvzj54m1rf80mzrzqn4mg8axmc4mgf-icestorm-0-unstable-2025-06-03", "icepack"),
    "shell": ("1.37.0", "8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0", "sh"),
    "bwrap": ("0.11.0", "csk28n2yj6pzwkslf3mn2gdxz33xxazr-bubblewrap-0.11.0", "bwrap"),
    "verilator": ("5.046", "3761xqvdyh2f5962f8ivl5lshp97y6fm-verilator-5.046", "verilator"),
    "cxx": ("21.1.8", "hh6y3s72d21whp6q98h4dh0valxiaw69-clang-wrapper-21.1.8", "clang++"),
    "make": ("4.4.1", "0xw6y53ijaqwfd9c99wyaqiinychzv1f-gnumake-4.4.1", "make"),
    "ar": ("2.44", "rfp8lhk4dl9syfn64rwb3h3c73426p08-binutils-2.44", "ar"),
}


def load(path):
    data = Path(path).read_bytes()
    if len(data) > MAX_JSON:
        raise RuntimeError("proof JSON exceeds byte bound")
    return json.loads(data)


def save(path, value):
    with Path(path).open("x") as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write("\n")


def declare(args):
    if not args.nix or not Path(args.nix).is_absolute():
        raise RuntimeError("declare requires --nix with an explicit absolute Nix executable")
    declarations = {"simulation": {}}
    for role, (version, basename, executable) in PINNED_ROOTS.items():
        root = "/nix/store/" + basename
        facts = json.loads(checked([args.nix, "path-info", "--offline", "--recursive", "--json", root]))
        closure = sorted(facts.keys() if isinstance(facts, dict) else (entry["path"] for entry in facts))
        tool = {"version": version, "path": root + "/bin/" + executable, "closure": closure}
        (declarations if role in ROLES else declarations["simulation"])[role] = tool
    save(args.output, declarations)


def command(args, timeout=700):
    result = subprocess.run([str(a) for a in args], env={"LC_ALL": "C"},
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout)
    if len(result.stdout) + len(result.stderr) > 2 * MAX_JSON:
        raise RuntimeError("proof subprocess log bound")
    return result


def checked(args):
    result = command(args)
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode(errors="replace"))
    return result.stdout.decode()


def digest(adapter, path):
    return checked([adapter, "hash", path]).strip()


def file_pin(adapter, path, name=None):
    path = Path(path).absolute()
    return {"path": str(path), "name": name or path.name, "blake3": digest(adapter, path)}


def validate_tool(adapter, tool):
    roots = tool["closure"]
    if not roots or len(roots) > 256 or roots != sorted(set(roots)):
        raise RuntimeError("closure must be explicit, sorted, unique, and bounded")
    for root in roots:
        path = Path(root)
        if not root.startswith("/nix/store/") or path.parent != Path("/nix/store"):
            raise RuntimeError("closure entries must be immutable Nix store roots")
        if not path.is_dir() or path.resolve() != path:
            raise RuntimeError(f"missing/noncanonical declared closure: {root}")
    executable = Path(tool["executable"]["path"])
    canonical = executable.resolve(strict=True)
    if not any(canonical.is_relative_to(Path(root)) for root in roots):
        raise RuntimeError("tool executable escaped declared closure")
    if digest(adapter, executable) != tool["executable"]["blake3"]:
        raise RuntimeError(f"tool content pin mismatch: {executable}")


def pin(args):
    declarations = load(args.inputs)
    cohort = {"schema": "mantle-fpga-proof-cohort-v1", "simulation": {}}
    for group, roles in ((declarations, ROLES), (declarations["simulation"], SIM_ROLES)):
        output = cohort if group is declarations else cohort["simulation"]
        for role in roles:
            declared = group[role]
            tool = {"version": declared["version"], "closure": declared["closure"],
                    "executable": file_pin(args.adapter, declared["path"])}
            validate_tool(args.adapter, tool)
            output[role] = tool
    save(args.output, cohort)


def make_profile(adapter, repo, cohort):
    fixture = repo / "tests/fixtures/fpga"
    return {
        "schema": "mantle-fpga-profile-v1", "part": "ice40-hx1k", "package": "tq144",
        "rtl": [file_pin(adapter, repo / "tests/fixtures/hardware-simulation/selected-ip/rtl/tiny_adder.sv"),
                file_pin(adapter, fixture / "registered_adder.sv")],
        "includes": [file_pin(adapter, fixture / "include/config.vh", "include/config.vh")],
        "include_dirs": ["include"], "top": "registered_adder", "defines": {"FPGA_BUILD": "1"},
        "parameters": {"WIDTH": 2}, "synthesis_script": file_pin(adapter, fixture / "synth.ys"),
        "synthesis_passes": ["check"], "flatten": True,
        "pins": {"a[0]": "78", "a[1]": "79", "b[0]": "80", "b[1]": "81", "clk": "21",
                 "sum[0]": "99", "sum[1]": "98", "sum[2]": "97"},
        "clocks": [{"port": "clk", "report_net": "clk$SB_IO_IN_$glb_clk", "frequency_khz": 10000}],
        "constraints": file_pin(adapter, fixture / "pins.pcf"), "seed": 1, "threads": 1,
        **{role: cohort[role] for role in ROLES},
        "database": {"part": "ice40-hx1k", "package": "tq144",
                     "nextpnr_binary_blake3": cohort["nextpnr"]["executable"]["blake3"]},
        "utilization_limits": {"ICESTORM_LC": 100, "SB_IO": 8, "SB_GB": 1},
        "timeout_ms": 120000, "max_log_bytes": 262144, "max_output_bytes": 16777216,
    }


def run_profile(adapter, profile, output, label, expected=None):
    profile_path = output / f"{label}.json"
    save(profile_path, profile)
    result = command([adapter, "run", profile_path, output / label])
    (output / f"{label}.stdout").write_bytes(result.stdout)
    (output / f"{label}.stderr").write_bytes(result.stderr)
    if expected is None:
        if result.returncode != 0:
            raise RuntimeError(f"{label}: {result.stderr.decode(errors='replace')}")
        return json.loads(result.stdout)
    if result.returncode == 0 or expected not in result.stderr.decode(errors="replace"):
        raise RuntimeError(f"{label}: expected {expected!r}, got {result.stderr!r}")
    if (output / label / "manifest.json").exists():
        raise RuntimeError(f"{label}: rejected build published a success manifest")
    return {"rejected": expected}


def pcf(profile):
    text = "".join(f"set_io {port} {pin}\n" for port, pin in sorted(profile["pins"].items()))
    for clock in profile["clocks"]:
        khz = clock["frequency_khz"]
        text += f"set_frequency {clock['port']} {khz // 1000}.{khz % 1000:03d}\n"
    return text


def rebind_pcf(adapter, profile, output, label):
    path = output / f"{label}.pcf"
    path.write_text(pcf(profile))
    profile["constraints"] = file_pin(adapter, path, "pins.pcf")


def plan(adapter, profile, output, label, expected=None):
    path = output / f"plan-{label}.json"
    save(path, profile)
    result = command([adapter, "plan", path])
    if expected:
        if result.returncode == 0 or expected not in result.stderr.decode(errors="replace"):
            raise RuntimeError(f"plan {label}: did not reject {expected}: {result.stderr!r}")
        return {"rejected": expected}
    if result.returncode != 0:
        raise RuntimeError(f"plan {label}: {result.stderr!r}")
    return json.loads(result.stdout)


def invalidation_proof(adapter, baseline, output):
    original = plan(adapter, baseline, output, "baseline")
    results = {}
    for label, expected_stages in (
            ("rtl", (True, True, True)), ("constraints", (False, True, True)),
            ("seed", (False, True, True)), ("yosys", (True, True, True)),
            ("database", (False, True, True)), ("packer", (False, False, True))):
        changed = copy.deepcopy(baseline)
        different = "c" * 64
        if label == "rtl":
            changed["rtl"][0]["blake3"] = different
        elif label == "constraints":
            changed["constraints"]["blake3"] = different
        elif label == "seed":
            changed["seed"] += 1
        elif label == "database":
            changed["nextpnr"]["executable"]["blake3"] = different
            changed["database"]["nextpnr_binary_blake3"] = different
        else:
            changed[label]["executable"]["blake3"] = different
        observed = plan(adapter, changed, output, label)
        stages = ("synthesis", "place_route", "packing")
        actual = tuple(original[stage] != observed[stage] for stage in stages)
        if actual != expected_stages:
            raise RuntimeError(f"{label}: wrong invalidation boundary {actual}")
        results[label] = {stage: actual[i] for i, stage in enumerate(stages)}
    return results


def negative_proof(adapter, baseline, output):
    results = {}
    for label, field, value, error in (
        ("unsupported-part", "part", "ice40-up5k", "unsupported-schema-part-or-package"),
        ("missing-pins", "pins", {}, "missing-or-excess-pin-clock-constraints"),
        ("missing-clocks", "clocks", [], "missing-or-excess-pin-clock-constraints"),
    ):
        changed = copy.deepcopy(baseline)
        changed[field] = value
        results[label] = plan(adapter, changed, output, label, error)
    changed = copy.deepcopy(baseline)
    changed["database"]["nextpnr_binary_blake3"] = "0" * 64
    results["wrong-database"] = plan(adapter, changed, output, "wrong-database", "wrong-embedded-device-database")
    changed = copy.deepcopy(baseline)
    changed["constraints"]["path"] = str(output / "absent.pcf")
    results["missing-pcf"] = run_profile(adapter, changed, output, "missing-pcf", "inspect")
    changed = copy.deepcopy(baseline)
    invalid = output / "invalid.pcf"
    invalid.write_text("set_io clk 21\n")
    changed["constraints"] = file_pin(adapter, invalid, "pins.pcf")
    results["invalid-pcf"] = run_profile(adapter, changed, output, "invalid-pcf", "invalid-pcf-does-not-match")
    changed = copy.deepcopy(baseline)
    del changed["pins"]["a[0]"]
    rebind_pcf(adapter, changed, output, "unconstrained-pin")
    results["unconstrained-pin"] = run_profile(adapter, changed, output, "unconstrained-pin", "unconstrained-or-unknown-required-pin")
    changed = copy.deepcopy(baseline)
    changed["pins"]["a[0]"] = "1000"
    rebind_pcf(adapter, changed, output, "invalid-package-pin")
    results["invalid-package-pin"] = run_profile(adapter, changed, output, "invalid-package-pin", "place-route-failed")
    changed = copy.deepcopy(baseline)
    changed["clocks"][0]["report_net"] = "missing_clock"
    results["unreported-clock"] = run_profile(adapter, changed, output, "unreported-clock", "required-clock-not-reported")
    changed = copy.deepcopy(baseline)
    changed["clocks"][0]["frequency_khz"] = 1000000
    rebind_pcf(adapter, changed, output, "timing-failure")
    results["timing-failure"] = run_profile(adapter, changed, output, "timing-failure", "declared-timing-violation")
    changed = copy.deepcopy(baseline)
    changed["utilization_limits"]["ICESTORM_LC"] = 0
    results["utilization-failure"] = run_profile(adapter, changed, output, "utilization-failure", "declared-utilization-violation")
    return results


def sandbox(cohort, directory, executable, args, label):
    roots = sorted(set(root for tool in cohort["simulation"].values() for root in tool["closure"])
                   | set(cohort["shell"]["closure"]))
    bwrap = cohort["bwrap"]["executable"]["path"]
    shell = cohort["shell"]["executable"]["path"]
    argv = [bwrap, "--unshare-all", "--die-with-parent", "--new-session", "--clearenv",
            "--proc", "/proc", "--dev", "/dev", "--tmpfs", "/tmp", "--dir", "/nix",
            "--dir", "/nix/store", "--dir", "/bin", "--dir", "/build"]
    for root in roots:
        argv += ["--ro-bind", root, root]
    argv += ["--ro-bind", shell, "/bin/sh", "--bind", str(directory), "/build", "--chdir", "/build"]
    environment = {"HOME": "/build", "TMPDIR": "/tmp", "LC_ALL": "C", "LANG": "C", "TZ": "UTC",
                   "SOURCE_DATE_EPOCH": "0", "OMP_NUM_THREADS": "1", "PYTHONHASHSEED": "0",
                   "MAKE": cohort["simulation"]["make"]["executable"]["path"],
                   "PATH": ":".join(root + "/bin" for root in roots)}
    for key, value in environment.items():
        argv += ["--setenv", key, value]
    argv += ["--", shell, "-c", 'ulimit -c 0; ulimit -f 65536; exec "$@"', "simulation", executable, *args]
    result = command(argv)
    (directory / f"{label}.stdout").write_bytes(result.stdout)
    (directory / f"{label}.stderr").write_bytes(result.stderr)
    if result.returncode != 0:
        raise RuntimeError(f"simulation {label}: {result.stderr.decode(errors='replace')}")
    return result.stdout.decode().strip()


def copy_pinned(adapter, source, destination, observations, expected):
    pin = file_pin(adapter, source)
    if pin["path"] in expected and pin["blake3"] != expected[pin["path"]]:
        raise RuntimeError("simulation RTL drifted from synthesized source pin")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    if digest(adapter, destination) != pin["blake3"]:
        raise RuntimeError("simulation source changed while snapshotting")
    observations.append(pin)


def simulation_proof(adapter, repo, cohort, output, baseline):
    sim = cohort["simulation"]
    if sim["verilator"]["version"] != "5.046" or sim["cxx"]["version"] != "21.1.8":
        raise RuntimeError("simulation requires existing Verilator 5.046 / Clang 21.1.8 cohort")
    observations = {}
    expected = {pin["path"]: pin["blake3"] for pin in baseline["rtl"] + baseline["includes"]}
    for top in ("tiny_adder", "registered_adder"):
        directory = output / f"simulate-{top}"
        directory.mkdir()
        fixture = repo / "tests/fixtures/hardware-simulation"
        input_pins = []
        copy_pinned(adapter, fixture / "selected-ip/rtl/tiny_adder.sv", directory / "tiny_adder.sv", input_pins, expected)
        sources = ["tiny_adder.sv", "sim_main.cpp"]
        if top == "tiny_adder":
            copy_pinned(adapter, fixture / "selected-vip/model/sim_main.cpp", directory / "sim_main.cpp", input_pins, expected)
            copy_pinned(adapter, fixture / "selected-vip/model/reference.cpp", directory / "reference.cpp", input_pins, expected)
            sources.append("reference.cpp")
        else:
            fpga = repo / "tests/fixtures/fpga"
            copy_pinned(adapter, fpga / "sim_main.cpp", directory / "sim_main.cpp", input_pins, expected)
            copy_pinned(adapter, fpga / "registered_adder.sv", directory / "registered_adder.sv", input_pins, expected)
            copy_pinned(adapter, fpga / "include/config.vh", directory / "include/config.vh", input_pins, expected)
            sources.append("registered_adder.sv")
        makeflags = (f"CXX={sim['cxx']['executable']['path']} LINK={sim['cxx']['executable']['path']} "
                     f"AR={sim['ar']['executable']['path']} SHELL=/bin/sh")
        args = ["--cc", "--exe", "--build", "-j", "1", "--top-module", top,
                "--Mdir", "obj", "-Iinclude", "-CFLAGS", "-std=c++17 -O1", "-LDFLAGS", "-pthread",
                "-MAKEFLAGS", makeflags, *sources]
        sandbox(cohort, directory, sim["verilator"]["executable"]["path"], args, "compile")
        binary = "/build/obj/V" + top
        if top == "tiny_adder":
            vectors = []
            for inputs, expected in (("0,0", "0"), ("1,2", "3"), ("255,255", "510")):
                observed = sandbox(cohort, directory, binary, [inputs], "vector-" + inputs.replace(",", "-"))
                if observed != expected:
                    raise RuntimeError(f"existing simulation fixture mismatch {inputs}: {observed}")
                vectors.append({"input": inputs, "observed": observed})
            observations[top] = vectors
        else:
            observed = sandbox(cohort, directory, binary, [], "smoke")
            if observed != "registered-adder: 16 input pairs passed after two clocks":
                raise RuntimeError("registered fixture simulation did not report exact checked vectors")
            observations[top] = observed
        observations[top + "-binary-blake3"] = digest(adapter, directory / "obj" / ("V" + top))
        observations[top + "-source-inputs"] = input_pins
    return observations


def prove(args):
    repo = Path(args.repo).resolve(strict=True)
    output = Path(args.output).absolute()
    output.mkdir()  # A repeat must never reuse a stale output directory.
    cohort = load(args.inputs)
    if cohort["schema"] != "mantle-fpga-proof-cohort-v1":
        raise RuntimeError("unsupported cohort schema")
    for role in ROLES:
        validate_tool(args.adapter, cohort[role])
    for role in SIM_ROLES:
        validate_tool(args.adapter, cohort["simulation"][role])
    save(output / "cohort.json", cohort)
    baseline = make_profile(args.adapter, repo, cohort)
    first = run_profile(args.adapter, baseline, output, "baseline")
    second = run_profile(args.adapter, baseline, output, "clean-repeat")
    left = (output / "baseline/packing/bitstream.bin").read_bytes()
    right = (output / "clean-repeat/packing/bitstream.bin").read_bytes()
    if not left or left != right or first["stages"] != second["stages"]:
        raise RuntimeError("clean-repeat bitstream comparison failed")
    bitstream_hash = digest(args.adapter, output / "baseline/packing/bitstream.bin")
    evidence = {"schema": "mantle-fpga-proof-v1", "profile": baseline,
                "cohort_blake3": digest(args.adapter, output / "cohort.json"),
                "adapter_blake3": digest(args.adapter, args.adapter),
                "proof_driver_blake3": digest(args.adapter, Path(__file__).resolve()),
                "bitstreams_equal": True, "bitstream_blake3": bitstream_hash,
                "bitstream_bytes": len(left), "stages": first["stages"],
                "actual_nextpnr_report": first["report"],
                "invalidation": invalidation_proof(args.adapter, baseline, output),
                "negative_cases": negative_proof(args.adapter, baseline, output),
                "simulation": simulation_proof(args.adapter, repo, cohort, output, baseline),
                "non_claims": first["non_claims"]}
    save(output / "proof.json", evidence)
    print(json.dumps({"proof": str(output / "proof.json"), "bitstream_blake3": bitstream_hash}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("declare", "pin", "prove"))
    parser.add_argument("--adapter", type=lambda p: str(Path(p).resolve(strict=True)))
    parser.add_argument("--inputs")
    parser.add_argument("--nix")
    parser.add_argument("--output", required=True)
    parser.add_argument("--repo", default=".")
    args = parser.parse_args()
    try:
        if args.mode != "declare" and (not args.adapter or not args.inputs):
            raise RuntimeError("pin/prove require --adapter and --inputs")
        {"declare": declare, "pin": pin, "prove": prove}[args.mode](args)
    except (RuntimeError, OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        print(f"fpga-proof: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
