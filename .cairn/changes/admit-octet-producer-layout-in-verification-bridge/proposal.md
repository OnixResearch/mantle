# Proposal: Admit Octet's current producer layout in the verification bridge

## Why

The bridge could not observe a real producer output. Two rules were written
against the recorded fixture (Octet `4a540dff`) instead of the producer
contract:

- The provider profile path was hard-coded to
  `proofs/aeneas/provider-profile.json` in
  `crates/crunch-verification-bridge-core/src/lib.rs` and repeated in
  `crates/crunch-verification-bridge/src/lib.rs`. Octet's current producer
  (`d5b34c2d`) writes the profile under its rail-named proof directory
  (`proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.aeneas/`),
  and both receipts already locate it in `artifact_identities` with a `blake3`
  equal to `consumer.provider_profile_identity`.
- `check_tool_graph` accepted only the legacy `nix derivation show` map.
  Nix 2.36 and Octet's `--graph-format nix-derivation-show-v4` export emit a
  `{"version": 4, "derivations": {...}}` document with store basenames
  throughout, which Octet's exporter validates and its job declares.

The Cairn convergence pilot stopped on exactly these two blockers.

## What Changes

- The core gains a pure, bounded `octet_provider_profile_locator(receipt)` that
  selects the single `artifact_identities` row whose digest equals the
  consumer's provider-profile identity and whose locator names
  `provider-profile.json`, requires a relative path, and fails closed on zero
  or multiple matches. `check_octet_provider_profile` and the `checked` list
  use it; the shell's `octet_material_paths` calls the same function instead of
  duplicating the rule.
- `check_tool_graph` accepts exactly two shapes: the versioned (`version: 4`)
  map keyed by store basenames, and the legacy map keyed by full store paths.
  A graph with a top-level `version`, a versioned node, and a legacy node are
  each validated under their own rules; mixed or unsupported shapes fail closed
  with stable error variants. The derivation-count bound is unchanged, and
  `tool_graph_identity` remains the digest of the exact graph bytes.
- Positive and negative tests cover both defects, including a real-layout copy
  of the recorded fixture built in a temporary directory.
- ADR 0167 records the decision; `docs/verification-build-bridge.md` states the
  receipt-derived locator and the two accepted graph formats.

Mantle gains no proof authority: the bridge only locates bytes and parses Nix
derivation graphs. It does not admit theorems or reimplement the producer's
verification policy.

## Impact

- **Immediate outcome**: `octet-job` on the realized pilot output exits 0 with
  `proof_acceptance: externally-reported-by-octet`.
- **Durable capability**: the provider-profile location is derived from the
  receipt, and both graph formats the producer declares are admitted.
- **Maintenance owner**: verification-bridge / producer-contract owners.
- **Evidence**: focused tests, clippy, fmt, the no-std core gate, the Cairn
  gates, and the real producer-output run.

## Out of Scope

- Any theorem admission, proof registry, or proof-truth evaluation in Mantle.
- Writes under the pilot realization directories or the pilot store.
- Reformatting or normalizing producer metadata; `export` preserves exact bytes.
