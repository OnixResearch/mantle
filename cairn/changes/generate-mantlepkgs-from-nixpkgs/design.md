# Design: generated Mantlepkgs from concrete Nixpkgs graphs

## Context

Nixpkgs source uses Nix functions, overlays, recursive package sets, string contexts, dynamic imports, and evaluator-specific builtins. A general source translator cannot preserve those meanings reliably.

Nix can already evaluate a locked package selection into concrete derivation data. Mantle already admits, rewrites, plans, and realizes that data.

The missing layer is a package collection. It must bind selected package names to admitted Mantle roots and make unsupported packages visible.

## Decisions

### Decision: Convert evaluated graphs, not Nix source

**Choice:** Use Nix as a producer for locked package selections. Convert its concrete derivation graphs into Mantle package records.

The converter does not translate Nix functions, overlays, or package-set code into Nickel functions.

**Rationale:** Concrete graphs contain the build facts that Mantle consumes. Source translation would add an incomplete Nix evaluator to the package API.

### Decision: Keep producer and consumer separate

**Choice:** The producer shell may run a recorded Nix command before it publishes artifacts. Later catalog validation, planning, and building must not run Nix.

The producer receipt binds the Nix implementation, Nixpkgs lock identity, systems, selectors, command class, and emitted artifact identities.

**Rationale:** Mantle can replace Nix during consumption without pretending that it evaluated Nixpkgs itself.

### Decision: Use Nickel for the reviewed package-set contract

**Choice:** A typed Nickel manifest defines source identity, systems, selectors, aliases, conversion policy, source policy, and named limits.

The generated Nickel catalog contains package names, systems, artifact references, BLAKE3 digests, root identities, provenance, and blocker dispositions.

Large derivation graphs remain versioned machine artifacts in their native foreign-import format. The catalog refers to them with confined relative paths and exact digests.

**Rationale:** Nickel is suitable for reviewed configuration and package selection. It is not a useful text encoding for every low-level derivation field.

### Decision: Share graph nodes across selected packages

**Choice:** The pure converter keys each foreign node by its canonical foreign identity. It deduplicates shared nodes before target compilation.

Every selected package receives one explicit disposition. A package is either buildable or blocked with ordered reason codes.

**Rationale:** Repeating a complete graph for every package wastes space and can hide inconsistent translations of one shared dependency.

### Decision: Recompute Mantle identities and rebuild

**Choice:** The converter uses the existing recompute policy and the configured Mantle store prefix. Mantle computes target derivation and output identities with BLAKE3.

A clean rebuild proof disables substitution and requires the ordinary Mantle worker to report each selected root as built.

**Rationale:** Mantlepkgs is a Mantle package collection. Exact Nix cache paths are not a goal for this route.

### Decision: Keep source material explicit

**Choice:** Catalog generation emits a complete source requirement inventory. Existing Mantle source bundles remain the required baseline transport.

A later Nario v2 adapter may satisfy matching requirements. Nario support is optional and must not change recipe identity.

**Rationale:** Nario carries store payloads. It does not carry Nix package meaning or replace the concrete derivation graph.

### Decision: Fail closed for unsupported package behavior

**Choice:** The converter rejects or blocks packages with unsupported builtins, missing sources, stale locks, ambiguous aliases, graph errors, or unresolved foreign paths.

The converter also scans declared recipe fields for unsupported hard-coded source-store paths. It does not rewrite opaque source payload bytes by default.

**Rationale:** A generated catalog is useful only when unsupported entries remain visible and cannot appear buildable by accident.

### Decision: Use a functional core and an imperative shell

**Choice:** A pure core validates manifests, merges graphs, derives catalog records, orders diagnostics, and computes canonical BLAKE3 preimages.

The shell evaluates Nickel, runs the producer command, reads artifacts, stages output files, invokes Mantle planning and building, and writes receipts.

**Rationale:** Conversion decisions must be testable without Nix, the filesystem, the network, or a running store.

### Decision: Publish the catalog atomically

**Choice:** The shell writes a private catalog stage, validates all referenced artifacts, then performs one no-clobber publication step.

A failed selected package keeps the batch unsuccessful. The shell must not publish a partial success catalog that silently omits it.

**Rationale:** Readers must not observe mixed graphs, indexes, receipts, or Nickel catalog data.

## Risks / Trade-offs

- Some Nixpkgs builders assume `/nix/store` even after normal recipe fields are rewritten.
- Source payloads can contain hidden store paths that require package-specific repair.
- A first supported cohort does not establish full Nixpkgs coverage.
- Nix remains required on the producer side for updates.
- Generated package entries are reviewable data, not idiomatic handwritten Nickel recipes.
