## ADDED Requirements

### Requirement: Offline build runbook is discoverable and claim-bounded

r[operator_diagnostics.offline_build_runbook] Mantle MUST document and diagnose the offline build workflow as an explicit, claim-bounded operator path. The runbook and diagnostics MUST cover source-bundle export, source-bundle import with pinning, source-bundle verify or offline preflight, `mantle build --offline-source-preflight --no-substitute`, evidence inspection, and common blocker remediation without presenting source readiness, route eligibility, or offline Cargo evidence as build success or stronger proof than the underlying evidence supports.

#### Scenario: operator can find the offline command sequence

GIVEN an operator reads the README, operator workflow guide, or command reference
WHEN they look for an offline build workflow
THEN the docs MUST show the ordered source-bundle export/import/pin/preflight/build/evidence command sequence
AND the docs MUST name the expected human or JSON evidence fields to inspect.

#### Scenario: offline blockers include safe next actions

GIVEN an offline build or preflight finds missing, stale, unpinned, unsupported, untrusted, network-required, or malformed source/evidence state
WHEN Mantle renders human or JSON diagnostics
THEN the diagnostics SHOULD include a bounded next action such as exporting/importing/pinning a source bundle, inspecting adapter metadata, or rerunning an explicit non-offline workflow
AND JSON output MUST remain parseable with stable fields.

#### Scenario: runbook does not overclaim

GIVEN source-bundle readiness, route-plan eligibility, or offline Cargo evidence is cited by docs, diagnostics, tasks, or status replies
WHEN the claim is made
THEN the claim MUST stay within the underlying evidence boundary
AND it MUST NOT claim build success, output trust, Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness without separate evidence.

#### Scenario: sensitive diagnostic data stays redacted

GIVEN offline diagnostics mention stores, source records, trusted keys, substituters, remote builders, or environment-derived configuration
WHEN Mantle renders runbook hints or JSON diagnostics
THEN it MUST omit bearer tokens, private key paths, raw environment values, and unbounded source lists
AND it MAY include bounded counts, stable blocker classes, and digest references.
