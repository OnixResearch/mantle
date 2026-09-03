# Archive verification

- Cairn archived the completed change with receipt BLAKE3 `925fa264993e4d9bd2d0195a61cd0a9b2495c94d6d872d7fa076899ca7f430a1`.
- Cairn reported archive mutation-manifest BLAKE3 `f04f75064d09b9afce68f2c566955ff6aed953a77396d5bf0f546fea76a8c84f`.
- The operator corrected Cairn's known `1970-01-01` date to `2026-09-03` before validation.
- The active change path is absent.
- The archived task file has 12 checked tasks and no unchecked task.
- The accepted application-architecture spec contains all five synchronized requirement IDs.
- The accepted spec BLAKE3 is `52d28c143e62c4f1f73fc02c1b330dd22d96c8faa4b0f00c791e27e7fd2dfb43`.
- Post-archive Cairn validation reports `"valid": true`.
- Post-archive Tracey reports 155/155 referenced for its configured corpus.
- Post-archive `nix flake check --no-build -L` passes.
- The maintained CLI architecture checker reports zero repository findings after archive.
- The post-archive whitespace check passes.

The ordinary full-flake run remains blocked by the recorded Rust-source import hash mismatch. The local-builder run reached the root suite and retained eight unrelated host or fixture failures. Neither boundary was weakened.

`archive-files.blake3` lists 119 regular archive files. It excludes itself and `archive-manifest.blake3`. The adjacent hash file records the final BLAKE3 of that sorted file manifest.

This archive proves lifecycle and recorded source-boundary facts. It does not prove the failed full checks, current-source fixed-point parity, external effect success, provider correctness, deployment, or release eligibility.
