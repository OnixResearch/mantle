#!/usr/bin/env python3
from __future__ import annotations

import argparse
import io
import os
import selectors
import shutil
import signal
import json
import re
import subprocess
import sys
import tempfile
import time
import tomllib
from contextlib import contextmanager
from collections import defaultdict
from dataclasses import dataclass
from functools import cache
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
FUNCTIONAL_CORE_SPEC_ROOT = REPO_ROOT / "openspec" / "specs" / "functional-core"
VALIDATION_ROOT = FUNCTIONAL_CORE_SPEC_ROOT / "validation"
ALLOWLIST_PATH = VALIDATION_ROOT / "deps-allowlist.txt"
INVENTORY_PATH = VALIDATION_ROOT / "adopted-core-inventory.toml"
OWNERSHIP_REVIEW_PATH = FUNCTIONAL_CORE_SPEC_ROOT / "evidence" / "ownership-review.md"
ACTIVE_CHANGES_ROOT = REPO_ROOT / "openspec" / "changes"
ARCHIVE_CHANGES_ROOT = ACTIVE_CHANGES_ROOT / "archive"
OWNERSHIP_CHANGE_NAMES = ("no-std-functional-core", "delta-functional-core")
EMPTY_TREE_OBJECT_HASH = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
TOP_LEVEL_WORKSPACE_RUST_DIRS = {"src", "tests", "examples", "benches"}
CRATE_WORKSPACE_RUST_DIRS = {"src", "tests", "examples", "benches"}
ADOPTED_CORE_INVENTORY_VERSION = 1
REQUIRED_OWNERSHIP_REVIEW_MARKERS = ("## Review verdict", "Reviewer:", "adapter-only", "unrelated")
ADOPTED_CORE_VERDICT_MARKER = (
    "shell/release business logic remains in `crunch-shell-core` and `crunch-release-core`, "
    "while delta planning/protocol business logic remains in `crunch-delta-core`"
)
ALLOWED_LEGACY_PREFIXES = ("use ", "pub use ", "type ", "pub type ")
COMMENT_PREFIXES = ("//", "///", "//!", "/*", "*", "*/")


@dataclass(frozen=True)
class AdoptedCore:
    package: str
    crate_dir: Path
    lib_path: Path
    manifest_path: Path
    required_exports: tuple[str, ...]
    legacy_std_files: tuple[Path, ...]
    required_std_adapter_files: tuple[Path, ...]

    @property
    def source_dir(self) -> Path:
        return self.crate_dir / "src"
TREE_PACKAGE_RE = re.compile(r"(?:^|[|` +\\-]+)([A-Za-z0-9_.-]+) v[0-9]")
TREE_FEATURE_RE = re.compile(r"(?:^|[|` +\\-]+)([A-Za-z0-9_.-]+) feature \"([^\"]+)\"")
IMPL_KEYWORD_RE = re.compile(r"(?m)^[ \t]*impl\b")
PUBLIC_FN_RE = re.compile(r"\bpub\s+fn\b")
PUBLIC_STRUCT_RE = re.compile(r"\bpub\s+struct\b")
PUBLIC_ENUM_RE = re.compile(r"\bpub\s+enum\b")
PUBLIC_TYPE_RE = re.compile(r"\bpub\s+type\b")
PUBLIC_TRAIT_RE = re.compile(r"\bpub\s+trait\b")
CFG_TEST_ATTR = "#[cfg(test)]"
WASM_TARGET = "wasm32-unknown-unknown"
CARGO_METADATA_FORMAT_VERSION = "1"
SCALAR_TYPES = {"bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"}
CONTAINER_TYPES = {"String", "Box", "Vec", "Option", "Result", "BTreeMap", "BTreeSet"}
BANNED_TYPE_PATTERNS = (
    ("reference", re.compile(r"&")),
    ("impl trait", re.compile(r"\bimpl\b")),
    ("trait object", re.compile(r"\bdyn\b")),
    ("Rc", re.compile(r"\bRc\b")),
    ("Arc", re.compile(r"\bArc\b")),
    ("Cow", re.compile(r"\bCow\b")),
    ("Path", re.compile(r"\bPath\b")),
    ("PathBuf", re.compile(r"\bPathBuf\b")),
    ("Command", re.compile(r"\bCommand\b")),
    ("std path", re.compile(r"\bstd::")),
    ("OsStr", re.compile(r"\bOsStr\b")),
    ("OsString", re.compile(r"\bOsString\b")),
    ("HashMap", re.compile(r"\bHashMap\b")),
    ("HashSet", re.compile(r"\bHashSet\b")),
)
BANNED_PURITY_PATTERNS = (
    ("std path", re.compile(r"\bstd::")),
    ("println", re.compile(r"\bprintln!")),
    ("eprintln", re.compile(r"\beprintln!")),
    ("dbg", re.compile(r"\bdbg!")),
    ("log", re.compile(r"\blog::")),
    ("tracing", re.compile(r"\btracing::")),
    ("rand", re.compile(r"\brand::")),
    ("getrandom", re.compile(r"\bgetrandom::")),
    ("fastrand", re.compile(r"\bfastrand::")),
    ("thread_rng", re.compile(r"\bthread_rng\b")),
    ("OnceLock", re.compile(r"\bOnceLock\b")),
    ("LazyLock", re.compile(r"\bLazyLock\b")),
    ("lazy_static", re.compile(r"\blazy_static!")),
    ("thread_local", re.compile(r"\bthread_local!")),
    ("static mut", re.compile(r"\bstatic\s+mut\b")),
)


