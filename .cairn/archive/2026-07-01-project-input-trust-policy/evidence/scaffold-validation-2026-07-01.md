# Scaffold validation — project input trust policy — 2026-07-01

This transcript validates the Cairn scaffold for project input and patch trust policies plus bounded trust-evidence wording. Tasks remain intentionally unchecked; this proves only scaffold validity and gate shape, not implementation or verifier support.

## Commands

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-input-trust-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design project-input-trust-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-input-trust-policy --root .
```

## Transcript

```text
== git diff --check ==
status: ok
== cairn validate ==
  "issues": [],
  "valid": true
== proposal project-input-trust-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design project-input-trust-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks project-input-trust-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
```
