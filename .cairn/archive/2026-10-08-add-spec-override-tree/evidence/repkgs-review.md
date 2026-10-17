# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `nix/overrides.nix`: the override tree — verbs `set`, `append`, `prepend`,
  `merge`, `remove`, plus `edit` per package; layers merged into one tree
  before application; every path checked with the path in the message;
  dependency strings resolved against the final set.
- `docs/design.md` "Overrides": measured about 54 MB for 10k edits through
  the merged tree against 250–580 MB through `//` overlays,
  `makeOverridable`, per-package `extend`, or module systems, which all pay
  per layer.

## Adaptation boundary

Design prior only, gated on consumer admission aligned with the package
layer. Pure-core semantics are Mantle-owned; nothing is copied.
