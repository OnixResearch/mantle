# Mantle executable transcripts

Mantle transcripts are Markdown documents that double as CLI examples and
checked reproducers. They are intended for operator docs, bug reports, and
release-proof walkthroughs where examples should stay executable.

## Block types

A runner reads fenced code blocks whose info string starts with one of these
Mantle transcript block types. Other Markdown is prose.

| Block | Meaning | Visible in rendered docs? |
| --- | --- | --- |
| `mantle` | Run a Mantle CLI command that is expected to exit zero. | yes |
| `mantle:error` | Run a Mantle CLI command that is expected to exit non-zero. | yes |
| `expect` | Stable output fragments expected from the preceding command. | yes |
| `expect:json` | JSON-field expectations for the preceding command. | yes |
| `setup:hide` | Shell setup required before visible commands run. | no |
| `cleanup:hide` | Shell cleanup that runs after the transcript, even on failure. | no |
| `transcript:options` | YAML-ish transcript options, such as `in_place: true`. | no |

Each `mantle` or `mantle:error` block is one command stanza. Continuation lines
are allowed, but every command is executed by the runner from the transcript's
working directory with a runner-provided environment.

## Command blocks

`mantle` blocks are successful commands:

```mantle
mantle doctor --profile build
```

`mantle:error` blocks are expected failures. They pass only when the command
exits non-zero and at least one following `expect` fragment matches the combined
normalized stdout/stderr:

```mantle:error
mantle eval missing-file.ncl
```

```expect
missing-file.ncl
```

An unexpected success in `mantle:error` is a transcript failure. An unexpected
failure in `mantle` is also a transcript failure.

## Expected output

`expect` blocks contain stable fragments, not whole raw output transcripts. A
fragment matches after output normalization. Multiple non-empty lines are matched
in order, but unrelated output may appear between them.

`expect:json` blocks are for structured output from commands invoked with
`--json`. The initial runner contract only needs dotted-path equality checks:

```expect:json
schema = "crunch-doctor-report-v1"
status = "ok"
```

## Hidden setup and cleanup

`setup:hide` blocks run before the next visible command stanza. They are for
creating temporary fixtures, writing tiny `.ncl` inputs, or seeding local files
that would distract from the operator-facing example.

Hidden setup output is captured in the transcript evidence artifact, but hidden
commands are not rendered as user-facing steps. Hidden setup failures are always
unexpected failures unless the visible stanza that depends on them is skipped by
the future runner.

`cleanup:hide` blocks run at transcript end and should be best-effort. Cleanup
failures are reported in the evidence artifact but must not hide an earlier
command mismatch.

## State defaults and in-place opt-in

By default, the runner supplies fresh temporary state for each transcript:

- `--store <tmp>/store`
- `--state-dir <tmp>/state`
- `MANTLE_TRANSCRIPT_TMP=<tmp>` for hidden setup and command blocks

The runner must not mutate the operator's default store/state unless the
transcript explicitly opts in:

```transcript:options
in_place: true
reason: "documents repair of an existing operator store"
```

The initial runner should reject `in_place: true` unless the caller also passes an
explicit in-place allow flag. This makes the safe path the default for docs and
bug repros.

## Output normalization

Before matching `expect` fragments, the runner normalizes volatile values:

- absolute transcript temp directories become `$MANTLE_TRANSCRIPT_TMP`
- physical store/state temp directories become `$MANTLE_TRANSCRIPT_STORE` and
  `$MANTLE_TRANSCRIPT_STATE`
- Windows and Unix line endings normalize to `\n`
- trailing whitespace is stripped from each line
- repeated blank lines collapse to a single blank line

The runner should preserve command exit code, raw stdout/stderr paths, normalized
stdout/stderr paths, elapsed time, and expectation results in its durable evidence
artifact.

## Minimal fixtures

Fast fixture transcripts should live under `tests/fixtures/transcripts/fast/`.
The fixture suite begins with:

- `doctor-success.md` — a trivial successful CLI workflow
- `expected-error.md` — a non-zero command that passes because its diagnostic is
  expected
