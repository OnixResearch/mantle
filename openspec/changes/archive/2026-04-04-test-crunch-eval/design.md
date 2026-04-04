## Context

crunch-eval wraps `nickel_lang::Context` with two evaluation paths:

1. **Direct**: `evaluate()` / `evaluate_str()` → `Expr` → `.to_serde::<T>()`
2. **JSON round-trip**: `evaluate_to_json()` / `evaluate_str_to_json()` →
   JSON string → `serde_json::from_str::<T>()`

Path 2 exists because Nickel enum tags don't serialize to plain strings via
`to_serde()` — they need the JSON export layer. The `CrunchDerivation`
type uses the `NickelString` serde wrapper to accept both, but other
types go through path 2.

The stdlib module (`stdlib.rs`) includes `.ncl` files at compile time and
extracts them to `$XDG_CACHE_HOME/crunch/stdlib/`. It has a cache-skip
optimization (don't rewrite if content unchanged). None of this is tested.

## Goals / Non-Goals

**Goals:** Cover the two eval paths, stdlib I/O, import resolution, and all
error variants.

**Non-Goals:** Don't re-test Nickel language semantics (merging, typing,
pattern matching) — that's nickel-lang's job. Don't test `CrunchDerivation`
deserialization — that's crunch-glue's territory.

## Decisions

### 1. Use tempdir for file-based tests

**Choice:** Create temporary directories with `.ncl` files for `evaluate()`
and import-path tests.

**Rationale:** `evaluate()` reads from disk and resolves imports relative to
the file's parent. We need real files to test this path.

**Alternative:** Mock the filesystem. Rejected — crunch-eval delegates to
nickel_lang which does its own file I/O. Can't intercept it.

### 2. Test stdlib extraction in a temp directory

**Choice:** Call `write_stdlib(Some(tmpdir))` and verify all files exist
with correct content.

**Rationale:** The default path (`write_stdlib(None)`) writes to
`$XDG_CACHE_HOME` which is global state. Using `Some(tmpdir)` isolates
the test.

### 3. Error variant assertions, not just is_err()

**Choice:** Match on `Error::Io`, `Error::Eval`, `Error::Serde` specifically.

**Rationale:** A test that just checks `is_err()` passes for the wrong reason
if the error type changes. Matching the variant documents the expected
failure mode.

## Risks / Trade-offs

**[Nickel API changes]** → `Context::eval_deep_for_export` and
`Context::expr_to_json` are stable Nickel API. Low risk.

**[Stdlib content changes]** → Tests that assert specific stdlib file names
(`lib.ncl`, `contracts.ncl`, etc.) break if we add/rename files. Acceptable —
we'd want the test to flag that.
