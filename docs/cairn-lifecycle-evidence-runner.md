# Cairn lifecycle evidence runner

Mantle keeps proof-before-claim evidence in each Cairn change. The lifecycle
runner is a repo-owned helper for one active change at a time. It keeps the proof
commands explicit, captures transcripts, checks Cairn gates, verifies accepted
spec requirement IDs, records archive paths, and preserves same-run status output
before a change is reported as drained.

The runner does not choose implementation tests for you. Put the commands that
prove the change in a text file and review that file before running the helper.
Blank lines and `#` comments are ignored.

## Dry run

Lifecycle state lives under `.cairn/` (`.cairn/changes/<change>/`,
`.cairn/archive/<date>-<change>/`). Two inputs select the Cairn build and the
archive directory name:

- `--cairn <value>` (default: `$CAIRN`, else `path:../cairn#cairn`). A value
  containing `#` is a flake reference run through `nix run <ref> --`; any other
  value is executed directly, e.g. a prebuilt `…/bin/cairn`.
- `--archive-date YYYY-MM-DD` (default: `$CAIRN_ARCHIVE_DATE`, else the current
  UTC date). Dates before 2000-01-01, such as the `1970-01-01` epoch default
  that produced misdated archives, are rejected.

Use dry-run mode before mutation:

```bash
nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs \
  --change <change-name> \
  --commands .cairn/changes/<change-name>/evidence/commands.txt \
  --cairn <flake-ref-or-cairn-binary> \
  --requirement-id verification_evidence.example_requirement \
  --dry-run
```

Dry-run mode reads the tasks file and command list, prints the selected Cairn,
archive date and planned archive path, and lists the exact commands that would
be captured. It does not run Cairn, mutate specs, archive the change, or write
evidence.

## Fixture and self-test checks

The runner has a fixture mode for validating the pure lifecycle decision logic:

```bash
nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs \
  --fixture tests/fixtures/cairn-lifecycle/complete \
  --requirement-id verification_evidence.cairn_lifecycle_runner \
  --requirement-id verification_evidence.lifecycle_evidence_transcript \
  --requirement-id verification_evidence.lifecycle_runner_fail_closed \
  --dry-run
```

Run the built-in positive and negative cases with:

```bash
nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs --self-test
```

The self-test covers unchecked tasks, missing command transcripts, failed Cairn
gates, accepted-spec sync gaps, dirty status output, missing command lists, and
missing post-archive validation output.

## Execute mode

Execute mode is intentionally boring and explicit:

1. Runs every command from the command list with `/bin/sh -c` from the repo root.
2. Records bounded stdout/stderr and the exit code of each command.
3. Runs Cairn validate, proposal/design/tasks gates, sync dry-run/execute,
   archive dry-run/execute (with `CAIRN_ARCHIVE_DATE` set to the selected date),
   post-archive validation, and `git status --short --branch`.
4. Stops at the first non-zero command, then still writes the transcript with a
   `FAILED` result so a blocked run leaves evidence.
5. Writes `evidence/lifecycle-runner.md` into `.cairn/changes/<change>/`, or into
   `.cairn/archive/<date>-<change>/` when the archive step moved the change.

Use execute mode only after reviewing the dry-run plan and confirming the task
boxes are ready:

```bash
nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs \
  --change <change-name> \
  --commands .cairn/changes/<change-name>/evidence/commands.txt \
  --cairn <flake-ref-or-cairn-binary> \
  --requirement-id verification_evidence.example_requirement
```

If Cairn sync produces a skeleton or drops requirement text, use the accepted-spec
ID check in the runner summary as a blocker and repair the accepted spec before
archive. A final status or post-archive validation claim should cite the archived
`evidence/lifecycle-runner.md` transcript, not a chat log.
