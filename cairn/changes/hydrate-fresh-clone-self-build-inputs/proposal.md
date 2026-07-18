# Change: Hydrate fresh-clone self-build inputs from a verified source bundle

## Why

A current Mantle checkout can self-build from its ignored `vendor-deps/` directory and a pinned legacy seed-provider input, but a fresh clone contains neither payload. The last fixed-point proof therefore establishes the prepared checkout only. Operators need a deterministic offline handoff that reconstructs those explicit inputs without consulting ambient Cargo caches, language-package caches, or the network and without committing roughly 811 MiB of generated vendor material to Git.

## What Changes

- Add an operator-facing source-bundle hydration command that requires an out-of-band expected manifest BLAKE3 before mutating a fresh checkout. r[bootstrap_inventory.fresh_clone_source_hydration]
- Materialize the bundle's single vendored-Cargo record through a validated staging directory and atomic no-replace publication, then run Mantle's existing `Cargo.lock` and `.cargo-checksum.json` guard over the hydrated checkout. r[bootstrap_inventory.fresh_clone_source_hydration]
- Import and pin the same bundle's legacy provider records into Mantle source state so `bootstrap --fetch --offline-source-preflight` can consume the provider without a live fetch. r[bootstrap_inventory.fresh_clone_source_hydration]
- Add positive fresh-clone and negative digest/tamper/missing-record/no-clobber coverage, a machine-readable hydration report, an operator runbook, and bounded evidence. r[bootstrap_inventory.fresh_clone_source_hydration]

## Impact

- **Public CLI:** adds `mantle source bundle hydrate-self-build --from <bundle> --expected-manifest-blake3 <digest> --checkout <fresh-clone>`.
- **Mutation:** creates only an absent `<fresh-clone>/vendor-deps`, imports verified records under the selected Mantle state directory, and writes a pin for the verified manifest; existing checkout paths are never replaced.
- **Trust:** the expected manifest BLAKE3 is an explicit out-of-band input. The bundle's self-declared digest alone is not treated as authority.
- **Non-claims:** hydration proves only identity-matched source/input availability. It does not prove fixed-point self-build success, complete capture of every future bootstrap source, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment, or full Cargo compatibility.