def die(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


class CheckFailure(Exception):
    pass


class ValidationError(Exception):
    pass


def read_inventory_text() -> str:
    if not INVENTORY_PATH.exists():
        raise CheckFailure(f"missing adopted-core inventory: {INVENTORY_PATH.relative_to(REPO_ROOT)}")
    return INVENTORY_PATH.read_text()


def resolve_inventory_path(raw_value: object, field_name: str) -> Path:
    if not isinstance(raw_value, str) or not raw_value.strip():
        raise CheckFailure(f"inventory field {field_name} must be a non-empty string")
    resolved_path = REPO_ROOT / raw_value
    if not resolved_path.exists():
        raise CheckFailure(f"inventory path for {field_name} is missing: {raw_value}")
    return resolved_path


def resolve_inventory_paths(raw_value: object, field_name: str) -> tuple[Path, ...]:
    if not isinstance(raw_value, list):
        raise CheckFailure(f"inventory field {field_name} must be a list")
    resolved_paths: list[Path] = []
    for index_u32, path_value in enumerate(raw_value, 1):
        resolved_paths.append(resolve_inventory_path(path_value, f"{field_name}[{index_u32}]"))
    return tuple(resolved_paths)


def resolve_required_exports(raw_value: object, package_name: str) -> tuple[str, ...]:
    if not isinstance(raw_value, list) or not raw_value:
        raise CheckFailure(f"inventory package {package_name} must declare required_exports")
    exports: list[str] = []
    for index_u32, export_value in enumerate(raw_value, 1):
        if not isinstance(export_value, str) or not export_value.strip():
            raise CheckFailure(
                f"inventory package {package_name} required_exports[{index_u32}] must be a non-empty string"
            )
        exports.append(export_value)
    return tuple(exports)


def parse_adopted_core(raw_entry: object) -> AdoptedCore:
    if not isinstance(raw_entry, dict):
        raise CheckFailure("inventory core entry must be a table")
    raw_package = raw_entry.get("package")
    if not isinstance(raw_package, str) or not raw_package.strip():
        raise CheckFailure("inventory core entry missing package name")
    package_name = raw_package.strip()
    crate_dir = resolve_inventory_path(raw_entry.get("crate_dir"), f"{package_name}.crate_dir")
    lib_path = resolve_inventory_path(raw_entry.get("lib_path"), f"{package_name}.lib_path")
    manifest_path = resolve_inventory_path(raw_entry.get("manifest_path"), f"{package_name}.manifest_path")
    return AdoptedCore(
        package=package_name,
        crate_dir=crate_dir,
        lib_path=lib_path,
        manifest_path=manifest_path,
        required_exports=resolve_required_exports(raw_entry.get("required_exports"), package_name),
        legacy_std_files=resolve_inventory_paths(raw_entry.get("legacy_std_files", []), f"{package_name}.legacy_std_files"),
        required_std_adapter_files=resolve_inventory_paths(
            raw_entry.get("required_std_adapter_files", []), f"{package_name}.required_std_adapter_files"
        ),
    )


@cache
def adopted_cores() -> tuple[AdoptedCore, ...]:
    data = tomllib.loads(read_inventory_text())
    version = data.get("version")
    if version != ADOPTED_CORE_INVENTORY_VERSION:
        raise CheckFailure(
            f"unexpected adopted-core inventory version {version!r}; expected {ADOPTED_CORE_INVENTORY_VERSION}"
        )
    raw_entries = data.get("core")
    if not isinstance(raw_entries, list) or not raw_entries:
        raise CheckFailure(f"inventory has no [[core]] entries: {INVENTORY_PATH.relative_to(REPO_ROOT)}")
    parsed_cores: list[AdoptedCore] = []
    seen_packages: set[str] = set()
    for raw_entry in raw_entries:
        core = parse_adopted_core(raw_entry)
        if core.package in seen_packages:
            raise CheckFailure(f"duplicate inventory package entry: {core.package}")
        seen_packages.add(core.package)
        parsed_cores.append(core)
    return tuple(parsed_cores)


def core_crate_directories() -> set[Path]:
    return {core.crate_dir for core in adopted_cores()}


def core_packages() -> tuple[str, ...]:
    return tuple(core.package for core in adopted_cores())


def core_source_dirs() -> tuple[Path, ...]:
    return tuple(core.source_dir for core in adopted_cores())


def legacy_std_files() -> tuple[Path, ...]:
    return tuple(path for core in adopted_cores() for path in core.legacy_std_files)


def required_std_adapter_files() -> tuple[Path, ...]:
    return tuple(path for core in adopted_cores() for path in core.required_std_adapter_files)


def run_command(command: list[str]) -> str:
    result = subprocess.run(command, cwd=REPO_ROOT, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        cmd_text = " ".join(command)
        raise CheckFailure(f"command failed ({cmd_text}):\n{result.stderr}{result.stdout}")
    return result.stdout


def read_allowlist() -> set[str]:
    if not ALLOWLIST_PATH.exists():
        raise CheckFailure(f"missing allowlist: {ALLOWLIST_PATH.relative_to(REPO_ROOT)}")
    allowlist: set[str] = set()
    for raw_line in ALLOWLIST_PATH.read_text().splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        allowlist.add(line)
    if not allowlist:
        raise CheckFailure(f"allowlist is empty: {ALLOWLIST_PATH.relative_to(REPO_ROOT)}")
    return allowlist


def parse_tree(output: str) -> tuple[set[str], dict[str, set[str]]]:
    packages: set[str] = set()
    features: dict[str, set[str]] = defaultdict(set)
    for line in output.splitlines():
        feature_match = TREE_FEATURE_RE.search(line)
        if feature_match is not None:
            features[feature_match.group(1)].add(feature_match.group(2))
        package_match = TREE_PACKAGE_RE.search(line)
        if package_match is not None:
            packages.add(package_match.group(1))
    return packages, features


def run_git_command(args: list[str]) -> str:
    return run_command(["git", *args])


def git_revision_parent(revision: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", f"{revision}^"],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return result.stdout.strip()
    return EMPTY_TREE_OBJECT_HASH


def is_std_workspace_rust_source(relative_path: str) -> bool:
    path = Path(relative_path)
    if path.suffix != ".rs":
        return False
    if path.parts and path.parts[0] == "vendor":
        return False
    if path.parts and path.parts[0] in TOP_LEVEL_WORKSPACE_RUST_DIRS:
        return True
    if len(path.parts) < 4:
        return False
    if path.parts[0] != "crates":
        return False
    crate_dir = REPO_ROOT / path.parts[0] / path.parts[1]
    if crate_dir in core_crate_directories():
        return False
    return path.parts[2] in CRATE_WORKSPACE_RUST_DIRS


def resolve_change_history_paths(change_name: str) -> list[Path]:
    active_change_root = ACTIVE_CHANGES_ROOT / change_name
    archived_matches = sorted(ARCHIVE_CHANGES_ROOT.glob(f"*-{change_name}"))
    if len(archived_matches) > 1:
        matches = ", ".join(str(path.relative_to(REPO_ROOT)) for path in archived_matches)
        raise CheckFailure(f"multiple archived change roots found for {change_name}: {matches}")
    resolved_paths: list[Path] = []
    if active_change_root.exists():
        resolved_paths.append(active_change_root)
    if len(archived_matches) == 1:
        resolved_paths.append(archived_matches[0])
    if resolved_paths:
        return resolved_paths
    raise CheckFailure(f"missing active or archived change root for {change_name}")


def ownership_history_paths() -> list[Path]:
    resolved_paths: list[Path] = []
    for change_name in OWNERSHIP_CHANGE_NAMES:
        resolved_paths.extend(resolve_change_history_paths(change_name))
    unique_paths = sorted(set(resolved_paths), key=lambda path: str(path.relative_to(REPO_ROOT)))
    return unique_paths


def touched_std_workspace_source_files() -> list[str]:
    change_paths = ownership_history_paths()
    change_pathspecs = [str(path.relative_to(REPO_ROOT)) for path in change_paths]
    history = [
        line.strip()
        for line in run_git_command(["log", "--format=%H", "--reverse", "--", *change_pathspecs]).splitlines()
        if line.strip()
    ]
    if not history:
        joined_pathspecs = ", ".join(change_pathspecs)
        raise CheckFailure(f"no git history found for change paths: {joined_pathspecs}")
    diff_base = git_revision_parent(history[0])
    touched_paths = [
        line.strip()
        for line in run_git_command(["diff", "--name-only", "--diff-filter=ACMR", f"{diff_base}..HEAD", "--", "*.rs"]).splitlines()
        if line.strip()
    ]
    derived = []
    legacy_relative_paths = {str(path.relative_to(REPO_ROOT)) for path in legacy_std_files()}
    for relative_path in touched_paths:
        if relative_path in legacy_relative_paths:
            continue
        if not is_std_workspace_rust_source(relative_path):
            continue
        derived.append(relative_path)
    return sorted(set(derived))


def ownership_review_has_classification(artifact_text: str, relative_path: str) -> bool:
    pattern = re.compile(rf"^- `{re.escape(relative_path)}` → `(adapter-only|unrelated)`$", re.MULTILINE)
    return pattern.search(artifact_text) is not None


def read_workspace_feature_definitions() -> dict[str, dict[str, set[str]]]:
    metadata = json.loads(
        run_command(
            [
                "cargo",
                "metadata",
                "--format-version",
                CARGO_METADATA_FORMAT_VERSION,
                "--filter-platform",
                WASM_TARGET,
            ]
        )
    )
    package_features: dict[str, dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
    for package in metadata.get("packages", []):
        package_name = package.get("name")
        if not package_name:
            continue
        feature_map = package.get("features", {})
        for feature_name, members in feature_map.items():
            package_features[package_name][feature_name].update(str(member) for member in members)
    return {name: dict(features) for name, features in package_features.items()}


def feature_definition_requires_std(
    package_name: str,
    feature_name: str,
    package_definitions: dict[str, dict[str, set[str]]],
    stack: tuple[str, ...] = (),
) -> bool:
    if feature_name == "std":
        return True
    if feature_name in stack:
        return False
    feature_map = package_definitions.get(package_name, {})
    members = feature_map.get(feature_name, set())
    for member in members:
        normalized = member.strip()
        if not normalized:
            continue
        if normalized == "std":
            return True
        if normalized.endswith("/std") or normalized.endswith("?/std"):
            return True
        if normalized.startswith("dep:"):
            continue
        if "/" in normalized:
            dependency_feature = normalized.split("/", 1)[1]
            if dependency_feature == "std":
                return True
            continue
        if feature_definition_requires_std(package_name, normalized, package_definitions, stack + (feature_name,)):
            return True
    return False


def remove_cfg_test_items(text: str) -> str:
    pieces: list[str] = []
    cursor = 0
    while True:
        marker_index = text.find(CFG_TEST_ATTR, cursor)
        if marker_index == -1:
            pieces.append(text[cursor:])
            break
        pieces.append(text[cursor:marker_index])
        cursor = skip_cfg_test_item(text, marker_index)
    return "".join(pieces)


def skip_cfg_test_item(text: str, marker_index: int) -> int:
    cursor = marker_index + len(CFG_TEST_ATTR)
    while cursor < len(text) and text[cursor].isspace():
        cursor += 1
    while text.startswith("#[", cursor):
        attr_end = text.find("]", cursor)
        if attr_end == -1:
            return len(text)
        cursor = attr_end + 1
        while cursor < len(text) and text[cursor].isspace():
            cursor += 1
    if text.startswith("extern crate ", cursor):
        end = text.find(";", cursor)
        return len(text) if end == -1 else end + 1
    block_start = text.find("{", cursor)
    statement_end = text.find(";", cursor)
    if block_start != -1 and (statement_end == -1 or block_start < statement_end):
        return find_matching_delimiter(text, block_start, "{", "}") + 1
    if statement_end != -1:
        return statement_end + 1
    return len(text)


def find_matching_delimiter(text: str, start: int, open_char: str, close_char: str) -> int:
    depth = 0
    index = start
    in_string = False
    in_char = False
    line_comment = False
    block_comment = False
    raw_hash_count = -1

    while index < len(text):
        char = text[index]
        next_char = text[index + 1] if index + 1 < len(text) else ""

        if line_comment:
            if char == "\n":
                line_comment = False
            index += 1
            continue

        if block_comment:
            if char == "*" and next_char == "/":
                block_comment = False
                index += 2
                continue
            index += 1
            continue

        if in_string:
            if raw_hash_count >= 0:
                if char == '"' and text[index + 1 : index + 1 + raw_hash_count] == ("#" * raw_hash_count):
                    in_string = False
                    index += raw_hash_count + 1
                    raw_hash_count = -1
                    continue
                index += 1
                continue
            if char == "\\":
                index += 2
                continue
            if char == '"':
                in_string = False
            index += 1
            continue

        if in_char:
            if char == "\\":
                index += 2
                continue
            if char == "'":
                in_char = False
            index += 1
            continue

        if char == "/" and next_char == "/":
            line_comment = True
            index += 2
            continue
        if char == "/" and next_char == "*":
            block_comment = True
            index += 2
            continue
        if char == 'r':
            probe = index + 1
            hash_count = 0
            while probe < len(text) and text[probe] == '#':
                hash_count += 1
                probe += 1
            if probe < len(text) and text[probe] == '"':
                in_string = True
                raw_hash_count = hash_count
                index = probe + 1
                continue
        if char == '"':
            in_string = True
            raw_hash_count = -1
            index += 1
            continue
        if char == "'":
            if next_char and (
                not (next_char.isalpha() or next_char == "_")
                or text[index + 2 : index + 3] == "'"
            ):
                in_char = True
                index += 1
                continue
        if char == open_char:
            depth += 1
        elif char == close_char:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    raise ValidationError(f"unmatched delimiter in source starting at offset {start}")


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def split_top_level(text: str, separator: str) -> list[str]:
    parts: list[str] = []
    start = 0
    paren_depth = 0
    bracket_depth = 0
    brace_depth = 0
    angle_depth = 0
    for index, char in enumerate(text):
        if char == "(":
            paren_depth += 1
        elif char == ")":
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            bracket_depth -= 1
        elif char == "{":
            brace_depth += 1
        elif char == "}":
            brace_depth -= 1
        elif char == "<":
            angle_depth += 1
        elif char == ">":
            if angle_depth > 0:
                angle_depth -= 1
        elif char == separator and paren_depth == 0 and bracket_depth == 0 and brace_depth == 0 and angle_depth == 0:
            parts.append(text[start:index])
            start = index + 1
    parts.append(text[start:])
    return parts


def collect_signature(text: str, start: int) -> str:
    cursor = start
    paren_depth = 0
    bracket_depth = 0
    brace_depth = 0
    angle_depth = 0
    while cursor < len(text):
        char = text[cursor]
        if char == "(":
            paren_depth += 1
        elif char == ")":
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            bracket_depth -= 1
        elif char == "<":
            angle_depth += 1
        elif char == ">":
            if angle_depth > 0:
                angle_depth -= 1
        elif char == "{" and paren_depth == 0 and bracket_depth == 0 and brace_depth == 0 and angle_depth == 0:
            return text[start:cursor].strip()
        elif char == ";" and paren_depth == 0 and bracket_depth == 0 and brace_depth == 0 and angle_depth == 0:
            return text[start:cursor].strip()
        cursor += 1
    raise ValidationError("unterminated public function signature")


def collect_braced_item(text: str, start: int) -> str:
    brace_index = text.find("{", start)
    if brace_index == -1:
        raise ValidationError("expected braced public item")
    end_index = find_matching_delimiter(text, brace_index, "{", "}")
    return text[start : end_index + 1]


def collect_statement_item(text: str, start: int) -> str:
    cursor = start
    paren_depth = 0
    bracket_depth = 0
    brace_depth = 0
    angle_depth = 0
    while cursor < len(text):
        char = text[cursor]
        if char == "(":
            paren_depth += 1
        elif char == ")":
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            bracket_depth -= 1
        elif char == "<":
            angle_depth += 1
        elif char == ">":
            if angle_depth > 0:
                angle_depth -= 1
        elif char == ";" and paren_depth == 0 and bracket_depth == 0 and brace_depth == 0 and angle_depth == 0:
            return text[start : cursor + 1]
        cursor += 1
    raise ValidationError("expected semicolon-terminated public item")


def normalize_type_expression(expr: str) -> str:
    return " ".join(expr.strip().split())


def split_generic(expr: str) -> tuple[str, str | None]:
    angle_index = expr.find("<")
    if angle_index == -1:
        return expr.strip(), None
    end_index = find_matching_delimiter(expr, angle_index, "<", ">")
    root = expr[:angle_index].strip()
    inner = expr[angle_index + 1 : end_index].strip()
    suffix = expr[end_index + 1 :].strip()
    if suffix:
        raise CheckFailure(f"unsupported type suffix in boundary expression: {expr}")
    return root, inner


def validate_boundary_type(expr: str, where: str) -> None:
    normalized = normalize_type_expression(expr)
    if not normalized:
        raise CheckFailure(f"empty boundary type in {where}")
    for label, pattern in BANNED_TYPE_PATTERNS:
        if pattern.search(normalized):
            raise CheckFailure(f"{where} uses banned boundary type form ({label}): {normalized}")
    if normalized.startswith("["):
        if not normalized.endswith("]"):
            raise CheckFailure(f"{where} has malformed array or slice type: {normalized}")
        inner = normalized[1:-1]
        if ";" not in inner:
            raise CheckFailure(f"{where} exposes a slice type, which is forbidden: {normalized}")
        parts = split_top_level(inner, ";")
        if len(parts) != 2:
            raise CheckFailure(f"{where} has malformed owned-bytes array: {normalized}")
        element_type = normalize_type_expression(parts[0])
        if element_type != "u8":
            raise CheckFailure(f"{where} only allows owned byte arrays, got: {normalized}")
        if not parts[1].strip():
            raise CheckFailure(f"{where} has empty array length: {normalized}")
        return
    if normalized.startswith("("):
        raise CheckFailure(f"{where} exposes a tuple type, which is forbidden: {normalized}")
    root, generic_args = split_generic(normalized)
    root = root.strip()
    if root in SCALAR_TYPES or root in {"String", "Self"}:
        if generic_args is not None:
            raise CheckFailure(f"{where} has unexpected generic arguments on scalar or String: {normalized}")
        return
    if root in CONTAINER_TYPES:
        if generic_args is None:
            raise CheckFailure(f"{where} must specify generic arguments for container type: {normalized}")
        arguments = [normalize_type_expression(part) for part in split_top_level(generic_args, ",") if part.strip()]
        if not arguments:
            raise CheckFailure(f"{where} has empty generic argument list: {normalized}")
        for argument in arguments:
            validate_boundary_type(argument, where)
        return
    if "::" in root:
        root_parts = root.split("::")
        first_segment = root_parts[0]
        last_segment = root_parts[-1]
        if first_segment in {"crate", "self", "super"}:
            if generic_args is not None:
                for argument in split_top_level(generic_args, ","):
                    if argument.strip():
                        validate_boundary_type(argument, where)
            return
        if first_segment in {"alloc", "core"} and last_segment in CONTAINER_TYPES | {"String"} | SCALAR_TYPES:
            if generic_args is not None:
                for argument in split_top_level(generic_args, ","):
                    if argument.strip():
                        validate_boundary_type(argument, where)
            return
        raise CheckFailure(f"{where} exposes a non-local multi-segment type path: {normalized}")
    if generic_args is not None:
        for argument in split_top_level(generic_args, ","):
            if argument.strip():
                validate_boundary_type(argument, where)


def validate_function_signature(signature: str, where: str) -> None:
    paren_start = signature.find("(")
    if paren_start == -1:
        raise CheckFailure(f"malformed public function signature in {where}: {signature}")
    paren_end = find_matching_delimiter(signature, paren_start, "(", ")")
    params = signature[paren_start + 1 : paren_end]
    for param in split_top_level(params, ","):
        normalized = normalize_type_expression(param)
        if not normalized:
            continue
        if normalized in {"self", "mut self"}:
            continue
        if ":" not in normalized:
            raise CheckFailure(f"public function parameter lacks explicit type in {where}: {normalized}")
        _, type_expr = normalized.split(":", 1)
        validate_boundary_type(type_expr, f"{where} parameter")
    arrow_index = signature.find("->", paren_end)
    if arrow_index != -1:
        return_type = signature[arrow_index + 2 :].strip()
        validate_boundary_type(return_type, f"{where} return type")


def validate_struct_item(item: str, where: str) -> None:
    if "{" not in item:
        paren_start = item.find("(")
        if paren_start == -1:
            return
        paren_end = find_matching_delimiter(item, paren_start, "(", ")")
        for field in split_top_level(item[paren_start + 1 : paren_end], ","):
            normalized = normalize_type_expression(field)
            if not normalized:
                continue
            if normalized.startswith("pub "):
                normalized = normalized[4:].strip()
            validate_boundary_type(normalized, f"{where} tuple field")
        return
    body = item[item.find("{") + 1 : item.rfind("}")]
    for field in split_top_level(body, ","):
        normalized = normalize_type_expression(field)
        if not normalized or not normalized.startswith("pub "):
            continue
        if ":" not in normalized:
            raise CheckFailure(f"malformed public struct field in {where}: {normalized}")
        _, type_expr = normalized.split(":", 1)
        validate_boundary_type(type_expr, f"{where} field")


def validate_enum_item(item: str, where: str) -> None:
    body = item[item.find("{") + 1 : item.rfind("}")]
    for variant in split_top_level(body, ","):
        normalized = normalize_type_expression(variant)
        if not normalized:
            continue
        if "(" in normalized:
            paren_start = normalized.find("(")
            paren_end = find_matching_delimiter(normalized, paren_start, "(", ")")
            for payload in split_top_level(normalized[paren_start + 1 : paren_end], ","):
                if payload.strip():
                    validate_boundary_type(payload, f"{where} variant payload")
            continue
        if "{" in normalized:
            brace_start = normalized.find("{")
            brace_end = find_matching_delimiter(normalized, brace_start, "{", "}")
            fields = normalized[brace_start + 1 : brace_end]
            for field in split_top_level(fields, ","):
                field_text = normalize_type_expression(field)
                if not field_text:
                    continue
                if ":" not in field_text:
                    raise CheckFailure(f"malformed enum field in {where}: {field_text}")
                _, type_expr = field_text.split(":", 1)
                validate_boundary_type(type_expr, f"{where} variant field")


def production_text(path: Path) -> str:
    return remove_cfg_test_items(path.read_text())


def find_top_level_keyword_offset(text: str, keyword: str) -> int:
    paren_depth = 0
    bracket_depth = 0
    brace_depth = 0
    angle_depth = 0
    for index, char in enumerate(text):
        if char == "(":
            paren_depth += 1
        elif char == ")":
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            bracket_depth -= 1
        elif char == "{":
            brace_depth += 1
        elif char == "}":
            brace_depth -= 1
        elif char == "<":
            angle_depth += 1
        elif char == ">":
            if angle_depth > 0:
                angle_depth -= 1
        if paren_depth != 0 or bracket_depth != 0 or brace_depth != 0 or angle_depth != 0:
            continue
        if not text.startswith(keyword, index):
            continue
        before_ok = index == 0 or not (text[index - 1].isalnum() or text[index - 1] == "_")
        after_index = index + len(keyword)
        after_ok = after_index >= len(text) or not (text[after_index].isalnum() or text[after_index] == "_")
        if before_ok and after_ok:
            return index
    return -1


def find_item_body_start(text: str, start: int) -> int:
    cursor = start
    paren_depth = 0
    bracket_depth = 0
    brace_depth = 0
    angle_depth = 0
    while cursor < len(text):
        char = text[cursor]
        if char == "(":
            paren_depth += 1
        elif char == ")":
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            bracket_depth -= 1
        elif char == "<":
            angle_depth += 1
        elif char == ">":
            if angle_depth > 0:
                angle_depth -= 1
        elif char == "{" and paren_depth == 0 and bracket_depth == 0 and brace_depth == 0 and angle_depth == 0:
            return cursor
        cursor += 1
    raise ValidationError("unterminated impl item")


def trait_impl_ranges(text: str) -> list[tuple[int, int]]:
    ranges: list[tuple[int, int]] = []
    cursor = 0
    while True:
        match = IMPL_KEYWORD_RE.search(text, cursor)
        if match is None:
            return ranges
        body_start = find_item_body_start(text, match.start())
        header = text[match.start() : body_start]
        where_index = find_top_level_keyword_offset(header, "where")
        header_prefix = header if where_index == -1 else header[:where_index]
        if find_top_level_keyword_offset(header_prefix, "for") != -1:
            body_end = find_matching_delimiter(text, body_start, "{", "}")
            ranges.append((match.start(), body_end + 1))
            cursor = body_end + 1
            continue
        cursor = body_start + 1


def mask_ranges(text: str, ranges: list[tuple[int, int]]) -> str:
    if not ranges:
        return text
    characters = list(text)
    for start, end in ranges:
        for index in range(start, end):
            if characters[index] != "\n":
                characters[index] = " "
    return "".join(characters)


def production_api_text(path: Path) -> str:
    text = production_text(path)
    return mask_ranges(text, trait_impl_ranges(text))


def command_deps() -> None:
    allowlist = read_allowlist()
    feature_definitions = read_workspace_feature_definitions()
    closure_packages: set[str] = set()
    package_features: dict[str, set[str]] = defaultdict(set)
    for package in core_packages():
        output = run_command(
            [
                "cargo",
                "tree",
                "-e",
                "normal,features,no-proc-macro",
                "-p",
                package,
                "--target",
                WASM_TARGET,
                "--charset",
                "ascii",
            ]
        )
        packages, features = parse_tree(output)
        closure_packages.update(packages)
        for name, feature_values in features.items():
            package_features[name].update(feature_values)
    unexpected = sorted(closure_packages - allowlist)
    stale = sorted(allowlist - closure_packages)
    failures: list[str] = []
    if unexpected:
        failures.append(f"unexpected no-std core dependencies: {', '.join(unexpected)}")
    if stale:
        failures.append(f"stale allowlist entries: {', '.join(stale)}")
    for package_name in sorted(closure_packages):
        enabled_features = package_features.get(package_name, set())
        if "std" in enabled_features:
            failures.append(f"std feature enabled in no-std closure: {package_name}")
        if "default" in enabled_features and feature_definition_requires_std(
            package_name, "default", feature_definitions
        ):
            failures.append(f"default feature set requires std in no-std closure: {package_name}")
    if failures:
        raise CheckFailure("\n".join(failures))
    print("dependency allowlist OK:", ", ".join(sorted(closure_packages)))


def command_purity() -> None:
    failures: list[str] = []
    for source_dir in core_source_dirs():
        for path in sorted(source_dir.glob("*.rs")):
            text = production_text(path)
            for label, pattern in BANNED_PURITY_PATTERNS:
                for match in pattern.finditer(text):
                    failures.append(
                        f"{path.relative_to(REPO_ROOT)}:{line_number(text, match.start())}: banned purity pattern ({label})"
                    )
    if failures:
        raise CheckFailure("\n".join(failures))
    print("purity check OK")


def command_scope() -> None:
    failures: list[str] = []
    for core in adopted_cores():
        text = core.lib_path.read_text()
        if "#![no_std]" not in text:
            failures.append(f"{core.lib_path.relative_to(REPO_ROOT)} missing #![no_std]")
        if "extern crate alloc;" not in text:
            failures.append(f"{core.lib_path.relative_to(REPO_ROOT)} missing extern crate alloc;")
        for required_export in core.required_exports:
            if required_export not in text:
                failures.append(f"{core.lib_path.relative_to(REPO_ROOT)} missing required export: {required_export}")
        cargo_manifest = tomllib.loads(core.manifest_path.read_text())
        features = cargo_manifest.get("features", {})
        if "std" in features:
            failures.append(f"{core.manifest_path.relative_to(REPO_ROOT)} defines forbidden std feature")
        default_features = features.get("default", [])
        if isinstance(default_features, list) and "std" in default_features:
            failures.append(f"{core.manifest_path.relative_to(REPO_ROOT)} default features include std")
    if failures:
        raise CheckFailure("\n".join(failures))
    print("scope check OK")


def command_api_shape() -> None:
    failures: list[str] = []
    for source_dir in core_source_dirs():
        for path in sorted(source_dir.glob("*.rs")):
            text = production_api_text(path)
            for match in PUBLIC_TRAIT_RE.finditer(text):
                failures.append(f"{path.relative_to(REPO_ROOT)}:{line_number(text, match.start())}: public trait is forbidden")
            try:
                for match in PUBLIC_FN_RE.finditer(text):
                    signature = collect_signature(text, match.start())
                    validate_function_signature(signature, f"{path.relative_to(REPO_ROOT)}:{line_number(text, match.start())}")
                for match in PUBLIC_STRUCT_RE.finditer(text):
                    brace_index = text.find("{", match.start())
                    semicolon_index = text.find(";", match.start())
                    if semicolon_index != -1 and (brace_index == -1 or semicolon_index < brace_index):
                        item = collect_statement_item(text, match.start())
                    else:
                        item = collect_braced_item(text, match.start())
                    validate_struct_item(item, f"{path.relative_to(REPO_ROOT)}:{line_number(text, match.start())}")
                for match in PUBLIC_ENUM_RE.finditer(text):
                    item = collect_braced_item(text, match.start())
                    validate_enum_item(item, f"{path.relative_to(REPO_ROOT)}:{line_number(text, match.start())}")
                for match in PUBLIC_TYPE_RE.finditer(text):
                    item = collect_statement_item(text, match.start())
                    alias_body = item.split("=", 1)[1].rsplit(";", 1)[0]
                    validate_boundary_type(alias_body, f"{path.relative_to(REPO_ROOT)}:{line_number(text, match.start())} type alias")
            except (CheckFailure, ValidationError) as error:
                failures.append(str(error))
    if failures:
        raise CheckFailure("\n".join(failures))
    print("API shape check OK")


def command_ownership() -> None:
    failures: list[str] = []
    derived_touched_std_paths = touched_std_workspace_source_files()
    required_adapter_review_paths = sorted(
        {str(path.relative_to(REPO_ROOT)) for path in required_std_adapter_files()}
    )
    required_classified_paths = sorted(set(derived_touched_std_paths) | set(required_adapter_review_paths))
    for path in legacy_std_files():
        for index, raw_line in enumerate(path.read_text().splitlines(), 1):
            stripped = raw_line.strip()
            if not stripped:
                continue
            if stripped.startswith(COMMENT_PREFIXES):
                continue
            if stripped.startswith("#"):
                continue
            if stripped.startswith(ALLOWED_LEGACY_PREFIXES):
                continue
            failures.append(f"{path.relative_to(REPO_ROOT)}:{index}: legacy file must stay adapter-only: {stripped}")
    if not OWNERSHIP_REVIEW_PATH.exists():
        failures.append(f"missing ownership review artifact: {OWNERSHIP_REVIEW_PATH.relative_to(REPO_ROOT)}")
    else:
        artifact_text = OWNERSHIP_REVIEW_PATH.read_text()
        for required_text in REQUIRED_OWNERSHIP_REVIEW_MARKERS:
            if required_text not in artifact_text:
                failures.append(
                    f"{OWNERSHIP_REVIEW_PATH.relative_to(REPO_ROOT)} missing required review marker: {required_text}"
                )
        if ADOPTED_CORE_VERDICT_MARKER not in artifact_text:
            failures.append(
                f"{OWNERSHIP_REVIEW_PATH.relative_to(REPO_ROOT)} missing adopted-core review verdict: {ADOPTED_CORE_VERDICT_MARKER}"
            )
        for legacy_path in legacy_std_files():
            relative_text = str(legacy_path.relative_to(REPO_ROOT))
            if relative_text not in artifact_text:
                failures.append(
                    f"{OWNERSHIP_REVIEW_PATH.relative_to(REPO_ROOT)} missing legacy path review entry: {relative_text}"
                )
        for reviewed_path in required_classified_paths:
            if not ownership_review_has_classification(artifact_text, reviewed_path):
                failures.append(
                    f"{OWNERSHIP_REVIEW_PATH.relative_to(REPO_ROOT)} missing touched std classification entry: {reviewed_path}"
                )
    if failures:
        raise CheckFailure("\n".join(failures))
    print(
        "ownership check OK:",
        ", ".join(required_classified_paths) if required_classified_paths else "no touched std files outside legacy paths",
    )

# Compiler views deliberately share the Rust guard's authority inventory. No
# second Rust-source parser is used: rustc resolves imports and expands macros.
ARCHITECTURE_CAPTURE_BYTES_MAX = 256 * 1024 * 1024
# Pretty-printed HIR includes indentation and expanded derives that are not
# retained. Bound its drained stream separately from compact DefId evidence.
ARCHITECTURE_COMPILER_OUTPUT_BYTES_MAX = 4 * 1024 * 1024 * 1024
ARCHITECTURE_DIAGNOSTIC_BYTES_MAX = 8 * 1024 * 1024
ARCHITECTURE_FAILURE_COUNT_MAX = 1024
ARCHITECTURE_COMMAND_SECONDS_MAX = 600
ARCHITECTURE_CAMPAIGN_SECONDS_MAX = 3600
RESOLVED_DEF_RE = re.compile(r"DefId\(\d+:\d+\s+~\s+([A-Za-z0-9_]+)\[[^\]]+\](?:::([^\n)]*))?\)")
# thir-flat prints every indexed THIR node rather than recursively reprinting
# subtrees with depth-proportional indentation. Both modes iterate all body
# owners and use the same thir_body query (rustc c756124775121dea0e640652c5ee3c89e3dd0eb4,
# compiler/rustc_driver_impl/src/pretty.rs and rustc_mir_build/src/thir/print.rs).
ARCHITECTURE_COMPILER_VIEWS = ("hir-tree", "thir-flat")


class ResolvedDefCapture:
    """Drain every compiler byte; retain every DefId and its original line."""

    def __init__(self, destination, label: str):
        self.destination = destination
        self.label = label
        self.pending = bytearray()
        self.raw_bytes = 0
        self.retained_bytes = 0
        self.line_count = 0
        self.definition_count = 0

    def retain(self, data: bytes) -> None:
        self.retained_bytes += len(data)
        if self.retained_bytes > ARCHITECTURE_CAPTURE_BYTES_MAX:
            raise CheckFailure(f"{self.label}: compact compiler evidence exceeds capture bound")
        self.destination.write(data)

    def line(self, data: bytes) -> None:
        self.line_count += 1
        text = data.decode("utf-8")
        matches = list(RESOLVED_DEF_RE.finditer(text))
        if text.count("DefId(") != len(matches) or text.strip() == "error":
            raise CheckFailure(f"{self.label}:{self.line_count}: malformed/unsupported compiler DefId output")
        for match in matches:
            self.definition_count += 1
            self.retain(f"{self.line_count}\t{match.group(0)}\n".encode())

    def write(self, data: bytes) -> None:
        self.raw_bytes += len(data)
        if self.raw_bytes > ARCHITECTURE_COMPILER_OUTPUT_BYTES_MAX:
            raise CheckFailure(f"{self.label}: compiler output exceeds capture bound")
        self.pending.extend(data)
        start = 0
        while (end := self.pending.find(b"\n", start)) >= 0:
            if end - start > ARCHITECTURE_DIAGNOSTIC_BYTES_MAX:
                raise CheckFailure(f"{self.label}: compiler line exceeds diagnostic byte bound")
            self.line(bytes(self.pending[start:end]))
            start = end + 1
        del self.pending[:start]
        if len(self.pending) > ARCHITECTURE_DIAGNOSTIC_BYTES_MAX:
            raise CheckFailure(f"{self.label}: compiler line exceeds diagnostic byte bound")

    def finish(self) -> None:
        if self.pending:
            self.line(bytes(self.pending))
            self.pending.clear()
        if self.definition_count == 0:
            raise CheckFailure(f"{self.label}: compiler view has no resolved DefId records; unsupported/empty compiler output")
        self.retain(f"# complete\t{self.raw_bytes}\t{self.line_count}\t{self.definition_count}\n".encode())



@contextmanager
def architecture_scratch(prefix: str):
    scratch = Path(tempfile.mkdtemp(prefix=prefix))
    try:
        yield scratch
    except BaseException:
        print(f"architecture compiler evidence retained: {scratch}", file=sys.stderr)
        raise
    else:
        shutil.rmtree(scratch)


def architecture_capture(command: list[str], cwd: Path, scratch: Path, label: str,
                         campaign_deadline: float, environment: dict[str, str] | None = None,
                         resolved_definitions: bool = False) -> Path:
    output_path = scratch / f"{label}.stdout"
    error_path = scratch / f"{label}.stderr"
    deadline = min(campaign_deadline, time.monotonic() + ARCHITECTURE_COMMAND_SECONDS_MAX)
    if deadline <= time.monotonic():
        raise CheckFailure("architecture campaign deadline exceeded")
    process = subprocess.Popen(command, cwd=cwd, env=environment, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, start_new_session=True)
    try:
        with output_path.open("wb") as output, error_path.open("wb") as error, selectors.DefaultSelector() as selector:
            assert process.stdout is not None and process.stderr is not None
            definitions = ResolvedDefCapture(output, label) if resolved_definitions else None
            stdout_bound = (ARCHITECTURE_COMPILER_OUTPUT_BYTES_MAX if resolved_definitions
                            else ARCHITECTURE_CAPTURE_BYTES_MAX)
            selector.register(process.stdout, selectors.EVENT_READ,
                              (definitions if definitions is not None else output, stdout_bound))
            selector.register(process.stderr, selectors.EVENT_READ, (error, ARCHITECTURE_DIAGNOSTIC_BYTES_MAX))
            counts = {process.stdout: 0, process.stderr: 0}
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise CheckFailure(f"architecture compiler deadline exceeded: {' '.join(command)}")
                for key, _ in selector.select(min(remaining, 1.0)):
                    data = os.read(key.fileobj.fileno(), 65536)
                    if not data:
                        selector.unregister(key.fileobj)
                        continue
                    destination, bound = key.data
                    counts[key.fileobj] += len(data)
                    if counts[key.fileobj] > bound:
                        raise CheckFailure(f"architecture compiler capture exceeds {bound} bytes: {' '.join(command)}")
                    destination.write(data)
            status = process.wait(timeout=max(0.001, deadline - time.monotonic()))
        if time.monotonic() > deadline:
            raise CheckFailure(f"architecture compiler deadline exceeded: {' '.join(command)}")
        if status != 0:
            diagnostic = error_path.read_text(errors="replace")
            raise CheckFailure(f"architecture compiler failed ({status}): {' '.join(command)}\n{diagnostic}")
        if definitions is not None:
            with output_path.open("ab") as output:
                definitions.destination = output
                definitions.finish()
        return output_path
    except BaseException:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
        raise
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        if process.stdout is not None:
            process.stdout.close()
        if process.stderr is not None:
            process.stderr.close()


def authority_path_matches(path: str, forbidden: str) -> bool:
    forbidden = forbidden.removesuffix("::")
    if forbidden.endswith("_"):
        return path.startswith(forbidden)
    return path == forbidden or path.startswith(forbidden + "::")


def resolved_authority_failures(path: Path, owner: str, authorities: list[str]) -> list[str]:
    failures: set[str] = set()
    definition_count = 0
    previous_line = 0
    complete = False
    retained_bytes = 0
    with path.open("rb") as source:
        for data in iter(lambda: source.readline(ARCHITECTURE_DIAGNOSTIC_BYTES_MAX + 1), b""):
            retained_bytes += len(data)
            if retained_bytes > ARCHITECTURE_CAPTURE_BYTES_MAX or len(data) > ARCHITECTURE_DIAGNOSTIC_BYTES_MAX or complete:
                raise CheckFailure(f"{owner}: malformed/over-bound compact compiler evidence: {path.name}")
            line = data.decode("utf-8").rstrip("\n")
            if line.startswith("# complete\t"):
                _, raw_bytes, raw_lines, records = line.split("\t")
                if not (0 < int(raw_bytes) <= ARCHITECTURE_COMPILER_OUTPUT_BYTES_MAX
                        and previous_line <= int(raw_lines) <= int(raw_bytes)
                        and int(records) == definition_count and definition_count > 0):
                    raise CheckFailure(f"{owner}: invalid compiler completion counts: {path.name}")
                complete = True
                continue
            line_number, definition = line.split("\t", 1)
            original_line = int(line_number)
            match = RESOLVED_DEF_RE.fullmatch(definition)
            if match is None or original_line < max(1, previous_line):
                raise CheckFailure(f"{owner}: malformed compiler definition record: {path.name}")
            previous_line = original_line
            definition_count += 1
            resolved = match.group(1) + ("::" + match.group(2) if match.group(2) else "")
            for authority in authorities:
                expected = authority.replace("crate::", owner.replace("-", "_") + "::", 1)
                if authority_path_matches(resolved, expected):
                    if len(failures) < ARCHITECTURE_FAILURE_COUNT_MAX:
                        failures.add(f"{owner}: resolved authority {resolved}; compiler view {path.name}:{original_line}")
                    else:
                        failures.add("additional failures not enumerated; full bounded DefId evidence retained")
    if not complete:
        raise CheckFailure(f"{owner}: compiler view is missing its complete bounded read/count: {path.name}")
    return sorted(failures)


def runtime_dependency_failures(metadata: dict, roots: list[dict], authorities: list[str],
                                core_authorities: list[str], runtime_edges: dict[str, set[tuple[str, str]]]) -> list[str]:
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    failures = []
    for owner in roots:
        rules = authorities + (core_authorities if owner["name"].endswith("-core") else [])
        pending = [(owner["id"], [owner["name"]])]
        visited = {owner["id"]}
        while pending:
            package_id, chain = pending.pop()
            for dependency in nodes[package_id]["deps"]:
                if (package_id, dependency["pkg"]) not in runtime_edges[owner["id"]]:
                    continue
                # Build scripts and proc-macro execution are host tooling, not
                # runtime authority supplied to a protected library.
                if not any(kind["kind"] is None for kind in dependency["dep_kinds"]):
                    continue
                target = packages[dependency["pkg"]]
                if any("proc-macro" in item["kind"] for item in target["targets"]):
                    continue
                name = target["name"].replace("-", "_")
                next_chain = chain + [f"{dependency['name']}={target['name']} ({target['id']})"]
                if any(authority_path_matches(name, rule) for rule in rules):
                    failures.append(f"{owner['manifest_path']}: forbidden normal runtime dependency: " + " -> ".join(next_chain))
                if target["id"] not in visited:
                    visited.add(target["id"])
                    pending.append((target["id"], next_chain))
    return failures


def architecture_toolchain(root: Path, scratch: Path, deadline: float) -> str:
    compiler = os.environ.get("RUSTC", "rustc")
    for label, command in [("cargo-identity", "cargo"), ("rustc-identity", compiler)]:
        output = architecture_capture([command, "--version", "--verbose"], root, scratch, label, deadline)
        print(output.read_text().rstrip())
    return compiler


def architecture_metadata(root: Path, scratch: Path, deadline: float) -> dict:
    output = architecture_capture(["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
                                  root, scratch, "metadata", deadline)
    return json.loads(output.read_text())


def architecture_runtime_edges(metadata: dict, owner: dict, root: Path, scratch: Path,
                               deadline: float) -> set[tuple[str, str]]:
    # metadata alone unifies workspace features. Cargo's package-selected tree
    # supplies the actual normal runtime edges, avoiding unrelated std features.
    output = architecture_capture(["cargo", "tree", "--locked", "--offline", "-p", owner["name"],
        "--edges", "normal,no-proc-macro", "--prefix", "depth", "--charset", "ascii", "--no-dedupe",
        "--format", "{p}"], root, scratch, f"{owner['name']}-runtime-tree", deadline)
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    stack: list[str] = []
    edges: set[tuple[str, str]] = set()
    pattern = re.compile(r"^(\d+)\s*([A-Za-z0-9_.-]+) v([^\s]+)(?:\s|$)")
    for line in output.read_text().splitlines():
        if not line.strip():
            continue
        match = pattern.match(line)
        if match is None:
            raise CheckFailure(f"unsupported Cargo runtime tree record: {line}")
        depth, name, version = int(match.group(1)), match.group(2), match.group(3)
        if depth == 0:
            if stack or name != owner["name"] or version != owner["version"]:
                raise CheckFailure(f"unexpected Cargo runtime tree root: {line}")
            stack = [owner["id"]]
            continue
        if depth > len(stack):
            raise CheckFailure(f"invalid Cargo runtime tree depth: {line}")
        parent = stack[depth - 1]
        candidates = {dependency["pkg"] for dependency in nodes[parent]["deps"]
                      if packages[dependency["pkg"]]["name"] == name
                      and packages[dependency["pkg"]]["version"] == version
                      and any(kind["kind"] is None for kind in dependency["dep_kinds"])}
        if len(candidates) > 1:
            candidates = {candidate for candidate in candidates
                          if str(Path(packages[candidate]["manifest_path"]).parent) in line
                          or (packages[candidate].get("source") or "no-source-match").removeprefix("git+") in line}
        if len(candidates) != 1:
            raise CheckFailure(f"Cargo runtime tree package identity is not unique in resolved metadata: {line}")
        target = candidates.pop()
        edges.add((parent, target))
        stack[depth:] = [target]
    if not stack:
        raise CheckFailure(f"empty Cargo runtime tree for {owner['name']}")
    return edges


def command_architecture(authorities: list[str], core_authorities: list[str]) -> None:
    if not authorities or not core_authorities:
        raise CheckFailure("architecture authority inventory must come from scripts/check-cli-architecture.rs")
    deadline = time.monotonic() + ARCHITECTURE_CAMPAIGN_SECONDS_MAX
    with architecture_scratch("mantle-architecture-") as directory:
        scratch = Path(directory)
        architecture_toolchain(REPO_ROOT, scratch, deadline)
        metadata = architecture_metadata(REPO_ROOT, scratch, deadline)
        members = set(metadata["workspace_members"])
        roots = sorted((package for package in metadata["packages"] if package["id"] in members
                        and Path(package["manifest_path"]).is_relative_to(REPO_ROOT / "crates")
                        and (package["name"].endswith("-core") or package["name"] == "mantle-application-contract")),
                       key=lambda package: package["name"])
        if not roots or not any(package["name"] == "mantle-application-contract" for package in roots):
            raise CheckFailure("architecture metadata is missing the protected first-party inventory")
        runtime_edges = {owner["id"]: architecture_runtime_edges(metadata, owner, REPO_ROOT, scratch, deadline)
                         for owner in roots}
        failures = runtime_dependency_failures(metadata, roots, authorities, core_authorities, runtime_edges)
        if failures:
            raise CheckFailure("\n".join(failures))
        environment = os.environ.copy()
        # A fresh, private target prevents cached unpretty invocations from
        # producing empty output and never competes for the integrator's target.
        environment["CARGO_TARGET_DIR"] = str(scratch / "target")
        for owner in roots:
            if not any("lib" in target["kind"] for target in owner["targets"]):
                raise CheckFailure(f"protected owner has no library target: {owner['manifest_path']}")
            rules = authorities + (core_authorities if owner["name"].endswith("-core") else [])
            for view in ARCHITECTURE_COMPILER_VIEWS:
                output = architecture_capture(
                    ["cargo", "rustc", "--locked", "--offline", "-p", owner["name"], "--lib", "--",
                     f"-Zunpretty={view}"], REPO_ROOT, scratch, f"{owner['name']}-{view}", deadline, environment,
                    resolved_definitions=True)
                failures = resolved_authority_failures(output, owner["name"], rules)
                if failures:
                    raise CheckFailure("\n".join(failures))
            print(f"architecture compiler owner: {owner['manifest_path']}: macro-expanded HIR + typed HIR: PASS")
        if time.monotonic() > deadline:
            raise CheckFailure("architecture campaign deadline exceeded")
        print("cli architecture compiler and resolved runtime dependencies: PASS")


def architecture_capture_self_test(scratch: Path) -> None:
    # These fixtures exercise chunking, original-line evidence, and fail-closed
    # completion independently of compiler formatting or Cargo fixture results.
    definition = b"DefId(1:2 ~ std[fixture]::process::Command::new)"
    raw = b"\n" + b" " * 65530 + definition + b" " + definition + b"\n" + definition
    output = io.BytesIO()
    capture = ResolvedDefCapture(output, "capture-thir-flat")
    capture.write(raw[:65536])
    capture.write(raw[65536:])
    capture.finish()
    path = scratch / "capture-thir-flat.stdout"
    compact = output.getvalue()
    path.write_bytes(compact)
    failures = resolved_authority_failures(path, "fixture-core", ["std::process"])
    if (capture.raw_bytes != len(raw) or capture.line_count != 3 or capture.definition_count != 3
            or not any("capture-thir-flat.stdout:2" in failure for failure in failures)
            or not any("capture-thir-flat.stdout:3" in failure for failure in failures)):
        raise CheckFailure(f"chunked compiler capture lost records or original lines: {failures}")
    # A genuine stream larger than the retained-evidence budget must still be
    # checked completely; padding does not inflate the compact DefId artifact.
    output = io.BytesIO()
    capture = ResolvedDefCapture(output, "capture-expanded-hir")
    padding = b" " * 65535 + b"\n"
    for _ in range(ARCHITECTURE_CAPTURE_BYTES_MAX // len(padding) + 1):
        capture.write(padding)
    capture.write(definition + b"\n")
    capture.finish()
    path.write_bytes(output.getvalue())
    failures = resolved_authority_failures(path, "fixture-core", ["std::process"])
    if len(failures) != 1 or "std::process::Command::new" not in failures[0]:
        raise CheckFailure(f"large streamed compiler output lost its resolved authority: {failures}")
    for label, malformed in [("empty", b""), ("unresolved", b"Thir {}\n"),
                              ("bad-defid", b"DefId(not-supported)\n"), ("error-body", definition + b"\nerror\n"),
                              ("oversized-line", definition + b"\n" + b" " * (ARCHITECTURE_DIAGNOSTIC_BYTES_MAX + 1))]:
        candidate = ResolvedDefCapture(io.BytesIO(), label)
        try:
            candidate.write(malformed)
            candidate.finish()
        except CheckFailure:
            continue
        raise CheckFailure(f"compiler capture accepted {label} output")
    for label, field, bound in [("raw", "raw_bytes", ARCHITECTURE_COMPILER_OUTPUT_BYTES_MAX),
                                ("retained", "retained_bytes", ARCHITECTURE_CAPTURE_BYTES_MAX)]:
        candidate = ResolvedDefCapture(io.BytesIO(), label)
        setattr(candidate, field, bound)
        try:
            candidate.write(definition + b"\n")
        except CheckFailure:
            continue
        raise CheckFailure(f"compiler capture accepted output beyond its {label} byte bound")
    for malformed in [compact.rsplit(b"# complete", 1)[0], compact + b"trailing\n",
                      compact.replace(b"# complete\t", b"# incomplete\t"), compact.replace(b"\t3\n", b"\t4\n")]:
        path.write_bytes(malformed)
        try:
            resolved_authority_failures(path, "fixture-core", ["std::process"])
        except (CheckFailure, ValueError):
            continue
        raise CheckFailure("compiler capture accepted incomplete/malformed completion evidence")
    # Failure enumeration is bounded, but admission must still read to the end.
    rows = b"".join(str(number).encode() + b"\t" + definition + b"\n"
                    for number in range(1, ARCHITECTURE_FAILURE_COUNT_MAX + 2))
    path.write_bytes(rows + b"malformed-tail\n")
    try:
        resolved_authority_failures(path, "fixture-core", ["std::process"])
    except (CheckFailure, ValueError):
        pass
    else:
        raise CheckFailure("compiler capture stopped validating after its failure enumeration bound")
    print("architecture compiler fixture: bounded chunked DefId capture and complete counts: accepted/rejected")


def command_architecture_self_test(authorities: list[str], core_authorities: list[str]) -> None:
    if not authorities or not core_authorities:
        raise CheckFailure("architecture fixtures require the production authority inventory")
    deadline = time.monotonic() + ARCHITECTURE_CAMPAIGN_SECONDS_MAX
    fixtures = [
        ("pure", True, '#![no_std]\nextern crate alloc;\nuse alloc::vec::Vec;\nuse core::time::Duration;\npub fn plan(value: Duration) -> Vec<Duration> { alloc::vec![value] }\n'),
        ("pure-local-mantle", True, '#![no_std]\npub fn sample(mantle: u32) -> u32 { let observed = mantle; observed + mantle }\n'),
        ("pure-reexport", True, '#![no_std]\nextern crate std;\nmod pure { pub use std::time::Duration; }\npub fn plan(value: pure::Duration) -> pure::Duration { value }\n'),
        ("cross-module-alias", False, '#![no_std]\nextern crate std;\nmod aliases { pub use std::fs as files; }\npub fn sample() { let _ = aliases::files::read("input"); }\n'),
        ("macro-process", False, '#![no_std]\nextern crate std;\nmacro_rules! spawn { () => { std::process::Command::new("cc") }; }\npub fn sample() { let _ = spawn!(); }\n'),
        ("macro-output", False, '#![no_std]\nextern crate std;\nmacro_rules! report { () => { std::println!("output") }; }\npub fn sample() { report!(); }\n'),
        ("cross-crate-reexport", False, '#![no_std]\nextern crate host_alias as bridge;\npub fn sample() { let _ = bridge::Command::new("cc"); }\n'),
    ]
    with architecture_scratch("mantle-architecture-fixtures-") as directory:
        scratch = Path(directory)
        compiler = architecture_toolchain(scratch, scratch, deadline)
        architecture_capture_self_test(scratch)
        helper = scratch / "host_alias.rs"
        helper.write_text('#![no_std]\nextern crate std;\npub use std::process::Command;\n')
        architecture_capture([compiler, "--edition=2024", "--crate-name", "host_alias", "--crate-type=rlib",
                              str(helper), "--out-dir", str(scratch)], scratch, scratch, "helper", deadline)
        for label, accepted, source in fixtures:
            path = scratch / f"{label}.rs"
            path.write_text(source)
            view_failures = {}
            # Keep the former recursive view as a small fixture oracle only:
            # production remains HIR + exhaustive indexed typed HIR, same bounds.
            for view in (*ARCHITECTURE_COMPILER_VIEWS, "thir-tree"):
                output = architecture_capture([compiler, "--edition=2024", "--crate-name", "fixture_core",
                    "--crate-type=lib", str(path), "--extern", f"host_alias={scratch / 'libhost_alias.rlib'}",
                    f"-Zunpretty={view}"], scratch, scratch, f"{label}-{view}", deadline,
                    resolved_definitions=True)
                failures = resolved_authority_failures(output, "fixture-core", authorities + core_authorities)
                if accepted == bool(failures):
                    raise CheckFailure(f"compiler authority fixture {label}/{view}: expected accepted={accepted}, saw {failures}")
                view_failures[view] = {failure.split("; compiler view ")[0] for failure in failures}
            if not view_failures["thir-tree"].issubset(view_failures["thir-flat"]):
                raise CheckFailure(f"indexed typed HIR lost recursive fixture authority: {label}: {view_failures}")
            print(f"architecture compiler fixture: {label}: {'accepted' if accepted else 'rejected'}")
        # Cargo itself resolves the renamed/transitive fixture edges. No fake
        # metadata graph is substituted for that dependency-boundary behavior.
        root = scratch / "graph"
        for name in ("fixture-core", "bridge", "tokio", "derive_host"):
            target = root / name
            (target / "src").mkdir(parents=True)
            source = ("extern crate proc_macro;\n#[proc_macro]\npub fn identity(input: proc_macro::TokenStream) -> proc_macro::TokenStream { input }\n"
                      if name == "derive_host" else "#![no_std]\npub fn value() -> u32 { 7 }\n")
            (target / "src/lib.rs").write_text(source)
            dependency = {'fixture-core': 'bridge = { path = "../bridge" }\nderive_host = { path = "../derive_host" }\n',
                          'bridge': 'renamed = { package = "tokio", path = "../tokio" }\n',
                          'derive_host': 'tokio = { path = "../tokio" }\n', 'tokio': ''}[name]
            (target / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\n{dependency}')
            if name == "derive_host":
                with (target / "Cargo.toml").open("a") as manifest:
                    manifest.write("[lib]\nproc-macro = true\n")
        (root / "Cargo.toml").write_text('[workspace]\nresolver = "3"\nmembers = ["fixture-core", "bridge", "tokio", "derive_host"]\n')
        architecture_capture(["cargo", "generate-lockfile", "--offline"], root, scratch, "fixture-lock", deadline)
        metadata = architecture_metadata(root, scratch, deadline)
        owners = [package for package in metadata["packages"] if package["name"] == "fixture-core"]
        runtime_edges = {owner["id"]: architecture_runtime_edges(metadata, owner, root, scratch, deadline)
                         for owner in owners}
        failures = runtime_dependency_failures(metadata, owners, authorities, core_authorities, runtime_edges)
        if not any("renamed=tokio" in failure and "bridge=bridge" in failure for failure in failures):
            raise CheckFailure(f"renamed transitive dependency fixture did not reject its real resolved graph: {failures}")
        print("architecture compiler fixture: renamed transitive normal dependency: rejected")
        (root / "bridge/Cargo.toml").write_text('[package]\nname = "bridge"\nversion = "0.0.0"\nedition = "2024"\n[build-dependencies]\nrenamed = { package = "tokio", path = "../tokio" }\n')
        architecture_capture(["cargo", "generate-lockfile", "--offline"], root, scratch, "tooling-fixture-lock", deadline)
        metadata = architecture_metadata(root, scratch, deadline)
        owners = [package for package in metadata["packages"] if package["name"] == "fixture-core"]
        runtime_edges = {owner["id"]: architecture_runtime_edges(metadata, owner, root, scratch, deadline)
                         for owner in owners}
        failures = runtime_dependency_failures(metadata, owners, authorities, core_authorities, runtime_edges)
        if failures:
            raise CheckFailure(f"build/proc-macro dependencies were misclassified as runtime authority: {failures}")
        print("architecture compiler fixture: build-only and proc-macro host dependencies: accepted")
    if time.monotonic() > deadline:
        raise CheckFailure("architecture fixture campaign deadline exceeded")
    print("cli architecture compiler self-test: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("deps", "purity", "scope", "api-shape", "ownership", "architecture", "architecture-self-test"))
    parser.add_argument("--authority", action="append", default=[])
    parser.add_argument("--core-authority", action="append", default=[])
    args = parser.parse_args()
    try:
        if args.command == "deps":
            command_deps()
        elif args.command == "purity":
            command_purity()
        elif args.command == "scope":
            command_scope()
        elif args.command == "api-shape":
            command_api_shape()
        elif args.command == "ownership":
            command_ownership()
        elif args.command == "architecture":
            command_architecture(args.authority, args.core_authority)
        elif args.command == "architecture-self-test":
            command_architecture_self_test(args.authority, args.core_authority)
        else:
            raise AssertionError(f"unexpected command: {args.command}")
    except CheckFailure as error:
        die(str(error))
    except ValidationError as error:
        die(str(error))
    except (OSError, subprocess.TimeoutExpired, ValueError, KeyError) as error:
        die(f"architecture/tool input could not be admitted: {error}")


if __name__ == "__main__":
    main()
