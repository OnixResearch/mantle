# 0167. Admit Octet's current producer layout in the verification bridge

Status: Accepted

## Context

The verification bridge was written against the recorded fixture, which came
from Octet `4a540dff99aa2040e267e42bbafe09144f276572`. Octet's producer at
`d5b34c2d64b1227417a46df8ed8427f2d10ed3c8` writes a different layout, measured
on the realized convergence-pilot output:

- The provider profile lives under the rail-named proof directory
  (`proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.aeneas/provider-profile.json`),
  not at `proofs/aeneas/provider-profile.json`. Both receipts already locate it
  in `artifact_identities` as `{locator, blake3}`, with `blake3` equal to
  `consumer.provider_profile_identity` (`b3:cd11b49d…` in both).
- Nix 2.36 and the producer export (`--graph-format
  nix-derivation-show-v4`) emit
  `{"version": 4, "derivations": {"<hash>-<name>.drv": {"version": 4,
  "inputs": {"drvs": {"<basename>.drv": {"outputs": [...],
  "dynamicOutputs": {}}}, "srcs": ["<basename>", ...]}}}}`, with store
  basenames throughout, where the bridge accepted only the legacy
  `nix derivation show` map of full store paths, `inputDrvs`, and `inputSrcs`.

The producer job declares both graph formats in
`standards/proof-rails/aeneas-lean/mantle-job.ncl`, and Octet's
`tools/mantle-producer/export.py` validates each with the same rules. The two
defects were the only blockers on the realized output.

## Decision Drivers

- The bridge must consume the producer's current output rather than one
  fixture's directory names.
- The receipt is the authority for where its own material lives.
- Admission must follow the producer contract, not a Mantle-invented wire
  format.
- Fail-closed behavior with stable, distinguishable errors.
- Mantle gains no proof authority.

## Decision

- Replace the `PROVIDER_PROFILE_PATH` constant with
  `octet_provider_profile_locator(receipt: &Value) -> Result<&str, BridgeError>`
  in `crunch-verification-bridge-core`. It selects the single
  `artifact_identities` row whose `blake3` equals
  `consumer.provider_profile_identity` (ignoring the `b3:` prefix) and whose
  locator's final component is `provider-profile.json`, requires
  `is_relative_path`, and fails closed with `MissingProviderProfileLocator`
  (zero matches) or `AmbiguousProviderProfileLocator` (more than one). The
  scan is bounded by `MAX_MATERIALS`. `check_octet_provider_profile` and the
  checked-material list use it, and the shell's `octet_material_paths` calls
  the same core function instead of repeating the rule.
- `check_tool_graph` accepts exactly two shapes:
  - A top-level object with a `version` key must have `version == 4` and a
    `derivations` object. Nodes are keyed by store basenames ending in `.drv`;
    each node carries integer `version == 4` and no `inputDrvs`/`inputSrcs`.
    `inputs.drvs` keys are basenames inside the closure, and each value carries
    `outputs` as an array of strings and an empty `dynamicOutputs` object.
    `inputs.srcs` is an array of store basenames. The root is looked up by the
    final path component of `root_derivation`.
  - Otherwise the legacy rules apply unchanged, and any node with `version` or
    `inputs` is rejected as mixed.
  - Version other than 4 yields `UnsupportedToolGraphVersion`; legacy keys in a
    versioned node or versioned keys in a legacy node yield
    `MixedToolGraphShape`; a non-empty dynamic-output map yields
    `NonEmptyDynamicToolOutputs`. The existing shape, root/count, closure, and
    source variants keep their meanings. `tool_graph_identity` remains the
    digest of the exact graph bytes and `MAX_TOOL_DERIVATIONS` is unchanged.
- The bridge locates bytes and parses derivation graphs only. It does not admit
  theorems, evaluate proof truth, or reimplement the producer's verification
  policy.

## Consequences

- `octet-job` on the realized pilot output exits 0 with
  `proof_acceptance: externally-reported-by-octet`.
- The provider-profile location follows the receipt, so a producer that renames
  its rail directory still binds, while a missing, duplicated, or
  digest-mismatched row fails closed.
- Both graph formats the producer job declares are admitted; a third shape, a
  mixed node, or an outside-closure dependency fails closed.
- The recorded fixture and its replay tests are unchanged in behavior.
- Mantle's authority is unchanged: the observation records externally reported
  acceptance and byte links, not proof admission.
