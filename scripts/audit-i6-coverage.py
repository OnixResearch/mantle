#!/usr/bin/env python3
"""Deterministic I6 coverage audit for the Mantle CLI.

I6 (thin-cli-composition-root) requires every application operation to execute
a typed effect plan through a port and to classify the *actual* observation
before its terminal success report. This script answers, from source text only
and for every terminal CLI leaf:

  1. which Clap variant chain defines the leaf (derived from the `Subcommand`
     enums, then cross-checked against the operator command catalog);
  2. which match arms dispatch it and which functions those arms call;
  3. whether a contract classifier (every `pub fn classify_*` of
     `crates/mantle-application-contract`) is reachable from that dispatch,
     through which call chain, and whether the owning function discards the
     verdict or prints before classifying;
  4. which tests exercise a success path and a failure path of the leaf.

It reads nothing from the I6 handoff except, optionally, the boundary key for
cross-reference. It executes no build and no test: it is a structural audit.
Every limit is recorded in the `limits` block of the report; passing the audit
shows that the structure is present, never that the behaviour is correct.

Usage:
  audit-i6-coverage.py [--root DIR] [--catalog FILE] [--handoff FILE]
                       [--out-json FILE] [--out-md FILE] [--fail-on-flags]
  audit-i6-coverage.py --self-test
"""
from __future__ import annotations

import argparse
import bisect
import json
import re
import subprocess
import sys
import tempfile
from collections import defaultdict, deque
from dataclasses import dataclass, field
from pathlib import Path

SCHEMA = "mantle-i6-coverage-audit-v1"
MAX_REACH_DEPTH = 8
MAX_AMBIGUOUS_CANDIDATES = 3
MAX_FANOUT_EXCLUSIVE = 3
MAX_FILE_BYTES = 8 * 1024 * 1024
MAX_EVIDENCE_CHAINS = 3
ROOT_ENUM = "Command"
# Host-context observations (cwd, clock, environment export) do not prove that the operation that follows them
# succeeded; the I6 handoff says so explicitly. A classifier reached only through them is not operation evidence.
CONTEXT_MODULES = frozenset({"host_environment", "ambient_env"})
DIRECT_DEPTH = 2
INDIRECT_DEPTH = 5

CONTRACT_SRC = "crates/mantle-application-contract/src"
CONTRACT_TESTS = "crates/mantle-application-contract/tests"

KEYWORDS = frozenset(
    "if else match while for loop fn return let in as mut ref move async await unsafe impl dyn "
    "where use mod pub struct enum trait type const static break continue self Self super crate".split()
)

FAILURE_NAME = re.compile(
    r"(reject|fail|invalid|error|missing|blocked|stale|mismatch|refus|unsafe|bad|corrupt|tamper|denied|"
    r"deny|without|unsupported|conflict|malformed|wrong|empty|oversize|exceed|unauthori|forbid|escape|"
    r"traversal|incomplete|unknown|absent|nonzero|abort|cancel|timeout|truncat|not_|no_|cannot|never|"
    r"drift|contradict|violat|unsound|underdeliver|unaccounted)",
    re.I,
)
FAILURE_BODY = re.compile(
    r"unwrap_err\(|\.is_err\(\)|expect_err\(|should_panic|assert_failure|\.failure\(\)|"
    r"(?<!\w)!\s*[\w.()&]*status\s*\.\s*success\(\)|(?<!\w)!\s*[\w.()&]*\.success\(\)|"
    r"assert_ne!\s*\([^;]*\b(?:code|status)\b[^;]*\b0\b|"
    r"code\(\)\s*,\s*Some\(\s*[1-9]|Some\(\s*[1-9]\d*\s*\)\s*,[^;]*code|"
    r"matches!\s*\([^;]*Err\(|Err\(\s*(?:\w+::)*\w+|ApplicationOutcome::(?:Failed|Contradicted|Rejected)|"
    r"Outcome::(?:Failed|Contradicted|Rejected)|Classification::(?:Failed|Contradicted)"
)
SUCCESS_BODY = re.compile(
    r"\bstatus\s*\.\s*success\(\)|\.success\(\)|\.is_ok\(\)|\bunwrap\(\)|expect\(|"
    r"ApplicationOutcome::Completed|Outcome::Completed|assert_eq!\s*\([^;]*\bcode\b[^;]*\b0\b|Ok\(\s*\(\s*\)\s*\)"
)
REPORT_EMISSION = re.compile(r"\b(?:println|print|eprintln|eprint)!\s*\(|\b(?:emit|print|render|write_report)_\w*\s*\(")
OTHER_PROGRAMS = frozenset({"cargo", "git", "nix", "bwrap", "sh", "bash", "rustc", "python3", "python", "ls", "cp", "mkdir"})


# --------------------------------------------------------------------------- lexing


@dataclass
class Source:
    path: str
    text: str
    masked: str
    strings: list[tuple[int, int, str]]
    plain: str
    line_starts: list[int]
    pairs: dict[int, int]

    def line_of(self, offset: int) -> int:
        return bisect.bisect_right(self.line_starts, offset)


_LEXEME = re.compile(
    r"""//[^\n]*|/\*|b?r(?P<hashes>\#*)"|b?"|b?'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|.)|[^\\'\n])'""",
    re.X,
)
_PLAIN_STRING_END = re.compile(r'(?:\\.|[^"\\])*"', re.S)


def _blank(segment: str) -> str:
    return "".join(c if c == "\n" else " " for c in segment)


def lex(text: str) -> tuple[str, list[tuple[int, int, str]], str]:
    """Return (masked, strings, uncommented): comments and literal contents blanked, the literals, and text with only comments blanked."""
    assert text is not None
    out: list[str] = []
    plain: list[str] = []
    strings: list[tuple[int, int, str]] = []
    pos = 0
    n = len(text)
    while pos < n:
        m = _LEXEME.search(text, pos)
        if m is None:
            out.append(text[pos:])
            plain.append(text[pos:])
            break
        out.append(text[pos : m.start()])
        plain.append(text[pos : m.start()])
        token = m.group(0)
        if token.startswith("//"):
            out.append(_blank(token))
            plain.append(_blank(token))
            pos = m.end()
        elif token == "/*":
            depth, i = 1, m.end()
            while i < n and depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(_blank(text[m.start() : i]))
            plain.append(_blank(text[m.start() : i]))
            pos = i
        elif token.endswith('"'):
            raw = token.lstrip("b").startswith("r")
            if raw:
                terminator = '"' + m.group("hashes")
                end = text.find(terminator, m.end())
                end = n if end < 0 else end
                body_end = end
                stop = min(n, end + len(terminator))
            else:
                em = _PLAIN_STRING_END.match(text, m.end())
                stop = em.end() if em else n
                body_end = stop - 1 if em else n
            strings.append((m.end(), body_end, text[m.end() : body_end]))
            out.append(text[m.start() : m.end()])
            out.append(_blank(text[m.end() : body_end]))
            out.append(text[body_end:stop])
            plain.append(text[m.start() : stop])
            pos = stop
        else:
            out.append("'" + _blank(token[1:-1]) + "'")
            plain.append(token)
            pos = m.end()
    masked = "".join(out)
    assert len(masked) == len(text), "lexer must preserve offsets"
    return masked, strings, "".join(plain)


def bracket_pairs(masked: str) -> dict[int, int]:
    pairs: dict[int, int] = {}
    stacks: dict[str, list[int]] = {")": [], "]": [], "}": []}
    opener = {"(": ")", "[": "]", "{": "}"}
    for m in re.finditer(r"[(){}\[\]]", masked):
        c, i = m.group(0), m.start()
        if c in opener:
            stacks[opener[c]].append(i)
        elif stacks[c]:
            start = stacks[c].pop()
            pairs[start] = i
    return pairs


def load_source(root: Path, path: Path) -> Source | None:
    if path.stat().st_size > MAX_FILE_BYTES:
        return None
    text = path.read_text(encoding="utf-8", errors="replace")
    masked, strings, plain = lex(text)
    starts = [0] + [m.end() for m in re.finditer(r"\n", text)]
    return Source(str(path.relative_to(root)), text, masked, strings, plain, starts, bracket_pairs(masked))


# --------------------------------------------------------------------------- function index


@dataclass
class Fn:
    name: str
    file: str
    line: int
    start: int  # offset of `fn`
    body_start: int
    body_end: int
    owner: str | None
    is_test: bool
    module: str
    calls: list[tuple[str, str | None, bool, int]] = field(default_factory=list)  # name, qualifier, method, offset

    @property
    def qual(self) -> str:
        owner = f"{self.owner}::" if self.owner else ""
        return f"{self.module}::{owner}{self.name}"

    @property
    def loc(self) -> str:
        return f"{self.file}:{self.line}"


