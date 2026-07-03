## Why

Recent Mantle work repeats the same manual lifecycle sequence: focused validation, formatting checks, Cairn validate/gates, evidence append, sync, archive, post-archive validation, and status capture. Manual repetition risks missing a post-archive transcript, accidentally relying on hidden output, or losing full-spec sync repairs.

Mantle needs a small deterministic helper that automates the evidence workflow while keeping each command explicit and reviewable.

## What Changes

- Add a repo-owned lifecycle evidence runner for one active Cairn change.
- Require the runner to execute configured validation commands, capture stdout/stderr, append bounded evidence, run Cairn validate/gates, sync/archive, and rerun post-archive validation.
- Fail closed when tasks are unchecked, evidence files are missing, gates fail, sync drops requirement text, or the worktree has unexpected changes.
- Emit a deterministic receipt/summary that can be committed with the archived change.

## Impact

- **Files**: scripts/tooling, docs for maintainers, tests with fake Cairn command fixtures, and evidence examples.
- **Testing**: positive dry-run/execute fixture, negative missing evidence/gate failure/skeleton accepted-spec cases.

## Out of Scope

- Replacing Cairn itself.
- Automatically choosing which implementation tests prove a change.
- Pushing commits or opening pull requests.
