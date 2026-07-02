# Scaffold validation — forge-agnostic VCS inputs — 2026-07-01

This transcript validates the Cairn scaffold for Darcs, Pijul, and Fossil project-input support. Tasks remain intentionally unchecked; this proves only scaffold validity and gate shape, not implementation or fixture support.

## Commands

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal forge-agnostic-vcs-inputs --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design forge-agnostic-vcs-inputs --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks forge-agnostic-vcs-inputs --root .
```

## Transcript

```text
== git diff --check ==
status: ok
== cairn validate ==
  "issues": [],
  "valid": true
== proposal forge-agnostic-vcs-inputs ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design forge-agnostic-vcs-inputs ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks forge-agnostic-vcs-inputs ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
```
