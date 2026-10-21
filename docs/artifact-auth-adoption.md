# Artifact-auth compatibility for action results

Mantle declares `artifact-auth-core` and `artifact-auth-ed25519` from `ssh://git@github.com/OnixResearch/onix-artifact.git` at revision `5f3962d7471d1547e044f57daeda72f56621fd52`. Cargo and the non-flake Nix input must resolve that revision. Flake evaluation checks the complete four-package AGPL-3.0-or-later source workspace and limits Mantle's consumer graph to the two authentication packages. It rejects mixed sources, sibling paths, widened graphs, and incompatible licensing. The predecessor Radicle source remains historical evidence only.

The published cohort contains `artifact-auth-core`, `artifact-auth-ed25519`,
`artifact-binding-core`, and `artifact-transfer-core`; the latter two are not
Mantle dependencies. The pin update does not grant Artifact new authority or
change the authentication boundary below. Final Cargo/Nix lock and vendor
regeneration, exact source admission, and current-consumer execution remain
integration-owner checks, not results established by the declaration change.

The integration owner's fresh primary GitHub SSH acquisition measured NAR
`sha256-Xr7ocFkPN15XYb6NA4bv3I9Q42NvoSYtXWfsrj7I3WY=`
(`5ebee870590f375e5761be8d0386efdc8f50e3636fa1262d5d67ecae3ec8dd66`
in the reviewed recursive-SHA-256 source table). The source assertion and
current lock check require that exact NAR. The published core and Ed25519
`src/lib.rs` BLAKE3 identities are respectively
`839ea3e35429e2a1252020dd779b46ebd185b6911674036820da118d487d1d21`
and `7fd3987fe3ddc6d30efca37878952135f19905a00b52f349db237054be6b3633`,
unchanged from the prepared source. This source-byte observation is not a
consumer-execution, transferred-source gate, or release-acceptance claim.

The accepted 2026-08-06 `artifact-workspace-migration-v1` receipt and its pure
validator describe their original revision and license. They remain immutable
historical evidence, not a current-source allowlist or an automatic fallback.

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
