# Implementation and integration tasks

Unchecked items are not claimed from code inspection alone.

## Producer materialization

- [x] [serial] Preserve the independently rechecked NASA source revision, exact archive, lock and existing reference profile. r[mantle.spacewasm_pilot.fixtures]
  - Source prefetch matched SHA256 `sha256-iNgipIlAPOJU38oOqvqxXQF15FJ0ooN3rH5DzHiKOTU=` and BLAKE3 `0db57636fb83a8c47f0c55601236cab648dd5edfdd2567cae0619aa74b8c4da3`; extracted upstream lock BLAKE3 matched `4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1`.
- [x] [serial] Author additive segmentation and full canonical ABI positive/negative fixture sources, descriptors and binary readback derivations. r[mantle.spacewasm_pilot.fixtures]
- [x] [serial] Include all new member identities through the existing real reference bundler and retain the ABI generator authoring source. r[mantle.spacewasm_pilot.fixtures]
- [x] [serial] Add real-bundle missing, mismatch and substitution negative controls and document external publication/local-build admission responsibility. r[mantle.spacewasm_pilot.boundary]
- [x] [serial] Run the producer, complete-bundle independent verifier and existing negative check on the tree integrated with upstream `main`. r[mantle.spacewasm_pilot.fixtures] r[mantle.spacewasm_pilot.boundary]
  - Evidence: [integration run](evidence/integration-2026-10-30.md): bundle identity `774331f6c2defe75e30c354b2b3080838cf56438bbd99268779fe4590f8c9f39` verified valid; the pilot missing/mismatch/substitution controls and the five existing negative controls were rejected.

## Consumer and publication

- [ ] [serial] Consumer owners (Molten, ChaosControl) run the positive/negative ABI, hostcall, memory, trap, stack and cumulative-fuel scenarios against the exact member bytes and keep those outcomes in their own receipts. r[mantle.spacewasm_pilot.boundary]
- [ ] [serial] Publish the exact immutable Mantle revision and bind the observed bundle identity through release authority before consumer pilot execution. r[mantle.spacewasm_pilot.boundary]
