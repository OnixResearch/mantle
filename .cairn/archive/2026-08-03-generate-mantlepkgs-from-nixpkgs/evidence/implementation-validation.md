# Mantlepkgs implementation validation

## Result

The implementation generated one deterministic catalog from a locked nixpkgs revision.

It rebuilt the complete selected cohort under `/mantle/store`. Nix was absent from `PATH`, network access was disabled, and substitution was disabled during each rebuild.

## Producer boundary

The source lock is `github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e`.

The producer used this canonical executable:

`/nix/store/2x1gaar1s91fg29iky9xx97k28pc360j-nix-2.35.0/bin/nix`

The producer facts are:

- Version: `nix (Nix) 2.35.0`
- Executable BLAKE3: `21f23076871987c41c76f10c62de02236d0651ea1e5b912d800b0644956e57fc`
- Source-lock BLAKE3: `44015a40ba7858db1fa8df4f2bcca11499b04b90be1f8eb8ae3c353009203274`
- Command class: `nix-eval-drv-path-derivation-show-recursive-and-build-fixed-output-seeds-v1`

The producer created 352 deterministic seed-retention links. A fresh validation found no dangling link. These links keep production-time fixed-output paths live until source bundling finishes.

Nix ran only at this producer boundary. Catalog verification, package selection, planning, source import, and realization did not run Nix.

## Published catalog

The accepted generation has these facts:

- Catalog BLAKE3: `fe5ef37d81a8492ddad4232cfc942e80b70fcfa75a2b45816e2eb96f4855d1d8`
- Manifest BLAKE3: `b2bcb96e755c60835a3fdb43e89824d527c54739a9f21016e7e8d700856d7ff9`
- Producer plan BLAKE3: `33cde4c0e7a05e96277c24a17cc4f16d4fff5fc3cc0e0b56315b3040d9424637`
- Producer receipt BLAKE3: `75e68df96792d23a3ee6b5dfd41bf2e8f4eac1a5ca00cc2b9cb3a60c0a3d562a`
- Packages: 3
- Shared graph nodes: 954
- Source requirements: 279
- Bound publication artifacts: 6
- System: `x86_64-linux`
- Target store prefix: `/mantle/store`

All three package dispositions are `buildable`:

| Package | Root node | Graph BLAKE3 |
|---|---|---|
| `hello` | `nix:7mdg60drrnh0wq1j8hmmbhll47czm107-hello-2.12.3.drv` | `29279d24d77604bbb74c04e8bc23b0ce8ca1f52263fb947cdb60bcc639e50fea` |
| `zlib` | `nix:fbnvy50vkiahjzb91x1jk63lilkvfagy-zlib-1.3.2.drv` | `6fc2e9c23b3ca1e720353ec8de170c652e9abf805e34304a365d593519b584b7` |
| `libpng` | `nix:ygxqgz5yg896gqzkl7r9adkr1kmdss8k-libpng-apng-1.6.55.drv` | `ea824f18a4d28799cc0f77c56990f260d00484fbe59794441a0387116698e3bd` |

Publication used a staged, validated, atomic no-replace rename. A repeated publication to the same identity was rejected.

## Source bundles

Each bundle includes the selected source closure and content-bound fixed-output seeds.

| Package | Records | Bundle BLAKE3 |
|---|---:|---|
| `hello` | 277 | `124b2c5f0b61dd5ed9b21a8999058b2249b7ffebbd5d6b354642fa94b81eced6` |
| `zlib` | 250 | `32e9c1ae3333239882e84063b2aa82a95b0609595145ef1fbfaa8dc6e7c1868b` |
| `libpng` | 278 | `b4fcc59cf9346d1d370f059a42ea297bf8672623e30e599b3ccd4d430489e555` |

Bundle import recomputed each Mantle path. It rejected missing records, identity mismatches, collisions, tampering, traversal, and escaping links.

## Nix-free realization

Each receipt has status `complete`, strongest state `realized`, and `failure: null`.

