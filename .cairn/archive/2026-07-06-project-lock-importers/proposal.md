## Why

The `project-workflows` spec already accepts
`[depends:project_workflows.project_lock_importers]` and
`[depends:project_workflows.nixtamal_importer]`: Mantle MUST provide a no-mutate import
planning workflow and an explicit apply workflow that convert supported external
pinning files (including Nixtamal) into Mantle project manifests, lockfiles, and
generated input files, preserving or blocking source kind, URL/repo, mirrors,
patches, hash algorithm, expected hash, frozen state, freshness, fetch policy,
and trust policy. Partial surface exists (`src/pin_import.rs`, `src/cargo_import.rs`,
`crates/crunch-project/src/lib.rs`, `src/main.rs`), but the plan/apply boundary
and the composition-semantics blocker behavior are not proven as a bounded
offline proof rail with versioned, non-overclaiming evidence.

## What Changes

- Audit the existing importer surface against every scenario clause of the
  accepted requirements.
- Provide a bounded local offline proof rail that exercises the external pin
  import `--plan` and `--apply` workflow for a supported importer (Nixtamal):
  plan is side-effect free and reports planned file operations, mapped
  inputs/patches, unsupported semantics, and blockers; apply writes only
  Mantle-owned files named by the plan and fails before writing on conflicts,
  unsupported semantics, or plan drift.
- Emit a versioned, redacted, non-overclaiming evidence record that states the
  generated project remains a build-tool handoff (not Onix/NixOS module
  semantics).
- Add negative cases: composition semantics (recursive graph, flake output
  composition, module-layer behavior, follows-like rewriting, overlays) become
  explicit source inputs with bounded meaning or deterministic blockers, not
  hidden core semantics; apply fails on plan drift or conflict.

## Impact

- **Files**: `src/pin_import.rs`, `src/cargo_import.rs`, `src/main.rs`,
  `crates/crunch-project/src/lib.rs`, a bounded offline proof rail, and an
  evidence-render helper.
- **Testing**: positive plan-then-apply case, no-mutate plan assertion,
  composition-blocker negative case, apply-drift negative case, build-tool-handoff
  non-claim assertion, and the Cairn gates.

## Out of Scope

- Importing unsupported pinning formats beyond Nixtamal and Cargo scaffolds.
- Treating import as build success or deployability.
- Changes to the accepted `project_lock_importers` or `nixtamal_importer`
  requirement text.
