# Change: Add cross-run resume and runtime confirmation for the dev cache

## Why

The dev-cache foundation can adopt verified provider outputs and reuse persistent store state. It cannot resume every completed stage from a fresh staging directory.

The current evidence also lacks a complete cold-to-cached-to-adopt runtime cycle and a fresh promoted cold-path comparison. These stronger facts need separate implementation and long-run evidence.

## What Changes

- Persist a content-addressed resume bundle for each completed stage, including the transition execution tree and required stage outputs.
- Revalidate source, plan, policy, stage, output, and producer identities before restored state can skip work.
- Restore validated state into a fresh staging directory and continue at the first incomplete stage.
- Prove a complete cold-to-cached-to-adopt dev cycle with dev-only reports and no promoted aliases.
- Confirm that a fresh promoted run remains cache-disabled and starts from empty authority.

## Dependencies

- `dev-cache-source-built-fixed-point` supplies the admitted cache foundation.
- `prove-source-built-mantle-fixed-point` supplies the current cold proof graph and runtime prerequisites.

## Non-Goals

- Using resumed or cached state to satisfy a promoted proof.
- Treating a restored marker as execution evidence for skipped work.
- Weakening source, policy, protected-execution, or effect authority.
- Proving compiler correctness or broad reproducibility.
