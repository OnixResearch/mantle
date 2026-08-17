## Why

Mantle already exposes `--strict-hermetic`, but release, deterministic, self-hosting, and witness proof flows can still overclaim if practical or impure evidence is accepted without a hard admission boundary. Proof evidence should be eligible only when the selected execution mode, audit events, closure facts, and host-tool controls satisfy the stronger hermetic contract.

## What Changes

- Add a proof-eligibility gate that admits proof evidence only from strict hermetic runs.
- Classify practical, impure, fallback, missing-closure, protected-environment, and undeclared-host-tool outcomes as proof blockers.
- Surface a deterministic proof-mode decision in release, deterministic-release, self-hosting, and witness verification reports.
- Keep practical mode useful for local diagnostics while preventing it from satisfying proof claims.

## Impact

- **Files**: proof eligibility core, release/self-build/witness CLI shells, JSON/human report rendering, docs, and Cairn verification-evidence spec delta.
- **Testing**: positive strict proof eligibility fixture; negative practical, impure, and degraded hermeticity fixtures; Cairn validation and gates.

## Out of Scope

- Removing practical mode for local development.
- Proving compiler correctness or full-source bootstrap completeness.
