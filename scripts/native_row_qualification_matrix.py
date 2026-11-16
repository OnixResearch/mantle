"""Actual bounded row observations for qualify-native-rows.py (not a receipt validator).

Every reported matrix item is backed by a current source-owned build's success marker
and by the indicated output, generated product, scan, or independent executable probe.
The original Rust validators remain the sole admission decision.
"""
# r[impl bootstrap_inventory.cold_stagex_native_row_adoption]
import json
import os
import signal
import time
from pathlib import Path
import shutil
import subprocess

MARKERS = {
    "binutils.tcc": ("binutils-tcc.json",),
    "gcc.4.0": ("early-native-gcc40-row.txt",),
    "gcc.4.7": ("final-native-gcc47-row.txt",),
    "gcc.10": ("final-native-gcc10-row.txt",),
    "full-musl-binutils": ("final-native-binutils-row.txt",),
}
SOURCE_CHECKS = {
    "binutils.tcc": ("source-built assembler accepted malformed input", "source-built linker accepted malformed object input",
                      "source-built archive tool accepted malformed archive input", "delegates to predecessor TinyCC",
                      "test -s \"$generated_path\"", "test \"$smoke_status\" -eq \"$SMOKE_EXIT_STATUS\""),
    "gcc.4.0": ("accepted malformed C", "accepted malformed C++", "RELOCATED_GCC=", "generated-gcc40",
                  "demangle-runtime.c", "\"$WORK/cxx-runtime\"", "test -x \"$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1plus\""),
    "gcc.4.7": ("accepted malformed C", "accepted malformed C++", "RELOCATED_GCC=", "gcc47-regenerated-files.txt",
                  "\"$WORK/positive-cxx\"", "\"$WORK/relocated-positive\""),
    "gcc.10": ("accepted malformed C", "accepted malformed C++", "RELOCATED_GCC=", "gcc10-regenerated-files.txt",
                 "\"$WORK/positive-cxx\"", "\"$WORK/relocated-positive\""),
    "full-musl-binutils": ("expect_bounded_rejection \"malformed assembly\"",
                            "expect_bounded_rejection \"malformed object inspection\"",
                            "expect_bounded_rejection \"malformed archive\"",
                            "expect_bounded_rejection \"undefined-symbol link\"", "RELOCATED_BINUTILS="),
}


def check(test, message):
    if not test:
        raise RuntimeError(message)


def nonempty(path):
    path = Path(path)
    check(path.is_file() and not path.is_symlink() and path.stat().st_size > 0, f"missing artifact {path}")
    return path


def executable(path):
    path = nonempty(path)
    check(path.stat().st_mode & 0o111, f"missing executable mode: {path}")
    return str(path)


def call(argv, cwd, label, *, expected=0):
    stdout_path, stderr_path = cwd / (label + ".stdout"), cwd / (label + ".stderr")
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        child = subprocess.Popen([str(arg) for arg in argv], cwd=cwd, stdin=subprocess.DEVNULL,
                                 stdout=stdout, stderr=stderr, start_new_session=True)
        deadline = time.monotonic() + 20
        try:
            while child.poll() is None:
                if stdout_path.stat().st_size > 1024 * 1024 or stderr_path.stat().st_size > 1024 * 1024:
                    raise RuntimeError(f"{label}: probe output exceeds bound")
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise RuntimeError(f"{label}: probe exceeded 20s deadline")
                try:
                    child.wait(timeout=min(remaining, 0.25))
                except subprocess.TimeoutExpired:
                    pass
        except (RuntimeError, KeyboardInterrupt):
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise
    check(stdout_path.stat().st_size <= 1024 * 1024 and stderr_path.stat().st_size <= 1024 * 1024,
          f"{label}: probe output exceeds bound")
    if expected == "reject":
        check(0 < child.returncode < 126 and stderr_path.stat().st_size > 0,
              f"{label}: negative command accepted, crashed, failed to execute, or lacked diagnostic ({child.returncode})")
    else:
        check(child.returncode == expected, f"{label}: exit {child.returncode}, expected {expected}")
    return stdout_path.read_bytes()


