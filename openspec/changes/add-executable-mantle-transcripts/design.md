## Context

Mantle needs evidence that examples actually run. Shell snippets alone are too informal; full integration tests are often too opaque for docs and issue reports. A transcript runner bridges those surfaces.

## Goals / Non-Goals

**Goals:**
- Define a simple markdown transcript format.
- Run transcripts in isolated temp stores/state by default.
- Support expected failures and hidden setup blocks.

**Non-Goals:**
- Replace Rust unit/integration tests.
- Execute untrusted transcripts without sandbox review.
- Require every README snippet to become a transcript immediately.

## Decisions

### 1. Markdown fenced block format

**Choice:** Use fenced blocks such as `mantle`, `mantle:error`, `expect`, and `setup:hide`.

**Rationale:** This keeps examples readable in docs and structured enough for a runner.

### 2. Ephemeral state by default

**Choice:** Each transcript runs with fresh temporary `--store` and `--state-dir` unless explicitly marked in-place.

**Rationale:** Reproducers should not mutate the operator's normal store.

### 3. Output capture as evidence

**Choice:** Successful runs write a `.output` or receipt artifact that records commands, normalized outputs, and failures.

**Rationale:** Reviewers need durable evidence for docs and bug repros.

## Risks / Trade-offs

**Non-deterministic CLI output** → transcript expectations should support stable fragments or JSON field checks rather than raw full-output matching only.

**Long-running transcripts** → quality gates can separate fast transcript smoke from heavy release/bootstrap transcript suites.
