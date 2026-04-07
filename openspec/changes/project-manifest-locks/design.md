## Context

crunch currently exposes engine-oriented commands (`build`, `eval`,
`bootstrap`, `store`, `self-build`) and a Nickel stdlib for derivations and
fetchers. That is enough to build single derivations, but not enough to manage
pinned project inputs over time.

The missing pieces are not in the scheduler or the store. They are in the
project layer:

- a manifest the user edits
- a lockfile the machine edits
- refresh logic that resolves new revisions and hashes
- stale detection
- upgrade logic for format changes
- a stable way for package code to consume locked inputs

`../nixtamal/` is a good model for that split. It is not a good fit to copy
verbatim. Its KDL syntax and Nix-prefetch commands are tied to the Nix world.
crunch should reuse the shape of the workflow, not the implementation.

## Goals / Non-Goals

**Goals:**
- Keep project input management in a dedicated workspace crate
- Use Nickel for the human-edited manifest
- Version both manifest and lock formats from the start
- Keep evaluation and build execution pure with respect to network updates
- Materialize locked inputs in a form package code can import directly
- Add CLI commands for init, check, show, refresh, list-stale, and upgrade

**Non-Goals:**
- Replacing the existing build pipeline
- Making the project layer a separate repo or standalone tool
- Copying nixtamal's KDL manifest format
- Running network fetches during `crunch eval` or normal `crunch build`
- Adding Darcs, Pijul, or Fossil support in this change
- Building a TUI

## Decisions

### 1. Add a workspace crate, not a separate repo and not binary-only code

**Choice:** Add `crates/crunch-project` to the workspace. The `crunch` binary
parses CLI flags and delegates project commands to that crate.

**Rationale:** Project input management is a separate layer from
`crunch-pipeline`, `crunch-build`, and `crunch-store`. It has its own data
model, migrations, and user-facing workflows. Putting it in `src/main.rs`
would bloat the binary crate. Making it a separate repo would split tightly
coupled semantics across projects with no clear gain.

**Alternative rejected:**
- Add the logic directly to the binary crate. Rejected because the binary would
  accumulate parsing, refresh, upgrade, and materialization logic that does
  not belong to CLI dispatch.
- Create an external tool. Rejected because the manifest, lock, and generated
  input file are crunch-specific and should evolve with crunch's stdlib and
  fetchers.

### 2. Human manifest in Nickel, machine lock in JSON

**Choice:** Use `crunch-project.ncl` for the human-edited manifest and
`crunch.lock` for the machine-edited lockfile.

**Rationale:** The user-facing format should match crunch's language and reuse
Nickel contracts. The lockfile should optimize for stable serialization,
rewrites, and upgrades. JSON is easier to write atomically and migrate than a
machine-edited Nickel file.

**Alternative rejected:**
- KDL manifest. Rejected because crunch is Nickel-first and would gain a
  second config language for little benefit.
- Nickel lockfile. Rejected because a machine-edited Nickel file is harder to
  normalize and migrate cleanly.

### 3. Generate `.crunch/inputs.ncl` from the lock

**Choice:** Materialize a generated `.crunch/inputs.ncl` file from the lock.
Package code imports that file to get resolved inputs.

**Rationale:** This keeps the normal eval/build path pure. `crunch refresh`
does the impure update work once, writes the lock, and regenerates the inputs
file. After that, `crunch eval` and `crunch build` only read local files.
This also avoids adding hidden evaluator state or special runtime injection.

**Alternative rejected:**
- Inject locked inputs directly into the evaluator at runtime. Rejected because
  it hides dependencies and makes `crunch eval` depend on extra ambient state.
- Fetch during `crunch build`. Rejected because it mixes project updates with
  actual builds.

### 4. Reuse existing fetch/build primitives

**Choice:** `crunch-project` resolves metadata, but actual fetching, patch
application, and fixed-output verification stay in the existing fetcher and
build layers.

**Rationale:** The project layer should decide *what* inputs are locked.
`crunch-build` and the Nickel stdlib should still decide *how* those inputs are
fetched and built. This keeps one implementation of network fetch, hash
verification, and patch application.

**Alternative rejected:** Add a second fetch implementation inside the project
crate. Rejected because it would duplicate verification, decompression, and
future fetcher fixes.

### 5. Mirrors, patches, and refresh metadata belong in the manifest model

**Choice:** The manifest model includes mirrors, patch lists, frozen inputs,
and refresh metadata. The lockfile stores the resolved values needed to build
those inputs repeatably.

**Rationale:** These are project authoring concerns. They should be reviewed
in the manifest and concretized in the lock. Keeping them out of package files
reduces repeated boilerplate and gives `refresh` and `list-stale` a stable data
model to operate on.

### 6. Version the format immediately and ship `crunch upgrade`

**Choice:** Both `crunch-project.ncl` and `crunch.lock` carry schema versions.
`crunch-project` owns migrations and the CLI exposes `crunch upgrade`.

**Rationale:** This layer will change. Adding versions after the first format
ship makes every migration harder.

## Data Flow

```text
crunch-project.ncl   crunch.lock
        │                │
        ├──── merge/validate ────┐
        │                        │
        └──── refresh/update ────┘
                    │
                    ▼
           .crunch/inputs.ncl
                    │
                    ▼
        package Nickel imports locked inputs
                    │
                    ▼
          crunch eval / crunch build
```

## Risks / Trade-offs

**[Generated file drift]** `crunch.lock` and `.crunch/inputs.ncl` can diverge
if one write succeeds and the other fails. Mitigation: write both atomically
via temp files + rename, and make `crunch check` detect drift.

**[Two user-visible files instead of one]** Manifest + lock + generated inputs
is more surface area than a single file. Mitigation: keep only the manifest
human-edited; treat the other two as generated artifacts.

**[Scope creep]** Mirrors, patches, refresh, and upgrades can balloon into a
second package manager. Mitigation: keep the project crate focused on pinning
and materialization. Build execution stays in the existing engine.

**[Format lock-in]** Choosing JSON for the lock constrains future extensions.
Mitigation: version the schema from the start and keep migrations inside the
project crate.
