# Proposal: Raise foreign-import graph bounds to the producer contract

## Why

Foreign-import admission rejected every real producer closure. `MAX_GRAPH_NODES`
was 2,048 and `MAX_SOURCE_PAYLOADS` 512, while the verification bridge admits
producer tool graphs of `MAX_TOOL_DERIVATIONS = 32_768` derivations
(`crates/crunch-verification-bridge-core/src/producer.rs`) and the producer
exporter admits 32,768 derivations and 16,384 source objects. `produce-nix` on a
7,157-derivation nixpkgs closure returned `nix-derivation-count-out-of-range`,
so the Octet producer job graph could not be planned or realized.

Two secondary limits were mis-derived from the old node bound:
`MAX_HASH_DOMAIN_RECORDS` (one Nix-identity record is produced per node) and the
`prepare-sources` read budget (one admitted read per source binding).

## What Changes

- `MAX_GRAPH_NODES` becomes 32,768, equal to the bridge
  `MAX_TOOL_DERIVATIONS` and asserted equal by a unit test.
  `MAX_ATERM_BUNDLE_DERIVATIONS` stays derived from it.
- `MAX_SOURCE_PAYLOADS` becomes 16,384, with explicit fail-fast lowering
  instead of a debug assertion.
- `MAX_HASH_DOMAIN_RECORDS` is derived from the node bound with saturating
  arithmetic.
- `MAX_FOREIGN_READ_CALLS` is derived from the source-payload bound plus the
  fixed `prepare-sources` reads.
- Graph compilation grows one exact-path index incrementally instead of
  rebuilding the combined foreign→target map (and its admitted-target set) per
  node, removing quadratic work from the 32,768-node path.
- ADR 0166 records the bound decision.

## Impact

- **Immediate outcome**: a 7,157-derivation nixpkgs closure, and any closure up
  to 32,768 derivations with up to 16,384 source objects, lowers, validates,
  and plans.
- **Durable capability**: foreign-import limits have one authority — the
  producer tool-graph contract — and the tie is test-visible.
- **Maintenance owner**: foreign-import / producer contract owners.
- **Evidence**: focused `mantle` unit tests for the positive and negative
  bounds, plus the build/clippy/fmt/no-std gate transcript.

## Out of Scope

- The producer job-graph realization itself (needs a realized producer output).
- Byte-level resource budgets (ATerm 16 MiB per derivation, 256 MiB bundle,
  256 MiB JSON artifact); they scale with per-derivation size, not node count.
- Per-node surface limits (input edges per node, mirrors, cache hints, sandbox
  capabilities, package-index entries, plan diagnostics).
