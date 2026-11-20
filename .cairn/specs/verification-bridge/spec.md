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
