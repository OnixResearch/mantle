# Validation plan

## Outcome checkpoint

- **Question:** Can the eight pipeline findings close without changing evaluation, build, root, cache, evidence, or key semantics?
- **Inspected evidence:** the focused finding transcript, 58 passing pre-change tests, relevant source paths, and the accepted build-layer boundary evidence.
- **Decision:** use bounded functional planning, existing bundle ownership, named private requests, and capability-aligned decomposition.
- **Owner:** `repair-crunch-pipeline-tigerstyle`.
- **Next action:** implement the bounded repair, run focused checks, and preserve the next exact repository blocker.

## Required evidence

- Focused and repository Tiger Style transcripts.
- Positive and negative pre-change and post-change package tests.
- Strict Clippy, formatting, caller, diff, and suppression checks.
- Local-builder and ordinary full flake-check transcripts.
- Cairn validation, Tracey coverage, lifecycle gates, and archive manifest.
