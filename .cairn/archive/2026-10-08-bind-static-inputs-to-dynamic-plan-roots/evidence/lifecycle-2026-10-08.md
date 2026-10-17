# Lifecycle evidence (2026-10-08 UTC)

Leviathan worktree `wt/bind-static-inputs-to-dynamic-plan-roots`, implementation commit `1bfdbb784181df6d6cea4cc35544beb9d40b1068` (`Bind static inputs to dynamic-plan roots`), Cairn CLI `~/mantle-cairn-finish/cairn-result/bin/cairn` (cairn `007ba51`, the campaign canonical build). Raw JSON is in `~/mantle-cairn-finish/logs/bind-static-inputs-to-dynamic-plan-roots/gates/`.

```text
$ cairn validate --root .   # exit 0
{'valid': True, 'changes': 35, 'issues': [], 'layout': 'cairn', 'policy': 'mantle-default'}
$ cairn gate proposal bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0
{'stage': 'proposal', 'verdict': 'PASS', 'valid': True, 'issues': []}
$ cairn gate design bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0
{'stage': 'design', 'verdict': 'PASS', 'valid': True, 'issues': []}
$ cairn gate tasks bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0
{'stage': 'tasks', 'verdict': 'PASS', 'valid': True, 'issues': []}
$ cairn traceability coverage --root . --json   # exit 0
{'verdict': 'pass', 'valid': True, 'requirements': 155, 'referenced': 155, 'missing': [], 'dangling': [], 'profile_id': 'mantle-default'}
$ cairn change depend list bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0
{'dependencies': [], 'issues': []}
$ cairn sync bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0 (dry run)
{'blocked': False, 'dry_run': True, 'actions': ['sync_delta_spec ./.cairn/changes/bind-static-inputs-to-dynamic-plan-roots/specs/dynamic-plan-output-inputs/spec.md']}
$ cairn sync bind-static-inputs-to-dynamic-plan-roots --root . --execute   # exit 0
{'blocked': False, 'dry_run': False, 'actions': ['sync_delta_spec ./.cairn/changes/bind-static-inputs-to-dynamic-plan-roots/specs/dynamic-plan-output-inputs/spec.md']}
$ CAIRN_ARCHIVE_DATE=2026-10-08 cairn archive bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0 (dry run, before T4.5 was checked)
{'blocked': True, 'reasons': ['bind-static-inputs-to-dynamic-plan-roots: tasks not archive-ready (todo: 1, in_progress: 0, unmarked: 0)']}
```

The executed sync created `.cairn/specs/dynamic-plan-output-inputs/spec.md` from the change delta (five added requirements). The traceability profile `mantle-default` scopes requirement sources to the release-provenance spec, so its pass does not cover these requirements; the `r[impl]`/`r[verify]` markers for all five are listed in `finish-2026-10-07.md`.

After checking T4.5:

```text
$ cairn gate tasks bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0
{'stage': 'tasks', 'verdict': 'PASS', 'valid': True, 'issues': [], 'task_done': None, 'task_todo': None}
$ CAIRN_ARCHIVE_DATE=2026-10-08 cairn archive bind-static-inputs-to-dynamic-plan-roots --root .   # exit 0 (dry run)
{'blocked': False, 'reasons': []}
$ CAIRN_ARCHIVE_DATE=2026-10-08 cairn archive bind-static-inputs-to-dynamic-plan-roots --root . --execute
```

The executed archive and the following `cairn validate --root .` are recorded in the archive commit message and in `logs/bind-static-inputs-to-dynamic-plan-roots/gates/archive-execute.txt` / `validate-after-archive.json` on Leviathan.
