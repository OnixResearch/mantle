## Why

The repo's top-level docs no longer match the shipped operator surface.

Current drift is visible in the tree today:

- `README.md`'s CLI section still omits shipped command families such as
  `doctor`, `attest`, `release`, `shell`, and `develop`
- the README documents `crunch doctor` and `crunch build --plan` earlier in the
  file, but that operator workflow is not carried through the command summary
  or deeper workflow guidance
- `--strict-hermetic`, attestation inspection, release-evidence verification,
  and dev-shell entry points exist in `src/main.rs`, but the README does not
  give them a clear operator path
- focused docs already exist for benchmark workflow and bootstrap/release claim
  boundaries, but they need a deliberate reread against the current CLI and
  current main specs so README wording and deeper docs do not drift apart again

This is documentation drift, not a feature gap. The source of truth is the
current CLI and the current main specs.

## What Changes

- audit `README.md` and focused docs against the current shipped CLI surface
  and the relevant main specs
- update the README so it covers the current operator workflows for planning,
  diagnostics, dev shells, attestations, release evidence, and strict
  hermetic builds
- refresh benchmark and bootstrap/release docs so their command examples,
  cross-links, and claim boundaries match the current tree
- keep the README workflow-first while pushing detail into focused docs only
  when a longer explanation would make the top-level document harder to use;
  any new focused doc page must be linked from the README and carry current
  command examples for its workflow

## Capabilities

### Modified Capabilities
- `cli`: the README covers the shipped operator command families and workflow
  toggles
- `performance`: benchmark docs describe the checked-in benchmark workflow and
  sparse metric behavior accurately
- `release-evidence`: README and bootstrap-facing docs describe release
  verification with the same bounded claim language

## Non-Goals

- no source-code or runtime-behavior changes
- no new CLI commands, flags, or help-text semantics
- no `AGENTS.md` rewrite as part of this docs pass

## Impact

- **Files**: `README.md`, `docs/benchmark-suite.md`,
  `docs/bootstrap-stage0-inventory.md`, and any new focused doc page needed to
  keep the README readable
- **APIs**: none
- **Dependencies**: none
- **Testing**: verify docs against current `crunch --help`, relevant
  subcommand help, and the main OpenSpec requirements they summarize

## Notes

This change should not add or change runtime behavior. It should make the docs
honest, current, and easier to navigate.