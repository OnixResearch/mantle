#!/usr/bin/env python3
"""Verify completed clean SDK builds after an interrupted proof rail, without rebuilding them."""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import sys


spec = importlib.util.spec_from_file_location("mantle_apk_proof", Path(__file__).with_name("prove-android-apk.py"))
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)

def admit_digest_tools(args, verifier_root):
    python = args.python.resolve()
    b3sum = args.b3sum.resolve()
    if Path(sys.executable).resolve() != python or not b3sum.is_file():
        raise RuntimeError("pinned Python interpreter or BLAKE3 executable is absent")
    proof.run(["nix", "store", "verify", "--offline", "--sigs-needed", "1",
               str(python.parent.parent), str(b3sum.parent.parent)], verifier_root)
    os.environ["PATH"] = str(b3sum.parent) + os.pathsep + os.environ.get("PATH", "")
    if shutil.which("b3sum") != str(b3sum):
        raise RuntimeError("signed pinned BLAKE3 tool cannot be selected")


def snapshot(root, manifest_path, nar_pin, profile_pin, manifest_pin):
    root = root.resolve()
    manifest_path = manifest_path.resolve()
    if manifest_path.is_relative_to(root):
        raise RuntimeError("snapshot manifest must be outside its immutable tree")
    for value in [nar_pin, profile_pin, manifest_pin]:
        if len(value) != 64 or any(c not in "0123456789abcdef" for c in value):
            raise RuntimeError("source snapshot identities must be independently pinned lowercase digests")
    manifest_blake3 = proof.blake3(manifest_path, root)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if (manifest_blake3 != manifest_pin or manifest.get("schema") != "mantle-apk-immutable-source-snapshot-v1"
            or manifest.get("snapshot_root") != str(root) or manifest.get("nix_interop_nar_sha256") != nar_pin
            or manifest.get("source_profile_blake3") != profile_pin or not isinstance(manifest.get("entries"), list)
            or manifest.get("file_count") != len(manifest["entries"]) or manifest["file_count"] < 1):
        raise RuntimeError("independently pinned complete source snapshot manifest drifted")
    profile = {"entries": manifest["entries"], "git_head": manifest.get("git_head")}
    if proof.blake3_bytes(json.dumps(profile, sort_keys=True, separators=(",", ":")).encode()) != profile_pin:
        raise RuntimeError("source profile differs from independently pinned member identities")
    proof.verify_snapshot_entries(root, manifest["entries"])
    if proof.nar_sha256(root, root) != nar_pin:
        raise RuntimeError("complete immutable source NAR SHA-256 changed")
    return {"root": str(root), "profile_blake3": profile_pin, "nix_interop_nar_sha256": nar_pin,
            "manifest": str(manifest_path), "manifest_blake3": manifest_blake3,
            "file_count": manifest["file_count"], "ignored_vendor_deps": manifest["excluded_ignored_vendor_deps"]}


def inputs(args, build_root, verifier_root):
    tools = {name: getattr(args, name).resolve() for name in ["python", "b3sum", "busybox", "bwrap"]}
    if Path(sys.executable).resolve() != tools["python"]:
        raise RuntimeError("verification interpreter differs from pinned signed Python")
    for name, tool in tools.items():
        if not tool.is_file() or not str(tool).startswith("/nix/store/"):
            raise RuntimeError(f"signed verifier tool is absent from the Nix store: {name}")
    proof.run(["nix", "store", "verify", "--offline", "--sigs-needed", "1",
               *[str(tool.parent.parent) for tool in tools.values()], str(proof.GLIBC), str(proof.LIBGCC)], verifier_root)
    os.environ["PATH"] = str(tools["b3sum"].parent) + os.pathsep + os.environ.get("PATH", "")
    if shutil.which("b3sum") != str(tools["b3sum"]):
        raise RuntimeError("signed BLAKE3 tool cannot be selected")
    runtime_hashes = proof.run(["nix", "hash", "path", "--type", "sha256", "--base16",
                                str(proof.GLIBC), str(proof.LIBGCC)], verifier_root).stdout.splitlines()
    if runtime_hashes != [proof.GLIBC_NAR_SHA256, proof.LIBGCC_NAR_SHA256]:
        raise RuntimeError("reviewed signed native runtime tree drifted")
    binary = args.mantle.resolve()
    if not binary.is_file() or proof.digest(binary) != args.expected_mantle_sha256:
        raise RuntimeError("independently pinned first Mantle executable is absent or changed")
    bundle = args.bundle.resolve()
    if not bundle.is_file() or proof.blake3(bundle, verifier_root) != args.expected_bundle_blake3:
        raise RuntimeError("original-HTTPS reviewed source bundle BLAKE3 changed")
    archives = {name: getattr(args, name).resolve() for name in proof.REVIEWED_ARCHIVES}
    for name, path in archives.items():
        if not path.is_file() or proof.digest(path) != proof.REVIEWED_ARCHIVES[name]:
            raise RuntimeError(f"original reviewed {name} HTTPS archive changed")
    source_state = args.build_run_dir.resolve() / "source-state"
    preflight = proof.run([str(binary), "--json", "--state-dir", str(source_state), "source", "bundle", "preflight",
                           "--build-root", str(build_root / proof.SOURCE_ROOT), "--import-path", str(build_root / "lib")], build_root)
    source_report = json.loads(preflight.stdout)
    if source_report.get("ready_class") != "ready" or source_report.get("record_count") != 3:
        raise RuntimeError("completed offline source replay lost its three ready build roots")
    return tools, binary, bundle, archives, source_report


