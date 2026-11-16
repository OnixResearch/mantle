#!/usr/bin/env python3
"""Signed StageX native-row operator. Requires a completed, externally pinned adoption.

No discovery of toolchains, caches, or compilers. A failed run retains its evidence,
never writes a successful receipt set, and never alters the frozen source checkout.
"""
# r[impl bootstrap_inventory.cold_stagex_native_row_adoption]
import argparse
import os
import json
from pathlib import Path
import re
import stat
import signal
import subprocess
import time
import sys

import native_row_qualification_matrix as matrix
import native_row_qualification_profile as profile_reader

PREFIX = "/mantle/store"
RESERVED = {
    "stagex-transition": PREFIX + "/ki5gkg5d6si77dl5k4mav4s6x9s8l25r-mantle-stagex-transition",
    "stagex-provider": PREFIX + "/snzd91n8dv6l21xa89vml67229n9svkg-mantle-stagex-intermediate-provider",
}
ROOTS = [
    ("binutils.tcc", "bootstrap/binutils-tcc.ncl", "binutils-2.30-tcc-source-v1"),
    ("gcc.4.0", "bootstrap/gcc-4.0.ncl", "gcc-4.0.4-musl-cxx-v8"),
    ("gcc.4.7", "bootstrap/gcc-4.7.ncl", "gcc-4.7.4-musl-gcc40-v1"),
    ("gcc.10", "bootstrap/gcc-10.ncl", "gcc-10.5.0-musl-gcc47-v2"),
    ("musl-final", "bootstrap/musl-full.ncl", "musl-1.2.5-gcc10-v1"),
    ("gcc10-final-cycle", "bootstrap/gcc-10-final.ncl", "gcc-10.5.0-musl-final-v1"),
    ("full-musl-binutils", "bootstrap/binutils-full.ncl", "binutils-2.41-gcc10-v1"),
]
# Exact current derivation names, from the selected bootstrap/*.ncl `name` declarations.
PREDECESSORS = {
    "bison-2.3": "bison-2.3-musl", "flex-2.6.4": "flex-2.6.4-musl",
    "gawk-3.0.4": "gawk-3.0.4-musl", "m4-1.4.7": "m4-1.4.7-musl",
    "native-musl": "musl-1.1.24-native-candidate", "tcc-v2": "tcc-0.9.27-musl-v2",
    "early-binutils": "binutils-2.30-tcc-source-v1",
    "gcc-built-binutils": "binutils-2.30-gcc-pass4-v17",
    "gcc-pass6": "gcc-4.0.4-musl-pass6-v1",
    "musl-pass5": "musl-1.1.24-gcc-pass5-v1",
    "native-tcc": "tcc-0.9.27-musl-native-runtime",
    "gcc40": "gcc-4.0.4-musl-cxx-v8", "binutils": "binutils-2.30-gcc-pass4-v17",
    "musl": "musl-1.1.24-gcc-pass5-v1", "gmp": "gmp-6.2.1-gcc40-v1",
    "mpfr": "mpfr-4.1.0-gcc40-v1", "mpc": "mpc-1.2.1-gcc40-v1",
    "autoconf": "autoconf-2.64-gcc-v1", "bison": "bison-3.4.1-gcc-v2",
    "flex": "flex-2.6.4-gcc-v1", "gcc47": "gcc-4.7.4-musl-gcc40-v1",
    "regenerated-source": "gcc-10.5.0-regenerated-source-v1",
    "gcc10": "gcc-10.5.0-musl-gcc47-v2",
    "gcc10-final-cycle": "gcc-10.5.0-musl-final-v1", "musl-final": "musl-1.2.5-gcc10-v1",
}
HEX = re.compile(r"[0-9a-f]{64}\Z")
VERIFY = re.compile(r"OK (\S+)  trusted_signatures=([1-9][0-9]*)/([1-9][0-9]*)\Z")
MAX_JSON = 16 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def bounded_file(path, limit=MAX_JSON):
    path = Path(path)
    meta = path.lstat()
    require(stat.S_ISREG(meta.st_mode) and 0 < meta.st_size <= limit,
            f"expected bounded regular file: {path}")
    return path.read_bytes()


def document(path):
    return json.loads(bounded_file(path))


