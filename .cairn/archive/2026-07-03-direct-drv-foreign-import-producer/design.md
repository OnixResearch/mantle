## Decisions

### 1. Treat `.drv` files as producer inputs, not store discovery roots

**Choice:** The direct producer accepts explicit logical `.drv` path plus filesystem `.drv` file pairs. The CLI shell reads those files and the core receives an in-memory map from logical derivation path to parsed derivation facts.

**Rationale:** A `.drv` file is concrete graph data, but discovering its recursive closure from `/nix/store` would reintroduce host-store coupling. Explicit mappings make the producer deterministic and easy to test with `PATH` stripped of Nix tools.

### 2. Reuse the existing Nix lowering core

**Choice:** Convert parsed `nix_compat::derivation::Derivation` values into the same `NixDerivationJsonClosure` structure used by derivation-JSON export, then call the existing lowering path.

**Rationale:** The existing lowering path already owns canonical node IDs, source refs, fixed-output metadata, cache hints, package index, and hash-domain records. A separate `.drv` lowering path would risk duplicate business logic.

### 3. Keep Nix-compatible identity conservative

**Choice:** The direct `.drv` path records the operator-supplied logical derivation paths as Nix-compatible identity records and parses ATerm only for graph facts. It does not recompute upstream Nix derivation store paths with Mantle's locally modified derivation hashing.

**Rationale:** Mantle's vendored `nix-compat` has hash-domain modifications. Avoiding recomputation keeps this slice useful without pretending the local crate proves upstream cache identity for arbitrary derivations.

## Risks / Trade-offs

- Operators must supply all input derivations explicitly until a separate crate-only closure discovery mechanism exists.
- Parsed derivations with unsupported dynamic derivation/output forms may need deterministic rejection rather than approximation.
- Full nixpkgs source evaluation remains future `snix-eval` producer work, not part of this change.
