# Nixpkgs Multiverse reference audit

- Repository: `https://github.com/fzakaria/nixpkgs-multiverse`
- Reviewed revision: `824872b69c38eb024ab85ad15933133a0802aa8d`
- Target system: `x86_64-linux`
- License observation: `MIT`
- Source NAR BLAKE3: `852552bb709d34bd307ea9b3da6ad4049e204a1162c0edb1d3131be9f2881b1a`
- License-file BLAKE3: `d6a530b4bcb9f92842fcc2ca9101335be0772e037398c876c78aeb2ceaa552f8`
- `flake.nix` BLAKE3: `b1ded155b4fe041b2c1d3bde272ab64cc91395b5f5bc1acaf9ad2f434ca75d16`
- `tools/build-index.sh` BLAKE3: `8ea1b465ddbcd9aa971e5f26c07d4f829aa2217632afa6d0ef53f8c5c200070e`

## Inspected design

The reviewed flake has no inputs. It reads a compact version index and uses `builtins.fetchTree` for selected revisions.

The index builder evaluates the package `version` attribute. It retains the newest revision for each attribute and version pair.

The revision collector uses published channel records. This selection gives evidence of channel publication, not current cache retention.

## Mantle decision

Mantle uses the compact-index and newest-revision design before catalog production. Mantle does not import the upstream flake API.

Mantle records failed and unavailable observations. It also rechecks the exact revision before it emits a Mantlepkgs manifest.

## Authority boundary

This audit is comparison evidence only. The upstream repository does not grant source, package, output, trust, or release authority to Mantle.
