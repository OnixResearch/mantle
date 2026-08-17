## Why

Mantle needs one crisp operator story: a copied source tree plus declared source-root inputs can produce a Cargo-free fixed-point proof bundle, while Cargo, Nix, rustup, and ambient wrappers are guarded. That demo should be audit-grade, not a chat summary or a loose collection of logs. Claims like "Nix-free" or "beats Nix on this proof" must be backed by a bundle that includes positive proof evidence, negative guard evidence, and explicit non-claims.

## What Changes

- Define a demo-grade proof bundle profile for source-root Cargo-free fixed-point runs.
- Emit a concise operator README plus machine summary that names commands, digests, guard results, fixed-point verdict, replay hints, and non-claims.
- Require positive evidence for matching stage digests and negative evidence for Cargo/Nix/rustup/ambient-wrapper denial before any Nix-free demo claim is surfaced.
- Keep generated heavy artifacts under the output bundle while committing only lifecycle transcripts and digest summaries.

## Impact

- **Files**: proof-bundle summary/report code, docs, release/readiness summaries, CLI tests, Cairn verification-evidence spec delta.
- **Testing**: positive demo bundle validation, negative missing-proof-material validation, negative guard-failure bundle validation, docs/readiness checks, Cairn validation/gates.
