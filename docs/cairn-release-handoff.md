# Cairn release handoff

Mantle can package Cairn lifecycle evidence without claiming to own Cairn's
readiness semantics. Release assembly opens the descriptor and every referenced
artifact through no-follow capability roots, enforces a 1 MiB descriptor bound
and a 64 MiB per-artifact bound while reading, and measures the exact bytes with
BLAKE3. It also measures the pinned Cairn authenticated-input archive receipt.
It checks all declared typed identities, writes those already-measured bytes
under `cairn-handoff/` without reopening the source path, and records a
`mantle-cairn-release-handoff-validation-v2` receipt in `manifest.json`.

Create a descriptor with schema `mantle-cairn-release-handoff-input-v2`:

```json
{
  "schema": "mantle-cairn-release-handoff-input-v2",
  "authentication": {
    "schema": "mantle.cairn-authentication-dependency.v1",
    "change_name": "authenticate-stack-provenance-inputs",
    "cairn_revision": "f4a1f8df0d430c1b9431358a388ac1d3c1a823ec",
    "archive_manifest_blake3": "40ea9765488bd02e362f70d5c9c498932544c80f2231a70f9b5c3ef81cd7df83",
    "archive_mutation_receipt_blake3": "8a4250a7db47dd4c013467d65e5667af188aa66599a1b89df6788ef067598fa9",
    "archive_receipt_path": "cairn-authentication.json",
    "archive_receipt_digest_blake3": "bf33d82555c7bd3afcbc7adac743782327a5d08e536266f0dfda97d6fe342edf"
  },
  "rows": [{
    "artifact_id": "release-readiness",
    "role": "cairn-release-readiness-receipt",
    "schema_id": "cairn.release-readiness.v1",
    "artifact_path": "readiness.json",
    "artifact_digest_blake3": "<64 lowercase hex chars>",
    "cairn_policy_path": "policy.ncl",
    "cairn_policy_digest_blake3": "<64 lowercase hex chars>",
    "release_readiness_id": "<Cairn readiness id>",
    "covers": ["<requirement id>"],
    "non_claims": ["not release correctness"]
  }]
}
```

Paths are resolved relative to the descriptor. Supported role/schema pairs are
bounded and exact. Empty handoffs, duplicate artifacts, role/schema swaps,
fabricated digests, stale policy bytes, missing non-claims, stale Cairn revisions,
stale archive identities, tampered archive-receipt bytes, and overclaiming
language fail closed. The reviewed dependency receipt is checked in at
`cairn-policy/evidence/cairn-authenticated-inputs-archive-receipt.json`.

Pass the descriptor during assembly:

```bash
mantle release create \
  --release-id <release-id> \
  --binary <stage2-mantle> \
  --proof-bundle <self-hosting-proof-bundle> \
  --cairn-handoff <handoff.json>
```

The validation receipt binds the measured handoff to the exact release id,
source archive digest, complete binary digest set, self-hosting proof-bundle
digest, prerequisite-inventory digest, and a domain-separated BLAKE3 identity
of the complete non-Cairn manifest projection. `mantle release verify` reopens
the bundle-local Cairn files with the same bounded no-follow policy, remeasures
them, and revalidates that binding. Copying a receipt to another release or
bundle, removing it under a required profile, replacing a path with a symlink,
or changing artifact or policy bytes is rejected.

## Onix admission

`mantle release verify --release-profile onix-stack` requires all of:

- valid bundle-local Valence stack-provenance evidence;
- a valid same-bundle Cairn handoff validation receipt bound to the pinned
  archived Cairn authenticated-input prerequisite; and
- deterministic release proof evidence accepted by Mantle's existing strict
  hermetic policy, including strict hermeticity mode, successful sandbox
  isolation checks, host/network denial, perturbations, normalization controls,
  matching output digests, and no degraded audit events.

Practical/impure or host-influenced deterministic evidence is therefore not
admitted by the Onix profile. Generic verification keeps a missing Cairn handoff
advisory.

## Authentication boundary

The receipt records
`authentication_status: "archive-authentication-prerequisite-bound-v1"`. Mantle
remeasures the exact reviewed dependency receipt, binds it into the same release
bundle and validation receipt, and rejects legacy handoffs that omit it. The
pinned receipt names Cairn commit `f4a1f8df0d430c1b9431358a388ac1d3c1a823ec`,
the archived `2026-07-14-authenticate-stack-provenance-inputs` package, and its
archive mutation and package-manifest identities.

This status proves only bundle-local measured identity, typed role/schema
linkage, Cairn policy identity, readiness/coverage linkage, same-bundle binding,
and presence of the reviewed archived authentication prerequisite. Mantle does
**not** independently re-run producer-signature verification and does not claim
producer authorization, Cairn correctness, source correctness, build
correctness, release correctness, deployment safety, verifier soundness, or
Cairn lifecycle readiness.
