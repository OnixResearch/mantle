#!/usr/bin/env python3
"""Real generic-Cargo -> existing-APK proof; no downloads, fake SDK or device claims.

All source/tool/store admission happens through Mantle before this driver starts.
See docs/android-apk-adapter.md for the run-manifest contract and exact cohort.
"""
import argparse
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import zipfile

PAGE = 16384
MAX_FILE = 512 * 1024 * 1024
HOST_MARKER = Path("/tmp/mantle-native-undeclared-proof")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def read_json(path):
    return json.loads(Path(path).read_text())


def identical_streams(left, right):
    while True:
        block = left.read(65536)
        if block != right.read(65536):
            return False
        if not block:
            return True


class Proof:
    def __init__(self, manifest, output):
        self.config = read_json(manifest)
        self.output = output
        output.mkdir(mode=0o700, parents=True, exist_ok=False)
        self.environment = {key: value for key, value in os.environ.items()
                            if not key.startswith(("CARGO_", "RUSTUP_", "ANDROID_", "NDK_"))}
        self.environment.update(LC_ALL="C", TZ="UTC")
        self.pins = {entry["path"]: entry["blake3"] for entry in self.config["pinned_files"]}
        require(len(self.pins) == len(self.config["pinned_files"]), "duplicate proof input pin")
        required = ["mantle", "apk_adapter", "b3sum", "apk_plan_factory", "apk_inputs",
                    "changed_asset_inputs", "native_recipe", "changed_native_recipe"]
        for field in required:
            require(self.config[field] in self.pins, f"missing BLAKE3 pin for {field}")
        for case in self.config["negative_native_recipes"].values():
            require(case["recipe"] in self.pins, "negative recipe must be pinned")
        for prefix in self.config["tools"].values():
            require(prefix and Path(prefix[0]).is_absolute(), "verifier command needs explicit executable")
            for argument in prefix:
                if Path(argument).is_file():
                    require(argument in self.pins, f"verifier member not pinned: {argument}")
        require(self.config["store"] == self.config["store_prefix"],
                "this proof requires one exact logical/physical store namespace")
        self.check_pins()

    def run(self, argv, label, success=True):
        result = subprocess.run(argv, env=self.environment, text=True, capture_output=True, timeout=3600)
        (self.output / f"{label}.stdout").write_text(result.stdout)
        (self.output / f"{label}.stderr").write_text(result.stderr)
        require((result.returncode == 0) == success,
                f"{label}: expected {'success' if success else 'rejection'}, got {result.returncode}")
        return result

    def digest(self, path):
        result = subprocess.run([self.config["b3sum"], str(path)], env=self.environment,
                                text=True, capture_output=True, check=True, timeout=60)
        digest = result.stdout.split()[0]
        require(re.fullmatch(r"[0-9a-f]{64}", digest), "invalid BLAKE3 tool result")
        return digest

    def check_pins(self):
        for path, expected in self.pins.items():
            require(re.fullmatch(r"[0-9a-f]{64}", expected), "invalid pinned BLAKE3")
            require(self.digest(path) == expected, f"proof input content drift: {path}")

    def build(self, recipe, label, success=True):
        cfg = self.config
        result = self.run([cfg["mantle"], "--json", "--store", cfg["store"], "--store-prefix", cfg["store_prefix"],
                           "--state-dir", cfg["state_dir"], "build", str(recipe), "--offline-source-preflight",
                           "--no-substitute", "--signing-key", cfg["store_signing_key"],
                           "--trusted-public-keys", cfg["store_trusted_public_key"]], label, success)
        report = json.loads(result.stdout)
        if success:
            require(report["counts"]["failed_total"] == 0, f"{label}: failed derivations")
        return report

    def native(self, recipe, label, warm=False):
        report = self.build(recipe, label)
        plans = report.get("native_dynamic_plans", [])
        require(len(plans) == 1 and plans[0].get("accepted_unit_ids"), "one admitted native plan required")
        admission = plans[0]
        require(not admission.get("rejection_reason"), "native plan rejected")
        plan = read_json(admission["plan_artifact_path"])
        units = {unit["id"]: unit for unit in plan["units"]}
        require(set(admission["accepted_unit_ids"]) == set(units), "accepted native unit set differs")
        producers = [row for row in report["outcomes"] if row["drv_key"] == admission["producer_key"]]
        require(len(producers) == 1, "native producer observation missing")
        producer_output = next(output["path"] for output in producers[0]["outputs"] if output["name"] == "out")
        identities = read_json(Path(producer_output) / "cargo-unit-bindings.json")
        require(identities["schema"] == "mantle-cargo-unit-bindings-v1", "native binding schema differs")
        require(set(identities["roots"]) == set(plan["roots"]), "native root identity differs")
        roots = [unit for unit in identities["roots"] if identities["units"][unit]["target_kind"] == "cdylib"]
        require(len(roots) == 1, "one declared cdylib root required")
        identity = identities["units"][roots[0]]
        require(identity["triple"] == "aarch64-linux-android" and identity["execution_kind"] == "target",
                "cdylib root is not an Android target unit")
        host_kinds = set()
        for unit_id, fact in identities["units"].items():
            if fact["execution_kind"] not in ("host", "host-dependency"):
                continue
            require(fact["triple"] == "x86_64-unknown-linux-gnu", "host unit crossed Android compilation boundary")
            if fact["target_kind"] not in ("custom-build", "proc-macro"):
                continue
            unit_rows = [row for row in report["outcomes"] if row["drv_key"].endswith("-" + units[unit_id]["derivation"]["name"] + ".drv")]
            require(len(unit_rows) == 1, "host compiler unit was not realized")
            artifacts = self.artifacts(unit_rows[0])
            artifacts = [path for path in artifacts if path.name.endswith(".so") or path.parent.name == "bin"]
            require(len(artifacts) == 1, "one host compiler artifact required")
            with artifacts[0].open("rb") as stream:
                header = stream.read(20)
            require(header[:7] == b"\x7fELF\x02\x01\x01" and struct.unpack_from("<H", header, 18)[0] == 62,
                    "host build script/proc macro is not x86_64 ELF")
            host_kinds.add(fact["target_kind"])
        require(host_kinds == {"custom-build", "proc-macro"}, "fixture must exercise host build.rs and proc macros")
        (self.output / f"{label}-unit-identities.json").write_text(json.dumps(identities, indent=2))
        (self.output / f"{label}-admitted-plan.json").write_text(json.dumps(plan, indent=2))
        if warm:
            require(report["counts"]["built_total"] == 0, "unchanged native graph rebuilt")
        unit_rows = [row for row in report["outcomes"] if row["drv_key"].endswith("-" + units[roots[0]]["derivation"]["name"] + ".drv")]
        require(len(unit_rows) == 1, "native cdylib root was not realized")
        candidates = [(path, unit_rows[0]["cached"]) for path in self.artifacts(unit_rows[0])
                      if path.name.startswith(self.config["artifact_stem"]) and path.name.endswith(".so")]
        require(len(candidates) == 1, "expected exactly one target cdylib artifact")
        if not warm:
            require(not candidates[0][1], "cold/changed native artifact was already cached")
        return candidates[0][0]

    @staticmethod
    def artifacts(outcome):
        root = Path(next(output["path"] for output in outcome["outputs"] if output["name"] == "out"))
        value = read_json(root / "share/mantle/unit-dependencies-v1.json")
        require(value["schema"] == "mantle-unit-dependencies-v1", "invalid compiler artifact manifest")
        for artifact in value["artifacts"]:
            require(not Path(artifact).is_absolute() and ".." not in Path(artifact).parts,
                    "escaping native artifact manifest")
        return [root / artifact for artifact in value["artifacts"]]

    @staticmethod
    def nickel_path(path):
        require(re.fullmatch(r"[A-Za-z0-9/._+-]+", str(path)), "unsafe Nickel import path")
        return json.dumps(str(path))

    def package(self, library, inputs, label, warm=False):
        bindings = {"library_store_file": str(library), "ndk_sysroot": self.config["ndk_sysroot"],
                    "api_level": self.config.get("api_level", 24), "signing": self.config.get("apk_signing")}
        bindings_path = self.output / f"{label}-bindings.json"
        bindings_path.write_text(json.dumps(bindings))
        plan = self.output / f"{label}-plan.ncl"
        plan.write_text(f"let make = import {self.nickel_path(self.config['apk_plan_factory'])} in "
                        f"make (import {self.nickel_path(bindings_path)})\n")
        graph = self.output / f"{label}-graph.ncl"
        rendered = self.run([self.config["apk_adapter"], "--plan", str(plan), "--inputs", inputs,
                             "--store-prefix", self.config["store_prefix"], "--output", str(graph)], f"{label}-prepare")
        receipt = json.loads(rendered.stdout)
        report = self.build(graph, f"{label}-build")
        require((report["counts"]["built_total"] == 0) if warm else (report["counts"]["built_total"] > 0),
                "incorrect APK warm reuse/invalidation")
        apk = Path(receipt["final_output"])
        require(apk.is_file(), "final APK was not exported")
        observed = self.inspect(apk, library, read_json(inputs)["source_root"], label)
        return plan, observed

    def elf(self, path, label, aligned=True):
        require(path.stat().st_size <= MAX_FILE, "oversized inspected ELF")
        data = path.read_bytes()
        require(data[:7] == b"\x7fELF\x02\x01\x01", "not little-endian ELF64")
        require(struct.unpack_from("<HH", data, 16) == (3, 183), "not an AArch64 shared library")
        offset = struct.unpack_from("<Q", data, 32)[0]
        stride, count = struct.unpack_from("<HH", data, 54)
        require(stride == 56 and 0 < count <= 128, "invalid ELF program headers")
        loads = []
        android_api_notes = []
        for index in range(count):
            kind, _, file_offset, address, _, file_size, memory_size, alignment = struct.unpack_from("<IIQQQQQQ", data, offset + index * stride)
            if kind == 4:
                cursor, end = file_offset, file_offset + file_size
                require(end <= len(data) and file_size <= 65536, "invalid ELF note segment")
                while cursor < end:
                    require(cursor + 12 <= end, "truncated ELF note header")
                    name_size, value_size, note_kind = struct.unpack_from("<III", data, cursor)
                    cursor += 12
                    name = data[cursor:cursor + name_size]
                    cursor += (name_size + 3) & ~3
                    require(cursor + value_size <= end, "truncated ELF note value")
                    if name == b"Android\0" and note_kind == 1:
                        require(value_size >= 4, "Android API note has no API value")
                        android_api_notes.append(struct.unpack_from("<I", data, cursor)[0])
                    cursor += (value_size + 3) & ~3
                require(cursor == end, "invalid ELF note padding")
            if kind == 1:
                require(file_offset + file_size <= len(data) and file_size <= memory_size, "invalid ELF load bounds")
                if aligned:
                    require(alignment >= PAGE and alignment & (alignment - 1) == 0
                            and file_offset % PAGE == address % PAGE, "ELF load is not 16KiB aligned")
                loads.append({"offset": file_offset, "address": address, "alignment": alignment})
        require(loads, "ELF has no loadable segments")
        if aligned:
            require(android_api_notes == [self.config.get("api_level", 24)], "ELF Android API note differs from plan")
        symbols = self.run(self.config["tools"]["readelf"] + ["--wide", "--dyn-syms", str(path)], label + "-symbols").stdout
        exports, imports = set(), set()
        for line in symbols.splitlines():
            fields = line.split()
            if len(fields) < 8 or not re.fullmatch(r"\d+:", fields[0]):
                continue
            name = fields[7].split("@")[0]
            if fields[6] != "UND" and fields[4] in ("GLOBAL", "WEAK") and fields[5] in ("DEFAULT", "PROTECTED"):
                exports.add(name)
            if fields[6] == "UND" and fields[4] == "GLOBAL":
                imports.add(name)
        dynamic = self.run(self.config["tools"]["readelf"] + ["--dynamic", str(path)], label + "-dynamic").stdout
        require(not re.search(r"\((?:RPATH|RUNPATH|TEXTREL)\)", dynamic), "forbidden ELF dynamic search/relocation tag")
        needed = re.findall(r"\(NEEDED\).*?\[([^]]+)\]", dynamic)
        require(all(re.fullmatch(r"[A-Za-z0-9._+-]+", name) for name in needed), "unsafe ELF dependency")
        return {"loads": loads, "exports": exports, "imports": imports, "needed": needed}

    def inspect(self, apk, library, source, label):
        members, assets = [], {}
        expected_library = "lib/arm64-v8a/" + self.config["library_filename"]
        source = Path(source)
        expected_assets = {"assets/" + item["destination"]: source / item["source_member"] for item in self.config["assets"]}
        with zipfile.ZipFile(apk) as archive, apk.open("rb") as raw:
            names = archive.namelist()
            require(len(names) == len(set(names)), "duplicate APK members")
            require("AndroidManifest.xml" in names and "classes.dex" in names, "missing Android/DEX payload")
            actual_native = [name for name in names if name.endswith(".so")]
            require(actual_native == [expected_library], "undeclared APK native library or ABI")
            for entry in archive.infolist():
                require(not entry.filename.startswith("/") and ".." not in Path(entry.filename).parts,
                        "escaping APK member")
                require(not entry.flag_bits & 1, "encrypted APK member")
                require(entry.file_size <= MAX_FILE, "oversized APK member")
                if entry.filename == expected_library:
                    require(entry.compress_type == zipfile.ZIP_STORED, "APK .so must be uncompressed")
                    raw.seek(entry.header_offset)
                    header = raw.read(30)
                    require(header[:4] == b"PK\x03\x04", "invalid ZIP local header")
                    name_bytes, extra_bytes = struct.unpack_from("<HH", header, 26)
                    payload_offset = entry.header_offset + 30 + name_bytes + extra_bytes
                    require(payload_offset % PAGE == 0, "APK .so payload is not 16KiB aligned")
                    with archive.open(entry) as payload, library.open("rb") as built:
                        require(identical_streams(payload, built), "APK native bytes differ from admitted compiler output")
                    members.append({"member": entry.filename, "payload_offset": payload_offset})
                if entry.filename.startswith("assets/") and not entry.is_dir():
                    require(entry.filename in expected_assets, "undeclared APK asset")
                    source_file = expected_assets[entry.filename]
                    with archive.open(entry) as payload, source_file.open("rb") as admitted:
                        require(identical_streams(payload, admitted), "APK asset changed")
                    assets[entry.filename] = self.digest(source_file)
            require(set(assets) == set(expected_assets), "missing APK asset")
        parsed = self.elf(library, label + "-native")
        require(set(self.config["required_jni_exports"]) <= parsed["exports"], "JNI exports absent")
        closure, pending = {self.config["library_filename"]: parsed}, set(parsed["needed"])
        while pending:
            name = pending.pop()
            if name in closure:
                continue
            require(len(closure) < 256, "oversized ELF dependency closure")
            stub = Path(self.config["ndk_sysroot"]) / "usr/lib/aarch64-linux-android" / str(self.config.get("api_level", 24)) / name
            item = self.elf(stub, label + "-stub-" + name, aligned=False)
            pending.update(item["needed"])
            closure[name] = item
        exports = set().union(*(item["exports"] for item in closure.values()))
        require(parsed["imports"] <= exports, "API-stub/native dependency closure lacks strong imports")
        self.run(self.config["tools"]["zipalign"] + ["-c", "-P", "16", "4", str(apk)], label + "-zipalign")
        badging = self.run(self.config["tools"]["aapt2"] + ["dump", "badging", str(apk)], label + "-badging").stdout
        require(f"sdkVersion:'{self.config.get('api_level', 24)}'" in badging, "APK minSdk differs from native API")
        signed = bool(self.config.get("apk_signing"))
        if signed:
            verifier = self.config["tools"]["apksigner"]
            self.run(verifier + ["verify", "--verbose", "--print-certs", str(apk)], label + "-signature")
            tampered = self.output / f"{label}-tampered.apk"
            tampered.write_bytes(apk.read_bytes())
            with zipfile.ZipFile(apk) as archive:
                dex = archive.getinfo("classes.dex")
            with tampered.open("r+b") as file:
                file.seek(dex.header_offset + 26)
                name_size, extra_size = struct.unpack("<HH", file.read(4))
                file.seek(dex.header_offset + 30 + name_size + extra_size)
                byte = file.read(1)
                require(byte, "empty signed DEX payload")
                file.seek(-1, 1)
                file.write(bytes([byte[0] ^ 1]))
            self.run(verifier + ["verify", str(tampered)], label + "-tampered-signature", success=False)
        return {"apk_blake3": self.digest(apk), "native_blake3": self.digest(library),
                "native_members": members, "needed": sorted(closure), "signature_verified": signed,
                "assets": assets}

    def negatives(self, plan):
        cfg = self.config
        mismatched_api = 35 if cfg.get("api_level", 24) != 35 else 24
        cases = [("wrong-abi", "abi", "x86_64", "InvalidNativeInput"), ("wrong-api", "api_level", 23, "InvalidNativeInput"),
                 ("wrong-ndk", "ndk_revision", "28.0.0", "InvalidNativeInput"),
                 ("elf-api-mismatch", "api_level", mismatched_api, "Android ELF API note")]
        for name, field, value, diagnostic in cases:
            invalid = self.output / f"{name}.ncl"
            invalid.write_text(f"let plan = import {self.nickel_path(plan)} in "
                               f"std.record.update \"native\" (std.record.update {json.dumps(field)} "
                               f"{json.dumps(value)} plan.native) plan\n")
            result = self.run([cfg["apk_adapter"], "--plan", str(invalid), "--inputs", cfg["apk_inputs"],
                               "--store-prefix", cfg["store_prefix"], "--output", str(self.output / f"{name}-graph.ncl")],
                              name, success=False)
            require(diagnostic in result.stderr or diagnostic.lower() in result.stderr,
                    f"{name}: wrong rejection category")
        cases = cfg["negative_native_recipes"]
        require(set(cases) == {"wrong-toolchain", "missing-dependency", "undeclared-input", "network"}, "incomplete native negative matrix")
        diagnostics = {"wrong-toolchain": "can't find crate for", "missing-dependency": "failed to load manifest",
                       "undeclared-input": "undeclared input denied by native sandbox", "network": "network attempt denied by native sandbox"}
        require(all(case["diagnostic"] == diagnostics[name] for name, case in cases.items()), "negative diagnostic contract drift")
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
            listener.bind(("127.0.0.1", 41237))
            listener.listen(2)
            with socket.create_connection(("127.0.0.1", 41237), timeout=1):
                control, _ = listener.accept()
                control.close()
            with HOST_MARKER.open("x") as marker:
                marker.write("Present on host; MUST NOT be readable in native build sandbox.\n")
            try:
                for name, case in cases.items():
                    report = self.build(case["recipe"], "negative-" + name, success=False)
                    messages = json.dumps(report)
                    for failure in report.get("failed", []):
                        if failure.get("saved_log_path"):
                            log = Path(failure["saved_log_path"])
                            require(log.resolve().is_relative_to(Path(cfg["state_dir"]).resolve()), "failure log escaped state directory")
                            require(log.stat().st_size <= 4 * 1024 * 1024, "oversized negative failure log")
                            messages += log.read_text(errors="replace")
                    require(case["diagnostic"] and case["diagnostic"] in messages, f"{name}: expected clear diagnostic absent")
            finally:
                HOST_MARKER.unlink()

    def prove(self):
        cfg = self.config
        library = self.native(cfg["native_recipe"], "native-cold")
        plan, cold = self.package(library, cfg["apk_inputs"], "apk-cold")
        reused = self.native(cfg["native_recipe"], "native-warm", warm=True)
        require(reused == library, "native warm output changed")
        _, warm = self.package(reused, cfg["apk_inputs"], "apk-warm", warm=True)
        require(cold == warm, "APK warm output changed")
        changed_library = self.native(cfg["changed_native_recipe"], "native-changed")
        _, native_change = self.package(changed_library, cfg["apk_inputs"], "apk-native-changed")
        require(native_change["native_blake3"] != cold["native_blake3"] and native_change["apk_blake3"] != cold["apk_blake3"], "native change failed to invalidate APK")
        same_library = self.native(cfg["native_recipe"], "native-for-asset-change", warm=True)
        _, asset_change = self.package(same_library, cfg["changed_asset_inputs"], "apk-asset-changed")
        require(asset_change["native_blake3"] == cold["native_blake3"] and asset_change["assets"] != cold["assets"]
                and asset_change["apk_blake3"] != cold["apk_blake3"], "asset/native invalidation boundary failed")
        self.negatives(plan)
        self.check_pins()
        return {"schema": "mantle-android-native-build-structure-proof-v1", "cold": cold, "warm": warm,
                "native_change": native_change, "asset_change": asset_change,
                "negative_cases": ["wrong-abi", "wrong-api", "wrong-ndk", "elf-api-mismatch", *cfg["negative_native_recipes"]],
                "build_structure": "observed", "device_execution": "not-performed",
                "representative_workload": cfg["workload_identity"], "pinned_inputs": cfg["pinned_files"],
                "api_level": cfg.get("api_level", 24), "verifiers": cfg["tools"],
                "non_claim": "no install, launch, device JNI execution, release credential creation or publication"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    proof = Proof(args.manifest.resolve(), args.output.resolve())
    receipt = proof.prove()
    (proof.output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"receipt": str(proof.output / "receipt.json"), "device_execution": "not-performed"}))


if __name__ == "__main__":
    main()
