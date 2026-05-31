# Design: Proof-before-claim evidence rail

## Context

Cairn exists to make project claims traceable to requirements, tasks, and evidence. Mantle already uses Cairn tasks and evidence files, but the general rule was not written as an accepted Mantle requirement. That gap allowed agent status replies or completion summaries to sound stronger than the evidence that was actually inspected.

## Decision

Create a dedicated `verification-evidence` spec with one root requirement: no proof, no claim. The requirement covers human-facing summaries, evidence files, commit messages, task completions, and review/gate summaries.

## Enforcement model

- Immediate enforcement is review/gate policy: reviewers can reject claims that lack current evidence or exceed evidence scope.
- Cairn tasks must reference durable evidence or oracle checkpoints before being checked complete.
- Prompt guidance remains in `AGENTS.md`, but the authoritative policy lives in Cairn.

## Non-goals

- This change does not add a natural-language checker that proves every sentence in a final response.
- This change does not require every exploratory note to carry evidence; it applies when making status, completion, validation, feature-support, or build-success claims.
