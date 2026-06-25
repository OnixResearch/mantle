# Design: record deterministic release evidence

## Context

Mantle's verification evidence policy requires current evidence before status, completion, or feature-support claims. The deterministic release proof has current command output, but the proof receipts themselves are generated artifacts. A durable evidence transcript is the right checked-in surface because it records the proof without committing multi-gigabyte generated bundles.

## Decisions

### 1. Store a concise transcript, not proof payloads

The repository should track a Markdown evidence file with command lines, selected output snippets, digests, and bounded interpretation. The transcript may reference local generated paths as evidence locations, but it must not copy `target/` payloads into Git.

### 2. Record both positive proof and claim bounds

The transcript should include the successful `release reproduce`, successful required `release verify`, and successful receipt checker output. It should also state that the claim is bounded to packaged artifact determinism and provider fixed-point release-adjacent evidence.

### 3. Keep validation evidence attached to lifecycle state

The evidence transcript should live under Cairn lifecycle evidence so future status or release-readiness summaries can cite a tracked file. The final validation step should include `cairn validate --root .` and tracked status evidence.

## Validation

Validation is complete when the transcript cites current command output for the deterministic release proof, the required verifier, the receipt checker, Cairn validation, and tracked status. No generated `target/` proof payload should be staged.
