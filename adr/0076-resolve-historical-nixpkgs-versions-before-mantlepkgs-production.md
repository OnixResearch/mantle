# ADR 0076: Resolve historical Nixpkgs versions before Mantlepkgs production

## Status

Accepted

## Context

Mantlepkgs generates package catalogs from exact Nixpkgs source locks. One generation has one source revision and one source-tree BLAKE3 identity.

Operators can need a package version that is absent from the current Nixpkgs revision. `nixpkgs-multiverse` demonstrates a compact attribute-and-version index over historical channel revisions.

Its flake API lazily fetches selected revisions during Nix evaluation. Direct adoption would place package discovery, network materialization, and external API semantics inside the consumer path.

Mantle ADR 0010 keeps frontend package policy above Mantle. ADR 0056 keeps Mantlepkgs consumers independent from Nix after explicit production.

## Decision Drivers

- Resolve requested package versions to exact, reviewable Nixpkgs source facts.
- Preserve the Nix-free Mantlepkgs consumer boundary.
- Keep package-version observations separate from source and output identity.
- Reuse existing one-source generation and multi-shard domain composition.
- Bind Nix interoperability hashes without changing Mantle BLAKE3 meaning.
- Reject unavailable, ambiguous, stale, or cross-system resolution inputs.

## Decision

Mantlepkgs will adapt the compact historical revision-index design before catalog production.

A typed request will bind the target system, attribute, reported version, public selector, selection policy, and named limits.

A bounded producer shell will observe declared channel revisions for one system and attribute allowlist. The initial method will require the package `version` attribute.

A pure core will validate observations and resolve the newest sampled published revision that reports the requested version. The policy name will remain explicit.

Each result will use a separate versioned resolution receipt. It will bind the exact revision, Nix SHA-256 `narHash`, index BLAKE3, observation-set BLAKE3, and reported version.

The producer will recheck the exact selected revision before generation. It will compute the source-tree BLAKE3 used by the existing Mantlepkgs source lock.

The source identity binds relative paths, file bytes, executable bits, directories, and symlink text. It does not follow symlinks and rejects special files.

Requests will be grouped by exact revision. Each group will use one existing Mantlepkgs generation, and domain composition will combine the resulting shards.

Public selectors will include versions when several versions coexist. An unversioned alias will require one explicit unique default.

A pinned `nixpkgs-multiverse` revision may serve as comparison evidence. It will not become a flake input, runtime dependency, or trust authority.

OnixOS may consume accepted versioned catalog entries and resolution receipts. It will not generate revision indexes or run historical Nixpkgs discovery.

## Alternatives Considered

### Adopt the upstream flake directly

Rejected because it introduces external flake semantics, network-backed evaluation, and weaker receipt boundaries into package use.

### Add many historical Nixpkgs flake inputs

Rejected because all declared inputs can become eager metadata and maintenance surfaces. The cost does not remain proportional to selected revisions.

### Extend one Mantlepkgs manifest with several source locks

Rejected because the existing generation model has one exact source lock. Domain shards already provide the required composition boundary.

### Treat the package version as package identity

Rejected because one reported version can occur in several revisions with different sources, patches, dependencies, and outputs.

### Resolve versions in OnixOS

Rejected because Nixpkgs graph discovery and exact source preparation belong to Mantlepkgs production. OnixOS owns package intent and target admission.

### Parse versions from derivation names

Rejected because package names can be ambiguous and policy-dependent. The first implementation requires an explicit package `version` field.

## Consequences

- Mantle gains a new bounded producer workflow and versioned resolution receipt.
- Existing Mantlepkgs v1 manifests and no-Nix consumers remain compatible.
- Index production is system-specific and can require substantial Nix evaluation.
- Historical revisions can fail with the current Nix implementation.
- SHA-256 remains required for Nix `narHash` interoperability.
- Mantle-owned indexes, receipts, plans, and artifacts continue to use BLAKE3.
- Success does not prove package correctness, compatibility, cache retention, evaluator parity, reproducibility, deployment safety, or release eligibility.

## References

- <https://github.com/fzakaria/nixpkgs-multiverse>
- <https://fzakaria.com/2025/04/07/nixpkgs-multiverse>
- [ADR 0010](0010-keep-mantle-build-tool-boundary.md)
- [ADR 0056](0056-generate-mantlepkgs-from-concrete-package-graphs.md)
- [ADR 0057](0057-keep-composition-plans-concrete-and-frontend-neutral.md)
- [ADR 0062](0062-adapt-ekala-package-maintenance-patterns-without-transferring-authority.md)