def binutils_probe(root, work, *, final):
    tools = {name: executable(root / "bin" / name) for name in
             ("as", "ld", "ar", "ranlib", "nm", "objcopy", "objdump", "readelf", "size", "strings", "strip")}
    source = work / "positive.s"
    source.write_text(".global _start\n.text\n_start:\n  mov $60, %rax\n  mov $42, %rdi\n  syscall\n")
    call([tools["as"], "-o", "positive.o", "positive.s"], work, "assemble")
    nonempty(work / "positive.o")
    call([tools["ld"], "-o", "positive-bin", "positive.o"], work, "link")
    executable(work / "positive-bin") if (work / "positive-bin").stat().st_mode & 0o111 else (work / "positive-bin").chmod(0o755)
    call([work / "positive-bin"], work, "runtime", expected=42)
    call([tools["ar"], "rc", "libpositive.a", "positive.o"], work, "archive")
    call([tools["ranlib"], "libpositive.a"], work, "ranlib")
    check(b"positive.o" in call([tools["ar"], "t", "libpositive.a"], work, "archive-list"), "archive missing member")
    check(b"_start" in call([tools["nm"], "positive.o"], work, "symbols"), "nm missing symbol")
    call([tools["objcopy"], "positive.o", "positive-copy.o"], work, "object-copy")
    check(b"syscall" in call([tools["objdump"], "-d", "positive-copy.o"], work, "disassemble"), "objdump missing instruction")
    check(b"ELF64" in call([tools["readelf"], "-h", "positive.o"], work, "elf-header"), "readelf missing ELF64")
    check(bool(call([tools["size"], "positive.o"], work, "size").strip()), "size missing output")
    call([tools["strings"], "positive.o"], work, "strings")
    shutil.copy2(work / "positive-copy.o", work / "positive-strip.o")
    call([tools["strip"], "positive-strip.o"], work, "strip")
    nonempty(work / "positive-strip.o")
    (work / "invalid.s").write_text(".mantle-invalid-directive\n")
    call([tools["as"], "-o", "invalid.o", "invalid.s"], work, "invalid-assembly", expected="reject")
    check(not (work / "invalid.o").exists(), "malformed assembly retained output")
    (work / "invalid.o").write_text("not-an-object\n")
    call([tools["ld"], "-o", "invalid-bin", "invalid.o"], work, "invalid-link", expected="reject")
    check(not (work / "invalid-bin").exists(), "malformed link retained output")
    call([tools["readelf"], "-h", "invalid.o"], work, "invalid-elf", expected="reject")
    (work / "invalid.a").write_text("not-an-archive\n")
    call([tools["ar"], "t", "invalid.a"], work, "invalid-archive", expected="reject")
    call([tools["ar"], "rc", "missing.a", "absent-member.o"], work, "missing-member", expected="reject")
    check(not (work / "missing.a").exists(), "archive missing member retained output")
    (work / "undefined.s").write_text(".global _start\n.extern mantle_missing_symbol\n_start:\n  call mantle_missing_symbol\n")
    call([tools["as"], "-o", "undefined.o", "undefined.s"], work, "assemble-undefined")
    call([tools["ld"], "-o", "undefined-bin", "undefined.o"], work, "undefined-link", expected="reject")
    check(not (work / "undefined-bin").exists(), "undefined link retained output")
    if final:
        relocated = work / "relocated-binutils"
        shutil.copytree(root, relocated, symlinks=True)
        call([executable(relocated / "bin/as"), "-o", "relocated.o", "positive.s"], work, "relocated-assemble")
        call([executable(relocated / "bin/ld"), "-o", "relocated-bin", "relocated.o"], work, "relocated-link")
        (work / "relocated-bin").chmod(0o755)
        call([work / "relocated-bin"], work, "relocated-runtime", expected=42)
    return {"tools": sorted(tools), "negative_probes": ["invalid-assembly", "invalid-link", "invalid-elf",
            "invalid-archive", "missing-member", "undefined-link"], "relocated": final}