def completed_builds(run_dir, source_report):
    replay_path = run_dir / "offline-source-replay.json"
    offline = json.loads(replay_path.read_text(encoding="utf-8"))
    counts = offline["counts"]
    if (counts["succeeded_total"], counts["built_total"], counts["cached_total"], counts["failed_total"]) != (3, 3, 0, 0):
        raise RuntimeError("completed offline archive replay was not three fresh successful builds")
    if offline["failed"] or offline["fod_mismatches"]:
        raise RuntimeError("completed offline archive replay had failures or fixed-output drift")
    outcomes = offline["outcomes"]
    if {entry["label"] for entry in outcomes} != {"build_tools", "jdk", "platform"}:
        raise RuntimeError("offline source outcomes do not bind the three APK inputs")
    for entry in outcomes:
        if entry["cached"] or len(entry["outputs"]) != 1:
            raise RuntimeError("offline source outcome lacks a fresh single archive")
        if proof.digest(Path(entry["outputs"][0]["path"])) != proof.REVIEWED_ARCHIVES[entry["label"]]:
            raise RuntimeError("offline fetched archive bytes differ from original reviewed HTTPS input")
    log = run_dir / "real-build.log"
    text = log.read_text(encoding="utf-8")
    lines = [line for line in text.splitlines() if "test result:" in line]
    if len(lines) != 1 or "test result: ok. 1 passed; 0 failed" not in lines[0]:
        raise RuntimeError("ignored real SDK test did not finish with exactly one passing result")
    observations = json.loads((run_dir / "builder-observations.json").read_text(encoding="utf-8"))
    names = ["clean-one", "clean-two", "java-perturbed"]
    cases = observations["runs"]
    if ([case["run"] for case in cases] != names
            or observations["source_state_blake3"] != source_report["source_state_blake3"]):
        raise RuntimeError("recorded independent build sequence or offline source state drifted")
    for case in cases:
        name = case["run"]
        if text.count(f"APK_PROOF_REAL_BUILD_START {name}") != 1 or text.count(f"APK_PROOF_REAL_BUILD_DONE {name}") != 1:
            raise RuntimeError(f"recorded real build did not start and complete exactly once: {name}")
        observation = case["observation"]
        store = (run_dir / name / "store").resolve()
        steps = observation["steps"]
        roots = observation["archive_fods"] + observation["extractions"] + steps
        if (observation["altered_java"] != (name == "java-perturbed") or len(observation["archive_fods"]) != 3
                or len(observation["extractions"]) != 3 or len(steps) != 6
                or [step["kind"] for step in steps] != ["Aapt2Compile", "Aapt2Link", "Javac", "D8", "Zipalign", "Apksigner"]
                or not (run_dir / name / "state").is_dir() or not store.is_dir()
                or any(not Path(entry["store_path"]).resolve().is_relative_to(store) for entry in roots)
                or Path(observation["apk"]).resolve() != Path(steps[-1]["output"]).resolve()
                or not Path(observation["apk"]).resolve().is_relative_to(store)):
            raise RuntimeError(f"incomplete or non-independent signed SDK build: {name}")
    return offline, cases, lines, log


