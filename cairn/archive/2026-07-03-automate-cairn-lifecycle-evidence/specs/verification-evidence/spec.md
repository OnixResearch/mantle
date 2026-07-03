# Verification Evidence Specification

## Purpose

Defines requirements for automating Mantle's Cairn lifecycle evidence workflow without hiding proof commands.

## Requirements

### Requirement: Cairn lifecycle evidence runner

r[verification_evidence.cairn_lifecycle_runner] Mantle SHOULD provide a repo-owned helper that runs an explicit Cairn change lifecycle command list and produces durable evidence for validation, gates, sync, archive, post-archive validation, and final status.

#### Scenario: Runner executes an explicit command list

GIVEN an active Cairn change and a configured list of validation commands
WHEN the lifecycle runner executes the change workflow
THEN it MUST run the listed commands, record their stdout/stderr or bounded summaries, and identify the command associated with each evidence claim
AND it MUST NOT infer proof success from commands that were not run.

#### Scenario: Runner archives only after passing gates

GIVEN a change has completed implementation tasks and validation commands
WHEN the runner reaches the archive step
THEN it MUST require Cairn validate, proposal gate, design gate, and tasks gate to pass before executing sync/archive
AND it MUST rerun validation after archive.

### Requirement: Lifecycle evidence transcript is durable

r[verification_evidence.lifecycle_evidence_transcript] Lifecycle automation MUST append or write durable transcripts that include command lines, relevant output summaries, receipt hashes when available, archive paths, post-archive validation output, and final status evidence.

#### Scenario: Post-archive evidence is captured

GIVEN Cairn archive has executed
WHEN post-archive validation passes
THEN the archived evidence MUST contain the exact post-archive validation command and output
AND final summaries MAY cite that archived transcript.

#### Scenario: Final status claim has same-turn evidence

GIVEN the runner reports a clean or expected dirty worktree state
WHEN it writes the lifecycle summary
THEN the summary MUST include same-run VCS status output
AND it MUST NOT generalize from pre-archive status.

### Requirement: Lifecycle runner fails closed on missing proof

r[verification_evidence.lifecycle_runner_fail_closed] Lifecycle automation MUST fail closed when required lifecycle evidence is absent, incomplete, contradictory, or disconnected from the tasks and requirement IDs it claims to prove.

#### Scenario: Missing transcript blocks archive

GIVEN a verification task claims a command was run but no transcript is present
WHEN the runner evaluates lifecycle readiness
THEN it MUST refuse to archive the change
AND it MUST identify the missing command evidence.

#### Scenario: Accepted spec sync drops requirement text

GIVEN sync/archive produces an accepted spec that lacks the new requirement IDs or contains only a skeleton
WHEN the runner validates the archived state
THEN it MUST fail with a deterministic sync-repair diagnostic
AND it MUST NOT report the lifecycle as complete.
