# Proposal: Frontend artifact spec admission

## Summary

Add a Mantle build-tool contract for frontend-provided artifact specs. Mantle should remain agnostic about frontend semantics while admitting artifacts whose manifests validate against a supplied Nickel, Rust, or equivalent spec. Build reports should attest the spec identity, validator identity, validation result, artifact ref, and provenance without hard-coding Onix or any other frontend artifact kind.

## Motivation

Onix wants a `mantle-onix-activation-closure`, but Mantle should not learn Onix activation semantics as a built-in platform. The right boundary is generic: a frontend supplies build inputs and an artifact spec; Mantle builds an artifact, validates the artifact manifest against that spec, and records evidence. Other frontends should be able to use the same mechanism for their own deployable artifact kinds.

## Scope

- Define a frontend artifact spec reference model with id, version, validator kind, content hash, and input refs.
- Define validation result and attestation fields that can be included in Mantle build reports.
- Require Mantle to treat artifact kinds as frontend data, not core enums, except for generic validation and reporting rules.
- Add fail-closed behavior for missing specs, hash mismatches, unsupported validators, invalid manifests, and attempted built-in interpretation of frontend kinds.
- Add positive and negative tests for spec admission and report attestation.

## Non-goals

- No Onix-specific activation semantics in Mantle core.
- No raw Onix inventory/module/provider interpretation.
- No implementation code in this change package.
- No default support claim for any frontend artifact kind without validation evidence.
- No unbounded or ambient validator execution; validators must be content-addressed and run through declared Mantle execution boundaries.

## Target Spec Domains

- `build-tool-boundary` for spec-admitted frontend artifacts and frontend-neutral artifact kinds.
- `verification-evidence` for proof-before-claim rules around spec-admitted deployable artifact claims.
