#!/usr/bin/env python3
"""Real generic-build GGUF proof. Requires pre-acquired reviewed tools, no downloads.

This driver does not build tools or accept version strings as content authority.
The reviewed lock binds every tool/native/runtime store tree by BLAKE3.
Nix is used only for explicit source admission/signing, never inside conversion.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

REPO = Path(__file__).resolve().parent.parent


def run(command, *, data=None, env=None, negative=False):
    p = subprocess.run([str(x) for x in command], input=data, text=True, capture_output=True, env=env, timeout=900)
    if (p.returncode == 0) == negative:
        raise RuntimeError(f"unexpected exit {p.returncode}: {command!r}\n{p.stdout[-8000:]}\n{p.stderr[-8000:]}")
    return p


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["mantle", "lowerer", "nix", "b3sum", "cohort-lock", "run-dir"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--cohort-lock-blake3", required=True)
    a = parser.parse_args()
    a.run_dir = a.run_dir.resolve()
    a.run_dir.mkdir()  # Never reuse state, input caches, outputs, or evidence.
    evidence = {"schema": "mantle-model-conversion-proof-v1", "status": "blocked"}

    def write(name, value):
        (a.run_dir / name).write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")

    def digest(path):
        value = run([a.b3sum, "--no-names", path]).stdout.strip()
        if not re.fullmatch("[0-9a-f]{64}", value):
            raise RuntimeError("invalid explicit BLAKE3 observer")
        return value

    try:
        if digest(a.cohort_lock) != a.cohort_lock_blake3:
            raise RuntimeError("reviewed cohort lock content drift")
        plan = json.loads(a.cohort_lock.read_text())
        tool_roots = {t["path"] for t in plan["tools"]}
        if not tool_roots or not all(p.startswith("/nix/store/") and Path(p).is_dir() for p in tool_roots):
            raise RuntimeError("proof needs the complete pre-acquired Nix tool closure")
        closure = run([a.nix, "path-info", "--offline", "--recursive", *sorted(tool_roots)]).stdout.splitlines()
        if set(closure) != tool_roots:
            raise RuntimeError(f"lock omitted transitive native/runtime closure: {sorted(set(closure) - tool_roots)}")
        pyenv = {"PYTHONPATH": plan["python_site"], "PYTHONNOUSERSITE": "1", "PYTHONHASHSEED": "0", "PATH": "", "LC_ALL": "C", "PYTHONDONTWRITEBYTECODE": "1"}
        source = a.run_dir / "fixture"
        generator = REPO / "tests/fixtures/model-conversion/generate.py"
        run([plan["python"], "-s", generator, source], env=pyenv)
        key = a.run_dir / "signing-key"
        secret = run([a.nix, "key", "generate-secret", "--key-name", "mantle-model-proof-1"]).stdout
        key.write_text(secret)
        key.chmod(0o600)
        public = run([a.nix, "key", "convert-secret-to-public"], data=secret).stdout.strip()
        cache = a.run_dir / "signed-input-cache"
        base = [a.mantle, "--json", "--nix-compat", "--store", a.run_dir / "store", "--state-dir", a.run_dir / "state"]

        def admit(roots, label):
            run([a.nix, "copy", "--offline", "--to", f"file://{cache}?compression=none&secret-key={key}", *roots])
            p = run([*base, "store", "pull", "--from", cache, "--trusted-public-keys", public, "--all"])
            (a.run_dir / f"{label}-admission.log").write_text(p.stdout + p.stderr)
            counts = re.search(r"imported=([0-9]+) skipped_present=([0-9]+) skipped_untrusted=([0-9]+) skipped_hash_mismatch=([0-9]+) skipped_missing_nar=([0-9]+) skipped_parse_error=([0-9]+)", p.stderr)
            if counts is None or int(counts[1]) + int(counts[2]) < len(roots) or any(int(counts[i]) for i in range(3, 7)):
                raise RuntimeError("signed input admission rejected a declared root")

        admit(sorted(tool_roots), "tools")

        def bind_model(value, model, refresh=True):
            value = copy.deepcopy(value)
            root = run([a.nix, "store", "add-path", "--offline", model]).stdout.strip()
            if not root.startswith("/nix/store/"):
                raise RuntimeError("fixture did not enter immutable source store")
            admit([root], model.name)
            value["model_root"] = root
            if refresh:
                value["files"] = [{"member": p.name, "bytes": p.stat().st_size, "blake3": digest(p)} for p in sorted(model.iterdir()) if p.is_file()]
            return value

        def lower(value, label, negative=False):
            p = run([a.lowerer, "/nix/store"], data=json.dumps(value), negative=negative)
            (a.run_dir / f"{label}-lowering.stderr").write_text(p.stderr)
            if negative:
                return None
            drv = json.loads(p.stdout)
            path = a.run_dir / f"{label}.json"
            write(path.name, drv)
            recipe = a.run_dir / f"{label}.ncl"
            helper = json.dumps(str(REPO / "lib/model_conversion.ncl"))
            recipe.write_text(f"(import {helper}).derivation (import {json.dumps(str(path))})\n")
            return recipe, drv

        def build(lowered, label, negative=False, expected_error=None):
            recipe, drv = lowered
            p = run([*base, "build", recipe, "--no-substitute", "--strict-hermetic", "--signing-key", key, "--trusted-public-keys", public,
                     "--import-path", REPO / "lib"], negative=negative)
            (a.run_dir / f"{label}-build.stdout").write_text(p.stdout)
            (a.run_dir / f"{label}-build.stderr").write_text(p.stderr)
            if negative:
                report = json.loads(p.stdout)
                logs = [Path(row["saved_log_path"]) for row in report["failed"]]
                if not expected_error or not any(expected_error in path.read_text(errors="replace") for path in logs):
                    raise RuntimeError(f"negative control did not reach expected admission denial: {label}")
                return {"exit_code": p.returncode, "expected_error": expected_error, "saved_logs": [str(p) for p in logs]}
            report = json.loads(p.stdout)
            matches = [o["path"] for row in report["outcomes"] for o in row["outputs"] if row["label"] == drv["name"] and o["name"] == "out"]
            if len(matches) != 1:
                raise RuntimeError("generic build omitted unique converted output")
            physical = Path(matches[0])
            logical = Path("/nix/store") / physical.name
            if physical != a.run_dir / "store" / logical.name:
                raise RuntimeError("generic build report did not export to the isolated physical store")
            verified = run([*base, "store", "verify", logical.name, "--trusted-public-keys", public])
            if not re.search(rf"OK {re.escape(logical.name)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*", verified.stdout) or "1 checked, 0 mismatches" not in verified.stderr:
                raise RuntimeError("converted output lacks verified signed PathInfo")
            write(f"{label}-build-report.json", report)
            provenance = json.loads((physical / "provenance.json").read_text())
            for observed in provenance["runs"]:
                scope = observed["smoke"]["scope"]
                if scope != {"bounded_tensor_projection_smoke": True, "model_quality": False, "semantic_equivalence": False}:
                    raise RuntimeError("numeric receipt overclaims quality/equivalence or omits bounded scope")
            return {"logical": str(logical), "physical": str(physical), "report": report, "provenance": provenance}

        plan = bind_model(plan, source)
        original = lower(plan, "baseline")
        first = build(original, "baseline")
        cached = build(original, "cache-replay")
        if first["logical"] != cached["logical"] or cached["report"]["counts"]["built_total"] != 0 or cached["report"]["counts"]["cached_total"] != 1:
            raise RuntimeError("identical declared action was rebuilt instead of reused")
        changed = a.run_dir / "changed-weights"
        run([plan["python"], "-s", generator, changed, "--changed-weights"], env=pyenv)
        changed_plan = bind_model(plan, changed)
        changed_lowered = lower(changed_plan, "changed-weights")
        if changed_lowered[1]["name"] == original[1]["name"]:
            raise RuntimeError("changed input weights did not invalidate plan identity")
        changed_result = build(changed_lowered, "changed-weights")
        if changed_result["logical"] == first["logical"] or changed_result["provenance"]["runs"][0]["f32_blake3"] == first["provenance"]["runs"][0]["f32_blake3"]:
            raise RuntimeError("changed source tensor failed to change converted bytes/output identity")
        negatives = {}
        denials = {"missing-shard": "undeclared/missing model file", "corrupt-shard": "invalid safetensors header",
                   "tokenizer-mismatch": "tokenizer config mismatch", "architecture": "unsupported architecture/config/options",
                   "undeclared-code": "undeclared/missing model file", "pickle": "undeclared/missing model file"}
        for label, expected_error in denials.items():
            root = a.run_dir / label
            shutil.copytree(source, root)
            refresh = True
            if label == "missing-shard":
                next(root.glob("*.safetensors")).unlink()
                refresh = False
            elif label == "corrupt-shard":
                shard = next(root.glob("*.safetensors"))
                raw = bytearray(shard.read_bytes())
                raw[:8] = (2 ** 64 - 1).to_bytes(8, "little")
                shard.write_bytes(raw)
            elif label == "tokenizer-mismatch":
                path = root / "tokenizer_config.json"
                obj = json.loads(path.read_text()); obj["bos_token"] = "wrong"
                path.write_text(json.dumps(obj))
            elif label == "architecture":
                path = root / "config.json"
                obj = json.loads(path.read_text()); obj["architectures"] = ["UnsupportedForCausalLM"]
                path.write_text(json.dumps(obj))
            else:
                (root / ("pytorch_model.bin" if label == "pickle" else "model.py")).write_bytes(b"undeclared input must never execute\n")
                refresh = False
            value = bind_model(plan, root, refresh)
            negatives[label] = build(lower(value, label), label, negative=True, expected_error=expected_error)
        for label, mutate in [
            ("quantization-option", lambda p: p.update(quantization="Q4_K_M")),
            ("remote-code-option", lambda p: p.update(trust_remote_code=True)),
            ("converter-revision", lambda p: p.update(converter_revision="0" * 40)),
            ("calibration", lambda p: p.update(calibration=[p["files"][0]])),
        ]:
            value = copy.deepcopy(plan); mutate(value)
            lower(value, label, negative=True)
            negatives[label] = {"lowering_rejected": True}
        identities = {}
        for label, mutate in [
            ("settings", lambda p: p["limits"].update(cpu_seconds=299 if p["limits"]["cpu_seconds"] == 300 else p["limits"]["cpu_seconds"] + 1)),
            ("converter-tool-bytes", lambda p: p["tools"][0].update(blake3="a" * 64 if p["tools"][0]["blake3"] != "a" * 64 else "b" * 64)),
        ]:
            value = copy.deepcopy(plan); mutate(value)
            action = lower(value, label)
            if action[1]["name"] == original[1]["name"]:
                raise RuntimeError("changed declared identity failed invalidation")
            identities[label] = action[1]["name"]
            if label == "converter-tool-bytes":
                negatives[label] = build(action, label, negative=True, expected_error="tool/runtime/native dependency content drift")
        evidence.update(status="observed", cohort_lock_blake3=a.cohort_lock_blake3,
                        proof_source_blake3=digest(Path(__file__).resolve()),
                        executable_blake3={name: digest(path) for name, path in [("mantle", a.mantle), ("lowerer", a.lowerer), ("nix", a.nix), ("b3sum", a.b3sum)]},
                        tool_roots=sorted(tool_roots), generator_blake3=digest(generator),
                        baseline=first, cache=cached, changed_weights=changed_result,
                        negative_controls=negatives, invalidated_identities=identities,
                        non_claims=["No model quality/equivalence claim", "No cross-host byte claim", "No source-to-binary compiler correctness claim"])
    except Exception as error:
        evidence["error"] = str(error)
        raise
    finally:
        write("proof.json", evidence)


if __name__ == "__main__":
    main()
