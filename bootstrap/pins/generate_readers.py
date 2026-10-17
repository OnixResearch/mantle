#!/usr/bin/env python3
"""Render checked Nickel JSON readers from the authoritative bootstrap TOML pins."""
import argparse
import json
import resource
import subprocess
import tempfile
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parent
PIN_FIELDS = ("schema", "source", "package_url", "version", "release_date", "artifacts", "resolve", "recipes")
ARTIFACT_FIELDS = ("url_template", "url", "hash_kind", "hash")
RESOLVE_FIELDS = ("kind", "url", "version_field", "date_field")


def ordered_record(record):
    if set(record) != set(PIN_FIELDS):
        raise ValueError(f"pin fields must be exactly {PIN_FIELDS}; got {sorted(record)}")
    result = {name: record[name] for name in PIN_FIELDS}
    result["artifacts"] = {
        role: {name: artifact[name] for name in ARTIFACT_FIELDS}
        for role, artifact in sorted(record["artifacts"].items())
        if set(artifact) == set(ARTIFACT_FIELDS)
    }
    if len(result["artifacts"]) != len(record["artifacts"]):
        raise ValueError("invalid artifact fields")
    if set(record["resolve"]) != set(RESOLVE_FIELDS):
        raise ValueError("invalid resolve fields")
    result["resolve"] = {name: record["resolve"][name] for name in RESOLVE_FIELDS}
    return result

def fixed_output_fetches(value):
    if isinstance(value, dict):
        if isinstance(value.get("fixed_output"), dict) and isinstance(value.get("env"), dict):
            yield (value["env"].get("url"), value["fixed_output"])
        for child in value.values():
            yield from fixed_output_fetches(child)
    elif isinstance(value, list):
        for child in value:
            yield from fixed_output_fetches(child)


# r[impl mantle.bootstrap_source_pins.nickel_reads_only]
def check_recipe(pin, recipe, project_root):
    if Path(recipe).name != recipe or not recipe.endswith(".ncl"):
        raise ValueError(f"invalid declared recipe filename: {recipe}")
    path = project_root / "bootstrap" / recipe
    if not path.is_file():
        raise ValueError(f"missing declared Nickel recipe: {path}")
    def limit_output():
        resource.setrlimit(resource.RLIMIT_FSIZE, (8 * 1024 * 1024, 8 * 1024 * 1024))
        resource.setrlimit(resource.RLIMIT_CPU, (30, 30))
    with tempfile.TemporaryFile() as stderr, tempfile.NamedTemporaryFile() as output:
        try:
            result = subprocess.run(
                ["nickel", "export", "-I", "lib", "--format", "json", "--output", output.name, str(path)],
                cwd=project_root, stdout=subprocess.DEVNULL, stderr=stderr, timeout=45,
                preexec_fn=limit_output, check=False,
            )
        except subprocess.TimeoutExpired as exc:
            raise ValueError(f"{recipe}: Nickel export timed out") from exc
        if result.returncode:
            stderr.seek(0)
            raise ValueError(f"{recipe}: Nickel export failed ({result.returncode}): {stderr.read(4096).decode(errors='replace')}")
        if Path(output.name).stat().st_size > 8 * 1024 * 1024:
            raise ValueError(f"{recipe}: Nickel export exceeds 8 MiB")
        with open(output.name, encoding="utf-8") as reader:
            evaluated = json.load(reader)
    observed = tuple(fixed_output_fetches(evaluated))
    for role, artifact in pin["artifacts"].items():
        expected = "recursive" if artifact["hash_kind"] == "tree" else "flat"
        if not any(url == artifact["url"] and fixed.get("hash") == artifact["hash"]
                   and fixed.get("algo") == "sha256" and fixed.get("mode") == expected
                   for url, fixed in observed):
            raise ValueError(f"{recipe}: evaluated {role} fetch URL, sha256 hash, or flat/tree mode differs from pin")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail when a derived reader differs")
    parser.add_argument("--check-recipes", action="store_true", help="check reader bytes and bounded pure Nickel fetch semantics")
    args = parser.parse_args()
    sources = sorted(ROOT.glob("*.toml"))
    if not 1 <= len(sources) <= 256:
        raise ValueError("pin directory must have 1..=256 TOML records")
    for source in sources:
        pin = ordered_record(tomllib.loads(source.read_text(encoding="utf-8")))
        if source.stem != pin["source"]:
            raise ValueError(f"{source}: source differs from filename")
        output = ROOT / "generated" / (source.stem + ".json")
        expected = (json.dumps(pin, indent=2, ensure_ascii=False) + "\n").encode()
        if args.check or args.check_recipes:
            if not output.exists() or output.read_bytes() != expected:
                raise ValueError(f"stale derived pin reader: {output}")
        else:
            output.parent.mkdir(exist_ok=True)
            output.write_bytes(expected)
        print(output.relative_to(ROOT))
        if args.check_recipes:
            project_root = ROOT.parent.parent
            for recipe in pin["recipes"]:
                check_recipe(pin, recipe, project_root)
                print(f"evaluated {recipe}")


if __name__ == "__main__":
    main()
