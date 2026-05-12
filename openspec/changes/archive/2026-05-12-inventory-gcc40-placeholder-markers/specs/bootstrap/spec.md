## MODIFIED Requirements

### Requirement: Bootstrap parity gap report

The bootstrap parity report MUST classify intentional GCC 4.0 pass1 bridge markers using a checked placeholder inventory receipt. The receipt MUST use schema `crunch-gcc40-placeholder-inventory-v1`, MUST name `bootstrap/gcc-4.0.ncl`, MUST mark the inventory as `inventory-only`, and MUST enumerate the exact standalone placeholder-marker occurrences currently present in the derivation. The report MUST fail closed when the receipt is missing or when the recomputed marker set differs from the receipt. A matching inventory MAY classify `gcc.4.0` as evidence-backed `partial`, but MUST NOT mark it `complete` or unblock live-bootstrap/Guix parity without native compiler correctness evidence.

#### Scenario: GCC 4.0 placeholder inventory matches

- GIVEN `bootstrap/gcc-4.0.ncl` contains intentional pass1 bridge markers
- AND `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` enumerates the exact marker set
- WHEN the parity report evaluates `gcc.4.0`
- THEN the row reports `partial` rather than unclassified `placeholder`
- AND the row remains a live-bootstrap and Guix blocker

#### Scenario: GCC 4.0 placeholder inventory drifts

- GIVEN a standalone marker is added, removed, moved, or renamed without updating the receipt
- WHEN the parity report evaluates `gcc.4.0`
- THEN the evidence check fails
- AND the row remains a blocker with a drift note