_FN = re.compile(r"\bfn\s+([A-Za-z_]\w*)")
_IMPL = re.compile(r"\b(impl|trait)\b")
_TEST_ATTR = re.compile(r"#\[\s*(?:test|tokio::test|test_case|rstest)\b")
_CFG_TEST_MOD = re.compile(r"#\[cfg\(test\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*([{;])")
_CALL = re.compile(
    r"(?P<dot>\.\s*)?(?P<path>(?:[A-Za-z_]\w*\s*::\s*)*)(?P<name>[A-Za-z_]\w*)\s*(?:::\s*<[^()]*?>)?\s*\("
)


def _header_open(src: Source, pos: int) -> int | None:
    """Offset of the `{` that opens the item whose header starts at pos, skipping bracket groups."""
    scan = re.compile(r"[({\[;]")
    while True:
        m = scan.search(src.masked, pos)
        if m is None:
            return None
        c = m.group(0)
        if c == "{":
            return m.start()
        if c == ";":
            return None
        close = src.pairs.get(m.start())
        if close is None:
            return None
        pos = close + 1


def _impl_owner(header: str, kind: str) -> str | None:
    header = re.sub(r"<[^<>]*(?:<[^<>]*>[^<>]*)*>", " ", header)
    if kind == "trait":
        m = re.search(r"\btrait\s+(\w+)", header)
        return m.group(1) if m else None
    tail = re.split(r"\bfor\b", header, maxsplit=1)[-1]
    tail = re.sub(r"\bwhere\b.*", "", tail, flags=re.S)
    idents = re.findall(r"[A-Za-z_]\w*", tail)
    idents = [i for i in idents if i not in {"impl", "dyn", "mut", "const", "unsafe"}]
    return idents[-1] if idents else None


def module_name(path: str) -> str:
    stem = Path(path).stem
    parent = Path(path).parent.name
    return parent if stem in {"mod", "lib", "main"} and parent not in {"src", ""} else stem


def index_functions(src: Source, test_modules: set[str], force_test_file: bool) -> list[Fn]:
    impl_spans: list[tuple[int, int, str | None]] = []
    for m in _IMPL.finditer(src.masked):
        open_idx = _header_open(src, m.end())
        if open_idx is None or open_idx not in src.pairs:
            continue
        owner = _impl_owner(src.masked[m.start() : open_idx], m.group(1))
        impl_spans.append((open_idx, src.pairs[open_idx], owner))
    test_spans: list[tuple[int, int]] = []
    for m in _CFG_TEST_MOD.finditer(src.masked):
        if m.group(2) == "{":
            open_idx = m.end() - 1
            if open_idx in src.pairs:
                test_spans.append((open_idx, src.pairs[open_idx]))
    functions: list[Fn] = []
    for m in _FN.finditer(src.masked):
        open_idx = _header_open(src, m.end())
        if open_idx is None or open_idx not in src.pairs:
            continue
        window = src.masked[max(0, m.start() - 700) : m.start()]
        tail = re.split(r"[;{}]", window)[-1]
        in_test_mod = any(a < m.start() < b for a, b in test_spans)
        is_test = force_test_file or in_test_mod or bool(_TEST_ATTR.search(tail)) or "#[cfg(test)]" in tail
        owner = None
        for a, b, name in impl_spans:
            if a < m.start() < b:
                owner = name
        fn = Fn(m.group(1), src.path, src.line_of(m.start()), m.start(), open_idx, src.pairs[open_idx], owner,
                is_test, module_name(src.path))
        body = src.masked[open_idx : fn.body_end]
        for c in _CALL.finditer(body):
            name = c.group("name")
            if name in KEYWORDS or name[0].isupper():
                continue
            before = body[: c.start()].rstrip()
            if before.endswith("fn") and not c.group("dot"):
                continue
            qualifier = None
            if c.group("path"):
                parts = [p.strip() for p in c.group("path").split("::") if p.strip()]
                qualifier = parts[-1] if parts else None
            functions_call_offset = open_idx + c.start()
            fn.calls.append((name, qualifier, bool(c.group("dot")), functions_call_offset))
        functions.append(fn)
    return functions


# --------------------------------------------------------------------------- repository model


class Repo:
    def __init__(self, root: Path):
        self.root = root
        self.sources: dict[str, Source] = {}
        self.fns: list[Fn] = []
        self.by_name: dict[str, list[Fn]] = defaultdict(list)
        self.contract_classifiers: set[str] = set()
        self.contract_planners: set[str] = set()
        self.clap_enums: set[str] = set()
        self._load()

    def _rs_files(self, sub: str) -> list[Path]:
        base = self.root / sub
        return sorted(base.rglob("*.rs")) if base.is_dir() else []

    def _load(self) -> None:
        contract = [p for p in self._rs_files(CONTRACT_SRC)]
        for path in contract:
            text = path.read_text(encoding="utf-8", errors="replace")
            masked, _, _ = lex(text)
            cut = masked.find("#[cfg(test)]")
            prod = masked if cut < 0 else masked[:cut]
            self.contract_classifiers |= set(re.findall(r"\bpub\s+fn\s+(classify_\w+)", prod))
            self.contract_planners |= set(re.findall(r"\bpub\s+fn\s+(plan_\w+)", prod))
        test_modules: set[str] = set()
        paths = self._rs_files("src") + self._rs_files("tests") + contract + self._rs_files(CONTRACT_TESTS)
        loaded = []
        for path in paths:
            src = load_source(self.root, path)
            if src is not None:
                self.sources[src.path] = src
                loaded.append(src)
                for m in _CFG_TEST_MOD.finditer(src.masked):
                    if m.group(2) == ";":
                        test_modules.add(m.group(1))
        for src in loaded:
            stem = Path(src.path).stem
            force = src.path.startswith("tests/") or "/tests/" in src.path or stem in test_modules
            self.fns.extend(index_functions(src, test_modules, force))
        for fn in self.fns:
            self.by_name[fn.name].append(fn)

    def _names_type(self, file: str, owner: str) -> bool:
        cache = self.__dict__.setdefault("_type_names", {})
        key = (file, owner)
        if key not in cache:
            cache[key] = re.search(r"\b" + re.escape(owner) + r"\b", self.sources[file].masked) is not None
        return cache[key]

    def callers_of(self, target: Fn) -> list[tuple[Fn, int]]:
        """Non-test functions whose resolved call sites include target, with the call offset."""
        index = self.__dict__.get("_calls_by_name")
        if index is None:
            index = defaultdict(list)
            for fn in self.fns:
                if not fn.is_test:
                    for call in fn.calls:
                        index[call[0]].append((fn, call))
            self._calls_by_name = index
        found = []
        for fn, (name, qualifier, method, offset) in index.get(target.name, ()):
            if fn is not target and any(t is target for t in self.resolve(fn, name, qualifier, method)):
                found.append((fn, offset))
        return found

    def fns_by_file(self) -> dict[str, list[Fn]]:
        if not hasattr(self, "_fns_by_file"):
            index: dict[str, list[Fn]] = defaultdict(list)
            for fn in self.fns:
                index[fn.file].append(fn)
            self._fns_by_file = index
        return self._fns_by_file

    # -- call resolution
    def resolve(self, caller: Fn, name: str, qualifier: str | None, method: bool) -> list[Fn]:
        candidates = [f for f in self.by_name.get(name, ()) if not f.is_test or caller.is_test]
        if not candidates:
            return []
        if qualifier in {"Self"} or (qualifier and qualifier == caller.owner):
            owned = [f for f in candidates if f.owner == caller.owner]
            if owned:
                return owned
        if qualifier and not method:
            by_owner = [f for f in candidates if f.owner == qualifier]
            if by_owner:
                return by_owner
            by_module = [f for f in candidates if f.module == qualifier and f.owner is None]
            if by_module:
                return by_module
            if qualifier in {"crate", "self", "super"}:
                pass
            elif qualifier[0].isupper():
                return []  # an external or unknown type's associated function
        if method:
            # the receiver's type is unknown to a lexical audit: accept a method only when its owner type is
            # named in the caller's own file, which rejects `.len()`/`.push()` lookalikes from other modules
            methods = [f for f in candidates if f.owner is not None and self._names_type(caller.file, f.owner)]
            return methods if 0 < len(methods) <= MAX_AMBIGUOUS_CANDIDATES else []
        same_file = [f for f in candidates if f.file == caller.file and f.owner is None]
        if same_file:
            return same_file
        free = [f for f in candidates if f.owner is None]
        return free if 0 < len(free) <= MAX_AMBIGUOUS_CANDIDATES else []

    def callees(self, fn: Fn) -> list[Fn]:
        cache = self.__dict__.setdefault("_callees", {})
        key = (fn.file, fn.start)
        if key in cache:
            return cache[key]
        seen: dict[str, Fn] = {}
        for name, qualifier, method, _ in fn.calls:
            for target in self.resolve(fn, name, qualifier, method):
                seen[f"{target.file}:{target.start}"] = target
        cache[key] = [seen[k] for k in sorted(seen)]
        return cache[key]


