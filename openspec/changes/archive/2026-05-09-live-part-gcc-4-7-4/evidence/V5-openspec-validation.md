# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.gcc.4.7.4

## Pre-archive validation

Commands:

```sh
openspec validate live-part-gcc-4-7-4 --strict --json > /tmp/openspec-gcc474-final-prearchive.json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-gcc-4-7-4 > /tmp/openspec-helper-gcc474-final-prearchive.log 2>&1 || true
git diff --check
openspec validate live-part-gcc-4-7-4 --strict --json > /tmp/openspec-gcc474-final-prearchive-2.json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-gcc-4-7-4 > /tmp/openspec-helper-gcc474-final-prearchive-2.log 2>&1 || true
git diff --check
```

Result: strict OpenSpec validation passed twice; whitespace diff check passed twice. Helper verification reported only advisory missing-heading-id warnings.

## Post-archive validation

Commands:

```sh
openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-gcc474-archive.json
openspec validate --all --strict --json > /tmp/openspec-all-after-gcc474-archive.json
git diff --check
```

Result: post-archive `bootstrap` and `--all` strict validation passed; whitespace diff check passed. Archive path: `openspec/changes/archive/2026-05-09-live-part-gcc-4-7-4`.
