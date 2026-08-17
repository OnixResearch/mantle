# Release Provenance Specification

## Purpose

Adds capability-bounded local filesystem roots for Mantle release and build evidence shell boundaries.

## Requirements

### Requirement: Mantle release evidence filesystem authority is capability-rooted
r[mantle.release_provenance.cap_std_boundary.root_wrappers] Mantle MUST represent local release-evidence, witness-rebuild, bootstrap, build-artifact, and store filesystem authority as typed capability roots opened at the shell boundary.

#### Scenario: Valid relative path stays under the declared root
r[mantle.release_provenance.cap_std_boundary.tests.positive]
- GIVEN Mantle has opened an operator-declared evidence or artifact root
- WHEN release or build code reads or writes a valid relative path through the typed root
- THEN the operation MUST remain confined to that declared root and preserve existing valid-input behavior.

#### Scenario: Invalid path cannot escape the root
r[mantle.release_provenance.cap_std_boundary.tests.negative]
- GIVEN input names `../` traversal, an absolute path, a missing capability root, or a symlink escape attempt
- WHEN Mantle resolves the path through the typed root
- THEN Mantle MUST fail closed with deterministic diagnostics before reading or writing outside the declared root.

### Requirement: Capability dependency stays at shell boundaries
r[mantle.release_provenance.cap_std_boundary.dependency] Mantle MUST add `cap-std` only to crates or modules that own filesystem shell/adaptor behavior.

#### Scenario: Pure planning cores stay dependency-free
r[mantle.release_provenance.cap_std_boundary.conversion]
- GIVEN Mantle pure planning, provenance, and build decision cores are reviewed
- WHEN cap-std adoption is complete
- THEN those cores MUST receive in-memory data or validated relative locators and MUST NOT depend on `cap-std` or open ambient filesystem paths directly.

### Requirement: Capability boundary is documented without release overclaims
r[mantle.release_provenance.cap_std_boundary.docs] Mantle MUST document that capability roots bound local filesystem authority only and do not prove artifact truth, build correctness, external-evidence semantics, or release eligibility.

#### Scenario: Operator docs state non-claims
r[mantle.release_provenance.cap_std_boundary.validation]
- GIVEN the cap-std release-boundary change is ready to archive
- WHEN focused tests, release-evidence checks, build checks, Cairn validation, and Cairn gates run
- THEN the evidence MUST include positive and negative filesystem-boundary coverage and visible non-claim documentation.