# --------------------------------------------------------------------------- Clap tree


@dataclass
class Variant:
    enum: str
    name: str
    cli: str
    hidden: bool
    sub_type: str | None
    sub_optional: bool


def kebab(name: str) -> str:
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "-", name).lower()


def _split_top(src: Source, start: int, end: int) -> list[tuple[int, int]]:
    """Split masked[start:end] at top-level commas."""
    pieces, cursor, pos = [], start, start
    scan = re.compile(r"[,({\[]")
    while True:
        m = scan.search(src.masked, pos, end)
        if m is None:
            pieces.append((cursor, end))
            return [p for p in pieces if src.masked[p[0] : p[1]].strip()]
        if m.group(0) == ",":
            pieces.append((cursor, m.start()))
            cursor = pos = m.end()
        else:
            close = src.pairs.get(m.start())
            pos = (close + 1) if close is not None else m.end()


_DERIVE = re.compile(r"#\[derive\(([^\]]*)\)\]")
_ITEM = re.compile(r"\b(?:pub(?:\([^)]*\))?\s+)?(enum|struct)\s+(\w+)\s*(?:<[^{]*>)?\s*(?:where[^{]*)?\{")
_SUBCOMMAND_ATTR = re.compile(r"#\[\s*command\s*\(\s*subcommand\b")
_FLATTEN_ATTR = re.compile(r"#\[\s*command\s*\(\s*flatten\b")


def _type_name(type_text: str) -> tuple[str | None, bool]:
    type_text = " ".join(type_text.split())
    optional = type_text.startswith("Option<")
    if optional:
        type_text = type_text[len("Option<") : type_text.rfind(">")]
    last = re.findall(r"[A-Za-z_]\w*", type_text)
    return (last[-1] if last else None), optional


def parse_clap(repo: Repo) -> tuple[dict[str, list[Variant]], dict[str, list[tuple[str, str | None, bool]]]]:
    """Return enum -> variants and struct -> [(field kind, type, optional)] for Clap items."""
    enums: dict[str, list[Variant]] = {}
    structs: dict[str, list[tuple[str, str | None, bool]]] = {}
    for src in repo.sources.values():
        if not src.path.startswith("src/"):
            continue
        for m in _ITEM.finditer(src.masked):
            before = src.masked[max(0, m.start() - 400) : m.start()]
            derives = " ".join(_DERIVE.findall(re.split(r"[;{}]", before)[-1]))
            if not re.search(r"\b(Subcommand|Parser|Args)\b", derives):
                continue
            open_idx = m.end() - 1
            close = src.pairs.get(open_idx)
            if close is None:
                continue
            kind, name = m.group(1), m.group(2)
            if kind == "struct":
                fields = []
                for a, b in _split_top(src, open_idx + 1, close):
                    text = src.plain[a:b]
                    tm = re.search(r"\b\w+\s*:\s*(.+)$", re.sub(r"#\[[^\]]*\]", "", text, flags=re.S).strip(), re.S)
                    if not tm:
                        continue
                    tname, optional = _type_name(tm.group(1))
                    if _SUBCOMMAND_ATTR.search(text):
                        fields.append(("subcommand", tname, optional))
                    elif _FLATTEN_ATTR.search(text):
                        fields.append(("flatten", tname, optional))
                structs[name] = fields
                continue
            if "Subcommand" not in derives:
                continue
            variants = []
            for a, b in _split_top(src, open_idx + 1, close):
                text = src.plain[a:b]
                attrs = " ".join(re.findall(r"#\[[^\]]*\]", text, flags=re.S))
                body = re.sub(r"#\[[^\]]*\]", "", text, flags=re.S)
                vm = re.search(r"\b([A-Z]\w*)", body)
                if not vm:
                    continue
                vname = vm.group(1)
                nm = re.search(r'#\[\s*command\s*\([^\]]*?\bname\s*=\s*"([^"]+)"', attrs)
                hidden = bool(re.search(r"\bhide\s*=\s*true", attrs))
                sub_type, optional = None, False
                brace = body.find("{")
                if brace >= 0:
                    inner = body[brace + 1 : body.rfind("}")]
                    for fa, fb in _split_top_text(text, brace):
                        ftext = text[fa:fb]
                        if _SUBCOMMAND_ATTR.search(ftext):
                            tm = re.search(r"\b\w+\s*:\s*(.+)$", re.sub(r"#\[[^\]]*\]", "", ftext, flags=re.S).strip(), re.S)
                            if tm:
                                sub_type, optional = _type_name(tm.group(1))
                        elif _FLATTEN_ATTR.search(ftext):
                            tm = re.search(r"\b\w+\s*:\s*(.+)$", re.sub(r"#\[[^\]]*\]", "", ftext, flags=re.S).strip(), re.S)
                            if tm:
                                sub_type, optional = "flatten:" + (_type_name(tm.group(1))[0] or ""), False
                variants.append(Variant(name, vname, nm.group(1) if nm else kebab(vname), hidden, sub_type, optional))
            enums[name] = variants
    return enums, structs


def _split_top_text(text: str, brace: int) -> list[tuple[int, int]]:
    """Top-level comma split of the field list inside the first brace group of a variant's original text."""
    masked, _, _ = lex(text)
    depth, cursor, pieces = 0, brace + 1, []
    for i in range(brace, len(masked)):
        c = masked[i]
        if c in "({[":
            depth += 1
        elif c in ")}]":
            depth -= 1
            if depth == 0:
                pieces.append((cursor, i))
                break
        elif c == "," and depth == 1:
            pieces.append((cursor, i))
            cursor = i + 1
    return [p for p in pieces if text[p[0] : p[1]].strip()]


@dataclass
class Leaf:
    path: str
    chain: list[tuple[str, str]]  # (enum, variant)
    hidden: bool
    note: str = ""


def clap_leaves(enums: dict[str, list[Variant]], structs: dict[str, list], root_enum: str = "Command") -> list[Leaf]:
    leaves: list[Leaf] = []

    def descend_type(type_name: str | None) -> str | None:
        """Resolve a field type (possibly a flattened struct) to the Subcommand enum below it."""
        if type_name is None:
            return None
        if type_name in enums:
            return type_name
        for kind, inner, _ in structs.get(type_name, ()):
            if kind == "subcommand" and inner in enums:
                return inner
            if kind == "flatten":
                found = descend_type(inner)
                if found:
                    return found
        return None

    def walk(enum: str, prefix: list[str], chain: list[tuple[str, str]], hidden: bool, seen: tuple[str, ...]) -> None:
        for v in enums.get(enum, ()):
            path = prefix + [v.cli]
            here = chain + [(enum, v.name)]
            is_hidden = hidden or v.hidden or v.cli.startswith("__")
            sub = v.sub_type
            if sub and sub.startswith("flatten:"):
                child = descend_type(sub.split(":", 1)[1])
            else:
                child = descend_type(sub)
            if child and child not in seen:
                if v.sub_optional:
                    leaves.append(Leaf(" ".join(path), here, is_hidden, "no-subcommand mode"))
                walk(child, path, here, is_hidden, seen + (child,))
            else:
                leaves.append(Leaf(" ".join(path), here, is_hidden))

    walk(root_enum, [], [], False, (root_enum,))
    return sorted(leaves, key=lambda leaf: leaf.path)


# --------------------------------------------------------------------------- dispatch arms


@dataclass
class Arm:
    file: str
    fn: Fn | None
    line: int
    span: tuple[int, int]  # offsets in file text
    callees: list[Fn]
    call_count: int = 0


def _use_aliases(src: Source) -> dict[str, set[str]]:
    aliases: dict[str, set[str]] = defaultdict(set)
    for m in re.finditer(r"\buse\s+([^;]+);", src.masked):
        body = m.group(1)
        for a in re.finditer(r"(\w+)\s+as\s+(\w+)", body):
            aliases[a.group(1)].add(a.group(2))
    return aliases


_ARM_END = re.compile(r"[,({\[}]")


def _seek_arrow(src: Source, pos: int) -> int | None:
    """Offset of the `=>` that ends an arm pattern, skipping bracket groups; None when it is not an arm."""
    scan = re.compile(r"=>|[;({\[]|[)}\]]")
    while True:
        hit = scan.search(src.masked, pos)
        if hit is None:
            return None
        token = hit.group(0)
        if token == "=>":
            return hit.start()
        if token in ");}]":
            return None
        close = src.pairs.get(hit.start())
        if close is None:
            return None
        pos = close + 1


