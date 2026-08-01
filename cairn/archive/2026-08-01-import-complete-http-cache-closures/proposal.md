# Proposal: Import complete HTTP cache closures

## Why

`mantle store pull` imports only the explicit HTTP cache paths named by the operator. A successful root import therefore does not prove that its referenced runtime closure is present.

Nixpkgs substitution-first use needs a stricter boundary. Mantle must discover and validate the complete signed narinfo graph before it downloads NAR content or reports a usable root.

## What Changes

- Add a bounded, pure closure planner over owned narinfo observations.
- Bind the root, cache authority, trust policy, store prefix, limits, and member facts into a BLAKE3 plan identity.
- Add strict HTTP closure discovery before NAR download.
- Reuse complete local members and fetch missing or incomplete members.
- Import dependencies before the selected root.
- Add `mantle store pull --closure <root>` for one explicit HTTP root.
- Emit closure-plan facts in the pull report without claiming package correctness or rebuild compatibility.
- Add positive and negative fixtures for graph shape, limits, trust failures, missing members, incomplete local content, and root-last mutation.

## Dependencies

- ADR 0054 keeps selected Snix transport fixes compatible with Mantle.
- Existing `crunch-store` PathInfo, castore, signature, NAR hash, store-prefix, export, and completeness checks remain authoritative.
- The existing foreign-import producer and planner remain separate. A later change can bind this generic closure result to a foreign-import receipt.

## Non-Goals

- Nix, flake, overlay, or nixpkgs evaluation.
- `snix-eval` integration.
- `nix-bindings-rust` integration.
- Private Cachix credentials or cache publication.
- Multi-root atomic publication.
- Local rebuild compatibility for imported Nix derivations.
- Package correctness, reproducibility, or release eligibility claims.

## Impact

- **Store core**: a new pure HTTP closure-planning module.
- **Store shell**: bounded narinfo discovery and strict planned-member admission.
- **CLI**: an explicit `--closure` mode for HTTP pull.
- **Evidence**: deterministic local fixtures and a bounded public-cache proof when network access is available.
