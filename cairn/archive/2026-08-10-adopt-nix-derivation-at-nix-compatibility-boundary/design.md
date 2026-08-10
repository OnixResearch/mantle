# Design: Adopt `nix-derivation` at the Nix compatibility boundary

## Context

Mantle uses one adapted `nix-compat` type across native derivation construction, build scheduling, store behavior, foreign imports, and Nix protocol support. Native Mantle behavior requires BLAKE3 and configurable logical store prefixes. Concrete Nix `.drv` admission requires the algorithms and canonical bytes defined by Nix.

The new dependency covers only derivation metadata. It does not implement a store, NAR, execution, daemon protocol, or filesystem hashing.

## Architecture

```text
owned .drv bytes and logical identity
  -> shell-owned byte and closure bounds
  -> pure Mantle Nix-derivation adapter
  -> nix-derivation parse and optional validation
  -> bounded Mantle compatibility projection
  -> existing foreign-derivation graph core
  -> existing planning, realization, store, and evidence paths
```

File reads, directory discovery, backend execution, and artifact writes remain in the shell. The adapter receives owned bytes, the logical `.drv` path, the declared source prefix, and explicit limits.

## Dependency admission

Mantle will review upstream commit `2cfc0f90ed83ea3cc983e5c305f89494a6df073e` and exact crate version `0.1.0`. The crates.io package checksum is `a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074`.

A deterministic guard must validate that the packaged source matches the reviewed source under a declared packaging comparison. Any drift blocks production use.

The dependency must remain confined to the adapter. A source guard rejects direct production imports elsewhere.

## Parsing and projection

The upstream API receives the derivation name outside the ATerm bytes. The adapter derives that name from the logical `.drv` path. It rejects mismatched or invalid names before graph publication.

Parsing and semantic validation remain separate. This permits exact syntax inspection and deterministic unsupported-feature classification without treating every parseable value as buildable.

The adapter maps every upstream output variant explicitly. Supported variants become existing foreign graph facts. Unsupported floating, deferred, impure, Git, text, or recursive dynamic forms produce stable unsupported-feature records or deterministic rejection.

Environment values remain byte strings at the parser boundary. Projection into the current string-based foreign IR rejects non-UTF-8 values with a stable diagnostic. It does not silently use lossy conversion.

Structured `__json` data uses the dependency's parsed structured-attribute representation. Candidate order and original bytes remain available for Mantle's existing fetch-candidate rules.

## Hash domains

Nix `.drv` hashing, output paths, placeholders, and standard store paths use the algorithms required by Nix. Mantle does not substitute BLAKE3 in this domain.

Mantle graph, policy, plan, receipt, and evidence identities continue to use domain-separated BLAKE3. Adapter types keep Nix digests and Mantle digests nominally separate.

Native Mantle derivations continue to use the adapted implementation. This change does not alter `/mantle/store`, custom prefixes, or native derivation identities.

## Parity and cutover

The first phase uses `nix-derivation` as a test oracle. It compares canonical ATerm bytes, parsed graph facts, derivation hashes, store paths, structured attributes, and stable diagnostics.

The corpus includes current Mantle Nix fixtures and pinned Nix 2.34 fixtures. Positive cases include traditional and versioned ATerms, every output variant, structured attributes, duplicate semantics, and nested dynamic inputs.

Negative cases include malformed, truncated, oversized, over-depth, invalid-name, wrong-prefix, wrong-domain, invalid structured-data, and incomplete-closure inputs.

A mismatch blocks cutover until the lifecycle record classifies it. Existing behavior is not authoritative when it differs from pinned Nix behavior.

The first production cutover covers `/nix/store` inputs only. Guix and other prefix-rewrite paths remain on the existing implementation until their own dual-run parity passes.

## Rollback

The adoption evidence records the prior Mantle revision, dependency state, adapter state, and fixture identities. Rollback restores the adapter and dependency state together.

Removing the new package without restoring the old call sites is invalid. Keeping the package while bypassing the adapter is also invalid.

## Claims

Passing parity validates agreement on the recorded corpus and versions only. It does not prove arbitrary Nix compatibility, evaluator parity, build success, output correctness, store trust, or release eligibility.
