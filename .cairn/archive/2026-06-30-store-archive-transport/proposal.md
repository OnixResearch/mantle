# Proposal: Store archive transport

## Summary

Add a Mantle-native streaming store archive transport inspired by Determinate Nix 3.12.0's `nix nario` work. Operators should be able to export a selected output closure to one file, carry it to an offline or air-gapped machine, list the file contents without import, and import only missing paths with bounded memory while preserving signed PathInfo, content-addressed output metadata, store-prefix identity, artifact attestations, and closure facts.

## Motivation

Mantle can already publish and consume flat binary-cache directories and HTTP caches, but those flows assume a reachable cache service or a directory layout with many files. Offline transfer currently lacks a first-class single-file closure artifact. The old Nix `nix-store --export` shape is specifically what we should avoid: it is hard to inspect, weak around newer metadata, and can force importers to buffer or scratch-write large payloads before deciding whether a path is already present.

A Mantle archive transport should adopt the useful nario-v2 property: metadata is available before payload bytes, so import can validate and skip already-present paths cheaply. Mantle should keep this native to its store-prefix, CAS, PathInfo, attestation, and substitution semantics rather than depending on `nix-store`, `/nix/store`, or Nix daemon behavior.

## Scope

- Define a versioned Mantle store-archive format with a bounded header, deterministic path records, metadata-before-payload ordering, and explicit compatibility claims.
- Add `mantle store archive export`, `mantle store archive import`, and `mantle store archive list` CLI/API surfaces.
- Export recursive closures from local PathInfo and closure metadata, including root selectors, references, signatures, CA metadata, NAR hashes/sizes, artifact attestation refs when present, and store-prefix identity.
- Import archives with bounded memory, idempotent skip of already-present paths, signature/hash/prefix verification, PathInfo persistence, and output materialization when requested.
- List archive contents quickly without mutating store state or materializing payloads.
- Add positive and negative tests for large payload streaming, skip-existing import, signature/hash/prefix failures, malformed frames, and unsupported compatibility claims.

## Non-goals

- No dependency on `nix-store --export`, `nix-store --import`, `nix copy`, or the Nix daemon.
- No claim of byte-compatible Determinate/Nix nario v2 import/export unless fixture-based compatibility tests prove that exact format.
- No weakening of Mantle's signed PathInfo, store-prefix, artifact attestation, or substitution trust policy.
- No remote-cache service requirement; this is a single-file or stream transport for offline handoff.
- No frontend-specific deploy semantics; archives carry build/store facts, not Onix module meaning.

## Reference

Determinate Nix 3.12.0 introduced `nix nario` and nario format version 2 to solve offline closure transfer with lower importer memory, metadata preservation, fast repeat import, and archive listing. Mantle should adopt those properties in a native archive transport while keeping compatibility claims evidence-gated.

Source reviewed: <https://determinate.systems/blog/changelog-determinate-nix-3120/>

## Target Spec Domains

- `store-transports` for streaming store archive format, export, import, list, and compatibility-proof requirements.
- `verification-evidence` for proof-before-claim behavior around archive compatibility and transfer support claims if implementation evidence needs a cross-domain hook later.
