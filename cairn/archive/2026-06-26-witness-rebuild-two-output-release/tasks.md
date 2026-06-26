# Tasks

## Contract

- [x] [serial] Specify that witness rebuild MUST bind every published release output it signs to an inspected rebuilt proof artifact by digest, and MUST fail closed for missing or mismatched outputs. r[verification_evidence.release_witness_rebuild_multi_output]

## Implementation

- [x] [serial] Extend witness proof-manifest parsing and output collection to support multiple release outputs while preserving digest-first matching and path containment. r[verification_evidence.release_witness_rebuild_multi_output]
- [x] [serial] Keep the witness rebuild shell path unchanged except for consuming the expanded output list, so workflow execution, logs, audit metadata, and sidecar creation stay on the existing boundary. r[verification_evidence.release_witness_rebuild_multi_output]

## Verification

- [x] [serial] Add positive tests for two-output proof collection and one-output legacy collection. r[verification_evidence.release_witness_rebuild_multi_output]
- [x] [serial] Add negative tests for missing expected proof output and escaping proof artifact paths. r[verification_evidence.release_witness_rebuild_multi_output]
- [x] [serial] Run focused witness rebuild tests, release CLI tests affected by witness rebuild, Cairn validation, and a full helper rerun or record the remaining blocker. r[verification_evidence.release_witness_rebuild_multi_output]
