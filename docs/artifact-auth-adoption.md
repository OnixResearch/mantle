# Artifact-auth compatibility for action results

Mantle pins `artifact-auth-core` and `artifact-auth-ed25519` from `ssh://git@github.com/OnixResearch/artifact-auth.git` at revision `799459346d5416fbd7b9f55840a7371441b55afa`. Cargo and the non-flake Nix input must resolve that full revision. Flake evaluation requires exactly that two-package set, rejects source mismatch or an incompatible standalone license, and makes Crane vendor the exact reviewed input without a sibling checkout. SSH credentials authorize retrieval only.

## Pure dual-run boundary

`crunch_action_result_core::artifact_auth` maps a validated action-result record, legacy candidate decision, explicit signer/currentness observations, and separate standalone cryptographic observations. It binds the result subject, output-object parents, publication-policy context, full-key identities, distinct-key threshold, required labels, and an optional complete OCI/metadata SHA-256 pair.

A legacy detached signature or verified signer label cannot prove `artifact-auth.statement.v1`. Each signer therefore supplies an explicit `CryptographicObservation` for the exact standalone statement. Duplicate full keys, missing required labels, malformed typed refs, incomplete OCI pairs, identity/decision/non-claim drift, and unrelated rejection causes block compatibility.

Compatibility fixes `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`. Mantle retains repository authorization, registry routing, credentials, signing, OCI construction, cache/build admission, receipts, and release policy.

## Exact shell pilot

`crunch_action_result_core::artifact_auth::map_mantle_artifact_auth_statement` is the pure source of the signer-specific standalone preimage. `crunch_build::artifact_auth` requires an admitted candidate decision, the exact signed action-result record, a matching producer/key trust basis, a separately verified legacy record signature, current key state, and the full BLAKE3 identity of the Nix-compatible Ed25519 public key before signing.

The legacy signature authorizes use of the existing product key but is never reused as standalone proof. The shell signs canonical standalone bytes separately, records BLAKE3 references for statement/public-key/signature/authorization carriers, and independently calls pinned `artifact-auth-ed25519`. Evaluation recomputes every carrier reference and preserves stable signature failure classes. The authorization reference is an external observation over the legacy signature carrier; it is not included in the standalone preimage.

The pilot uses supplied in-memory product observations and keys. It performs no key-file discovery, filesystem/network access, currentness refresh, registry authorization, publication, persistence, deployment, or release decision. Deterministic tests therefore do not admit standalone authority.

## Cross-consumer readiness

| Consumer | Exact separate signature | Product authorization guard | Independent verifier | Current operational receipt | Standalone authority |
|---|---:|---:|---:|---:|---:|
| Molten | yes | capability/key-currentness guard | yes | no | unadmitted |
| Valence | yes | Radicle actor/operation guard | yes | no | unadmitted |
| Mantle | yes | action-result decision/signature/currentness guard | yes | no | unadmitted |

All three pilots reject legacy-preimage signature reuse and retain rollback. None proves live trust discovery, revocation/currentness freshness, durable publication, or release operation. A cross-consumer authority-admission change remains blocked until those product-owned operational receipts exist.

## Update and rollback

1. Review the candidate standalone release and `config/consumers/mantle.ncl`.
2. Change exact Cargo and Nix revisions together.
3. Regenerate `Cargo.lock` with Cargo and `flake.lock` with Nix; never edit either lock manually.
4. Run action-result positive/negative tests, strict first-party Clippy/Tiger Style, Cairn validation, source checks, and `nix flake check`.

Runtime rollback stops supplying standalone observations and continues legacy candidate admission. Dependency rollback restores the last reviewed Cargo/Nix declarations as one VCS change, regenerates both locks with their owning tools, and preserves the compatibility evidence explaining the rejection.

Standalone success proves neither artifact/OCI truth, repository authorization, trust-root correctness, revocation freshness, cache/build correctness, provenance, deployment safety, nor release eligibility.
