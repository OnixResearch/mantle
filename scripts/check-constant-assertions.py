#!/usr/bin/env python3
"""Scoped guard and audit tool for inside-fn constant-only assertions."""

from __future__ import annotations

import argparse
import pathlib
import re
import sys
import tempfile

OWNED_CLEAN_TARGETS = [
    "src/shell_cmd.rs",
    "src/lib.rs",
    "src/main.rs",
    "src/bin/crunch.rs",
    "src/cli.rs",
    "src/rust_plan_execution.rs",
    "src/build_correctness.rs",
    "src/profile_cli.rs",
    "src/machine_contract_producer_tests.rs",
    "crates/mantle-rust-plan-core/src",
    "crates/mantle-rust-plan-app/src",
    "crates/crunch-test-core/src",
    "crates/crunch-test-app/src",
    "crates/crunch-test/src",
]

M4_C2_TARGETS = {
    "src/rust_plan.rs",
    "src/stagex_transition.rs",
}

FORBIDDEN_LANE_C_TARGETS = {
    "src/build_report.rs",
    "src/remote_build.rs",
    "src/store_cmd.rs",
    "src/semantic_graph.rs",
    "src/presentation/semantic_graph.rs",
    "src/self_build.rs",
    "src/source_bundle.rs",
    "src/elf_local_symbol_core.rs",
    "src/stagex_archive_core.rs",
}

ASSERT_MACROS = (
    "debug_assert_eq",
    "debug_assert_ne",
    "debug_assert",
    "assert_eq",
    "assert_ne",
    "assert",
)

TWO_ARG_MACROS = {"assert_eq", "debug_assert_eq", "assert_ne", "debug_assert_ne"}

RUST_KEYWORDS_AND_PRIMITIVES = {
    "as",
    "bool",
    "char",
    "const",
    "crate",
    "else",
    "false",
    "f32",
    "f64",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "if",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "mut",
    "pub",
    "ref",
    "return",
    "Self",
    "sizeof",
    "str",
    "super",
    "true",
    "type",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "where",
    "Some",
    "None",
    "Ok",
    "Err",
}

IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
SCREAMING_SNAKE_RE = re.compile(r"^[A-Z][A-Z0-9_]*$")
NUMERIC_LITERAL_RE = re.compile(
    r"(?<![A-Za-z0-9_])"
    r"(?:"
    r"0x[0-9a-fA-F_]*[0-9a-fA-F][0-9a-fA-F_]*(?:_*(?:[ui](?:128|64|32|16|8|size)))?"
    r"|0b[01_]*[01][01_]*(?:_*(?:[ui](?:128|64|32|16|8|size)))?"
    r"|0o[0-7_]*[0-7][0-7_]*(?:_*(?:[ui](?:128|64|32|16|8|size)))?"
    r"|[0-9][0-9_]*(?:\.(?![.A-Za-z_])[0-9_]*)?(?:[eE][+-]?[0-9_]*[0-9][0-9_]*)?(?:_*(?:[ui](?:128|64|32|16|8|size)|f(?:32|64)))?"
    r")"
    r"(?![A-Za-z0-9_])"
)


