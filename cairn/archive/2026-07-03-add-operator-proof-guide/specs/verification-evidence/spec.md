## ADDED Requirements

### Requirement: Operator proof guide

r[verification_evidence.operator_proof_guide] Mantle MUST provide an operator-facing proof guide that explains how to run, inspect, and report self-build, Cargo-free fixed-point, and Nix-free demo-bundle evidence without overstating the proven scope.

#### Scenario: guide names commands and outputs

GIVEN an operator wants to run Mantle proof workflows
WHEN they read the proof guide
THEN the guide MUST name the relevant commands or scripts, prerequisite categories, output bundle locations, important receipt files, and cleanup considerations.
AND command snippets MUST be kept consistent with current CLI help or documented compatibility surfaces.

#### Scenario: guide explains proof outcomes

GIVEN a proof workflow succeeds, blocks, fails, or has stale evidence
WHEN the guide explains status reporting
THEN it MUST describe the allowed claim for each outcome.
AND blocked or stale evidence MUST NOT be described as proof success.

#### Scenario: guide records non-claims

GIVEN the guide describes Nix-free demo or fixed-point proof evidence
WHEN it lists what the evidence proves
THEN it MUST also list relevant non-claims such as compiler correctness, full Cargo compatibility, release reproducibility, deploy success, and general Nix replacement completeness unless separately proven.
AND those non-claims MUST align with demo bundle validation wording.

#### Scenario: guide drift is detected

GIVEN CLI help, proof-bundle fields, or documented commands change
WHEN guide drift validation runs
THEN stale command snippets, missing proof fields, or overbroad proof wording MUST be reported deterministically.
AND the guide MUST be updated before guide-backed proof-readiness claims are made.
