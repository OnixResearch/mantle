# Contribution Policy Delta

## ADDED Requirements

### Requirement: Security reports have a private route

r[contribution_policy.private_reporting] The repository MUST document a private
vulnerability-reporting route and MUST discourage public disclosure of
exploitable findings before coordination.

#### Scenario: A reporter finds an exploitable issue

GIVEN a suspected vulnerability
WHEN the reporter follows the repository policy
THEN they MUST have a private advisory/contact route
AND the documented scope and expectations MUST be explicit.

### Requirement: Contribution terms and gates are documented

r[contribution_policy.guide] The repository MUST document the supported
development shell, required gates, Cairn lifecycle expectations, evidence
rules, and the licensing terms that apply to contributions.

#### Scenario: A contributor prepares a change

GIVEN a proposed code or documentation change
WHEN the contributor reads the guide
THEN the documented commands MUST match the repository's actual gates
AND the contribution license MUST be stated.
