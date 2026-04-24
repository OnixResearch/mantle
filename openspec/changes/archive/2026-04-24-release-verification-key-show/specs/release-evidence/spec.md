## ADDED Requirements

### Requirement: Witnessed self-hosting workflow MUST show key exchange steps

The documented witnessed self-hosting workflow MUST show how release signers and
witness rebuilders obtain the exact `--trusted-public-key <name:base64>` values
needed for verification.
ID: release.evidence.workflow.witnessed.keyexchange

#### Scenario: Witness workflow docs show trusted-key export

- GIVEN a contributor follows the documented witnessed self-hosting workflow
- WHEN they reach the `release-verify` step
- THEN the docs show how the release signer and witness rebuilder run
  `crunch attest key-show` for their signing keys
- AND the docs use those exported values in the `--trusted-public-key`
  examples instead of only placeholder text
