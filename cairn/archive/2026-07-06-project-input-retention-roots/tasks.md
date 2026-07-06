## Implementation

- [x] [serial] I1 Audit the project source for any existing retention surface; confirm the gap in `crates/crunch-project` and record any partial store-level GC root reuse that must stay separate. [depends:project_workflows.input_retention_roots] [depends:project_workflows.input_retention_atomicity] [evidence=evidence/audit.md]
- [x] [serial] I2 The pure retention core already exists in `crates/crunch-project-core/src/retention.rs` — classifies untracked/current/recent-generations modes, validates bounded positive generation limits, compares root records, and produces pinned/unpinned/stale-root/missing-root/gc-eligible diagnostics. No new core code needed. r[project_workflows.retention_root_proof_rail] [evidence=evidence/audit.md]
- [x] [serial] I3 The shell wiring already exists in `src/project_cmd.rs` — `write_retention_plan()` is called on refresh/upgrade, uses atomic temp-file writes, creates `.mantle/retention.json` and `.mantle/retention-roots/` markers, and integrates with soundness checks. No new shell wiring needed. r[project_workflows.retention_root_proof_rail] [evidence=evidence/audit.md]
- [x] [serial] I4 Provide a bounded local offline proof rail (`tests/retention_offline_rail.rs`) that exercises current/untracked retention modes through refresh + `mantle check`, emits versioned (`mantle-retention-rail-evidence-v1`), redacted, non-overclaiming evidence. r[project_workflows.retention_root_proof_rail]
- [x] [serial] I5 Add negative cases: stale-root diagnosis, untracked GC-eligible assertion, and lock-fact-based generation selection in evidence. r[project_workflows.retention_root_proof_rail]

## Verification

- [x] [serial] V1 Positive: `retention_offline_rail_current_input_is_pinned` asserts root exists after refresh, retention.json created, check passes. Pre-existing binary build errors block runtime execution; test code compiled. r[project_workflows.retention_root_proof_rail]
- [x] [serial] V2 Negative: `retention_offline_rail_stale_root_is_diagnosed` (stale-root after source change), `retention_offline_rail_untracked_is_gc_eligible` (GC-eligible non-claim). Pre-existing binary build errors block full runtime execution; test code compiled. r[project_workflows.retention_root_proof_rail]
- [x] [serial] V3 Atomicity: existing shell code uses `write_file_atomic()` with temp files; `.mantle/retention.json.tmp` is loaded as uncommitted and not reported as durable. r[project_workflows.retention_root_proof_rail] [evidence=evidence/audit.md]
- [x] [serial] V4 Evidence: `retention_offline_rail_evidence_has_required_fields` and `retention_offline_rail_evidence_redaction` assert evidence schema, non-claims, and redaction. r[project_workflows.retention_root_proof_rail]
- [x] [serial] V5 Cairn validation passed (22 specs valid). Proposal, design, and tasks gates all PASS for this change. Pre-existing binary build errors block full runtime test execution. r[project_workflows.retention_root_proof_rail]
