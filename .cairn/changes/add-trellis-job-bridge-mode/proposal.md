# Proposal: Admit a Trellis-owned Verus producer job in the verification bridge

## Why

The Cairn convergence pilot needs Mantle to observe a producer job that is not
Octet's: Terrane authors `trellis-mantle-producer-job/v1` with owner `Trellis`,
emits a `trellis-verus-producer-profile/v1` profile, Octet's
`octet-verus-proof-receipt/v1` Verus receipt, and its own
`trellis-verus-producer-report/v1` report.

The bridge has no mode for it. The existing `trellis` mode is the recorded
Aeneas cross-verifier pilot: it requires `trellis-aeneas-cross-verifier-profile/v1`
and `trellis-aeneas-proof-receipt/v1`, a `supported` outcome, Aeneas extraction
options, and the Aeneas artifact layout. Reusing it would mislabel Verus
evidence as an Aeneas theorem-over-generated-model result. The job binder is
also Octet-specific: it hard-codes `octet-mantle-producer-job/v1`, owner
`Octet`, and the receipt's `rail_id` field, so no Trellis job graph can bind.

## What Changes

- Add the fifth CLI mode
  `mantle-verification-bridge trellis-job OWNER_OUTPUT PRODUCER_EXPORT SOURCE_REVISION OUTPUT`
  (`OUTPUT '-'` inspects only).
- `consume_trellis_job` reads the profile, the receipt named by
  `verus.receipt_path`, the producer report, and every material the profile
  names, inside the existing byte, count, and no-symlink bounds.
- `observe_trellis_job` checks the three schema versions, the profile and
  provider identities, a `passed` receipt with a `checked_scope`, every source
  path as an exact material, every receipt source binding against a profile
  source path by digest and path suffix, the tool/options contract digests, the
  profile artifact locator set against the artifacts the receipt binds
  (`stdout_artifact`, `stderr_artifact`, and `proof_artifact_identity`'s
  `<locator>#<digest>`) with exact bytes and byte counts, and the profile's
  `verus.receipt_path` (and optional `verus.receipt_identity`) against the
  selected receipt bytes. The exact producer-report bytes become
  `external_report_identity`; `proof_acceptance` stays
  `not-evaluated-by-mantle`.
- `bind_trellis_job` shares the Octet job binder with the expected schema,
  owner, and receipt rail field as parameters. The Trellis job must name
  `trellis-mantle-producer-job/v1`, owner `Trellis`, the observed
  profile/provider identities, `execution.rail_id` equal to the receipt's
  `checked_scope`, denied network, the packaging source revision, and a
  complete valid tool graph rooted at `root_derivation`. `check_tool_graph` is
  unchanged.
- New stable error variants for the new schemas, owner, source bindings, and
  missing report; synthetic positive and negative tests; ADR 0168; the bridge
  document's `trellis-job` subsection.

Mantle gains no proof authority: it links exact bytes and identity, never runs
Verus or judges the bounds theorem, and does not interpret the producer report,
proof cards, or evidence.

## Impact

- **Immediate outcome**: a producer output following the design output contract
  observes with bound job/graph identities and
  `proof_acceptance: not-evaluated-by-mantle`.
- **Durable capability**: any owner-scoped producer job the bridge is taught a
  contract for can bind its graph through one binder; Octet behavior is
  unchanged.
- **Files**: `crates/crunch-verification-bridge/src/{main,lib}.rs`,
  `crates/crunch-verification-bridge-core/src/{lib,producer,error}.rs`,
  `crates/crunch-verification-bridge/tests/trellis_job.rs`,
  `adr/0168-…`, `docs/verification-build-bridge.md`.
- **Testing**: focused bridge tests, clippy, fmt, `./scripts/check-no-std-core.sh`
  (steps 1-11; step 12 already fails at `origin/main`), Cairn validate and the
  proposal/design/tasks gates.
- **Maintenance owner**: verification-bridge and producer-contract owners.

## Out of Scope

- Any theorem admission, proof registry, or proof-truth evaluation in Mantle.
- Running Verus, Nix, or any subprocess from the bridge.
- The real Terrane producer output and its realization (WP8 stage 2).
- Changing the existing `octet`, `octet-verified`, `trellis`, or `octet-job`
  behavior or their recorded fixtures.
