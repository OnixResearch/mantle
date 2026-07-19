# Artifact-auth compatibility for action results

Mantle pins `artifact-auth-core` from `ssh://git@github.com/OnixResearch/artifact-auth.git` at revision `799459346d5416fbd7b9f55840a7371441b55afa`. Cargo and the non-flake Nix input must resolve that full revision. Flake evaluation rejects duplicate lock packages, source mismatch, or a standalone license other than `MIT OR Apache-2.0`; Crane vendors the exact reviewed input without a sibling checkout. SSH credentials authorize retrieval only.

## Pure dual-run boundary

`crunch_action_result_core::artifact_auth` maps a validated action-result record, legacy candidate decision, explicit signer/currentness observations, and separate standalone cryptographic observations. It binds the result subject, output-object parents, publication-policy context, full-key identities, distinct-key threshold, required labels, and an optional complete OCI/metadata SHA-256 pair.

A legacy detached signature or verified signer label cannot prove `artifact-auth.statement.v1`. Each signer therefore supplies an explicit `CryptographicObservation` for the exact standalone statement. Duplicate full keys, missing required labels, malformed typed refs, incomplete OCI pairs, identity/decision/non-claim drift, and unrelated rejection causes block compatibility.

Compatibility fixes `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`. This change rejects runtime cutover until a later change wires exact shell verification and current operational evidence. Mantle retains repository authorization, registry routing, credentials, signing, OCI construction, cache/build admission, receipts, and release policy.

## Update and rollback

1. Review the candidate standalone release and `config/consumers/mantle.ncl`.
2. Change exact Cargo and Nix revisions together.
3. Regenerate `Cargo.lock` with Cargo and `flake.lock` with Nix; never edit either lock manually.
4. Run action-result positive/negative tests, strict first-party Clippy/Tiger Style, Cairn validation, source checks, and `nix flake check`.

Runtime rollback stops supplying standalone observations and continues legacy candidate admission. Dependency rollback restores the last reviewed Cargo/Nix declarations as one VCS change, regenerates both locks with their owning tools, and preserves the compatibility evidence explaining the rejection.

Standalone success proves neither artifact/OCI truth, repository authorization, trust-root correctness, revocation freshness, cache/build correctness, provenance, deployment safety, nor release eligibility.