def verify_apks(cases, run_dir, scratch, build_root, tools):
    results = []
    for case in cases:
        apk = Path(case["observation"]["apk"])
        structure = proof.observe_apk(apk)
        verification = proof.verify_with_reviewed_apksigner(apk, case, build_root, tools["bwrap"])
        if verification["exit_code"] != 0 or "Verified using v2 scheme (APK Signature Scheme v2): true" not in verification["stdout"]:
            raise RuntimeError(f"reviewed apksigner did not verify v2 for {case['run']}: {verification}")
        results.append({"run": case["run"], "apk": str(apk), "apk_blake3": proof.blake3(apk, build_root),
                        "structure": structure, "apksigner_verify": verification,
                        "archive_fods": case["observation"]["archive_fods"],
                        "extractions": case["observation"]["extractions"], "steps": case["observation"]["steps"]})
    if results[0]["structure"]["entries"] != results[1]["structure"]["entries"]:
        raise RuntimeError("clean APK ZIP entry ordering drifted")
    if results[0]["apk_blake3"] != results[1]["apk_blake3"]:
        raise RuntimeError("two independent clean signed APK BLAKE3 digests differ")
    if results[0]["apk_blake3"] == results[2]["apk_blake3"]:
        raise RuntimeError("Java perturbation did not change signed APK BLAKE3 digest")
    tampered = scratch / "tampered-signed.apk"
    shutil.copyfile(results[0]["apk"], tampered)
    with tampered.open("r+b") as output:
        offset = results[0]["structure"]["dex_offset"]
        output.seek(offset)
        original = output.read(1)
        if len(original) != 1:
            raise RuntimeError("signed DEX tamper offset is outside the APK")
        output.seek(offset)
        output.write(bytes([original[0] ^ 1]))
    negative = proof.verify_with_reviewed_apksigner(tampered, cases[0], build_root, tools["bwrap"])
    if negative["exit_code"] == 0:
        raise RuntimeError("tampered signed DEX byte passed reviewed apksigner verification")
    return results, {"flipped_signed_dex_payload_byte_offset": offset, "apksigner_verify": negative}


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["build-source-root", "build-source-snapshot-manifest", "verification-source-snapshot-manifest",
                 "mantle", "bundle", "build-tools", "jdk", "platform", "commandline-tools", "busybox", "bwrap",
                 "python", "b3sum", "build-run-dir", "scratch-dir", "evidence-dir"]:
        parser.add_argument("--" + name, type=Path, required=True)
    for name in ["build-source-nar-sha256", "build-source-profile-blake3", "build-source-manifest-blake3",
                 "verification-source-nar-sha256", "verification-source-profile-blake3", "verification-source-manifest-blake3",
                 "expected-mantle-sha256", "expected-bundle-blake3"]:
        parser.add_argument("--" + name, required=True)
    return parser.parse_args()


