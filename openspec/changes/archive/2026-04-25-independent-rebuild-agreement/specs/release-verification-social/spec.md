## ADDED Requirements

### Requirement: Policy defines independent rebuild agreement thresholds

Crunch MUST let verifier-local policy define the witness count and independence domains required for independent rebuild agreement.
ID: release.verification.social.independent.agreement.policy

The policy MUST support at least witness identity, signer key name, and host-class independence selectors. Policy evaluation MUST reject witness sets that meet the count threshold only by duplicating the same configured independence domain.

#### Scenario: Host-class independence is required

- GIVEN policy requires two matching witnesses across distinct host classes
- AND two witnesses match the release digests but report the same host class
- WHEN policy evaluation runs
- THEN independent rebuild agreement is not satisfied
- AND the diagnostic names the repeated host class

#### Scenario: Signer-key independence is required

- GIVEN policy requires two matching witnesses across distinct signer key names
- AND two witnesses match with distinct trusted signing keys
- WHEN policy evaluation runs
- THEN the independence predicate is satisfied
