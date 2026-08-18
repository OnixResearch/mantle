## Why

Provider-bound releases currently can package a reduced source archive while the publisher's provider fixed-point proof was built from the full live path package source. Native Rust planning includes the package source digest in unit identity, so replaying the provider proof from the reduced archive can produce a different provider binary even when the proof is internally fixed-point.

## What Changes

- Make `mantle release create` package tracked source files that native path-source hashing can observe, not only the self-build staging allowlist.
- Continue excluding ignored/private runtime surfaces such as `target/`, `.pi/`, VCS metadata, and root Cairn lifecycle evidence from the release source archive.
- Add positive and negative tests proving tracked package files are included while untracked/private paths are excluded.

## Impact

- **Files**: `src/release_source.rs`, Cairn verification-evidence spec delta, validation evidence.
- **Testing**: focused `release_source` unit tests, then release/witness proof reruns before making any independent witness claim.
