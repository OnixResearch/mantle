## ADDED Requirements

### Requirement: Project soundness claims are evidence bounded [r[verification_evidence.project_soundness_claims]]

Mantle MUST keep project soundness claims bounded to the inspected command evidence. A clean project soundness check MUST NOT be presented as proof of build success, source availability, input trust, release reproducibility, or clean VCS state.

#### Scenario: Soundness claim cites check evidence [r[verification_evidence.project_soundness_claims.scenario.claim]]

- GIVEN a task, evidence file, status reply, commit message, or final summary claims project files are sound
- WHEN the claim is made
- THEN it MUST cite current `mantle check` output, a Cairn evidence transcript, or inspected diagnostic artifact
- AND the claim MUST be no broader than the checks and modes that were actually run.

#### Scenario: Static check does not prove dynamic facts [r[verification_evidence.project_soundness_claims.scenario.dynamic-non-claim]]

- GIVEN only default no-network project soundness evidence has been inspected
- WHEN readiness is summarized
- THEN Mantle MUST NOT claim current upstream freshness, remote source availability, trust verification, or build success
- AND the summary MUST state those facts are not proven unless explicit probe, trust, fetch, or build evidence exists.
