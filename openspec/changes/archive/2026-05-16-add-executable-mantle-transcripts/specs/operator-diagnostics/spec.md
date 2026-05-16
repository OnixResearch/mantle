## ADDED Requirements

### Requirement: Mantle transcripts MUST provide executable markdown repros [r[operator-diagnostics.mantle-transcripts]]

Mantle MUST define an executable markdown transcript format for CLI workflows. The format MUST support Mantle command blocks (`mantle`), expected output checks (`expect` and `expect:json`), expected-error blocks (`mantle:error`), hidden setup blocks (`setup:hide`), hidden cleanup blocks (`cleanup:hide`), and transcript options (`transcript:options`). Running a transcript MUST produce a durable output artifact summarizing command results.

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

#### Scenario: In-place transcripts require explicit caller approval [r[operator-diagnostics.mantle-transcripts.in-place-opt-in]]

- GIVEN a transcript declares `in_place: true` in `transcript:options`
- WHEN the runner is invoked without an explicit in-place allow flag
- THEN the runner rejects the transcript before running command blocks
- AND it reports that in-place state mutation needs operator approval

#### Scenario: Hidden setup is not an operator step [r[operator-diagnostics.mantle-transcripts.hidden-setup]]

- GIVEN a transcript contains `setup:hide` before a visible command
- WHEN the runner executes the transcript
- THEN hidden setup output is captured in the evidence artifact
- AND the hidden setup command is not rendered as a user-facing operator step

#### Scenario: Output matching uses normalized fragments [r[operator-diagnostics.mantle-transcripts.normalized-output]]

- GIVEN command output contains temporary store/state paths or platform-specific line endings
- WHEN an `expect` block checks stable output fragments
- THEN the runner matches against normalized output
- AND raw stdout and stderr remain available in the evidence artifact
