## Why

crunch-build is 1,436 lines across three files. The existing tests cover:

- `build_request.rs`: 5 tests for `derivation_to_build_request()` — builder
  args, sandbox env, constraints, relative output paths, refscan needles.
- `orchestrate.rs`: 20+ tests for `verify_fod_hash()` and `nar_hash()` —
  flat/NAR/text mode, all hash algorithms, mismatch detection.

Nothing tests the `Builder` struct or the orchestration logic:

- **Cache checking** — `all_outputs_exist()`, `load_cached_outputs()`. The
  build pipeline spec requires skipping builds when outputs exist. Untested.
- **Recursive dependency building** — `build_derivation_inner()` walks
  `input_derivations` recursively. No test verifies that inputs are built
  before dependents, or that diamond deps build once.
- **Source input validation** — the spec requires failing with a clear error
  when a source path is missing. `SourceNotFound` is defined but untested.
- **`collect_input_paths()`** — wires input_sources and input_derivation
  outputs into the sandbox path set. Has no tests.
- **`resolve_references()`** — maps refscan needle indices back to store
  paths. Pure function, zero tests.
- **Placeholder replacement** — `replace_placeholders()` and
  `replace_placeholders_bstr()` substitute `hash_placeholder(output)` with
  actual paths. Zero tests.

The Builder itself can't be unit-tested without a mock `BuildService`, but
the pure functions it calls can.

## What Changes

- Add unit tests for `collect_input_paths()`, `resolve_references()`,
  `replace_placeholders()`, and `replace_placeholders_bstr()`.
- Add tests for `all_outputs_exist()` / `path_exists_on_disk()` behavior
  (these check filesystem state — use tempdir).
- Add a mock `BuildService` for testing `Builder.build()` against
  synthetic derivation graphs.

## Capabilities

### New Capabilities
- `test-collect-inputs`: Verify input path collection for source-only,
  derivation-only, and mixed input graphs.
- `test-resolve-references`: Verify needle-to-store-path mapping for
  self-references, input references, and out-of-range indices.
- `test-replace-placeholders`: Verify placeholder substitution in both
  String and BString variants, including no-op (no placeholders) and
  multi-output cases.
- `test-builder-cache`: Verify Builder skips builds when outputs exist.
- `test-builder-mock`: Verify Builder calls BuildService in dependency
  order using a mock.

## Impact

- **Files**: `crates/crunch-build/src/build_request.rs` (tests added),
  `crates/crunch-build/src/orchestrate.rs` (tests added)
- **APIs**: `replace_placeholders` and `resolve_references` may need
  `pub(crate)` visibility if currently private
- **Dependencies**: None new (tokio already a dev-dep)
- **Testing**: `cargo test -p crunch-build`
