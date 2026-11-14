#!/usr/bin/env python3
"""Read-only, bounded inspection of six published source-built recipes.

Run: python3 /path/to/source22-graph-review-script /tmp/mantle-source22-graph-review
Uses only locally transferred inert JSON source bytes; it does not run any recipe.
The named remote paths and their BLAKE3 digests are pinned below. No profile JSON
or generated proof outcome is consumed. The result is written under the given
scratch directory and printed as a small summary.
"""
import collections
import json
import pathlib
import subprocess
import sys

COHORT = "/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed"
STAGES = [
    ("stagex-transition", "stagex-transition.json", f"{COHORT}/program-sources/source-built-stagex-programs/stagex-transition.json", 31730468, "8340c69c2e898193ba7a95d861718c1681e5d446c7e5cf7210be784eb18d9e34"),
    ("stagex-provider-publication", "stagex-provider-publication.json", f"{COHORT}/program-sources/source-built-stagex-programs/stagex-provider-publication.json", 52226, "95b34b8881195c596046834e806f0324150387b2fc4a91c09ef05f499d4eccd8"),
    ("full-source-native-provider", "native-program.json", f"{COHORT}/program-sources/fixed-point-native-source-program/source-built/native/program.json", 18268649, "aaf5403428b921f312b8c74336a8dba56b230fd657b88b4b5fa9d28b0fdd413c"),
    ("full-source-rust-provider", "rust-program.json", f"{COHORT}/program-sources/fixed-point-rust-source-program/source-built/rust/program.json", 259029252, "2401c71c1efe2053251f9e65021b4a9ec16fcc98359f5b4536359ca75c01a407"),
    ("mantle-stage1", "mantle-stage1.json", f"{COHORT}/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json", 172660412, "cf357884b5684f5bc5c327133657490199da2defcb6c91621c297eadd0056c62"),
    ("mantle-stage2", "mantle-stage2.json", f"{COHORT}/inputs/mantle-source/bootstrap/source-built/mantle-stage2.json", 172660412, "55d0b619ef57aa41f71e0c54ef42569a07005ca5e0f13cd6ed85124bb12dd04e"),
]
STAGE_ORDER = [s for s, *_ in STAGES]
FROZEN = "/home/brittonr/sbfp-20261113-berdKH/runner-source-22eb45ca/src/"
AUTHORING_DIGESTS = {
    "source_built_recipe.rs": "28ca15f8081b88c5747a851a1474d92277fba142f0b7fdfa1f74d8a48374d009",
    "source_built_mantle_programs.rs": "70973ba46431cbbda391a55c9a0724215e3005f25d2f1a1a9d4227d4b5f07921",
    "source_built_mantle_programs_graph.rs": "6d8ec9bd415be54d08fcb71fba531920075c235c0bf56081d5992365a0c6de10",
}


def identity(file, expected_bytes, expected_hash):
    size = file.stat().st_size
    assert size == expected_bytes, f"{file}: expected {expected_bytes} bytes; got {size}"
    digest = subprocess.run(["b3sum", str(file)], capture_output=True, text=True, check=True).stdout.split()[0]
    assert digest == expected_hash, f"{file}: unexpected BLAKE3 {digest} != {expected_hash}"
    return {"bytes": size, "blake3": digest}


def produced(value):
    return isinstance(value, dict) and value.get("kind") == "produced" and "action_id" in value and "output_id" in value


def embedded_references(obj, path="operation"):
    """Also walk controller declarations: fixed/inventory/slot refs are dependencies."""
    if produced(obj):
        yield path, obj
    elif isinstance(obj, dict):
        for key, item in obj.items():
            yield from embedded_references(item, f"{path}.{key}")
    elif isinstance(obj, list):
        for i, item in enumerate(obj):
            yield from embedded_references(item, f"{path}[{i}]")


def owner(action_id):
    for stage in STAGE_ORDER:
        if isinstance(action_id, str) and action_id.startswith(stage + "/"):
            return stage
    return None


def append_issue(issues, key, sample):
    issue = issues[key]
    issue["count"] += 1
    if len(issue["samples"]) < 8:
        issue["samples"].append(sample)