def strip_comments_and_literals_preserving_lines(text: str) -> str:
    """Replace comments and string/byte/char literals with spaces, preserving newlines."""
    out: list[str] = []
    n = len(text)
    i = 0
    block_depth = 0

    while i < n:
        ch = text[i]
        if block_depth > 0:
            if ch == "/" and i + 1 < n and text[i + 1] == "*":
                block_depth += 1
                out.extend("  ")
                i += 2
                continue
            if ch == "*" and i + 1 < n and text[i + 1] == "/":
                block_depth -= 1
                out.extend("  ")
                i += 2
                continue
            out.append("\n" if ch == "\n" else " ")
            i += 1
            continue

        # Line comment
        if ch == "/" and i + 1 < n and text[i + 1] == "/":
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
            continue

        # Block comment
        if ch == "/" and i + 1 < n and text[i + 1] == "*":
            block_depth = 1
            out.extend("  ")
            i += 2
            continue

        # Raw string / raw byte string: r#"..."#, r"...", br#"..."#, br"..."
        raw_prefix = 0
        if ch == "r" and i + 1 < n and text[i + 1] in ('"', "#"):
            raw_prefix = 1
        elif (
            ch == "b"
            and i + 2 < n
            and text[i + 1] == "r"
            and text[i + 2] in ('"', "#")
        ):
            raw_prefix = 2

        if raw_prefix:
            j = i + raw_prefix
            hashes = 0
            while j < n and text[j] == "#":
                hashes += 1
                j += 1
            if j < n and text[j] == '"':
                out.extend(" " * (j - i + 1))
                j += 1
                closing = '"' + ("#" * hashes)
                while j < n:
                    if text.startswith(closing, j):
                        out.extend(" " * len(closing))
                        j += len(closing)
                        break
                    out.append("\n" if text[j] == "\n" else " ")
                    j += 1
                i = j
                continue

        # Normal or byte string: "..." or b"..."
        if ch == '"' or (ch == "b" and i + 1 < n and text[i + 1] == '"'):
            start = i
            if ch == "b":
                out.append(" ")
                i += 1
            out.append(" ")
            i += 1
            while i < n:
                if text[i] == "\\":
                    out.append("\n" if i + 1 < n and text[i + 1] == "\n" else " ")
                    if i + 1 < n:
                        out.append("\n" if text[i + 1] == "\n" else " ")
                        i += 2
                    else:
                        i += 1
                    continue
                if text[i] == '"':
                    out.append(" ")
                    i += 1
                    break
                out.append("\n" if text[i] == "\n" else " ")
                i += 1
            _ = start
            continue

        # Char or byte char literal (avoid lifetimes like 'a or 'static)
        if ch == "'" or (ch == "b" and i + 1 < n and text[i + 1] == "'"):
            j = i + (2 if ch == "b" else 1)
            if j < n and text[j] == "\\":
                k = j + 2
                while k < n and text[k] not in ("'", "\n") and (k - j) <= 12:
                    k += 1
                if k < n and text[k] == "'":
                    out.extend(" " * (k - i + 1))
                    i = k + 1
                    continue
            elif j + 1 < n and text[j + 1] == "'" and text[j] != "\n":
                out.extend(" " * (j + 2 - i))
                i = j + 2
                continue

        out.append(ch)
        i += 1

    return "".join(out)


def mask_span(chars: list[str], start: int, end: int) -> None:
    for idx in range(start, end):
        if chars[idx] != "\n":
            chars[idx] = " "


def find_matching_delim(text: str, open_idx: int, open_ch: str, close_ch: str) -> int:
    depth = 0
    i = open_idx
    n = len(text)
    while i < n:
        if text[i] == open_ch:
            depth += 1
        elif text[i] == close_ch:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return n - 1