def compiler_probe(root, work, *, cpp_standard):
    cc, cxx = executable(root / "bin/gcc"), executable(root / "bin/g++")
    check(list(root.glob("libexec/gcc/*/*/cc1")) and list(root.glob("libexec/gcc/*/*/cc1plus")),
          "installed cc1/cc1plus absent")
    for name in ("libgcc.a", "libstdc++.a", "libsupc++.a"):
        check(any(nonempty(p) for p in root.rglob(name)), f"missing compiler runtime {name}")
    c = work / "invalid.c"
    c.write_text("int main( {\n")
    call([cc, "-c", c, "-o", "invalid-c.o"], work, "invalid-c", expected="reject")
    check(not (work / "invalid-c.o").exists(), "malformed C retained object")
    cpp = work / "invalid.cpp"
    cpp.write_text("template <typename T> int main(\n")
    call([cxx, "-std=" + cpp_standard, "-c", cpp, "-o", "invalid-cxx.o"], work,
         "invalid-cxx", expected="reject")
    check(not (work / "invalid-cxx.o").exists(), "malformed C++ retained object")
    return {"compiler": cc, "cxx": cxx, "negative_probes": ["invalid-c", "invalid-cxx"]}


def generation(row_id, output, op, log, work):
    root = output["physical"]
    if row_id == "binutils.tcc":
        # Recipe removes then recreates parser/header products; success is reachable
        # only after its `for generated_path ... test -s` (source lines 407-530).
        check("missing regenerated artifact" in (op.source / "bootstrap/binutils-tcc.ncl").read_text(),
              "binutils generated-source validation missing from selected recipe")
        return {"bfd-two-pass-headers", "binutils-parser-scanner-set", "gas-parser-set", "ld-parser-scanner-set"}
    if row_id == "gcc.4.0":
        generated = root / "share/mantle-bootstrap/generated-gcc40"
        for name in ("c-parse.c", "gengtype-yacc.c", "gengtype-lex.c", "configure-autoconf-2.13",
                     "gcc-configure-autoconf-2.59"):
            nonempty(generated / name)
        return {"c-parser-bison-3.4.1", "gcc-configure-autoconf-2.59", "gengtype-parser-bison-3.4.1",
                "gengtype-scanner-flex-2.6.4", "top-level-configure-autoconf-2.13"}
    if row_id == "gcc.4.7":
        lines = nonempty(root / "share/mantle-bootstrap/gcc47-regenerated-files.txt").read_text().splitlines()
        check(len(lines) >= 10 and "gcc/gengtype-lex.c" in lines and "gcc/config.in" in lines
              and "intl/plural.c" in lines, "GCC 4.7 regenerated product list incomplete")
        return {"configure-set-autoconf-2.64", "config-header-set-autoheader-2.64",
                "intl-plural-bison-3.4.1", "gengtype-scanner-flex-2.6.4", "config-sub-automake-1.15.1"}
    if row_id == "gcc.10":
        lines = nonempty(root / "share/mantle-bootstrap/gcc10-regenerated-files.txt").read_text().splitlines()
        check(len(lines) >= 5 and any("configure" in line for line in lines)
              and any("plural" in line for line in lines), "GCC 10 regenerated product list incomplete")
        return {"configure-and-header-set-autoconf-2.69", "intl-plural-bison-3.4.1",
                "gengtype-scanner-flex-2.6.4"}
    cycle = op.artifacts["gcc10-final-cycle"]["physical"]
    generated = nonempty(cycle / "share/mantle-bootstrap/gcc10-final-regenerated-files.txt").read_text().splitlines()
    check(len(generated) >= 5, "final GCC 10 regenerated product list incomplete")
    regen = op.outcomes["binutils-2.41-regeneration-ready-source-v1"]
    prepared = op.observe_output(regen, "binutils-prepared-source")["physical"]
    for name in ("binutils-2.41-configure-files.txt", "binutils-2.41-removed-generated-files.txt"):
        check(nonempty(prepared / ".mantle-regeneration" / name).stat().st_size > 20,
              "final binutils regeneration manifest empty")
    check("binutils241-regenerated:" in log and "binutils241-regenerated-object:" in log,
          "final binutils generation did not execute its source-owned checks")
    return {"gcc10-final-regenerated-source-set", "binutils-configure-set-autoconf-2.69",
            "binutils-parser-scanner-set-bison-flex"}


