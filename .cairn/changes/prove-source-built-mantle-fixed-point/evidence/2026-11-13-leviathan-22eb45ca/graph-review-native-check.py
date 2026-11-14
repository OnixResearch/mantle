#!/usr/bin/env python3
"""Focused read-only follow-on to source22-graph-review-script.

Run the core graph script first, then:
  python3 local://source22-graph-review-native-check /tmp/mantle-source22-graph-review
The original core result counts all Produced refs and all controller declarations;
this script adds semantic-root, canonical export, native-script state, and
PerlAsm checks, then rewrites only the disposable structured result JSON.
"""
import collections
import json
from pathlib import Path
import sys

ROOT = Path(sys.argv[1])
PATH = ROOT / "source22-graph-review.json"
result = json.loads(PATH.read_text(encoding="utf-8"))
assert result["total"]["steps"] == sum(v["steps"] for v in result["recipes"].values())
source = "/home/brittonr/sbfp-20261113-berdKH/runner-source-22eb45ca/src/"
result["frozen_authoring_source_blake3"].update({source + name: digest for name, digest in {
    "source_built_program_authoring.rs": "23c52720c463872d698140fd2043fd938125692991e2b57a3fccb089d3bf7c9b",
    "source_built_stagex_programs.rs": "72d8bbcc80dd9eecf127b4d465e05f1b1aa14bca7423921fdf69c8636c5bc050",
    "source_built_native_programs.rs": "f9279a4e40d4e58ea2752211f795d1e8b6f35f5279e47e4d0f7612c3491efeae",
    "source_built_rust_programs.rs": "dd4c2abfec0de583680a72dec332586ec329b138ade03102d466773c77eccd7e",
    "source_built_recipe_controller.rs": "ddfd8444bcc9e70d3c32fb4dcd2af4b181109df4b8cf0e62527bd09e8721e755",
    "source_built_fixed_point_stagex_actions.rs": "8e9d7f09cd03c5880a3e4fdeb59dd599cac609e4b427e0d8153657dbd97b4784",
    "source_built_mantle_programs_native.rs": "825b16784805d18c65db9243227771b0e816dee14d33fa2f70cb7d4bf18eb122",
    "source_built_mantle_programs_native_request.rs": "ea8e6aae0bd87fcbaf5428d07324fdc80bf0af9a9ce97eeb6962ff3681fb886c",
}.items()})

def is_produced(ref):
    return isinstance(ref, dict) and ref.get("kind") == "produced"


def ref_key(ref):
    return (ref.get("action_id"), ref.get("output_id"))


def issue(key, item):
    bucket = result["issues"].setdefault(key, {"count": 0, "samples": []})
    bucket["count"] += 1
    if len(bucket["samples"]) < 8:
        bucket["samples"].append(item)

# The broad name-based debug counter in the core script includes native-request-tool:
# genuine native-provider executables are intentionally external. Preserve
# those counters with their honest name; focused state counts below exclude tools.
result["total"]["broad_native_alias_bindings"] = result["total"].pop("native_script_bindings")
examples = result["total"].pop("native_script_nonlocal_examples")
result["total"]["broad_native_alias_external_tool_examples"] = examples
for stage in ("mantle-stage1", "mantle-stage2"):
    recipe_info = result["recipes"][stage]
    recipe_info["broad_native_alias_bindings"] = recipe_info.pop("native_script_local_bindings")

rust_stage = "full-source-rust-provider"
with (ROOT / "rust-program.json").open(encoding="utf-8") as handle:
    rust = json.load(handle)
