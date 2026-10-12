# Evidence: repkgs source review

## Source

- Repository: https://github.com/mic92/repkgs
- Reviewed revision: `1cd7b8b` (2026-09-10), local clone at
  `/tmp/repkgs-review` during the 2026-09-10 session.

## Mechanism mapped by this change

- `nix/fetch.nix` and `builder/fetch/*.nu` (especially `cargo.nu`,
  `dyn-drv.nu`): a producer derivation reads the lock from the fetched source
  and writes one `builtin:fetchurl` derivation per artifact using the lock's
  own sha256, plus a collecting derivation that lays out the vendor
  directory. Rules: never copy an upstream lock into the repo, never invent a
  hash.
- `locks/*.toml` with `merge=union` in `.gitattributes`: shared sorted tables
  for ecosystems whose locks lack hashes (go, hackage, luarocks); only the
  producer reads the whole table; per-package output mentions only its own
  subset.

## Adaptation boundary

repkgs needs a Nix worker-protocol client for this; Mantle's own orchestrator
already chains fixed-output derivations and admits dynamic derivations
through the accepted `dynamic-derivation-admission` family, so the producer
calls Mantle-native admission. No repkgs code is copied.
