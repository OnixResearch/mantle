# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.part.perl.5.000

## Pre-archive validation

Commands:

```sh
openspec validate live-part-perl-5-000 --strict --json > /tmp/openspec-perl5000-final-prearchive.json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-perl-5-000 > /tmp/openspec-helper-perl5000-final-prearchive.log 2>&1 || true
git diff --check
openspec validate live-part-perl-5-000 --strict --json > /tmp/openspec-perl5000-final-prearchive-2.json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-part-perl-5-000 > /tmp/openspec-helper-perl5000-final-prearchive-2.log 2>&1 || true
git diff --check
```

Result: strict OpenSpec validation passed twice; whitespace diff check passed twice. Helper verification reported only advisory missing-heading-id warnings.

## Post-archive validation

Commands:

```sh
openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-perl5000-archive.json
openspec validate --all --strict --json > /tmp/openspec-all-after-perl5000-archive.json
git diff --check
```

Result: post-archive `bootstrap` and `--all` strict validation passed; whitespace diff check passed. Archive path: `openspec/changes/archive/2026-05-09-live-part-perl-5-000`.
