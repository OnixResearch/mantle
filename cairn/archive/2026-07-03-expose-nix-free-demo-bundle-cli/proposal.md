## Why

The Nix-free demo bundle validator exists as a pure core, but operators cannot yet invoke it directly on a copied bundle. A CLI surface would make the new evidence gate usable outside unit tests and would let demos, release-readiness checks, and handoff workflows validate the same machine summary before using Nix-free wording.

## What Changes

- Add a `mantle` CLI path that validates a Nix-free demo bundle machine summary.
- Add a rendering path that produces the operator README from the validated machine summary.
- Emit stable JSON diagnostics for missing fixed-point evidence, missing guard evidence, and malformed summaries.
- Keep file I/O in the CLI shell and validation/rendering in the pure core.

## Impact

- **Files**: CLI command dispatch, `src/nix_free_demo_bundle.rs`, JSON fixtures, README/help text, Cairn verification-evidence spec delta.
- **Testing**: positive CLI validation, negative missing fixed-point evidence, negative missing guard evidence, README rendering consistency, JSON stdout/stderr behavior, Cairn validation/gates.
