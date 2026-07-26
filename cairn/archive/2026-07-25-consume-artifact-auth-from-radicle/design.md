# Design: Mantle artifact-auth Radicle cutover

## Identity contract

The cutover accepts only `https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git` at revision `799459346d5416fbd7b9f55840a7371441b55afa`. Cargo manifest entries, Cargo lock sources, the Nix flake input, and the Nix lock node must agree. The locked Nix `narHash` must remain `sha256-nEgz2FtVuDesX95yyxidp0vhjxL4INB6Ve8rkpLyJk0=`.

The accepted publication evidence is revision `e41340bec587b6d049b5cc518ec7db925dde84be`, receipt type `artifact-auth.radicle-publication.v1`, and BLAKE3 `e58a3de4d6b3b32a547c3cfe5c3e829292cda73891c7776f214f5d4edce10b1c`.

## Functional core and imperative shell

No Rust implementation changes are required. Existing pure action-result mapping, canonical statement, Ed25519, currentness, trust, receipt, replay, and authority-boundary tests remain the behavioral oracle. Deterministic Nickel/Nix validation owns source and receipt admission; Cargo and Nix own network fetching and lock regeneration.

## Fail-closed checks

Validation rejects revision drift, RID or URL disagreement, GitHub fallback in executable manifests/locks, duplicate or absent package entries, changed Nix content identity, changed crate graph, missing positive or negative evidence, and weakened authority/non-claim boundaries.

## Rollback

Rollback is an explicit reviewed source-identity change to the same Git object with owning-tool lock regeneration. Mantle has no automatic GitHub fallback because fallback would hide a forge outage and make dependency evidence ambiguous.

## Evidence

A typed Nickel receipt exports canonical JSON and a BLAKE3 sidecar. It binds the publication receipt, public source identity, Cargo/Nix agreement, package graph, baseline/post-cutover observations, rollback boundary, and explicit non-claims.
