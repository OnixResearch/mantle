# Design: Native workspace package inheritance planning

## Model

Native manifest parsing represents package version fields as either literal strings or `{ workspace = true }` references. For the bounded inheritance case, the planner resolves the version from the root manifest's `[workspace.package].version` before computing package ids, target facts, source facts, and oracle comparison material.

## Failure behavior

The planner returns deterministic blockers when:

- a package lacks a literal version and does not request workspace inheritance.
- a package requests `version.workspace = true` but the root manifest lacks `[workspace.package].version`.
- the root workspace manifest cannot be read or parsed.
- the version field uses unsupported table keys beyond `{ workspace = true }`.

## Test strategy

Add a `rust_plan_cli` fixture with a root package using `[workspace.package]` plus `version.workspace = true`, `edition.workspace = true`, and `license.workspace = true`. The test asserts native package planning is ready and records the inherited version in native package facts.
