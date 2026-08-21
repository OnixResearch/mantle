# Tasks: Pilot transactional reconciliation core

## Pilot

- [x] [serial] Pin the shared core revision in Cargo and Nix and add one identity-binding pilot test. r[mantle.reconciliation_pilot.shared_core]
- [x] [parallel] Run positive current-plan and negative changed-plan classification cases. r[mantle.reconciliation_pilot.validation]
- [x] [serial] Record pilot evidence and run focused tests with scoped first-party gates. r[mantle.reconciliation_pilot.validation]

## Verification Coverage

- `Scenario: Shared core is selected` -> pinned-source pilot test
- `Scenario: Changed plan is stale` -> stale classification case
