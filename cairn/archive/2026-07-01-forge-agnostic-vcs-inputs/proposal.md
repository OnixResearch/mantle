# Proposal: Forge-agnostic VCS project inputs

## Summary

Extend project inputs beyond Git with forge-agnostic Darcs, Pijul, and Fossil source kinds. Mantle should lock these inputs by VCS-native stable identities and fetch them through Mantle source/build services without privileged forge URL schemes.

## Motivation

Nixtamal's broader VCS support is a strong fit for Mantle's build-tool boundary. Git-only project inputs make non-Git source projects second-class and push users toward forge-specific shorthands. Mantle should treat VCSs as source transports with explicit repository URLs, selectors, mirrors, freshness probes, and lock metadata.

This also reinforces the policy that Mantle should not encode GitHub/GitLab semantics as special project behavior. Hosts and forges are endpoints; the VCS source kind owns the semantics.

## Scope

- Add project input kinds for Darcs, Pijul, and Fossil alongside Git.
- Define lock metadata for each VCS: Darcs context or weak-hash facts, Pijul channel/state/change facts, and Fossil check-in/tag/branch facts.
- Support mirrors where the underlying fetcher can verify the same locked identity.
- Integrate with freshness probes, fetch policy, source bundles, and generated Nickel inputs.
- Fail closed when required VCS tooling, identity data, or mirror verification is unavailable.

## Non-goals

- No forge-specific URL shortcuts such as privileged `github:` or `codeberg:` schemes.
- No recursive input resolution.
- No promise that every VCS feature, submodule-like feature, or server extension is supported in the first implementation.

## Target Spec Domains

- `project-workflows` for manifest, lockfile, refresh, and generated input semantics.
- A future accepted `source-transports` spec may receive lower-level source transport details after the current source-bundle work lands.
