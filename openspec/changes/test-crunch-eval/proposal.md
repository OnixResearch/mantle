## Why

crunch-eval has 8 unit tests, all in `lib.rs`, all exercising `evaluate_str`
on inline Nickel snippets. None of these test:

- **File-based evaluation** (`evaluate()`, `evaluate_to_json()`) — the path
  the CLI actually uses. Import resolution, parent-directory scoping, and
  `OsString` path handling are untested.
- **Stdlib integration** — `stdlib_import_path()`, `write_stdlib()`,
  `source_stdlib_dir()`. The stdlib is included at compile time and
  extracted at runtime. No test verifies the extraction round-trip, the
  cache-skip logic, or that the stdlib files are importable.
- **`evaluate_and_deserialize` / `evaluate_str_and_deserialize`** — the JSON
  round-trip path. These go through `expr_to_json` + `serde_json::from_str`,
  which differs from direct `to_serde()`. Enum tag handling differs between
  the two paths (the `NickelString` wrapper exists precisely because of this).
- **Error paths** — missing files, syntax errors, typecheck failures, contract
  violations that produce multi-line diagnostics. The single
  `eval_contract_violation` test just checks `is_err()` without inspecting
  the error variant.

This is a 227-line crate. The untested surface is larger than the tested
surface.

## What Changes

- Add tests for file-based evaluation (tempfile `.ncl` files).
- Add tests for stdlib write/extract/import-path resolution.
- Add tests for the JSON round-trip deserialization path.
- Add error-path tests with variant assertions.

## Capabilities

### New Capabilities
- `test-eval-file`: Verify `evaluate()` reads `.ncl` files, resolves
  imports from the parent directory, and respects extra import paths.
- `test-eval-stdlib`: Verify stdlib extraction writes all files, skips
  re-writes when unchanged, and produces importable Nickel code.
- `test-eval-json-roundtrip`: Verify `evaluate_and_deserialize` and
  `evaluate_str_and_deserialize` handle records, enums, nested structures.
- `test-eval-errors`: Verify error variants for missing files, parse errors,
  typecheck failures, deserialization failures.

## Impact

- **Files**: `crates/crunch-eval/src/lib.rs` (tests added), `crates/crunch-eval/src/stdlib.rs` (tests added)
- **APIs**: None changed
- **Dependencies**: `tempfile` as dev-dependency for crunch-eval
- **Testing**: `cargo test -p crunch-eval`