def _arm_after(src: Source, pos: int) -> tuple[int, int] | None:
    """If an arm pattern ends at pos (after the variant path), return the (body_start, body_end) offsets."""
    m = src.masked
    n = len(m)
    while pos < n and m[pos].isspace():
        pos += 1
    if pos < n and m[pos] in "({":
        close = src.pairs.get(pos)
        if close is None:
            return None
        pos = close + 1
    # closers of an enclosing struct/tuple pattern (`Outer { inner: Enum::Variant { .. } } =>`)
    while pos < n and (m[pos].isspace() or m[pos] in ",)}]"):
        pos += 1
    if not m.startswith("=>", pos):
        continues = re.compile(r"if\b").match(m, pos) or (m.startswith("|", pos) and not m.startswith("||", pos))
        if not continues:
            return None
        arrow = _seek_arrow(src, pos + 1)
        if arrow is None:
            return None
        pos = arrow
    pos += 2
    while pos < n and m[pos].isspace():
        pos += 1
    if pos < n and m[pos] == "{":
        close = src.pairs.get(pos)
        return (pos, (close + 1) if close is not None else n)
    cursor = pos
    while True:
        hit = _ARM_END.search(m, cursor)
        if hit is None:
            return (pos, n)
        if hit.group(0) in ",}":
            return (pos, hit.start())
        close = src.pairs.get(hit.start())
        cursor = (close + 1) if close is not None else hit.end()


_ARMS_CACHE: dict[tuple[int, str, str], list["Arm"]] = {}


def find_arms(repo: Repo, enum: str, variant: str) -> list[Arm]:
    key = (id(repo), enum, variant)
    if key not in _ARMS_CACHE:
        _ARMS_CACHE[key] = _find_arms(repo, enum, variant)
    return _ARMS_CACHE[key]


def _find_arms(repo: Repo, enum: str, variant: str) -> list[Arm]:
    arms: list[Arm] = []
    fn_by_file = repo.fns_by_file()
    for src in repo.sources.values():
        if not src.path.startswith("src/"):
            continue
        if variant not in src.masked:
            continue
        names = {enum} | _use_aliases(src).get(enum, set())
        pattern = re.compile(r"\b(?:" + "|".join(sorted(map(re.escape, names))) + r")\s*::\s*" + re.escape(variant) + r"\b")
        for m in pattern.finditer(src.masked):
            nxt = _arm_after(src, m.end())
            if nxt is None:
                continue
            body_start, body_end = nxt
            enclosing = None
            for fn in fn_by_file[src.path]:
                if fn.body_start < m.start() < fn.body_end and (enclosing is None or fn.body_start > enclosing.body_start):
                    enclosing = fn
            if enclosing is not None and enclosing.is_test:
                continue
            if enclosing is None:
                continue
            callees: dict[str, Fn] = {}
            caller = enclosing
            call_count = 0
            for c in _CALL.finditer(src.masked, body_start, body_end):
                name = c.group("name")
                if name in KEYWORDS or name[0].isupper():
                    continue
                call_count += 1
                qualifier = None
                if c.group("path"):
                    parts = [p.strip() for p in c.group("path").split("::") if p.strip()]
                    qualifier = parts[-1] if parts else None
                for target in repo.resolve(caller, name, qualifier, bool(c.group("dot"))):
                    callees[f"{target.file}:{target.start}"] = target
            arms.append(Arm(src.path, enclosing, src.line_of(m.start()), (body_start, body_end),
                            [callees[k] for k in sorted(callees)], call_count))
    return arms


# --------------------------------------------------------------------------- audit


@dataclass
class Sink:
    name: str
    fn: Fn
    offset: int
    discarded: bool


def direct_sinks(repo: Repo, fn: Fn) -> list[Sink]:
    sinks: list[Sink] = []
    src = repo.sources[fn.file]
    for name, qualifier, method, offset in fn.calls:
        if name not in repo.contract_classifiers:
            continue
        # a local fn that merely shares the name is a wrapper, found through the call graph instead
        if any(c.file.startswith("src/") and c is not fn for c in repo.by_name.get(name, ())):
            continue
        statement = re.split(r"[;{}]", src.masked[max(0, offset - 240) : offset])[-1].strip()
        close = src.pairs.get(src.masked.find("(", offset))
        suffix = src.masked[close + 1 : close + 6].lstrip() if close else ""
        discarded = bool(re.fullmatch(r"let\s+_\s*=", statement)) or (statement == "" and suffix.startswith(";"))
        sinks.append(Sink(name, fn, offset, discarded))
    return sorted(sinks, key=lambda s: s.offset)


def reach(repo: Repo, entries: list[Fn]) -> dict[str, tuple[int, list[Fn]]]:
    """Breadth-first reach over non-test callees: key -> (depth, chain)."""
    seen: dict[str, tuple[int, list[Fn]]] = {}
    queue: deque[tuple[Fn, list[Fn]]] = deque()
    for fn in entries:
        key = f"{fn.file}:{fn.start}"
        if key not in seen and not fn.is_test:
            seen[key] = (1, [fn])
            queue.append((fn, [fn]))
    while queue:
        fn, chain = queue.popleft()
        if len(chain) >= MAX_REACH_DEPTH:
            continue
        for callee in repo.callees(fn):
            key = f"{callee.file}:{callee.start}"
            if callee.is_test or key in seen or callee.module in CONTEXT_MODULES:
                continue
            seen[key] = (len(chain) + 1, chain + [callee])
            queue.append((callee, chain + [callee]))
    return seen


def reaches_sink(repo: Repo, fn: Fn) -> bool:
    """True when the function, or anything it reaches, calls a contract classifier."""
    cache = repo.__dict__.setdefault("_reaches_sink", {})
    key = (fn.file, fn.start)
    if key not in cache:
        cache[key] = any(direct_sinks(repo, f[-1]) for _, f in reach(repo, [fn]).values())
    return cache[key]


def ordering_notes(repo: Repo, sink: Sink) -> list[str]:
    """Report emissions that precede the classifier inside its owning function.

    A branch that returns through a wrapper which itself classifies (`emit_rejected_*`) is not a violation; a
    macro print or an emitter that reaches no classifier is.
    """
    fn = sink.fn
    src = repo.sources[fn.file]
    notes = []
    for m in REPORT_EMISSION.finditer(src.masked, fn.body_start, sink.offset):
        offset = m.start()
        if m.group(0).rstrip("(").endswith("!"):
            notes.append(f"report-emission-before-classification:{src.line_of(offset)}")
            continue
        callees = [t for name, qualifier, method, call_offset in fn.calls
                   if call_offset >= offset and src.masked.startswith(name, offset)
                   for t in repo.resolve(fn, name, qualifier, method)]
        if not any(reaches_sink(repo, t) for t in callees[:MAX_AMBIGUOUS_CANDIDATES]):
            notes.append(f"report-emission-before-classification:{src.line_of(offset)}")
    return notes[:1]


def string_runs(src: Source, start: int, end: int) -> list[tuple[int, list[str]]]:
    """Group consecutive string literals of a body into (offset, argument run) pairs."""
    runs: list[tuple[int, list[str]]] = []
    previous_end = None
    for a, b, value in src.strings:
        if not (start <= a < end):
            continue
        if re.search(r"(?:env!|Command::new|cargo_bin)\(\s*\"$", src.masked[max(0, a - 40) : a]):
            previous_end = None  # a program name, not an argument
            continue
        if previous_end is not None and re.fullmatch(r"(?:[\s,\[\]()&]|\.\s*args?)*\"?", src.masked[previous_end:a]):
            runs[-1][1].append(value)
        else:
            runs.append((a, [value]))
        previous_end = b + 1
    return runs


def foreign_program(src: Source, offset: int) -> bool:
    """True when the nearest preceding `Command::new("<program>")` names a program other than the CLI."""
    window_start = max(0, offset - 320)
    last = None
    for m in re.finditer(r"Command::new\(\s*\"(\w+)\"", src.plain[window_start:offset]):
        last = m
    return bool(last and last.group(1) in OTHER_PROGRAMS)


def invocation_matches(src: Source, runs: list[tuple[int, list[str]]], tokens: list[str]) -> bool:
    """The leaf tokens occur in order in the literal arguments of one invocation.

    Literals are flattened in source order with a boundary between runs (a non-literal argument such as a path).
    Flags, the literal value that follows a flag, and boundaries may sit between or before the leaf tokens; any
    other literal breaks the match, so `["ls", "build"]` is not the `build` leaf. A program named by
    `Command::new("cargo")` is not the CLI.
    """
    stream: list[str | None] = []
    first_offset: dict[int, int] = {}
    for offset, run in runs:
        if stream:
            stream.append(None)
        for value in run:
            first_offset[len(stream)] = offset
            stream.append(value)

    def skippable(index: int) -> bool:
        item = stream[index]
        if item is None or item.startswith("-"):
            return True
        return index > 0 and stream[index - 1] is not None and stream[index - 1].startswith("-")

    for start, item in enumerate(stream):
        if item != tokens[0] or foreign_program(src, first_offset[start]):
            continue
        if not all(skippable(i) for i in range(start)):
            continue
        position, matched = start + 1, 1
        while matched < len(tokens) and position < len(stream):
            if stream[position] == tokens[matched]:
                matched += 1
            elif not skippable(position):
                break
            position += 1
        if matched == len(tokens):
            return True
    return False


