# Change: Surface foreign realization diagnostics

## Why

Two operator-visible diagnostics disappear after a `foreign-import realize` run.

The HTTP NAR ingestion behind cache-inputs hydration (`crates/crunch-store/src/pull.rs`,
`ingest_http_nar`) returns `Ok(None)` for six different refusals and only logs the reason through
`tracing::warn!`. With `--json` tracing is uninitialized, so the strict closure import reports
`http-closure-content-admission-failed: <path>` with no cause. In the Octet pilot a full ZFS pool
made the rust-docs NAR fail and the operator had no way to see why.

`foreign-import realize` never persists a build log. `mantle build` writes every outcome's captured
log to `<state-dir>/logs/<drv>.log` and `mantle log` reads it, but a successful realization keeps no
log at all, so the Octet job's CPU-affinity line cannot be quoted after the run.

## What Changes

- Return the HTTP NAR refusal reason as a value: `ingest_http_nar` yields
  `Result<NarIngestion, Error>` where `NarIngestion` is `Ingested(IngestedNar)` or
  `Refused(NarRefusal)`, and `NarRefusal` carries a stable kebab-case refusal class plus a detail
  string bounded by a named `MAX_*` byte limit and truncated on a UTF-8 boundary.
- The strict seeded closure import reports
  `http-closure-content-admission-failed: <path>: <class>: <detail>`; the flat HTTP pull path keeps
  skipping and counting refusals exactly as it does today. The existing `tracing::warn!` lines stay.
- After `crunch_pipeline::build_registered_derivations` returns, `src/foreign_realization_shell.rs`
  writes the log of every outcome that has one and of every failed root that has one through the
  existing `build_log`/`build_cmd::write_log` writer, with the same `<state-dir>/logs/<drv>.log`
  name and header format as `mantle build`. Cached or reused outcomes write nothing. A log-write
  failure is reported as a diagnostic persistence failure and never changes the realization result
  or the receipt. Receipt schemas do not change.

## Impact

- **Files**: `crates/crunch-store/src/pull.rs`, `src/foreign_realization_shell.rs`,
  `src/foreign_import_cmd.rs`, `src/build_cmd.rs` (visibility of the existing warning printer),
  `tests/cache_inputs_build_route.rs`, `tests/fixtures/foreign-import/realize-two-node-builder.sh`.
- **Testing**: `cargo test -p crunch-store --lib` (refusal class/detail and unchanged skip path),
  `cargo test -p mantle --test cache_inputs_build_route` (log persistence and reuse),
  `cargo test -p mantle --bin mantle -- foreign_`, fmt and strict Clippy for `-p mantle`,
  `-p crunch-store`, `-p crunch-build`, `-p crunch-pipeline`.
- **Non-goals**: changing receipt v1 or v2 fields, changing the flat pull's counters, adding a
  second log format, changing `mantle log`, or changing which routes hydrate.
