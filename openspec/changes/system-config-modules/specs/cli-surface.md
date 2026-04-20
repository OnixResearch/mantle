# CLI Surface

## Overview

User-facing commands that wire the module pipeline together. Thin
imperative shell over the pure core.

## Requirements

### CLI-1: System eval command

`crunch system eval <inventory.ncl>` MUST evaluate modules, collect
fragments, and run the assembler in dry-run mode. Output is the list
of derivation records that would be built, printed as JSON to stdout.
No builds are performed.

The `--stop-after` flag MAY override the pipeline depth:
- `--stop-after=fragments` — print merged config trees (post-collector,
  pre-assembler).
- `--stop-after=derivations` (default) — print derivation records
  (post-assembler dry-run).

### CLI-2: System build command

`crunch system build <inventory.ncl>` MUST evaluate modules, assemble
derivations, and build them through `crunch-pipeline::build()`. Output
follows the existing `--json` build report format.

### CLI-3: Module directory discovery

Both commands MUST accept `--modules <dir>` to specify the module
directory. If omitted, the default MUST be `./modules/` relative to the
inventory file's parent directory.

### CLI-4: Machine filter

Both commands MUST accept `--machine <name>` to restrict evaluation and
building to a single machine. May be repeated for multiple machines.

### CLI-5: Assembler selection

`crunch system build` MUST accept `--assembler <name>` to override the
per-machine assembler backend. When omitted, the backend is determined
from inventory metadata.

### CLI-6: Eval output formats

`crunch system eval` MUST support `--format json` (default).
`--format nickel` SHOULD be supported but MAY be deferred to a
follow-on change. If deferred, the CLI MUST accept `--format nickel`
and return a clear error message indicating it is not yet implemented.
