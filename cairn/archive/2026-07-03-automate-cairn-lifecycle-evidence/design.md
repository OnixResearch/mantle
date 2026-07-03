## Context

Mantle already stores lifecycle evidence in archived change directories. The recurring risk is not missing commands conceptually; it is that a human can run commands in separate tool calls, forget to append output, archive before sync repair, or claim clean status from stale evidence.

The helper should be a thin imperative shell around explicit commands and a pure classifier that decides whether the lifecycle state is acceptable.

## Decisions

### 1. Command list is explicit input

**Choice:** The runner accepts or reads an explicit list of validation commands for a change rather than guessing tests from diffs.

**Rationale:** Evidence should name the command that proved the claim.

### 2. Lifecycle state checks are pure

**Choice:** Parse command results, tasks content, Cairn JSON, sync/archive plans, and git status into a deterministic summary using pure functions that are unit tested with fixtures.

**Rationale:** The decision logic should be testable without running Cairn or git.

### 3. Execution shell owns side effects

**Choice:** The shell runs commands, captures transcripts, appends evidence, invokes Cairn sync/archive, and checks worktree status.

**Rationale:** File mutation and process execution must stay outside core logic.

### 4. Accepted-spec sync repair is detected

**Choice:** After sync/archive, the runner verifies that new requirement IDs exist in accepted specs and fails if Cairn produced only a skeleton or dropped requirement text.

**Rationale:** Recent sync behavior required a manual repair; the automation should make that state visible.

## Risks / Trade-offs

- A runner can encode bad defaults if it hides command choices; keeping command lists explicit avoids this.
- Post-archive validation can still be expensive for large changes.
- The first implementation should prefer correctness over convenience flags.
