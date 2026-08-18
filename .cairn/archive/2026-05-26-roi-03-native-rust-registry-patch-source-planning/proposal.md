# Proposal: Native Rust registry patch source planning

## Summary

Represent bounded `[patch.crates-io]` / local override source facts for registry package identities without Cargo fallback.

## Motivation

Vendored and patched registry sources are common in real crate graphs. Mantle should distinguish explicit local patch facts from unsupported ambient resolver behavior.

## Scope

- parse supported local `[patch.crates-io]` entries from native workspace manifests.
- bind patched package identity to lockfile/vendor or explicit local source digests.
- prefer explicit patched source facts when planning registry dependency edges.
- emit deterministic blockers for ambiguous patches, missing manifests, mismatched package identity, or unsupported patch source kinds.

## Non-goals

- general Cargo source replacement compatibility.
- git/network patch fetches.
- version solving or lockfile mutation.
- implicit Cargo cache fallback.

## Expected outcome

Mantle gains a bounded, receipt-backed native Rust planning slice with positive and negative CLI coverage, validated Cairn gates, and accepted spec synchronization after archive.
