# Design: Plan complete cache metadata before root admission

## Context

Mantle can import one explicit HTTP cache path. The current path verifies the narinfo signature, store path, NAR hash, and store prefix before persistence.

That path does not recurse through narinfo references. It can therefore import a root while required runtime members remain absent.

## Goals

- Discover the complete signed reference graph before NAR download.
- Keep graph planning pure and deterministic.
- Bound all untrusted graph and metadata inputs.
- Preserve Nix store identities while using BLAKE3 for Mantle plan identity.
- Reuse existing single-member content admission.
- Admit the selected root only after every dependency succeeds.

## Decisions

### Decision 1: Keep substitution identity separate from native rebuild identity

**Choice:** Closure pull uses the exact requested Nix-compatible store paths from narinfo metadata. It does not translate them into Mantle BLAKE3 derivation paths.

**Rationale:** A Nix binary cache is keyed by Nix store identity. A translated Mantle path cannot address the same cache object.

### Decision 2: Use a pure observation-driven planner

**Choice:** A pure planner owns the pending set, observed members, depth facts, aggregate NAR size, deterministic member order, and BLAKE3 plan input. The shell supplies one verified owned narinfo observation at a time.

The planner performs no file, network, store, clock, environment, or process access.

**Rationale:** Graph limits, cycle handling, duplicate handling, order, and identity can be tested without HTTP or store services.

### Decision 3: Discover all metadata before NAR download

**Choice:** The HTTP shell fetches bounded narinfo text, parses it with the selected store prefix, checks the exact requested path, and verifies the configured signature policy. It adds the owned facts to the planner.

No NAR request starts until the planner has a complete graph.

**Rationale:** A missing, malformed, untrusted, conflicting, or oversized member must fail before content mutation.

### Decision 4: Bind authority and limits into the plan

**Choice:** The plan identity includes the schema, normalized cache authority, trust-policy BLAKE3, logical store prefix, root, fixed limits, and canonical member facts. Member facts include path, references, NAR SHA-256, NAR size, and raw narinfo BLAKE3.

Public-key material participates in the trust-policy digest. Raw secrets do not enter plans or reports.

**Rationale:** The same graph under different cache authority, keys, store prefix, or limits is a different admission plan.

### Decision 5: Enforce explicit bounded failures

**Choice:** The planner returns typed errors for member count, depth, aggregate NAR size, duplicate references, conflicting digest-to-path identities, unexpected observations, and incomplete finalization. It does not use assertions for untrusted limit enforcement.

The shell also bounds each narinfo response before UTF-8 parsing.

**Rationale:** Remote metadata is untrusted input. Panics and silent truncation are not valid policy outcomes.

### Decision 6: Reuse complete local members only

**Choice:** A local PathInfo counts as present only when its exact store path matches and its full castore node is complete. An incomplete local member is fetched again.

**Rationale:** PathInfo presence alone does not prove usable content.

### Decision 7: Import dependencies before one root

**Choice:** The first CLI slice accepts exactly one HTTP root. It imports all non-root members in canonical order, then imports the root last.

A dependency failure can leave verified dependency content as resumable cache state. It cannot persist or export the selected root. Root failure emits no success result.

**Rationale:** One root permits a clear root-last mutation guarantee without a second staging store.

### Decision 8: Keep explicit pulls non-recursive

**Choice:** Existing HTTP pull behavior remains unchanged unless `--closure` is present. Directory pulls do not accept `--closure` in this change.

**Rationale:** Recursion changes network and mutation scope. Operators must request it explicitly.

### Decision 9: Keep foreign receipt binding for a later change

**Choice:** This change provides the generic strict closure mechanism and report facts. It does not claim that a foreign import receipt selected the root.

**Rationale:** Cache transport and producer/evaluator integration are separate trust domains.

## Failure Semantics

- Missing or forbidden narinfo fails discovery before NAR download.
- Invalid signatures fail discovery before NAR download.
- A returned store path mismatch fails discovery before mutation.
- A duplicate or conflicting reference fails deterministically.
- A member, depth, metadata, or NAR-size limit fails closed.
- A dependency content failure leaves the root unpersisted and unexported.
- An incomplete local member is fetched again.
- A plan/member metadata mismatch during admission fails before that member persists.

## Validation Strategy

- Run focused `crunch-store` closure and pull tests before implementation.
- Test pure linear, diamond, cycle, deterministic-order, and limit behavior.
- Test signed HTTP root and dependency import with dependency-first requests.
- Test missing, malformed, untrusted, conflicting, oversized, and incomplete-local cases.
- Test CLI rejection for zero roots, multiple roots, directory closure mode, and `--all --closure`.
- Run focused package tests, formatting, Clippy, Cairn gates, and Tracey coverage.
- Run a bounded `cache.nixos.org` proof for a recorded root when network access is available.

## Non-Claims

- The plan does not prove package correctness.
- Successful import does not prove local rebuild compatibility.
- Signature trust does not prove publisher correctness.
- This change does not prove evaluator parity or nixpkgs evaluation support.
- This change does not provide private-cache authentication.
