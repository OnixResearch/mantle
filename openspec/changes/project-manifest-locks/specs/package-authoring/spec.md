## ADDED Requirements

### Requirement: Locked project inputs are importable from generated state

Package Nickel code MUST be able to import locked project inputs from the
project layer's generated file, `.crunch/inputs.ncl`, instead of hand-writing
source records or seed files for every pinned external input.

#### Scenario: Package imports generated locked inputs

- GIVEN a project with a current `crunch.lock`
- AND `.crunch/inputs.ncl` generated from that lock
- WHEN a package Nickel file imports `.crunch/inputs.ncl`
- THEN the package can reference locked inputs from that file
- AND those inputs correspond to the current lock state

#### Scenario: Generated locked inputs replace hand-maintained source records

- GIVEN a package that previously repeated pinned source metadata in local
  Nickel code
- WHEN the package is updated to import `.crunch/inputs.ncl`
- THEN the package no longer needs duplicate hand-maintained source records for
  those locked project inputs
