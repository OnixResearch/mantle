# Bootstrap blocker inventory signal evidence

Task-ID: V4
Covers: r[bootstrap_inventory.blocker_signal]

## Baseline

Before implementation, `./scripts/check-bootstrap-blocker-inventory.sh --self-test --report-only` exited with status 1 and wrote no report output through the wrapper in pueue task 1594. Default enforcement also exited with status 1.

## Implementation evidence

- `scripts/check-bootstrap-blocker-inventory.sh` now compiles the std-only Rust core directly with the selected nightly `rustc`, avoiding the cargo-script frontmatter runner failure.
- Report-only mode omits `--enforce` and exits successfully after valid report generation even when blockers remain.
- Default enforcement still passes `--enforce --require-clean` and fails when unsuppressed actionable findings remain.
- JSON schema version 2 adds `actionable_finding_count`, `metadata_suppression_count`, and `source_suppression_count`, plus dedicated `metadata_suppressions` and `source_suppressions` arrays. Markdown mirrors those sections.

## Focused validation transcript

Command run from `/home/brittonr/git/mantle` in pueue task 1685:

```text
== report-only self-test ==
bootstrap blocker inventory: 38 findings across 4 classes, 396 evidence-backed suppressions, 0 promotion claims, enforce=false
report-only status=0
== default enforcement ==
bootstrap blocker inventory: 38 findings across 4 classes, 396 evidence-backed suppressions, 0 promotion claims, enforce=true
FAIL: bootstrap blocker inventory is not clean; expected 0 findings and 0 promotion claims
enforcement status=1
== shell + diff checks ==
bash-n status=0
diff-check status=0
```

Full generated JSON was parsed with Steel after the run:

```text
json-ok
```

## Cairn lifecycle validation

Pueue task 1699 ran proposal/design/tasks gates plus validation for this change. The tasks gate ended with:

```text
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```

The same task's validation summary ended with:

```text
"change_issues": [],
"issues": [],
"spec_issues": [],
"valid": true
```

Pueue task 1706 executed `cairn sync tighten-bootstrap-blocker-inventory-signal --execute` with `"reasons": []`, then `cairn validate --root /home/brittonr/git/mantle` reported `"valid": true` and `"specs_validated": 17`.
