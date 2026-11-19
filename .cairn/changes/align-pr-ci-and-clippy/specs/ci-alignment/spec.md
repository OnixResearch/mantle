# CI Alignment Delta

## ADDED Requirements

### Requirement: Pull requests receive repository validation

r[ci_alignment.pull_request] The flake-check workflow MUST run for pull
requests from the same repository and forks, using read-only permissions and no
personal credentials.

#### Scenario: A fork pull request is opened

GIVEN a pull request from a fork
WHEN the workflow runs
THEN it MUST check the reviewed PR source/merge revision
AND it MUST NOT receive privileged publication or secret authority.

### Requirement: Strict Clippy covers first-party targets

r[ci_alignment.clippy_targets] The flake Clippy gate MUST lint first-party
library, binary, and test targets with `--all-targets` and MUST preserve the
canonical vendored-member exclusions.

#### Scenario: A binary-only lint regression is introduced

GIVEN a first-party violation that appears only in a binary or test target
WHEN the flake Clippy gate runs
THEN it MUST fail rather than escaping a library-only leg.
