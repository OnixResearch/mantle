# Specification: Tiger Style hardening backlog

## ADDED Requirements

### Requirement: The backlog is scanned one package at a time

r[mantle.tigerstyle_backlog.scan] The Tiger Style backlog scan MUST run the
lint once per package in `workspace.metadata.tigerstyle.default_scope`, so a
package that fails under lint does not hide the findings of the packages
scanned after it. A package whose library fails type checking under lint MUST
be rescanned after its repairs until it reports no further finding.

#### Scenario: An early package fails under lint

- GIVEN the first scoped package fails to compile under lint
- WHEN the backlog scan runs
- THEN the findings of every later scoped package MUST still be recorded

### Requirement: Mechanical findings are fixed in the source

r[mantle.tigerstyle_backlog.fix] A mechanical Tiger Style finding MUST be
removed from the source. The fix MUST keep the accepted and rejected inputs,
the error codes and error text, the published public APIs, the serde shapes
and the requirement implementation and verification markers of the affected
code.

#### Scenario: A compound condition is decomposed

- GIVEN a validator rejects an input through a compound condition
- WHEN the condition is decomposed into named predicates
- THEN the same input MUST be rejected with the same error text

#### Scenario: A recursive walk becomes iterative

- GIVEN a tree walk whose recursion depth is bounded only by the tree depth
- WHEN it becomes an explicit stack
- THEN the measured entries and the resulting digest MUST be unchanged

### Requirement: Deferred findings are recorded, not suppressed

r[mantle.tigerstyle_backlog.record] A finding that cannot be removed without
changing observable behaviour, changing a published public API, or
restructuring code outside its package MUST be recorded as a file-scoped
`#![allow(tigerstyle::...)]` scoped to exactly the categories present in that
file, with a `HARDENING-BACKLOG` comment naming the change's evidence log.
`dylint.toml` MUST NOT change, no non-`tigerstyle` allow MAY be added, and no
backlog block recorded before the change MAY gain a category.

#### Scenario: A published DTO keeps an optional serde default

- GIVEN removing `#[serde(default)]` would reject documents that omit the
  optional field
- WHEN the finding is handled
- THEN it MUST be recorded behind a file-scoped allow with its reason
- AND the accepted documents MUST be unchanged

### Requirement: The backlog evidence names every deferred finding

r[mantle.tigerstyle_backlog.evidence] The change MUST retain a deferred-finding
log with one row per recorded file, naming the file, the lint categories, the
finding and the reason it was deferred, and MUST retain the transcript of the
Tiger Style gate passing over the declared scope.

#### Scenario: A reviewer audits a backlog allow

- GIVEN a source file carrying a `HARDENING-BACKLOG` block of this change
- WHEN the reviewer reads the deferred-finding log
- THEN the log MUST contain a row for that file with the same categories

### Requirement: The gates are run and their output retained

r[mantle.tigerstyle_backlog.verify] The change MUST run the Tiger Style check
over the declared scope, the first-party strict Clippy gate, the workspace
library and integration tests and the format checks of the affected packages,
and MUST retain their exact output. A green Tiger Style gate MUST NOT be
reported as a correctness, release or adapter claim.

#### Scenario: The gate passes

- GIVEN every scoped package compiles under lint with no finding
- WHEN the evidence is recorded
- THEN it MUST name the revision and the exact command and exit status
- AND it MUST NOT claim that the deferred findings are fixed
