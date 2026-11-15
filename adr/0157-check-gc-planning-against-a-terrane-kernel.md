# 0157. Check GC planning against a Terrane kernel

Status: Proposed

Amended by ADR 0158: the kernel, the projection, and the parity rail now live
in `crunch-gc-check`, which runs the kernel at runtime.

## Context

`crunch_gc_core::plan_gc` decides which store paths are retained and which
overlay paths become deletion candidates. A wrong decision deletes a live
path. Its rules are stated in prose and covered by example tests: roots and
references must name known entries, everything reachable from a root is
retained, base layers are never deleted, and a base path must not reference a
reclaimable overlay path.

Terrane owns reusable Verus-checked logic, and ADR 0060 established how Mantle
consumes it: an unchanged source import pinned by revision and BLAKE3, a
test-only parity rail, and no runtime dependency on the verifier toolchain.

## Decision Drivers

- State the safety rules of GC planning as a checked model with named proofs.
- Keep the store shell, plan identity, and removal execution owned by Mantle.
- Keep `vstd` and the verifier out of the production dependency graph.

## Decision

Terrane provides a product-neutral `reclaim_plan` kernel over indexed nodes,
references, roots, and a reclaimable flag per node. `check_plan` is proved
equal to its specification: roots and references are in range, retained is
closed under references and contains every root, and the candidates are
exactly the reclaimable nodes that are not retained. Named proofs show that a
checked plan never makes a live node a candidate, never leaves a retained node
referencing a candidate, never makes a protected node a candidate, and that
`plan` reclaims every unreachable reclaimable node. `plan` is proved to return
a checked plan on every admissible input and to reject every other input.

`scripts/import-terrane-reclaim-plan.py` imports the kernel and its tests
unchanged into `vendor/terrane-reclaim-plan/`, checks exact BLAKE3 identities,
and records the producer's and Mantle's Verus standard library pins
separately. `crunch-gc-core` compiles the kernel only under `cfg(test)` with
its own `vstd` dev-dependency.

A parity rail projects each `GcPlanRequest` into kernel indices (entries
sorted by path, references and roots deduplicated, overlay ownership as
reclaimable, unknown paths out of range) and requires, for every enumerated
store:

- Mantle plans exactly when the kernel plans;
- retained paths, candidate paths, deletion intents, and protected base paths
  equal the kernel's result;
- the roots retaining each path equal the roots whose single-root kernel plan
  retains it;
- the kernel's `check_plan` accepts Mantle's own output;
- each Mantle rejection names a violation the kernel also finds, and names
  the only one when only one is present. When several violations coexist,
  which one is reported first is not compared.

The enumeration covers every store of up to three entries, every reference
set, every root set, every assignment of overlay or base layer 0, and one
missing-root and one dangling-reference variant of each: 99,098 cases. There are no named
divergences.

Runtime authority stays with `plan_gc` in this change.

## Consequences

- A change to `plan_gc` that retains too little, deletes a live or base path,
  or accepts an input the kernel rejects fails the parity rail. Seven
  mutations of `plan_gc` were tried; six fail the rail, and the seventh
  (removing the up-front missing-root check) is caught by the reachability
  walk and changes only which error is reported first.
- The import pins a Terrane revision; the pin must be reachable on the
  published Terrane history before this change can be accepted.
- Calling the kernel's `check_plan` on every runtime plan before deletion is
  possible future work and would be a separate decision.

## Non-Claims

Passing parity is executable agreement on enumerated cases, not a proof about
Mantle source. The kernel does not verify that roots or references are
complete, byte totals, plan identity, size limits, duplicate path identifiers,
the store shell's replan before deletion (ADR 0064), or removal execution.
Layer indices other than 0 and the order in which several coexisting violations
are reported are not compared.
Compiling the kernel under Mantle's `vstd` is not Verus verification.
