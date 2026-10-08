#!/usr/bin/env python3
"""Detached, fail-closed proof of the reviewed signed Java APK example.

Run with pueue and keep its full task log. No device or emulator is involved.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import stat
import shutil
import subprocess
import struct
import sys
import zipfile

REVIEWED_ARCHIVES = {
    "build_tools": "bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88",
    "jdk": "992f96e7995075ac7636bb1a8de52b0c61d71ed3137fafc979ab96b4ab78dd75",
    "platform": "0988cacad01b38a18a47bac14a0695f246bc76c1b06c0eeb8eb0dc825ab0c8e0",
    "commandline_tools": "7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee",
}
GLIBC = Path("/nix/store/n51dhmdbik1kfrsm62j5knavmigwrl1a-glibc-2.42-84")
LIBGCC = Path("/nix/store/ssvq1r0xd8f7paf6zqgpfql1a4drwhy2-xgcc-15.3.0-libgcc")
BUSYBOX = Path("/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox")
GLIBC_NAR_SHA256 = "428a192cf9765f6fddaa77b31a028e3cda7351bc61c5b2f02f34d2a0e843f47a"
LIBGCC_NAR_SHA256 = "70cbaf1ca29943bbace24a54038d2d3716699104118afb7a7d6b591c03d570af"
SOURCE_ROOT = Path("examples/android-reviewed-toolchain-fetch.ncl")
EXAMPLE = Path("examples/android-minimal.ncl")


def run(command, cwd, *, environment=None, check=True):
    completed = subprocess.run(command, cwd=cwd, env=environment, text=True, capture_output=True, check=False)
    if check and completed.returncode != 0:
        raise RuntimeError(
            f"command exited {completed.returncode}: {command!r}\n"
            f"stdout: {completed.stdout[-8000:]}\nstderr: {completed.stderr[-8000:]}"
        )
    return completed


def nix_command(*command):
    return ["nix", "develop", "--offline", "--no-write-lock-file", "--command", *command]


def digest(path):
    observer = hashlib.sha256()
    with path.open("rb") as payload:
        while block := payload.read(1 << 20):
            observer.update(block)
    return observer.hexdigest()


def blake3(path, cwd):
    observed = run(["b3sum", str(path)], cwd).stdout.split()[0]
    if len(observed) != 64:
        raise RuntimeError(f"invalid b3sum result for {path}")
    return observed

def blake3_bytes(data):
    observed = subprocess.run(["b3sum", "--no-names"], input=data, capture_output=True, check=True).stdout.decode().strip()
    if len(observed) != 64:
        raise RuntimeError("invalid BLAKE3 result for canonical source profile")
    return observed

def nar_sha256(path, cwd):
    return run(["nix", "hash", "path", "--type", "sha256", "--base16", str(path)], cwd).stdout.strip()

def verify_snapshot_entries(repo, entries):
    required = {
        "Cargo.toml", "Cargo.lock", ".cargo/config.toml",
        "crates/crunch-build/src/lib.rs", "crates/crunch-store/src/lib.rs",
        "crates/crunch-android-core/src/lib.rs", "crates/crunch-android/src/lib.rs",
        "scripts/snapshot-android-apk-source.py", "scripts/prove-android-apk.py",
    }
    if not required <= {entry["path"] for entry in entries}:
        raise RuntimeError("controlled snapshot omitted Cargo lock, an owner crate, or the proof rail")
    for entry in entries:
        member = PurePosixPath(entry["path"])
        if member.is_absolute() or not member.parts or ".." in member.parts or ".git" in member.parts:
            raise RuntimeError(f"unsafe snapshot member: {entry['path']!r}")
        path = repo / entry["path"]
        metadata = path.lstat()
        if entry["kind"] == "symlink":
            target = os.readlink(path)
            if target != entry["target"] or not path.resolve().is_relative_to(repo):
                raise RuntimeError(f"snapshot symlink drift: {member}")
            observed = blake3_bytes(b"symlink\0" + os.fsencode(target))
        elif entry["kind"] == "file" and stat.S_ISREG(metadata.st_mode):
            mode = stat.S_IMODE(metadata.st_mode)
            if mode != (entry["mode"] & ~0o222) or metadata.st_size != entry["size"]:
                raise RuntimeError(f"read-only snapshot mode or size drift: {member}")
            observed = blake3(path, repo)
        else:
            raise RuntimeError(f"unsupported snapshot member: {member}")
        if observed != entry["blake3"]:
            raise RuntimeError(f"source BLAKE3 drift: {member}")


def observe_apk(path):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        if len(names) != len(set(names)):
            raise RuntimeError("APK contains duplicate ZIP entry names")
        if not {"AndroidManifest.xml", "classes.dex", "resources.arsc"} <= set(names):
            raise RuntimeError("compiled manifest, DEX, or linked resources are missing")
        for entry in archive.infolist():
            if entry.is_dir():
                continue
            with archive.open(entry) as contents:
                while contents.read(1 << 20):
                    pass
        if not archive.read("AndroidManifest.xml").startswith(b"\x03\x00\x08\x00"):
            raise RuntimeError("manifest is not aapt2 compiled binary XML")
        if not archive.read("classes.dex").startswith(b"dex\n"):
            raise RuntimeError("classes.dex is not DEX bytecode")
        dex = archive.getinfo("classes.dex")
        with path.open("rb") as payload:
            payload.seek(dex.header_offset)
            local_header = payload.read(30)
            if len(local_header) != 30 or local_header[:4] != b"PK\x03\x04":
                raise RuntimeError("DEX local ZIP header is invalid")
            filename_size, extra_size = struct.unpack_from("<HH", local_header, 26)
            if payload.read(filename_size) != b"classes.dex":
                raise RuntimeError("DEX local ZIP header name differs from central directory")
            dex_offset = dex.header_offset + 30 + filename_size + extra_size
        if dex.compress_size < 4:
            raise RuntimeError("DEX payload cannot support a tamper control")
    with path.open("rb") as payload:
        if b"APK Sig Block 42" not in payload.read():
            raise RuntimeError("APK Signing Block is absent")
    return {"entries": names, "compiled_manifest": True, "dex": True, "apk_signing_block": True, "dex_offset": dex_offset}


def verify_with_reviewed_apksigner(apk, case, cwd):
    observation = case["observation"]
    extracts = {entry["name"]: Path(entry["store_path"]) for entry in observation["extractions"]}
    build_tools = extracts["android-extract-android-build-tools"] / "android-15"
    jdk = extracts["android-extract-temurin-jdk"] / "jdk-17.0.17+10"
    store = Path(observation["steps"][-1]["store_path"]).parent
    glibc = store / ("c" * 32 + "-runtime-glibc")
    libgcc = store / ("d" * 32 + "-runtime-libgcc")
    # The verifier sees only the already-admitted runtime and extracted SDK
    # contents plus this one APK; host /lib64 and /nix/store remain unmounted.
    command = nix_command(
        "bwrap", "--unshare-user", "--uid", "1000", "--gid", "100", "--tmpfs", "/",
        "--dev", "/dev", "--proc", "/proc", "--tmpfs", "/tmp", "--dir", "/store",
        "--dir", "/build", "--ro-bind", str(glibc), "/store/runtime",
        "--ro-bind", str(libgcc), "/store/libgcc",
        "--ro-bind", str(build_tools), "/store/buildtools",
        "--ro-bind", str(jdk), "/store/jdk",
        "--ro-bind", str(apk), "/store/reviewed.apk",
        "--chdir", "/build", "--",
        "/store/runtime/lib/ld-linux-x86-64.so.2", "--library-path",
        "/store/runtime/lib:/store/libgcc/lib:/store/buildtools/lib64:/store/jdk/lib:/store/jdk/lib/server",
        "/store/jdk/bin/java", "-jar", "/store/buildtools/lib/apksigner.jar",
        "verify", "--verbose", "/store/reviewed.apk",
    )
    completed = run(command, cwd, check=False)
    return {"exit_code": completed.returncode, "stdout": completed.stdout, "stderr": completed.stderr}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", type=Path, required=True, help="already built current-tree Mantle binary")
    parser.add_argument("--run-dir", type=Path, required=True, help="new scratch directory on a spacious filesystem")
    parser.add_argument("--bundle", type=Path, required=True, help="prefetched source bundle, or path to fetch it into")
    parser.add_argument("--build-tools", type=Path, required=True)
    parser.add_argument("--jdk", type=Path, required=True)
    parser.add_argument("--platform", type=Path, required=True)
    parser.add_argument("--commandline-tools", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True, help="private Cargo target, never the shared target")
    parser.add_argument("--evidence-dir", type=Path, required=True, help="receipt/log directory outside immutable source snapshot")
    parser.add_argument("--source-snapshot-manifest", type=Path, required=True, help="controlled-copy manifest outside source snapshot")
    parser.add_argument("--expected-source-nar-sha256", required=True, help="full immutable source snapshot NAR SHA-256")
    parser.add_argument("--expected-source-profile-blake3", required=True, help="Mantle-owned canonical source profile BLAKE3")
    parser.add_argument("--expected-source-manifest-blake3", required=True, help="controlled-copy manifest BLAKE3")
    parser.add_argument("--expected-mantle-sha256", required=True, help="independently pinned first Mantle binary SHA-256")
    parser.add_argument("--expected-bundle-blake3", help="BLAKE3 of the previously exported original-HTTPS source bundle")
    arguments = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    evidence = arguments.evidence_dir.resolve()
    for label, location in [
        ("--evidence-dir", evidence),
        ("--target-dir", arguments.target_dir.resolve()),
        ("--run-dir", arguments.run_dir.resolve()),
        ("--bundle", arguments.bundle.resolve()),
        ("--source-snapshot-manifest", arguments.source_snapshot_manifest.resolve()),
    ]:
        if location.is_relative_to(repo):
            parser.error(f"{label} must be outside the immutable source snapshot")
    receipt = evidence / "signed-java-hello-world.json"
    blocker = evidence / "official-build-blocker.json"
    stage = "prerequisites"
    run_dir = arguments.run_dir.resolve()
    try:
        if receipt.exists():
            raise RuntimeError(f"success receipt already exists; refuse to overwrite evidence: {receipt}")
        if run_dir.exists():
            raise RuntimeError(f"proof run directory must be fresh: {run_dir}")
        if shutil.disk_usage(run_dir.parent).free < 16 * (1 << 30):
            raise RuntimeError("proof scratch filesystem has less than 16 GiB free")
        for executable in [arguments.mantle, BUSYBOX, GLIBC / "lib/ld-linux-x86-64.so.2", LIBGCC / "lib/libgcc_s.so.1"]:
            if not executable.is_file():
                raise RuntimeError(f"reviewed executable/runtime prerequisite missing: {executable}")
        if shutil.which("nix") is None or shutil.which("b3sum") is None:
            raise RuntimeError("Nix and b3sum are required to verify the reviewed cohort")
        expected_source = arguments.expected_source_nar_sha256.lower()
        expected_mantle = arguments.expected_mantle_sha256.lower()
        expected_profile = arguments.expected_source_profile_blake3.lower()
        expected_manifest = arguments.expected_source_manifest_blake3.lower()
        if any(
            len(value) != 64 or any(char not in "0123456789abcdef" for char in value)
            for value in [expected_source, expected_mantle, expected_profile, expected_manifest]
        ):
            raise RuntimeError("source NAR/first executable SHA-256 and source profile/manifest BLAKE3 identities must be 64 hex digits")
        snapshot_manifest = arguments.source_snapshot_manifest.resolve()
        if not snapshot_manifest.is_file():
            raise RuntimeError(f"controlled source snapshot manifest missing: {snapshot_manifest}")
        initial_manifest_blake3 = blake3(snapshot_manifest, repo)
        manifest = json.loads(snapshot_manifest.read_text(encoding="utf-8"))
        if (
            initial_manifest_blake3 != expected_manifest
            or manifest.get("schema") != "mantle-apk-immutable-source-snapshot-v1"
            or manifest.get("nix_interop_nar_sha256") != expected_source
            or manifest.get("source_profile_blake3") != expected_profile
            or manifest.get("snapshot_root") != str(repo)
            or not isinstance(manifest.get("entries"), list)
            or not isinstance(manifest.get("file_count"), int)
            or manifest["file_count"] != len(manifest["entries"])
            or manifest["file_count"] < 1
        ):
            raise RuntimeError("controlled source snapshot manifest does not match this immutable tree")
        if arguments.target_dir.resolve().is_relative_to(Path(manifest["source_root"]).resolve()):
            raise RuntimeError("APK Cargo target must remain outside the shared original checkout")
        profile = {"entries": manifest["entries"], "git_head": manifest.get("git_head")}
        if blake3_bytes(json.dumps(profile, sort_keys=True, separators=(",", ":")).encode("utf-8")) != expected_profile:
            raise RuntimeError("controlled source profile BLAKE3 differs from canonical manifest entries")
        verify_snapshot_entries(repo, manifest["entries"])
        if nar_sha256(repo, repo) != expected_source:
            raise RuntimeError("immutable source snapshot NAR SHA-256 drift before SDK proof")
        if digest(arguments.mantle) != expected_mantle:
            raise RuntimeError("first pinned Mantle executable SHA-256 drift before SDK proof")
        initial_mantle_blake3 = blake3(arguments.mantle, repo)
        archives = {
            "build_tools": arguments.build_tools.resolve(),
            "jdk": arguments.jdk.resolve(),
            "platform": arguments.platform.resolve(),
            "commandline_tools": arguments.commandline_tools.resolve(),
        }
        for label, archive in archives.items():
            if not archive.is_file() or digest(archive) != REVIEWED_ARCHIVES[label]:
                raise RuntimeError(f"{label} archive absent or differs from original reviewed SHA-256")
        run(["nix", "store", "verify", "--offline", "--sigs-needed", "1", str(GLIBC), str(LIBGCC), str(BUSYBOX.parent.parent)], repo)
        nar_hashes = run(["nix", "hash", "path", "--type", "sha256", "--base16", str(GLIBC), str(LIBGCC)], repo).stdout.splitlines()
        if nar_hashes != [GLIBC_NAR_SHA256, LIBGCC_NAR_SHA256]:
            raise RuntimeError("signed runtime cohort NAR content differs from independently pinned digests")
        run_dir.mkdir()
        # The pinned devshell creates a temporary Cargo home under TMPDIR.
        # /tmp may be full; keep every devshell invocation on proof scratch.
        os.environ["TMPDIR"] = str(run_dir)
        run(nix_command("bwrap", "--unshare-user", "--tmpfs", "/", "--dev", "/dev", "--proc", "/proc", "--dir", "/bin", "--ro-bind", str(BUSYBOX), "/bin/sh", "--", "/bin/sh", "-c", "exit 0"), repo)
        bundle = arguments.bundle.resolve()
        if bundle.exists() and not arguments.expected_bundle_blake3:
            raise RuntimeError("prefetched reviewed source bundle requires an independently pinned BLAKE3")
        source_state = run_dir / "source-state"
        source_store = run_dir / "source-store"
        source_store.mkdir()
        mantle = str(arguments.mantle.resolve())
        if not bundle.exists():
            stage = "connected-source-export"
            bundle.parent.mkdir(parents=True, exist_ok=True)
            run([mantle, "--state-dir", str(source_state), "source", "bundle", "export", "--build-root", str(SOURCE_ROOT), "--import-path", "lib", "--fetch-missing", "--to", str(bundle)], repo)
        initial_bundle_blake3 = blake3(bundle, repo)
        if arguments.expected_bundle_blake3 and initial_bundle_blake3 != arguments.expected_bundle_blake3.lower():
            raise RuntimeError("prefetched source bundle BLAKE3 differs from reviewed snapshot")
        stage = "offline-source-import"
        run([mantle, "--state-dir", str(source_state), "source", "bundle", "import", "--from", str(bundle), "--pin"], repo)
        stage = "offline-source-preflight"
        preflight = run([mantle, "--json", "--state-dir", str(source_state), "source", "bundle", "preflight", "--build-root", str(SOURCE_ROOT), "--import-path", "lib"], repo)
        source_report = json.loads(preflight.stdout)
        if source_report.get("ready_class") != "ready" or source_report.get("record_count") != 3:
            raise RuntimeError(f"offline source prerequisite refused: {source_report}")
        source_state_blake3 = source_report["source_state_blake3"]
        stage = "SourceFetchOverridePlan-offline-build"
        offline = run([mantle, "--json", "--state-dir", str(source_state), "--store", str(source_store), "build", str(SOURCE_ROOT), "--import-path", "lib", "--offline-source-preflight", "--no-substitute"], repo)
        (run_dir / "offline-source-replay.json").write_text(offline.stdout, encoding="utf-8")
        offline_report = json.loads(offline.stdout)
        counts = offline_report["counts"]
        if (counts["succeeded_total"], counts["built_total"], counts["cached_total"], counts["failed_total"]) != (3, 3, 0, 0):
            raise RuntimeError(f"offline fixed-output source build did not freshly succeed three times: {counts}")
        if offline_report["failed"] or offline_report["fod_mismatches"]:
            raise RuntimeError("offline reviewed sources failed or had fixed-output digest mismatches")
        source_outcomes = offline_report["outcomes"]
        if {item["label"] for item in source_outcomes} != {"build_tools", "jdk", "platform"}:
            raise RuntimeError("offline source override outcomes do not match the three APK inputs")
        if any(
            item["cached"] or len(item["outputs"]) != 1
            or not Path(item["outputs"][0]["path"]).is_file()
            for item in source_outcomes
        ):
            raise RuntimeError("offline reviewed archive outputs must be fresh materialized files")
        stage = "example-evaluation"
        exported = run(nix_command("nickel", "export", "--format", "json", str(EXAMPLE)), repo)
        example_json = run_dir / "example.json"
        example_json.write_text(exported.stdout, encoding="utf-8")
        if not json.loads(exported.stdout)["plan"]["signing"]:
            raise RuntimeError("example does not request signing")
        stage = "test-only-keystore"
        host_jdk = run_dir / "host-jdk"
        host_jdk.mkdir()
        run(["tar", "-xzf", str(archives["jdk"]), "-C", str(host_jdk)], repo)
        keytool = Path("/store/jdk/bin/keytool")
        if not (host_jdk / "jdk-17.0.17+10/bin/keytool").is_file():
            raise RuntimeError("reviewed Temurin keytool absent")
        signing = run_dir / "signing"
        signing.mkdir()
        keystore = signing / "release.jks"
        password = signing / "key-password.txt"
        password_value = "mantle-test-only-not-a-release-key"
        password.write_text(password_value + "\n", encoding="ascii")
        password.chmod(0o600)
        run(nix_command(
            "bwrap", "--unshare-user", "--tmpfs", "/", "--dev", "/dev", "--proc", "/proc",
            "--dir", "/store", "--ro-bind", str(GLIBC), "/store/runtime",
            "--ro-bind", str(LIBGCC), "/store/libgcc",
            "--ro-bind", str(host_jdk / "jdk-17.0.17+10"), "/store/jdk",
            "--bind", str(signing), "/store/signing",
            "--setenv", "JAVA_HOME", "/store/jdk", "--setenv", "LC_ALL", "C",
            "--setenv", "TZ", "UTC",
            "--", "/store/runtime/lib/ld-linux-x86-64.so.2",
            "--library-path", "/store/runtime/lib:/store/libgcc/lib:/store/jdk/lib:/store/jdk/lib/server",
            str(keytool), "-genkeypair", "-alias", "mantle", "-keyalg", "RSA", "-keysize", "2048",
            "-validity", "3650", "-dname", "CN=Mantle APK proof,O=Test only", "-storetype", "PKCS12",
            "-keystore", "/store/signing/release.jks", "-storepass", password_value,
            "-keypass", password_value, "-noprompt",
        ), repo)
        keystore.chmod(0o600)
        stage = "three-real-sandbox-builds"
        environment = os.environ.copy()
        environment.update({
            "TMPDIR": str(run_dir),
            "CARGO_TARGET_DIR": str(arguments.target_dir.resolve()),
            "CRUNCH_NO_FUSE": "1",
            "SNIX_BUILD_SANDBOX_SHELL": str(BUSYBOX),
            "APK_PROOF_RUN_DIR": str(run_dir),
            "APK_PROOF_EXAMPLE_JSON": str(example_json),
            "APK_PROOF_SOURCE_STATE_BLAKE3": source_state_blake3,
            "APK_PROOF_BUILD_TOOLS_ARCHIVE": str(archives["build_tools"]),
            "APK_PROOF_JDK_ARCHIVE": str(archives["jdk"]),
            "APK_PROOF_PLATFORM_ARCHIVE": str(archives["platform"]),
            "APK_PROOF_KEYSTORE": str(keystore),
            "APK_PROOF_PASSWORD_FILE": str(password),
        })
        build_log = run_dir / "real-build.log"
        with build_log.open("w", encoding="utf-8") as output:
            result = subprocess.run(
                nix_command("cargo", "test", "-p", "crunch-android", "--lib", "--locked", "--offline", "reviewed_signed_java_apk_clean_rebuilds_and_perturbation", "--", "--ignored", "--nocapture"),
                cwd=repo, env=environment, stdout=output, stderr=subprocess.STDOUT, check=False,
            )
        if result.returncode != 0:
            raise RuntimeError(f"real SDK build test failed ({result.returncode}); full log: {build_log}\n{build_log.read_text(encoding='utf-8')[-12000:]}")
        observations = json.loads((run_dir / "builder-observations.json").read_text(encoding="utf-8"))
        cases = observations["runs"]
        if [case["run"] for case in cases] != ["clean-one", "clean-two", "java-perturbed"]:
            raise RuntimeError("fresh build sequence incomplete")
        stage = "structure-and-apksigner-verification"
        results = []
        for case in cases:
            apk = Path(case["observation"]["apk"])
            structure = observe_apk(apk)
            verification = verify_with_reviewed_apksigner(apk, case, repo)
            if verification["exit_code"] != 0:
                raise RuntimeError(f"reviewed apksigner rejected {case['run']}: {verification}")
            if "Verified using v2 scheme (APK Signature Scheme v2): true" not in verification["stdout"]:
                raise RuntimeError(f"reviewed apksigner did not confirm v2 signing for {case['run']}: {verification}")
            results.append({"run": case["run"], "apk": str(apk), "apk_blake3": blake3(apk, repo), "structure": structure, "apksigner_verify": verification, "archive_fods": case["observation"]["archive_fods"], "extractions": case["observation"]["extractions"], "steps": case["observation"]["steps"]})
        if results[0]["structure"]["entries"] != results[1]["structure"]["entries"]:
            raise RuntimeError("clean APK ZIP entry ordering drift")
        if results[0]["apk_blake3"] != results[1]["apk_blake3"]:
            for first, second in zip(results[0]["steps"], results[1]["steps"], strict=True):
                if first["output_nar_sha256"] != second["output_nar_sha256"]:
                    raise RuntimeError(
                        f"clean APK BLAKE3 mismatch; first differing producing step {first['kind']}: "
                        f"{first['output_nar_sha256']} != {second['output_nar_sha256']}"
                    )
            raise RuntimeError("clean APK BLAKE3 mismatch at producing step Apksigner, despite matching signed stage NAR hashes")
        if results[0]["apk_blake3"] == results[2]["apk_blake3"]:
            raise RuntimeError("perturbed Java source did not change APK BLAKE3 digest")
        stage = "signed-region-tamper-negative-control"
        signed = Path(results[0]["apk"])
        tampered = run_dir / "tampered-signed.apk"
        shutil.copyfile(signed, tampered)
        with tampered.open("r+b") as output:
            output.seek(results[0]["structure"]["dex_offset"])
            old = output.read(1)
            output.seek(results[0]["structure"]["dex_offset"])
            output.write(bytes([old[0] ^ 1]))
        tamper_result = verify_with_reviewed_apksigner(tampered, cases[0], repo)
        if tamper_result["exit_code"] == 0:
            raise RuntimeError("tampered signed region unexpectedly passed reviewed apksigner verify")
        lines = [line for line in build_log.read_text(encoding="utf-8").splitlines() if "test result:" in line]
        if not any("1 passed; 0 failed" in line for line in lines):
            raise RuntimeError("real builder ignored test lacks executed passing test result line")
        stage = "immutable-source-and-binary-recheck"
        if nar_sha256(repo, repo) != expected_source:
            raise RuntimeError("source snapshot changed during reviewed SDK execution")
        verify_snapshot_entries(repo, manifest["entries"])
        if digest(arguments.mantle) != expected_mantle:
            raise RuntimeError("first pinned Mantle executable changed during SDK execution")
        if blake3(arguments.mantle, repo) != initial_mantle_blake3:
            raise RuntimeError("first executable BLAKE3 changed during SDK execution")
        if blake3(bundle, repo) != initial_bundle_blake3:
            raise RuntimeError("reviewed source bundle changed during SDK execution")
        if blake3(snapshot_manifest, repo) != initial_manifest_blake3:
            raise RuntimeError("controlled source snapshot manifest changed during SDK execution")
        for label, archive in archives.items():
            if digest(archive) != REVIEWED_ARCHIVES[label]:
                raise RuntimeError(f"{label} original reviewed archive changed during SDK execution")
        stage = "success-receipt"
        evidence.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(build_log, evidence / "real-build-full.log")
        success = {
            "schema": "mantle-android-reviewed-signed-java-apk-proof-v1",
            "status": "verified",
            "example": str(EXAMPLE),
            "source_bundle": str(bundle),
            "source_bundle_blake3": initial_bundle_blake3,
            "source_snapshot": {
                "root": str(repo),
                "profile_blake3": expected_profile,
                "nix_interop_nar_sha256": expected_source,
                "manifest": str(snapshot_manifest),
                "manifest_blake3": initial_manifest_blake3,
                "file_count": manifest["file_count"],
                "ignored_vendor_deps": manifest["excluded_ignored_vendor_deps"],
            },
            "mantle_binary": {"path": str(arguments.mantle.resolve()), "first_executable_sha256_pin": expected_mantle, "blake3": initial_mantle_blake3},
            "source_state_blake3": source_state_blake3,
            "offline_source_preflight": source_report,
            "offline_replay": {
                "mode": "SourceFetchOverridePlan --offline-source-preflight --no-substitute",
                "counts": offline_report["counts"],
                "outcomes": offline_report["outcomes"],
                "advisory_ast_grep_structural_evidence_diagnostics": offline_report["ast_grep_structural_evidence_diagnostics"],
                "full_report": str(run_dir / "offline-source-replay.json"),
            },
            "original_reviewed_archives_sha256": REVIEWED_ARCHIVES,
            "signed_runtime_cohort": {
                "glibc": {"source_store_path": str(GLIBC), "nar_sha256": GLIBC_NAR_SHA256, "nar_size": 35073128},
                "libgcc": {"source_store_path": str(LIBGCC), "nar_sha256": LIBGCC_NAR_SHA256, "nar_size": 197672},
            },
            "environment": {
                "target_system": "x86_64-linux",
                "host_os": sys.platform,
                "host_arch": platform.machine(),
                "source_fetch_mode": "offline-pinned-source-bundle-and-sha256-checked-file-overrides",
                "SOURCE_DATE_EPOCH": 315532800,
                "LC_ALL": "C",
                "TZ": "UTC",
                "sandbox_shell_blake3": blake3(BUSYBOX, repo),
                "fresh_store_and_state_count": 3,
            },
            "builds": results,
            "tamper_negative": {"flipped_signed_dex_payload_byte_offset": results[0]["structure"]["dex_offset"], "apksigner_verify": tamper_result},
            "test_result_lines": lines,
            "full_build_log": str(evidence / "real-build-full.log"),
            "non_claims": ["No install on a device or emulator", "No launch or runtime-behavior proof", "Test-only keystore, not a production release key"],
        }
        pending = receipt.with_suffix(".json.partial")
        pending.write_text(json.dumps(success, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        receipt_blake3 = blake3(pending, repo)
        if blocker.exists():
            blocker.unlink()
        pending.replace(receipt)
        print(f"APK_PROOF_SUCCESS receipt={receipt} blake3={receipt_blake3}")
        for line in lines:
            print(line)
    except Exception as error:
        evidence.mkdir(parents=True, exist_ok=True)
        blocker.write_text(json.dumps({"schema": "mantle-android-reviewed-signed-java-apk-proof-v1", "status": "blocked", "stage": stage, "error": str(error), "success_receipt_written": False}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"APK_PROOF_BLOCKED stage={stage} blocker={blocker}: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
