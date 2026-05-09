# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.heirloom.devtools

## Pre-archive validation

Commands:

```sh
openspec validate live-part-heirloom-devtools --strict --json > /tmp/openspec-heirloom-final-prearchive.json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-heirloom-devtools > /tmp/openspec-helper-heirloom-final-prearchive.log 2>&1 || true
git diff --check
openspec validate live-part-heirloom-devtools --strict --json > /tmp/openspec-heirloom-final-prearchive-2.json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-heirloom-devtools > /tmp/openspec-helper-heirloom-final-prearchive-2.log 2>&1 || true
git diff --check
```

Result: strict OpenSpec validation passed twice; whitespace diff check passed twice. Helper verification reported only advisory missing-heading-id warnings.

## Post-archive validation

Commands:

```sh
openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-heirloom-archive.json
openspec validate --all --strict --json > /tmp/openspec-all-after-heirloom-archive.json
git diff --check
```

Result: post-archive `bootstrap` and `--all` strict validation passed; whitespace diff check passed. Archive path: `openspec/changes/archive/2026-05-09-live-part-heirloom-devtools`.
