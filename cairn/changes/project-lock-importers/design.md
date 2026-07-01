# Design: Project lock importers

## Architecture

Importers use an imperative parser shell and pure conversion core.

- Pure core: normalized external pin model validation, compatibility mapping, unsupported-surface classification, Mantle manifest/lock generation planning, file operation planning, and non-claim diagnostics.
- Imperative shell: reading external files, parsing KDL/JSON/Nix-adjacent lock data through bounded parsers, writing reviewed Mantle files, and CLI rendering.

The core should not depend on Nixtamal, flake, npins, or niv parsers directly. It should consume a normalized `ExternalPinSet` and produce an `ImportPlan`.

## Import flow

`plan` reads external files and emits a deterministic report with candidate Mantle project files, preserved semantics, rewritten semantics, blockers, and non-claims. It must not mutate files. `apply` requires a blocker-free plan or an explicit reviewed plan artifact and writes only Mantle-owned files named by that plan.

Nixtamal import should map supported inputs, mirrors, patches, default/per-input hash algorithms, frozen flags, freshness probes, fetch policy, and trust policy when available. Unsupported command probes or VCS kinds should be blockers until Mantle has equivalent support.

## Boundary rules

Importers must map source pinning, not project composition semantics. Flake output schemas, module systems, follows semantics, overlays, and recursive input graphs must either become explicit source inputs with bounded meaning or blockers.

## Validation strategy

Pure positive tests should cover Nixtamal fixture imports for file, tarball, Git, mirrors, patches, BLAKE3, frozen inputs, and supported freshness/fetch/trust metadata. Pure negative tests should cover unsupported source kinds, malformed external files, recursive semantics, lossy hash downgrade, unknown patch references, conflicting existing files, and apply attempts with blockers.

CLI tests should prove plan is no-mutate and apply writes only planned files.
