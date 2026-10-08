# Design: Add dynamic-plan source slices

## Goal and scope

Producers declare per-unit sources as slices of their own outputs. The worker
admits each slice as a content-addressed store object before it registers the
plan's units. The change adds one schema version and one admission step. It
does not change scheduling, sandboxing, or `mantle-plan-v1`.

## Current behavior

- `mantle-plan-v1` is decoded and validated in a pure core with named limits:
  4 MiB, 4,096 units, 256 inputs per unit, 16 outputs, 512 environment
  entries, and 16 KiB strings (`crates/crunch-build/src/dynamic_plan.rs:19-28`,
  `616-630`). Wire types deny unknown fields.
- A source is `{id, path, nar_blake3}` (`dynamic_plan.rs:339-344`). The worker
  only parses the path (`worker.rs:420-437`); prepare later requires PathInfo
  for it (`orchestrate.rs:931-951`).
- The worker reads a declared plan output as one bounded regular file from the
  producer's build result (`worker.rs:1523-1612`) and registers every unit
  before wanting the roots (`worker.rs:481-549`, `1638-1654`).
- The store already exposes verified source ingest
  (`SourceAdmission::ingest`, `crates/crunch-store/src/capability.rs:843-855`)
  and NAR hashing over castore nodes (`capability.rs:440-496`).
- The accepted `build-correctness` dynamic-plan compatibility requirements
  keep `mantle-plan-v1` canonical bytes and plan digests stable unless a
  versioned schema change approves a difference.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Sandbox store socket | Builder calls `AddToStore` (`builder-rpc-v0`, recursive-nix) | Rejected: protocol surface inside the sandbox; objects exist before admission | None |
| One output per source | Producer declares one output per source | Rejected: output names are fixed at evaluation and capped at 16 | None |
| Copy units | A content-addressed unit copies one subtree | Rejected for now: content-addressed unit outputs cannot back plan placeholders (`worker.rs:723-734`), and dependents key on the copy unit's unresolved derivation path | Revisit after `resolve-content-addressed-inputs-before-dispatch` |
| Whole-output source | Units read the producer's whole output | Current behavior; rejected as the granular answer: any byte moves every unit | None |
| Plan-declared slices | Worker admits declared subtrees of producer outputs | Selected | Two-run identity fixture, negative admission fixtures |

## Contract and component ownership

- **Pure core** (`dynamic_plan.rs`, `dynamic_plan/wire.rs`): `mantle-plan-v2`
  wire and admitted types, slice grammar, limits, canonical bytes, and plan
  digest. A slice shares the source-id namespace with store-path sources.
- **Pure admission planner**: over shell-supplied tree facts (node kind,
  digest, NAR BLAKE3, and size) for each requested subpath, it returns an
  ordered admission plan or one typed rejection. It performs no I/O.
- **Shell** (worker plus store capability): reads the producer's output nodes,
  walks each subpath without following symlinks, computes observed digests,
  runs verified-source admission and PathInfo signing, then hands the
  resulting store paths to unit registration.

## Decisions

### Decision: Version the schema instead of extending `mantle-plan-v1`

**Choice:** Add `mantle-plan-v2`, a superset of v1 with one new source form.
The decoder dispatches on `schema`; v1 keeps its DTOs, canonical bytes, and
digests.

**Rationale:** v1 wire types deny unknown fields, and the accepted
compatibility requirement forbids changing v1 bytes in place. A version
string makes producer intent explicit and keeps existing fixtures valid.

### Decision: Slices come only from the producer's declared outputs

**Choice:** A slice names one of the producing derivation's declared outputs
and a relative subpath inside it. Host paths, other derivations' outputs, and
the plan output itself are not slice roots.

**Rationale:** The producer's outputs are admitted build results with
recorded provenance. Other roots would let a plan launder content it did not
produce.

### Decision: Identity is content plus declared name

**Choice:** The slice's store path derives from its NAR content and its
declared store name under the configured logical prefix, using the store's
existing source-admission identity rules. Producer identity, output name, and
subpath do not enter the path.

**Rationale:** Unchanged package content must map to the same path across
producer reruns, or dependent units lose their cache identity.

### Decision: The expected digest is required

**Choice:** Every slice carries `nar_blake3`. The worker admits a slice only
when the observed digest equals it.

**Rationale:** The plan digest then binds slice content, so a changed slice
changes the plan identity, and reports show exactly what was admitted.

### Decision: Admission is atomic per plan and precedes registration

**Choice:** The worker plans every slice first and rejects the whole plan on
any failure. It then publishes slices in canonical source-id order and
registers units only after every slice is published.

**Rationale:** Registry mutation already requires full admission. Partial
publication would leave units that reference sources that were never
admitted.

## Failure behavior and ordering

Validation order is schema, limits, slice grammar, graph, admission plan, then
shell effects. Rejection kinds are stable: `slice-output-undeclared`,
`slice-subpath-invalid`, `slice-absent`, `slice-symlink-traversal`,
`slice-digest-mismatch`, `slice-limit`, and `slice-conflict`. Report rows are
sorted by producer, output, and source id. An identical slice declared twice
with the same content is an idempotent duplicate; one source id with different
content is a conflict.

## Tests

- Positive: a v2 plan with two slices schedules a unit that reads both through
  `{{mantle-source:ID}}`; two slices with identical content and name resolve
  to one store path; a producer rerun with an unchanged slice keeps the unit
  derivation path.
- Negative: absolute, `..`, empty-component, and over-long subpaths; an
  undeclared output; an absent subpath; a symlink on the walk path; a digest
  mismatch; over-limit count and bytes; a duplicate id with conflicting
  content; v2 fields inside a v1 document.
- Compatibility: accepted v1 golden fixtures keep canonical bytes and plan
  digests.

## Risks / Trade-offs

- NAR hashing cost grows with slice size. Named byte limits bound it, and
  producers choose slice granularity.
- Every distinct slice becomes a store object. Units reference slices as
  sources, so root-driven garbage collection reclaims unused ones.
- Declared names affect deduplication. Producers should derive names from
  stable facts such as package name and version.

## Claim boundary

Slice evidence proves only schema admission, subtree identity, and
publication of declared producer content. It does not prove source trust,
license compliance, producer correctness, build success, or release
eligibility.
