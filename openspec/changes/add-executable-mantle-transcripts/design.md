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

**Choice:** Use fenced blocks such as `mantle`, `mantle:error`, `expect`, `expect:json`, `setup:hide`, `cleanup:hide`, and `transcript:options`.

**Rationale:** This keeps examples readable in docs and structured enough for a runner. Visible command and expectation blocks remain documentation; hidden blocks allow fixture setup without teaching operators to run scaffolding commands.

**Implementation:** The normative format reference is `docs/mantle-transcripts.md`, with fast smoke fixtures under `tests/fixtures/transcripts/fast/`. The first runner should parse only these block types and reject unknown `mantle:*`, `expect:*`, or `*:hide` variants so typos do not silently become prose.

### 2. Ephemeral state by default

**Choice:** Each transcript runs with fresh temporary `--store` and `--state-dir` unless explicitly marked in-place with `transcript:options`.

**Rationale:** Reproducers should not mutate the operator's normal store.

**Implementation:** The runner provides `MANTLE_TRANSCRIPT_TMP`, rewrites Mantle command invocations with transcript-local `--store` and `--state-dir` unless already supplied by the block, and rejects `in_place: true` unless the caller passes an explicit in-place allow flag.

### 3. Expected-error semantics

**Choice:** `mantle:error` commands pass only when they exit non-zero and their following `expect` or `expect:json` checks match the normalized output.

**Rationale:** Expected failure examples should remain fail-closed: a command that unexpectedly starts succeeding is a behavior change, and a command that fails for the wrong reason is not a valid repro.

### 4. Output normalization

**Choice:** Match expectations against normalized output fragments rather than raw process bytes.

**Rationale:** Paths, temp directories, line endings, and blank-line churn are not the behavior under test.

**Implementation:** Normalize transcript temp paths to symbolic variables, normalize line endings to `\n`, strip trailing line whitespace, and collapse repeated blank lines before matching ordered fragments.

### 5. Output capture as evidence

**Choice:** Successful runs write a `.output` or receipt artifact that records commands, normalized outputs, and failures.

**Rationale:** Reviewers need durable evidence for docs and bug repros.

## Risks / Trade-offs

**Non-deterministic CLI output** → transcript expectations should support stable fragments or JSON field checks rather than raw full-output matching only.

**Long-running transcripts** → quality gates can separate fast transcript smoke from heavy release/bootstrap transcript suites.
