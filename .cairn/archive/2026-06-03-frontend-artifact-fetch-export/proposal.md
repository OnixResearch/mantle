# Proposal: Frontend artifact fetch/export

## Summary

Add a frontend-neutral Mantle boundary for fetching or exporting spec-admitted artifacts by artifact ref. Mantle already admits frontend artifact specs and records validation attestations; the next boundary is a generic way for callers to retrieve admitted artifact contents while preserving the spec proof and provenance. This enables frontends such as Onix to deploy `mantle://...` artifacts without requiring Mantle to understand Onix activation semantics.

## Motivation

Onix can now validate `mantle-onix-activation-closure` proofs, but its deploy path cannot move off `/nix/store` and `nix copy` until Mantle exposes a generic artifact transfer surface. That surface belongs in Mantle because Mantle owns build execution, artifact storage, content addressing, and provenance. It must remain frontend agnostic: artifact kinds such as `mantle-onix-activation-closure` are data admitted under frontend specs, not Mantle built-ins.

## Scope

- Define an admitted-artifact fetch/export request and result model keyed by artifact ref, spec-admission attestation, and expected artifact identity.
- Add fail-closed validation for missing admission proof, missing artifact ref, unsupported ref scheme, artifact/proof mismatch, and unavailable content.
- Preserve frontend artifact attestation and build provenance in export receipts or sidecars.
- Provide a CLI/API seam that frontends can call before their own deploy logic.
- Add positive and negative tests showing retrieval succeeds only for spec-admitted artifacts.

## Non-goals

- No Onix activation, role, tag, provider, inventory, or deploy-policy semantics in Mantle core.
- No `nixos-activation-closure` or `/nix/store` requirement for frontend artifacts.
- No silent fallback to `nix copy` or other ambient Nix runtime commands for `mantle://...` refs.
- No direct dependency on Snix in this change package. Snix/castore can inform the storage design, but Mantle must own its artifact boundary.
- No support claim for any frontend artifact kind without current admission and export evidence.

## Target Spec Domains

- `build-tool-boundary` for the frontend-neutral artifact fetch/export boundary.
- `verification-evidence` for proof-before-claim rules around artifact export and deploy-transfer claims.
