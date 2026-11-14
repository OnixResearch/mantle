#!/usr/bin/env python3
"""Read-only exact guarded-output admission of six captured source recipes.

Run after the graph inspection:
  python3 local://source22-graph-review-guard-check /tmp/mantle-source22-graph-review
Mirror frozen source_built_recipe.rs:693-699,788-887,1823-1840. This
checks guard predicates, forbidden guarded roots/exports/semantic outputs,
producer guard dominance on direct inputs, permitted optional command outputs,
prior-stage guarded inputs, and explicit ObservePhase boundaries. It does not
repeat the previously completed generic graph/reachability/cycle inspection.
"""
import json
from pathlib import Path
import sys

ROOT = Path(sys.argv[1])
GRAPH_PATH = ROOT / "source22-graph-review.json"
REPORT_PATH = ROOT / "source22-guard-review.json"
STAGES = [
    ("stagex-transition", "stagex-transition.json"),
    ("stagex-provider-publication", "stagex-provider-publication.json"),
    ("full-source-native-provider", "native-program.json"),
    ("full-source-rust-provider", "rust-program.json"),
    ("mantle-stage1", "mantle-stage1.json"),
    ("mantle-stage2", "mantle-stage2.json"),
]


def produced(ref):
    return isinstance(ref, dict) and ref.get("kind") == "produced" and isinstance(ref.get("action_id"), str) and isinstance(ref.get("output_id"), str)


def valid_codes(codes):
    return isinstance(codes, list) and 1 <= len(codes) <= 8 and all(type(code) is int and 0 <= code <= 255 for code in codes) and all(a < b for a, b in zip(codes, codes[1:]))


def issue(issues, name, sample):
    bucket = issues.setdefault(name, {"count": 0, "examples": []})
    bucket["count"] += 1
    if len(bucket["examples"]) < 8:
        bucket["examples"].append(sample)


def issue_count(issues):
    return sum(item["count"] for item in issues.values())


def lower_guard(stage, step, actions, issues):
    guard = step["operation"].get("guard")
    if guard is None:
        return None
    consumer = stage + "/" + step["id"]
    conditions = guard.get("conditions") if isinstance(guard, dict) else None
    if not isinstance(conditions, list) or not 1 <= len(conditions) <= 8:
        issue(issues, "invalid_guard_predicate", {"action": consumer, "reason": "requires 1..8 conditions"})
        return ()
    result, seen = [], set()
    for condition in conditions:
        if not isinstance(condition, dict):
            issue(issues, "invalid_guard_predicate", {"action": consumer, "reason": "malformed condition"})
            continue
        alias, codes = condition.get("status_input"), condition.get("accepted_codes")
        if not valid_codes(codes):
            issue(issues, "invalid_guard_predicate", {"action": consumer, "status_input": alias, "reason": "codes must be sorted distinct normal exit codes", "codes": codes})
        ref = step["inputs"].get(alias)
        if not produced(ref) or ref["output_id"] != "exit-status":
            issue(issues, "invalid_guard_predicate", {"action": consumer, "status_input": alias, "reason": "status input is not a Produced exit-status", "reference": ref})
            continue
        action = ref["action_id"]
        if action in seen:
            issue(issues, "invalid_guard_predicate", {"action": consumer, "status_input": alias, "reason": "duplicate status producer", "producer": action})
        seen.add(action)
        status = actions.get(action)
        if status is None or status["ordinal"] >= actions[consumer]["ordinal"] or status["kind"] not in ("exec", "controller") or status["outputs"].get("exit-status", {}).get("kind") != "text":
            issue(issues, "invalid_guard_predicate", {"action": consumer, "status_input": alias, "reason": "status must be a prior local command's Text exit-status", "producer": action})
        result.append((action, tuple(codes) if isinstance(codes, list) else ()))
    return tuple(result)


