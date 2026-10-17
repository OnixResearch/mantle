# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `builder/env.nu` and `builder/prepare.nu`: dependency closures render
  `CPPFLAGS`, `LDFLAGS`, `PKG_CONFIG_PATH`, `CMAKE_PREFIX_PATH` from
  `exports.json` records; nothing a dependency ships runs code in the
  consumer build; `exports = false` marks non-link outputs.
- `nix/build-systems.nix` and `nix/package.nix`: build systems as registered
  modules with typed options validated at evaluation time; unknown field or
  option is an evaluation error; phases are data (`"<bs>.<phase>"` or inline
  named steps).
- `docs/design.md` "Builders": the blind test of five builder API shapes;
  explicit build systems with a plain phase list won every round.

## Adaptation boundary

Design prior only. The admission gate keeps implementation blocked until a
named consumer exists; Mantle's contracts are Nickel, and no repkgs code is
copied.