| Package | Units | Plan BLAKE3 | Build report BLAKE3 | Receipt BLAKE3 |
|---|---:|---|---|---|
| `hello` | 951 | `cce1dd1c867a565291ab130162ad4e920e7ad6f3fd6bc5e87dd84b81bef0ee16` | `55473cdf05be2f60c952f3785be8debac06a1aab6c5aff7e532429b2afada7d0` | `3bdfd6c06d511f6b695b7f0ba1a86c3e73526c8862d9c33942c566cd3c6f35a7` |
| `zlib` | 760 | `0c646206764123c636da839ee8027afea24c7c1b3b8c6d14e4b2dcdb4c4dbbca` | `cfc349478e8757499453dec1a7340ae896934057a91b535b0a221c4f58efc8ed` | `f7bf3c68238b34ecb76c1bf50b4a6ec9f0678d3ac2186ca22b001b79d7c45fb1` |
| `libpng` | 951 | `90fa492c91fdf6fc723e15da39346040636ceb0a4a8387bb01094b06060ccf78` | `95ad28da81b7688a5be83a6eb2dd338bec1149434abe4f3a3206f9835c46a072` | `4bf949f61e39fafd5d75d82acc158a59809da873f469ebca09c02f706bb8d27a` |

The import-receipt BLAKE3 values are:

- `hello`: `2dea59be1320ec7390679bdf3ee78d47268ad800b2d07d24a5acfde6eeb14be4`
- `zlib`: `471afb7cc375fbef89cb6bb5cb16fc3edf15c6c74e39a52e9cc4ac40555fdc58`
- `libpng`: `f480fafdaa65579e73812cf299e246209d4a514f03bea9d2da72aaed4fdd9fed`

The realized primary outputs are:

- `hello`: `/mantle/store/8qyddg2msyld601hhq8c7cfmdag0v8yq-hello-2.12.3`
- `zlib`: `/mantle/store/49c6ibf6sqi7ipnqmf1w88ci7jzw4d16-zlib-1.3.2`
- `libpng`: `/mantle/store/hdg2k5dwd6s80w36gii0z02i811fwbnq-libpng-apng-1.6.55`

Their protocol NAR SHA-256 values are `86f3553d8e0e4a16fcd9dc22fef9ab24c42a3494882871a9bcf44929104b698c`, `c39c9a18f77e6306e94b917b7479b83feecb0f95e75e53f0e30bf0d02da1e723`, and `262aec1d10c16ad3699e7f9f45554b150729fbd24fee2ef8c8ab49cbb260617f`.

SHA-256 is present only at Nix-compatible protocol boundaries. Mantle-owned identities use BLAKE3.

## Foreign Nix protocol support

The realization rail now preserves the required Nix builder protocols:

- complete PathInfo input-reference closures in no-FUSE sandboxes;
- output-placeholder replacement before payload expansion;
- exact Snix `passAsFile` additional-file paths;
- canonical structured-attribute JSON and shell files;
- active-store output bindings and collision checks;
- required Nix environment variables;
- selective `@storeDir@` rewriting;
- normal `/proc` metadata for foreign Nix builds;
- real random devices for foreign Nix builds.

Default and foreign Guix sandboxes retain the existing `/proc` and random-device masks.

## Negative evidence

Tests and live failures cover these cases:

- stale locks and stale artifact digests;
- missing source records and undeclared seed outputs;
- bundle and graph tampering;
- traversal and escaping symlinks;
- alias and graph conflicts;
- graph cycles and incomplete batches;
- unsupported builtins and unresolved paths;
- leftover source-store paths;
- structured-attribute collisions and malformed payloads;
- unknown placeholders;
- duplicate publication;
- producer timeout and output limits.

A failed producer run wrote a durable report at `failures/e9e0dc2134f4bbe52a7d2e28a82314a891ad38e661c2444a62a04b63742fb856.json`. It did not publish partial success.

## Claim boundary

This evidence proves bounded catalog generation, identity and linkage checks, source retention, Nix-free planning, and successful realization for the selected cohort.

It does not prove full nixpkgs coverage, evaluator parity, package correctness, reproducibility, bootstrap parity, deployment fitness, or release eligibility.
