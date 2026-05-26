# Proposal: Native Rust registry build-dependency topology planning

## Summary

Plan bounded vendored-registry `build-dependencies` from native facts and connect them to existing build-script/host-artifact topology rails without Cargo orchestration.

## Motivation

Mantle currently blocks nonempty `build-dependencies`, but real registry crates commonly use registry build dependencies for build scripts. Existing registry source and host-artifact execution rails make this a high-ROI bounded extension.

## Scope

- parse supported `[build-dependencies]` entries for vendored-registry packages from native manifests.
- require ready native registry source facts and ready native host-unit graph facts before execution claims.
- feed selected build-dependency host artifacts/metadata into existing host-artifact and unified topology receipts.
- emit deterministic blockers before `rustc` for unsupported build-dependency shapes, missing source facts, or ambiguous host/target behavior.

## Non-goals

- general Cargo build-dependency resolver compatibility.
- network/index access, `$CARGO_HOME`, or registry cache fallback.
- version solving or lockfile mutation.
- broad build-script emulation beyond existing supported metadata rails.

## Expected outcome

Mantle gains a bounded, receipt-backed native Rust planning slice with positive and negative CLI coverage, validated Cairn gates, and accepted spec synchronization after archive.
