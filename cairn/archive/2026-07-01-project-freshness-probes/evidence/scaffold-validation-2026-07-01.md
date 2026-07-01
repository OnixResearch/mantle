# Scaffold validation — project freshness probes — 2026-07-01

This transcript validates the Cairn scaffold for project freshness probe schemas, observation records, and refresh/list-stale semantics. Tasks remain intentionally unchecked; this proves only scaffold validity and gate shape, not implementation or adapter support.

## Commands

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-freshness-probes --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design project-freshness-probes --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-freshness-probes --root .
```

## Transcript

```text
== git diff --check ==
status: ok
== cairn validate ==
  "issues": [],
  "valid": true
== proposal project-freshness-probes ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design project-freshness-probes ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks project-freshness-probes ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
```
