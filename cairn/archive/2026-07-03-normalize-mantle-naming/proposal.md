## Why

Mantle's user-facing name is Mantle, but older Crunch wording still appears in docs and status surfaces. Some `crunch-*` names are real crate, path, package, command, or compatibility identifiers and must remain exact. The repo needs a bounded naming cleanup that improves public prose without breaking binary-compatible surfaces or historical archives.

## What Changes

- Audit current user-facing prose for stale Crunch project naming.
- Replace stale prose with Mantle where it is not an exact identifier.
- Preserve exact `crunch-*` crate/package/path/command surfaces that still require that spelling.
- Add or update a guard so future docs/examples keep the same distinction.

## Impact

- **Files**: README/docs/examples/help text, naming drift checks, possibly bridge tests, Cairn build-tool-boundary spec delta.
- **Testing**: positive allowed-identifier fixtures, negative stale-prose fixtures, docs/example checks, `git diff --check`, Cairn validation/gates.
