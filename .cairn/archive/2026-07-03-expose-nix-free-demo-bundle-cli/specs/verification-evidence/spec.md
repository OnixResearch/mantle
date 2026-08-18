## ADDED Requirements

### Requirement: Nix-free demo bundle CLI

r[verification_evidence.nix_free_demo_bundle_cli] Mantle MUST provide an operator-invocable CLI path for validating and rendering the Nix-free fixed-point demo bundle profile without bypassing the demo bundle validator.

#### Scenario: claimable bundle validates through CLI

GIVEN an operator has a demo bundle machine summary with matching stage digests, source-root/toolchain evidence, required guard-denial records, replay hints, and explicit non-claims
WHEN the operator validates it through the CLI
THEN Mantle MUST report the bundle as demo-claimable using the same decision as the pure validator.
AND JSON output MUST include stable profile, claimability, verdict, and diagnostic fields.

#### Scenario: missing fixed-point evidence is rejected through CLI

GIVEN a demo bundle machine summary lacks successful stage1/stage2 fixed-point digest evidence
WHEN the operator validates it through the CLI
THEN Mantle MUST report `missing-fixed-point-evidence`.
AND human output MUST NOT include a Nix-free fixed-point success claim.

#### Scenario: missing guard evidence is rejected through CLI

GIVEN a demo bundle machine summary lacks a required Cargo, Nix, rustup, or ambient-wrapper denial record
WHEN the operator validates it through the CLI
THEN Mantle MUST report `missing-guard-evidence`.
AND the command MAY still describe narrower evidence that is actually present.

#### Scenario: README rendering is derived

GIVEN an operator asks the CLI to render the demo bundle README
WHEN Mantle writes the README text
THEN pass/fail status, digests, guard statuses, replay hints, and non-claims MUST be derived from the machine summary and validation result.
AND hand-edited README fields MUST NOT be required for claimability.
