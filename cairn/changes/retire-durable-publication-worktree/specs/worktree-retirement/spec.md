# Mantle Worktree Retirement Specification Delta

## ADDED Requirements

### Requirement: Promotion prerequisites

r[mantle.worktree_retirement.prerequisites]

Cleanup MUST require the archived promotion commit to be visible on Mantle remote `main`. It MUST run from a separate finalizer checkout.

#### Scenario: Promotion archive is remote

- GIVEN a finalizer checkout and fresh remote observation
- WHEN the archive commit is an ancestor of remote `main`
- THEN cleanup eligibility MAY be evaluated.

#### Scenario: Promotion archive is not remote

- GIVEN a missing or feature-only archive commit
- WHEN cleanup starts
- THEN cleanup MUST stop.

### Requirement: Pure cleanup eligibility

r[mantle.worktree_retirement.eligibility]

Eligibility MUST be deterministic over exact path, registration, clean status, branch reachability, remote ancestry, unpushed commit count, and active-operation facts.

#### Scenario: Exact target is eligible

- GIVEN a clean, published, idle, path-exact target
- WHEN eligibility is evaluated
- THEN the decision MUST permit only that target.

#### Scenario: Unsafe target is supplied

- GIVEN dirty, untracked, unpublished, wrong-path, or active-operation facts
- WHEN eligibility is evaluated
- THEN the decision MUST reject cleanup.

### Requirement: Version-control worktree removal

r[mantle.worktree_retirement.removal]

The shell MUST remove `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-durable-publication` through the version-control worktree operation. It MUST NOT use recursive deletion or delete the branch.

#### Scenario: Approved removal succeeds

- GIVEN an eligible exact target
- WHEN removal and prune run
- THEN the target directory and registration MUST be removed.

#### Scenario: Worktree removal fails

- GIVEN a version-control removal failure
- WHEN the shell handles the result
- THEN it MUST report failure without recursive fallback.

### Requirement: Original dirty checkout preservation

r[mantle.worktree_retirement.preservation]

Cleanup MUST preserve the original dirty Mantle checkout, all other worktrees, local branches, remote refs, and unrelated files.

#### Scenario: Unrelated manifests remain equal

- GIVEN before-and-after manifests excluding the target
- WHEN cleanup completes
- THEN every unrelated entry MUST remain unchanged.

#### Scenario: Original dirty checkout changes

- GIVEN any change to the original checkout
- WHEN preservation is checked
- THEN cleanup evidence MUST fail.

### Requirement: Bounded cleanup evidence

r[mantle.worktree_retirement.evidence]

Evidence MUST bind prerequisites, eligibility, removal, preservation, negative tests, BLAKE3 identities, and explicit non-claims.

#### Scenario: Complete evidence passes

- GIVEN matching facts and successful exact removal
- WHEN evidence is reviewed
- THEN the change MAY synchronize and archive.

#### Scenario: Evidence omits a rejection path

- GIVEN missing negative cases or mixed snapshots
- WHEN evidence is reviewed
- THEN the change MUST remain incomplete.
