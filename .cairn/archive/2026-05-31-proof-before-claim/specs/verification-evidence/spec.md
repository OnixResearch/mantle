## ADDED Requirements

### Requirement: Proof before claim

r[verification_evidence.proof_before_claim] Mantle work MUST NOT claim status, completion, feature support, pass/fail validation, or build success unless current evidence has been produced or inspected before the claim is made.

#### Scenario: claim includes current evidence

GIVEN an agent, task, evidence file, commit message, status reply, or final summary states that Mantle functionality works, validation passed, work is complete, or the tree is clean
WHEN that claim is made
THEN the claim MUST name or include current evidence such as command output, Cairn evidence file, task transcript, build report, test log, VCS status output, or inspected artifact content.
AND the claim MUST be narrower than or equal to the evidence being cited.

#### Scenario: evidence is absent

GIVEN current evidence has not been produced or inspected for a potential claim
WHEN status is reported
THEN the report MUST say the claim is not proven or state only the narrower fact that is supported by available evidence.
AND it MUST NOT generalize from mock tests, stale logs, prior sessions, or uninspected artifacts to end-to-end support.

#### Scenario: Cairn task completion is evidence-gated

GIVEN a Cairn task is marked complete
WHEN the task records its completion summary
THEN the task MUST reference durable evidence or an oracle checkpoint that supports the completed task text.
AND review or gate summaries MUST reject completion claims whose evidence proves only a narrower behavior.

#### Scenario: post-archive validation claim is archived

GIVEN a change has been archived
WHEN a commit message, evidence summary, status reply, or final response claims post-archive Cairn validation passed
THEN the archived change evidence transcript MUST include the exact post-archive validation command and output.
AND the claim MUST NOT rely only on pre-archive gates or unstored chat transcript output.
