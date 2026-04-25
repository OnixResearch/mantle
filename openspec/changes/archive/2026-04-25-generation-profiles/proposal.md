## Why

crunch can build packages and activate dev shells, but there's no persistent "these tools are available in my user environment" model. You can `crunch shell` into a project, but you can't say "install ripgrep, fd, and jq globally for my user" and have them persist across shell sessions with rollback.

This is the `nix profile` / `nix-env -i` equivalent — but designed for crunch's model, not Nix's.

## What Changes

- **`crunch profile` command family**: `crunch profile install <pkg>`, `crunch profile remove <pkg>`, `crunch profile list`, `crunch profile rollback`, `crunch profile switch <generation>`.
- **Generation model**: Each profile mutation creates a new generation — a symlink forest of the currently installed packages' `bin/`, `lib/`, `share/` directories. Generations are numbered and retained until explicitly garbage-collected.
- **Profile manifest**: A JSON or TOML file in `~/.local/share/crunch/profiles/default/manifest.json` tracking which packages (by store path or derivation reference) are in the current generation.
- **PATH integration**: `crunch profile` outputs an activation snippet (for fish/bash/zsh) that adds the current generation's `bin/` to PATH. Users source this from their shell rc. Alternatively, a symlink at `~/.local/share/crunch/profile/current/bin/` that always points to the active generation.
- **Rollback**: `crunch profile rollback` switches the `current` symlink to the previous generation. `crunch profile switch N` jumps to any retained generation.
- **GC integration**: `crunch store gc` respects profile roots — packages referenced by any retained generation are not collected.

## Capabilities

### New Capabilities
- `profile-install`: Add a package to the active profile
- `profile-remove`: Remove a package from the active profile
- `profile-list`: Show installed packages and available generations
- `profile-rollback`: Revert to the previous generation
- `profile-switch`: Jump to a specific generation
- `profile-gc`: Remove old generations

### Modified Capabilities
- `store-gc`: Respects profile generation roots

## Impact

- **Files**: New `src/profile_cmd.rs`, profile manifest schema, GC root integration in `crates/crunch-store/`
- **APIs**: New CLI subcommand family
- **Dependencies**: None beyond what's already in the tree
- **Testing**: Unit tests for manifest manipulation and generation bookkeeping. Integration test for install/rollback cycle.
