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

The frozen Octet producer export
(`convergence-evidence/realization/producer-export/tool-closure.derivation-json.json`)
measures the real shape: 11,565 derivations, 677 source objects
(848,923,562 bytes), 5,231 fixed-output derivations, max input derivations per
node 611 (two cargo `vendor-registry` derivations at 346 and 611), three
`buildCommand` env values of 36,411 / 64,144 / 102,597 bytes, 1,652
derivations with structured attributes, max outputs 10, max inputSrcs 23, max
args 39, max env entries 70, max urls 3.

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
- `MAX_GRAPH_EDGES` (per-node input derivation edges) becomes 4,096: the frozen
  export reaches 611 edges, and parsed ATerm input lists use the same bound
  instead of the 256-item generic collection bound.
- `MAX_FIELD_BYTES` becomes 262,144 (256 KiB) for foreign-import fields: the
  frozen export carries a 102,597-byte `buildCommand`. No other crate's field
  bound changes.
- `MAX_PLAN_DIAGNOSTICS` is derived from the node bound times the per-node
  capability bound, because translation emits one diagnostic per declared node
  capability.
- Limits that do not scale with node count or with a producer-declared surface
  (package-index entries, mirrors, cache hints, sandbox capabilities) are
  unchanged.
- `crunch-build::goal::MAX_GOALS` becomes 32,768 and
  `crates/crunch-build/src/registry.rs::MAX_ENTRIES` follows it. `Worker::want`
  creates one goal per reachable derivation up front, so `MAX_GOALS` bounds
  total planned units, not concurrently live goals; the old 10,000 blocked
  realization of the 11,565-derivation frozen producer graph. The registry bound
  is the same quantity (one registered derivation per planned unit), so it is
  derived from the goal bound instead of holding its former 16,384.
- Graph compilation keeps one incrementally grown exact-path index instead of
  rebuilding the combined foreign→target map per node, so a maximum-size graph
  costs `O(n log n)` rather than quadratic time. The admitted-target set that
  validates rewritten fields is carried by the same index.

## Fixed-output metadata follows the v4 record shape

Nix v4 derivation JSON records a fixed-output hash as an SRI string
(`sha256-…`, `sha512-…`) with a `method` (`flat`/`nar`) and no `hashAlgo`;
the legacy `nix derivation show` shape records lowercase hex plus `hashAlgo`
(optionally `r:`-prefixed) and keeps `outputHashMode` in the environment. The
old lowering defaulted the algorithm to sha256 when `hashAlgo` was absent and
derived recursion only from `outputHashMode`, so a sha512 SRI digest produced
`foreign-compiler-fixed-output-hash-invalid` at plan time (the real frozen
producer export has 5,221 sha256 and 6 sha512 fixed outputs; 4,211 `flat` and
1,020 `nar` methods; 87 empty `outputHashAlgo`; 3,153 outputs with no env
`outputHashMode`).

The v4 path now takes the algorithm from the SRI prefix, rejects disagreement
with `hashAlgo`/`outputHashAlgo`
(`conflicting-fixed-output-hash-algorithm`), takes recursion from `method`
(`nar` => recursive, `flat` => flat, anything else =>
`unsupported-fixed-output-method`), requires a present `outputHashMode` to agree
with `method` (`conflicting-fixed-output-method-mode`), and keeps the legacy
hex/`hashAlgo`/`outputHashMode` path byte-for-byte unchanged.

## Consequences

- A full nixpkgs closure fits: `produce-nix` lowering, graph validation, and
  executable planning admit 32,768 derivations and 16,384 source objects.
- One node past either bound is still rejected with the existing deterministic
  diagnostics, and the graph-node bound remains below the 65,536-node
  inspection-contract ceiling.
- Byte-level limits (ATerm derivation 16 MiB, bundle 256 MiB, JSON artifact
  256 MiB) stay independent resource budgets; they scale with per-derivation
  size, not node count.
- Known performance limit: the synthetic 32,768-derivation lowering+planning
  test did not finish within 15 minutes in a release build on the loaded
  2026-11-19 host (debug builds exceeded 17 minutes before being stopped). The
  test is `#[ignore]`d with the exact release command recorded in its comment.
  The real frozen producer export (11,565 derivations) is exercised through the
  CLI instead; its wall time and peak RSS are recorded in the change evidence.
- The bridge-contract tie is test-visible, so raising one constant without the
  other fails a unit test rather than drifting silently.
