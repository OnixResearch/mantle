# Verification Bridge Specification

## Purpose

Defines the `verification-bridge` capability.

## Requirements

### Requirement: The bridge locates producer material by its receipts

r[verification_bridge.octet_producer_layout] The verification bridge MUST
derive the provider-profile location from the selected Octet receipt rather
than from a fixed fixture path, and MUST admit exactly the two producer
tool-graph wire formats the producer job declares. The receipt-derived locator
MUST be the single `artifact_identities` row whose `blake3` equals
`consumer.provider_profile_identity` (with the `b3:` prefix ignored) and whose
locator's final component is `provider-profile.json`, MUST be a safe relative
path, and MUST be shared by the core check and the shell material plan. Tool
graphs MUST be admitted as either the legacy `nix derivation show` map or the
`version: 4` `derivations` map keyed by store basenames, with legacy nodes
carrying no `version`/`inputs` and versioned nodes carrying `version: 4`,
static output selections, and an empty dynamic-output map. Everything else MUST
fail closed with a stable error variant. The bridge MUST NOT gain proof
authority: it locates and parses bytes, never admits theorems or reimplements
the producer's verification policy.

#### Scenario: A rail-named provider profile is located from the receipt

GIVEN a receipt whose `artifact_identities` lists
`proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.aeneas/provider-profile.json`
with a `blake3` equal to `consumer.provider_profile_identity`
WHEN the bridge observes the Octet handoff
THEN it MUST select that row as the provider-profile material
AND the checked-material list MUST name that exact locator.

#### Scenario: A missing, duplicated, or mismatched provider row fails closed

GIVEN a receipt with no provider-profile row, with two provider-profile rows,
or with a provider-profile digest that differs from the consumer identity
WHEN the bridge observes the Octet handoff
THEN it MUST fail with `MissingProviderProfileLocator` or
`AmbiguousProviderProfileLocator` respectively
AND no observation MUST be produced.

#### Scenario: A versioned producer graph is admitted

GIVEN a graph `{"version": 4, "derivations": {...}}` whose nodes carry store
basenames, `version: 4`, `inputs.drvs` with string outputs and empty
`dynamicOutputs`, and `inputs.srcs` of store basenames
WHEN the tool graph is admitted
THEN every dependency MUST resolve inside the closure and the root MUST be
looked up by the final component of `root_derivation`
AND `tool_graph_identity` MUST remain the digest of the exact graph bytes.

#### Scenario: Mixed, unsupported, and incomplete graphs fail closed

GIVEN a versioned node carrying `inputDrvs`, a top-level `version: 5`, a v4
root absent from the closure, a v4 dependency outside the closure, a v4 dynamic
output selection, or a legacy node carrying `version`
WHEN the tool graph is admitted
THEN it MUST fail with `MixedToolGraphShape`,
`UnsupportedToolGraphVersion`, `ToolGraphRootOrCountBound`,
`IncompleteToolClosure`, or `NonEmptyDynamicToolOutputs` respectively
AND no observation MUST be produced.

### Requirement: The bridge binds a Trellis-owned Verus producer job

r[verification_bridge.trellis_job_mode] The verification bridge MUST offer a
`trellis-job` mode that observes a Trellis-owned Verus producer output and an
explicit producer export without executing the producer. The mode MUST read
only the profile, the receipt the profile declares, Terrane's producer report,
and the materials those documents name, inside the existing document, material,
aggregate-byte, count and no-symlink bounds. It MUST require the profile schema
`trellis-verus-producer-profile/v1`, the receipt schema
`octet-verus-proof-receipt/v1`, and the report schema
`trellis-verus-producer-report/v1`; MUST require the profile bytes to hash to
the packaging profile identity and `consumer.provider.profile_identity` to equal
the packaging provider identity; MUST require receipt `status` to be `passed`
and `checked_scope` to be present; MUST require every profile source path to be
an exact material; MUST require every receipt source binding to name a profile
source path by digest and path suffix and the selected source path to carry the
packaging source identity; MUST require the profile's artifact locator set to
equal the artifacts the receipt binds — its `stdout_artifact`, its
`stderr_artifact`, and the `<locator>#<digest>` in `proof_artifact_identity` —
with every such artifact an exact material carrying the receipt's digest and
recorded byte count and any profile-declared `blake3` or `bytes` in agreement;
and MUST require the profile's `verus.receipt_path` to name the selected
receipt bytes, with a declared `verus.receipt_identity` in agreement. The exact
producer report bytes MUST become `Observation.external_report_identity`, and
the association identity MUST bind it. `proof_acceptance` MUST remain
`not-evaluated-by-mantle`; the bridge MUST NOT run Verus, interpret the report,
proof cards or evidence, re-derive a receipt field, or admit the proved
theorem.

#### Scenario: A conforming Trellis producer job is observed

GIVEN a producer output whose profile, receipt, report, source, artifacts and
named documents satisfy the contract
AND an acquisition export whose job declares `trellis-mantle-producer-job/v1`,
owner `Trellis`, the observed profile and provider identities,
`execution.rail_id` equal to the receipt's `checked_scope`, denied network, the
packaging source revision, and a complete tool graph rooted at
`root_derivation`
WHEN the bridge observes with `trellis-job`
THEN it MUST produce one observation with bound `producer_job_identity` and
`tool_graph_identity`
AND `external_report_identity` MUST be the digest of the exact report bytes
AND `proof_acceptance` MUST be `not-evaluated-by-mantle`
AND exporting and replaying from the exported directory MUST reproduce the same
association identity without an ambient Terrane, Nix, or Verus lookup.

#### Scenario: A producer job with a wrong owner-scoped contract fails closed

GIVEN a job whose schema is other than `trellis-mantle-producer-job/v1`, whose
owner is not `Trellis`, whose rail differs from the receipt `checked_scope`,
whose execution allows the network, or whose flake revision differs from the
packaging source revision
WHEN the bridge binds the job
THEN it MUST fail with a stable error variant
AND no observation MUST be produced.

#### Scenario: Stale or tampered producer documents fail closed

GIVEN an unsupported profile, receipt, or report schema, a failed receipt
status, a mutated profile source identity, a mutated receipt source binding or
artifact row, tampered artifact bytes, a missing report or evidence document,
an absent tool-graph root, or an incomplete tool closure
WHEN the bridge observes or binds the producer job
THEN it MUST fail with a stable error variant
AND no observation MUST be produced.

#### Scenario: Existing producer modes are unchanged

GIVEN the recorded Octet and Trellis fixtures and the realized Octet producer
output
WHEN the bridge observes with `octet`, `octet-verified`, `trellis`, or
`octet-job`
THEN their checks, association identities, and `proof_acceptance` values MUST
remain as before.
