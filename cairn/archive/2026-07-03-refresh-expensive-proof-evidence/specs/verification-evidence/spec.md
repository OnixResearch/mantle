## ADDED Requirements

### Requirement: Expensive proof evidence refresh

r[verification_evidence.expensive_proof_evidence_refresh] Mantle MUST keep expensive self-build and Cargo-free fixed-point proof claims tied to current, inspectable evidence or to an explicit current blocker artifact.

#### Scenario: refreshed proof evidence is current

GIVEN an operator refreshes an expensive Mantle proof on the current tree
WHEN the evidence summary is written
THEN it MUST include the exact command or script, relevant environment selections, output bundle path, final verdict, and BLAKE3 digests for produced proof-critical artifacts when present.
AND any status or docs claim MUST cite that refreshed evidence.

#### Scenario: blocked proof remains honest

GIVEN the expensive proof blocks or fails
WHEN the evidence summary and docs are updated
THEN they MUST name the blocker class, command, output path, and next action.
AND they MUST NOT describe the proof as successful, Nix-free, release-ready, or fixed-point complete unless the recorded verdict proves that claim.

#### Scenario: stale evidence is rejected

GIVEN a proof summary points to missing artifacts, mismatched digests, a stale tree identity, or an uninspected historical transcript
WHEN readiness or status wording evaluates that summary
THEN Mantle MUST treat the proof claim as not current.
AND it MUST require a rerun or narrower blocker report before the claim is surfaced.

#### Scenario: heavy artifacts are summarized

GIVEN a proof run produces stores, binaries, logs, or large bundles
WHEN the change is committed
THEN source control MUST contain only concise transcripts, metadata, digest summaries, and docs needed to find or reproduce the evidence.
AND large generated proof artifacts MUST remain outside the repository unless a separate artifact policy explicitly allows them.
