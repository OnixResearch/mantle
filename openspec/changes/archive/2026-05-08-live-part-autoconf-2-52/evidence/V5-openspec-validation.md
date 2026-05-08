# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.autoconf.2.52

## Pre-archive validation

- `openspec validate live-part-autoconf-2-52 --strict --json > /tmp/openspec-autoconf252-final-prearchive.json`: passed.
- `python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-autoconf-2-52 > /tmp/openspec-helper-autoconf252-final-prearchive.log 2>&1`: exit 0; reported the expected pre-checkbox `tasks incomplete` warning plus heading-id warnings.
- `openspec validate live-part-autoconf-2-52 --strict --json > /tmp/openspec-autoconf252-final-prearchive-2.json`: passed after V5 checkbox.
- `python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-autoconf-2-52 > /tmp/openspec-helper-autoconf252-final-prearchive-2.log 2>&1`: exit 0; reported only heading-id warnings.
- `git diff --check`: passed before archive.

## Post-archive verification

- `openspec archive live-part-autoconf-2-52 --yes`: succeeded; archive path `openspec/changes/archive/2026-05-08-live-part-autoconf-2-52`.
- `openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-autoconf252-archive.json`: passed.
- `openspec validate --all --strict --json > /tmp/openspec-all-after-autoconf252-archive.json`: passed.
- `git diff --check`: passed after archive.
