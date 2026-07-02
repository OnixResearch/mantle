## ADDED Requirements

### Requirement: Bootstrap blocker inventory signal

r[bootstrap_inventory.blocker_signal] Mantle MUST distinguish successful bootstrap blocker inventory report generation from clean-baseline enforcement failure, while preserving fail-closed promotion-claim and unsuppressed-blocker detection.

#### Scenario: report-only inventory succeeds with remaining blockers

GIVEN bootstrap-critical sources still contain known blocker markers
WHEN an operator runs the blocker inventory in report-only mode
THEN Mantle MUST write valid JSON and Markdown reports and exit successfully if report generation succeeds
AND the report MUST state that blockers remain without presenting the repository as clean.

#### Scenario: enforcement fails on blockers or promotion claims

GIVEN unsuppressed bootstrap blockers or bootstrap promotion claims are present
WHEN the inventory runs in enforcement mode
THEN Mantle MUST fail closed with a deterministic diagnostic
AND it MUST preserve enough report output for the operator to identify the blocker class and source.

#### Scenario: checked metadata is separated from actionable findings

GIVEN checked evidence metadata contains bridge, placeholder, or frontier wording that is covered by a durable suppression reason
WHEN the inventory report is rendered
THEN Mantle MUST keep that metadata auditable in a separate suppressed or informational section
AND it MUST NOT let metadata-only records obscure the primary actionable source-finding list.

#### Scenario: suppression does not hide live source blockers

GIVEN a bootstrap source file contains an unsuppressed blocker marker with wording similar to a suppressed evidence metadata record
WHEN the inventory classifies findings
THEN the source marker MUST remain counted as actionable
AND the report MUST NOT classify it as metadata-only without an explicit evidence-backed suppression.
