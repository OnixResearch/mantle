# Oracle checkpoint: distributed evaluation feasibility

Change: `explore-distributed-evaluation`
Task: I7

| Field | Value |
|---|---|
| Question | Can Mantle's current design host distributed evaluation, and which route is actionable now? |
| Inspected evidence | Seam inventory (`evidence/inventory.md`), positive and negative probe transcripts (`evidence/probe-and-report.md`, `probe-evidence.json`, `probe-evidence-negative.json`), deterministic report (`evidence/assessment-report.json`), prior-art design section (`design.md`), 24 core unit tests, self-test |
| Decision | The eval-worker boundary is transportable for the probed fixture. Evaluate-once is `candidate` and is the actionable route. Eval-as-a-service is `blocked` pending cross-boundary streaming, dynamic-goal, and suspension evidence. No authority changed. Recorded in `adr/0078-explore-distributed-evaluation.md`. |
| Owner | Mantle maintainers, review before any eval-service adoption change |
| Next action | A later Cairn change must demonstrate cross-boundary streaming overlap, dynamic-goal admission, and evaluator suspension before an eval-service route can open. The evaluate-once route needs no authority change and can proceed through the existing `nickel-export-core` / `mantlepkgs` pattern. |

## Non-claims

- The probe proves transportability for one static-record fixture only.
- The report does not prove evaluator equivalence, remote-build correctness,
  cross-boundary streaming or dynamic goals, or release eligibility.
- A `candidate` outcome does not modify product behavior.
