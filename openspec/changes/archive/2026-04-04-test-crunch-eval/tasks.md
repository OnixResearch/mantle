## Phase 1: File-based evaluation

- [x] Add `tempfile` dev-dependency to crunch-eval
- [x] Test `evaluate()` on a simple `.ncl` file returns expected Expr ✅ eval_file_simple_record
- [x] Test `evaluate()` resolves imports from parent directory (two-file import) ✅ eval_file_resolves_import_from_parent_dir
- [x] Test `evaluate()` with extra import paths resolves cross-directory imports ✅ eval_file_extra_import_path
- [x] Test `evaluate()` on nonexistent file returns `Error::Io` ✅ eval_file_nonexistent_returns_io_error
- [x] Test `evaluate_to_json()` returns valid JSON for a record ✅ eval_file_to_json_returns_valid_json

## Phase 2: Stdlib

- [x] Test `write_stdlib(Some(tmpdir))` creates all expected files ✅ write_stdlib_creates_all_files
- [x] Test `write_stdlib` skips rewrite when content unchanged (check mtime) ✅ write_stdlib_skips_rewrite_when_unchanged
- [x] Test `stdlib_import_path()` returns a dir containing `lib.ncl` ✅ stdlib_import_path_returns_dir_with_lib_ncl
- [x] Test that stdlib `lib.ncl` is importable via `evaluate_str` with the stdlib path ✅ stdlib_is_importable

## Phase 3: JSON round-trip deserialization

- [x] Test `evaluate_str_and_deserialize` on a flat record ✅ eval_str_and_deserialize_flat_record
- [x] Test `evaluate_str_and_deserialize` with enum tags (the JSON path handles these) ✅ eval_str_and_deserialize_enum_tags
- [x] Test `evaluate_str_and_deserialize` with nested records ✅ eval_str_and_deserialize_nested_records
- [x] Test `evaluate_str_and_deserialize` returns `Error::Serde` on type mismatch ✅ eval_str_and_deserialize_type_mismatch_returns_serde_error

## Phase 4: Error paths

- [x] Test `evaluate_str` with syntax error returns `Error::Eval` ✅ eval_str_syntax_error_returns_eval
- [x] Test `evaluate_str` with typecheck failure returns `Error::Eval` ✅ eval_str_typecheck_failure_returns_eval
- [x] Test `evaluate_and_deserialize` on a non-record returns `Error::Serde` ✅ eval_and_deserialize_non_record_returns_serde
- [x] Test `Error::Eval` Display output contains useful context ✅ eval_error_display_has_context
