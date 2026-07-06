# Audit: Project Input Retention Roots Surface

Audited against `r[project_workflows.input_retention_roots]` and
`r[project_workflows.input_retention_atomicity]` requirement clauses, and the
change's own `r[project_workflows.retention_root_proof_rail]` requirement.

## Source Files Audited

- `crates/crunch-project-core/src/retention.rs` (400 lines)
- `crates/crunch-project-core/src/soundness.rs` (for retention diagnostics)
- `crates/crunch-project-core/src/lib.rs` (retention exports)
- `src/project_cmd.rs` (retention root diagnostic rendering)
- `tests/project_refresh_cli.rs` (no retention-specific tests)

## Core Retention Implementation

### `plan_retention_roots()` — Pure Core
- ✅ Receives `RetentionPlanRequest` with lock version info, manifest inputs,
  current lock entries, existing retention state.
- ✅ Produces `RetentionPlan` with `actions` vector.
- ✅ Classifies inputs into `untracked`, `current`, `recent-generations` modes.

### Retention Modes
- ✅ `InputRetentionPolicy::Untracked` — no root planned.
- ✅ `InputRetentionPolicy::Current` — root planned for current lock digest.
- ✅ `InputRetentionPolicy::RecentGenerations { generations }` — retains up to
  the configured bounded positive integer.

### Generation Limits
- ✅ `MAX_RETENTION_GENERATIONS` constant.
- ✅ `MAX_RETENTION_ROOT_RECORDS` constant.
- ✅ Generation selection based on lock/source digest and version, not filesystem
  timestamps.
- ✅ Bounded positive integer validation when parsing.

### Root Record Structure
- `RetentionRootRecord` with `input_name`, `lock_digest`, `source_identity`,
  `content_digest`, `kind`, `version`, `digest` fields.
- `RetentionRootKind::SourcePin` — standard source material.
- `RetentionRootFact` for evidence.

### Diagnostic Classification
- `RetentionInputState` enum: `Pinned`, `Unpinned`, `StaleRoot`, `MissingRoot`,
  `GarbageCollectionEligible`.
- `RetentionDiagnosticKind` with cases for each classification.
- `locked_source_identity()` and `lock_entry_digest()` helpers.

## Shell Wiring Gaps

- ⚠️ No `.mantle/retention.json` materialization in `src/project_cmd.rs` or
  `src/project_resolve.rs`. The core plan is computed but never persisted.
- ⚠️ No `.mantle/retention-roots/` directory creation or marker file writes.
- ✅ `cmd_check` in `project_cmd.rs` renders retention state from the core
  plan — it calls retention planning but reads the current on-disk state
  from nonexistent files.
- ✅ Retention-related error handling exists (`removing stale retention root`)
  but only triggers when stale root files happen to exist on disk.
- ⚠️ No atomic temp-file commit pattern for retention root updates.

## Soundness Integration
- ⚠️ `check_project_soundness()` does not call `plan_retention_roots()` or emit
  retention-root diagnostics.
- ✅ Soundness non-claims would cover retention if wired.

## Negative Cases

| Case | Code | Test |
|---|---|---|
| Interrupted root update not durable | ❌ | ❌ |
| Stale-root diagnosed | ⚠️ Core classification exists | ❌ |
| Missing-root diagnosed | ⚠️ Core classification exists | ❌ |
| Untracked = GC-eligible | ⚠️ Core classification exists | ❌ |
| Generation based on lock facts, not timestamps | ✅ Core | ❌ |
| Atomic temp-file commit | ❌ | ❌ |

## Summary

| Feature | Core Logic | Shell Wiring | Test |
|---|---|---|---|
| Current-input pinned | ✅ | ❌ | ❌ |
| Generations bounded | ✅ | ❌ | ❌ |
| Untracked = GC-eligible | ✅ | ❌ | ❌ |
| Stale/missing diagnosis | ✅ | ❌ | ❌ |
| Atomic persistence | ❌ | ❌ | ❌ |
| Soundness integration | ❌ | ❌ | ❌ |
| Evidence record | ❌ | ❌ | ❌ |

**The retention core already exists in `crunch-project-core/src/retention.rs`.
The main gaps are:**
1. Shell wiring: materialize retention state into `.mantle/retention.json` and
   `.mantle/retention-roots/` with atomic temp-file commits
2. Soundness integration: call `plan_retention_roots()` during `mantle check`
3. Proof rail: composed fixture exercising all three modes
4. Evidence record: versioned, redacted, non-overclaiming