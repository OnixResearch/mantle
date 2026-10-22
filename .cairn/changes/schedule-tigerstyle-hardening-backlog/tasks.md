# Tasks: Schedule the Tiger Style hardening backlog

## Scope

- [ ] [task:S1][serial] Record the per-package Tiger Style findings for every
  package in `workspace.metadata.tigerstyle.default_scope`, one package per
  run so a failing package does not hide the packages behind it.
  r[mantle.tigerstyle_backlog.scan]
- [ ] [task:S2][after:S1] Close the mechanical findings in the source across
  the 24 affected packages: assertion density, compound conditions, numeric
  unit naming, bool naming, explicit defaults, unchecked unsigned arithmetic,
  function length, nested conditionals, parameter count, ambiguous
  parameters, collection growth and loop bounds. r[mantle.tigerstyle_backlog.fix]
- [ ] [task:S3][after:S1] Record every finding that cannot be removed without
  changing observable behaviour, changing a published public API, or
  restructuring code outside the affected package, as a file-scoped
  `#![allow(tigerstyle::...)]` with a `HARDENING-BACKLOG` comment naming
  this change's evidence file. Do not extend any existing backlog block
  beyond its recorded categories. r[mantle.tigerstyle_backlog.record]
- [ ] [task:S4][after:S2][after:S3] Write the deferred-finding log and the
  green gate transcript into this change's evidence. r[mantle.tigerstyle_backlog.evidence]
- [ ] [task:V1][after:S4] Run `cargo tigerstyle check` over the declared
  scope, the first-party strict Clippy gate, the workspace lib and tests, and
  the affected packages' format checks, and retain the exact output.
  r[mantle.tigerstyle_backlog.verify]

## Boundaries

- `dylint.toml` is consumer-owned and is not changed by this change.
- No non-`tigerstyle` allow is added anywhere.
- No finding is suppressed by removing a test, weakening a check or
  accepting a different input.