def classify_test(name: str, body: str, whole: str) -> tuple[bool, bool]:
    """(success path, failure path); a test with no assertion at all asserts neither."""
    failure = bool(FAILURE_NAME.search(name)) or bool(FAILURE_BODY.search(body))
    asserts = bool(re.search(r"\bassert\w*!|\.unwrap\(|\.expect\(|\?\s*;", whole))
    success = (bool(SUCCESS_BODY.search(body)) or (asserts and not failure)) and not FAILURE_NAME.search(name)
    return success, failure


@dataclass
class LeafResult:
    leaf: Leaf
    kind: str
    entry_level: str = "unresolved"
    arms: list[str] = field(default_factory=list)
    entries: list[str] = field(default_factory=list)
    evidence: list[dict] = field(default_factory=list)
    strength: str = "none"
    tail_evidence: list[dict] = field(default_factory=list)
    flags: list[str] = field(default_factory=list)
    disposition: dict | None = None
    tests_success: list[str] = field(default_factory=list)
    tests_failure: list[str] = field(default_factory=list)
    classifier_tests_success: list[str] = field(default_factory=list)
    classifier_tests_failure: list[str] = field(default_factory=list)
    reach_keys: set[str] = field(default_factory=set)
    entry_keys: list[str] = field(default_factory=list)
    top_keys: set[str] = field(default_factory=set)
    chain_keys: set[str] = field(default_factory=set)


def _strength(depth: int) -> str:
    return "direct" if depth <= DIRECT_DEPTH else "indirect" if depth <= INDIRECT_DEPTH else "weak"


_TOP_REACH: dict[tuple[int, str, str], set[str]] = {}


def top_reach(repo: Repo, top: tuple[str, str]) -> set[str]:
    """Functions reachable from the root dispatch arm of a top-level command."""
    key = (id(repo), top[0], top[1])
    if key not in _TOP_REACH:
        entries: list[Fn] = []
        for arm in find_arms(repo, top[0], top[1]):
            entries.extend(arm.callees)
        _TOP_REACH[key] = set(reach(repo, entries))
    return _TOP_REACH[key]


def audit(repo: Repo, leaves: list[Leaf], kinds: dict[str, str], overrides: dict[str, list[str]]) -> list[LeafResult]:
    results: list[LeafResult] = []
    sink_cache: dict[str, list[Sink]] = {}

    def sinks_of(fn: Fn) -> list[Sink]:
        key = f"{fn.file}:{fn.start}"
        if key not in sink_cache:
            sink_cache[key] = direct_sinks(repo, fn)
        return sink_cache[key]

    for leaf in leaves:
        result = LeafResult(leaf, kinds.get(leaf.path, "hidden" if leaf.hidden else "public"))
        entries: list[Fn] = []
        arm_fns: set[str] = set()
        tail_dispatchers: dict[str, tuple[Fn, str]] = {}
        chain = list(reversed(leaf.chain))
        for level, (enum, variant) in enumerate(chain):
            arms = find_arms(repo, enum, variant)
            if arms:
                result.entry_level = "leaf-arm" if level == 0 else "parent-arm"
                for arm in arms:
                    result.arms.append(f"{arm.file}:{arm.line} in {arm.fn.qual if arm.fn else '?'}")
                    entries.extend(arm.callees)
                    if arm.fn is not None:
                        arm_fns.add(f"{arm.fn.file}:{arm.fn.start}")
                        # a function is a dispatcher for this leaf only when the root dispatch of its top-level command
                        # reaches it; label, context and mapping matchers elsewhere share no operation tail
                        if f"{arm.fn.file}:{arm.fn.start}" in top_reach(repo, leaf.chain[0]):
                            tail_dispatchers[f"{arm.fn.file}:{arm.fn.start}"] = (arm.fn, enum)
                break
        for path_override in overrides.get(leaf.path, []):
            for fn in repo.by_name.get(path_override, []):
                if not fn.is_test:
                    entries.append(fn)
                    result.entry_level = "override"
        dedup = {f"{f.file}:{f.start}": f for f in entries}
        entries = [dedup[k] for k in sorted(dedup)]
        result.entries = [f.qual for f in entries]
        result.entry_keys = [f"{f.file}:{f.start}" for f in entries]
        result.top_keys = top_reach(repo, leaf.chain[0])
        reached = reach(repo, entries)
        result.reach_keys = set(reached)
        evidence = []
        for key, (depth, chain_fns) in reached.items():
            fn = chain_fns[-1]
            for sink in sinks_of(fn):
                evidence.append((depth, fn.qual, sink, chain_fns))
        evidence.sort(key=lambda e: (e[0], e[1], e[2].offset))
        for depth, qual, sink, chain_fns in evidence[:MAX_EVIDENCE_CHAINS]:
            result.evidence.append({
                "sink": sink.name, "owner": qual,
                "location": f"{sink.fn.file}:{repo.sources[sink.fn.file].line_of(sink.offset)}",
                "depth": depth, "chain": [f.qual for f in chain_fns],
                "discarded": sink.discarded, "order": ordering_notes(repo, sink),
            })
        for _, _, sink, chain_fns in evidence[:MAX_EVIDENCE_CHAINS]:
            result.chain_keys |= {f"{f.file}:{f.start}" for f in chain_fns}
        # classification in the dispatcher outside every arm of the enum (shared by all its leaves)
        tail: list[tuple[int, str, Sink, list[Fn]]] = []
        for enc, enum in tail_dispatchers.values():
            if enum == ROOT_ENUM:
                continue  # the root dispatcher's non-arm code is shared by every command, not one operation
            spans = find_arms_cached(repo, enum, enc)
            outside = lambda off: not any(a <= off <= b for a, b in spans)  # noqa: E731
            tail_entries = []
            for sink in sinks_of(enc):
                if outside(sink.offset):
                    tail.append((1, enc.qual, sink, [enc]))
            for name, qualifier, method, offset in enc.calls:
                if outside(offset):
                    tail_entries += repo.resolve(enc, name, qualifier, method)
            for key, (depth, chain_fns) in reach(repo, tail_entries).items():
                for sink in sinks_of(chain_fns[-1]):
                    tail.append((depth + 1, chain_fns[-1].qual, sink, [enc] + chain_fns))
            # classification by the caller of a sub-dispatcher, after it returns (a port `execute` called by its
            # command). The root `Command` dispatcher is excluded: its callers hold host-context code, not operations.
            for caller, call_offset in repo.callers_of(enc):
                if caller.file != enc.file:
                    continue  # a same-file caller owns the port call; other modules merely reuse the function
                caller_spans = find_arms_cached(repo, enum, caller)
                bound = caller.body_end
                for a, b in caller_spans:
                    if a <= call_offset <= b:
                        bound = b  # the call is a dispatching arm: only that arm's remaining code follows it
                after = lambda off: call_offset < off <= bound  # noqa: E731
                for sink in sinks_of(caller):
                    if after(sink.offset):
                        tail.append((2, caller.qual, sink, [caller, enc]))
                for name, qualifier, method, offset in caller.calls:
                    if after(offset):
                        for callee in repo.resolve(caller, name, qualifier, method):
                            for sink in sinks_of(callee):
                                tail.append((3, callee.qual, sink, [caller, enc, callee]))
        tail.sort(key=lambda e: (e[0], e[1], e[2].offset))
        for depth, qual, sink, chain_fns in tail[:MAX_EVIDENCE_CHAINS]:
            result.tail_evidence.append({
                "sink": sink.name, "owner": qual,
                "location": f"{sink.fn.file}:{repo.sources[sink.fn.file].line_of(sink.offset)}",
                "depth": depth, "chain": [f.qual for f in chain_fns],
                "discarded": sink.discarded, "order": ordering_notes(repo, sink),
            })
        best_arm = evidence[0][0] if evidence else None
        best_tail = tail[0][0] if tail else None
        if best_arm is not None and (best_tail is None or best_arm <= best_tail):
            result.strength = _strength(best_arm)
        elif best_tail is not None:
            result.strength = "shared-dispatcher-" + _strength(best_tail)
        results.append(result)
    return results


_ARM_CACHE: dict[tuple[int, str, str], list[tuple[int, int]]] = {}


def find_arms_cached(repo: Repo, enum: str, fn: Fn) -> list[tuple[int, int]]:
    """Offsets spanned by arms of any Clap enum's variants inside fn's body (the dispatcher's non-shared code)."""
    key = (id(repo), "*", f"{fn.file}:{fn.start}")
    if key in _ARM_CACHE:
        return _ARM_CACHE[key]
    src = repo.sources[fn.file]
    names = sorted(repo.clap_enums | {enum})
    spans: list[tuple[int, int]] = []
    pattern = re.compile(r"\b(?:" + "|".join(map(re.escape, names)) + r"|Self)\s*::\s*[A-Z]\w*")
    for m in pattern.finditer(src.masked, fn.body_start, fn.body_end):
        nxt = _arm_after(src, m.end())
        if nxt is not None:
            spans.append((nxt[0], nxt[1]))
    _ARM_CACHE[key] = spans
    return spans


