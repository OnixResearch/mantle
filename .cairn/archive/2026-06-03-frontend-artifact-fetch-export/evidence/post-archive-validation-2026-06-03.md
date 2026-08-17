# Evidence: post-archive validation

Date: 2026-06-03

## Command

```console
$ git diff --check && cairn validate --root .
```

## Result

The command exited successfully after archiving `frontend-artifact-fetch-export` and manually promoting the accepted spec deltas.

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
```

The remaining active change counted by `changes: 1` is unrelated: `cairn/changes/source-built-rust-seed-closure/`.