def command(argv, *, cwd=None, output, timeout=1800, stdout_limit=64 * 1024 * 1024):
    output.parent.mkdir(parents=True, exist_ok=True)
    stdout_path = output.with_suffix(output.suffix + ".stdout")
    stderr_path = output.with_suffix(output.suffix + ".stderr")
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        child = subprocess.Popen([str(a) for a in argv], cwd=cwd, stdin=subprocess.DEVNULL,
                                 stdout=stdout, stderr=stderr, start_new_session=True)
        deadline = time.monotonic() + timeout
        try:
            while child.poll() is None:
                if stdout_path.stat().st_size > stdout_limit or stderr_path.stat().st_size > 16 * 1024 * 1024:
                    raise RuntimeError(f"command output exceeds retained bound: {argv[0]}")
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise RuntimeError(f"command exceeded {timeout}s deadline: {argv[0]}")
                try:
                    child.wait(timeout=min(remaining, 1))
                except subprocess.TimeoutExpired:
                    pass
        except (RuntimeError, KeyboardInterrupt):
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise
    require(stdout_path.stat().st_size <= stdout_limit and stderr_path.stat().st_size <= 16 * 1024 * 1024,
            f"command output exceeds retained bound: {argv[0]}")
    if child.returncode:
        with stderr_path.open("rb") as error:
            error.seek(max(0, stderr_path.stat().st_size - 2000))
            detail = error.read()
        raise RuntimeError(f"command failed (exit {child.returncode}): {argv[0]} {argv[1:4]}: {detail!r}")
    return stdout_path.read_text(encoding="utf-8").strip()


def selected_executable(path):
    require(path.is_absolute() and not path.is_symlink(), f"selected tool must be an absolute regular path: {path}")
    metadata = path.stat()
    require(stat.S_ISREG(metadata.st_mode) and 0 < metadata.st_size <= 1024 * 1024 * 1024,
            f"selected executable must be a bounded regular file: {path}")


