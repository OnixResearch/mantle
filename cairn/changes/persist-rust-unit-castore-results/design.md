# Design: Local castore-backed Rust unit results

## Context

Mantle already records detailed Rust unit execution receipts. It also has BLAKE3 castore services and separate action-result semantics.

Current Rust unit reuse reads artifacts from the prior execution directory. It does not restore deleted outputs from castore.

Object presence alone cannot prove which Rust action produced an object. ADR 0024 therefore keeps CAS objects, action-result discovery, and execution separate.

## Goals

- Reuse supported Rust unit outputs after the prior execution directory is removed.
- Preserve explicit action identity, admission, receipt, and non-claim boundaries.
- Share physical castore chunks with other Mantle content without sharing semantic authority.
- Keep identity and admission logic pure and deterministic.

## Decisions

### Decision 1: Add a reusable functional core and a thin storage shell

**Choice:** Add `crunch-rust-cache-core` for canonical schemas, BLAKE3 identities, bounds, candidate classification, and admission decisions.

Add `crunch-rust-cache` for castore access, local indexes, filesystem staging, atomic commit, and orchestration.

**Rationale:** `rust-plan`, a later remote adapter, and a later Cargo wrapper can share one tested identity contract. The pure core will not read files, inspect environment variables, open stores, or spawn processes.

### Decision 2: Use a versioned canonical action identity

**Choice:** Define a Rust-owned canonical action schema. Compute its action reference with BLAKE3 over canonical bytes.

The schema will bind these facts:

- unit, package, crate, host, target, profile, mode, and feature identities;
- source closure and unit source digests;
- compiler executable content, verbose version, sysroot or provider closure, and execution-platform identities;
- normalized semantic compiler arguments after dependency and host-artifact rebinding;
- the admitted environment map and compiler-policy identity;
- dependency, host, proc-macro, native-link, and build-script output digests;
- build-script metadata and generated `OUT_DIR` tree identity;
- the selected C and linker toolchain route when linking is required;
- schema and policy versions.

The action schema will exclude the physical execution output root and unclassified temporary paths. An unclassified absolute path will block publication and strong reuse.

**Rationale:** Compiler flags and normalized source paths alone do not identify a Rust compilation safely.

### Decision 3: Keep result records separate from output trees

**Choice:** Define a versioned Rust unit result record containing the action reference, castore root node, bounded artifact manifest, producer receipt linkage, and record identity.

The castore tree will contain declared compiler artifacts. It will not contain the mutable per-run execution receipt. Mantle will create the current receipt after restoration or compilation.

**Rationale:** Immutable compiler bytes and current execution evidence have different lifecycles.

### Decision 4: Preserve ordered local reuse routes

**Choice:** Use this lookup order:

1. Verify the existing execution-directory receipt and artifacts.
2. Query the local Rust unit result index by canonical action reference.
3. Invoke the compiler after an admitted miss.

A castore hit must pass schema, action, manifest, bounds, object-completeness, and artifact-digest admission before Mantle skips compiler execution.

Conflicting admissible results for one action will produce explicit nondeterminism evidence. Mantle will not select a result by insertion order.

**Rationale:** Existing output reuse is cheaper than materialization. Conflicts must remain visible.

### Decision 5: Restore through verified atomic materialization

**Choice:** Export the admitted output tree into a new sibling staging directory. Verify every declared path, type, mode, size bound, and BLAKE3 digest before commit.

Commit the complete directory with a no-partial replacement protocol. Remove staging data after every failed restore.

Do not mount read-only castore FUSE over a writable Cargo or Rust execution output root.

**Rationale:** The current Snix FUSE surface is read-only. A mutable compiler output directory needs explicit filesystem ownership and failure cleanup.

### Decision 6: Publish only after successful execution and ingestion

**Choice:** After a successful compiler invocation, digest declared artifacts, ingest the artifact tree, verify the resulting tree, and publish the immutable result record last.

A failed compiler invocation, partial output tree, failed ingestion, or failed admission will not publish an index candidate.

**Rationale:** Readers must never observe a result record before all referenced content exists.

### Decision 7: Give Rust unit records explicit retention authority

**Choice:** Local Rust unit indexes will participate in store retention. Garbage collection will either retain every referenced tree or remove the stale result reference before deleting unreachable objects.

All collection sizes, metadata sizes, tree depth, file count, total bytes, and candidate counts will have named limits.

**Rationale:** A result index that points to collected content creates false cache hits.

### Decision 8: Keep cache use explicit and evidence-bearing

**Choice:** Add an explicit local Rust unit cache mode for `rust-plan` execution. Receipts will identify output-directory reuse, local castore reuse, local miss, rejection, conflict, and compiler execution.

The evidence will preserve these non-claims:

- no compiler correctness claim;
- no full Cargo compatibility claim;
- no remote trust claim;
- no proof that equal inputs always produce equal outputs.

**Rationale:** Micro-cache reuse is a bounded execution optimization, not a stronger correctness result.

## Failure Semantics

- Stale artifacts under the active execution output root keep the existing stale-output blocker behavior.
- A malformed or incomplete local castore candidate is rejected with a stable reason.
- An eligible compile can continue after a rejected advisory candidate when policy permits execution.
- Conflicting admissible candidates produce a nondeterminism blocker for strong reuse.
- A failed restore leaves no published partial output directory.

## Risks / Trade-offs

- Strong toolchain closure identity can add hashing cost on cold runs.
- Atomic directory replacement needs explicit Linux filesystem semantics and cross-filesystem rejection.
- New result roots increase garbage-collection state and policy complexity.
- Shared physical chunks do not remove the need for separate Rust unit trust and retention records.
