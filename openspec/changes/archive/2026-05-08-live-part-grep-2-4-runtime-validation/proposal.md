# Complete grep 2.4 runtime validation

## Why

Parent change `live-part-grep-2-4` hardened `bootstrap/grep-2.4-musl.ncl` so compile failures are no longer suppressed and the output contract is checked. Runtime validation then exceeded the local drain command budget while building prerequisite bootstrap stages through Crunch.

## What Changes

- Re-run `bootstrap/grep-2.4-musl.ncl` with bubblewrap available and writable local state/store directories.
- Record the completed build transcript, output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test `grep`, `egrep`, and `fgrep`.
- Scan derivation and transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to complete grep 2.4 validation.

Out of scope: changing unrelated bootstrap parts or downstream transition chains.