def mask_cfg_test_and_const_items(cleaned: str) -> str:
    """Mask #[cfg(test)] items, top-level const _: () = ..., and inline const { ... }."""
    chars = list(cleaned)
    text = cleaned
    n = len(text)

    # 1. Mask #[cfg(test)] items (including external `mod foo;` and `{ ... }` blocks/fns)
    cfg_re = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
    for match in cfg_re.finditer(text):
        pos = match.end()
        # Skip subsequent attributes #[...]
        while pos < n:
            while pos < n and text[pos].isspace():
                pos += 1
            if pos < n and text[pos] == "#":
                bracket = text.find("[", pos)
                if bracket == -1:
                    break
                pos = find_matching_delim(text, bracket, "[", "]") + 1
            else:
                break
        # Find next ';' or '{' at item level
        brace_depth = 0
        paren_depth = 0
        while pos < n:
            c = text[pos]
            if c == "(":
                paren_depth += 1
            elif c == ")":
                paren_depth = max(0, paren_depth - 1)
            elif c == ";" and brace_depth == 0 and paren_depth == 0:
                mask_span(chars, match.start(), pos + 1)
                break
            elif c == "{" and paren_depth == 0:
                end_brace = find_matching_delim(text, pos, "{", "}")
                mask_span(chars, match.start(), end_brace + 1)
                break
            pos += 1

    # Rebuild masked text so const pass skips masked cfg(test)
    text = "".join(chars)

    # 2. Mask `const _: ... = ...;` (including `const _: () = { ... };`)
    const_anon_re = re.compile(r"\bconst\s+_\s*:")
    for match in const_anon_re.finditer(text):
        pos = match.end()
        brace_depth = 0
        while pos < n:
            c = text[pos]
            if c == "{":
                brace_depth += 1
            elif c == "}":
                brace_depth = max(0, brace_depth - 1)
            elif c == ";" and brace_depth == 0:
                mask_span(chars, match.start(), pos + 1)
                break
            pos += 1

    text = "".join(chars)

    # 3. Mask inline `const { ... }` blocks (NOT `const fn` or `const X: T = ...`)
    inline_const_re = re.compile(r"\bconst\s*\{")
    for match in inline_const_re.finditer(text):
        open_brace = match.end() - 1
        end_brace = find_matching_delim(text, open_brace, "{", "}")
        mask_span(chars, match.start(), end_brace + 1)

    return "".join(chars)


def split_top_level_comma(expr: str) -> list[str]:
    parts: list[str] = []
    start = 0
    depth_p = 0
    depth_b = 0
    depth_c = 0
    for i, ch in enumerate(expr):
        if ch == "(":
            depth_p += 1
        elif ch == ")":
            depth_p = max(0, depth_p - 1)
        elif ch == "[":
            depth_b += 1
        elif ch == "]":
            depth_b = max(0, depth_b - 1)
        elif ch == "{":
            depth_c += 1
        elif ch == "}":
            depth_c = max(0, depth_c - 1)
        elif ch == "," and depth_p == 0 and depth_b == 0 and depth_c == 0:
            parts.append(expr[start:i])
            start = i + 1
    parts.append(expr[start:])
    return parts


def is_constant_only_expr(expr: str, static_names: set[str]) -> bool:
    """Return True if expr references only constants/literals/keywords (no runtime variables/statics/calls)."""
    # Remove `.method_or_field` accesses (not `..ident` ranges) so `CONST.is_empty()`
    # leaves only `CONST`, while `var.is_empty()` and `(1..x).is_empty()` still retain `var`/`x`.
    stripped = re.sub(r"(?<!\.)\.\s*[A-Za-z_][A-Za-z0-9_]*", " ", expr)
    # Remove `::ASSOC_CONST` segments so `Type::MAX` leaves `Type`,
    # while `module::fn_call()` leaves `fn_call`.
    stripped = re.sub(r"::\s*[A-Z][A-Z0-9_]*\b", " ", stripped)
    # Strip Rust numeric literals (radix, underscore-separated, exponent/float, and type suffixes)
    # before identifier extraction without splitting digit-containing identifiers like V1_NAME / U256_MAX.
    stripped = NUMERIC_LITERAL_RE.sub(" ", stripped)

    idents = IDENT_RE.findall(stripped)
    for ident in idents:
        if ident in static_names:
            return False
        if ident in RUST_KEYWORDS_AND_PRIMITIVES:
            continue
        if SCREAMING_SNAKE_RE.match(ident):
            continue
        # Any other identifier (local variable, function name, parameter, PascalCase fn/type? wait)
        # Check PascalCase: if it's followed by `::` in `Type::CONST`, check if `ident` is a PascalCase type
        # used only as qualifier in `Type::` vs a constructor/value. Usually `Type` in `Type::ASSOC` is fine,
        # e.g., `u64::MAX` (u64 is primitive) or `Size::MAX`. Let's allow PascalCase ONLY if followed by `::`
        # and not `(` in original expr.
        if ident[0].isupper() and re.search(rf"\b{re.escape(ident)}\s*::", expr):
            continue
        return False
    return True


