## Context

This change does not introduce new project-management requirements. The core
resolver, hashing, and stale-detection behavior is already captured in
`openspec/specs/project-management/spec.md`. The delta carried by this change
is the CLI-facing failure-reporting behavior in `specs/cli/spec.md`.

The project-management layer is split correctly on paper:

- `crunch-project` owns manifest, lock, refresh, and stale logic
- the `crunch` binary owns CLI parsing and I/O

That split is still the right one. The problem is that the imperative shell is
unfinished. `src/project_cmd.rs` still constructs `StubResolver`, so the pure
core never receives real git or hashing facts.

A second issue sits at the API boundary. `list_stale()` currently collapses
results down to a list of stale names, which throws away resolver failures.
That makes it too easy for the CLI to print a clean result when checks were
never completed.

## Goals / Non-Goals

**Goals:**
- Finish the binary-shell resolver without moving I/O into `crunch-project`
- Match refresh hash semantics to the existing fetch/build helpers
- Bring the implementation into conformance with the existing
  `project-management` spec
- Preserve stale-check failures all the way to CLI rendering
- Keep partial success behavior: one failing input must not block unrelated
  updates

**Non-Goals:**
- Async refresh or background refresh jobs
- New input kinds
- Authentication or private-repo support
- A generic download library for unrelated commands

## Decisions

### 1. Keep the live resolver in the binary crate

**Choice:** Implement `LiveResolver` in `src/project_cmd.rs` or a nearby
binary-owned module.

**Rationale:** The resolver performs subprocess, filesystem, and network I/O.
That belongs in the imperative shell. `crunch-project` stays deterministic and
unit-testable.

**Alternative rejected:** Move the resolver into `crunch-project`. Rejected
because it would violate the FCIS split already documented for the crate.

### 2. Return structured stale results, not only stale names

**Choice:** Change the stale-reporting path so the core returns stale items and
failed checks separately.

**Rationale:** The CLI cannot render truthful output once failures are reduced
away. The API must preserve that distinction.

**Alternative rejected:** Re-run resolution in the CLI just to recover failure
information. Rejected because it duplicates work and splits logic across
layers.

### 3. Reuse fetch/build hash semantics as the source of truth

**Choice:** Match existing fetch helpers exactly:
- plain files and patch files use flat content hashes
- tarballs and git checkouts use recursive/NAR tree hashes

**Rationale:** Refresh only works if the resulting lock values are accepted by
`crunch.fetchTarball`, `crunch.fetchGit`, and patch application later.

**Alternative rejected:** Use simpler archive-byte hashing for tarballs.
Rejected because it would write lock entries the fetch path rejects.

### 4. Permit partial refresh writes, but fail the command

**Choice:** Successful refresh outcomes may still update the lockfile even when
other selected inputs fail. The command exits non-zero and reports the failed
items.

**Rationale:** This matches the current spec direction and avoids throwing away
useful successful updates.

## Risks / Trade-offs

**[Semantic drift from fetchers]** A resolver implementation can quietly drift
from build-time hashing rules. Mitigation: add integration tests that compare
real lock output for git and tarball inputs.

**[Tooling assumptions about `list-stale`]** Callers may assume an empty stale
list means success. Mitigation: return structured results and update the CLI to
make failure states explicit.

**[Host dependency on `git`]** Git resolution depends on the host binary.
Mitigation: return a clear `git not found` error and cover it in tests.
