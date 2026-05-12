## Context

`crunch bootstrap validate` is now the shared evidence runner for bootstrap runtime-validation work, but its CLI regression coverage only proves preflight failure. A passing build path must prove that logs, summary JSON, Markdown evidence, and status rendering stay stable.

## Goals / Non-Goals

**Goals:**
- Create a bounded, inspectable change package for `add-bootstrap-validate-success-regression`.
- Keep verification local and deterministic where possible.
- Avoid broad behavior changes outside the stated files and capability.

**Non-Goals:**
- Do not claim full bootstrap completion, GCC correctness, or distributed provider readiness unless the task evidence proves it.
- Do not add optional provider/runtime dependencies by default.

## Decisions

### 1. Smallest verifiable slice first

**Choice:** Implement the smallest slice that satisfies the added requirement and produces durable evidence.

**Rationale:** The repo has many bootstrap-adjacent surfaces; small checks and receipts reduce repeated rediscovery.

**Alternative:** Batch multiple candidates into one implementation commit. Rejected because it would blur evidence and rollback boundaries.

**Implementation:** Follow `tasks.md` top to bottom and commit each completed task with its evidence.

## Risks / Trade-offs

**Scope creep** → Defer broader behavior changes to sub-changes rather than expanding this change.

**False confidence** → Prefer commands, logs, summaries, and artifact inspections over prose-only claims.

## Validation Plan

- `openspec validate add-bootstrap-validate-success-regression --strict` before implementation.
- Task-specific checks listed in `tasks.md`.
- `openspec validate --all --strict` before archive.
