# Proposal: Build realization routing policy

## Summary

Add a deterministic realization-routing planner that decides how Mantle should attempt a requested build across local cache, remote substitution, offline store archives, source-bundle readiness, P2P remote builders, and local execution. The planner should explain the chosen path and the rejected alternatives before any mutating work starts.

## Motivation

The remote/offline roadmap now has several independent transports: binary-cache substitution, delta/full substitution, store archives, source bundles, and P2P remote builders. Without an explicit routing policy, operators cannot predict why Mantle picked local build, remote build, import-first, substitute, or fail-closed offline behavior. Hidden fallback order would also weaken proof-before-claim behavior: a successful output might be attributed to the wrong trust basis.

Mantle already exposes build planning rows such as `cached`, `substitute`, `build`, and `preflight-error`. Remote/offline work needs that plan to become a first-class pure decision surface that includes trust roots, network policy, source/input readiness, archive availability, builder capabilities, and requested claim strength.

## Scope

- Define a pure routing model over requested roots, local store facts, source-bundle readiness, archive availability, remote builder profiles, substitution/trust policy, network/offline mode, platform/profile facts, and requested evidence strength.
- Extend build planning output with outcome classes such as local cache hit, trusted substitute, archive-import candidate, source-bundle-required, P2P remote-build candidate, local-build candidate, and fail-closed preflight error.
- Require remote-builder eligibility to depend on concrete evaluated build inputs, compatible capabilities, source/input availability or upload plan, output trust preflight, resource limits, and explicit upload/privacy policy.
- Require offline mode to fail closed when any selected route would need network access, ambient package-manager/build caches, undeclared source material, or untrusted output import.
- Make route ranking deterministic with named tie-breakers so route choice is replayable from the reported facts.
- Bind route eligibility to requested claim strength: strong action-correctness routes must require stronger receipts than practical cache/build routes.
- Record rejected alternatives with deterministic reason codes without leaking secrets.

## Non-goals

- No implementation of the store archive or P2P protocols themselves.
- No remote evaluation or frontend module lowering on builders.
- No automatic trust in any builder, archive, or cache just because a route exists.
- No heuristic route that silently falls back to ambient network, package-manager caches, build output directories, or language-specific caches under offline mode.
- No claim of build success from a route plan.

## Target Spec Domains

- `realization-routing` for deterministic build route planning, offline fail-closed behavior, remote-builder eligibility, and route report semantics.
