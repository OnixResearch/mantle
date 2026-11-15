#!/usr/bin/env python3
"""Trace inert invocation-template candidates to physical recipe consumers."""
import collections
import json
import pathlib
import sys
from scan import digest, word_issues


def words(value):
    stack = [(value, "$")]
    while stack:
        node, where = stack.pop()
        if isinstance(node, dict):
            if "parts" in node:
                yield where, node
            stack.extend((child, where + "." + key) for key, child in node.items())
        elif isinstance(node, list):
            stack.extend((child, where + "[" + str(at) + "]") for at, child in enumerate(node))
        elif isinstance(node, str) and node.lstrip().startswith(("{", "[")):
            try:
                child = json.loads(node)
            except (ValueError, RecursionError):
                continue
            if isinstance(child, (dict, list)):
                stack.append((child, where + "<decoded-json>"))


def main():
    inventory = json.loads(pathlib.Path(sys.argv[1]).read_text())
    item = next(item for item in inventory["recipes"] if item["stage"] == "full-source-rust-provider")
    path = pathlib.Path(item["physical_source"])
    if digest(path) != item["blake3"]:
        raise RuntimeError("current recipe identity changed")
    recipe = json.loads(path.read_text())
    counts = collections.Counter()
    patterns = {}
    bad_actions = {}
    for at, step in enumerate(recipe["steps"]):
        operation = step["operation"]
        text = operation.get("text")
        if not isinstance(text, str):
            continue
        try:
            decoded = json.loads(text)
        except (ValueError, RecursionError):
            continue
        action = item["stage"] + "/" + step["id"]
        for where, word in words(decoded):
            for failure in word_issues(word):
                counts[failure] += 1
                key = failure + ":" + json.dumps(word, sort_keys=True)
                record = patterns.setdefault(key, {"issue": failure, "word": word, "count": 0, "examples": []})
                record["count"] += 1
                bad_actions.setdefault(action, {"step_index": at, "step_id": step["id"], "operation_kind": operation["kind"], "candidate_locations": [], "immediate_consumers": []})["candidate_locations"].append({"where": where, "issue": failure})
                if len(record["examples"]) < 5:
                    record["examples"].append({"producer": action, "where": where})
    for step in recipe["steps"]:
        for alias, source in step["inputs"].items():
            if not isinstance(source, dict) or source.get("kind") != "produced":
                continue
            owner = bad_actions.get(source.get("action_id"))
            if owner is None:
                continue
            operation = step["operation"]
            invocation = operation.get("invocation", {})
            owner["immediate_consumers"].append({"step_id": step["id"], "alias": alias, "output_id": source["output_id"], "operation_kind": operation["kind"], "executable": invocation.get("executable"), "executable_input": step["inputs"].get(invocation.get("executable")), "arguments": invocation.get("arguments")})
    report = {"scope": "Every failing word candidate in statically JSON-decodable operation.text fields of the current Rust recipe, plus immediate Produced input consumers; no execution", "recipe_path": str(path), "recipe_blake3": item["blake3"], "issues": counts, "distinct_word_patterns": len(patterns), "patterns": list(patterns.values()), "producers": bad_actions, "non_claim": "A template candidate is not itself a defect; actual source-owned template transformations and consumers must be inspected."}
    pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
