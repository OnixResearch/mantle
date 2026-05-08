# V5 OpenSpec validation and gate evidence

Task-ID: V5
Covers: bootstrap.gcc40.runtime-validation
Captured: 2026-05-08T20:53:38Z

## Commands

```sh
openspec validate live-bootstrap-gcc-4-0-runtime-validation --strict --json
{
  "items": [
    {
      "id": "live-bootstrap-gcc-4-0-runtime-validation",
      "type": "change",
      "valid": true,
      "issues": [],
      "durationMs": 2
    }
  ],
  "summary": {
    "totals": {
      "items": 1,
      "passed": 1,
      "failed": 0
    },
    "byType": {
      "change": {
        "items": 1,
        "passed": 1,
        "failed": 0
      }
    }
  },
  "version": "1.0"
}
```

```sh
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-gcc-4-0-runtime-validation --json
{
  "live-bootstrap-gcc-4-0-runtime-validation": [
    {
      "severity": "warning",
      "message": "tasks incomplete: {'done': 4, 'todo': 1, 'in_progress': 0}"
    }
  ]
}
```

```sh
command -v openspec_gate || true
```

## Result

- Strict OpenSpec validation passed.
- Helper verification was rerun after V4 blocker evidence was captured; any incomplete-task warning is expected until this V5 checkbox is marked complete.
- `openspec_gate` is unavailable in this environment; no model gate pass is claimed.

## Final pre-archive task verification

After marking V5 complete:

```sh
openspec validate live-bootstrap-gcc-4-0-runtime-validation --strict
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-gcc-4-0-runtime-validation --json
```

Result:

```text
Change 'live-bootstrap-gcc-4-0-runtime-validation' is valid
{
  "live-bootstrap-gcc-4-0-runtime-validation": []
}
```

## Post-archive validation

```sh
openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-gcc40-archive.json
openspec validate --all --strict --json > /tmp/openspec-all-after-gcc40-archive.json
git diff --check
```

Result: all commands exited 0 before staging.
