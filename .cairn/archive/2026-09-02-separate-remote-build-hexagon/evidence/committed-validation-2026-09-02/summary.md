# Committed-source validation

Source commit: `26bdd0fa0a5ff6e7a814db9d6e1a3c4a39af512f`

## Accepted evidence

- `crunch-remote-core`: 12 tests and one compile-fail doctest pass.
- `crunch-remote`: 5 application and port tests pass.
- `crunch-build distributed`: 118 tests pass. The baseline was 117; the added test checks exact legacy realization-key bytes.
- Mantle `remote_build::`: 146 tests pass, equal to baseline.
- Mantle `remote_hexagon::`: 3 adapter and wire-compatibility tests pass.
- Host and `wasm32-unknown-unknown` checks pass.
- Strict first-party Clippy and the exact Tiger Style Nix gate pass.
- The remote architecture checker reports zero findings and detects all 13 negative authority fixtures.
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

The check reaches this external import after local architecture work has started successfully. No hash, input pin, or full-check gate was changed to accommodate the cache.

## Octet diagnostic boundary

The targeted deterministic-core profile remains non-green only on its required function-address corpus. The separate fresh deny-all diagnostic is preserved and no baseline or disabled lint was added. These diagnostics are not used as acceptance evidence for V5.

## Decision

Accept the remote-build hexagon extraction. Synchronize the four requirements and archive the change while preserving the full-check and Octet diagnostic boundaries.
