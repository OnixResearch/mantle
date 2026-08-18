# Design: Cargo project import scaffold

## User flow

The intended operator flow is:

```sh
mantle import cargo --plan
mantle import cargo --apply
mantle build .#my-bin
mantle run .#my-bin -- --help
```

Exact command spelling may change during implementation, but the boundary must preserve separate no-mutate planning and explicit apply phases.

## Pure planner

The pure planner accepts normalized Cargo facts and import options:

- workspace root and member list;
- package names, versions, editions, target kinds, and binary names;
- lockfile digest and selected package lock entries;
- local path dependency roots;
- vendored registry/git source roots when present;
- desired Mantle project file paths;
- conflict policy and naming policy.

It returns a deterministic `CargoImportPlan` containing:

- generated file operations with full content digests;
- selected package outputs and default package choice;
- required source-closure entries;
- unsupported-surface blockers;
- conflict diagnostics;
- post-apply build hints.

The planner must be testable without touching the filesystem.

## Imperative shell

The shell owns file reads, path canonicalization, command-line parsing, conflict checks against the live worktree, and writes. `--plan` must not mutate project files, lockfiles, generated directories, store state, or Cargo files. `--apply` may only write the bounded files named in the accepted plan.

## Generated files

The first implementation should generate Mantle-canonical surfaces:

- `mantle-project.ncl` for the project manifest;
- `mantle.lock` only when the workflow owns Mantle-specific source input state;
- `.mantle/inputs.ncl` for generated source bindings.

Compatibility aliases such as `crunch.ncl` may remain supported, but new scaffold prose and generated comments should prefer Mantle naming unless an exact compatibility path requires `crunch`.

## Conflict behavior

Planning must report conflicts for existing non-equivalent files, mixed canonical/legacy project surfaces, missing lockfile/source material, unsupported target kinds, ambiguous default package selection, or output names that would collide after Nickel quoting. Apply must fail before writing when any blocking conflict remains.

## Verification strategy

- Positive planner tests with a small Cargo workspace produce deterministic file operations and selected package outputs.
- Positive apply tests write expected Mantle files to a temp workspace and then parse/evaluate the generated Nickel.
- Negative tests cover existing conflicting files, missing `Cargo.lock`, unsupported dependency source, ambiguous binary selection, malformed package names, and non-UTF-8 paths.
- Integration smoke builds the generated project through the offline Cargo build lane once that lane exists.

## Requirement trace

- r[project_workflows.cargo_import_scaffold]
