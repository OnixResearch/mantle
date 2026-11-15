#!/usr/bin/env python3
"""Read-only lexical RecipeWord inspection; no product or source execution."""
import collections
import json
import pathlib
import subprocess
import sys

B3 = "/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum"
FIELDS = {
    "literal": {"kind", "text"},
    "literal-data": {"kind", "text"},
    "input": {"kind", "name", "path"},
    "output": {"kind", "name", "path"},
    "runtime": {"kind", "mount", "path"},
    "workspace": {"kind", "mount", "path"},
    "cwd_relative": {"kind", "path"},
    "directory_separator": {"kind"},
    "text_input": {"kind", "name"},
    "text_slice": {"kind", "name", "offset_bytes", "length_bytes"},
}
PATH_KINDS = {"input", "output", "runtime", "workspace"}
MAX_RECIPE_BYTES = 512 * 1024 * 1024
MAX_SOURCE_DATA_BYTES = 64 * 1024 * 1024
MAX_WORD_BYTES = 65536
MAX_U64 = (1 << 64) - 1


def digest(path):
    return subprocess.check_output([B3, "--no-names", str(path)], text=True).strip()


def member_error(value, empty):
    if not isinstance(value, str):
        return "member_not_string"
    if not value and not empty:
        return "empty_member"
    if len(value.encode("utf-8")) > 4096 or "\0" in value or "\\" in value:
        return "member_bytes_or_delimiter"
    if value and any(part in ("", ".", "..") for part in value.split("/")):
        return "noncanonical_member"
    return None


def word_issues(word):
    failures = []
    if set(word) != {"parts"}:
        failures.append("word_extra_fields")
    parts = word.get("parts")
    if not isinstance(parts, list) or not 1 <= len(parts) <= 4096:
        return failures + ["word_part_count"]
    literal_bytes = 0
    for at, part in enumerate(parts):
        if not isinstance(part, dict):
            failures.append("part_not_object")
            continue
        kind = part.get("kind")
        if kind not in FIELDS or set(part) != FIELDS[kind]:
            failures.append("unknown_part_or_fields")
            continue
        if kind in ("literal", "literal-data"):
            text = part["text"]
            if not isinstance(text, str):
                failures.append("literal_not_string")
                continue
            literal_bytes += len(text.encode("utf-8"))
            if "\0" in text:
                failures.append("literal_nul")
            if kind == "literal" and ("/" in text or "\\" in text):
                failures.append("literal_filesystem_path")
        elif kind in PATH_KINDS or kind == "cwd_relative":
            value = part["path"]
            if kind == "cwd_relative" and isinstance(value, str):
                value = value.removeprefix("./")
            failure = member_error(value, kind != "cwd_relative")
            if failure:
                failures.append(failure)
            if kind == "cwd_relative" and len(parts) != 1:
                failures.append("cwd_relative_partial_word")
        elif kind == "directory_separator":
            boundary = at + 1 == len(parts)
            if not boundary:
                following = parts[at + 1]
                boundary = isinstance(following, dict) and following.get("kind") == "literal" and following.get("text") in (":", ",")
            previous = parts[at - 1] if at else {}
            if not boundary or not at or not isinstance(previous, dict) or previous.get("kind") not in PATH_KINDS:
                failures.append("directory_separator_position")
            literal_bytes += 1
        elif kind == "text_slice":
            offset, length = part["offset_bytes"], part["length_bytes"]
            if type(offset) is not int or type(length) is not int or not 0 <= offset <= MAX_U64 or not 0 <= length <= MAX_U64:
                failures.append("text_slice_u64")
            elif offset + length > MAX_SOURCE_DATA_BYTES or length > MAX_WORD_BYTES:
                failures.append("text_slice_bounds")
        for key in ("name", "mount"):
            if key in part and not isinstance(part[key], str):
                failures.append("alias_not_string")
    if literal_bytes > MAX_WORD_BYTES:
        failures.append("word_literal_bytes")
    return failures


def inspect(recipe):
    groups = {name: {"words": 0, "parts": 0, "kinds": collections.Counter(), "issues": collections.Counter(), "examples": []} for name in ("direct_recipe", "encoded_data")}
    encoded_roots = 0
    skipped_encoded_depth = 0
    stack = [(recipe, "$", False, 0)]
    while stack:
        value, location, encoded, depth = stack.pop()
        if isinstance(value, dict):
            if "parts" in value:
                group = groups["encoded_data" if encoded else "direct_recipe"]
                group["words"] += 1
                if isinstance(value["parts"], list):
                    group["parts"] += len(value["parts"])
                    group["kinds"].update(part.get("kind", "<missing>") if isinstance(part, dict) else "<nonobject>" for part in value["parts"])
                for failure in word_issues(value):
                    group["issues"][failure] += 1
                    if len(group["examples"]) < 40:
                        group["examples"].append({"where": location, "issue": failure})
            stack.extend((child, location + "." + key, encoded, depth) for key, child in value.items())
        elif isinstance(value, list):
            stack.extend((child, location + "[" + str(at) + "]", encoded, depth) for at, child in enumerate(value))
        elif isinstance(value, str) and value.lstrip().startswith(("{", "[")):
            if depth >= 4:
                skipped_encoded_depth += 1
                continue
            try:
                decoded = json.loads(value)
            except (ValueError, RecursionError):
                continue
            if isinstance(decoded, (dict, list)):
                encoded_roots += 1
                stack.append((decoded, location + "<decoded-json>", True, depth + 1))
    return {"groups": groups, "decoded_static_json_roots": encoded_roots, "deeper_encoded_json_candidates_not_decoded": skipped_encoded_depth}


def main():
    inventory = json.loads(pathlib.Path(sys.argv[1]).read_text())
    cohort = pathlib.Path(inventory["cohort"]).resolve(strict=True)
    consumer = pathlib.Path(inventory["consumer_source"])
    if digest(consumer) != inventory["consumer_blake3"]:
        raise RuntimeError("frozen consumer identity changed")
    report = {"scope": "all direct RecipeWord dictionaries: lexical/shape checks only; encoded JSON data candidates reported separately", "manifest_blake3": inventory["manifest_blake3"], "consumer_source": str(consumer), "consumer_blake3": inventory["consumer_blake3"], "recipes": {}}
    for item in inventory["recipes"]:
        path = pathlib.Path(item["physical_source"])
        if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_RECIPE_BYTES:
            raise RuntimeError("unbounded/nonregular physical recipe")
        path.resolve(strict=True).relative_to(cohort)
        observed = digest(path)
        if observed != item["blake3"]:
            raise RuntimeError("current recipe differs from current graph inventory")
        recipe = json.loads(path.read_text())
        if recipe["stage_id"] != item["stage"]:
            raise RuntimeError("stage identity differs from inventory")
        result = inspect(recipe)
        report["recipes"][item["stage"]] = {"path": str(path), "bytes": path.stat().st_size, "blake3": observed, **result}
    report["direct_recipe_lexical_outcome"] = "reject" if any(item["groups"]["direct_recipe"]["issues"] for item in report["recipes"].values()) else "accept"
    report["limitations"] = ["No input/output/guard/runtime/workspace alias or kind binding decision", "No complete RecipeOperation/expansion/lineage validation", "Encoded JSON data is not necessarily a directly executed invocation; candidate issues require tracing its actual consumer", "Decoded JSON nesting above four layers is explicitly counted but not expanded", "No product/helper/controller/recipe execution, dynamic request observation, provider proof, reconciliation or launch verdict"]
    pathlib.Path(sys.argv[2]).write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
