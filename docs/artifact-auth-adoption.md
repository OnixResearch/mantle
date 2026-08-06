# Artifact-auth compatibility for action results

Mantle pins `artifact-auth-core` and `artifact-auth-ed25519` from `OnixResearch/onix-artifact` at revision `c932138d880ddf4c2967f4c024b489b5c0022bf1`. Cargo and the non-flake Nix input must resolve that revision. Flake evaluation checks the complete four-package source workspace and limits Mantle's consumer graph to the two authentication packages. It rejects mixed sources, sibling paths, widened graphs, and incompatible licensing. The predecessor Radicle source remains historical evidence only.

## Pure dual-run boundary

`crunch_action_result_core::artifact_auth` maps a validated action-result record, legacy candidate decision, explicit signer/currentness observations, and separate standalone cryptographic observations. It binds the result subject, output-object parents, publication-policy context, full-key identities, distinct-key threshold, required labels, and an optional complete OCI/metadata SHA-256 pair.

A legacy detached signature or verified signer label cannot prove `artifact-auth.statement.v1`. Each signer therefore supplies an explicit `CryptographicObservation` for the exact standalone statement. Duplicate full keys, missing required labels, malformed typed refs, incomplete OCI pairs, identity/decision/non-claim drift, and unrelated rejection causes block compatibility.

Compatibility fixes `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`. Mantle retains repository authorization, registry routing, credentials, signing, OCI construction, cache/build admission, receipts, and release policy.

## Exact shell pilot

`crunch_action_result_core::artifact_auth::map_mantle_artifact_auth_statement` is the pure source of the signer-specific standalone preimage. `crunch_build::artifact_auth` requires an admitted candidate decision, the exact signed action-result record, a matching producer/key trust basis, a separately verified legacy record signature, current key state, and the full BLAKE3 identity of the Nix-compatible Ed25519 public key before signing.

The legacy signature authorizes use of the existing product key but is never reused as standalone proof. The shell signs canonical standalone bytes separately, records BLAKE3 references for statement/public-key/signature/authorization carriers, and independently calls pinned `artifact-auth-ed25519`. Evaluation recomputes every carrier reference and preserves stable signature failure classes. The authorization reference is an external observation over the legacy signature carrier; it is not included in the standalone preimage.

The pilot uses supplied in-memory product observations and keys. It performs no key-file discovery, filesystem/network access, currentness refresh, registry authorization, publication, persistence, deployment, or release decision. Deterministic tests therefore do not admit standalone authority.

## Local operational receipt

`crunch_build::artifact_auth` now captures a bounded operational receipt only after separate standalone signing and verification pass. The product shell derives signer currentness from the full trusted verifying keys, validity window, expected policy identity, and revoked full-key digests in a Mantle trust snapshot. `portable_receipt::artifact_auth_trust_snapshot` maps the repository-owned `ReceiptBundle` and `TrustVerificationContext` into that snapshot; same-name key substitution, stale validity windows, policy drift, unknown keys, and revocation fail closed.

The receipt is created immutably at `action-results/v1/artifact-auth/<statement-digest>.json`, is bounded to one MiB, rejects symlink or non-regular-file substitution, and is reopened and validated before capture returns. Replay reloads the receipt, recomputes trust/currentness from a fresh context, independently reruns exact standalone verification, and requires every persisted carrier, decision, authority flag, non-claim, and BLAKE3 identity to match.

This is real local action-result persistence, not remote trust discovery or proof of revocation freshness. Registry publication, cache/build admission, deployment, release eligibility, and standalone authority remain outside the receipt. Passing replay keeps `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`.

## Cross-consumer readiness

| Consumer | Exact separate signature | Product authorization guard | Independent verifier | Current operational receipt | Standalone authority |
|---|---:|---:|---:|---:|---:|
| Molten | yes | capability/key-currentness guard | yes | no | unadmitted |
| Valence | yes | Radicle actor/operation guard | yes | no | unadmitted |
| Mantle | yes | action-result decision/signature/currentness guard | yes | local action-result receipt | unadmitted |

All three pilots reject legacy-preimage signature reuse and retain rollback. Mantle now proves bounded local persistence and fresh-context replay, but it does not prove remote trust discovery, revocation freshness, registry publication, or release operation. Published Molten and Valence operational receipts plus cross-consumer parity review remain required before any authority-admission change.

## Update and rollback

1. Review the candidate Artifact revision and package boundaries.
2. Change exact Cargo and Nix revisions together.
3. Regenerate `Cargo.lock` with Cargo and `flake.lock` with Nix. Never edit either lock manually.
4. Run positive and negative action-result, source, Clippy, Tiger Style, Cairn, and Nix checks.

Runtime rollback stops supplying standalone observations and continues legacy candidate admission. Dependency rollback restores the last reviewed Cargo/Nix declarations as one VCS change, regenerates both locks with their owning tools, and preserves the compatibility evidence explaining the rejection.

Standalone success proves neither artifact/OCI truth, repository authorization, trust-root correctness, revocation freshness, cache/build correctness, provenance, deployment safety, nor release eligibility.
