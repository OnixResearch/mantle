## ADDED Requirements

### Requirement: Repository maintains complete Tiger Style conformance

r[build_correctness.repository_tiger_conformance] Mantle MUST keep the complete configured first-party Rust library scope within the pinned Tiger Style policy without lint allowances, warning budgets, finding baselines, target-scope reductions, or weaker full-check enforcement, while preserving operator wire compatibility, remediation order, bootstrap identity, protected-execution authority, supervision behavior, audit identity, error-envelope compatibility, and public behavior except the checker-required fixed-width audit-count normalization.

#### Scenario: complete repository check accepts all configured libraries

GIVEN all previously exposed package and root-library findings have structural repairs
WHEN `nix build .#checks.x86_64-linux.tigerstyle --no-link -L --builders ''` runs
THEN it MUST exit successfully with zero Tiger Style findings
AND the result MUST cover every package in `workspace.metadata.tigerstyle.default_scope` with the configured `--lib` target policy.

#### Scenario: optional operator fields are absent

GIVEN a compatible operator contract omits a field that previously defaulted to an empty collection
WHEN Mantle deserializes and validates the contract
THEN it MUST construct the same explicit empty value and preserve validation behavior
AND malformed, duplicate, unsorted, or over-limit input MUST still fail through typed errors.

#### Scenario: remediation facts match more than one family

GIVEN failure facts could satisfy multiple remediation predicates
WHEN Mantle classifies the failure
THEN it MUST select the same first matching policy family as before decomposition
AND the safe subject, retry command, documentation reference, and matched pattern MUST remain unchanged.

#### Scenario: protected execution reaches a limit or kernel failure

GIVEN protected execution encounters count overflow, invalid source authority, response failure, unreadable tracee memory, or descendant timeout
WHEN Mantle handles that condition
THEN it MUST reject or report the condition through the existing typed fail-closed boundary
AND its public audit count MUST use `u32` with checked caller conversion
AND it MUST NOT widen executable authority, truncate counts, discard an error, or continue an unapproved execution.

#### Scenario: serialization fails during error rendering

GIVEN error-envelope or remediation serialization returns an error
WHEN Mantle renders operator output
THEN it MUST not panic
AND it MUST return a deterministic bounded error representation through the compatible output boundary.

#### Scenario: full checks advance beyond Tiger Style

GIVEN the complete repository Tiger Style command passes
WHEN local-builder and ordinary `nix flake check -L` run
THEN neither run MUST fail on a Tiger Style finding
AND any independent infrastructure or fixed-output failure MUST remain an exact blocker without weakening its gate.
