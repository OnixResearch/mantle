# Proposal: two-output release witness rebuild

## Summary

Repair `mantle release witness-rebuild` so the checked-in witness helper can produce witness sidecars for refreshed provider-bound release bundles that publish both the provider-bound Mantle binary and the self-hosting stage2 Mantle binary.

## Motivation

The refreshed `provider-bound-release-evidence-2026-06-26` bundle publishes two binaries. The publisher/witness handoff path verifies and exports the request, but the full witness helper currently fails closed after a successful self-hosting proof because it assumes the supported workflow produces exactly one rebuilt output. Operators then have to fall back to portable replay material, which proves the signing/import path but cannot be claimed as an independent external rebuild.

## Scope

- Teach witness rebuild planning/collection to map a self-hosting proof bundle to all release outputs it can support.
- Preserve fail-closed behavior when an expected release output has no trustworthy rebuilt source, when counts diverge, or when any digest mismatches.
- Add positive and negative coverage for two-output witness collection and final digest validation.
- Record validation evidence without claiming a successful fresh external rebuild until the repaired helper has been rerun.

## Non-goals

- No new release evidence schema version.
- No weakening of digest comparison or release-attestation matching.
- No claim that portable replay is independent external rebuild evidence.
- No change to release signing key policy or witness trust semantics.

## Target Spec Domains

- `verification-evidence` for proof-before-claim and release witness evidence boundaries.
