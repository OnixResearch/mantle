# Proposal: provider-bound witness replay artifacts

## Summary

Teach `mantle release witness-rebuild` to accept a fresh provider fixed-point proof bundle produced by the witness workflow and use its stage binary as a rebuilt candidate for provider-bound release artifacts, while still requiring digest-first matching before signing witness sidecars.

## Motivation

The refreshed provider-bound release publishes two artifacts with different evidence classes: a provider fixed-point Mantle binary and a self-hosting stage2 Mantle binary. The repaired multi-output collector can bind multiple outputs only when all matching digests are present in the self-hosting proof bundle. A real witness handoff needs a separate witness-produced provider proof candidate for the provider-bound artifact instead of reusing the publisher's bundled provider proof.

## Scope

- Expose a provider fixed-point proof output directory to the witness workflow environment.
- Include validated provider fixed-point stage binaries as rebuilt output candidates when the workflow produces that directory.
- Keep candidate selection by BLAKE3 digest and fail closed before witness signing when the provider proof is absent, invalid, path-escaping, or mismatched.
- Add positive and negative coverage for provider proof candidate collection.

## Non-goals

- No claim that the current retained request has been independently rebuilt until the full helper succeeds.
- No reuse of bundled publisher provider proof as witness output evidence.
- No new release attestation trust policy.

## Target Spec Domains

- `verification-evidence` for provider-bound witness replay evidence boundaries.
