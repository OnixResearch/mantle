## Context

The README still carries assumptions from earlier milestones:

- `/nix/store` as the only logical store prefix
- an engine-only command set
- older build-environment requirements
- little to no documentation for signing and trust controls

Those mismatches are expensive because they affect the first-run experience.
A new reader will copy flags and defaults from the README before they inspect
`src/main.rs`.

## Goals / Non-Goals

**Goals:**
- Make the README match the current CLI and runtime model
- Explain the logical-vs-physical store split clearly
- Cover project-management and signature-verification workflows at a level
  useful to operators
- Remove stale setup requirements that the current tree no longer needs

**Non-Goals:**
- Writing a full manual for every subcommand
- Replacing `--help` output with prose
- Adding new CLI features in this change

## Decisions

### 1. Use Clap help and current code as the source of truth

**Choice:** Update the README from the current command definitions in
`src/main.rs` and related handlers, not from historical README phrasing.

**Rationale:** The help text and code reflect what actually ships.

### 2. Focus on top-level user workflows, not exhaustive flag dumps

**Choice:** Document the major workflows and the flags that change behavior:
store prefix selection, project-management commands, substitution/signing
controls, and store verification/signing commands.

**Rationale:** The README should orient readers. Exhaustive per-flag detail can
stay in `--help`.

### 3. Treat old environment notes as suspect until rechecked

**Choice:** Re-verify every setup requirement called out in the README.

**Rationale:** The tree has already moved past at least one stale claim
(`protoc`). A short re-audit is cheaper than shipping another round of drift.

## Risks / Trade-offs

**[README gets too long]** Adding every new command can bloat the top-level
document. Mitigation: keep the README focused on workflows and move repeated
flag detail to concise bullet lists.

**[Docs drift again]** A one-time edit can decay quickly. Mitigation: align the
README with stable command groupings and defaults that are easy to re-audit
against `--help`.
