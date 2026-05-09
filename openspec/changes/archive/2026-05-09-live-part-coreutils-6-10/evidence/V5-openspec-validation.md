# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.coreutils.6.10

## Pre-archive validation

- `openspec validate live-part-coreutils-6-10 --strict --json > /tmp/openspec-coreutils610-final-prearchive.json`: passed.
- `python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-coreutils-6-10 > /tmp/openspec-helper-coreutils610-final-prearchive.log 2>&1`: ran before V5 checkbox; warnings reviewed.
- `openspec validate live-part-coreutils-6-10 --strict --json > /tmp/openspec-coreutils610-final-prearchive-2.json`: passed after V5 checkbox.
- `python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-coreutils-6-10 > /tmp/openspec-helper-coreutils610-final-prearchive-2.log 2>&1`: ran after V5 checkbox; only non-blocking helper warnings reviewed.
- `git diff --check`: passed before archive.

## Post-archive verification

- Archive path: `openspec/changes/archive/2026-05-09-live-part-coreutils-6-10`.
- `openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-coreutils610-archive.json`: passed.
- `openspec validate --all --strict --json > /tmp/openspec-all-after-coreutils610-archive.json`: passed.
- `git diff --check`: passed after archive.
