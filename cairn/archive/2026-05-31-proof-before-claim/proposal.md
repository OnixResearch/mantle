# Proposal: Require proof before Mantle claims

## Why

Mantle status, completion, validation, and feature-support claims need the same evidence discipline that Cairn tasks and gates are meant to provide. Prompt-only guidance is too weak: it can be forgotten, and it does not give reviewers a repo-local requirement to cite.

## What Changes

- Add a Cairn `verification-evidence` capability requiring proof before claims.
- Require status, completion, feature-support, pass/fail, and build-success claims to cite current evidence.
- Require unproven status to say "not proven" or narrow the claim to what evidence supports.
- Require completed Cairn tasks to reference durable evidence or oracle checkpoints that support the exact task text.

## Impact

- Reviewers get a canonical requirement for rejecting overbroad claims.
- Future Mantle changes can cite `r[verification_evidence.proof_before_claim]` when adding evidence or task-completion rails.
- This complements `AGENTS.md`; Cairn is the durable project law, while `AGENTS.md` is agent behavior guidance.
