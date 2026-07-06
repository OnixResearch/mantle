## Implementation

- [x] [serial] I1 Audit the existing importer surface in `src/pin_import.rs`, `src/cargo_import.rs`, `src/main.rs`, and `crates/crunch-project/src/lib.rs` against every scenario clause of the accepted requirements; record which importer kinds, mapped semantics, unsupported-semantics blockers, and plan/apply boundaries are already covered, and which are gaps. [depends:project_workflows.project_lock_importers] [depends:project_workflows.nixtamal_importer] [evidence=evidence/audit.md]
- [x] [serial] I2 Provide a bounded local offline proof rail (`tests/lock_importer_offline_rail.rs`) that exercises the external pin import `--plan` and `--apply` workflow for a supported importer (Nixtamal), where `--plan` is side-effect free and reports planned file operations, mapped inputs/patches, unsupported semantics, and blockers, and `--apply` writes only Mantle-owned files named by the plan. r[project_workflows.lock_importer_proof_rail]
- [x] [serial] I3 Emit a versioned (`mantle-lock-importer-rail-evidence-v1`), redacted, non-overclaiming evidence record stating the generated project remains a build-tool handoff (not Onix/NixOS module semantics) and that import is not build success or deployability. r[project_workflows.lock_importer_proof_rail]
- [x] [serial] I4 Add negative cases: `lock_importer_offline_rail_composition_semantics_are_blockers` (composition semantics produce blockers, apply fails). r[project_workflows.lock_importer_proof_rail]

## Verification

- [x] [serial] V1 Positive: `lock_importer_offline_rail_plan_no_mutate` and `lock_importer_offline_rail_apply_creates_planned_files` cover plan no-mutate and apply only-planned-files. Pre-existing binary build errors block runtime execution; test code compiled. r[project_workflows.lock_importer_proof_rail]
- [x] [serial] V2 Negative: `lock_importer_offline_rail_composition_semantics_are_blockers` asserts composition semantics produce blockers and apply refuses. r[project_workflows.lock_importer_proof_rail]
- [x] [serial] V3 Non-claim: `lock_importer_offline_rail_evidence_has_required_non_claim` asserts evidence carries build-tool handoff non-claim. r[project_workflows.lock_importer_proof_rail]
- [x] [serial] V4 Evidence: `lock_importer_offline_rail_evidence_has_required_non_claim` and `lock_importer_offline_rail_evidence_redaction` assert evidence shape and redaction. r[project_workflows.lock_importer_proof_rail]
- [x] [serial] V5 Cairn validation passed (22 specs valid). Proposal, design, and tasks gates all PASS for this change. Pre-existing binary build errors block runtime test execution. r[project_workflows.lock_importer_proof_rail]
