# Design: Admit Octet's current producer layout in the verification bridge

## Context

`crunch-verification-bridge-core` is the `no_std` + `alloc` identity consumer;
`crunch-verification-bridge` performs the bounded filesystem reads. The bridge
consumes externally authored bytes and never reimplements producer policy.

Two measured defects blocked the real producer output:

| Defect | Recorded-fixture assumption | Current producer |
| --- | --- | --- |
| Provider profile | `proofs/aeneas/provider-profile.json` constant | rail-named directory; receipt `artifact_identities` row |
| Tool graph | legacy `nix derivation show` map | `{"version": 4, "derivations": {basename: …}}` |

In both realized receipts the provider profile appears once in
`artifact_identities`, with `blake3` equal to
`consumer.provider_profile_identity` (prefix-free in the report row, `b3:` in
the consumer binding).

## Decisions

### Decision: Derive the provider-profile locator from the receipt

**Choice:** `octet_provider_profile_locator(receipt: Value) -> Result<String, BridgeError>`
takes and returns owned values, because the no-std core API-shape check bans
references in public boundary types. In-crate callers share a borrowed helper
with the same rule. The rule
scans at most `MAX_MATERIALS` `artifact_identities` rows, keeps those whose
final locator component is `provider-profile.json` and whose `blake3` equals
`consumer.provider_profile_identity` (ignoring the `b3:` prefix), requires
`is_relative_path`, and returns the sole match. Zero matches yield
`MissingProviderProfileLocator`; more than one yields
`AmbiguousProviderProfileLocator`; an unsafe path yields the existing
`UnsafeArtifactPath`.

**Rationale:** The receipt already carries the authoritative location as data.
A constant encodes one fixture's directory name and drifts with the producer.
Uniqueness is required so the checked-material list names exactly one profile.

**Rejected alternative:** Accepting any `provider-profile.json` anywhere in the
material set. That would let an unrelated file satisfy the identity check.

### Decision: Accept exactly the two producer graph shapes

**Choice:** A top-level object with a `version` key follows the versioned rules:
`version == 4`, `derivations` object keyed by store basenames ending in `.drv`,
each node `version == 4`, `inputs.drvs` values with string `outputs` and empty
`dynamicOutputs`, `inputs.srcs` an array of store basenames, dependencies inside
the closure, and the root looked up by the final path component of
`root_derivation`. Any other top-level object follows the unchanged legacy
rules, and a legacy node carrying `version` or `inputs` is rejected as mixed.

Stable error variants: `UnsupportedToolGraphVersion` (version other than 4),
`MixedToolGraphShape` (legacy keys in a versioned node or versioned keys in a
legacy node), `NonEmptyDynamicToolOutputs` (a dynamic output selection), plus
the existing shape, bound, closure, and source variants.

**Rationale:** Nix 2.36 emits the versioned document, Octet's exporter mirrors
this validation, and the producer job declares both formats. Mirroring the
producer's rules keeps admission on the producer contract rather than on
Mantle's own guess. `tool_graph_identity` remains the digest of the exact graph
bytes, so the wire format is visible in the identity.

**Rejected alternative:** Converting the versioned graph to the legacy shape
before admission. That would rewrite producer bytes and change the recorded
`tool_graph_identity`.

### Decision: Share one locator between core and shell

**Choice:** `octet_material_paths` calls
`crunch_verification_bridge_core::octet_provider_profile_locator`; the core's
`check_octet_provider_profile` takes the locator it already selected.

**Rationale:** Two copies of the rule would drift. The shell only reads bytes;
the decision stays in the functional core.

## Risks / Trade-offs

- A versioned graph is larger to walk than the legacy map by key length alone;
  the graph byte bound (256 MiB) and the 32,768-derivation bound still cap it.
- Requiring a unique provider row is stricter than the recorded fixture, which
  had exactly one. Real receipts satisfy it.
- The locator scan is `O(MAX_MATERIALS)` per observation, which is negligible
  against the bounded material reads.

## Verification

- Positive: the recorded fixture still replays; a real-layout copy with the
  rail-named proof directory passes; a synthetic versioned graph and the legacy
  graph are admitted.
- Negative: provider row missing, duplicated, or digest-mismatched; an escaping
  locator; a versioned node carrying `inputDrvs`; top-level `version: 5`; v4
  root absent; v4 dependency outside the closure; v4 dynamic output non-empty;
  a legacy node carrying `version`.
- Real run: `octet-job` on the realized pilot output exits 0 with
  `proof_acceptance: externally-reported-by-octet`.
