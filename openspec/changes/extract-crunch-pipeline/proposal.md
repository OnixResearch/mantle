# Extract crunch-pipeline crate

## Why

`main.rs` is 1153 lines. Of those, ~600 lines are pipeline orchestration
that has nothing to do with CLI argument parsing: `execute_builds_streaming`
(187 lines), `execute_builds` (135 lines, dead-code fallback),
`deserialize_derivations_from_json`, `handle_fod_mismatch`, `auto_fix_hash`,
`parse_fod_mismatch_error`, `write_log`, `state_dir`, `log_dir`,
`build_import_paths`, `resolve_max_jobs`.

This code is untestable as-is. The `#[cfg(target_os = "linux")]` blocks
in `execute_builds_streaming` mean the pipeline can only be exercised
through the binary. `cmd_self_build` partially reimplements the pipeline.
Future consumers (daemon mode, `crunch check --dry-run`, IDE integration)
would have to reach into the binary or duplicate logic.

The architecture spec says "each stage MUST be a separate module/crate with
a defined interface between them." Right now the binary owns the glue
between stages, which makes it the biggest and least-tested crate.

## What Changes

- New `crunch-pipeline` crate in `crates/crunch-pipeline/`.
- Owns: eval dispatch, JSON deserialization, convert loop, registry
  bridging, store+builder construction, streaming Worker invocation,
  result collection, and FOD mismatch detection as returned data.
- The binary keeps: CLI args (`clap`), stderr formatting, exit codes,
  log writing, `--fix` source rewriting, bootstrap staging, and self-build
  verification.
- `main.rs` becomes a thin shell for the standard build path: parse args,
  call `crunch_pipeline::build()`, then format and print the result.
- `cmd_self_build` delegates the actual build to `crunch_pipeline::build()`
  instead of reimplementing store/builder/worker setup.

## Capabilities

### New Capabilities
- `pipeline-crate`: The eval->convert->build pipeline is a library crate
  with a documented entry point.
- `pipeline-testable`: Pipeline stages can be integration-tested without
  invoking the binary.
- `pipeline-reusable`: `cmd_self_build` and future commands share one
  pipeline implementation.

### Modified Capabilities
- `cli-thin-shell`: The crunch binary becomes argument parsing + error
  formatting. No business logic.

## Impact

- **Files**: New crate `crates/crunch-pipeline/`. `src/main.rs` shrinks
  from ~1150 to ~300 lines. `src/self_build.rs` shrinks (delegates to
  pipeline). `Cargo.toml` gets a new workspace member + dependency.
- **APIs**: Public `crunch_pipeline::build()` function. Public
  `BuildConfig` struct replacing the 9-parameter function signature.
  Public `PipelineResult` aggregating outcomes + failures.
- **Dependencies**: crunch-pipeline depends on crunch-eval, crunch-glue,
  crunch-build, crunch-store. The binary uses crunch-pipeline for the
  standard build path and keeps direct dependencies that remain necessary
  for `eval`, `bootstrap`, and `store` helpers.
- **Testing**: New integration tests in crunch-pipeline exercising the
  full eval->build path on trivial .ncl files.
