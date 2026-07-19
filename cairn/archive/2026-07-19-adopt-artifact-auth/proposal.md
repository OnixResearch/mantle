## Why

Mantle already owns OCI/action-result canonicalization, cache and repository authorization, trust selection, signing, registry transport, build admission, and release evidence. The independent `artifact-auth` repository now publishes a reviewed Mantle mapping profile at immutable revision `799459346d5416fbd7b9f55840a7371441b55afa`, but producer-side publication cannot silently migrate these Mantle-owned decisions.

A consumer-owned change is required to baseline Mantle's current signature behavior, review the profile against current code, select exact Cargo/Nix source identities, and dual-run before any standalone decision becomes authoritative. Until that work is accepted, Mantle's existing path remains authoritative.

## What Changes

- Review `config/consumers/mantle.ncl` and `fixtures/consumers/mantle.json` from the immutable standalone revision against current Mantle signature and action-result behavior.
- Pin Cargo and Nix to one reviewed standalone source only after repository-local dependency, licensing, and source-isolation checks pass.
- Add pure Mantle adapters for OCI subjects/parents, full-key identity, threshold, purpose, revocation/currentness observations, and compatibility identities.
- Dual-run positive and tamper cases, classify every difference, and retain a bounded rollback until cutover admission passes.
- Keep repository authorization, registry routing, credentials, signing, OCI semantics, build/cache admission, receipts, and release policy in Mantle.

## Impact

- **Planned surfaces**: Mantle signature/action-result cores and shell adapters, exact Cargo/Nix pins, compatibility fixtures, operator migration documentation, and release evidence.
- **No producer-side migration**: this package creates no implementation change by itself and does not authorize dependency or runtime edits until its tasks are executed and reviewed in this repository.
- **Claims**: standalone authentication remains one bounded input; it does not prove OCI truth, repository authorization, cache trust, build correctness, or release eligibility.
