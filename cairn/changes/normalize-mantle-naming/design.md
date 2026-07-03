## Context

The project was formerly Crunch and still has crate/package identifiers that use that spelling. A blind rename would break paths, public compatibility surfaces, and archived evidence. A no-op leaves confusing public prose.

## Decisions

### 1. User-facing prose prefers Mantle

**Choice:** README/docs/examples/status wording should say Mantle when referring to the project or product.

**Rationale:** New users should see one product name.

### 2. Exact identifiers are preserved

**Choice:** Keep `crunch-*` only when naming exact crate/package identifiers, paths, binaries, commands, archive text, or compatibility surfaces that still require those names.

**Rationale:** Naming cleanup must not become a breaking rename.

### 3. The guard is allowlist-based

**Choice:** Add a deterministic check with explicit allowed contexts rather than a broad grep-only ban.

**Rationale:** The repo still intentionally contains Crunch identifiers, so the check needs to distinguish prose drift from required exact spelling.

## Risks / Trade-offs

- Over-tightening the guard can create churn in historical archive or compatibility code.
- Under-tightening leaves stale public wording in new docs.
- Some docs may need paired edits to avoid changing command examples that still require old binary names.
