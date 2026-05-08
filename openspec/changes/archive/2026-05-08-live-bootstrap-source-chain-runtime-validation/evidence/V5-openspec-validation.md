# V5 OpenSpec validation evidence

Task-ID: V5
Covers: bootstrap.source.chain.runtime-validation
Captured: 2026-05-08T21:05:19Z

## Commands

```sh
openspec validate live-bootstrap-source-chain-runtime-validation --strict --json
{
  "items": [
    {
      "id": "live-bootstrap-source-chain-runtime-validation",
      "type": "change",
      "valid": true,
      "issues": [],
      "durationMs": 0
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
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-source-chain-runtime-validation --json
{
  "live-bootstrap-source-chain-runtime-validation": [
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
- Helper verification was rerun after V2-V4 blocker evidence was captured; any incomplete-task warning is expected until this V5 checkbox is marked complete.
- `openspec_gate` is unavailable in this environment; no model gate pass is claimed.

## Final pre-archive task verification

After marking V5 complete:

```sh
openspec validate live-bootstrap-source-chain-runtime-validation --strict
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-source-chain-runtime-validation --json
```

Result:

```text
Change 'live-bootstrap-source-chain-runtime-validation' is valid
{
  "live-bootstrap-source-chain-runtime-validation": []
}
```

## Post-archive validation

```sh
openspec validate bootstrap --strict --json > /tmp/openspec-bootstrap-source-chain-archive.json
openspec validate --all --strict --json > /tmp/openspec-all-after-source-chain-archive.json
git diff --check
```

Result: all commands exited 0 before staging.
