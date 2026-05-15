## ADDED Requirements

### Requirement: Project refactor sessions MUST support plan, check, and explicit apply phases [r[project-management.refactor-sessions]]

Mantle project-management commands for structured refactor sessions MUST support a no-mutate plan/check phase before any apply phase. The plan/check phase MUST report intended file, alias, store-prefix, and compatibility changes without modifying project files or store state. Apply MUST require explicit operator selection of the session to run.

#### Scenario: Refactor plan is side-effect free [r[project-management.refactor-sessions.plan]]

- GIVEN a project eligible for a structured refactor session
- WHEN the operator runs the no-mutate plan/check command
- THEN Mantle reports intended changes and conflicts
- AND it does not modify project files, lockfiles, generated directories, or store state

#### Scenario: Apply requires explicit session [r[project-management.refactor-sessions.apply-explicit]]

- GIVEN one or more structured refactor sessions are available
- WHEN the operator requests apply without naming a session
- THEN Mantle refuses to apply changes
- AND it lists the available sessions and their risk summaries
