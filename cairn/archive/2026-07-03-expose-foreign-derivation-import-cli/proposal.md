## Why

Mantle now has a pure adapter-neutral foreign derivation import core, but operators can only exercise it through tests. To make the boundary usable, Mantle needs a thin CLI surface that validates graph/index/policy files and emits reviewable receipts and adapter plans without invoking foreign frontends during consumption.

## What Changes

- Add `mantle foreign-import validate` for graph, package-index, and policy JSON inputs.
- Add `mantle foreign-import plan` to emit translated graph receipt and Mantle adapter plan JSON.
- Ship checked-in Guix-like and Nix-like hello fixtures for positive smoke coverage.
- Keep all file I/O and JSON rendering in the CLI shell while the existing translation core remains pure.
- Reject attempts to require `guix`, `nix`, flake evaluation, Nix expression evaluation, overlays, or package-module evaluation during consumption.

## Impact

- **Files**: CLI command module, fixture JSON files, integration tests, README/operator guide examples.
- **Testing**: positive fixture validation/planning, malformed input tests, stale receipt tests, and boundary tests proving no foreign frontend process is invoked.

## Out of Scope

- Generating foreign graphs from live Guix or Nix frontends.
- Realizing full imported package closures.
- Treating Nix flakes or Guix modules as stable Mantle ABI.
