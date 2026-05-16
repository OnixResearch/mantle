## ADDED Requirements

### Requirement: Quality gates SHOULD support executable Mantle transcripts [r[quality-gates.mantle-transcripts]]

Mantle SHOULD provide a maintained quality-gate entrypoint that runs selected executable Mantle transcripts. Transcript checks MUST report the transcript path, failing stanza, command, normalized output path, and whether the failure was expected or unexpected.

#### Scenario: Fast transcript suite passes [r[quality-gates.mantle-transcripts.fast-pass]]

- GIVEN the fast transcript suite is selected
- WHEN the transcript quality gate runs
- THEN each transcript executes in isolated store/state by default
- AND the gate exits zero only when all non-bug stanzas match their expectations

#### Scenario: Unexpected transcript failure is diagnostic [r[quality-gates.mantle-transcripts.failure]]

- GIVEN a transcript command fails without an expected-error marker
- WHEN the transcript runner reports the failure
- THEN the quality gate exits non-zero
- AND it reports the transcript path, stanza, command, and saved output artifact
