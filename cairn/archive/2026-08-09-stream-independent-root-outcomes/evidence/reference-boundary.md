# `nix-eval-jobs` reference boundary

Question: Which external concepts can Mantle adapt without importing external code or compatibility authority?

Inspected evidence:

- Repository: `NixOS/nix-eval-jobs`
- Revision: `a0cd02231c58974a6b5aaa3712069b071047162e`
- `README.md` at that revision
- `LICENSE.md` at that revision

License decision:

- The referenced project uses GNU GPL version 3.
- Mantle imports no source, generated file, library, schema, fixture, or runtime dependency from that project.
- This change records behavior concepts only.

Selected concepts:

- Independent jobs can report individual evaluation errors.
- Machine consumers can read one JSON record per line.
- Bounded workers can evaluate roots in parallel.
- Cache observations can appear as optional per-job facts.

Rejected compatibility behavior:

- Nix attribute traversal and `recurseForDerivations`
- `--force-recurse`
- arbitrary `--apply` or `--select` evaluator callbacks
- Hydra aggregate and constituent semantics
- Nix private evaluator libraries
- Nix GC-root behavior
- Nix attribute paths as Mantle root identity
- external output fields as Mantle wire compatibility requirements

Non-claims:

- This review does not prove semantic equivalence.
- This review does not transfer Nix, Hydra, cache, store, build, or CI authority.
- This review does not authorize copied GPL source.
- Similar record streaming does not prove implementation independence by itself.

Implementation guard:

- `scripts/check-evaluation-stream-contract.rs` scans bounded first-party Rust and Cargo manifest inputs.
- The guard permits the external project name only in comments and the approved `docs/`, `adr/`, and `cairn/` review boundary.
- Its negative self-test proves that a product-source import is rejected.
- Pueue task `16923` passed the self-test and the repository check.

Decision: Adapt independent error reporting and NDJSON framing under Mantle-owned types, identity, bounds, tests, and claims.

Owner: `stream-independent-root-outcomes`.

Next action: Keep dependency and source guards in the implementation validation set.
