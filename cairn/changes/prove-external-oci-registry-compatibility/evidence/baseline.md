# Baseline: External OCI registry compatibility

Date: 2026-07-17

## Baseline evidence

- Existing in-process registry rail: 11 pure-core tests and 8 production CLI tests passed during `enforce-registry-signature-trust` closeout.
- Existing external fixture: absent from the repository flake and test suite.
- Independent implementation selected: OCI Distribution `registry`, package `distribution-3.1.0`, from the repository-pinned Nix package set.
- Local realization probe: `nix build --no-link --print-out-paths .#oci-distribution-registry` produced the pinned Distribution 3.1.0 package.
- First production push attempt failed before image publication because Distribution rejected `application/vnd.oci.artifact.manifest.v1+json`.
- After companion conversion, a second attempt reached image publication and failed because the ordinary OCI image manifest used `schemaVersion: 1` instead of required version 2.
- These independent failures proved the local in-process server was too permissive and expanded the implementation task to correct the OCI wire shape rather than waive compatibility.

## Bounded mechanism portfolio

1. Public hosted registry: rejected because credentials, mutable service policy, network availability, and cleanup make evidence non-deterministic.
2. Containerized registry: rejected because it adds Docker/Podman daemon authority unrelated to the protocol under test.
3. Mantle in-process registry: retained for deterministic protocol/adversarial coverage but not independent compatibility evidence.
4. Pinned external `distribution` process: selected because it is an independent implementation, locally isolated, reproducible through the flake pin, and needs no daemon or credentials.
5. Multiple external registries: deferred until one bounded compatibility rail is stable; one implementation cannot support an arbitrary-compatibility claim.

## Success boundary

Success requires the production CLI to complete signed push and fresh-state immutable-digest pull/admission against the external process, preserve exact layout bytes and contracted reports, and fail closed without outputs for a wrong signature-manifest digest. Process startup alone or push alone is insufficient.