canonical = rust["exports"].get(rust_stage)
actual_semantic = rust["semantic_roots"].get(rust_stage)
provider_steps = {rust_stage + "/" + step["id"]: step["outputs"] for step in rust["steps"]}
provider_ok = (
    is_produced(canonical)
    and canonical["action_id"].startswith(rust_stage + "/")
    and canonical["output_id"] == rust_stage
    and actual_semantic == [canonical]
    and canonical in rust["roots"]
    and provider_steps.get(canonical["action_id"], {}).get(canonical["output_id"], {}).get("kind") == "tree"
)
result["rust_provider_canonical_export"] = {"reference": canonical, "semantic_root": actual_semantic, "producer_output_kind": (provider_steps.get(canonical["action_id"], {}).get(canonical["output_id"], {}).get("kind") if is_produced(canonical) else None), "valid": provider_ok}
if not provider_ok:
    issue("noncanonical_rust_provider", {"export": canonical, "semantic": actual_semantic})
del rust, provider_steps

STATE_ALIASES = {"script", "original-script", "native-request", "native-request-invocation",
                 "native-environment-data", "native-compiler-files", "native-temporary"}
TREE_ALIASES = {"cc-work", "package", "source", "native-files", "native-compiler-files", "native-temporary"}
SCRIPT_WORD = {"input", "text_input", "text_slice"}

