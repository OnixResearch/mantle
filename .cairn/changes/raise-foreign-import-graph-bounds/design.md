# Design: Raise foreign-import graph bounds to the producer contract

## Context

`src/foreign_derivation_import.rs` owns admission for foreign derivation graphs
and `src/foreign_import_cmd.rs` owns the CLI read/effect budget.
`src/foreign_graph_compiler.rs` compiles an admitted graph into native units and
exact path maps.

Current bounds and their origins:

| Bound | Old | Owner |
| --- | --- | --- |
| `MAX_GRAPH_NODES` | 2048 | admission (nodes, closures, exports) |
| `MAX_SOURCE_PAYLOADS` | 512 | admission (source payloads, refs) |
| `MAX_HASH_DOMAIN_RECORDS` | 4096 | admission (hash-domain records) |
| `MAX_FOREIGN_READ_CALLS` | 4096 | CLI effect plan (`prepare-sources`) |
| `MAX_ATERM_BUNDLE_DERIVATIONS` | `MAX_GRAPH_NODES` | ATerm bundle file count |

Producers were tightened first: the verification bridge accepts producer tool
graphs of `MAX_TOOL_DERIVATIONS = 32_768` derivations
(`crates/crunch-verification-bridge-core/src/producer.rs`), and Octet's exporter
admits 32,768 derivations and 16,384 source objects.

## Decisions

### Decision: Take the derivation-count authority from the bridge contract

**Choice:** `MAX_GRAPH_NODES = 32_768`, documented as equal to
`crunch_verification_bridge_core::producer::MAX_TOOL_DERIVATIONS`, with the
bridge core crate added as a `mantle` dev-dependency and a unit test asserting
equality. `MAX_ATERM_BUNDLE_DERIVATIONS` remains `MAX_GRAPH_NODES`.

**Rationale:** One number governs how large a producer graph may be. A dev-only
dependency keeps the binary free of a verification crate while making the tie
fail a test if either side moves.

**Rejected alternative:** A comment-only tie, which drifts silently.

### Decision: Derive the hash-domain record bound from the node bound

**Choice:** `MAX_HASH_DOMAIN_RECORDS = MAX_GRAPH_NODES.saturating_mul(HASH_DOMAIN_RECORDS_PER_NODE)`
with `HASH_DOMAIN_RECORDS_PER_NODE = 1`.

**Rationale:** `nix_hash_domain_records` emits exactly one `nix-compatible`
derivation-store-path record per closure key. A maximum-size graph must not trip
the hash-domain gate for correctly produced records. Saturating multiplication
keeps the constant expression total.

### Decision: Size the `prepare-sources` read budget from the payload bound

**Choice:** `MAX_FOREIGN_READ_CALLS = MAX_SOURCE_PAYLOADS as u32 + PREPARE_SOURCES_FIXED_READ_CALLS`,
where the fixed part is the plan JSON read plus the bounded Nario archives.
A const assertion keeps the budget at or above the payload bound.

**Rationale:** `ForeignHostPort::plan_source_bundle` admits one read per source
binding, so the admitted budget must cover a maximum-size source set plus the
fixed reads. The existing `read_limit > MAX_FOREIGN_READ_CALLS` guard then never
rejects an admissible command.

### Decision: Keep per-node surface limits and byte budgets

**Choice:** Leave `MAX_GRAPH_EDGES` (256 input edges per node),
`MAX_ATERM_COLLECTION_ITEMS` (256 collection items per derivation),
`MAX_PACKAGE_INDEX_ENTRIES` (128), `MAX_PLAN_DIAGNOSTICS` (2048), mirrors (16),
cache hints (16), sandbox capabilities (16), `MAX_FIELD_BYTES` (32 KiB),
`MAX_ATERM_DERIVATION_BYTES` (16 MiB), `MAX_ATERM_BUNDLE_BYTES` (256 MiB), and
`DEFAULT_FOREIGN_ARTIFACT_BYTES_MAX` (256 MiB) unchanged.

**Rationale:** These bound one node's surface or the byte size of one artifact;
none is a function of graph node count. A node with more than 256 input
derivations, or a closure whose serialized bytes exceed 256 MiB, is rejected as
an explicit resource decision rather than silently admitted.

### Decision: Keep one incremental exact-path index during compilation

**Choice:** `compile_foreign_graph_with_profile_and_output_mode` builds an
`ExactPathIndex` once from the source path maps, extends it as each node's
derivation/output paths are inserted, and passes it to node compilation and
final unit validation. `contains_unmapped_source_store_object` reads the
index's target set instead of rebuilding it from the map on every call.

**Rationale:** The previous code rebuilt the combined foreign→target map per
node and rebuilt the permitted-target set per validated field, which is
quadratic in node count and would make a 32,768-node plan impractical.
Incremental `BTreeMap`/`BTreeSet` growth makes the pass `O(n log n)`.

## Risks / Trade-offs

- A 32,768-node plan is materially larger (memory and plan JSON size). The
  artifact byte budget still bounds the output; the admitted graph is the
  operator's declared producer closure.
- The bridge dev-dependency couples the test build to a workspace crate; it adds
  no binary dependency and is already a workspace member.
- Cross-class path collisions are now detected at insertion rather than by a
  full rebuild; the diagnostic class and message are unchanged, and the final
  index rebuild after fixed-output seed insertion retains the whole-map check.

## Verification

- Positive: 4,096-derivation export lowers, validates, and plans; 32,768
  derivations lower, validate, and plan; 16,384 source payloads lower and
  validate.
- Negative: 32,769 derivations normalize to
  `nix-derivation-count-out-of-range`; 16,385 source payloads fail lowering and
  graph validation with `source-payload-limit-exceeded`.
- Contract tie: a unit test asserts `MAX_GRAPH_NODES` equals the bridge
  `MAX_TOOL_DERIVATIONS`.
