# Scaffold validation — remote/offline build brainstorm — 2026-06-30

This transcript validates the generalized remote/offline scaffold packages after adding stronger guarantees. Added guarantee areas: language-neutral source adapter contracts, filesystem source canonicalization, atomic/pinned source imports, deterministic route ranking, claim-strength-aware routing, privacy-safe remote upload summaries, trust-root snapshot/revocation handling, and complete evidence-chain classification. Tasks remain intentionally unchecked; this proves only Cairn scaffold validity and gate shape.

## Commands

```sh
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal offline-source-bundle-manifest --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design offline-source-bundle-manifest --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks offline-source-bundle-manifest --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal build-realization-routing-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design build-realization-routing-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks build-realization-routing-policy --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal portable-build-receipt-bundles --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design portable-build-receipt-bundles --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks portable-build-receipt-bundles --root .
```

## Transcript

```text
== git diff --check ==
status: ok
== cairn validate ==
  "issues": [],
  "valid": true
== proposal offline-source-bundle-manifest ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design offline-source-bundle-manifest ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks offline-source-bundle-manifest ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== proposal build-realization-routing-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design build-realization-routing-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks build-realization-routing-policy ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== proposal portable-build-receipt-bundles ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== design portable-build-receipt-bundles ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
== tasks portable-build-receipt-bundles ==
  "issues": [],
  "valid": true,
  "verdict": "PASS"
```
