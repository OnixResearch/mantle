# Tasks: Ship delta substitution

## Phase 1: HTTP substitution integration

- [x] Add delta-capable negotiation to the trusted HTTP substituter path while
      keeping ordinary full-artifact fallback available and requiring delta
      negotiation to stay on the same trusted HTTP authority chosen for
      ordinary substitution. ⏱ started: 2026-04-16T01:38Z
- [x] Implement receiver compatibility-manifest construction from local
      `PathInfo` and castore presence for the requested output or closure,
      using cheap metadata paths such as `get_references()` where available so
      manifest building does not force full payload reads. ⏱ started: 2026-04-16T02:18Z
- [x] Wire the existing repo delta-transfer protocol and codec into the trusted
      HTTP substituter path, including any capability or parameter exchange
      needed before streaming starts. ⏱ started: 2026-04-16T13:16Z
- [x] Implement streamed delta transfer handling in the trusted HTTP
      substituter path, including applying the delta stream and reconstructing
      the final artifact before final acceptance. ⏱ started: 2026-04-16T13:34Z
- [x] Implement mid-stream delta failure handling that can restart through the
      ordinary full-fetch path when streamed delta transfer cannot finish.
- [x] Feed completed delta transfers into the existing final cache-hit
      acceptance path so signed `PathInfo` verification stays unchanged and the
      same local metadata or attestation consequences as ordinary substitution
      are recorded. ⏱ started: 2026-04-16T13:34Z

## Phase 2: Reporting and fallback behavior

- [x] Record substitution mode and byte counts for successful `delta` and
      `full` cache hits, plus an optional fallback reason only when delta
      negotiation was attempted and abandoned.
- [x] Define and snapshot-check the stable JSON reporting fields for
      substitution mode, transferred bytes, reused bytes, and conditional
      fallback reason. ⏱ started: 2026-04-16T14:47Z
- [x] Expose those reporting fields in both human-readable output and the JSON
      build report.
- [x] Document how a cache operator can tell whether a substitution used delta
      reuse or ordinary full fetch.

## Phase 3: Validation coverage

- [x] Add tests for same-authority delta negotiation and successful delta cache
      hits with reusable local content.
- [x] Add tests that delta negotiation is rejected or falls back safely before
      acceptance when it would cross to a different HTTP authority than the
      ordinary substituter authority.
- [x] Add tests that a trusted cache without delta support still serves a normal
      full-artifact substitution cache hit without error.
- [x] Add tests that missing local backing content is treated as absent from the
      compatibility manifest.
- [x] Add tests that manifest construction stays bounded to the requested
      output or closure and does not enumerate the whole local store first.
- [x] Add an end-to-end closure-scoped delta substitution test covering
      negotiation or fallback, final acceptance, and reporting for a requested
      closure.
- [x] Add tests that delta negotiation or stream failure falls back cleanly to
      ordinary substitution without changing final trust semantics. ⏱ started: 2026-04-16T13:45Z
- [x] Add tests that a completed delta transport is still rejected when the
      final signed `PathInfo` is untrusted. ⏱ started: 2026-04-16T13:45Z
- [x] Add tests that delta-backed and full-fetch cache hits record the same
      local metadata or attestation consequences after final acceptance.
- [x] Add tests that human-readable and JSON reporting both expose transfer
      mode and byte counts for successful delta and full-fetch cache hits, with
      deterministic assertions for expected transferred or reused byte counts or
      invariant relationships, and with fallback reason present only for
      successful full-fetch outcomes that abandoned attempted delta negotiation. ⏱ started: 2026-04-16T14:47Z
- [x] Exercise at least one real local HTTP cache fixture built from persisted
      `state_dir/pathinfo.redb`, `state_dir/blobs`, a tiny local HTTP server,
      and a fresh receiver `state_dir`, rather than an in-memory fake. ⏱ started: 2026-04-16T14:47Z

## Validation

- [x] Run the relevant substitution and reporting test suites with the repo's
      documented build environment and keep the `test result:` lines.
- [x] Run `openspec validate ship-delta-substitution`.
