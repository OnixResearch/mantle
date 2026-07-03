## ADDED Requirements

### Requirement: Mantle naming consistency

r[build_tool_boundary.mantle_naming_consistency] Mantle SHOULD use Mantle as the project-facing name in user-facing prose while preserving exact `crunch-*` identifiers only where those identifiers are still required by crates, paths, commands, compatibility surfaces, or archived evidence.

#### Scenario: stale user-facing prose is updated

GIVEN README, docs, examples, help text, or status output describe the project or product
WHEN naming drift checks inspect that prose
THEN the prose SHOULD use Mantle rather than Crunch.
AND any remaining Crunch wording MUST be tied to an exact required identifier or historical context.

#### Scenario: exact identifiers are not renamed accidentally

GIVEN a crate name, package name, path, binary-compatible command, compatibility API, or archived transcript still uses `crunch-*`
WHEN the naming cleanup runs
THEN Mantle MUST preserve that exact spelling unless a separate compatibility change owns the rename.
AND the naming guard MUST allow documented exact-identifier contexts.

#### Scenario: new docs cannot regress silently

GIVEN new user-facing docs or examples are added
WHEN naming drift validation runs
THEN stale Crunch project prose MUST be reported with a deterministic diagnostic.
AND accepted exceptions MUST name the file, context, and reason the exact identifier remains required.
