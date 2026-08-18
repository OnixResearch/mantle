# Design: Mantle artifact-auth operational receipt

## Goal and completion evidence

Complete when Mantle can derive signer state from its portable-receipt trust context, write a content-identified standalone receipt beside local action-result state, reopen it, recompute exact standalone verification, reject trust/carrier/file drift, and retain legacy authority.

False completion includes test-only in-memory reports, caller-declared `Current`, self-asserted receipt hashes, write-without-reload tests, name-only key matching, revocation bypass, or authority promotion.

## Portfolio registry

| Family | Mechanism | State | Evidence or blocker |
|---|---|---|---|
| portable-trust | Reuse `TrustVerificationContext` full keys, revocations, validity time, and policy hash | active | Existing receipt verification already rejects wrong same-name keys, revoked digests, and invalid windows. |
| signing-key-file | Treat possession of Mantle's private key file as currentness | falsified | Possession does not prove configured trust or non-revocation. |
| release-policy | Reuse release-attestation signer names and witness revocations | falsified | That policy signs a different subject and does not carry full release-key material. |
| external-status | Add a new online status service | blocked | New network/trust authority is outside this bounded change. |

## Functional core and shell

Pure functions derive a normalized trust observation, construct receipt identity material, and validate all duplicated refs, authority flags, and BLAKE3 identity. The local action-result store owns directory creation, bounded file I/O, immutable publication, and reload orchestration.

## Authority boundary

A passing receipt proves only that exact standalone bytes were signed, persisted, reloaded, and independently checked under the supplied Mantle trust snapshot. Action-result/cache/build/release decisions remain authoritative. Fresh remote trust and revocation discovery remain non-claims and blockers for cutover.

## Audit risks and budgets

Audit same-name key substitution, full-key digest profile confusion, stale validity windows, revoked-key replay, receipt self-hash exclusion, path traversal, replacement, truncation, malformed JSON, carrier drift, and false parity. Retrieval is bounded to existing action-result, portable-receipt, and artifact-auth surfaces; no new dependency or network authority is allowed.
