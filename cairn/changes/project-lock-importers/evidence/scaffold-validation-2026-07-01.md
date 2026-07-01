# Scaffold validation — project lock importers — 2026-07-01

This transcript validates the Cairn scaffold for external pin importers and the Nixtamal mapping surface. Tasks remain intentionally unchecked; this proves only scaffold validity and gate shape, not implementation or importer support.

## Commands

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-lock-importers --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design project-lock-importers --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-lock-importers --root .
```

## Transcript

```text
== git diff --check ==
status: ok
== cairn validate ==
  "issues": [],
  "valid": true
== proposal project-lock-importers ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design project-lock-importers ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks project-lock-importers ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
```
