# Cairn input update validation

## Results

- `readlink -f $(which cairn)` inside `nix develop`: resolves to `/nix/store/zddawm6d27cgvic58y2jipgnqfh395n0-cairn-0.1.0/bin/cairn`, the build of pinned revision `695124d459574ba7aeba6097310d237f393c243c`. Before this change it resolved to the ambient home-manager profile.
- `cairn validate --root .` inside the dev shell: `"valid": true`, zero issues, all 9 open changes parsed.
- `nix flake check -L`: fails only on `bootstrap-blocker-inventory`, byte-identical to the failure at base `71db5be7`. No new check failures.

## Non-claims

The pinned revision matches the one Lattice uses, which reduces cross-repo tool skew; it does not prove schema stability across future cairn revisions or establish release eligibility for either repository.