def inspect():
    graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    assert graph["issues"] == {} and graph["total"]["unresolved"] == 0, "run generic graph inspector first"
    report = {
        "scope": "Source-only guard admission; no generic graph rewalk or runtime proof",
        "source": "runner-source-22eb45ca/src/source_built_recipe.rs:693-699,788-887,1823-1840; source_built_fixed_point_action_trust.rs:79-82",
        "per_recipe": {}, "issues": {},
    }
    issues = report["issues"]
    prior = {}  # minimal exact predecessor metadata; do not rewalk their graphs
    for stage, file in STAGES:
        path = ROOT / file
        assert path.stat().st_size == graph["recipes"][stage]["file_identity"]["bytes"]
        with path.open(encoding="utf-8") as handle:
            recipe = json.load(handle)
        assert recipe["stage_id"] == stage
        actions = {}
        for ordinal, step in enumerate(recipe["steps"]):
            op = step["operation"]
            action = stage + "/" + step["id"]
            actions[action] = {
                "ordinal": ordinal, "kind": op["kind"], "outputs": step["outputs"],
                "guarded": op.get("guard") is not None,
                "optional_failure": op["kind"] == "exec" and op.get("exit_policy", {}).get("kind") in ("one-of", "normal") and "exit-status" in step["outputs"],
            }
        guards = {}
        counts = {key: 0 for key in (
            "guarded_producers", "optional_failure_producers", "guarded_root_references",
            "guarded_export_references", "guarded_semantic_root_references",
            "optional_failure_root_references", "optional_failure_export_references",
            "optional_failure_semantic_root_references", "guarded_input_edges",
            "optional_failure_input_edges", "guarded_prior_stage_input_edges",
            "invalid_guard_predicates", "invalid_guarded_input_edges",
            "phase_observations", "invalid_observed_phases",
        )}
        for step in recipe["steps"]:
            action = stage + "/" + step["id"]
            before = issue_count(issues)
            guards[action] = lower_guard(stage, step, actions, issues)
            counts["invalid_guard_predicates"] += issue_count(issues) - before
        counts["guarded_producers"] = sum(meta["guarded"] for meta in actions.values())
        counts["optional_failure_producers"] = sum(meta["optional_failure"] for meta in actions.values())
        for category, refs in (
            ("root", recipe["roots"]), ("export", recipe["exports"].values()),
            ("semantic_root", (ref for values in recipe["semantic_roots"].values() for ref in values)),
        ):
            for ref in refs:
                if not produced(ref):
                    continue  # general reference syntax and ownership checked previously
                producer = actions.get(ref["action_id"])
                if producer is None:
                    continue
                if producer["guarded"]:
                    counts[f"guarded_{category}_references"] += 1
                    issue(issues, "guarded_root_export_semantic", {"stage": stage, "category": category, "reference": ref})
                if producer["optional_failure"] and ref["output_id"] not in ("stdout", "stderr", "exit-status"):
                    counts[f"optional_failure_{category}_references"] += 1
        for step in recipe["steps"]:
            consumer = stage + "/" + step["id"]
            consumer_guard = guards[consumer]
            op = step["operation"]
            if op["kind"] == "observe-phase":
                counts["phase_observations"] += 1
                target = stage + "/" + op["phase"]
                producer = actions.get(target)
                state = op.get("state")
                bad = (
                    producer is None or not producer["guarded"] or producer["ordinal"] >= actions[consumer]["ordinal"]
                    or (state is not None and (
                        producer["outputs"].get(state.get("output"), {}).get("kind") != "tree"
                        or not produced(step["inputs"].get(state.get("skipped_input")))
                        or step["inputs"][state["skipped_input"]]["action_id"] == target
                    ))
                )
                if bad:
                    counts["invalid_observed_phases"] += 1
                    issue(issues, "invalid_observed_guarded_phase", {"consumer": consumer, "phase": target, "state": state})
            for alias, ref in step["inputs"].items():
                if not produced(ref):
                    continue
                target = ref["action_id"]
                producer = actions.get(target)
                if producer is None:
                    producer = prior.get(target)
                    if producer is not None and producer["guarded"]:
                        counts["guarded_prior_stage_input_edges"] += 1
                        issue(issues, "cross_stage_guarded_output", {"consumer": consumer, "alias": alias, "reference": ref})
                    continue
                producer_guard = guards[target]
                guarded = producer_guard is not None
                optional = producer["optional_failure"] and ref["output_id"] not in ("stdout", "stderr", "exit-status")
                counts["guarded_input_edges"] += guarded
                counts["optional_failure_input_edges"] += optional
                if not (guarded or optional):
                    continue
                fault = None
                if consumer_guard is None:
                    fault = "no consumer guard"
                else:
                    index = next((i for i, (action, _) in enumerate(consumer_guard) if action == target), None)
                    if index is None:
                        fault = "missing producer exit-status predicate"
                    elif guarded and consumer_guard[:index][:len(producer_guard)] != producer_guard:
                        fault = "no exact dominating producer guard prefix"
                if fault:
                    counts["invalid_guarded_input_edges"] += 1
                    issue(issues, "unguarded_or_nondominating_input", {"consumer": consumer, "alias": alias, "reference": ref, "fault": fault})
        counts["source_file_identity"] = graph["recipes"][stage]["file_identity"]
        report["per_recipe"][stage] = counts
        prior.update(actions)
        del recipe, actions, guards
    report["advisory_guard_admission"] = "pass" if not issues else "reject"
    fields = [field for field in next(iter(report["per_recipe"].values())) if field != "source_file_identity"]
    report["total"] = {field: sum(item[field] for item in report["per_recipe"].values()) for field in fields}
    report["boundary"] = "Optional-failure ordinary roots are counted, not prohibited, where the frozen validator permits them. Actual guarded outcomes and provider/compilation/fixed-point behavior remain execution-time obligations."
    REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    graph["guard_admission_supplement"] = {
        "result": str(REPORT_PATH), "source": report["source"],
        "advisory_guard_admission": report["advisory_guard_admission"],
        "per_recipe": report["per_recipe"], "issues": issues,
    }
    graph["advisory_static_admission"] = "pass" if not graph["issues"] and not issues else "reject"
    GRAPH_PATH.write_text(json.dumps(graph, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"advisory_guard_admission": report["advisory_guard_admission"], "total": report["total"],
                      "per_recipe": report["per_recipe"], "issues": issues, "result": str(REPORT_PATH)}, sort_keys=True))


if __name__ == "__main__":
    inspect()
