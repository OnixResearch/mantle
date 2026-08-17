## Why

Direct `.drv` import now avoids host Nix for explicitly listed files, but operators still have to spell every logical path-to-file pair. A directory-based mode is the next ergonomic step: point Mantle at a bundle of concrete `.drv` files, select one root, and have Mantle derive the reachable closure from explicit file contents without invoking `nix` or discovering the host store.

## What Changes

- Add a `--drv-dir` input mode to `foreign-import produce-nix`.
- Map direct child `*.drv` files to logical `/nix/store/<basename>` identities.
- Parse the directory into concrete derivation facts, select only the root-reachable closure, and fail closed when any referenced input `.drv` is missing from the bundle.
- Keep derivation JSON and explicit `--drv logical=file` modes intact.

## Impact

- **Files**: foreign import spec delta, pure reachable-closure selection helper, CLI directory shell, fixture tests, and evidence.
- **Testing**: positive directory import with an unrelated `.drv` ignored from the emitted graph, negative missing input `.drv`, focused import tests, formatting, and Cairn validation/gates.

## Out of Scope

- Recursive host store discovery.
- Calling `nix`, `nix-store`, flake evaluation, Nix expression evaluation, or overlays.
- Recomputing upstream Nix derivation store paths with Mantle-modified hash code.
- Claiming substitution, rebuild compatibility, output trust, package correctness, or reproducibility.
