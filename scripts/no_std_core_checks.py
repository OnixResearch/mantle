#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
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
NO_STD_CHANGE_NAME = "no-std-functional-core"
ACTIVE_CHANGE_ROOT = REPO_ROOT / "openspec" / "changes" / NO_STD_CHANGE_NAME
ARCHIVE_CHANGES_ROOT = REPO_ROOT / "openspec" / "changes" / "archive"
EMPTY_TREE_OBJECT_HASH = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
TOP_LEVEL_WORKSPACE_RUST_DIRS = {"src", "tests", "examples", "benches"}
CRATE_WORKSPACE_RUST_DIRS = {"src", "tests", "examples", "benches"}
ADOPTED_CORE_INVENTORY_VERSION = 1
REQUIRED_OWNERSHIP_REVIEW_MARKERS = ("## Review verdict", "Reviewer:", "adapter-only", "unrelated")
SECOND_WAVE_VERDICT_MARKER = "shell/release business logic remains in `crunch-shell-core` and `crunch-release-core`"
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


def resolve_change_history_paths() -> list[Path]:
    archived_matches = sorted(ARCHIVE_CHANGES_ROOT.glob(f"*-{NO_STD_CHANGE_NAME}"))
    if len(archived_matches) > 1:
        matches = ", ".join(str(path.relative_to(REPO_ROOT)) for path in archived_matches)
        raise CheckFailure(f"multiple archived change roots found for {NO_STD_CHANGE_NAME}: {matches}")
    if ACTIVE_CHANGE_ROOT.exists():
        return [ACTIVE_CHANGE_ROOT]
    if len(archived_matches) == 1:
        return [ACTIVE_CHANGE_ROOT, archived_matches[0]]
    raise CheckFailure(f"missing active or archived change root for {NO_STD_CHANGE_NAME}")


def touched_std_workspace_source_files() -> list[str]:
    change_paths = resolve_change_history_paths()
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
            if next_char and not (next_char.isalpha() or next_char == "_"):
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
        if SECOND_WAVE_VERDICT_MARKER not in artifact_text:
            failures.append(
                f"{OWNERSHIP_REVIEW_PATH.relative_to(REPO_ROOT)} missing second-wave review verdict: {SECOND_WAVE_VERDICT_MARKER}"
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("deps", "purity", "scope", "api-shape", "ownership"))
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
        else:
            raise AssertionError(f"unexpected command: {args.command}")
    except CheckFailure as error:
        die(str(error))
    except ValidationError as error:
        die(str(error))


if __name__ == "__main__":
    main()
