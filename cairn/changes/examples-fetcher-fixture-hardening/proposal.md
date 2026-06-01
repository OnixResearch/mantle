## Why

Fetcher examples are among the most important user entry points, but the current user-facing examples depend on real crates.io, GitHub, or raw-file URLs. Those are useful documentation examples, yet they make ordinary validation sensitive to external networks and do not fully exercise negative fixed-output behavior in a deterministic way.

## What Changes

- Add local/offline fetcher fixture examples that cover single-file fetches, tarball unpacking, and git checkouts without external network dependencies.
- Add negative fixed-output examples or fixtures for wrong hashes and expected `--fix` behavior.
- Keep real crates.io/GitHub examples as user-facing cookbook entries, but decouple CI-grade validation from live services.
- Assert that fetchGit examples do not require ambient host `git` for local fixture coverage unless the fixture setup itself explicitly uses a test-owned host helper.

## Impact

- **Files**: `examples/`, test fixtures under `tests/` or `examples/fixtures/`, `tests/examples_build.rs`, `tests/examples_eval.rs`, and docs.
- **Testing**: offline fetcher positive tests, fixed-output mismatch negative tests, `--fix` smoke where appropriate, `cairn validate --root .`, and task-gate evidence.
