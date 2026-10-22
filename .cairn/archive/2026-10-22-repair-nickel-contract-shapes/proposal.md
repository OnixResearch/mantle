# Proposal: Repair the Nickel standard-library contract shapes

## Why

The Nickel standard library does not evaluate under the pinned Nickel 1.17.0.
`nickel eval --import-path lib lib/lib.ncl` fails with `missing definition for
<field>`, and the same failure appears in every module that declares a plan
record with more than one required field. The defect is pre-existing: the
published baseline `43ccd6884` fails the same way, so it is not a
reconciliation regression.

The consequence is that every Nickel-evaluating test in the ordinary
first-party gate fails at import time, the reconciliation's gate evidence
cannot be produced, and no authored plan can be checked at all.

Two further defects share the same cause and are repaired with it: the
reconciled adapter modules export bindings whose field name shadows the
exported value, which Nickel reports as an infinite recursion, and
`lib/project.ncl` writes a catch-all field as `{ _ | Type }`, which Nickel
1.17.0 rejects.

## What Changes

- Restate every plan record that declares several required fields so that it
  evaluates under the pinned interpreter, without changing the declared
  fields, their defaults, their requiredness, or their closedness.
  r[mantle.nickel_contract_shapes.evaluable_shapes]
- Keep the Rust glue layer the admission authority for every evaluated plan;
  the Nickel shape is an authoring-time check. r[mantle.nickel_contract_shapes.admission_boundary]
- Keep the per-field documentation and the exported names of every standard
  library module. r[mantle.nickel_contract_shapes.published_surface]
- Record the repair decision and its non-claims.
  r[mantle.nickel_contract_shapes.decision_record]

## Impact

- Affected modules: `lib/contracts.ncl`, `lib/derivation.ncl`,
  `lib/fixed_output.ncl`, `lib/finish_gates.ncl`, `lib/fetch.ncl`,
  `lib/cmake.ncl`, `lib/esp_idf.ncl`, `lib/fpga.ncl`, `lib/gpu.ncl`,
  `lib/zephyr.ncl`, `lib/android.ncl`, `lib/android/sources.ncl`,
  `lib/wasm_component.ncl`, `lib/kernelscript_experiment.ncl`,
  `lib/remote-builders.ncl`, `lib/project.ncl`, `lib/seed.ncl`, and
  `lib/oci_registry_trust.ncl`.
- No Rust change, no store or build-path change, and no change to the
  embedded standard-library file list.
- The ordinary first-party gate becomes able to run its Nickel-evaluating
  suites. That gate result is evidence for the recorded revision only; it is
  not a bootstrap, release, reproducibility, or parity claim.
