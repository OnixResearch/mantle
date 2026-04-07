## Why

The README has drifted away from the shipped CLI.

At least three operator-facing mismatches are visible from the current tree:

- `src/main.rs` defaults the logical store prefix to `/crunch/store`, but the
  README still describes `/nix/store`-centric logical path behavior in several
  places
- the CLI now ships project-management commands (`init`, `check`, `show`,
  `refresh`, `list-stale`, `upgrade`), but the README command list still reads
  like an engine-only tool
- build, self-build, and store flows now expose signing and trust controls,
  but the README barely documents them

There are also stale environment notes: the current tree no longer requires
`protoc`, and the README still mixes old writable-`/nix/store` assumptions
with the newer castore/export behavior.

## What Changes

- Audit README sections against the current Clap command surface and defaults
- Update quick-start, CLI, store-path, and requirements sections to reflect
  `--store-prefix`, `--nix-compat`, and the difference between logical store
  paths and physical export directories
- Document the project-management commands and the intended project workflow
- Document signing/trust flags where they affect builds, substitutions, and
  store verification
- Remove or rewrite stale environment notes that no longer match the current
  implementation

## Capabilities

### Modified Capabilities
- `cli`: user-facing CLI documentation matches shipped commands and defaults

## Impact

- **Files**: `README.md`, and any nearby doc snippets or examples that repeat
  outdated CLI assumptions
- **APIs**: none
- **Dependencies**: none
- **Testing**: manual verification against `crunch --help` and relevant
  subcommand help output

## Notes

This is a documentation-alignment change. It should treat the code and current
help text as the source of truth rather than preserving older README wording.
