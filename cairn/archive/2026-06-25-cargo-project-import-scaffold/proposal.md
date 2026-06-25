# Proposal: Cargo project import scaffold

## Summary

Add a no-mutate planning and explicit apply workflow that imports an existing Cargo workspace into Mantle project files for the offline Cargo build lane.

## Motivation

Users should not have to hand-write the full Nickel project wrapper for a Rust workspace. Mantle needs a reviewable adapter that reads `Cargo.toml` and `Cargo.lock`, identifies supported package/target/source facts, and proposes the exact `mantle-project.ncl`, `.mantle/inputs.ncl`, and vendor/source configuration needed for `mantle build .#name`.

## Scope

- Add a pure import planner that reads normalized Cargo workspace facts and returns a deterministic Mantle project scaffold plan.
- Provide a no-mutate command to inspect the planned files, conflicts, supported packages, selected binaries, and blockers.
- Provide an explicit apply command that writes only bounded Mantle-owned files after conflict checks pass.
- Support the initial offline build target: local workspace package with a binary output and declared vendored or local path dependencies.
- Fail closed for unsupported Cargo surfaces instead of generating partial files that look supported.
- Add positive and negative tests for planning, conflict detection, apply behavior, generated Nickel syntax, and unsupported surfaces.

## Non-goals

- No automatic publication, release packaging, or cache push.
- No lockfile mutation or network vendoring in the first scaffold unless a later change explicitly defines that workflow.
- No Cargo-free native planner promotion.
- No module-layer inventory semantics; the scaffold emits build-shaped Mantle project data only.

## Target Spec Domains

- `project-workflows` for import/scaffold behavior and project-file mutation contracts.
- `build-tool-boundary` for keeping imported data build-shaped and frontend-neutral.
