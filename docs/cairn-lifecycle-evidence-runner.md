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

Use dry-run mode before mutation:

```bash
nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs \
  --change <change-name> \
  --commands cairn/changes/<change-name>/evidence/commands.txt \
  --requirement-id verification_evidence.example_requirement \
  --dry-run
```

Dry-run mode reads the tasks file and command list, prints the planned archive
path, and lists the exact commands that would be captured. It does not run
Cairn, mutate specs, archive the change, or write evidence.

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
2. Appends bounded stdout/stderr to `cairn/changes/<change>/evidence/lifecycle-runner.md`.
3. Runs Cairn validate, proposal/design/tasks gates, sync dry-run/execute,
   archive dry-run/execute, post-archive validation, and `git status --short --branch`.
4. Stops at the first non-zero command.

Use execute mode only after reviewing the dry-run plan and confirming the task
boxes are ready:

```bash
nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs \
  --change <change-name> \
  --commands cairn/changes/<change-name>/evidence/commands.txt \
  --requirement-id verification_evidence.example_requirement
```

If Cairn sync produces a skeleton or drops requirement text, use the accepted-spec
ID check in the runner summary as a blocker and repair the accepted spec before
archive. A final status or post-archive validation claim should cite the archived
`evidence/lifecycle-runner.md` transcript, not a chat log.
