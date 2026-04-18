## MODIFIED Requirements

### Requirement: Nickel evaluation via eval_full_for_export

The system MUST provide a stateful lazy session boundary that retains Nickel
program state across at least two operations:

1. a lazy root-discovery path that evaluates the top-level Nickel result only
   enough to determine whether it is a single derivation, an array of
   derivations, or a record of derivations, and
2. a selected-root forcing path that fully evaluates only the requested root
   value for typed extraction or export.

Build and planning execution MUST use the lazy discovery path before any
per-root forcing. Whole-program export-ready deep evaluation MAY still be used
for operator-visible `crunch eval` output or other callers that explicitly need
whole-program export semantics.

#### Scenario: Package-set root discovery does not require whole-program export

- GIVEN a `.ncl` file exporting a record of named derivations
- WHEN crunch-eval opens a lazy session and lists the available root labels
- THEN it returns the top-level labels
- AND it does not require a whole-program export-ready deep evaluation before
  returning those labels

#### Scenario: Selected root forcing matches the eager derivation shape

- GIVEN a `.ncl` file exporting a record of named derivations
- WHEN crunch-eval forces one selected root through the lazy session API
- THEN the resulting typed derivation matches the current eager path for that
  selected root
- AND sibling root labels remain discoverable through the same session

#### Scenario: Missing selected root reports an error

- GIVEN a `.ncl` file exporting named derivations
- WHEN crunch-eval is asked to force a root label that does not exist
- THEN it returns a `crunch-eval` error
- AND it does not silently pick a different root

#### Scenario: Invalid top-level shape reports an error during discovery

- GIVEN a `.ncl` file whose top-level value is neither a derivation, an array
  of derivations, nor a record of derivations
- WHEN crunch-eval performs lazy root discovery
- THEN it returns a `crunch-eval` error
- AND it does not fabricate root labels from that value

## ADDED Requirements

### Requirement: Root discovery supports multi-derivation outputs lazily

The system MUST discover top-level root identities for single-derivation,
array, and record outputs before eagerly deserializing every root value.

The discovery result MUST preserve the current root-label semantics:
- a single derivation uses its `name` field as the label
- an array of derivations uses each derivation's `name` field as the label
- a record of derivations uses each field name as the label

#### Scenario: Array labels come from derivation names without full sibling extraction

- GIVEN a `.ncl` file evaluating to an array of derivation records
- WHEN crunch-eval lists root labels lazily
- THEN each label comes from the corresponding derivation `name`
- AND accessing those `name` fields is the maximum acceptable partial forcing
  for discovery in that array case
- AND crunch-eval does not need to deserialize every sibling into a final Rust
  value before returning the label list

#### Scenario: Record labels come from top-level field names

- GIVEN a `.ncl` file evaluating to a record of derivations
- WHEN crunch-eval lists root labels lazily
- THEN each label is the corresponding top-level field name
- AND individual root values may still be forced later on demand
