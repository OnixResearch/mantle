# Tasks: Close the spec override tree as rejected at admission

The original implementation task list remains unchecked in
`evidence/original-scope/tasks.md`. No original implementation task was
performed.

## Phase 1: Admission decision

- [x] [serial] T1.1 Preserve the original proposal, design, tasks, metadata, and spec delta verbatim from published main `e24bbbc2f803` and record their blob identities in `evidence/original-scope/SOURCE.md`. a[spec-override-tree.original-scope]
- [x] [serial] T1.2 Run the admission review against the workspace foundation rule across Mantle and the sibling repositories; record the commands, revisions, and results in `evidence/admission-review-2026-10-07.md`. a[spec-override-tree.admission-review]
- [x] [serial] T1.3 Record the rejection, rejected alternatives, revisit triggers, and non-claims in ADR 0109 and its `adr/README.md` index row. a[spec-override-tree.decision-record]

## Phase 2: Lifecycle closure

- [x] [serial] T2.1 Remove the spec delta, select the `no-spec-delta` profile with a spec-effect rationale, and pass validation, the proposal, design, and tasks gates, and the no-op sync. a[spec-override-tree.no-accepted-requirement]
- [x] [serial] T2.2 Write `evidence/closure-2026-10-08.md` stating the change is closed as rejected at admission and not implemented, and confirm the archive preflight reports no blockers. a[spec-override-tree.closure]
