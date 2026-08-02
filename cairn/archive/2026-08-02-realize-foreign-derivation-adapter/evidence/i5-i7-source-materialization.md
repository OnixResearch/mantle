# I5-I7 source materialization and fetch policy

Task-ID: I5, I6, I7
Covers: r[foreign_derivation_import.source_materialization]
Date: 2026-08-02

## Implementation

- `src/foreign_realization.rs` admits source records without I/O.
- `src/source_bundle.rs` binds plan payload IDs to bounded source records.
- `crates/crunch-store/src/handle.rs` ingests verified payloads at exact logical paths.
- Store ingest signs `PathInfo`, permits exact reuse, and rejects conflicting content.
- `src/foreign_realization_shell.rs` retains admitted payloads for offline fixed-output fetch overrides.
- `crates/crunch-build/src/fetch_build_service.rs` preserves ordered candidates and structured attempt classes.
- Failed output admission retains the bounded fetch log and its originating derivation.

## Positive evidence

- Regular files preserve bytes and executable mode.
- Directory trees preserve files and safe relative symbolic links.
- Exact source ingest is idempotent.
- Ordered fallback selects the first available admitted source candidate.
- File, tarball, executable, and Git override kinds remain distinct.

## Negative evidence

- Missing, duplicate, incomplete, stale, and unexpected source records fail admission.
- Manifest, descriptor, target-path, content, and mode changes fail before source ingest.
- Conflicting existing `PathInfo` is not replaced.
- Fixed-output mismatch fails after a `selected-source-state` attempt and emits partial evidence.
- Unmatched offline candidates fail before network access.
- Git overrides require the exact revision.

## Claim boundary

Source admission proves exact record linkage and observed store ingest only.
It does not prove source authorship, package correctness, provenance, or reproducibility.
