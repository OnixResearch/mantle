# Restore durable publication broad validation

## Why

Focused durable publication adoption passes, but broad Mantle validation still has independent failures. The filtered Nix source omits `fixtures/semantic-operation`, the bootstrap blocker inventory can classify new source text as live findings, full dependency Clippy fails in vendored code, and pinned Octet rejects a diagnostic path that contains `..`.

## What Changes

- Make the Nix source closure include every checked semantic-operation fixture.
- Refresh blocker classification without hiding live bootstrap blockers or promotion claims.
- Separate Mantle-owned lint results from vendored dependency diagnostics while keeping dependency failures visible.
- Normalize or update the Octet diagnostic path through an accepted producer contract.
- Re-run workspace tests and the broad flake rail until they pass or expose a newly bounded blocker.

## Impact

This change repairs validation infrastructure. It does not weaken durable publication tests, bootstrap truth, dependency audits, or release claims.
