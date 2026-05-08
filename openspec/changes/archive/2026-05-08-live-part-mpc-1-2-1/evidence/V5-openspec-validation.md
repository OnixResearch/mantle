# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.mpc.1.2.1

## Pre-archive validation

- `openspec validate live-part-mpc-1-2-1 --strict --json > /tmp/openspec-mpc-final-prearchive-2.json`: passed.
- : exit 1; warnings reviewed.
- Helper warnings are heading-ID style warnings plus the expected pre-checkbox task warning in the earlier transcript; the final task file now has all tasks checked.
- `git diff --check`: passed before archive.

## Post-archive verification

- `openspec archive live-part-mpc-1-2-1 --yes`: succeeded; archive path `openspec/changes/archive/2026-05-08-live-part-mpc-1-2-1`.
- `openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-mpc-archive.json`: passed.
- `openspec validate --all --strict --json > /tmp/openspec-all-after-mpc-archive.json`: passed.
- `git diff --check`: passed after archive.
