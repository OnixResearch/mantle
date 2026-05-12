## ADDED Requirements

### Requirement: Bootstrap Drain State Hygiene [r[bootstrap-state-handoff-hygiene]]
Crunch MUST keep bootstrap drain handoff state consistent with live OpenSpec queue state so agents do not resume stale blockers as active work.

#### Scenario: Stale state is not presented as active [r[bootstrap-state-handoff-hygiene.1]]
- GIVEN there are no active OpenSpec changes
- WHEN a root drain-state file is present
- THEN it is either current, archived, or removed with context preserved elsewhere

#### Scenario: Cleanup is verified [r[bootstrap-state-handoff-hygiene.2]]
- GIVEN the handoff state is reconciled
- WHEN `openspec validate --all --strict` and git status are checked
- THEN the repository has no misleading active-drain artifact