class Operator:
    def __init__(self, a):
        self.a = a
        self.source = a.source_root.resolve(strict=True)
        self.root = a.run_root.resolve(strict=False)
        self.store = Path()
        self.state = Path()
        self.trust = ""
        self.contracts = {}
        self.outcomes = {}
        self.artifacts = {}
        self.sequence = 0

    def next_output(self, category, label):
        self.sequence += 1
        return self.root / category / f"{self.sequence:05d}-{label}.log"

    def checker(self, *args):
        return command([self.a.checker, *args], output=self.next_output("checker", str(args[0]).lstrip("-")))

    def digest(self, path):
        value = command([self.a.b3sum, "--no-names", "--", path],
                        output=self.next_output("hashes", "b3sum"), timeout=self.a.build_seconds)
        require(HEX.fullmatch(value), f"invalid BLAKE3 from selected b3sum for {path}")
        return value
    def source_tree_digest(self, phase):
        # Exact normalized GNU-tar stream used by the held cold driver; no archive
        # is materialized, and neither tar nor b3sum is selected from PATH.
        base = self.next_output("source-guard", phase)
        base.parent.mkdir(parents=True, exist_ok=True)
        tar_error = base.with_suffix(".tar.stderr")
        hash_error = base.with_suffix(".b3sum.stderr")
        digest_file = base.with_suffix(".b3sum.stdout")
        tar_args = [self.a.tar, "--sort=name", "--mtime=@0", "--owner=0", "--group=0",
                    "--numeric-owner", "--format=gnu", "--create", "--file", "-",
                    "--directory", self.source, "."]
        with tar_error.open("wb") as tar_stderr, hash_error.open("wb") as hash_stderr, digest_file.open("wb") as hash_stdout:
            tar = subprocess.Popen([str(arg) for arg in tar_args], stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=tar_stderr, start_new_session=True)
            try:
                hasher = subprocess.Popen([str(self.a.b3sum), "--no-names"], stdin=tar.stdout,
                                          stdout=hash_stdout, stderr=hash_stderr, start_new_session=True)
            except BaseException:
                try:
                    os.killpg(tar.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                tar.wait()
                raise
            tar.stdout.close()
            deadline = time.monotonic() + self.a.build_seconds
            try:
                while tar.poll() is None or hasher.poll() is None:
                    if digest_file.stat().st_size > 4096 or tar_error.stat().st_size > 1024 * 1024 or hash_error.stat().st_size > 1024 * 1024:
                        raise RuntimeError(f"{phase} source guard output exceeds bound")
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise RuntimeError(f"{phase} source guard exceeded {self.a.build_seconds}s deadline")
                    active = hasher if hasher.poll() is None else tar
                    try:
                        active.wait(timeout=min(remaining, 1))
                    except subprocess.TimeoutExpired:
                        pass
            except BaseException:
                for child in (tar, hasher):
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    child.wait()
                raise
        require(tar.returncode == 0 and hasher.returncode == 0
                and digest_file.stat().st_size <= 4096
                and tar_error.stat().st_size <= 1024 * 1024 and hash_error.stat().st_size <= 1024 * 1024,
                f"{phase} normalized source archive/hash failed")
        value = digest_file.read_text(encoding="utf-8").strip()
        require(HEX.fullmatch(value) and value == self.a.expected_source_tree_blake3,
                f"{phase} complete frozen source tree differs from independent BLAKE3 pin")
        return value

    def cli(self, *args, label, timeout=1800):
        return command([self.a.mantle, "--json", "--store", self.store,
                        "--state-dir", self.state, "--store-prefix", PREFIX,
                        "--store-backend", self.a.store_backend, *args], cwd=self.source,
                       output=self.next_output("commands", label), timeout=timeout)

    def read_store(self, logical, label):
        require(logical.startswith(PREFIX + "/") and logical.count("/") == 3,
                f"not a direct logical store path: {logical}")
        basename = logical[len(PREFIX) + 1:]
        info = json.loads(self.cli("store", "info", basename, label=label + "-info"))
        require(info.get("schema") == "mantle-store-info-v3" and info.get("backend") == self.a.store_backend,
                f"store info schema/backend mismatch: {logical}")
        paths = info.get("paths")
        require(isinstance(paths, list) and len(paths) == 1 and paths[0].get("store_path") == basename,
                f"store info not an exact unique PathInfo: {logical}")
        item = paths[0]
        require(item.get("layer") == {"kind": "overlay"} and not item.get("shadows"),
                f"path selected from unexpected overlay: {logical}")
        require(HEX.fullmatch(item.get("nar_sha256", "")) and type(item.get("nar_size")) is int
                and item["nar_size"] > 0 and item.get("signatures"), f"incomplete signed PathInfo: {logical}")
        verified = self.cli("store", "verify", basename, "--signing-key", self.a.signing_key,
                            "--trusted-public-keys", self.trust, label=label + "-verify")
        matched = VERIFY.fullmatch(verified)
        require(matched and matched.group(1) in (logical, basename)
                and int(matched.group(2)) <= int(matched.group(3))
                and int(matched.group(3)) == len(item["signatures"]),
                f"current NAR/signature verification did not accept {logical}: {verified}")
        physical = self.store / basename
        require(physical.is_dir() and not physical.is_symlink(), f"missing current physical output: {physical}")
        return item, physical

    def adoption(self):
        a = self.a
        require(HEX.fullmatch(a.expected_adoption_blake3)
                and self.digest(a.adoption_report) == a.expected_adoption_blake3,
                "adoption report bytes do not match independently expected BLAKE3")
        report = document(a.adoption_report)
        require(report.get("schema") == "mantle-cold-stagex-store-adoption-v1"
                and report.get("adopted") is True and report.get("strict") is True
                and report.get("signing_key_selected") is True
                and report.get("logical_store_prefix") == PREFIX,
                "adoption is not a completed strict signed StageX publication")
        self.store = Path(report["output_dir"]).resolve(strict=True)
        self.state = Path(report["state_dir"]).resolve(strict=True)
        require(self.store != self.state and self.store not in self.source.parents
                and self.state not in self.source.parents and self.source not in self.store.parents
                and self.source not in self.state.parents, "source/store/state roots overlap")
        require(self.store != self.source and self.state != self.source, "source overlaps adoption")
        require(Path(report["proof_root"]).is_dir() and HEX.fullmatch(report.get("proof_receipt_bytes_blake3", ""))
                and HEX.fullmatch(report.get("proof_plan_digest_blake3", ""))
                and HEX.fullmatch(report.get("source_authority_digest_blake3", "")),
                "adoption lacks verified proof authority")
        self.trust = report.get("trusted_public_key", "")
        require(self.trust == self.checker("--public-key", a.signing_key),
                "selected signing key differs from the adoption signer")
        outputs = report.get("outputs")
        require(isinstance(outputs, list) and len(outputs) == 2
                and {o.get("role") for o in outputs} == set(RESERVED),
                "adoption must contain exactly the two reserved roles")
        for output in outputs:
            logical = RESERVED[output["role"]]
            require(output.get("logical_path") == logical and HEX.fullmatch(output.get("source_artifact_digest_blake3", ""))
                    and HEX.fullmatch(output.get("published_tree_digest_blake3", ""))
                    and isinstance(output.get("source_relative_path"), str)
                    and type(output.get("signature_count")) is int and output["signature_count"] > 0,
                    f"incomplete adoption output {logical}")
            info, _ = self.read_store(logical, "adoption-" + output["role"])
            require(info["nar_sha256"] == output.get("nar_sha256")
                    and info["nar_size"] == output.get("nar_size")
                    and len(info["signatures"]) == output["signature_count"],
                    f"reserved PathInfo differs from adoption: {logical}")
        (self.root / "adoption-binding.json").write_text(json.dumps({
            "report": str(a.adoption_report), "bytes_blake3": a.expected_adoption_blake3,
            "proof_receipt_bytes_blake3": report["proof_receipt_bytes_blake3"],
            "proof_plan_digest_blake3": report["proof_plan_digest_blake3"],
            "source_authority_digest_blake3": report["source_authority_digest_blake3"],
            "outputs": outputs}, indent=2) + "\n")

    def profile(self):
        a = self.a
        require(HEX.fullmatch(a.expected_manifest_blake3), "expected profile identity must be lowercase BLAKE3")
        # Mantle validates the complete bundle and every record before local indexing.
        verified = json.loads(self.cli("source", "bundle", "verify", "--from", a.source_profile,
                                       label="profile-verify", timeout=a.build_seconds))
        require(verified.get("manifest_blake3") == a.expected_manifest_blake3
                and verified.get("ready_class") == "ready", "source profile verification refused")
        source_paths = profile_reader.mantle_source_paths(
            a.source_profile, a.expected_manifest_blake3,
            {rel for contract in self.contracts.values() for rel in contract["sources"]})
        for rel, expected in source_paths.items():
            require(HEX.fullmatch(expected or "") and expected == self.digest(self.source / rel),
                    f"frozen row source does not match authenticated profile: {rel}")
        imported = json.loads(self.cli("source", "bundle", "import", "--from", a.source_profile,
                                       "--pin", label="profile-import", timeout=a.build_seconds))
        require(imported.get("manifest_blake3") == a.expected_manifest_blake3
                and imported.get("pinned") is True, "source import did not pin verified profile")
        current = json.loads(self.cli("source", "bundle", "verify", "--from", a.source_profile,
                                      "--imported", label="profile-imported-verify", timeout=a.build_seconds))
        require(current.get("ready_class") == "ready"
                and current.get("manifest_blake3") == a.expected_manifest_blake3,
                "authenticated source import incomplete")

    def build(self, row_id, recipe, package):
        source = self.source / recipe
        require(source.is_file() and not source.is_symlink(), f"missing selected root: {recipe}")
        args = ["build", source, "--import-path", self.source / "lib", "--strict-hermetic",
                "--no-substitute", "--offline-source-preflight", "--jobs", str(self.a.jobs),
                "--signing-key", self.a.signing_key, "--trusted-public-keys", self.trust]
        report = json.loads(self.cli(*args, label="build-" + row_id, timeout=self.a.build_seconds))
        require(report.get("file") == str(source) and report.get("store_dir") == PREFIX
                and report.get("output_dir") == str(self.store) and report.get("state_dir") == str(self.state)
                and report.get("hermeticity_mode") == "strict", f"build context mismatch: {row_id}")
        counts = report.get("counts", {})
        require(counts.get("failed_total") == 0 and not report.get("failed")
                and type(counts.get("built_total")) is int and counts["built_total"] > 0,
                f"build did not execute real selected work: {row_id}")
        outcomes = report.get("outcomes", [])
        require(all(not item.get("rejections") for item in report.get("build_environment_reports", []))
                and all("degraded" not in str(item).lower() and "fallback" not in str(item).lower()
                        for item in report.get("hermeticity_audit_events", [])),
                f"build environment or hermeticity audit reports a fallback: {row_id}")
        require(isinstance(outcomes, list) and outcomes, f"no build outcomes: {row_id}")
        for outcome in outcomes:
            label = outcome.get("label")
            require(isinstance(label, str) and label, f"unlabelled build outcome: {row_id}")
            if label in self.outcomes:
                previous = self.outcomes[label]
                require(previous.get("drv_key") == outcome.get("drv_key")
                        and [(o.get("name"), o.get("path"), o.get("artifact_attestation"))
                             for o in previous.get("outputs", [])]
                        == [(o.get("name"), o.get("path"), o.get("artifact_attestation"))
                            for o in outcome.get("outputs", [])],
                        f"predecessor identity drift across row builds: {label}")
            else:
                self.outcomes[label] = outcome
        root = [o for o in outcomes if o.get("label") == package]
        require(len(root) == 1 and root[0].get("cached") is False,
                f"selected row root was not freshly built: {row_id} ({package})")
        require(root[0].get("outputs") and len(root[0]["outputs"]) == 1
                and root[0]["outputs"][0].get("name") == "out", f"ambiguous root output: {row_id}")
        log = Path(root[0].get("log_file") or "")
        require(log.is_file(), f"missing root build log: {row_id}")
        self.artifacts[row_id] = self.observe_output(root[0], row_id)
        return bounded_file(log, 64 * 1024 * 1024).decode("utf-8", errors="replace")

    def observe_output(self, outcome, label):
        require(len(outcome.get("outputs", [])) == 1, f"ambiguous output {label}")
        output = outcome["outputs"][0]
        require(output.get("name") == "out" and not output.get("substitution"),
                f"output substituted or not out: {label}")
        logical = output.get("artifact_attestation", {}).get("logical_path", "")
        require(output.get("path") == str(self.store / logical.rsplit("/", 1)[-1]),
                f"physical output mismatch: {label}")
        info, physical = self.read_store(logical, "output-" + label)
        attestation_path = Path(output["artifact_attestation"]["path"])
        require(attestation_path.is_relative_to(self.state) and not attestation_path.is_symlink(),
                f"artifact attestation outside selected state: {label}")
        attestation = json.loads(self.checker("--artifact", attestation_path))
        facts = attestation["attestation"]["facts"]
        require(facts.get("logical_path") == logical and facts.get("output_name") == "out"
                and facts.get("content_digest") == "nar-sha256:" + info["nar_sha256"],
                f"artifact attestation does not match signed NAR facts: {label}")
        return {"logical": logical, "physical": physical, "info": info,
                "digest": attestation["digest"], "attestation": attestation["attestation"],
                "source": str(attestation_path), "label": label,
                "report_label": outcome["label"]}

    def source_copy(self, contract, view):
        records = []
        for rel in contract["sources"]:
            path = self.source / rel
            require(path.is_relative_to(self.source) and not path.is_symlink(), f"unsafe source {rel}")
            data = bounded_file(path, 2 * 1024 * 1024)
            dest = view / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            if dest.exists():
                require(bounded_file(dest, 2 * 1024 * 1024) == data,
                        f"shared source differs between row observations: {rel}")
            else:
                dest.write_bytes(data)
            digest = self.digest(dest)
            require(digest == self.digest(path), f"source drift during observation: {rel}")
            records.append({"path": rel, "blake3": digest})
        for policy in contract["source_policies"]:
            text = (view / policy["path"]).read_text()
            require(all(s in text for s in policy["required"])
                    and all(s not in text for s in policy["forbidden"]),
                    f"current source policy mismatch: {policy['path']}")
        return records

    @staticmethod
    def emit(view, rel, value):
        path = view / rel
        require(not path.exists(), f"refusing to overwrite evidence {rel}")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    def receipt(self, contract, observations, view):
        row_id = contract["row_id"]
        output = self.artifacts[row_id]
        predecessor_records = []
        for predecessor in contract["predecessors"]:
            role = predecessor["role"]
            label = PREDECESSORS[role]
            require(label in self.outcomes, f"missing current predecessor {row_id}/{role}/{label}")
            predecessor_output = self.observe_output(self.outcomes[label], row_id + "-" + role)
            envelope = {"kind": "artifact", "digest": predecessor_output["digest"],
                        "attestation": predecessor_output["attestation"]}
            self.emit(view, predecessor["path"], envelope) if not (view / predecessor["path"]).exists() else self.same_evidence(view, predecessor["path"], envelope)
            predecessor_records.append({"role": role, "logical_path": predecessor_output["logical"],
                                        "artifact_evidence_path": predecessor["path"],
                                        "artifact_attestation_blake3": predecessor_output["digest"]})
        self.emit(view, contract["artifact_evidence_path"], {
            "schema": contract["artifact_evidence_schema"], "row_id": row_id,
            "digest": output["digest"], "attestation": output["attestation"]})
        trust = {"trust_unsigned": False, "signing_key_selected": True, "substitutions_enabled": False}
        fallback = observations["fallback"]
        require(fallback == {"host_tool_execution": False, "predecessor_delegation": False,
                            "release_generated_substitution": False,
                            "configure_bridge_compiler_use": False,
                            "fabricated_objects": [], "omitted_artifacts": []},
                f"forbidden actual fallback in {row_id}")
        require(set(observations["positive"]) == set(contract["positive"])
                and set(observations["rejection"]) == set(contract["rejection"])
                and set(observations["generated"]) == set(contract["generated"]),
                f"matrix/regeneration observation incomplete: {row_id}")
        acceptance = {"schema": contract["acceptance_evidence_schema"], "row_id": row_id,
                      "logical_path": output["logical"], "success_marker": contract["success_marker"],
                      "positive": sorted(observations["positive"]), "rejection": sorted(observations["rejection"]),
                      "trust": trust, "fallback": fallback,
                      "evidence_origin": observations["origin"]}
        self.emit(view, contract["acceptance_evidence_path"], acceptance)
        receipt = {"schema": contract["schema"], "row_id": row_id,
                   "derivation": contract["derivation"], "source_records": self.source_copy(contract, view),
                   "predecessors": predecessor_records,
                   "generated_artifacts": [{"id": item, "status": "regenerated"} for item in sorted(observations["generated"])],
                   "acceptance_evidence_path": contract["acceptance_evidence_path"],
                   "output": {"logical_path": output["logical"],
                              "artifact_evidence_path": contract["artifact_evidence_path"],
                              "artifact_attestation_blake3": output["digest"],
                              "nar_sha256": output["info"]["nar_sha256"]},
                   "behavior": {"positive": [{"id": i, "status": "passed"} for i in sorted(observations["positive"])],
                                "rejection": [{"id": i, "status": "passed"} for i in sorted(observations["rejection"])]},
                   "trust": trust, "fallback": fallback,
                   "bounded_claim": f"Current {row_id} selected source-built output and bounded source-owned plus independently executed matrix under signed no-substitution acceptance.",
                   "non_claims": contract["non_claims"]}
        self.emit(view, contract["receipt_path"], receipt)

    @staticmethod
    def same_evidence(view, rel, expected):
        require(document(view / rel) == expected, f"shared predecessor evidence conflict: {rel}")

    def run(self):
        a = self.a
        require(not self.root.exists(), "run root must be fresh and absent")
        require(self.root != self.source and not self.root.is_relative_to(self.source),
                "run root must not live inside immutable source")
        for path in (a.mantle, a.checker, a.signing_key, a.source_profile, a.adoption_report):
            require(path.is_file() and not path.is_symlink(), f"required explicit input absent: {path}")
        for path in (a.mantle, a.checker, a.tar, a.b3sum):
            selected_executable(path)
        candidate_adoption = document(a.adoption_report)
        for key in ("output_dir", "state_dir", "proof_root"):
            protected = Path(candidate_adoption[key]).resolve(strict=True)
            require(self.root != protected and not self.root.is_relative_to(protected)
                    and not protected.is_relative_to(self.root),
                    f"run root overlaps adopted {key}")
        self.root.mkdir(parents=True)
        require(HEX.fullmatch(a.expected_mantle_blake3)
                and self.digest(a.mantle) == a.expected_mantle_blake3,
                "selected Mantle executable differs from independently pinned BLAKE3")
        require(HEX.fullmatch(a.expected_checker_blake3)
                and self.digest(a.checker) == a.expected_checker_blake3,
                "compiled checker differs from independently pinned BLAKE3")
        require(HEX.fullmatch(a.expected_source_tree_blake3),
                "expected complete frozen source tree identity must be lowercase BLAKE3")
        source_tree_before = self.source_tree_digest("before-native-qualification")
        self.contracts = {x["row_id"]: x for x in json.loads(self.checker("--contract"))}
        require(set(self.contracts) == {x[0] for x in ROOTS if x[0] not in ("gcc10-final-cycle", "musl-final")},
                "checker does not describe exactly the five core rows")
        self.adoption()
        require(not self.root.is_relative_to(self.store) and not self.root.is_relative_to(self.state)
                and not self.root.is_relative_to(Path(document(a.adoption_report)["proof_root"]).resolve()),
                "evidence tree overlaps producer/store/state")
        self.profile()
        view = self.root / "verification-view"
        view.mkdir()
        for row_id, recipe, package in ROOTS:
            log = self.build(row_id, recipe, package)
            if row_id in self.contracts:
                observations = matrix.observe(self, self.contracts[row_id], log, view)
                self.receipt(self.contracts[row_id], observations, view)
        # All five original validators, not a second interpretation of their rules.
        verified = json.loads(self.checker("--view-root", view))
        require(verified.get("validated") is True and len(verified.get("rows", [])) == 5,
                "core receipt validation did not accept all five rows")
        for contract in self.contracts.values():
            for rel in contract["sources"]:
                require(self.digest(self.source / rel) == self.digest(view / rel),
                        f"frozen source changed after qualification: {rel}")
        for row_id, artifact in self.artifacts.items():
            current, _ = self.read_store(artifact["logical"], "final-output-" + row_id)
            require(current["nar_sha256"] == artifact["info"]["nar_sha256"]
                    and current["nar_size"] == artifact["info"]["nar_size"]
                    and current["signatures"] == artifact["info"]["signatures"],
                    f"signed native row output changed during qualification: {row_id}")
        source_state = json.loads(self.cli("source", "bundle", "verify", "--from", a.source_profile,
                                          "--imported", label="profile-final-verify", timeout=a.build_seconds))
        require(source_state.get("ready_class") == "ready"
                and source_state.get("manifest_blake3") == a.expected_manifest_blake3,
                "imported source authority changed during qualification")
        self.adoption()  # same report bytes, current signed reserved outputs, after all operations
        source_tree_after = self.source_tree_digest("after-native-qualification")
        self.emit(self.root, "qualification.json", {"schema": "mantle-native-row-qualification-v1",
            "qualified": True, "rows": verified["rows"], "source_root": str(self.source),
            "source_tree_blake3": a.expected_source_tree_blake3,
            "source_tree_before_blake3": source_tree_before,
            "source_tree_after_blake3": source_tree_after,
            "source_manifest_blake3": a.expected_manifest_blake3,
            "adoption_report_bytes_blake3": a.expected_adoption_blake3,
            "verification_view": str(view),
            "non_claims": ["provider or seed admission", "whole-bootstrap correctness",
                           "general compiler, libc, or binutils correctness"]})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ("mantle", "checker", "tar", "b3sum", "source-root", "source-profile", "adoption-report",
                "signing-key", "run-root"):
        parser.add_argument("--" + arg, type=Path, required=True)
    parser.add_argument("--expected-manifest-blake3", required=True)
    parser.add_argument("--expected-adoption-blake3", required=True)
    parser.add_argument("--expected-mantle-blake3", required=True)
    parser.add_argument("--expected-checker-blake3", required=True)
    parser.add_argument("--expected-source-tree-blake3", required=True)
    parser.add_argument("--build-seconds", type=int, default=86400)
    parser.add_argument("--store-backend", choices=("snix", "casita"), default="snix")
    parser.add_argument("--jobs", type=int, default=2)
    a = parser.parse_args()
    require(1 <= a.jobs <= 16 and 1 <= a.build_seconds <= 86400,
            "jobs must be 1..=16 and --build-seconds must be 1..=86400")
    Operator(a).run()


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        print(json.dumps({"qualified": False, "error": str(error)}), file=sys.stderr)
        sys.exit(1)
