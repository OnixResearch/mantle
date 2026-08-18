## Why

Mantle may have a stronger evidence architecture than ordinary Nix cache trust, but the claim needs empirical pressure on comparable builds. A Nix-vs-Mantle corpus should compare equivalent source/toolchain/package cases by output object or NAR digest, record unsupported differences, and avoid overclaiming when the systems intentionally differ.

## What Changes

- Define a representative comparison corpus covering Rust, C, shell-heavy builds, fixed-output fetches, multi-output artifacts, build scripts, and cache/substitution cases.
- Add a corpus runner/report that builds equivalent cases through Nix and Mantle, normalizes artifact identity, and compares output object or NAR digests.
- Record blockers for unsupported features, non-equivalent toolchains, path-prefix differences, expected metadata drift, and missing evidence.
- Publish comparison summaries as evidence, not as automatic superiority claims.

## Impact

- **Files**: comparison fixtures, runner/report code or scripts, docs, tests, Cairn verification-evidence spec delta.
- **Testing**: positive equivalent-output case, negative non-equivalent-toolchain blocker, unsupported-package blocker, Cairn validation/gates.
