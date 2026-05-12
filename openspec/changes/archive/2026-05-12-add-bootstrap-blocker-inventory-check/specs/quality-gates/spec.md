## ADDED Requirements

### Requirement: Bootstrap Blocker Inventory Gate [r[bootstrap-blocker-inventory-gate]]
Crunch MUST provide a deterministic inventory of bootstrap-critical placeholders, TODOs, and deferred work before claiming bootstrap completion.

#### Scenario: Known placeholders are categorized [r[bootstrap-blocker-inventory-gate.1]]
- GIVEN bootstrap-critical derivations contain intentional bridge placeholders
- WHEN the inventory check runs
- THEN findings are listed with stable categories instead of hidden in prose

#### Scenario: Unexpected placeholder blocks completion claims [r[bootstrap-blocker-inventory-gate.2]]
- GIVEN a new uncategorized placeholder appears in a bootstrap-critical surface
- WHEN the inventory check runs
- THEN the check reports it as an unexpected blocker
