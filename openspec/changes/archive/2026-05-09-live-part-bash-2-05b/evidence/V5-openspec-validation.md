# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.bash.2.05b

## Pre-archive validation

- `openspec validate live-part-bash-2-05b --strict --json > /tmp/openspec-bash205b-final-prearchive.json`: passed.
- `python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-bash-2-05b > /tmp/openspec-helper-bash205b-final-prearchive.log 2>&1`: ran before V5 checkbox; warnings reviewed.
- `openspec validate live-part-bash-2-05b --strict --json > /tmp/openspec-bash205b-final-prearchive-2.json`: passed after V5 checkbox.
- `python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-bash-2-05b > /tmp/openspec-helper-bash205b-final-prearchive-2.log 2>&1`: ran after V5 checkbox; only non-blocking helper warnings reviewed.
- `git diff --check`: passed before archive.

## Post-archive verification

- Archive path: `openspec/changes/archive/2026-05-09-live-part-bash-2-05b`.
- `openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-bash205b-archive.json`: passed.
- `openspec validate --all --strict --json > /tmp/openspec-all-after-bash205b-archive.json`: passed.
- `git diff --check`: passed after archive.
