## ADDED Requirements

### Requirement: Mantle transcripts MUST provide executable markdown repros [r[operator-diagnostics.mantle-transcripts]]

Mantle MUST define an executable markdown transcript format for CLI workflows. The format MUST support Mantle command blocks, expected output checks, expected-error blocks, hidden setup blocks, and isolated state defaults. Running a transcript MUST produce a durable output artifact summarizing command results.

#### Scenario: Expected error lets transcript continue [r[operator-diagnostics.mantle-transcripts.expected-error]]

- GIVEN a transcript stanza marked as an expected error
- WHEN the Mantle command in that stanza fails with the expected diagnostic fragment
- THEN the transcript runner records the stanza as passed
- AND subsequent stanzas continue executing

#### Scenario: Transcript defaults to isolated state [r[operator-diagnostics.mantle-transcripts.isolated]]

- GIVEN a transcript without an in-place marker
- WHEN the runner executes it
- THEN the runner uses a fresh temporary store and state directory
- AND the operator's default store/state are not mutated
