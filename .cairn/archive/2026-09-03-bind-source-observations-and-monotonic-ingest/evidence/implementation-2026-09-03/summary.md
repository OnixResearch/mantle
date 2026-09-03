# Source observation implementation summary

## Result

Mantle now has a backend-neutral `crunch-source-core`.
The crate is `no_std + alloc` and owns deterministic source policy.

The core provides:

- admitted source kinds and locator classes;
- explicit SHA-1 and SHA-256 Git revisions;
- normalized projections and snapshot profiles;
- measured content BLAKE3 values;
- domain-separated observation identities;
- explicit legacy provenance-unavailable results;
- add, identical-reuse, identity-conflict, and invalid-rejection ingest plans.

The source-bundle shell adapts existing `FetchUrl` and `GitRevision` values.
It measures canonical payload content separately from locator metadata.
It uses `durable-file-publication` revision `951c27f59003cea9bfdb40ed4d89653d50fada1f` on Linux.
Only an admitted `Add` plan can invoke create-new publication.

Release source acquisitions now include an optional canonical observation subject.
The subject omits locator and mutable-reference hints.
Legacy `SourceAcquisition` bytes remain unchanged when the field is absent.
The existing release-attestation signature covers the enclosing manifest identity.
No source-observation signer role exists.

Reviewed-source checks now require exact source-revision presence and value.
This keeps review evidence and the release observation on the same source content and revision facts.

## Focused evidence

The current working source passed these focused checks:

- `crunch-source-core`: 17 tests;
- source-bundle and ingest: 104 tests;
- `crunch-release-core`: 269 tests and one compile-fail doctest;
- release-source shell: 7 tests;
- witness source-acquisition shell: 6 tests;
- release CLI source filter: 26 passed and one fixture writer stayed ignored;
- project adapters: 7 tests;
- checked fetch and Git nominal values: 2 tests;
- source-core WASM check;
- strict source-core, release-core, and touched root Clippy checks;
- source-core Tiger Style check;
- source-observation shell adapter: 16 focused tests;
- source-observation architecture check with 16 negative fixtures;
- machine contracts: 27 contracted and 60 classified surfaces.

The selected source-bundle v1 fixture remains unchanged:

- file BLAKE3: `b8abb366a886bb79314411a3d479a97fe8481173334c238333b9118fe7ab8030`;
- embedded manifest BLAKE3: `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777`.

The new golden release source binding has BLAKE3:

`3a10bb482e9fec2acd73a4cd7ac97e7e449da5be8c54f63911f6487d1765360b`

## Claim boundary

This work binds supplied source facts to measured content.
It does not prove source ownership, upstream intent, review quality, license compliance, build correctness, or release eligibility.

The focused checks do not replace the repository-wide Nix, Cairn, Tracey, Clippy, or Tiger Style gates.
Those checks remain part of committed-source validation.
