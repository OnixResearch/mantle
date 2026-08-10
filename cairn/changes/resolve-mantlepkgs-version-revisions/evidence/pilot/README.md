# Historical Mantlepkgs pilot evidence

## Result

The pilot resolved two reported `hello` versions to two exact Nixpkgs revisions on `x86_64-linux`.

It rechecked both versions, emitted one Mantlepkgs v1 manifest for each revision group, generated both catalogs, adapted both catalogs into domain shards, and composed one Nix-free public domain catalog.

## Producer

- Canonical Nix path: `/nix/store/lhgrxz1snbqxgvgzf6z2s4dg77fqjnya-nix-2.35.0/bin/nix`
- Version: `nix (Nix) 2.35.0`
- Executable BLAKE3: `f9fda9d013aa084c19357712a964095a95b7a735a08643c1db39e34620189478`
- System: `x86_64-linux`

## Index

- Cohort BLAKE3: `efa3591a4b81ceace11c103af83c7e9c53cfaf80ea7364bfe43b0eba525d21ee`
- Observation-set BLAKE3: `6faa060b78d74ee042920d6a5e6622720dc4df70814729f3d7199c704e556850`
- Compact-index BLAKE3: `3956447f10a1373f80ca2d58fa664db045ea308b3a15fe5cfe7c8a9ddfbac3e2`
- Successful observations: `2`
- Unavailable observations: `0`
- Failed observations: `0`

The index maps `hello` `2.12.2` and `2.12.3` to their exact observed revisions.

## Resolution

- Resolution-set BLAKE3: `a4fa6e0840853da3ef4a438d72aa44a9045c17a972c3137abb5e6799978717ef`
- Production-plan BLAKE3: `6eeeefcd0fe87eee1526b76dddbdf2baa8959ce86373b11af692e481ee679478`
- Revision groups: `2`
- Blocked requests: `0`

`hello@2.12.2` resolves to `b40629efe5d6ec48dd1efba650c797ddbd39ace0`.

`hello@2.12.3` resolves to `241313f4e8e508cb9b13278c2b0fa25b9ca27163`.

The second request owns the explicit unversioned `hello` alias.

## Recheck

- Recheck-set BLAKE3: `a141198ab35fb5edd978d19c999bb9dfffe76e2675c3f5fb182373c035978df5`
- Successful manifests: `2`
- Source-tree BLAKE3 for `2.12.2`: `c8dd5e518c3a80141b91c61af57142119a57a91cee08097640f45d17d69d3f96`
- Source-tree BLAKE3 for `2.12.3`: `3ec6c13885467ce23bd05a77917ca0a6b3f9e144ced1f29267d008036fbbf4a9`

Each recheck observed the exact requested version. It copied the bound policy files beside each generated manifest.

## Catalog generation

- `hello@2.12.2` catalog BLAKE3: `1cff882a77f0dc913430885a394309b49897e0cc33b411847d4b1fc9a4a0cfbe`
- `hello@2.12.3` catalog BLAKE3: `74da1be5ec74dcb97ebb2bb7e9e22b9e52062c946e2ba872aedf4053fc85dd81`
- Published packages: one in each catalog

The pilot exposed two existing graph-compiler gaps. Ordinary Nix fixed-output derivations and exact store paths followed by a prefix-map `=` boundary now compile with positive and negative regression tests.

The evidence bundle keeps each catalog record, producer receipt, package index, and source-requirement inventory. Producer receipts bind the larger shared graph artifacts by BLAKE3.

## Domain composition

- `hello@2.12.2` shard BLAKE3: `3fc7be27473c918728acd4ca83e0c2e6b84c89a870e5c34f81554275a7bd91fb`
- `hello@2.12.3` shard BLAKE3: `c16b8c1928e67959d876210a9542afa3e3a390cdbc2a7b515bd2c641d703e98a`
- Composed catalog BLAKE3: `bc0a75f644b06fbdeb832c160d655863c8f3c3a91abe488752a4462b1c964f80`
- Public records: `5` (`2` versioned selectors and `3` aliases)

The final composition ran with `PATH=/nonexistent`. It did not execute Nix.

## Claim boundary

This pilot proves only the recorded observations, deterministic resolutions, exact rechecks, catalog publications, shard adaptations, and Nix-free domain composition.

It does not prove package correctness, compatibility, cache retention, evaluator parity, reproducibility, deployment safety, or release eligibility.
