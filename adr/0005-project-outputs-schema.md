# ADR 0005: Project Outputs Schema

## Status

Proposed

Rename amendment (2026-05-14): this ADR records the pre-rename command naming.
The canonical command surface is now `mantle build`, `mantle shell`, and
`mantle run`; the package/build project root intentionally remains the
compatibility-named `crunch.ncl`. Dependency-management defaults are now
`mantle-project.ncl`, `mantle.lock`, and `.mantle/inputs.ncl`, with the old
Crunch names retained only as migration inputs.

## Context

crunch has a project manifest (`crunch-project.ncl`) that pins external
inputs — tarballs, git repos, files. But there is no standard way to
declare what a project *produces*. Every `crunch build` invocation
requires an explicit file path. There is no `crunch build` (no args)
that does the obvious thing, no `crunch build .#hello` selector, no
`crunch develop` to enter a dev shell, no `crunch check` that runs
project-defined checks.

Nix flakes solved this with a fixed output schema (`packages`,
`devShells`, `checks`, `apps`, `overlays`, etc.) keyed by system.
The schema is useful but also widely criticized:

1. The `system` key forces cross-product boilerplate (`eachSystem`).
2. `apps` vs `packages` is a confusing distinction.
3. `overlays` and `nixosModules` couple the schema to Nix internals.
4. `checks` vs `packages` creates confusion about what gets built.
5. No typing — the schema is enforced by convention, not contracts.

crunch can learn from flakes while avoiding these problems. Nickel
contracts enforce the schema at eval time. crunch's default
`system = 'x86_64-linux` means single-system projects need no
system key at all. crunch has no overlay/module system to cater to.

## Decision

### Project root file

A `crunch.ncl` file in the project directory is the standard entry
point. It returns a record matching the `Project` contract (defined
in `lib/project_outputs.ncl`). `crunch build` with no file argument
looks for `crunch.ncl` in the current directory (then parents).

### Output categories

```nickel
{
  # The things you ship. Each value is a Derivation or record of Derivations.
  packages | { _ : Derivation } | default = {},

  # Development shells. Each is a Derivation whose $out/bin is on PATH.
  devShells | { _ : Derivation } | default = {},

  # CI checks. `crunch check` builds all of these.
  checks | { _ : Derivation } | default = {},

  # Default targets for bare `crunch build` / `crunch develop`.
  default | {
    package | String | optional,
    shell   | String | optional,
  } | default = {},
}
```

Four categories, not seven. No `apps` (just `packages` — if it has a
`bin/`, `crunch run` finds the executable). No `overlays` or `modules`
(crunch has no overlay or module system). No per-system key at the top
level (the `system` field on each derivation handles it; cross-compile
is a future concern).

### Selectors

`crunch build .#hello` resolves to `packages.hello`. The `.#` prefix
distinguishes project attribute selectors from file paths.

Resolution order for `.#foo`:
1. `packages.foo`
2. `checks.foo`
3. Top-level field `foo` (for backward compat with bare record sets)

`crunch build` (no args) resolves `default.package` → looks up that
name in `packages`. If unset, builds all `packages`.

### New commands

- `crunch build` — no args, uses `crunch.ncl`
- `crunch build .#name` — build specific output
- `crunch develop` — enter dev shell (`default.shell` or only shell)
- `crunch run .#name [-- args...]` — build + exec first `bin/*`
- `crunch check` gains dual behavior: with `crunch-project.ncl` it
  validates project state (current behavior); with `crunch.ncl` it
  also builds all `checks`

### Relation to crunch-project.ncl

`crunch-project.ncl` remains the input pinning manifest. `crunch.ncl`
is the output declaration. They're separate files. `crunch.ncl` can
import `.crunch/inputs.ncl` (generated from the lockfile) to get
pinned input store paths.

A project can have:
- Just `crunch.ncl` (no pinned inputs, like a simple example)
- Just `crunch-project.ncl` (input pinning only, no standard outputs)
- Both (full project setup)

### Contract enforcement

The `Project` contract is in `lib/project_outputs.ncl`. `crunch build`
applies it to the `crunch.ncl` output before extracting targets. Eval
errors from contract violations are clear Nickel diagnostics, not
runtime crashes.

### Backward compatibility

`crunch build file.ncl` still works for any .ncl file. The project
schema only activates when using selectors or bare `crunch build`.
Existing examples and bootstrap files are unaffected.

## Alternatives Considered

### Per-system keying at the top level

```nickel
{ packages.x86_64-linux.hello = ..., ... }
```

Rejected. Creates the same boilerplate problem as flakes. crunch
derivations already carry a `system` field. If cross-compilation is
needed later, add a `systems` helper function, not a schema change.

### Single flat namespace

All outputs in one record, tagged with metadata.

Rejected. Separating `packages` from `checks` from `devShells` has
real value: `crunch check` should only build checks, `crunch develop`
should only look at shells. Flat namespaces force the user to tag
everything manually.

### Embed outputs in crunch-project.ncl

Put the output schema in the input manifest.

Rejected. The manifest is loaded via `evaluate_and_deserialize` into
a Rust struct. Build targets need Nickel evaluation (they reference
store paths, compose derivations). Mixing both in one file forces
either two-pass evaluation or a much more complex Rust type.

## Consequences

- `crunch.ncl` becomes the canonical project root file.
- `crunch build` with no args does the right thing in a project dir.
- `crunch develop` and `crunch run` become possible.
- The `Project` contract catches schema violations at eval time.
- Existing `crunch build file.ncl` usage is unaffected.
- Future work: `crunch fmt`, `crunch template`, cachix-style push.
