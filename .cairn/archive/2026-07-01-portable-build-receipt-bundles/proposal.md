# Proposal: Portable build receipt bundles

## Summary

Add a portable receipt bundle for remote/offline build handoffs. Operators should be able to package the receipts, artifact attestations, action refs, semantic graph edges, and trust-basis metadata that explain a build result, carry them with a store archive or remote-build output, verify them without the original source checkout, and import them into local state only when the output identity and trust policy match.

## Motivation

Remote and offline transports can move bytes, but bytes alone do not preserve Mantle's stronger explanations. A store archive can carry PathInfo and payloads; a P2P builder can return signed outputs; source bundles can prove input availability. The remaining evidence gap is portable explanation: after a handoff, `mantle why`, build-correctness receipt checks, artifact attestation verification, and proof-before-claim summaries should still know which action, source bundle, sandbox policy, producer, signatures, and output objects were involved.

Without a first-class receipt bundle, remote/offline users can end up with importable outputs but incomplete evidence, leading to overbroad claims or dead semantic graph queries. Mantle should keep receipt portability separate from output import and require verification before treating imported evidence as proof.

## Scope

- Define a versioned portable receipt bundle that can reference or embed action receipts, source-bundle refs, PathInfo identities, artifact attestations, closure attestations, semantic graph edges, sandbox reports, output object refs, and trust-basis summaries.
- Bind trust-root snapshots, public key identities, policy hashes, revocation epochs, and expiration windows so portable verification does not silently use a different trust context than the handoff intended.
- Distinguish complete evidence chains from partial diagnostic bundles: source/input refs, action refs, sandbox/network policy, reference scans, output refs, attestations, and signatures must all be present for strong claims.
- Add `mantle receipt bundle export|list|verify|import` or equivalent API surfaces.
- Verify bundles without the original checkout by using content refs, copied source/archive/proof material, and declared policy hashes.
- Import verified receipt/graph material idempotently and only when it matches local output/store/source facts, current or explicitly replayed trust policy, and revocation state.
- Keep output bytes outside the receipt bundle unless explicitly embedded as a separate object payload; store archives remain the primary output-byte transport.
- Make missing receipt material fail closed for strong build-correctness or remote/offline support claims.

## Non-goals

- No replacement for store archive export/import of output payloads.
- No new trust root; receipt bundles explain and bind trust material but do not create trust by themselves.
- No claim that portable receipt verification proves compiler correctness, full reproducibility, or deploy success.
- No frontend-specific module semantics inside Mantle core.

## Target Spec Domains

- `portable-build-receipts` for receipt bundle format, verify/import behavior, semantic graph portability, and claim boundaries.
- `verification-evidence` may later receive cross-domain receipt-bundle proof-before-claim requirements if release-facing workflows depend on it.
