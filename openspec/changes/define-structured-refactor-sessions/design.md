## Context

Current project-identity specs define the canonical Mantle name and explicit Crunch compatibility. This change generalizes that lesson so future migrations are recorded, planned, checked, and applied through a repeatable workflow.

## Goals / Non-Goals

**Goals:**
- Define a structured refactor session record.
- Support dry-run planning and conflict diagnostics.
- Preserve legacy compatibility as explicit policy, not accidental mixed naming.

**Non-Goals:**
- Build a full semantic source-code refactoring engine.
- Rewrite historical archives.
- Remove existing Crunch compatibility unless a separate change says so.

## Decisions

### 1. Refactor sessions are records, not scripts only

**Choice:** Store migration/refactor intent in a machine-readable record naming old aliases, new aliases, affected surfaces, compatibility policy, and validation checks.

**Rationale:** Scripts alone do not make policy reviewable or queryable.

### 2. Plan/apply/check split

**Choice:** Provide a no-mutate plan/check path and a separate apply path.

**Rationale:** Operators need to understand impacts before touching project files or stores.

### 3. Conflicts fail clearly

**Choice:** Mixed canonical/legacy files or ambiguous aliases fail with a typed diagnostic and remediation.

**Rationale:** Silent mixed identity caused by partial renames weakens operator trust.

## Risks / Trade-offs

**Too broad for first implementation** → start with the Crunch-to-Mantle compatibility fixture and one generic schema version.

**Migration side effects** → keep dry-run as the first verified behavior and require explicit apply.