def inspect(root):
    result = {
        "scope": "Source-only graph inspection, not source-profile verification, execution, compiler correctness, fixed point or release eligibility",
        "cohort": COHORT,
        "frozen_source": "/home/brittonr/sbfp-20261113-berdKH/runner-source-22eb45ca",
        "frozen_revision": "22eb45ca5321416fce3278a228a43224dd0026c5",
        "frozen_authoring_source_blake3": {FROZEN + name: digest for name, digest in AUTHORING_DIGESTS.items()},
        "recipes": {}, "issues": {}, "consumer_semantics": {
            "reference_source": "source_built_recipe.rs:644-764; source_built_recipe_controller.rs:42-79",
            "reachability": "backwards over step inputs and ObservePhase phase, plus embedded controller specification refs; external stage refs checked against exact predecessor producers and outputs",
            "mantle_roots": "source_built_mantle_programs.rs:143-161,768-937; exactly semantic values plus binary and --help smoke, no blanket helper roots",
        },
    }
    issues = collections.defaultdict(lambda: {"count": 0, "samples": []})
    producers = {}  # action ID -> (stage, output names, ordinal)
    dependencies = {}  # action ID -> producer action IDs (including cross-stage)
    stage_ids = {}
    stage_info = {}
    nested_used = collections.Counter()
    native_aliases = collections.Counter()
    native_cross = []
    pending_refs = []
    order = dict(zip(STAGE_ORDER, range(len(STAGE_ORDER))))

    def reference(ref, source, location, root_type=None):
        if not produced(ref):
            append_issue(issues, "invalid_produced_reference", {"source": source, "location": location, "value": str(ref)[:180]})
            return
        target = ref["action_id"]
        if not isinstance(target, str) or not isinstance(ref["output_id"], str):
            append_issue(issues, "invalid_produced_reference", {"source": source, "location": location, "value": str(ref)[:180]})
            return
        location_stage = owner(source)
        target_stage = owner(target)
        if location_stage is None:  # roots/exports: source is a stage ID
            location_stage = source
        if root_type is not None and target_stage != location_stage:
            append_issue(issues, "nonlocal_root_export_semantic", {"source": source, "location": location, "target": target})
        if target_stage is None:
            append_issue(issues, "unknown_stage_producer", {"source": source, "location": location, "target": target})
        elif target_stage != location_stage and root_type is None:
            stage_info[location_stage]["cross_stage_references"] += 1
            stage_info[location_stage]["cross_stage_by_producer"][target_stage] += 1
            if order[target_stage] >= order[location_stage]:
                append_issue(issues, "cross_stage_forward_reference", {"source": source, "location": location, "target": target})
        stage_info[location_stage]["produced_references"] += 1
        pending_refs.append((source, location, target, ref["output_id"], root_type))
        if root_type is None and source in dependencies:
            dependencies[source].add(target)

    for stage, name, remote, size, digest in STAGES:
        path = root / name
        file_id = identity(path, size, digest)
        with path.open(encoding="utf-8") as source:
            recipe = json.load(source)
        assert recipe.get("schema") == "mantle-source-recipe-v1"
        assert recipe["stage_id"] == stage, (path, recipe.get("stage_id"))
        steps = recipe["steps"]
        assert 0 < len(steps) <= 100000, (stage, len(steps))
        stage_ids[stage] = []
        info = {
            "source_path": remote, "captured_path": str(path), "file_identity": file_id,
            "steps": len(steps), "roots": len(recipe["roots"]),
            "root_distinct_references": len({(r.get("action_id"), r.get("output_id")) for r in recipe["roots"]}),
            "exports": len(recipe["exports"]), "semantic_root_keys": len(recipe["semantic_roots"]),
            "semantic_root_references": sum(len(refs) for refs in recipe["semantic_roots"].values()),
            "produced_references": 0, "cross_stage_references": 0,
            "cross_stage_by_producer": collections.Counter(),
            "operation_kinds": collections.Counter(), "native_script_local_bindings": collections.Counter(),
            "nested_controller_specification_references": 0,
            "timeout_groups": len(recipe.get("timeout_groups", [])),
        }
        stage_info[stage] = info
        if not recipe["roots"]:
            append_issue(issues, "empty_roots", {"stage": stage})
        for i, step in enumerate(steps):
            action_id = stage + "/" + step["id"]
            if action_id in producers:
                append_issue(issues, "duplicate_producer", {"stage": stage, "step": step["id"]})
            producers[action_id] = (stage, set(step["outputs"]), i)
            dependencies[action_id] = set()
            stage_ids[stage].append(action_id)
            info["operation_kinds"][step["operation"]["kind"]] += 1
        # The local producer inventory exists before references are read; the
        # ordinal check below rejects cyclic or late edges even when IDs exist.
        for step in steps:
            source_id = stage + "/" + step["id"]
            for alias, ref in step["inputs"].items():
                if produced(ref):
                    reference(ref, source_id, "inputs." + alias)
                    if stage.startswith("mantle-") and (
                        alias in ("script", "original-script", "native-environment-data")
                        or alias.startswith(("native-script", "native-request", "native-compiler", "gcc-phase-environment"))
                        or alias.endswith(("-history", "-cwd"))
                    ):
                        native_aliases[(stage, alias)] += 1
                        owner_stage = owner(ref["action_id"])
                        info["native_script_local_bindings"]["local" if owner_stage == stage else "nonlocal"] += 1
                        if owner_stage != stage and len(native_cross) < 8:
                            native_cross.append({"step": source_id, "alias": alias, "target": ref})
                elif not isinstance(ref, dict) or ref.get("kind") not in ("source", "manifest-source"):
                    append_issue(issues, "invalid_input_reference", {"source": source_id, "alias": alias, "value": str(ref)[:180]})
            spec_refs = list(embedded_references(step["operation"]))
            if spec_refs:
                info["nested_controller_specification_references"] += len(spec_refs)
                declared = {(r["action_id"], r["output_id"]) for r in step["inputs"].values() if produced(r)}
                for location, ref in spec_refs:
                    nested_used[stage] += 1
                    reference(ref, source_id, location)
                    if (ref["action_id"], ref["output_id"]) not in declared:
                        append_issue(issues, "unbound_nested_controller_ref", {"source": source_id, "location": location, "target": ref})
            if step["operation"]["kind"] == "observe-phase":
                observed = stage + "/" + step["operation"]["phase"]
                dependencies[source_id].add(observed)
                if observed not in producers:
                    append_issue(issues, "unknown_observed_phase", {"source": source_id, "target": observed})
        for i, ref in enumerate(recipe["roots"]):
            reference(ref, stage, f"roots[{i}]", "root")
        for key, ref in recipe["exports"].items():
            reference(ref, stage, f"exports.{key}", "export")
        for key, refs in recipe["semantic_roots"].items():
            if not refs:
                append_issue(issues, "empty_semantic_root", {"stage": stage, "key": key})
            for i, ref in enumerate(refs):
                reference(ref, stage, f"semantic_roots.{key}[{i}]", "semantic")
        info["root_nodes"] = list(dict.fromkeys(r["action_id"] for r in recipe["roots"] if produced(r)))
        if stage.startswith("mantle-"):
            semantic = {(r["action_id"], r["output_id"]) for refs in recipe["semantic_roots"].values() for r in refs if produced(r)}
            expected_semantic = {r for r in recipe["semantic_roots"] if r.startswith(("rust-unit:", "build-script:"))}
            other_semantic = sorted(set(recipe["semantic_roots"]) - expected_semantic)
            if other_semantic:
                append_issue(issues, "mantle_nonselected_semantic_root", {"stage": stage, "keys": other_semantic[:8]})
            binary = recipe["exports"].get(stage)
            smoke = recipe["exports"].get("mantle-smoke")
            binary_ref = (binary.get("action_id"), binary.get("output_id")) if produced(binary) else None
            smoke_ref = (smoke.get("action_id"), smoke.get("output_id")) if produced(smoke) else None
            actual = [(r.get("action_id"), r.get("output_id")) for r in recipe["roots"]]
            expected = semantic | {binary_ref, smoke_ref}
            extra = set(actual) - expected
            missing = expected - set(actual)
            info["mantle_root_contract"] = {
                "rust_units": sum(k.startswith("rust-unit:") for k in recipe["semantic_roots"]),
                "build_scripts": sum(k.startswith("build-script:") for k in recipe["semantic_roots"]),
                "semantic_ref_distinct": len(semantic), "binary": binary,
                "smoke": smoke, "binary_in_semantic_products": binary_ref in semantic,
                "exact_union": not extra and not missing and len(actual) == len(set(actual)),
                "extra_sample": sorted(extra)[:5], "missing_sample": sorted(missing)[:5],
            }
            if not info["mantle_root_contract"]["exact_union"]:
                append_issue(issues, "mantle_root_union_mismatch", {"stage": stage, "extra": sorted(extra)[:5], "missing": sorted(missing)[:5]})
            # Every selected script has its two authentic stdout/out-dir
            # products; each rust-unit has exactly its selected product.
            for key, refs in recipe["semantic_roots"].items():
                outputs = {r.get("output_id") for r in refs}
                if (key.startswith("rust-unit:") and len(refs) != 1) or (key.startswith("build-script:") and outputs != {"stdout", "out-dir"}):
                    append_issue(issues, "mantle_semantic_product_shape", {"stage": stage, "key": key, "references": refs[:3]})
            authenticators = [s["id"] for s in steps if s["id"].endswith(".authenticate")]
            info["authenticator_steps"] = len(authenticators)
            info["authenticator_examples"] = authenticators[:5]
        del recipe, steps

    for source_id, location, target, output_id, root_type in pending_refs:
        target_record = producers.get(target)
        if target_record is None:
            append_issue(issues, "unresolved_producer", {"source": source_id, "location": location, "target": target})
        elif output_id not in target_record[1]:
            append_issue(issues, "unresolved_output", {"source": source_id, "location": location, "target": target, "output": output_id})
        elif root_type is None and source_id in producers:
            source_record = producers[source_id]
            if target_record[0] == source_record[0] and target_record[2] >= source_record[2]:
                append_issue(issues, "late_or_cyclic_producer", {"source": source_id, "location": location, "target": target})

    # Reject cycles over the entire six-stage graph; no recursion depth bound
    # depends on a document's graph height.
    in_degree = {action: len(neighbors & producers.keys()) for action, neighbors in dependencies.items()}
    reverse = collections.defaultdict(list)
    for action, neighbors in dependencies.items():
        for target in neighbors:
            if target in producers:
                reverse[target].append(action)
    queue = collections.deque(action for action, degree in in_degree.items() if degree == 0)
    processed = 0
    while queue:
        target = queue.popleft()
        processed += 1
        for consumer in reverse[target]:
            in_degree[consumer] -= 1
            if in_degree[consumer] == 0:
                queue.append(consumer)
    cyclic_candidates = {action for action, degree in in_degree.items() if degree}
    if cyclic_candidates:
        append_issue(issues, "cycle", {"unprocessed_nodes": len(cyclic_candidates), "sample": sorted(cyclic_candidates)[:8]})

    for stage in STAGE_ORDER:
        info = stage_info[stage]
        reachable = set()
        queue = list(info.pop("root_nodes"))
        while queue:
            action = queue.pop()
            if action not in reachable and action in producers and producers[action][0] == stage:
                reachable.add(action)
                queue.extend(dependencies[action])
        unreachable = sorted(set(stage_ids[stage]) - reachable)
        info["reachable"] = len(reachable)
        info["unreachable"] = len(unreachable)
        info["unreachable_samples"] = unreachable[:12]
        info["cross_stage_by_producer"] = dict(info["cross_stage_by_producer"])
        info["native_script_local_bindings"] = dict(info["native_script_local_bindings"])
        info["operation_kinds"] = dict(info["operation_kinds"])
        info["unresolved"] = sum(
            1 for src, _, target, out, _ in pending_refs
            if (src == stage or owner(src) == stage) and (target not in producers or out not in producers[target][1])
        )
        info["cycle_nodes"] = sum(action in cyclic_candidates for action in stage_ids[stage])
        info["cycles"] = 0 if not info["cycle_nodes"] else "possible; inspect cycle samples"
        if unreachable:
            append_issue(issues, "unreachable_actions", {"stage": stage, "count": len(unreachable), "sample": unreachable[:8]})
        if stage.startswith("mantle-"):
            info["authenticator_reachable"] = sum(stage + "/" + id in reachable for id in info["authenticator_examples"])
    result["issues"] = dict(issues)
    result["recipes"] = {stage: stage_info[stage] for stage in STAGE_ORDER}
    result["total"] = {
        "steps": sum(v["steps"] for v in stage_info.values()),
        "roots": sum(v["roots"] for v in stage_info.values()),
        "reachable": sum(v["reachable"] for v in stage_info.values()),
        "unreachable": sum(v["unreachable"] for v in stage_info.values()),
        "cross_stage_references": sum(v["cross_stage_references"] for v in stage_info.values()),
        "unresolved": sum(v["unresolved"] for v in stage_info.values()),
        "cycle_nodes": len(cyclic_candidates),
        "cycles": 0 if not cyclic_candidates else "possible",
        "nested_controller_specification_references": sum(nested_used.values()),
        "native_script_bindings": sum(native_aliases.values()),
        "native_script_nonlocal_examples": native_cross,
    }
    result["advisory_static_admission"] = "pass" if not issues else "reject"
    result["boundary"] = "No builder/provider execution, proof receipt, compiler correctness, fixed-point convergence, or release eligibility asserted"
    return result


def main():
    assert len(sys.argv) == 2, "Usage: python3 source22-graph-review-script /tmp/mantle-source22-graph-review"
    root = pathlib.Path(sys.argv[1])
    outcome = inspect(root)
    output = root / "source22-graph-review.json"
    output.write_text(json.dumps(outcome, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"result_path": str(output), "advisory_static_admission": outcome["advisory_static_admission"], "total": outcome["total"], "issues": outcome["issues"]}, sort_keys=True))


if __name__ == "__main__":
    main()
