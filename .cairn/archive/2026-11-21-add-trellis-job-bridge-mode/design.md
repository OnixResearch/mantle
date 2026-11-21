# Design: Admit a Trellis-owned Verus producer job in the verification bridge

## Context

`crunch-verification-bridge-core` is the `no_std` + `alloc` identity consumer;
`crunch-verification-bridge` performs bounded filesystem reads and publication.
The bridge consumes externally authored bytes and never reimplements producer
policy.

The producer contract for the WP8 pilot, measured by Terrane:

| Document | Schema | Fields the bridge reads |
| --- | --- | --- |
| `profile.json` | `trellis-verus-producer-profile/v1` | `consumer.provider.profile_identity`; `consumer.source.paths[{path,identity}]`; `provider_tools`; `extraction`; `artifacts[{role,locator}]`; `verus.receipt_path` (optional `verus.receipt_identity`) |
| receipt | `octet-verus-proof-receipt/v1` | `status`, `checked_scope`, `source_bindings[{path,blake3}]`, `stdout_artifact`, `stderr_artifact`, `proof_artifact_identity` |
| `producer-report.json` | `trellis-verus-producer-report/v1` | only `schema_version`; the exact bytes are the report identity |
| `job.json` | `trellis-mantle-producer-job/v1` | `owner`, `profile.identity`, `provider.identity`, `execution.rail_id`, `execution.network_allowed` |

Source paths are output-relative (`source/src/lib.rs`); receipt source
bindings are repository-relative (`src/lib.rs`) and carry only `{path, blake3}`.
The profile declares the artifact set as `{role, locator}` rows. The receipt
carries the identities: `stdout_artifact` and `stderr_artifact` rows with
`{locator, blake3, bytes, truncated}`, and `proof_artifact_identity` as
`<locator>#<blake3>` (the command record). `verus.command` therefore has no
`stdout_artifact`-style row; its digest comes from `proof_artifact_identity`.

## Decisions

### Decision: A fifth `trellis-job` mode, not a flag on `trellis`

**Choice:** Add `trellis-job OWNER_OUTPUT PRODUCER_EXPORT SOURCE_REVISION OUTPUT`
and `consume_trellis_job(root, producer_export, source_revision)`. The mode
reads `profile.json`, the receipt at the profile's `verus.receipt_path`,
`producer-report.json`, and every material the profile names: its source paths,
its artifact `locator`s, the receipt, the profile document itself, and the
fixed `proof-cards.json`/`evidence.json` documents. The profile document is
included as a material so the exported handoff is self-describing under the
producer's own names and replays with the exported directory as both roots.

**Rationale:** The pilot mode and the producer mode disagree on schemas,
outcome semantics, source binding, and artifact rows. A flag would force one
check sequence to branch on every document, and the wrong branch would be a
mislabel rather than a rejection. Packaging inputs reuse the Trellis derivation
(`consumer`, `provider_tools`, `extraction`) rather than a second convention.

**Rejected alternative:** Reusing `octet-mantle-producer-job/v1` output from
Terrane. That would claim Octet ownership of a Trellis job.

### Decision: Bind the report identity; do not interpret it

**Choice:** `observe_trellis_job` takes the exact `producer-report.json` material,
requires its schema version, records its BLAKE3 as
`Observation.external_report_identity`, pushes it and the uninterpreted
`proof-cards.json`/`evidence.json` into `checked_materials`, and extends the
association identity with the report digest under
`mantle.external-producer-report-association.v1`. `proof_acceptance` remains
`not-evaluated-by-mantle`; no receipt field is re-derived.

**Rationale:** Terrane's report is its own record and grants no acceptance. The
bridge needs a bound identity so Cairn can name the producer report without
pretending a verifier receipt is Mantle's verdict. The report, cards, and
evidence are hashed only; their internal claims remain external.

### Decision: One job binder parameterized by schema, owner, and receipt rail

**Choice:** `bind_octet_job` and `bind_trellis_job` call one
`bind_producer_job` with `JobExpectation { schema, owner, receipt_scope,
required_report }`. Octet keeps `/rail_id` and
`ProducerJobRequiresVerification`; Trellis uses `/checked_scope` and
`TrellisJobRequiresReport`. Everything after the rail check is unchanged:
denied network, bounded graph bytes, flake revision equal to the packaging
source revision, `check_tool_graph` (v4 or legacy, complete closure, rooted at
`root_derivation`), and the association extension with job/graph/root/lock
identities.

**Rationale:** The graph rules are producer-neutral and were already shared by
both formats. Duplicating them per owner would let the two copies drift and
would risk weakening the closure bound. The Octet association tuple and
`check_tool_graph` are byte-identical, so recorded Octet observations keep
their identities.

### Decision: Match the receipt's source bindings and artifact set structurally

**Choice:** Each receipt `source_bindings` row must name a profile source path
whose output-relative path equals the binding path or ends with `/{binding}`
and whose identity equals the binding digest; the selected profile source path
must be bound with `packaging.source_identity`. Each profile artifact row
declares a `role` and `locator`; the locator set must equal the artifacts the
receipt binds — `stdout_artifact`, `stderr_artifact`, and the
`<locator>#<digest>` in `proof_artifact_identity`. Every such artifact is an
exact material with the receipt's digest and recorded byte count; a
profile-declared `blake3`/`bytes` must agree when present; and the profile's
`verus.receipt_path` must name the selected receipt bytes (a declared
`verus.receipt_identity` must agree).

**Rationale:** This is the producer's measured contract: the profile declares
which artifacts exist, and Octet's Verus receipt carries the identities for the
two verifier streams and the proof command. Matching by digest and path suffix,
and by locator set, keeps both documents authoritative without inventing fields;
the receipt's source paths are repository-relative while the bridge materializes
the job's output-relative copies. An earlier synthetic fixture that duplicated
`blake3`/`bytes` into the profile rows and added a `receipt_identity` to the
profile did not match the real output and was corrected against the frozen
producer bytes.

## Risks / Trade-offs

- The bridge reads the profile twice (once for packaging inputs, once as a
  material). Both reads are inside the 1 MiB document bound and reject
  symlinks; the duplicate is the price of a self-describing export.
- The report schema is checked but its status is not interpreted. A report that
  disagreed with a `passed` receipt would still observe; the receipt is the
  verifier evidence and the report is recorded, not judged.
- Adding `Producer::TrellisJob` to the shell enum makes `consume_selected`
  reject that producer explicitly; the mode itself never takes the legacy
  path.
- `check_tool_graph` remains the only graph authority; if the producer adds a
  third wire shape, both modes reject it identically.

## Verification

- Positive: a synthetic output built in temporary directories following the
  design output contract observes with bound profile/report/job/graph
  identities and `not-evaluated-by-mantle`; the exported handoff replays with
  the exported directory as both roots.
- Negative: mutated profile source identity; failed receipt status; mutated
  receipt source binding; mutated receipt artifact row; unsupported profile,
  receipt, and report schemas; wrong job schema; wrong owner; wrong rail;
  network allowed; graph missing its root; incomplete graph; revision
  mismatch; tampered artifact bytes; missing report; missing evidence.
- Existing `octet`, `octet-verified`, `trellis`, and `octet-job` tests continue
  to pass unchanged.
