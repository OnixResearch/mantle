## ADDED Requirements

### Requirement: Continuous reproducibility gauntlet

r[verification_evidence.continuous_reproducibility_gauntlet] Mantle MUST aggregate reproducibility confirmation tracks into current, stale-aware gauntlet reports before claiming broad empirical strength.

#### Scenario: aggregate report binds track evidence

GIVEN repeatability, witness, comparison, hermeticity, cache attack, and bootstrap pressure track reports exist
WHEN Mantle creates a continuous reproducibility gauntlet report
THEN the report MUST bind each track report digest, source digest, policy digest, universe digest, toolchain digest, host class, witness set, run id, status, blockers, and schema version
AND the aggregate report digest MUST be deterministic for equivalent inputs.

#### Scenario: stale evidence is not promoted

GIVEN a prior gauntlet report was produced for a different source, policy, universe, toolchain, host class, witness set, or track schema version
WHEN Mantle evaluates current release-readiness or reproducibility status
THEN the report MUST classify the prior evidence as stale or out-of-scope
AND it MUST NOT promote stale evidence to a current global or broad reproducibility claim.

#### Scenario: flakes and blockers remain visible

GIVEN one or more gauntlet tracks fail, block, skip unsupported axes, or alternate between pass and fail across runs
WHEN Mantle renders the aggregate summary
THEN it MUST preserve pass, fail, blocked, unsupported, stale, and flaky states with first and last seen run ids
AND the summary MUST name next actions instead of collapsing partial evidence into a single green claim.
