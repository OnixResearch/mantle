## Why

`main.rs` is 1335 lines. It is not a thin CLI shell — it implements:

- `execute_builds_streaming()`: 160-line async function that creates
  KnownPaths, calls crunch_glue::convert, creates mpsc channels, feeds
  them to Worker::run_streaming, processes results, handles FOD mismatches,
  formats output
- `deserialize_derivations_from_json()`: decides if JSON is a single
  derivation or package set, deserializes CrunchDerivation structs
- `open_pathinfo_service()`, `open_blob_service()`, `build_remote_pathinfo()`:
  constructs concrete service implementations, wires them together
- `handle_fod_mismatch()`, `auto_fix_hash()`: FOD error handling with
  source rewriting
- `write_log()`, `cmd_log()`: log persistence and retrieval
- `state_dir()`, `log_dir()`: XDG state resolution

If you want to embed crunch as a library — in a CI system, an LSP, a REPL,
a test harness — you have to copy-paste from main.rs. The binary crate's
`lib.rs` is 4 lines that re-export nothing useful.

snix has `snix-glue` as the wiring layer. The CLI calls into it. crunch
has no equivalent.

## What Changes

Extract a `crunch-pipeline` (or `crunch-driver`) crate that owns the
eval→build→realize pipeline:

1. **Derivation deserialization** — single vs package-set detection,
   JSON→CrunchDerivation
2. **Pipeline orchestration** — eval, convert, stream to Worker, collect
   results
3. **FOD error handling** — hash mismatch detection, `--fix` rewriting
4. **Log management** — build log persistence and retrieval
5. **State directory resolution** — XDG paths, env overrides

The binary crate becomes: parse args → call `pipeline::build(file, opts)` →
format output. Under 200 lines.

## Capabilities

### New Capabilities
- `crunch-pipeline`: embeddable library for the full eval→build→realize flow
- `pipeline::build()`: single entry point for the build command
- `pipeline::eval()`: single entry point for the eval command
- `BuildResult`: structured result type (outputs, logs, failures)

### Modified Capabilities
- `main.rs`: thin CLI shell, delegates all logic to crunch-pipeline
- `src/lib.rs`: re-exports crunch-pipeline types for library consumers

## Impact

- **Files**: new `crates/crunch-pipeline/`, gutted `src/main.rs`
- **APIs**: new public API for library embedding
- **Dependencies**: crunch-pipeline depends on crunch-eval, crunch-glue,
  crunch-build, crunch-store
- **Testing**: pipeline integration tests move from `tests/` to the new
  crate, independently runnable
