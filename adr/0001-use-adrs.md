# 0001 — Use ADRs to record major decisions

**Status:** accepted
**Date:** 2026-04-04

## Context

Decisions made during agent sessions get lost between sessions. The next
agent (or human) shows up, hits the same question, and either re-derives
the answer or picks a different one. AGENTS.md carries operational
knowledge (how to build, what breaks), but it's the wrong place for
"why did we choose X over Y" — those entries bloat the file and get
stale without clear ownership.

## Decision

Record major decisions as numbered ADRs in `adr/`. Reference the
directory from AGENTS.md so every agent session knows to check and
update it. "Major" means anything that constrains future work: tool
choices, architectural patterns, conventions, things we tried and
rejected.

## Consequences

- New file per decision, easy to review in PRs.
- `adr/README.md` has the index — keep it updated when adding entries.
- Superseded decisions stay in the directory (status changed, not deleted)
  so the reasoning history is preserved.
- Agents are expected to write ADRs during sessions, not retroactively.
