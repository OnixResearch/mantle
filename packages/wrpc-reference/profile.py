import json
import pathlib
import sys

import blake3


def debug_record(name, fields):
    def value_text(value):
        if isinstance(value, bool):
            return str(value).lower()
        if isinstance(value, str):
            return json.dumps(value)
        return str(value)
    return name + " { " + ", ".join(
        key + ": " + value_text(value) for key, value in fields.items()
    ) + " }"


def generate(root, role, tools):
    wit = root / role / "wit/world.wit"
    profile = {
        "profile_id": "molten-wrpc-wasmtime33-unary-sync-v1",
        "evidence_scope": "production",
        "runtime_strategy": "cranelift",
        "toolchain": {
            "wasmtime": "33.0.0", "wasm_tools": tools["wasm-tools"],
            "wasmparser": "0.230.0", "wit_bindgen": "0.36.0",
            "wasmtime_wasi": "33.0.0", "wasi_package": "0.2.3",
        },
        "wit": {"package": "mantle:wrpc-pilot", "world": role,
                "source_ref": "blake3:" + blake3.blake3(wit.read_bytes()).hexdigest()},
        "features": dict.fromkeys([
            "component_model", "multi_value", "bulk_memory", "reference_types", "simd",
            "relaxed_simd", "threads", "tail_call", "multi_memory", "exceptions", "gc",
            "memory64", "extended_const", "function_references", "custom_page_sizes",
            "wide_arithmetic", "component_async",
        ], False),
        "determinism": {
            "fuel_interruption": True, "nan_canonicalization": True,
            "relaxed_simd_deterministic": True, "memory_growth": "bounded",
            "table_growth": "bounded", "host_inputs": "declared-only",
        },
        "resources": {
            "fuel": 10000000, "max_component_bytes": 16777216, "max_wit_bytes": 1048576,
            "max_memory_bytes": 67108864, "max_table_elements": 65536,
            "max_instances": 64, "max_memories": 16, "max_tables": 16,
            "max_stack_bytes": 1048576, "max_hostcall_bytes": 65536,
            "max_result_bytes": 65536, "max_concurrency": 1, "max_imports": 32,
            "max_exports": 32,
        },
        "slicing": None,
        "allowed_imports": ["wrpc-examples:hello/handler"] if role == "client" else [],
        "allowed_wasi_interfaces": [],
        "non_claims": ["not-component-behavior-correctness", "not-runtime-authority",
                       "not-runtime-sandboxing", "not-release-eligibility",
                       "not-remote-rpc-execution"],
    }
    if role == "telemetry":
        profile["profile_id"] = "molten-wrpc-wasmtime33-otel-tracing-v1"
        profile["wit"]["package"] = "mantle:wrpc-telemetry-pilot"
        profile["wit"]["world"] = "emitter"
        profile["allowed_imports"] = ["wasi:otel/tracing@0.2.0-rc.2"]
    for feature in ["component_model", "multi_value", "bulk_memory", "reference_types"]:
        profile["features"][feature] = True
    lines = ["profile-id:" + profile["profile_id"], "evidence-scope:production",
             "runtime-strategy:" + profile["runtime_strategy"]]
    lines += [key.replace("_", "-") + ":" + value for key, value in profile["toolchain"].items()]
    lines += ["wit-" + key.replace("_", "-") + ":" + value for key, value in profile["wit"].items()]
    for key, name in [("features", "FeatureCohort"), ("determinism", "DeterminismProfile"),
                      ("resources", "ResourceLimits")]:
        lines.append(key + ":" + debug_record(name, profile[key]))
    lines += ["import:" + value for value in sorted(profile["allowed_imports"])]
    lines += ["wasi:" + value for value in sorted(profile["allowed_wasi_interfaces"])]
    lines += ["non-claim:" + value for value in profile["non_claims"]]
    identity = blake3.blake3("\n".join(lines).encode()).hexdigest()
    (root / f"{role}/runtime-profile.json").write_text(json.dumps(profile, indent=2) + "\n")
    (root / f"{role}/runtime-profile-ref.txt").write_text(identity + "\n")


root = pathlib.Path(sys.argv[1])
manifest = json.loads(pathlib.Path(sys.argv[2]).read_text())
tools = {tool["name"]: tool["version"] for tool in manifest["tools"]}
for role in (["telemetry"] if len(sys.argv) > 3 and sys.argv[3] == "telemetry" else ["client", "server"]):
    generate(root, role, tools)
