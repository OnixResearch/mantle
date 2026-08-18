# Baseline: Fresh-clone self-build input hydration

## Current state

- Repository HEAD before implementation: `b207fd0c`.
- The worktree was clean and synchronized with `origin/main`.
- `.gitignore` excludes `vendor-deps/`.
- The prepared checkout's ignored `vendor-deps/` measures 810.82 MiB apparent size.
- `mantle source bundle bootstrap-profile --mode self-build-proof` can describe vendored Cargo and provider payloads, and source-bundle import can pin records, but no public command materializes the vendor record into a fresh clone.
- `mantle bootstrap --fetch --offline-source-preflight` can consume an imported provider archive, but the successful fixed-point proof used prepared checkout-local inputs.

## Candidate portfolio

1. **Commit `vendor-deps/` to Git:** rejected because it adds roughly 811 MiB of generated source and broad lock-update churn.
2. **Export a Nix-only fixed-output closure:** rejected as the primary contract because it requires Nix closure transfer and duplicates Mantle source-bundle records and pins.
3. **Add a second tar/archive format:** rejected because it duplicates path, symlink, digest, profile, and provider validation.
4. **Hydrate from `mantle-source-bundle-v1`:** selected because it reuses current deterministic payload identities, bootstrap profile classes, import state, pins, offline provider overrides, and bounded non-claims.

## Observable completion

- A fresh clone with absent `vendor-deps/` accepts a bundle only when an out-of-band expected manifest BLAKE3 matches.
- The hydrated vendor tree passes Mantle's host-tool-free `Cargo.lock` and Cargo checksum guard before publication.
- Publication uses atomic no-replace semantics and preserves any existing destination.
- Provider records are imported and pinned for offline provider preflight.
- Empty-Cargo-home locked offline metadata resolves without ambient registry/git caches.
- Failures publish no success report and do not leave a partial vendor directory.

## Claim boundary

This change can prove portable identity-matched availability for the explicit Cargo and legacy-provider inputs. It cannot by itself prove a full fixed-point rebuild, completeness for undeclared future bootstrap sources, compiler correctness, seed trust removal, release reproducibility, independent agreement, deployment, or full Cargo compatibility.