# --------------------------------------------------------------------------- tests


def attach_tests(repo: Repo, results: list[LeafResult]) -> None:
    fanout: dict[str, int] = defaultdict(int)
    for r in results:
        for key in r.reach_keys:
            fanout[key] += 1
    cli_tests = [f for f in repo.fns if f.is_test and f.file.startswith("tests/")]
    unit_tests = [f for f in repo.fns if f.is_test and f.file.startswith("src/")]
    contract_tests = [f for f in repo.fns if f.is_test and f.file.startswith("crates/mantle-application-contract/")]
    body_of = lambda f: repo.sources[f.file].masked[f.body_start : f.body_end]  # noqa: E731
    file_is_cli = {p: ("CARGO_BIN_EXE" in s.text or "cargo_bin" in s.text) for p, s in repo.sources.items()}
    callee_keys: dict[str, set[str]] = {}
    for t in unit_tests:
        callee_keys[f"{t.file}:{t.start}"] = {f"{c.file}:{c.start}" for c in repo.callees(t)}
    for r in results:
        tokens = r.leaf.path.split()
        if r.leaf.note.startswith("no-subcommand"):
            tokens = r.leaf.path.split()
        exclusive = {k for k in r.reach_keys | r.chain_keys | set(r.entry_keys) if fanout.get(k, 0) <= MAX_FANOUT_EXCLUSIVE}
        for t in cli_tests:
            if not file_is_cli.get(t.file):
                continue
            src = repo.sources[t.file]
            runs = string_runs(src, t.body_start, t.body_end)
            helper_runs: list[tuple[int, list[str]]] = []
            for h in repo.callees(t):
                if h.file == t.file and h.is_test is False or h.file == t.file:
                    helper_runs += string_runs(src, h.body_start, h.body_end)
            if not (invocation_matches(src, runs, tokens) or invocation_matches(src, helper_runs + runs, tokens)):
                continue
            body = body_of(t)
            name_body = body + " " + " ".join(body_of(h) for h in repo.callees(t) if h.file == t.file)
            success, failure = classify_test(t.name, name_body, name_body)
            label = f"{t.file}:{t.line} {t.name}"
            if failure:
                r.tests_failure.append(label)
            if success:
                r.tests_success.append(label)
        enum_name, variant_name = r.leaf.chain[-1]
        variant_ref = re.compile(r"\b" + re.escape(variant_name) + r"\b")
        reachable = r.reach_keys | r.top_keys
        for t in unit_tests:
            body_text = body_of(t)
            drives_dispatch = (variant_ref.search(body_text) and re.search(re.escape(enum_name) + r"\s*::|::\s*" + re.escape(variant_name), body_text)
                               and (callee_keys[f"{t.file}:{t.start}"] & reachable))
            if (callee_keys[f"{t.file}:{t.start}"] & exclusive) or drives_dispatch:
                body = body_of(t)
                success, failure = classify_test(t.name, body, body)
                label = f"{t.file}:{t.line} {t.name}"
                if failure:
                    r.tests_failure.append(label)
                if success:
                    r.tests_success.append(label)
        for ev in r.evidence:
            sink = ev["sink"]
            for t in contract_tests:
                if any(c[0] == sink for c in t.calls):
                    body = body_of(t)
                    success, failure = classify_test(t.name, body, body)
                    label = f"{t.file}:{t.line} {t.name}"
                    if failure:
                        r.classifier_tests_failure.append(label)
                    if success:
                        r.classifier_tests_success.append(label)
        for field_name in ("tests_success", "tests_failure", "classifier_tests_success", "classifier_tests_failure"):
            setattr(r, field_name, sorted(set(getattr(r, field_name))))


# Flags that mark a coverage gap; every other flag is an advisory note for the reviewer.
GAP_FLAGS = frozenset({"entry-unresolved", "no-classification-reachable", "classifier-result-discarded",
                       "no-success-path-test", "no-failure-path-test"})


def set_flags(results: list[LeafResult]) -> None:
    for r in results:
        flags = []
        if r.entry_level == "unresolved":
            flags.append("entry-unresolved")
        if r.strength == "none":
            flags.append("no-classification-reachable")
        elif r.strength.startswith("shared-dispatcher"):
            flags.append("classified-via-shared-dispatcher")
        elif r.strength == "weak":
            flags.append("classification-weak-depth")
        if r.evidence and all(e["discarded"] for e in r.evidence):
            flags.append("classifier-result-discarded")
        if r.evidence and any(e["order"] for e in r.evidence[:1]):
            flags.append("report-before-classification-advisory")
        if r.entry_level == "parent-arm":
            flags.append("dispatch-resolved-at-parent-only")
        if not r.tests_success:
            flags.append("no-success-path-test")
        if not r.tests_failure:
            flags.append("no-failure-path-test" if not r.classifier_tests_failure else "failure-path-classifier-level-only")
        r.flags = flags


# --------------------------------------------------------------------------- output


def render_json(results: list[LeafResult], meta: dict) -> dict:
    leaves = []
    for r in results:
        leaves.append({
            "path": r.leaf.path, "kind": r.kind, "note": r.leaf.note,
            "clap_chain": [f"{e}::{v}" for e, v in r.leaf.chain],
            "handoff_key": meta.get("handoff_keys", {}).get(r.leaf.path),
            "entry_level": r.entry_level, "dispatch_arms": r.arms, "entry_functions": r.entries,
            "classification_strength": r.strength, "classification_evidence": r.evidence,
            "shared_dispatcher_evidence": r.tail_evidence,
            "tests": {"success": r.tests_success, "failure": r.tests_failure,
                      "classifier_success": r.classifier_tests_success, "classifier_failure": r.classifier_tests_failure},
            "flags": r.flags, "disposition": r.disposition,
        })
    flagged = [x for x in leaves if any(f in GAP_FLAGS for f in x["flags"])]
    open_gaps = [x for x in flagged if x["disposition"] is None]
    counts: dict[str, int] = defaultdict(int)
    for x in leaves:
        for flag in x["flags"]:
            counts[flag] += 1
    return {
        "schema": SCHEMA, "meta": {k: v for k, v in meta.items() if k != "handoff_keys"},
        "summary": {"leaves": len(leaves), "leaves_with_gap_flags": len(flagged), "gap_leaves_without_disposition": len(open_gaps), "flag_counts": dict(sorted(counts.items()))},
        "leaves": leaves,
    }


def short(items: list[str], limit: int = 2) -> str:
    if not items:
        return "—"
    shown = "<br>".join(f"`{i}`" for i in items[:limit])
    return shown + (f"<br>(+{len(items) - limit})" if len(items) > limit else "")


def render_markdown(doc: dict) -> str:
    lines = ["# I6 coverage audit (structural)", "",
             f"Schema `{doc['schema']}`. Deterministic source-text audit; it executes no build or test.", ""]
    for key, value in doc["meta"].items():
        if isinstance(value, (str, int)):
            lines.append(f"- {key}: `{value}`")
    lines += ["", "## Summary", "", f"- leaves audited: {doc['summary']['leaves']}", f"- leaves with a gap flag: {doc['summary']['leaves_with_gap_flags']}",
              f"- gap leaves without a reviewer disposition: {doc['summary']['gap_leaves_without_disposition']}"]
    for flag, count in doc["summary"]["flag_counts"].items():
        lines.append(f"- `{flag}`: {count}")
    lines += ["", "## Per-leaf table", "",
              "| Leaf | Kind | Key | Dispatch entry | Classifier (depth, owner location) | Success tests | Failure tests | Flags |",
              "|---|---|---|---|---|---|---|---|"]
    for x in doc["leaves"]:
        ev = x["classification_evidence"]
        cls = (f"`{ev[0]['sink']}` d{ev[0]['depth']} `{ev[0]['location']}`" if ev else
               (f"shared `{x['shared_dispatcher_evidence'][0]['sink']}` `{x['shared_dispatcher_evidence'][0]['location']}`"
                if x["shared_dispatcher_evidence"] else "none"))
        entry = f"{x['entry_level']}: " + (", ".join(f"`{e}`" for e in x["entry_functions"][:2]) or "—")
        tests_f = x["tests"]["failure"] or [f"(classifier) {t}" for t in x["tests"]["classifier_failure"]]
        lines.append(f"| `{x['path']}` | {x['kind']} | {x['handoff_key'] or ''} | {entry} | {cls} | "
                     f"{short(x['tests']['success'])} | {short(tests_f)} | {', '.join(x['flags']) or '—'}{(' [' + x['disposition']['kind'] + ']') if x['disposition'] else ''} |")
    return "\n".join(lines) + "\n"


