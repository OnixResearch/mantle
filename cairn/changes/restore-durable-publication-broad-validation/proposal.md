# Restore durable publication broad validation

## Why

Focused durable publication adoption passes, but broad Mantle validation still has independent failures. The filtered Nix source omits `fixtures/content-bound-requirements`, the bootstrap blocker inventory reports live full-source findings, full dependency Clippy fails in vendored code, and the prior Octet owner revision rejected a diagnostic path that contained `..`.

## What Changes

- Make the Nix source closure include every checked content-bound requirement fixture.
- Refresh blocker classification without hiding live bootstrap blockers or promotion claims.
- Separate Mantle-owned lint results from vendored dependency diagnostics while keeping dependency failures visible.
- Bind validation to the accepted Octet owner revision that checks the current Mantle scope without the former parent-traversal diagnostic.
- Re-run workspace tests and the broad flake rail until they pass or expose a newly bounded blocker.

## Impact

This change repairs validation infrastructure. It does not weaken durable publication tests, bootstrap truth, dependency audits, or release claims.
