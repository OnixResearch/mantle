# Proposal: Project file generation

## Summary

Adopt Organist's useful file-generation idea in Mantle form: projects may declare generated files as typed data, and operators may run explicit `mantle filegen plan` and `mantle filegen apply` workflows to review and materialize those files.

## Motivation

Projects often need generated config files, evidence manifests, and frontend handoff files. Organist's `files.*` model is valuable because it centralizes generated-file content and lets Nickel contracts catch mistakes early. Mantle should adopt the explicit typed-file model while preserving Mantle's no-mutate-by-default operator style.

Unlike Organist, Mantle should not mutate the worktree implicitly when entering a shell. File generation should be reviewable, receipt-bound, and explicit.

## Scope

- Add project manifest support for generated file declarations with content, target, materialization method, and optional contract/schema binding.
- Add `mantle filegen plan` as a no-mutate review path.
- Add `mantle filegen apply` as an explicit mutation path that writes only the reviewed file operations.
- Report conflicts, stale generated content, target escapes, and unsupported materialization methods deterministically.
- Reuse Nickel export receipts when generated file content comes from Nickel evaluation.

## Non-goals

- No automatic regeneration during `mantle shell` or build unless a later explicit change opts in.
- No service manager or long-running process lifecycle.
- No hidden formatter, package-manager, or frontend-specific semantics.
- No writing outside the project root unless a future explicit policy defines that boundary.

## Target Spec Domains

- `project-workflows` for file generation declaration, plan/apply behavior, conflicts, and typed content validation.
- `operator-diagnostics` may receive follow-on rendering requirements if filegen diagnostics need more detail than project soundness diagnostics provide.
