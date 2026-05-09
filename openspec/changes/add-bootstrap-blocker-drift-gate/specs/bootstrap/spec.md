## ADDED Requirements

### Requirement: Bootstrap blocker inventory is deterministic [r[bootstrap.blocker-inventory.deterministic]]

Crunch MUST provide a checked-in bootstrap blocker inventory gate that deterministically derives remaining full-source bootstrap blockers from repository-controlled sources.

The inventory MUST classify at least bridge outputs, placeholder or normalization-only providers, legacy-provider fallback, host-tool fallback, prerequisite-gated evidence, and known compiler/runtime crash boundaries. Each finding MUST include a stable marker class, source path, line or artifact locator when available, and a short explanation of the blocked promotion claim.

#### Scenario: Current gated tree produces an inventory [r[bootstrap.blocker-inventory.deterministic.current-tree]]

- GIVEN the current repository still contains bootstrap bridge and prerequisite-gated markers
- WHEN the blocker inventory gate runs in report mode
- THEN it exits successfully without claiming full-source readiness
- AND it emits JSON and Markdown summaries containing every configured marker class found in the tree
- AND the report identifies `bootstrap/seed-full.ncl` or its provider status as gated rather than promoted

#### Scenario: Inventory is stable for automation [r[bootstrap.blocker-inventory.deterministic.stable-output]]

- GIVEN no blocker-relevant source files changed
- WHEN the blocker inventory gate runs twice
- THEN the JSON report contains stable ordering for marker classes and findings
- AND the Markdown report is suitable for checked-in evidence without timestamps or host-specific paths

### Requirement: Full-source promotion claims fail closed while blockers remain [r[bootstrap.blocker-inventory.promotion-drift]]

Crunch MUST fail a bootstrap readiness or promotion check when repository-controlled status text, manifests, reports, or seed-provider metadata claim full-source bootstrap readiness while configured blocker markers remain present.

The failure MUST name the conflicting promotion claim and at least one remaining blocker class. It MUST NOT require running the heavyweight self-hosting proof to reject an inconsistent readiness claim.

#### Scenario: Promotion drift is rejected [r[bootstrap.blocker-inventory.promotion-drift.rejects-conflict]]

- GIVEN a fixture or mutation that marks the full-source provider as promoted
- AND a known bridge, placeholder, fallback, or prerequisite-gated blocker remains
- WHEN the blocker inventory gate runs in enforcement mode
- THEN it exits nonzero
- AND the diagnostic names the promotion claim and remaining blocker class

#### Scenario: Gated status remains allowed [r[bootstrap.blocker-inventory.promotion-drift.allows-gated-status]]

- GIVEN the source tree explicitly labels full-source bootstrap status as gated
- AND blocker markers remain present
- WHEN the blocker inventory gate runs in enforcement mode
- THEN it does not fail merely because blockers exist
- AND it records the blockers as readiness debt rather than promotion evidence

### Requirement: Blocker taxonomy has a retirement workflow [r[bootstrap.blocker-inventory.taxonomy-retirement]]

The blocker inventory gate MUST document how marker classes are added, updated, and retired when a blocker is repaired. Retiring a marker class MUST require positive evidence for the repaired boundary and a negative drift check proving that overclaiming still fails for any remaining classes.

#### Scenario: Repaired blocker can be retired with evidence [r[bootstrap.blocker-inventory.taxonomy-retirement.evidence]]

- GIVEN a bootstrap blocker has been repaired with source-built positive evidence
- WHEN its marker class is removed or narrowed
- THEN the change includes the repair evidence path
- AND the remaining inventory still runs deterministically
- AND the promotion-drift negative fixture still fails for any remaining blocker class
