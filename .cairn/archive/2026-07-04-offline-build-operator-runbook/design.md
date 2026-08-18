## Context

The offline build workflow spans multiple commands and evidence surfaces. The absence of a runbook causes two problems: operators miss existing capabilities, and status reports can accidentally overclaim from source-bundle or offline Cargo evidence. A small UX/docs change can make the current system safer before deeper implementation work lands.

## Decisions

### 1. The runbook is command-first

**Choice:** The runbook presents an ordered workflow: plan/export source bundle on a connected or source-rich host, copy the bundle, import with `--pin`, verify/preflight source state, run `mantle build --offline-source-preflight --no-substitute`, and inspect `network_policy_reports`, `cargo_build_evidence[]`, route reports, and source-bundle reports.

**Rationale:** Operators need exact commands and output shapes, not scattered conceptual descriptions.

### 2. Diagnostics point to next safe action

**Choice:** Common offline blockers include bounded remediation hints: export/import source bundle, pin imported records, inspect unsupported adapter metadata, choose `--no-substitute`, or rerun with explicit network-enabled workflows when offline is not intended.

**Rationale:** A fail-closed diagnostic is more useful when it names the next non-destructive command.

### 3. JSON output remains parseable

**Choice:** Offline runbook diagnostics in JSON mode use documented fields and never mix human hints into stdout outside the command schema. Human hints go to documented human output or stderr as appropriate.

**Rationale:** Existing operator diagnostics require machine-readable output to remain clean.

### 4. Docs are validated like examples

**Choice:** Tests assert that README/operator docs mention the core offline commands, non-claims, and evidence fields. They reject wording that claims build success from source readiness alone.

**Rationale:** The runbook is part of the contract. Drift tests keep it aligned with the CLI.

### 5. The runbook is not a new proof class

**Choice:** The docs explain how to produce evidence, but they do not introduce a new claim stronger than the underlying source-bundle, offline Cargo, route, network-policy, or build report evidence.

**Rationale:** This preserves proof-before-claim and avoids a process checklist being mistaken for proof.

## Risks / Trade-offs

- Docs may get ahead of implementation if source-bundle realization work is not complete; the runbook must distinguish current preflight-only behavior from future source-bundle route execution.
- Too many command variants can overwhelm users; the runbook should separate minimal local examples from bootstrap/self-build/proof workflows.
- CLI hints must avoid leaking full paths, tokens, or unbounded source identities in JSON mode.
