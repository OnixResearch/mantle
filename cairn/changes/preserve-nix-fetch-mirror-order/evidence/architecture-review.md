# Architecture review

## Question

Where should Mantle convert Nix fetch mirror data into the ordered candidate model used by foreign realization?

## Inspected Evidence

- `src/foreign_derivation_import.rs` copies concrete Nix environment fields and emits empty source-payload mirror lists.
- `src/foreign_graph_compiler.rs` creates canonical ordered candidates only for explicit download and Git-download nodes.
- `crates/crunch-build/src/fetch_build_service.rs` consumes `url` and `__mantle_foreign_candidates`. It does not consume Nix `urls` or `__json`.
- Nix structured derivation fixtures store typed attributes in the `__json` environment field.
- Current Nixpkgs `fetchurl` writes its ordered `urls` list through structured attributes.
- ADR 0046 requires bounded sequential source fallback and terminal fixed-output mismatch behavior.
- `cairn/specs/foreign-derivation-import/spec.md` already requires deterministic mirror order and frontend-neutral consumption.

## Approach Registry

### Adapter normalization

- **Mechanism:** Parse supported Nix forms in a pure producer core and emit one canonical node-level candidate list.
- **Claim:** Nix details remain in the producer while order becomes graph, derivation, plan, and receipt data.
- **Artifact:** This change's design and delta requirement.
- **Evidence:** Existing adapter and compiler boundaries already separate frontend parsing from execution.
- **Gap strength:** simpler than runtime Nix parsing.
- **Known failure:** unresolved Nix mirror aliases require producer expansion or rejection.
- **State:** selected.

### Runtime Nix parsing

- **Mechanism:** Teach `FetchBuildService` to read `url`, `urls`, `__json`, and mirror tables.
- **Claim:** Existing graph output could remain unchanged.
- **Evidence:** The fetch service already has access to derivation environment data.
- **Gap strength:** stronger than needed.
- **Known failure:** this mixes frontend semantics with network execution and hides identity rules.
- **State:** rejected.

### Source-payload mirror reuse

- **Mechanism:** Attach Nix fetch candidates to an existing source payload.
- **Claim:** Reuse the current explicit-download mirror path without changing node data.
- **Evidence:** Explicit foreign download fixtures already use payload mirrors.
- **Gap strength:** unknown.
- **Known failure:** Nix fetch candidates describe a node output, not an arbitrary input source payload.
- **State:** rejected.

### First-address compatibility

- **Mechanism:** Continue using only `url` and ignore the remaining Nix candidates.
- **Claim:** Preserve current behavior without schema work.
- **Evidence:** This is the current Nix path.
- **Gap strength:** weaker than the accepted requirement.
- **Known failure:** it violates deterministic mirror preservation and can fail valid multi-address imports.
- **State:** falsified.

## Adversarial Audit

The selected route must reject these false-success cases:

- a foreign derivation injects Mantle's private candidate field;
- structured and unstructured candidate fields appear together;
- `url` disagrees with the first `urls` entry;
- candidate extraction silently sorts or removes entries;
- an unresolved `mirror://` value reaches the consumer;
- candidate presence converts an arbitrary fixed-output builder into a simple download;
- a content mismatch advances to another candidate;
- a candidate-order change leaves target recipe identity unchanged;
- ATerm and derivation-JSON inputs lower equal facts differently.

The delta spec and tasks include a negative test or explicit blocker for each case.

## Decision

Use a pure Nix producer normalization core and one canonical node-level ordered candidate list. The compiler derives private fetch-service fields from that list.

## Owner

The Mantle foreign-import and foreign-realization maintainers own implementation and review.

## Next Action

Complete task `V1` before code changes. Then implement the canonical model and pure Nix normalization tasks in dependency order.

## Non-Claims

This review does not prove implementation, arbitrary Nix fetcher compatibility, nixpkgs mirror expansion, package correctness, or build success.
