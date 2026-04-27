Task-ID: V4
Covers: Full-source bootstrap claim requires evidence, Legacy seed as development fast-path

# OpenSpec validation and gate evidence

## `openspec validate repair-live-bootstrap-archive-status`

Result:

```text
Change 'repair-live-bootstrap-archive-status' is valid
```

## `openspec_gate stage=proposal change=repair-live-bootstrap-archive-status`

Result: WARN, no blockers.

Key finding: successor-change tracking is represented conditionally in the delta
spec; design/tasks explicitly cover creating and verifying
`openspec/changes/live-bootstrap-source-chain/`.

## `openspec_gate stage=design change=repair-live-bootstrap-archive-status`

Result: PASS.

Key evidence: design covers status repair, `seed-legacy.ncl` restoration,
invalid completion evidence rejection, and implementation successor handoff.

## `openspec_gate stage=tasks change=repair-live-bootstrap-archive-status`

Result: PASS.

Key evidence: tasks trace legacy seed validation, invalid evidence rejection,
successor tracking, and final OpenSpec validation/gates. The first tasks gate run
reported V4 still open; this evidence file and the completed task line close it.
