# Tasks: Repair the Nickel standard-library contract shapes

- [x] [serial] 1.1 Confirm the pinned interpreter's record-contract, record-type, and record-recursion rules directly against Nickel 1.17.0. r[mantle.nickel_contract_shapes.evaluable_shapes]
- [x] [serial] 1.2 Add the `Shape` whole-shape check to `lib/contracts.ncl`. r[mantle.nickel_contract_shapes.evaluable_shapes]
- [x] [serial] 2.1 Repair `lib/derivation.ncl`, `lib/fixed_output.ncl`, `lib/finish_gates.ncl`, `lib/fetch.ncl`, `lib/cmake.ncl`, `lib/esp_idf.ncl`, `lib/zephyr.ncl`, `lib/seed.ncl`, and `lib/oci_registry_trust.ncl`. r[mantle.nickel_contract_shapes.evaluable_shapes]
- [x] [serial] 2.2 Repair `lib/android.ncl`, `lib/android/sources.ncl`, `lib/fpga.ncl`, `lib/gpu.ncl`, `lib/wasm_component.ncl`, `lib/kernelscript_experiment.ncl`, `lib/remote-builders.ncl`, and `lib/project.ncl`. r[mantle.nickel_contract_shapes.evaluable_shapes]
- [x] [serial] 2.3 Evaluate the whole standard library with the pinned interpreter: `nickel eval --import-path lib lib/lib.ncl`, and evaluate a derivation through the binary's embedded standard library with `CRUNCH_FORCE_EMBEDDED_STDLIB=1`. r[mantle.nickel_contract_shapes.evaluable_shapes]
- [x] [serial] 2.4 Check that every repaired shape still rejects a missing required field, an undeclared field, and a wrongly typed field, and still applies its declared defaults. r[mantle.nickel_contract_shapes.evaluable_shapes]
- [x] [serial] 2.5 Check that every module keeps its exported names and its per-field documentation. r[mantle.nickel_contract_shapes.published_surface]
- [x] [serial] 3.1 Run the Nickel-evaluating first-party suites and the ordinary first-party gate on the repaired revision, and retain the transcripts. The `stdlib_tests` Nickel contract suite reports 28 passed and 0 failed; the gate transcript for revision `2782a40ec` is retained in `.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/final-gate-v2/`. r[mantle.nickel_contract_shapes.decision_record]
- [x] [serial] 3.2 Record the decision in `adr/0124-express-nickel-plan-shapes-as-record-types.md` and update `docs/nickel-stdlib-contract-defect.md`. r[mantle.nickel_contract_shapes.decision_record]
- [x] [serial] 3.3 Close issue #24 with the verified evidence. r[mantle.nickel_contract_shapes.decision_record]
- [x] [serial] 4.1 Keep the Rust glue layer the admission authority for every evaluated plan, and keep the Nickel shape an authoring-time check. r[mantle.nickel_contract_shapes.admission_boundary]