def scan_file(path: pathlib.Path) -> list[tuple[int, str]]:
    text = path.read_text(encoding="utf-8")
    cleaned = strip_comments_and_literals_preserving_lines(text)
    masked = mask_cfg_test_and_const_items(cleaned)

    static_names = set(re.findall(r"\bstatic\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\b", masked))
    macro_re = re.compile(
        r"\b(" + "|".join(ASSERT_MACROS) + r")\s*!\s*([(\[{])"
    )

    findings: list[tuple[int, str]] = []
    close_for = {"(": ")", "[": "]", "{": "}"}

    for match in macro_re.finditer(masked):
        macro_name = match.group(1)
        open_ch = match.group(2)
        open_idx = match.end() - 1
        close_idx = find_matching_delim(masked, open_idx, open_ch, close_for[open_ch])
        body = masked[open_idx + 1 : close_idx]
        args = split_top_level_comma(body)
        checked_args = args[:2] if macro_name in TWO_ARG_MACROS else args[:1]
        expr_text = " , ".join(checked_args)
        if is_constant_only_expr(expr_text, static_names):
            line_no = masked[: match.start()].count("\n") + 1
            raw_line = text.splitlines()[line_no - 1].strip()
            findings.append((line_no, raw_line))

    return findings


def collect_target_files(root: pathlib.Path, paths: list[str]) -> list[pathlib.Path]:
    files: list[pathlib.Path] = []
    for rel in paths:
        p = root / rel
        if not p.exists():
            continue
        if p.is_file():
            files.append(p)
        else:
            files.extend(sorted(p.rglob("*.rs")))
    return sorted(set(files))


def collect_all_production_files(root: pathlib.Path) -> list[pathlib.Path]:
    files: list[pathlib.Path] = []
    src_dir = root / "src"
    if src_dir.is_dir():
        files.extend(sorted(src_dir.rglob("*.rs")))
    crates_dir = root / "crates"
    if crates_dir.is_dir():
        for crate_src in sorted(crates_dir.glob("*/src")):
            if crate_src.is_dir():
                files.extend(sorted(crate_src.rglob("*.rs")))
    return sorted(set(files))


def classify_bucket(rel: str) -> str:
    if rel == "src/shell_cmd.rs":
        return "M4-C.1 Owned"
    if rel in M4_C2_TARGETS:
        return "M4-C.2 Owned"
    if any(rel == t or rel.startswith(t + "/") for t in OWNED_CLEAN_TARGETS):
        return "Clean Owned Targets"
    if (
        rel in FORBIDDEN_LANE_C_TARGETS
        or rel.startswith("src/remote_build/")
        or rel.startswith("src/source_built_fixed_point")
        or rel.startswith("crates/crunch-build/")
        or rel.startswith("crates/crunch-store/")
    ):
        return "FORBIDDEN (Lane C)"
    return "Other Unowned"


