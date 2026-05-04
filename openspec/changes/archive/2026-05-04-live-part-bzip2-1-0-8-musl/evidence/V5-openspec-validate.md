# V5 OpenSpec validation

Commands:

```sh
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify --json live-part-bzip2-1-0-8-musl || true
openspec validate live-part-bzip2-1-0-8-musl --strict --json
openspec validate --all --strict --json > /tmp/openspec-strict-bzip2-before-archive.json
```

Results before marking this final checkbox complete:

- helper warning only: `tasks incomplete: {'done': 7, 'todo': 1, 'in_progress': 0}`
- change strict validation: `passed: 1`, `failed: 0`
- all strict validation: passed with `failed: 0`