def observe(op, contract, build_log, view):
    row_id = contract["row_id"]
    output = op.artifacts[row_id]
    root = output["physical"]
    work = op.root / "matrix" / row_id
    work.mkdir(parents=True)
    source = (op.source / contract["derivation"]).read_text()
    if row_id == "gcc.4.0":
        source += (op.source / "bootstrap/gcc-4.0-musl-cxx.ncl").read_text()
    for snippet in SOURCE_CHECKS[row_id]:
        check(snippet in source, f"selected source-owned matrix missing {row_id}: {snippet}")
    check(contract["success_marker"] in build_log,
          f"fresh build log lacks terminal source-owned success marker: {row_id}")
    marker_paths = [nonempty(root / "share/mantle-bootstrap" / name) for name in MARKERS[row_id]]
    marker_bytes = b"\n".join(path.read_bytes() for path in marker_paths)
    check(b"claim:" in marker_bytes or row_id == "binutils.tcc", f"missing actual output acceptance marker: {row_id}")
    if row_id == "binutils.tcc":
        marker = json.loads(marker_bytes)
        check(marker.get("schema") == "mantle-binutils-tcc-row-receipt-v1"
              and marker.get("status") == "complete-candidate" and marker.get("fallback_events") == [],
              "source-owned binutils candidate marker invalid")
        probes = binutils_probe(root, work, final=True)
    elif row_id == "full-musl-binutils":
        probes = binutils_probe(root, work, final=True)
        for component, filename in (("gcc10-final-cycle", "final-native-gcc10-cycle.txt"),
                                    ("musl-final", "final-native-musl-row.txt")):
            nonempty(op.artifacts[component]["physical"] / "share/mantle-bootstrap" / filename)
            component_log = Path(op.outcomes[{"gcc10-final-cycle": "gcc-10.5.0-musl-final-v1",
                                               "musl-final": "musl-1.2.5-gcc10-v1"}[component]]["log_file"])
            check("matrix passed" in component_log.read_text(errors="replace"),
                  f"component {component} source-owned matrix did not complete")
    else:
        standard = {"gcc.4.0": "c++98", "gcc.4.7": "c++11", "gcc.10": "c++17"}[row_id]
        probes = compiler_probe(root, work, cpp_standard=standard)
    generated = generation(row_id, output, op, build_log, work)
    # The source-owned builder's terminal echo follows its positive, malformed,
    # relocated, and generated-product tests. Independently rerun bounded malformed
    # probes above; all remaining categories require source-owned execution rather
    # than a report boolean. No status is accepted on cached output.
    check(row_id in op.artifacts and op.outcomes[output["report_label"]]["cached"] is False,
          "matrix was not tied to freshly executed selected source")
    for policy in contract["source_policies"]:
        text = (op.source / policy["path"]).read_text()
        check(all(x in text for x in policy["required"])
              and all(x not in text for x in policy["forbidden"]),
              f"source-owned fallback policy failed: {policy['path']}")
    forbidden = (b"/usr/bin/", b"/home/", b".pi/cairn-drain/")
    for binary in (root / "bin").iterdir():
        if binary.is_file() and not binary.is_symlink() and binary.stat().st_size < 100 * 1024 * 1024:
            check(not any(token in binary.read_bytes() for token in forbidden),
                  f"ambient fallback path embedded in selected output: {binary}")
    origin = {"recipe_source": contract["derivation"],
              "build_log": str(op.outcomes[output["report_label"]]["log_file"]),
              "output_marker_paths": [str(p) for p in marker_paths],
              "matrix_probe_dir": str(work), "observed_probes": probes,
              "signed_output_nar_sha256": output["info"]["nar_sha256"],
              "signed_output_nar_size": output["info"]["nar_size"],
              "source_owned_terminal_marker": contract["success_marker"]}
    return {"positive": contract["positive"], "rejection": contract["rejection"],
            "generated": sorted(generated), "origin": origin,
            "fallback": {"host_tool_execution": False, "predecessor_delegation": False,
                         "release_generated_substitution": False,
                         "configure_bridge_compiler_use": False,
                         "fabricated_objects": [], "omitted_artifacts": []}}
