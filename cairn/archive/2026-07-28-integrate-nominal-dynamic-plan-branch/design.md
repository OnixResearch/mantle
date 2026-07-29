# Design: Lossless integration of nominal dynamic-plan types

## Context

The source implementation was developed in an isolated worktree from commit `01dcfc6e05daf7dd56b0f483714bb695291f66ad`.

The reviewed implementation is commit `a2c872cfe6826df2d3ca7c85597365e6076fdfba`. Its tree is `17c155109a663ff3574c4865c3c905ee0e5ee486`.

At planning time, main points to `9f737cbc03ac1d17938d7bfdbdef32087bc32aae`. The main checkout also contains unrelated StageX edits.

A direct operation in the current checkout could mix integration work with those edits. A branch name alone also cannot bind immutable source content.

## Decisions

### Decision 1: Bind the immutable source commit and tree

**Choice:** Integration will use the recorded source commit and verify its commit, tree, parent, and changed-file set before application.

The mutable branch name is a discovery aid only. It is not the source-of-truth identity.

**Rationale:** Exact Git object identities prevent later branch movement from changing the reviewed integration input.

### Decision 2: Integrate only in a clean dedicated worktree

**Choice:** Select a committed main target after the unrelated StageX work has a preserved state. Create a dedicated integration branch and worktree from that target.

Do not apply the source commit in the current dirty main checkout.

**Rationale:** A clean worktree makes source changes, target changes, and conflict resolutions separately reviewable.

### Decision 3: Resolve conflicts as a semantic union

**Choice:** Preserve all unrelated target behavior while adding all reviewed nominal-domain behavior.

Conflict resolution will follow this file policy:

- `cairn/specs/build-correctness/spec.md`: keep all target requirements and add every nominal dynamic-plan requirement.
- `README.md` and documentation: keep target documentation and the nominal migration link and guidance.
- `Cargo.toml` and `dylint.toml`: keep target metadata and add the reviewed Octet scope and nominal-domain policy.
- `dynamic_plan.rs` and `dynamic_plan/wire.rs`: preserve target behavior while retaining private nominal values and explicit wire admission.
- `worker.rs`: preserve target worker changes while keeping typed maps, bindings, roots, and adapters.
- archived source evidence and the canonical fixture: import the reviewed files without rewriting their historical claims.

Conflict resolution must not add raw-string fallback paths into typed graph logic.

**Rationale:** Choosing either side wholesale can silently delete newer target work or validated nominal boundaries.

### Decision 4: Revalidate the resolved target, not only the source commit

**Choice:** Treat source-branch results as baseline evidence. Run all required checks again on the resolved integration tree.

The validation set includes:

- focused `crunch-build` positive, negative, compatibility, and compile-fail tests;
- canonical JSON comparison and the accepted plan BLAKE3 digest;
- Octet nominal-domain denial with zero targeted findings;
- first-party Clippy and Tiger Style checks;
- relevant Nix formatting and Clippy checks;
- broader workspace tests with exact triage for pre-existing or nondeterministic failures;
- Cairn validation and proposal, design, and tasks gates.

**Rationale:** A clean source branch does not prove that conflict resolution preserved behavior on a newer target.

### Decision 5: Keep claims and lifecycle records separate

**Choice:** Preserve the archived source change as implementation evidence. Use this active change only for integration evidence and target reconciliation.

After validation, sync this delta, archive this package, and commit the resolved integration on its dedicated branch.

Do not push or move main without explicit operator instruction.

**Rationale:** The source archive proves the isolated implementation review. The integration archive proves the resolved target review.

## Failure Semantics

- Abort if the source commit, tree, or parent differs from the recorded receipt.
- Abort if the selected target checkout is dirty before source application.
- Treat unresolved conflicts, deleted target changes, or missing source files as blockers.
- Treat any raw nominal-domain fallback or changed canonical digest as a blocker.
- Record broad test failures exactly. Do not hide them behind blanket exclusions.
- Keep the integration change active until required checks pass or an exact bounded blocker is accepted.

## Risks and Trade-offs

- Main can continue to advance before integration starts. The evidence must record the actual selected target commit.
- Source and target changes can overlap in large Rust files. Semantic review is required after textual conflict resolution.
- Broad workspace tests have known base failures and parallel flakes. Focused reruns must distinguish those from integration regressions.
- The extra integration archive adds lifecycle data, but it keeps source review and target review distinct.
