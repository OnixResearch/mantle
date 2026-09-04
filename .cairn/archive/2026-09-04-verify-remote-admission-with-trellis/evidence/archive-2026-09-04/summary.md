# Trellis admission archive

Date: 2026-09-04
Implementation commit: `6ff60dcd91437306180819ffc4da27b8b7c1c00f`
Evidence and spec-sync commit: `24a5652942d68764d1b4429eddd2ddfbfc7815de`

The archive dry run reported `blocked: false` and `mutated: false`.

The executed archive reported `blocked: false`, `mutated: true`, and no rejection reasons. It produced these identities:

- plan BLAKE3: `5f1e71ef7f05756d033ea553b4267fa9bcf0eb580aa265db44cb16ba1c15bab1`;
- receipt BLAKE3: `503256c91de27f6619c504a291e0391a618a33aaf778dc336961c3d9ad62a755`;
- mutation manifest BLAKE3: `152ae0fb91900ede76a13adde3c920f6add67ebae7cc4ca212e983b5d68e4b30`;
- archived change manifest BLAKE3: `9aa5c8aa195581b1ca96ca92074227c20c0c073cd93324572c36fc81e354ecb6`.

Cairn first created `.cairn/archive/1970-01-01-verify-remote-admission-with-trellis`. The directory was renamed to the explicit session date:

`.cairn/archive/2026-09-04-verify-remote-admission-with-trellis`

The accepted requirements remain in `.cairn/specs/remote-builds/spec.md`.

Post-archive Cairn validation, Tracey, machine contracts, evidence validation, staged diff validation, and Nix flake evaluation passed.
