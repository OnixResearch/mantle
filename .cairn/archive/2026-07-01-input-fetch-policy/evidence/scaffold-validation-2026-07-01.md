# Scaffold validation — input fetch policy — 2026-07-01

This transcript validates the Cairn scaffold for project input fetch policy and offline preflight semantics. Tasks remain intentionally unchecked; this proves only scaffold validity and gate shape, not implementation or offline enforcement.

## Commands

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal input-fetch-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design input-fetch-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks input-fetch-policy --root .
```

## Transcript

```text
== git diff --check ==
status: ok
== cairn validate ==
  "issues": [],
  "valid": true
== proposal input-fetch-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design input-fetch-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks input-fetch-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
```