for stage in ("mantle-stage1", "mantle-stage2"):
    with (ROOT / (stage + ".json")).open(encoding="utf-8") as handle:
        recipe = json.load(handle)
    steps = {stage + "/" + s["id"]: s for s in recipe["steps"]}
    edges = {name: {ref["action_id"] for ref in step["inputs"].values()
                     if is_produced(ref) and ref["action_id"] in steps}
             for name, step in steps.items()}
    for name, step in steps.items():
        if step["operation"]["kind"] == "observe-phase":
            edges[name].add(stage + "/" + step["operation"]["phase"])
    reachable = set()
    queue = [r["action_id"] for r in recipe["roots"] if is_produced(r)]
    while queue:
        name = queue.pop()
        if name in steps and name not in reachable:
            reachable.add(name)
            queue.extend(edges[name])
    categories = collections.Counter()
    failures = []

    def local_ref(name, alias, ref, category, kind=None):
        if not is_produced(ref):
            if not isinstance(ref, dict) or ref.get("kind") not in ("source", "manifest-source"):
                failures.append({"step": name, "alias": alias, "category": category, "reason": "invalid artifact reference"})
            return
        categories[category + "_produced"] += 1
        producer = steps.get(ref["action_id"])
        if producer is None or ref["output_id"] not in producer["outputs"]:
            failures.append({"step": name, "alias": alias, "category": category, "reference": ref, "reason": "nonlocal/missing native-script state"})
        elif kind is not None and producer["outputs"][ref["output_id"]]["kind"] != kind:
            failures.append({"step": name, "alias": alias, "category": category, "reference": ref, "reason": "wrong state artifact kind"})

    for name, step in steps.items():
        op = step["operation"]
        inputs = step["inputs"]
        native_step = "script.native" in step["id"]
        perl_step = ".sources.perl-" in step["id"]
        for alias, ref in inputs.items():
            if alias in ("script", "original-script") or alias.startswith("gcc-phase-environment-"):
                local_ref(name, alias, ref, "script_text")
            if native_step and alias in (STATE_ALIASES - {"script", "original-script"}):
                local_ref(name, alias, ref, "native_text_or_files")
            if (native_step or perl_step) and alias in TREE_ALIASES:
                local_ref(name, alias, ref, "script_tree")
                if alias == "cc-work":
                    local_ref(name, alias, ref, "prior_script_history", "tree")
        if native_step:
            if op.get("cwd_input") is not None:
                alias = op["cwd_input"]
                if alias in inputs:
                    local_ref(name, alias, inputs[alias], "script_cwd")
                else:
                    failures.append({"step": name, "reason": "unbound native-script cwd"})
            if op.get("invocation_input") is not None:
                alias = op["invocation_input"]
                if alias in inputs:
                    local_ref(name, alias, inputs[alias], "script_invocation_cwd_env", "text")
                else:
                    failures.append({"step": name, "reason": "unbound native-script invocation"})
            for env, word in op.get("environment", {}).items():
                for part in word.get("parts", []):
                    alias = part.get("name") if part.get("kind") in SCRIPT_WORD else None
                    if alias in ("native-environment-data", "native-request-invocation") or (alias or "").startswith("gcc-phase-environment-"):
                        if alias in inputs:
                            local_ref(name, alias, inputs[alias], "script_environment", "text")
                        else:
                            failures.append({"step": name, "environment": env, "alias": alias, "reason": "unbound native-script env"})
            state = op.get("command_state") or {}
            alias = state.get("prior_times_input")
            if alias:
                if alias in inputs:
                    local_ref(name, alias, inputs[alias], "command_history", "text")
                else:
                    failures.append({"step": name, "alias": alias, "reason": "unbound native-script command history"})
    auth = {name: step for name, step in steps.items() if step["id"].endswith(".authenticate")}
    auth_consumers = {ref["action_id"]: name for name, step in steps.items()
                      for alias, ref in step["inputs"].items()
                      if alias == "original-script" and is_produced(ref)
                      and ref["output_id"] == "authenticated"}
    authentication = {
        "authenticator_steps": len(auth),
        "reachable_from_genuine_roots": sum(name in reachable for name in auth),
        "authenticated_outputs_consumed": sum(name in auth_consumers for name in auth),
        "examples": [{"producer": p, "consumer": auth_consumers.get(p)} for p in list(auth)[:3]],
    }
    if len(auth) != 12 or authentication["reachable_from_genuine_roots"] != 12 or authentication["authenticated_outputs_consumed"] != 12:
        issue("missing_perlasm_authentication", {"stage": stage, "summary": authentication})
    if failures:
        for failure in failures:
            issue("nonlocal_or_invalid_script_state", failure)
    smoke = recipe["exports"].get("mantle-smoke")
    binary = recipe["exports"].get(stage)
    smoke_step = steps.get(stage + "/mantle-help", {})
    operation = smoke_step.get("operation", {})
    smoke_ok = (is_produced(smoke) and smoke["action_id"] == stage + "/mantle-help"
                and smoke["output_id"] == "stdout" and operation.get("kind") == "exec"
                and smoke_step["inputs"].get(operation.get("executable_input")) == binary
                and any(word.get("parts") == [{"kind": "literal", "text": "--help"}]
                        for word in operation.get("arguments", [])))
    binary_ok = (is_produced(binary) and binary["output_id"] == stage
                 and binary["action_id"] in steps
                 and steps[binary["action_id"]]["outputs"].get(stage, {}).get("kind") == "executable")
    if not smoke_ok or not binary_ok:
        issue("noncanonical_mantle_binary_smoke", {"stage": stage, "binary": binary, "smoke": smoke})
    result["recipes"][stage]["authenticator_reachable"] = authentication["reachable_from_genuine_roots"]
    result["recipes"][stage]["native_script_state"] = {
        "state_reference_categories": dict(categories),
        "invalid_or_nonlocal": len(failures),
        "examples": failures[:5],
        "perl_asm": authentication,
        "binary_executable": binary_ok,
        "smoke_help_uses_binary": smoke_ok,
        "all_actions_reachable": len(reachable) == len(steps),
    }
    if len(reachable) != len(steps):
        issue("reachability_crosscheck", {"stage": stage, "reachable": len(reachable), "steps": len(steps)})
    del recipe, steps, edges
result["consumer_semantics"]["native_script"] = (
    "Static Produced text/tree/cwd/env/history and generated invocation references checked against same-stage producer output kinds; "
    "predecessor native compiler/tool/runtime images are intentionally cross-stage; actual generated request bytes and dynamic cwd/env remain execution-time obligations."
)
result["advisory_static_admission"] = "pass" if not result["issues"] else "reject"
PATH.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(json.dumps({"result_path": str(PATH), "advisory_static_admission": result["advisory_static_admission"],
                  "rust_canonical": provider_ok,
                  "mantle": {stage: result["recipes"][stage]["native_script_state"] for stage in ("mantle-stage1", "mantle-stage2")},
                  "issues": result["issues"]}, sort_keys=True))