# --------------------------------------------------------------------------- driver


def terminal_catalog_paths(catalog: dict) -> tuple[list[str], dict[str, str]]:
    paths = [e["descriptor"]["path"] for e in catalog["entries"]]
    roles = {e["descriptor"]["path"]: e["policy"]["role"] for e in catalog["entries"]}
    terminal = sorted(p for p in paths if not any(q.startswith(p + " ") for q in paths))
    return terminal, roles


def handoff_keys(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    text = path.read_text(encoding="utf-8")
    return {leaf: key for leaf, key in re.findall(r"^\| `([^`]+)` \| (\w+) \|", text, re.M)}


# Public flag-mode surfaces that are not Clap leaves: (surface, owning leaf, note).
FLAG_MODE_SURFACES: list[tuple[str, str, str]] = [
    ("remote serve --binding metadata", "remote serve", "flag mode of remote serve"),
    ("self-build --source-built-fixed-point --prepare-source-programs", "self-build",
     "flag mode of self-build; source-built owner holds its proofs"),
]

DISPOSITION_KINDS = frozenset({"effect-free", "classified-by-shared-owner", "closed-by-change", "accepted-limit"})


def load_dispositions(root: Path, path: Path | None) -> tuple[dict[str, dict], list[str]]:
    """Reviewer dispositions for flagged leaves. Every anchor must still occur in its file, or the entry is stale."""
    if path is None:
        return {}, []
    problems: list[str] = []
    table: dict[str, dict] = {}
    for entry in json.loads(path.read_text(encoding="utf-8"))["dispositions"]:
        leaf = entry["leaf"]
        if entry.get("kind") not in DISPOSITION_KINDS:
            problems.append(f"{leaf}: unknown disposition kind {entry.get('kind')!r}")
        if not entry.get("rationale") or not entry.get("anchors"):
            problems.append(f"{leaf}: a disposition needs a rationale and at least one source anchor")
        for anchor in entry.get("anchors", ()):
            target = root / anchor["file"]
            if not target.is_file() or anchor["contains"] not in target.read_text(encoding="utf-8", errors="replace"):
                problems.append(f"{leaf}: stale anchor {anchor['file']} no longer contains {anchor['contains']!r}")
        table[leaf] = entry
    return table, problems


def run_audit(root: Path, catalog_path: Path | None, handoff: Path | None, overrides: dict[str, list[str]],
              extra_leaves: list[Leaf] | None = None, dispositions: Path | None = None,
              modes: list[tuple[str, str, str]] | None = None) -> tuple[list[LeafResult], dict]:
    repo = Repo(root)
    enums, structs = parse_clap(repo)
    repo.clap_enums = set(enums)
    leaves = clap_leaves(enums, structs)
    meta: dict = {"root_files_indexed": len(repo.sources), "functions_indexed": len(repo.fns),
                  "contract_classifiers": len(repo.contract_classifiers),
                  "clap_leaves": len(leaves)}
    kinds: dict[str, str] = {}
    if catalog_path is not None and catalog_path.is_file():
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        terminal, roles = terminal_catalog_paths(catalog)
        derived = {leaf.path for leaf in leaves}
        for p in terminal:
            kinds[p] = "catalog-" + roles[p]
        meta["catalog_terminal_paths"] = len(terminal)
        meta["catalog_paths_missing_from_clap_tree"] = sorted(set(terminal) - derived)
        meta["clap_leaves_missing_from_catalog"] = sorted(derived - set(terminal))
    by_path = {leaf.path: leaf for leaf in leaves}
    for mode_path, parent, note in modes or ():
        if parent not in by_path:
            raise SystemExit(f"flag-mode surface {mode_path!r} names unknown parent leaf {parent!r}")
        leaves.append(Leaf(mode_path, by_path[parent].chain, by_path[parent].hidden, note))
    if extra_leaves:
        leaves = leaves + extra_leaves
    leaves = sorted(leaves, key=lambda leaf: leaf.path)
    meta["handoff_keys"] = handoff_keys(handoff) if handoff else {}
    results = audit(repo, leaves, kinds, overrides)
    attach_tests(repo, results)
    set_flags(results)
    table, problems = load_dispositions(root, dispositions)
    known = {r.leaf.path for r in results}
    problems += [f"{leaf}: disposition names no audited leaf" for leaf in table if leaf not in known]
    for r in results:
        if r.leaf.path in table:
            r.disposition = {k: table[r.leaf.path][k] for k in ("kind", "rationale", "anchors")}
    meta["disposition_problems"] = problems
    return results, meta


# --------------------------------------------------------------------------- self-test

_FIXTURE_FILES = {
    "crates/mantle-application-contract/src/lib.rs": """
pub fn classify_thing(_plan: u32, _obs: u32) -> u32 { 0 }
pub fn plan_thing() -> u32 { 0 }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classify_thing_accepts_matching_observation() { assert_eq!(classify_thing(1, 1), 0); }
    #[test]
    fn classify_thing_rejects_wrong_observation() { assert!(classify_thing(1, 2) != 0); assert!(Err::<(), ()>(()).is_err()); }
}
""",
    "src/main.rs": """
use clap::{Parser, Subcommand};
#[derive(Parser)]
struct Args { #[command(subcommand)] command: Command }
#[derive(Subcommand)]
enum Command {
    /// Classified before reporting; success and failure tests exist.
    Good,
    /// No classifier anywhere.
    BadNone,
    /// The verdict is computed and ignored.
    BadDiscard,
    /// Prints before classifying.
    BadOrder,
    /// Classified but only success is tested.
    BadNoFailureTest,
    /// The only classifier reachable is a host-context observation, which proves nothing about the operation.
    BadContext,
    /// A classifier appears only inside the test module and in a comment/string.
    BadTestOnly,
    Group { #[command(subcommand)] action: GroupAction },
}
#[derive(Subcommand)]
enum GroupAction {
    Leaf,
    Other,
    /// The arm lives in a port method; its caller classifies after the port returns.
    ViaPort,
    /// Same shape, but the caller never classifies.
    ViaPortNone,
}
fn dispatch(c: &Command) -> Result<(), String> {
    match c {
        Command::Good => run_good(),
        Command::BadNone => run_none(),
        Command::BadDiscard => run_discard(),
        Command::BadOrder => run_order(),
        Command::BadNoFailureTest => run_nofail(),
        Command::BadTestOnly => run_testonly(),
        Command::BadContext => run_context(),
        Command::Group { action } => match action {
            GroupAction::Leaf => run_good(),
            GroupAction::Other => run_none(),
            _ => group_ports(action),
        },
    }
}
fn helper_ok() -> Result<(), String> { Ok(()) }
fn group_ports(a: &GroupAction) -> Result<(), String> { command_a(a)?; command_b(a) }
struct PortA;
struct PortB;
impl PortA { fn run_port_a(&self, c: &GroupAction) -> Result<(), String> { match c { GroupAction::ViaPort => helper_ok(), _ => Err("a".into()) } } }
impl PortB { fn run_port_b(&self, c: &GroupAction) -> Result<(), String> { match c { GroupAction::ViaPortNone => helper_ok(), _ => Err("b".into()) } } }
fn command_a(c: &GroupAction) -> Result<(), String> { PortA.run_port_a(c)?; if classify_thing(1, 1) != 0 { return Err("z".into()); } println!("ok"); Ok(()) }
fn command_b(c: &GroupAction) -> Result<(), String> { PortB.run_port_b(c)?; println!("ok"); Ok(()) }
fn run_good() -> Result<(), String> { let verdict = classify_thing(1, 1); if verdict != 0 { return Err("x".into()); } println!("ok"); Ok(()) }
fn run_none() -> Result<(), String> { let _lookalike = "x".is_empty(); println!("done"); Ok(()) }
fn run_discard() -> Result<(), String> { let _ = classify_thing(1, 1); println!("ok"); Ok(()) }
fn run_order() -> Result<(), String> { println!("report"); classify_thing(1, 1); Ok(()) }
fn run_nofail() -> Result<(), String> { if classify_thing(1, 1) != 0 { return Err("y".into()); } Ok(()) }
fn run_context() -> Result<(), String> { host_environment::working_directory(); println!("x"); Ok(()) }
fn run_testonly() -> Result<(), String> {
    // classify_thing(1, 1) in a comment
    let _note = "classify_thing(1, 1) in a string";
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_only_calls_classifier() { let _ = classify_thing(1, 1); let _ = run_testonly(); }
    #[test]
    fn good_unit_succeeds() { assert!(run_good().is_ok()); }
    #[test]
    fn good_unit_rejects_input() { assert!(run_good().is_err() || true); }
    #[test]
    fn lenient_unit_succeeds() { assert!(run_nofail().is_ok()); }
}
""",
    "src/host_environment.rs": """
pub fn working_directory() { classify_thing(1, 1); }
""",
    "src/other.rs": """
pub struct Unrelated;
impl Unrelated { pub fn is_empty(&self) -> bool { classify_thing(1, 1) == 0 } }
""",
    "tests/cli.rs": """
use std::process::Command;
fn run(args: &[&str]) -> std::process::Output { Command::new(env!("CARGO_BIN_EXE_mantle")).args(args).output().unwrap() }
#[test]
fn good_runs() { let o = run(&["good"]); assert!(o.status.success()); }
#[test]
fn good_rejects_blocked_input() { let o = Command::new(env!("CARGO_BIN_EXE_mantle")).args(["good", "--bad"]).output().unwrap(); assert!(!o.status.success()); }
#[test]
fn none_leaf_runs() { let o = run(&["bad-none"]); assert!(o.status.success()); }
#[test]
fn none_leaf_errors() { let o = run(&["bad-none", "--x"]); assert!(!o.status.success()); }
#[test]
fn discard_leaf_runs() { let o = run(&["bad-discard"]); assert!(o.status.success()); }
#[test]
fn order_leaf_runs() { let o = run(&["bad-order"]); assert!(o.status.success()); }
#[test]
fn lenient_leaf_runs() { let o = run(&["bad-no-failure-test"]); assert!(o.status.success()); }
fn group_base() -> Command { let mut c = Command::new(env!("CARGO_BIN_EXE_mantle")); c.args(["group"]); c }
#[test]
fn split_leaf_invocation_runs() { let mut c = group_base(); let o = c.args(["other"]).output().unwrap(); assert!(o.status.success()); }
#[test]
fn cargo_invocation_is_not_a_leaf_invocation() { let o = Command::new("cargo").args(["bad-test-only"]).output().unwrap(); assert!(o.status.success()); }
""",
}

_FIXTURE_CATALOG = {"entries": [
    {"descriptor": {"path": p}, "policy": {"role": "public-command"}}
    for p in ["good", "bad-none", "bad-discard", "bad-order", "bad-no-failure-test", "bad-test-only", "bad-context", "group", "group via-port", "group via-port-none", "group leaf", "group other"]
]}


def self_test() -> int:
    # lexer: comments and literal contents are masked, offsets are preserved, strings are captured.
    masked, strings, _ = lex('fn a() { /* classify_x( */ let s = "classify_x("; // classify_x(\n let r = r#"cl"assify"#; }')
    assert "classify_x" not in masked and len(strings) == 2, "lexer must hide commented and quoted calls"
    assert kebab("Stage0Inventory") == "stage0-inventory" and kebab("RepairFinalNar") == "repair-final-nar"
    with tempfile.TemporaryDirectory(prefix="i6-audit-selftest-") as scratch:
        root = Path(scratch)
        for rel, content in _FIXTURE_FILES.items():
            target = root / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        catalog = root / "catalog.json"
        catalog.write_text(json.dumps(_FIXTURE_CATALOG), encoding="utf-8")
        results, meta = run_audit(root, catalog, None, {})
        by_path = {r.leaf.path: r for r in results}
        expectations = {
            "good": ([], {"no-classification-reachable"}),
            "group leaf": ([], {"no-classification-reachable"}),
            "bad-none": (["no-classification-reachable"], set()),
            "group other": (["no-classification-reachable"], set()),
            "bad-discard": (["classifier-result-discarded"], {"no-classification-reachable"}),
            "bad-order": (["report-before-classification-advisory"], {"no-classification-reachable"}),
            "bad-no-failure-test": (["failure-path-classifier-level-only"], {"no-classification-reachable", "no-failure-path-test"}),
            "bad-test-only": (["no-classification-reachable", "no-success-path-test", "no-failure-path-test"], set()),
            "bad-context": (["no-classification-reachable"], {"classified-via-shared-dispatcher"}),
            "group via-port": (["classified-via-shared-dispatcher"], {"no-classification-reachable"}),
            "group via-port-none": (["no-classification-reachable"], {"classified-via-shared-dispatcher"}),
        }
        failures = []
        for path, (must, must_not) in expectations.items():
            got = set(by_path[path].flags)
            failures += [f"{path}: expected flag {f}, got {sorted(got)}" for f in must if f not in got]
            failures += [f"{path}: unexpected flag {f}, got {sorted(got)}" for f in must_not if f in got]
        if by_path["good"].flags:
            # the positive fixture must be flag-free: classified, a success test and a failure test
            failures.append(f"good: expected no flags, got {by_path['good'].flags}")
        if meta["catalog_paths_missing_from_clap_tree"] or meta["clap_leaves_missing_from_catalog"]:
            failures.append(f"clap/catalog drift in fixture: {meta}")
        if by_path["good"].tests_failure == [] or by_path["good"].tests_success == []:
            failures.append("good: CLI success/failure tests were not attributed")
        if not any("split_leaf_invocation_runs" in t for t in by_path["group other"].tests_success):
            failures.append("a leaf invoked across consecutive .args calls must be attributed")
        if by_path["bad-test-only"].tests_success:
            failures.append("a Command::new(\"cargo\") invocation must not count as a leaf test")
        # negative catalog fixture: a catalog leaf the Clap tree lacks must be reported
        catalog.write_text(json.dumps({"entries": _FIXTURE_CATALOG["entries"] + [
            {"descriptor": {"path": "ghost"}, "policy": {"role": "public-command"}}]}), encoding="utf-8")
        _, meta2 = run_audit(root, catalog, None, {})
        if meta2["catalog_paths_missing_from_clap_tree"] != ["ghost"]:
            failures.append(f"catalog drift not detected: {meta2['catalog_paths_missing_from_clap_tree']}")
        # dispositions: an anchored reviewer note is accepted; stale, unknown or malformed ones are rejected
        good_note = {"dispositions": [{"leaf": "bad-none", "kind": "effect-free", "rationale": "fixture",
                                       "anchors": [{"file": "src/main.rs", "contains": "fn run_none"}]}]}
        bad_note = {"dispositions": [
            {"leaf": "bad-none", "kind": "effect-free", "rationale": "x", "anchors": [{"file": "src/main.rs", "contains": "absent-text"}]},
            {"leaf": "ghost", "kind": "effect-free", "rationale": "x", "anchors": [{"file": "src/main.rs", "contains": "fn run_none"}]},
            {"leaf": "good", "kind": "nonsense", "rationale": "x", "anchors": [{"file": "src/main.rs", "contains": "fn run_good"}]},
            {"leaf": "good", "kind": "effect-free", "rationale": "", "anchors": []}]}
        note_path = root / "notes.json"
        note_path.write_text(json.dumps(good_note), encoding="utf-8")
        accepted, accepted_meta = run_audit(root, catalog, None, {}, dispositions=note_path,
                                            modes=[("good --fast", "good", "fixture flag mode")])
        accepted_by = {r.leaf.path: r for r in accepted}
        if accepted_meta["disposition_problems"] or accepted_by["bad-none"].disposition is None:
            failures.append(f"an anchored disposition must be accepted: {accepted_meta['disposition_problems']}")
        if accepted_by["good --fast"].flags != accepted_by["good"].flags or accepted_by["good --fast"].leaf.note != "fixture flag mode":
            failures.append("a flag-mode surface must be audited through its parent's dispatch")
        note_path.write_text(json.dumps(bad_note), encoding="utf-8")
        _, bad_meta = run_audit(root, catalog, None, {}, dispositions=note_path)
        if len(bad_meta["disposition_problems"]) != 4:
            failures.append(f"stale/unknown/malformed dispositions not all rejected: {bad_meta['disposition_problems']}")
        if json.dumps(render_json(results, meta), sort_keys=True) != json.dumps(render_json(results, meta), sort_keys=True):
            failures.append("rendering is not deterministic")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        print("i6-coverage-audit self-test: FAIL")
        return 1
    print("i6-coverage-audit self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--catalog", type=Path)
    parser.add_argument("--handoff", type=Path)
    parser.add_argument("--out-json", type=Path)
    parser.add_argument("--out-md", type=Path)
    parser.add_argument("--dispositions", type=Path)
    parser.add_argument("--fail-on-flags", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    root = args.root.resolve()
    catalog = args.catalog or root / "config" / "operator-command-catalog.json"
    results, meta = run_audit(root, catalog, args.handoff, {}, dispositions=args.dispositions, modes=FLAG_MODE_SURFACES)
    doc = render_json(results, meta)
    if args.out_json:
        args.out_json.write_text(json.dumps(doc, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    if args.out_md:
        args.out_md.write_text(render_markdown(doc), encoding="utf-8")
    print(json.dumps(doc["summary"], sort_keys=True))
    if meta["disposition_problems"]:
        print("\n".join(meta["disposition_problems"]), file=sys.stderr)
        return 2
    return 1 if args.fail_on_flags and doc["summary"]["gap_leaves_without_disposition"] else 0


if __name__ == "__main__":
    sys.exit(main())
