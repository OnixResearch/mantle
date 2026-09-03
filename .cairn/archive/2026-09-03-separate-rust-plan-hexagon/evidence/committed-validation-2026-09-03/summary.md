# Committed-source validation

Source commit: `93ab9dcc574dab9989a8528ff9297bafcf420dbc`

## Accepted evidence

- `mantle-rust-plan-core`: 21 tests and one compile-fail doctest pass.
- `mantle-rust-plan`: 6 application and port tests pass.
- Mantle `rust_plan::`: 222 tests pass, equal to baseline.
- Mantle `rust_plan_hexagon::`: 3 adapter tests pass.
- `tests/rust_plan_cli.rs`: 57 tests pass, equal to baseline.
- The Cargo-free bounded fixture proof succeeds, prints `42`, and does not invoke Cargo.
- Host and `wasm32-unknown-unknown` core checks pass.
- Strict first-party Clippy and the exact Tiger Style Nix gate pass.
- The architecture checker reports zero findings and detects all 13 negative authority fixtures.
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

The full check builds the local Tiger Style gate before it reaches this external import. No hash, source pin, package meaning, or full-check gate was changed to accommodate the cache.

## Decision

Accept the Rust-plan hexagon extraction. Synchronize the four requirements and archive the change while preserving the independent full-check blocker.
