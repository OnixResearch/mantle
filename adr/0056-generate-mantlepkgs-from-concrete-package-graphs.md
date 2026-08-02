# ADR 0056: Generate Mantlepkgs from concrete package graphs

## Status

Proposed

## Context

Mantle can consume concrete derivation graphs from Nixpkgs. It does not provide a reusable package collection generated from a locked Nixpkgs selection.

Direct Nix-to-Nickel source translation cannot preserve general Nix evaluation. Nixpkgs uses functions, overlays, recursive sets, string contexts, dynamic imports, and evaluator builtins.

Mantle already has the required lower boundary. A producer can evaluate Nixpkgs once and export concrete graphs for later Nix-free consumption.

Determinate Nix Nario v2 can transport store payloads and metadata. It does not contain package recipes or package-set meaning.

## Decision Drivers

- Generate a useful Mantle package collection without adding a Nix evaluator to Mantle consumption.
- Rebuild packages under Mantle-owned store paths and BLAKE3 identities.
- Keep package selection and policy reviewable in typed Nickel.
- Reuse shared graph nodes across package roots.
- Keep source transport separate from recipe meaning.
- Report unsupported packages without silent omission or Nix fallback.

## Decision

Mantlepkgs will be a generated package collection over concrete foreign derivation graphs.

A typed Nickel manifest will select one locked producer source, systems, package selectors, aliases, conversion policy, source policy, and named limits.

An explicit producer shell may run Nix before publication. It will emit foreign graphs, package indexes, source requirements, and producer receipts.

A pure Mantle core will merge selected graphs, reject conflicts, derive package dispositions, and produce a deterministic catalog plan.

The generated Nickel catalog will expose package names and bind confined graph references, root identities, policies, provenance, blockers, and BLAKE3 digests.

Large low-level graphs will remain generated machine artifacts. Mantle will not render them as handwritten-style Nickel derivation recipes.

Buildable entries will use recomputed target identities under the configured Mantle store prefix. Clean rebuild proofs will disable substitution.

The consumer path will not run Nix, `nix-store`, flakes, overlays, or another Nix evaluator.

Existing source bundles are the baseline source transport. Nario v2 can become an optional pinned input adapter without changing package recipe identity.

Each selected package will remain visible as buildable or blocked. Unsupported behavior will not cause silent omission, opaque source rewriting, or foreign-tool fallback.

## Alternatives Considered

### Translate Nixpkgs source directly into Nickel source

Rejected because general Nix evaluation cannot be preserved by a bounded source rewrite.

### Preserve original Nix output paths and use binary caches

Rejected as the primary Mantlepkgs route because the goal is Mantle-owned rebuilding and identity. Cache-preserved import remains a separate compatibility route.

### Store one complete graph for every package

Rejected because shared dependencies would be duplicated and conflicting translations could remain hidden.

### Put every low-level derivation field into generated Nickel

Rejected because it would create large generated source files without improving review, maintenance, or semantic clarity.

### Require Nario for all converter inputs

Rejected because Nario is a payload transport, not a recipe format. Existing source bundles must remain sufficient.

### Drop unsupported packages from the generated catalog

Rejected because absence would hide coverage gaps and make package-set updates hard to review.

## Consequences

- Nix remains a producer dependency for catalog updates.
- Mantle consumption and rebuilding can proceed without Nix.
- Generated catalogs remain tied to exact Nixpkgs locks, systems, selectors, policies, and source facts.
- Package identities differ from Nix cache identities.
- Some packages need explicit repairs for hard-coded `/nix/store` assumptions or unsupported builders.
- A successful initial cohort does not prove full Nixpkgs coverage, evaluator parity, correctness, reproducibility, or release eligibility.