def main():
    args = arguments()
    verifier_root = Path(__file__).resolve().parent.parent
    build_root = args.build_source_root.resolve()
    run_dir = args.build_run_dir.resolve()
    scratch = args.scratch_dir.resolve()
    evidence = args.evidence_dir.resolve()
    receipt = evidence / "signed-java-hello-world.json"
    blocker = evidence / "official-build-blocker.json"
    stage = "prerequisites"
    try:
        if receipt.exists() or not run_dir.is_dir() or scratch.exists():
            raise RuntimeError("completed run must exist, verifier scratch must be fresh, and success must not already exist")
        if (build_root == verifier_root or any(path.is_relative_to(root) for path in [run_dir, scratch, evidence]
                                               for root in [build_root, verifier_root])):
            raise RuntimeError("build and verification snapshots and all writable evidence locations must be separate")
        if shutil.disk_usage(scratch.parent).free < 1 << 30:
            raise RuntimeError("post-build verifier scratch filesystem has less than 1 GiB free")
        stage = "signed-identity-tool-admission"
        admit_digest_tools(args, verifier_root)
        stage = "immutable-build-and-verification-source-admission"
        build_snapshot = snapshot(build_root, args.build_source_snapshot_manifest, args.build_source_nar_sha256,
                                  args.build_source_profile_blake3, args.build_source_manifest_blake3)
        verifier_snapshot = snapshot(verifier_root, args.verification_source_snapshot_manifest,
                                     args.verification_source_nar_sha256, args.verification_source_profile_blake3,
                                     args.verification_source_manifest_blake3)
        stage = "original-reviewed-input-and-signed-verifier-tool-admission"
        tools, binary, bundle, archives, source_report = inputs(args, build_root, verifier_root)
        stage = "completed-three-builds-and-offline-source-replay"
        offline, cases, lines, build_log = completed_builds(run_dir, source_report)
        scratch.mkdir()
        stage = "structure-and-reviewed-apksigner-verification"
        results, tamper_negative = verify_apks(cases, run_dir, scratch, build_root, tools)
        stage = "post-verification-source-and-binary-recheck"
        snapshot(build_root, args.build_source_snapshot_manifest, args.build_source_nar_sha256,
                 args.build_source_profile_blake3, args.build_source_manifest_blake3)
        snapshot(verifier_root, args.verification_source_snapshot_manifest, args.verification_source_nar_sha256,
                 args.verification_source_profile_blake3, args.verification_source_manifest_blake3)
        if proof.digest(binary) != args.expected_mantle_sha256 or proof.blake3(bundle, verifier_root) != args.expected_bundle_blake3:
            raise RuntimeError("first Mantle executable or original-HTTPS source bundle changed during verification")
        for name, path in archives.items():
            if proof.digest(path) != proof.REVIEWED_ARCHIVES[name]:
                raise RuntimeError(f"original reviewed {name} archive changed during verification")
        stage = "success-receipt"
        evidence.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(build_log, evidence / "real-build-full.log")
        policy = json.loads((run_dir / "example.json").read_text(encoding="utf-8"))["plan"]["reproducibility"]
        success = {
            "schema": "mantle-android-reviewed-signed-java-apk-proof-v1", "status": "verified",
            "verification_mode": "completed-independent-three-build-run",
            "example": str(proof.EXAMPLE), "source_bundle": str(bundle),
            "source_bundle_blake3": args.expected_bundle_blake3,
            "source_snapshot": build_snapshot, "verification_source_snapshot": verifier_snapshot,
            "mantle_binary": {"path": str(binary), "first_executable_sha256_pin": args.expected_mantle_sha256,
                              "blake3": proof.blake3(binary, verifier_root)},
            "source_state_blake3": source_report["source_state_blake3"], "offline_source_preflight": source_report,
            "offline_replay": {"mode": "SourceFetchOverridePlan --offline-source-preflight --no-substitute",
                               "counts": offline["counts"], "outcomes": offline["outcomes"],
                               "advisory_ast_grep_structural_evidence_diagnostics": offline["ast_grep_structural_evidence_diagnostics"],
                               "full_report": str(run_dir / "offline-source-replay.json")},
            "original_reviewed_archives_sha256": proof.REVIEWED_ARCHIVES,
            "signed_runtime_cohort": {
                "glibc": {"source_store_path": str(proof.GLIBC), "nar_sha256": proof.GLIBC_NAR_SHA256, "nar_size": 35073128},
                "libgcc": {"source_store_path": str(proof.LIBGCC), "nar_sha256": proof.LIBGCC_NAR_SHA256, "nar_size": 197672}},
            "environment": {"target_system": "x86_64-linux", "host_os": sys.platform,
                            "host_arch": platform.machine(), "source_fetch_mode": "offline-pinned-source-bundle-and-sha256-checked-file-overrides",
                            "SOURCE_DATE_EPOCH": policy["entry_timestamp_epoch"], "LC_ALL": "C", "TZ": "UTC",
                            **{name + "_store_path": str(tool.parent.parent) for name, tool in tools.items()},
                            **{name + "_blake3": proof.blake3(tool, verifier_root) for name, tool in tools.items()},
                            "fresh_store_and_state_count": 3},
            "builds": results, "tamper_negative": tamper_negative, "test_result_lines": lines,
            "full_build_log": str(evidence / "real-build-full.log"),
            "non_claims": ["No install on a device or emulator", "No launch or runtime-behavior proof",
                           "Test-only keystore, not a production release key"]}
        pending = receipt.with_suffix(".json.partial")
        if pending.exists():
            raise RuntimeError("existing partial success receipt cannot be overwritten")
        pending.write_text(json.dumps(success, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        receipt_blake3 = proof.blake3(pending, verifier_root)
        blocker.unlink(missing_ok=True)
        pending.replace(receipt)
        print(f"APK_PROOF_SUCCESS receipt={receipt} blake3={receipt_blake3}")
        for line in lines:
            print(line)
    except Exception as error:
        evidence.mkdir(parents=True, exist_ok=True)
        blocker.write_text(json.dumps({"schema": "mantle-android-reviewed-signed-java-apk-proof-v1",
                                       "status": "blocked", "stage": stage, "error": str(error),
                                       "success_receipt_written": False}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"APK_PROOF_BLOCKED stage={stage} blocker={blocker}: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
