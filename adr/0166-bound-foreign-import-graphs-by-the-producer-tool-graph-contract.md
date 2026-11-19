# 0166. Bound foreign-import graphs by the producer tool-graph contract

Status: Accepted

## Context

Foreign-import admission carried limits chosen for a hello-sized fixture:
`MAX_GRAPH_NODES = 2048` nodes and `MAX_SOURCE_PAYLOADS = 512` source
payloads. Those limits predate the producer contract that now governs real
closures. The verification bridge admits producer tool graphs of
`MAX_TOOL_DERIVATIONS = 32_768` derivations
(`crates/crunch-verification-bridge-core/src/producer.rs`), and the external
producer exporter admits 32,768 derivations and 16,384 source objects.

A real nixpkgs closure does not fit the old limits. `produce-nix` over a
7,157-derivation closure already failed with
`nix-derivation-count-out-of-range`, so no real producer graph could be
planned, and the Octet producer job graph had no admitting consumer.

Two implementation facts matter beyond the constants:

- `nix_hash_domain_records` emits one `nix-compatible` derivation-store-path
  record per closure node, so the hash-domain record bound is a function of the
  node bound, not an independent budget.
- `prepare-sources` admits one read call per source binding, so its admitted
  read budget must cover the source-payload bound plus its fixed reads.

## Decision Drivers

- One derivation-count authority for producer graphs: the bridge contract.
- A maximum-size admitted graph must plan within practical time.
- Explicit, named limits with fail-fast diagnostics (Tiger Style), not silent
  truncation or per-call growth.
- No new runtime dependency of the `mantle` binary on a verification crate just
  to reuse an integer constant.

## Decision

- `MAX_GRAPH_NODES` becomes 32,768, equal to
  `crunch_verification_bridge_core::producer::MAX_TOOL_DERIVATIONS`. The tie is
  mechanical: `mantle` takes the bridge core crate as a dev-dependency and a
  unit test asserts the two constants are equal. `MAX_ATERM_BUNDLE_DERIVATIONS`
  stays derived as `MAX_GRAPH_NODES`.
- `MAX_SOURCE_PAYLOADS` becomes 16,384, matching the producer export contract.
  Lowering fails fast with `source-payload-limit-exceeded` instead of relying on
  a debug assertion.
- `MAX_HASH_DOMAIN_RECORDS` is derived from the node bound
  (`MAX_GRAPH_NODES * HASH_DOMAIN_RECORDS_PER_NODE`, saturating), because one
  Nix-identity record is produced per node.
- `MAX_FOREIGN_READ_CALLS` is derived from the source-payload bound plus the
  fixed `prepare-sources` reads (plan JSON plus the bounded Nario archives).
- Per-node limits that do not scale with node count (input edges per node,
  package-index entries, diagnostics, mirrors, cache hints, sandbox
  capabilities) are unchanged; they bound a node's surface, not the graph.
- Graph compilation keeps one incrementally grown exact-path index instead of
  rebuilding the combined foreign→target map per node, so a maximum-size graph
  costs `O(n log n)` rather than quadratic time. The admitted-target set that
  validates rewritten fields is carried by the same index.

## Consequences

- A full nixpkgs closure fits: `produce-nix` lowering, graph validation, and
  executable planning admit 32,768 derivations and 16,384 source objects.
- One node past either bound is still rejected with the existing deterministic
  diagnostics, and the graph-node bound remains below the 65,536-node
  inspection-contract ceiling.
- Byte-level limits (ATerm derivation 16 MiB, bundle 256 MiB, JSON artifact
  256 MiB) stay independent resource budgets; they scale with per-derivation
  size, not node count.
- The bridge-contract tie is test-visible, so raising one constant without the
  other fails a unit test rather than drifting silently.
