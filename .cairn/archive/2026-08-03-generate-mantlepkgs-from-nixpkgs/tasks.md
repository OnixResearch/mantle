# Tasks: generate Mantlepkgs from Nixpkgs

## Phase 1: Contracts and pure planning

- [x] [serial] I1 Add typed Nickel contracts for the Mantlepkgs source lock, systems, selectors, aliases, policies, and named limits. r[mantlepkgs.catalog_manifest]
- [x] [serial] I2 Add a pure core for manifest validation, graph merging, package dispositions, deterministic ordering, and BLAKE3 catalog identity. r[mantlepkgs.functional_core]
- [x] [parallel] I3 Add positive and negative core tests for duplicate nodes, conflicting nodes, aliases, ordering, stale digests, missing roots, and limit failures. r[mantlepkgs.functional_core]

## Phase 2: Producer and catalog publication

- [x] [serial] I4 Add the explicit Nixpkgs producer shell and bind its Nix version, Nixpkgs lock, systems, selectors, command class, and outputs. r[mantlepkgs.producer_boundary]
- [x] [serial] I5 Emit shared foreign graphs, package indexes, source requirements, producer receipts, and one generated Nickel catalog. r[mantlepkgs.catalog_generation]
- [x] [serial] I6 Stage, validate, and publish complete catalog generations atomically without silently dropping failed selections. r[mantlepkgs.catalog_generation]
- [x] [parallel] I7 Add deterministic diagnostics and catalog blockers for unsupported builtins, missing sources, unresolved paths, graph faults, and hard-coded source-store assumptions. r[mantlepkgs.unsupported_package_diagnostics]

## Phase 3: Mantle rebuild path

- [x] [serial] I8 Add package lookup that verifies the generated catalog, artifact confinement, artifact digests, system, package root, and conversion policy. r[mantlepkgs.recomputed_rebuild]
- [x] [serial] I9 Compile selected roots under the configured Mantle store prefix and realize them through the ordinary Mantle worker. r[mantlepkgs.recomputed_rebuild]
- [x] [parallel] I10 Ensure catalog consumption and rebuilding work with Nix commands absent from `PATH` and never fall back to Nix. r[mantlepkgs.producer_boundary]
- [x] [parallel] I11 Keep source-bundle preparation explicit and allow optional transport adapters without making transport identity part of recipe meaning. r[mantlepkgs.catalog_generation]

## Phase 4: Live package cohort and failure evidence

- [x] [serial] I12 Convert a pinned cohort with an executable root, a library root, and a root that consumes a selected library. r[mantlepkgs.validation]
- [x] [serial] I13 Rebuild the accepted cohort under the Mantle store prefix with substitution disabled and retain exact graph, plan, source, build, output, and receipt identities. r[mantlepkgs.validation]
- [x] [parallel] I14 Retain negative evidence for stale locks, missing source records, digest tampering, alias conflicts, cycles, unsupported builtins, leftover Nix paths, and partial batch failure. r[mantlepkgs.validation]
- [x] [parallel] I15 Document package dispositions and the explicit non-claims for source translation, evaluator parity, full Nixpkgs coverage, correctness, reproducibility, and release eligibility. r[mantlepkgs.claim_boundary]

## Phase 5: Verification and lifecycle

- [x] [serial] V1 Run focused core, producer, catalog, foreign-plan, source-materialization, realization, CLI, and machine-contract tests. Record exact output in `evidence/verification.md`. r[mantlepkgs.validation]
- [x] [serial] V2 Run formatting, focused Clippy, workspace checks, Nickel checks, `git diff --check`, and the live no-Nix rebuild rail. r[mantlepkgs.validation]
- [x] [serial] V3 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, and the relevant Nix checks. r[mantlepkgs.validation]
