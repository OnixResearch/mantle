# Design: Native git dependency source scope

## Context

`rust_plan.source_closure` already records git package evidence for `wu-manber`:

- package identity: `git+https://github.com/tvlfyi/wu-manber.git#wu-manber@0.1.0`
- lockfile source: `git+https://github.com/tvlfyi/wu-manber.git#0d5b22bea136659f7de60b102a7030e0daaa503d`
- resolved revision: `0d5b22bea136659f7de60b102a7030e0daaa503d`
- materialized manifest path under Cargo's git checkout

Native package dependency planning currently only resolves path dependencies and declared vendored-registry sources. It therefore rejects `wu-manber` as `unsupported-non-path-dependency`, which prevents native facts for `snix-castore` and cascades into later graph blockers.

## Decisions

### 1. Add a bounded native git source fact layer

**Choice:** Add native git source facts derived from the captured source-closure records for locked git packages.

**Rationale:** The lockfile and source closure already provide the exact URL, revision, package identity, and materialized source root. Reusing those explicit inputs is narrower and safer than invoking git, fetching the network, or attempting version solving inside native planning.

### 2. Bind git source roots with BLAKE3 tree digests

**Choice:** Each ready git fact records the lockfile digest, URL, resolved revision, source root/manifest path, and BLAKE3 source-tree digest. The URL/revision identify the locked package; the source bytes remain provider-agnostic captured source material so the default source path can be snix-store-backed BLAKE3 content rather than a hard-coded git pull.

**Rationale:** The lock revision alone proves identity, not the bytes fed to rustc. A BLAKE3 tree digest makes the source material auditable and comparable in the same style as vendored registry source facts while leaving materialization owned by the source-closure/store layer.

### 3. Dependency resolution consumes only ready facts

**Choice:** Native dependency edge resolution may resolve a selected dependency through a ready native git source fact only when package name, optional manifest version, git URL, and requested rev/tag/branch identity match. Missing, unreadable, mismatched, or ambiguous git source evidence remains a deterministic blocker before rustc execution.

**Rationale:** This unblocks the `snix-castore -> wu-manber` chain without weakening fail-closed boundaries for other non-path dependencies or coupling source materialization to git-specific fetching.

### 4. No network or ambient-cache discovery

**Choice:** The implementation must not run `git`, fetch URLs, scan `$CARGO_HOME`, or inspect arbitrary Cargo cache paths while building native git facts. It may only use package source roots already present in the captured source closure for the current oracle run.

**Rationale:** The native planner should move toward explicit source inputs, not hidden Cargo behavior. This keeps the first git slice bounded and reviewable.

## Risks / Trade-offs

- Using source-closure materialized paths still depends on the oracle phase to provide a checkout. This is acceptable for the bounded replacement phase but must remain explicit in receipts.
- Git submodules, sparse checkouts, path-within-repo packages, and multiple packages from one git repository may require later scope expansion. This change should fail closed unless the package root and manifest path are unambiguous.
- Adding a separate public receipt fragment is cleaner than overloading registry terminology, but it may require JSON consumers to tolerate a new optional field.
