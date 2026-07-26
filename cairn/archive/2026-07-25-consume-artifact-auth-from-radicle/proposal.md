# Consume artifact-auth from Radicle-backed HTTPS

## Why

Mantle already consumes the reviewed `artifact-auth` revision `799459346d5416fbd7b9f55840a7371441b55afa`, but Cargo and Nix fetch it from GitHub over SSH. The source has no registry release, so that transport remains an avoidable bootstrap dependency after the same Git object was accepted and published through the governed Radicle HTTPS adapter.

## Outcome

Change only the source transport to the accepted public Radicle HTTPS URL. Preserve the exact Git object, Nix content identity, package versions, crate graph, tests, and Mantle authority boundaries. Remove every executable GitHub fallback for this dependency and emit typed cutover evidence.

## Scope

- Align both Cargo manifests, `Cargo.lock`, `flake.nix`, and `flake.lock` on one Radicle HTTPS URL and exact revision.
- Preserve `artifact-auth-core` and `artifact-auth-ed25519` behavior and Mantle's existing action-result, signing, trust, cache, build, repository, and release authority.
- Validate focused positive and negative artifact-auth behavior, exact source admission, and fallback rejection.
- Retain GitHub only as historical prose where needed, never as an executable dependency fallback.

## Non-goals

This change does not migrate Mantle itself to Radicle, move Mantle to the unified `artifact` workspace, alter artifact-auth or Mantle Rust APIs, claim semantic equivalence, enable artifact-auth in Radicle CI, or grant source transport any product authority.

## Impact

- **Files**: Cargo and Nix manifests/locks, source-admission checks, documentation, typed evidence, and Cairn lifecycle artifacts.
- **Testing**: Focused artifact-auth tests before and after cutover, positive/negative receipt validation, lock/source agreement, formatting, and the smallest relevant Nix check.
