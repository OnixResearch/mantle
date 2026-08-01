## Context

The accepted cutover receipt records the exact `flake.nix` bytes observed on 2026-07-25. The live Nix check also requires every later `flake.nix` revision to retain that historical whole-file digest. This requirement is broader than the accepted source-agreement contract and fails after unrelated flake maintenance.

## Decisions

### Decision: Keep historical evidence immutable

**Choice:** Keep `evidence/radicle/artifact-auth-cutover-v1.ncl`, its JSON export, and its BLAKE3 sidecar unchanged.

**Rationale:** Those files record the reviewed cutover. Rewriting them after unrelated changes would replace historical evidence instead of validating it.

### Decision: Validate current source facts by scope

**Choice:** Use `nix-instantiate` to evaluate `inputs.artifactAuthSource.url` from the current flake. Require the accepted Radicle URL and revision. Continue to validate Cargo manifests, `Cargo.lock`, the artifact-auth `flake.lock` node, package membership, NAR identity, and forbidden fallbacks.

**Rationale:** Nix evaluation reads the selected attribute instead of accepting a matching comment or unrelated string. These scoped facts implement the accepted live source-agreement requirement. The digest of unrelated `flake.nix` content does not identify the artifact-auth source.

### Decision: Exercise positive and negative source fixtures

**Choice:** Run the scoped matcher against the current flake, a copy with an unrelated comment, and a copy with a wrong artifact-auth revision.

**Rationale:** The positive fixture prevents another whole-file binding. The negative fixture proves that source drift still fails closed.

## Risks / Trade-offs

- The focused derivation now includes the Nix evaluator. It evaluates one local attribute and does not fetch or write lock state.
- This repair does not generalize source admission for other dependencies.
