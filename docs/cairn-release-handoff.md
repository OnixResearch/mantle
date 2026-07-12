# Cairn release handoff

Mantle can package Cairn lifecycle evidence without claiming to own Cairn's
readiness semantics. Release assembly measures the referenced artifact and Cairn
policy bytes with BLAKE3, checks their declared typed identities, copies them
under `cairn-handoff/`, and records a
`mantle-cairn-release-handoff-validation-v1` receipt in `manifest.json`.

Create a descriptor with schema `mantle-cairn-release-handoff-input-v1`:

```json
{
  "schema": "mantle-cairn-release-handoff-input-v1",
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
fabricated digests, stale policy bytes, missing non-claims, and overclaiming
language fail closed.

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
of the complete non-Cairn manifest projection. `mantle release verify` remeasures
the bundle-local Cairn files and revalidates that binding. Copying a receipt to
another bundle, removing it under a required profile, or changing any bound
bytes is rejected.

## Onix admission

`mantle release verify --release-profile onix-stack` requires all of:

- valid bundle-local Valence stack-provenance evidence;
- a valid same-bundle Cairn handoff validation receipt; and
- deterministic release proof evidence accepted by Mantle's existing strict
  hermetic policy, including strict hermeticity mode, successful sandbox
  isolation checks, host/network denial, perturbations, normalization controls,
  matching output digests, and no degraded audit events.

Practical/impure or host-influenced deterministic evidence is therefore not
admitted by the Onix profile. Generic verification keeps a missing Cairn handoff
advisory.

## Authentication boundary

The receipt records `authentication_status: "not-authenticated"`. It proves only
bundle-local measured identity, typed role/schema linkage, Cairn policy identity,
readiness/coverage linkage, and same-bundle binding. It does **not** prove
producer authorization, source correctness, build correctness, release
correctness, deployment safety, verifier soundness, or Cairn lifecycle
readiness.

Authenticated promotion remains blocked until Cairn's active
`authenticate-stack-provenance-inputs` change is completed and archived with a
consumable receipt. Mantle rejects a handoff receipt that claims
`authentication_status: "authenticated"` before that dependency exists.
