# Committed-source validation

Source commit: `2a61508c11e0e5e06cc370fa986b72c8b0a24253`

## Accepted evidence

- `crunch-build-planning-core`: 37 tests pass.
- `crunch-pipeline`: 42 unit tests and 19 integration tests pass; 4 integration tests remain ignored.
- Mantle `realization_routing::`: 24 tests pass, equal to baseline.
- Mantle `build_plan::`: 4 tests pass and 1 remains ignored, equal to baseline.
- Mantle `build_planning_hexagon::`: 3 tests pass.
- The checked route matrix covers all seven compatibility routes.
- Strict first-party Clippy and the exact Tiger Style Nix gate pass.
- The architecture checker reports zero findings and detects all 14 negative authority fixtures.
- Focused Nix checks for the core, core `wasm32` build, and architecture rail pass.
- Machine-contract validation passes with 24 contracted and 57 classified surfaces.
- Durable-publication adoption passes with refreshed exact Cargo and flake bindings.
- Cairn validation reports `"valid": true`.
- Tracey reports 155/155.
- Proposal, design, and tasks gates return PASS; all 11 tasks are complete.
- `nix flake check --no-build -L` passes.

## Full-check boundary

`nix flake check -L` reaches the unchanged remote Rust source import blocker:

```text
error: hash mismatch importing path '/nix/store/3r2nwafkx9xha0y6xsd0w0557ba0c294-rust-src-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu';
         specified: sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=
         got:       sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=
```

The full check builds the local Tiger Style gate and starts local release and transcript checks before it reaches this external import. No hash, source pin, package meaning, or full-check gate was changed to accommodate the cache.

## Decision

Accept the build-planning core extraction. Synchronize the four requirements and archive the change while preserving the independent full-check blocker.
