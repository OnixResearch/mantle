# Change: Add promoted source-built proof checkpoints

## Why

Mantle already has receipt-validated dev provider reuse and a persistent dev store. The promoted source-built proof intentionally ignores that state.

This cold-only rule makes every retry rebuild StageX, the native provider, and the Rust provider. Receipt or later-stage fixes do not change those stage inputs, but they still cost hours of repeated work.

## What Changes

- Add stage-specific checkpoint identities for the first four provider stages.
- Publish a provider-closure checkpoint immediately after the Rust provider and toolchain closure complete.
- Store complete provider outputs, StageX evidence, native action plan and reconciliation, receipts, and transcripts under one immutable checkpoint manifest.
- Admit only checkpoints produced by promoted execution with exact source, policy, predecessor-output, resource, payload, execution-evidence, and action-trust bindings.
- Restore an admitted checkpoint into a fresh proof root and continue with Mantle stage1.
- Record restored and executed stages separately in the final receipt.
- Keep the cold path as the default when no checkpoint store is selected.

## Dependencies

- `dev-cache-source-built-fixed-point` supplies the existing receipt-validated cache and store patterns.
- `add-dev-cache-cross-run-resume` supplies the fresh-directory restore direction.
- `prove-source-built-mantle-fixed-point` supplies the six-stage plan and v2 receipt contract.
- ADR 0024 supplies the separation between content presence, result discovery, admission, and execution.

## Non-Goals

- Treating content presence or a store path as proof authority.
- Admitting dev checkpoints into a promoted proof.
- Claiming that a restored stage executed in the current attempt.
- Reusing a checkpoint after any relevant source, policy, predecessor output, resource, payload, or evidence identity changes.
- Proving compiler correctness or broad reproducibility.
