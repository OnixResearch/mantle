# Tasks: Repair the Nickel standard-library contract shapes

- [x] Confirm the pinned interpreter's record-contract, record-type, and
      record-recursion rules directly against Nickel 1.17.0.
- [x] Add the `Shape` whole-shape check to `lib/contracts.ncl`.
- [x] Repair `lib/derivation.ncl`, `lib/fixed_output.ncl`,
      `lib/finish_gates.ncl`, `lib/fetch.ncl`, `lib/cmake.ncl`,
      `lib/esp_idf.ncl`, `lib/zephyr.ncl`, `lib/seed.ncl`, and
      `lib/oci_registry_trust.ncl`.
- [x] Repair `lib/android.ncl`, `lib/android/sources.ncl`, `lib/fpga.ncl`,
      `lib/gpu.ncl`, `lib/wasm_component.ncl`,
      `lib/kernelscript_experiment.ncl`, `lib/remote-builders.ncl`, and
      `lib/project.ncl`.
- [x] Evaluate the whole standard library with the pinned interpreter:
      `nickel eval --import-path lib lib/lib.ncl`.
- [x] Check that every repaired shape still rejects a missing required field,
      an undeclared field, and a wrongly typed field, and still applies its
      declared defaults.
- [ ] Run the Nickel-evaluating first-party suites and the ordinary
      first-party gate on the repaired revision, and retain the transcripts.
- [x] Record the decision in `adr/0124-express-nickel-plan-shapes-as-record-types.md`
      and update `docs/nickel-stdlib-contract-defect.md`.
- [ ] Close issue #24 with the verified evidence.
