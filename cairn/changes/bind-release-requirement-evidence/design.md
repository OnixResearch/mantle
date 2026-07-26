# Design: Bind release requirement evidence

## Goal

Make a Mantle release name the exact accepted requirements and evidence artifacts it carries, not only their text identifiers.

## Core model

`crunch-release-core` adds no-std compatible DTOs for:

- content-bound requirement coverage rows;
- Cairn registry and Valence reference identities;
- source, test, proof, and receipt evidence references;
- a canonical evidence manifest and strict-profile result;
- deterministic issues and explicit non-claims.

The DTO mirrors the reviewed Valence wire contract. Frozen positive and negative fixtures detect schema or identity drift.

## Release coverage

A strict coverage row binds the requirement reference, registry BLAKE3, Valence reference BLAKE3, evidence-manifest row identities, and declared evidence role.

Requirement, registry, source, test, proof, receipt, release, and binary hashes remain separate domains. A valid digest cannot substitute for another role.

The core rejects duplicate requirement rows and duplicate evidence refs before canonical hashing.

## Evidence manifest

The manifest contains safe repository-relative paths, BLAKE3 content identities, optional bounded spans and symbols, evidence roles, producer receipt refs, and non-claims.

Mantle verifies supplied evidence bytes in the std shell. The no-std core validates already-loaded rows and relationships.

A release does not require a live source checkout. Each row must resolve to a bundle member, source envelope, or externally verified sidecar whose exact bytes and parent receipt are available.

The existing `tools/release_provenance_tracey_refs.rs` file remains a readable compatibility index. It cannot satisfy a strict profile without matching content-bound manifest rows.

## Release verification

Strict policy requires the selected Cairn registry, typed Valence references, and all required evidence rows. Optional absent rows remain absent.

Present optional rows still require complete validation. Stale, malformed, unsupported, or wrong-repository rows fail closed.

Legacy `covered_requirement_ids`, `covered_source_ids`, and `covered_function_ids` remain readable under the existing compatibility profile. They do not become typed evidence automatically.

## Cross-repository boundary

Mantle consumes frozen contract fixtures in the fast lane. A separate selected-revision integration lane validates current Cairn and Valence outputs.

The integration receipt names exact producer revisions, schema identities, fixture hashes, and non-claims. It does not claim arbitrary-version compatibility.

## Verification

Pure tests cover valid canonicalization and relationship checks. Shell tests cover exact file bytes, missing files, stale content, unsafe paths, and no-partial-output behavior.

Release CLI tests cover strict and compatibility profiles. Cross-repository fixtures mutate one boundary at a time.

## Rollout

The new fields are additive in a versioned release schema. Strict policy becomes selectable before any future default promotion.
