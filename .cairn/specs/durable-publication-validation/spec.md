# Durable Publication Validation Specification

## Purpose

Defines the `durable-publication-validation` capability.

## Requirements

### Requirement: Complete checked source closure

r[mantle.durable_publication_validation.source_closure]

Mantle's filtered Nix source MUST include every repository-level content-bound requirement fixture required by compiled tests and checked evidence inputs. It MUST NOT widen to unrelated secrets, build outputs, VCS metadata, or ambient files.

#### Scenario: Required fixture is present

- GIVEN the filtered source used by the Nix package
- WHEN content-bound requirement tests resolve their checked fixtures
- THEN every required fixture MUST be present.

#### Scenario: Fixture root is omitted

- GIVEN a negative source-filter fixture without the required root
- WHEN the Nix-built tests run
- THEN the build MUST fail with a bounded missing-input diagnostic.

### Requirement: Truthful bootstrap inventory

r[mantle.durable_publication_validation.inventory]

Bootstrap inventory suppressions MUST bind exact reviewed context and reason. Live blockers and promotion claims MUST remain unsuppressed and fatal in enforcement mode.

#### Scenario: Reviewed metadata is classified

- GIVEN lifecycle or evidence text that does not describe a live blocker
- WHEN inventory classification runs
- THEN it MAY be classified only with exact evidence.

#### Scenario: Live blocker or promotion claim appears

- GIVEN an unsolved source blocker or promotion claim
- WHEN enforcement runs
- THEN the inventory MUST fail.

### Requirement: Explicit validation-tool ownership

r[mantle.durable_publication_validation.tools]

Mantle-owned Clippy MUST remain warning-free with dependencies excluded. Vendored dependency failures MUST remain visible in a separate audit. The accepted Octet owner check MUST complete without the former invalid parent-traversal diagnostic and without a checker bypass.

#### Scenario: Product-owned lint passes

- GIVEN Mantle-owned targets and strict lint settings
- WHEN Clippy runs with dependency lint excluded
- THEN no Mantle-owned warning MAY remain.

#### Scenario: Vendored or Octet issue is hidden

- GIVEN a vendored warning or invalid diagnostic path
- WHEN validation evidence is generated
- THEN the issue MUST remain recorded until its owner-side resolution is accepted.

### Requirement: Broad validation continuation

r[mantle.durable_publication_validation.broad_rail]

The workflow MUST run focused repairs, workspace tests, policy checks, and the broad flake rail in order. It MUST record the next exact independent blocker when a later rail fails.

#### Scenario: All broad rails pass

- GIVEN complete source closure and resolved tool boundaries
- WHEN the full validation sequence runs
- THEN evidence MUST record successful results from one snapshot.

#### Scenario: A later rail fails

- GIVEN earlier focused checks pass and a later check fails
- WHEN the sequence completes
- THEN broad success MUST NOT be claimed.

### Requirement: Bounded broad-validation evidence

r[mantle.durable_publication_validation.evidence]

Evidence MUST bind source-closure, inventory, product lint, dependency audit, Octet, workspace-test, broad-flake, Cairn, and traceability results with explicit non-claims.

#### Scenario: Complete evidence passes

- GIVEN matching results and no hidden blocker
- WHEN evidence is reviewed
- THEN the change MAY synchronize and archive.

#### Scenario: Evidence merges unlike snapshots

- GIVEN results from different source snapshots or omitted failures
- WHEN evidence is reviewed
- THEN the change MUST remain incomplete.
