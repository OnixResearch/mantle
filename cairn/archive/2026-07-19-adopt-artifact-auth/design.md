## Context

The standalone source is `ssh://git@github.com/OnixResearch/artifact-auth.git` at revision `799459346d5416fbd7b9f55840a7371441b55afa`. Its reviewed Mantle profile freezes source baseline `26c6880c08cd730c9e9420f92ea91a87212e492d`, shared fields, compatibility extensions, and retained authority. Current Mantle code may have advanced since that baseline, so the profile is review input rather than automatic proof of current compatibility.

## Goals and non-goals

Goals are exact-source consumption, a pure consumer adapter, positive and negative dual-run evidence, explicit drift classification, and a reversible cutover decision. This change does not delegate signing, credentials, OCI construction, registry access, repository authorization, cache/build admission, receipt composition, or release policy.

## Decisions

### Exact source and profile identity precede implementation

Cargo and Nix SHALL resolve one immutable reviewed revision. The adoption must bind the exact standalone profile and checked JSON projection from that revision, reject sibling paths/floating refs/duplicate package records, and record retrieval failures separately from source identity.

### Mantle owns translation and currentness observations

A pure adapter SHALL map the immutable OCI manifest pair, signature domain, signer/key labels, raw Ed25519 key identity, minimum distinct-key threshold, required labels, and revocation observations. Existing Nix-format key identity and OCI SHA-256 references remain explicit compatibility/interoperability metadata. Shell code retains bytes, files, registries, credentials, signing, and transport.

### Cutover is evidence-gated and reversible

Legacy and standalone paths SHALL run over identical measured observations. Equal or intentionally mapped preimage, identity, decision, issue, and non-claim differences may make cutover admissible; every unexplained difference blocks. The legacy path remains authoritative until a separate admitted repository-local cutover, and rollback remains available for a bounded release.

## Risks

- Label identity could be mistaken for full-key identity and inflate threshold counts.
- Standalone success could accidentally bypass repository authorization or cache/build admission.
- The frozen source baseline could be stale relative to current Mantle behavior.
- Private SSH retrieval could be confused with reviewed semantic authority.

Each risk is addressed through exact pins, full-key fixtures, product-owned composition, current baseline capture, fail-closed drift, and explicit non-claims.
