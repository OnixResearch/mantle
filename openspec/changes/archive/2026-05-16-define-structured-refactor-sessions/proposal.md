## Why

Unison treats refactoring as a structured session over stable identities, not as a pile of text edits that leaves the codebase broken mid-way. Mantle has already gone through a Crunch-to-Mantle rename and will likely need more migrations: CLI aliases, store-prefix defaults, project file names, proof schema versions, and compatibility windows. These transitions should be encoded as explicit refactor/migration sessions with validation and rollback diagnostics.

## What Changes

- **Refactor session records**: Define a machine-readable record for name, prefix, file, CLI, and schema migrations.
- **Compatibility validation**: Check legacy and canonical surfaces together and report conflicts clearly.
- **Operator workflow**: Add plan/apply/check semantics for structured migrations without silent mixed identity.

## Capabilities

### New Capabilities
- `structured-refactor-sessions`: machine-readable migration/refactor workflows.

### Modified Capabilities
- `project-identity`: Crunch/Mantle compatibility becomes one instance of a general refactor session model.
- `project-management`: project migrations gain plan/apply/check semantics.

## Impact

- **Files**: project identity compatibility checks, migration planner, docs, tests.
- **APIs**: additive migration/refactor record schema.
- **Testing**: Crunch-to-Mantle fixture, conflicting-file negative tests, dry-run no-mutation tests.
