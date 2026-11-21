# 0168. Admit a Trellis-owned Verus producer job in the verification bridge

Status: Accepted

## Context

The bridge could observe two producer shapes: the Octet Aeneas handoff (six- and
five-argument modes) and Trellis's recorded Aeneas cross-verifier pilot
(`trellis` mode). Neither describes the WP8 producer job:

- The producer is Trellis, not Octet. Terrane authors
  `trellis-mantle-producer-job/v1` with owner `Trellis` and emits a
  `trellis-verus-producer-profile/v1` profile, an
  `octet-verus-proof-receipt/v1` receipt, and its own
  `trellis-verus-producer-report/v1` report.
- The existing `trellis` mode requires `trellis-aeneas-cross-verifier-profile/v1`
  and `trellis-aeneas-proof-receipt/v1`, a `supported` outcome, Aeneas
  extraction options, and an artifact row set shaped by the Aeneas pilot. A
  Verus job has none of those, and reusing the schema would mislabel the
  evidence.
- The bridge had no way to bind a non-Octet producer job graph: the job
  identity check hard-coded `octet-mantle-producer-job/v1` and owner `Octet`,
  and the rail check read the receipt's `rail_id` field.

The Cairn convergence pilot needs an observation that binds the Trellis job's
profile, receipt, report, and tool graph without pretending Mantle judged the
`bounds.clamp` theorem.

## Decision Drivers

- The producer's own output contract is the authority for paths, schemas, and
  identities; Mantle reads bytes, it does not invent a second contract.
- One observation must distinguish the externally reported Verus receipt from
  Mantle's own (absent) verdict; `proof_acceptance` stays
  `not-evaluated-by-mantle`.
- The job graph admission and its bounds are already producer-neutral and must
  not be duplicated or weakened.
- Fail-closed behavior with stable, distinguishable error variants.
- The no-std core API-shape rule bans references in public boundary types.

## Decision

- Add the fifth mode
  `mantle-verification-bridge trellis-job OWNER_OUTPUT PRODUCER_EXPORT SOURCE_REVISION OUTPUT`.
  `OUTPUT '-'` observes without exporting, exactly like the other modes.
- `crunch-verification-bridge::consume_trellis_job(root, producer_export,
  source_revision)` reads `profile.json`, the receipt at the path the profile's
  `verus.receipt_path` names, `producer-report.json`, and every material the
  profile names: its source paths, its artifact `locator`s, the receipt,
  `proof-cards.json`, and `evidence.json`. All reads stay inside the existing
  document, material, aggregate-byte, and count bounds and reject symlinks.
  Packaging identities are derived from the profile's `consumer.provider`,
  `consumer.source.paths[0].identity`, `provider_tools`, and `extraction`, reusing
  the Trellis derivation rather than a second convention.
- `crunch-verification-bridge-core::observe_trellis_job` checks, in order:
  profile schema `trellis-verus-producer-profile/v1`; receipt schema
  `octet-verus-proof-receipt/v1`; report schema
  `trellis-verus-producer-report/v1`; profile bytes hash to
  `packaging.profile_identity`; `consumer.provider.profile_identity` equals
  `packaging.provider_identity`; receipt `status == "passed"`; the profile's
  first source path is an exact material; every profile source path is an exact
  material; every receipt `source_bindings` row names a profile source path by
  digest and path suffix (the receipt binds the repository-relative path, the
  profile the output-relative copy), and the selected source path is bound with
  `packaging.source_identity`; `provider_tools`/`extraction` hashes equal the
  tool-contract and options identities; each profile artifact row declares a
  `role` and a `locator`, and the locator set equals the artifacts the receipt
  binds — its `stdout_artifact`, its `stderr_artifact`, and the
  `<locator>#<digest>` in `proof_artifact_identity`; every such artifact is an
  exact material with the receipt's digest and recorded byte count, a
  profile-declared `blake3`/`bytes` must agree when present, and the profile's
  `verus.receipt_path` names the selected receipt byte-for-byte (a declared
  `verus.receipt_identity` must also agree). The measured producer contract
  declares the artifact set in the profile and the identities in Octet's Verus
  receipt. `external_report_identity` is the BLAKE3 of the exact
  `producer-report.json` bytes, and the association identity is extended with
  it. `proof_acceptance` remains `not-evaluated-by-mantle` and no receipt field
  is re-derived.
- `crunch-verification-bridge-core::bind_trellis_job` shares one job binder with
  `bind_octet_job`: expected schema, owner, and the receipt field carrying the
  rail (`/rail_id` for Octet, `/checked_scope` for Trellis) are parameters. The
  Trellis job must declare `trellis-mantle-producer-job/v1`, owner `Trellis`,
  `execution.rail_id` equal to the receipt's `checked_scope`,
  `execution.network_allowed == false`, `profile.identity` and
  `provider.identity` equal to the observed packaging identities, a flake
  revision equal to `packaging.source_revision`, and a complete valid v4 or
  legacy tool graph rooted at `root_derivation`. `check_tool_graph` is
  unchanged. The association binds the job, graph, root derivation, flake
  metadata/lock, and the producer report.
- New stable error variants: `UnsupportedTrellisJobProfile`,
  `UnsupportedVerusProofReceipt`, `UnsupportedTrellisProducerReport`,
  `MissingTrellisProducerReport`, `MissingTrellisSourceBindings`,
  `TrellisSourceBindingCountBound`, `UnsupportedProducerJobSchema`,
  `UnsupportedProducerOwner`, and `TrellisJobRequiresReport`.
- `export` is unchanged, so the exported handoff carries the exact profile,
  receipt, report, cards, evidence, source, and artifact bytes plus the job
  metadata, and replays with the exported directory as both explicit roots.

Rejected alternatives:

- Reusing the `trellis` mode with new schemas behind a flag. The pilot profile
  and the producer profile disagree on source binding, artifact rows, and
  outcome semantics; one mode cannot truthfully cover both.
- Producing `octet-mantle-producer-job/v1` from Terrane to avoid the binder
  change. That would claim Octet ownership of a Trellis job.
- Treating the producer report as a Mantle verdict. It is Terrane's own record;
  the bridge hashes it and records the identity only.

## Consequences

- `trellis-job` on a producer output that satisfies the design's contract exits
  0 with `proof_acceptance: not-evaluated-by-mantle`,
  `external_report_identity` set, and bound job/graph identities.
- `octet-job`, `trellis`, `octet`, and `octet-verified` behavior is unchanged;
  the shared binder keeps `check_tool_graph` and the association domain for
  Octet byte-identical.
- A failed or blocked Verus receipt, a stale/tampered profile, receipt, report,
  or artifact, a wrong job schema, owner, rail, network declaration, revision,
  or an incomplete graph all fail closed with a stable variant.
- No theorem admission, proof registry, or verifier execution is added to
  Mantle. The real Terrane output and its realization are exercised in WP8
  stage 2; this change is validated with the synthetic contract fixture.
