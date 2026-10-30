import json
import pathlib
import sys

import blake3

MAX_MEMBERS = 40000
MAX_MEMBER_BYTES = 536870912


def measure(root, path):
    if path.is_symlink() or not path.is_file():
        raise ValueError("bundle members must be regular files")
    size = path.stat().st_size
    if size > MAX_MEMBER_BYTES:
        raise ValueError("bundle member exceeds byte limit")
    digest = blake3.blake3()
    with path.open("rb") as stream:
        while chunk := stream.read(65536):
            digest.update(chunk)
    relative = path.relative_to(root).as_posix()
    return {"path": relative, "role": relative.split("/")[0],
            "digest_blake3": digest.hexdigest(), "size_bytes": size}


def bundle_entry(root, role):
    bundle_path = root / role / "materialization-bundle.json"
    bundle = json.loads(bundle_path.read_text())
    return {
        "bundle": f"{role}/materialization-bundle.json",
        "bundle_identity_blake3": bundle["bundle_identity_blake3"],
        "final_portable": pathlib.Path(bundle["final_portable"]["logical_path"]).relative_to(root).as_posix(),
        "runtime_profile": f"{role}/runtime-profile.json",
        "runtime_profile_ref": "blake3:" + bundle["expected_runtime_profile_blake3"],
        "world": "emitter" if role == "telemetry" else role,
    }


root = pathlib.Path(sys.argv[1]).resolve()
telemetry = len(sys.argv) > 2 and sys.argv[2] == "telemetry"
entries = sorted(root.rglob("*"))
if any(path.is_symlink() for path in entries):
    raise ValueError("bundle may not retain symlinks, including dangling links")
paths = [path for path in entries if path.is_file()]
if len(paths) > MAX_MEMBERS:
    raise ValueError("bundle exceeds member bound")
manifest = {
    "capability_root": "capability",
    "member_count_max": MAX_MEMBERS,
    "schema": "mantle-wrpc-telemetry-reference-bundle-v1" if telemetry else "mantle-wrpc-reference-bundle-v1",
    "source": json.loads((root / "cohort.json").read_text()),
    "members": [measure(root, path) for path in paths],
    "non_claims": ["not-remote-rpc-execution", "not-runtime-authority",
                   "not-runtime-sandboxing", "not-release-eligibility"],
}
if telemetry:
    manifest.update({
        "telemetry": bundle_entry(root, "telemetry"),
        "effect_interface": "wasi:otel/tracing@0.2.0-rc.2",
        "type_only_dependencies": ["wasi:otel/types@0.2.0-rc.2", "wasi:clocks/wall-clock@0.2.0"],
        "entrypoint": "run", "modes": {"0": "safe-span", "1": "secret-attribute", "2": "33-spans"},
        "expected_result": 42,
        "comparison_only": ["telemetry/artifacts/runtime-composed.wasm", "comparison/sink.wasm"],
    })
else:
    manifest.update({
        "client": bundle_entry(root, "client"), "server": bundle_entry(root, "server"),
        "remote_interface": "wrpc-examples:hello/handler", "remote_function": "hello",
        "client_entrypoint": "run", "expected_client_result": 42,
        "expected_remote_result": "hello from Rust",
        "comparison_only": ["client/artifacts/runtime-composed.wasm"],
    })
encoded = json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()
manifest["manifest_identity_blake3"] = blake3.blake3(encoded).hexdigest()
(root / "manifest.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
