## ADDED Requirements

### Requirement: Bootstrap Validate Success Evidence Regression [r[bootstrap-validate-evidence-regression]]
Crunch MUST regression-test the successful `crunch bootstrap validate` evidence path so runtime-validation evidence bundles remain stable.

#### Scenario: Successful validation writes complete evidence [r[bootstrap-validate-evidence-regression.1]]
- GIVEN a minimal derivation that can build in a temporary store
- WHEN `crunch bootstrap validate` runs with `--evidence-dir`
- THEN the command exits successfully and writes doctor, build log, JSON summary, and Markdown summary evidence with `passed` status

#### Scenario: Failure coverage remains intact [r[bootstrap-validate-evidence-regression.2]]
- GIVEN the existing preflight-failure regression
- WHEN the success-path test is added
- THEN failure-path evidence assertions still pass