def run_self_test() -> int:
    positive_fixture = """
const SIDECAR_FILENAME: &str = ".crunch-shell.json";
const LIMIT: usize = 16;
const V1_NAME: &str = "v1";
const U256_MAX: usize = 256;
static LOCK: std::sync::OnceLock<()> = std::sync::OnceLock::new();

const _: () = {
    assert!(!SIDECAR_FILENAME.is_empty());
};

const _: () = assert!(LIMIT > 0);

fn valid(path: &str, items: &[u8], v1_name: &str, u256_count: usize) {
    const { assert!(LIMIT > 0); }
    let _macro_in_str = "assert!(true); debug_assert_eq!(1_usize, 1_usize);";
    debug_assert!(!path.is_empty());
    debug_assert!(items.len() <= LIMIT);
    debug_assert!(LOCK.get().is_some());
    debug_assert!(!v1_name.is_empty());
    debug_assert!(u256_count <= U256_MAX && V1_NAME.len() <= u256_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_const() {
        assert!(!SIDECAR_FILENAME.is_empty());
    }
}
"""
    negative_fixture = """
const SIDECAR_FILENAME: &str = ".crunch-shell.json";
const MAX_ITEMS: usize = 10;
const MODE_A: u8 = 1;
const MODE_B: u8 = 2;
const V1_NAME: &str = "v1";
const U256_MAX: usize = 256;

fn bad() {
    debug_assert!(!SIDECAR_FILENAME.is_empty());
    assert!(MAX_ITEMS > 0, "must be positive");
    debug_assert_ne!(MODE_A, MODE_B);
    assert!("abc".len() == 3);
    assert_ne!(1_usize, 2_usize);
    debug_assert!(0x10_u64 > 0);
    assert_eq!(0b1010_u8, 10_u8);
    debug_assert!(0o77_u32 == 63);
    assert!(1_000_i32 == 1_000_i32);
    debug_assert!(1_000.0_f64 > 0.0_f64);
    assert!(!V1_NAME.is_empty() && U256_MAX > 0);
}
"""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = pathlib.Path(tmpdir)
        pos_path = tmp / "pos.rs"
        neg_path = tmp / "neg.rs"
        pos_path.write_text(positive_fixture, encoding="utf-8")
        neg_path.write_text(negative_fixture, encoding="utf-8")

        pos_findings = scan_file(pos_path)
        if pos_findings:
            print(f"FAIL: positive fixture had unexpected findings: {pos_findings}", file=sys.stderr)
            return 1

        neg_findings = scan_file(neg_path)
        expected_lines = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
        actual_lines = [line for line, _ in neg_findings]
        if actual_lines != expected_lines:
            print(
                f"FAIL: negative fixture lines {actual_lines} != expected {expected_lines}; findings={neg_findings}",
                file=sys.stderr,
            )
            return 1

    print("check-constant-assertions self-test: PASS (10 positive, 11 negative)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path("."))
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--audit-all", action="store_true")
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        return run_self_test()

    root = args.root.resolve()
    if args.audit_all:
        files = collect_all_production_files(root)
        by_bucket: dict[str, list[tuple[str, list[tuple[int, str]]]]] = {}
        total_files = 0
        total_assertions = 0
        for path in files:
            findings = scan_file(path)
            if not findings:
                continue
            rel = path.relative_to(root).as_posix()
            bucket = classify_bucket(rel)
            by_bucket.setdefault(bucket, []).append((rel, findings))
            total_files += 1
            total_assertions += len(findings)

        print(f"Total production files with constant-only assertions: {total_files} ({total_assertions} assertions)")
        for bucket in ("M4-C.1 Owned", "M4-C.2 Owned", "Clean Owned Targets", "FORBIDDEN (Lane C)", "Other Unowned"):
            entries = by_bucket.get(bucket, [])
            count = sum(len(f) for _, f in entries)
            print(f"\n== {bucket}: {len(entries)} files, {count} assertions ==")
            for rel, findings in entries:
                lines = ", ".join(f"L{ln}" for ln, _ in findings)
                print(f"  {rel} ({len(findings)}): {lines}")
        if args.strict and total_assertions > 0:
            return 1
        return 0

    files = collect_target_files(root, OWNED_CLEAN_TARGETS)
    violations: list[tuple[str, int, str]] = []
    for path in files:
        rel = path.relative_to(root).as_posix()
        for line_no, raw_line in scan_file(path):
            violations.append((rel, line_no, raw_line))

    if violations:
        for rel, line_no, raw_line in violations:
            print(f"{rel}:{line_no}: constant-only runtime assertion: {raw_line}", file=sys.stderr)
        return 1

    print(f"check-constant-assertions: PASS ({len(files)} owned/clean files, 0 violations)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
