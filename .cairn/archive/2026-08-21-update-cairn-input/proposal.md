## Why

The dev shell resolved `cairn` from the ambient home-manager profile because no flake input provided it. That binary is older than the schema of the committed `cairn-policy/generated/cairn-policy.json`, so in-repo lifecycle commands failed with `policy has invalid field task_marker_policy.markers`.

## What Changes

- Add a root flake input `cairn` pinned to revision `695124d459574ba7aeba6097310d237f393c243c` (the same revision Lattice pins) and expose `cairn.packages.${system}.default` in the development shell.
- No policy regeneration is required: the committed generated policy parses cleanly under this revision.

## Impact

- **Files**: `flake.nix`, `flake.lock`
- **Testing**: in-repo `cairn validate --root .` passes; `nix flake check -L` reports only the pre-existing `bootstrap-blocker-inventory` failure identical at base
