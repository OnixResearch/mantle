## Phase 1: File-based evaluation

- [ ] Add `tempfile` dev-dependency to crunch-eval
- [ ] Test `evaluate()` on a simple `.ncl` file returns expected Expr
- [ ] Test `evaluate()` resolves imports from parent directory (two-file import)
- [ ] Test `evaluate()` with extra import paths resolves cross-directory imports
- [ ] Test `evaluate()` on nonexistent file returns `Error::Io`
- [ ] Test `evaluate_to_json()` returns valid JSON for a record

## Phase 2: Stdlib

- [ ] Test `write_stdlib(Some(tmpdir))` creates all expected files
- [ ] Test `write_stdlib` skips rewrite when content unchanged (check mtime or call twice)
- [ ] Test `stdlib_import_path()` returns a dir containing `lib.ncl`
- [ ] Test that stdlib `lib.ncl` is importable via `evaluate_str` with the stdlib path

## Phase 3: JSON round-trip deserialization

- [ ] Test `evaluate_str_and_deserialize` on a flat record
- [ ] Test `evaluate_str_and_deserialize` with enum tags (the JSON path handles these)
- [ ] Test `evaluate_str_and_deserialize` with nested records
- [ ] Test `evaluate_str_and_deserialize` returns `Error::Serde` on type mismatch

## Phase 4: Error paths

- [ ] Test `evaluate_str` with syntax error returns `Error::Eval`
- [ ] Test `evaluate_str` with typecheck failure returns `Error::Eval`
- [ ] Test `evaluate_and_deserialize` on a non-record returns `Error::Serde`
- [ ] Test `Error::Eval` Display output contains useful context (not just "Nickel evaluation error")
