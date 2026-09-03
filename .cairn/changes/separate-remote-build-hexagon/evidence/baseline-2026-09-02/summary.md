# Baseline

Source commit: `a32a585acd6ac52defacb2b3bec19a5cfd5cf2fc`

The baseline ran in a clean detached worktree with an isolated Cargo target before remote implementation files changed.

- `nix develop -c cargo test -p crunch-build distributed`: 117 passed; no failures.
- `nix develop -c cargo test -p mantle --bin mantle remote_build::`: 146 passed; no failures.
- Cairn validation: `"valid": true`.

Exact commands and output are beside this file.
