# Release filesystem capability boundaries

Mantle release code should treat local filesystem authority as an explicit shell
capability. The release capability boundary uses typed root roles for:

- release evidence roots;
- witness rebuild roots;
- bootstrap roots;
- build artifact roots; and
- store roots.

`release_capability::authorize_release_path` is the pure core that validates a
relative path request against a declared root authority. `ReleaseCapabilityRoot`
is the imperative shell wrapper around `cap-std` for relative reads and writes.
Pure release planning code must not open ambient paths, inspect the environment,
or resolve symlinks itself.

Passing this boundary proves only that a local file access was routed through a
declared capability root and relative path validator. It does not prove release
evidence correctness, build correctness, store correctness, or absence of bugs in
higher-level release policy.

## Functional-core API ownership

`crunch-release-core` takes owned local request values at its public decision
boundaries. Borrowed validation and hashing stay internal. Decisions that must
leave large release records available to the shell return those records alongside
their result: manifest evidence evaluations retain the manifest, canonical chapter
transport encoding retains the index or receipt, and Cairn validation retains the
handoff or validated receipt. Callers transfer ownership through these decisions
rather than cloning records merely to cross the boundary.

Publication staging validation returns the admitted plan for subsequent shell
phases. Commit eligibility observes only the publication phase and whether failures
exist; it does not allocate or grant filesystem authority. Tree-copy limits cross
the public boundary as local scalar values, with shared filesystem-planner types
confined to internal logic and shell conversion.

These ownership rules do not change release wire formats, digest domains, source
observation/review linkage, evidence eligibility, or the non-claims above.
