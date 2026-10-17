# Specification: Spec override tree

## ADDED Requirements

### Requirement: Checked override verbs

r[mantle.spec_override_tree.verb_semantics] Overrides MUST use a fixed verb
set over package specs: `set` assigns any field, `append`, `prepend`, and
`remove` operate on list fields, `merge` operates on record fields, and
`edit` maps a spec to a spec for one package.

A verb applied to a field of the wrong shape MUST fail with the package,
field, verb, and expected shape named. An edited spec MUST pass the same
validation as a hand-written spec before it produces a derivation.

#### Scenario: List append

- GIVEN a package with a list field and an override tree appending one entry
- WHEN the tree is applied
- THEN the resulting spec MUST contain the original entries plus the appended
  entry in order.

#### Scenario: Append to a non-list

- GIVEN an override tree applying `append` to a string field
- WHEN the tree is validated
- THEN application MUST fail naming the package, field, verb, and the list
  shape it requires.

### Requirement: Full path validation

r[mantle.spec_override_tree.path_validation] Every override path MUST be
validated before application: an unknown package, a field that does not exist
when the verb is not `set`, and a dependency reference naming no package in
the final set MUST all fail with the complete path in the message.

Dependency references MUST resolve by name against the final package set, so
an override may add a dependency that another override introduced.

#### Scenario: Unknown package

- GIVEN an override tree naming a package absent from the set
- WHEN validation runs
- THEN the error MUST name the full path including the missing package.

#### Scenario: Dependency name resolution

- GIVEN an override appending a dependency name introduced by another layer
- WHEN the merged tree resolves references against the final set
- THEN the reference MUST resolve to the introduced package.

### Requirement: Bounded merge cost

r[mantle.spec_override_tree.merge_cost_bound] Multiple override layers MUST
merge into one tree before application, so evaluation cost is bounded by the
packages touched plus the number of edits, independent of layer count.

The merged tree MUST preserve layer order within a field. A measurement over
many layers MUST demonstrate the bound once implemented.

#### Scenario: Ten layers cost as one

- GIVEN the same edits supplied as ten layers and as one merged tree
- WHEN both evaluate
- THEN the resulting specs MUST be identical and evaluation cost MUST remain
  within the declared bound for both.

#### Scenario: Order preservation

- GIVEN two layers appending different entries to the same list field
- WHEN the merged tree applies
- THEN the entries MUST appear in layer order.

### Requirement: Adoption is gated on a named consumer

r[mantle.spec_override_tree.bounded_adoption_gate] Implementation MUST NOT
start until a recorded admission decision names the first consumer, shared
with or explicitly aligned to the package-layer admission.

Until the admission is recorded, every implementation task MUST stay open.

#### Scenario: Admission aligned with the package layer

- GIVEN a recorded package-layer admission decision
- WHEN the override-tree admission is recorded
- THEN it MUST name the same consumer surface or record its own with owner
  and path.

#### Scenario: No admission

- GIVEN no admission decision recorded
- WHEN any implementation task is proposed for completion
- THEN the gate MUST treat the claim as blocked.
