#!/usr/bin/env python3
"""Inert summary of typed --source-package calls in the two frozen Mantle recipes."""
import json
from pathlib import Path

ROOT = Path("/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index/inputs/mantle-source/bootstrap/source-built")

def has_source_package(step):
    operation = step.get("operation", {})
    return any(
        part.get("kind") == "literal" and part.get("text") == "--source-package"
        for word in operation.get("arguments", [])
        for part in word.get("parts", [])
    )

report = {}
for stage in ("mantle-stage1", "mantle-stage2"):
    recipe = json.loads((ROOT / f"{stage}.json").read_text(encoding="utf-8"))
    report[stage] = []
    for step in recipe["steps"]:
        if not has_source_package(step):
            continue
        arguments = step["operation"]["arguments"]
        report[stage].append({
            "id": step["id"],
            "original_package": step["inputs"]["original-package"],
            "package_output_kind": step["outputs"]["package"]["kind"],
            "arguments": arguments,
        })
print(json.dumps(report, indent=2, sort_keys=True))
