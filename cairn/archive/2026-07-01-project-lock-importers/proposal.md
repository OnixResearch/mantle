# Proposal: Project lock importers

## Summary

Add no-mutate import planning and explicit apply workflows for migrating external pinning files into Mantle project manifests and lockfiles, starting with Nixtamal and leaving room for flakes, npins, and niv.

## Motivation

Nixtamal's roadmap includes importing other pinning tools. Mantle should reciprocate because migration is a major adoption barrier. Operators need a safe way to inspect what Mantle can preserve, what must be rewritten, and what is unsupported before any project files are changed.

Importers should map only source pinning facts into Mantle's build-tool project model. They must not import unrelated composition semantics or claim support for behavior Mantle cannot represent.

## Scope

- Add `mantle import pins plan|apply` or equivalent import surfaces.
- Define importer contracts for Nixtamal first, with future adapters for flakes, npins, and niv.
- Preserve supported source kinds, mirrors, patches, hash algorithms, frozen flags, freshness policy, fetch policy, and trust policy when Mantle can model them.
- Emit deterministic blockers for unsupported source kinds, command probes, hash algorithms, recursive inputs, or composition semantics.
- Keep plan no-mutate and make apply write only reviewed Mantle-owned project files.

## Non-goals

- No recursive flake graph solver.
- No import of NixOS/Onix module semantics into Mantle.
- No silent downgrade of unsupported freshness, trust, mirror, or patch semantics.

## Target Spec Domains

- `project-workflows` for import plan/apply behavior.
- `build-tool-boundary` for ensuring external composition semantics remain outside Mantle core.
