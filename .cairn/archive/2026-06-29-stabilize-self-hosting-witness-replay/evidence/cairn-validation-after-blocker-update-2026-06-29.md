# Cairn validation after blocker update

Date: 2026-06-29

## Command

```text
git diff --check && nix run path:/home/brittonr/git/cairn#cairn -- validate --root "$PWD"
```

## Output

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}
```
