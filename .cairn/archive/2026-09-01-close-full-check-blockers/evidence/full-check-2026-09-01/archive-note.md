# Archive normalization

`cairn archive close-full-check-blockers --root . --execute` created
`.cairn/archive/1970-01-01-close-full-check-blockers/`. The directory was
renamed to `.cairn/archive/2026-09-01-close-full-check-blockers/` for the
session date.

The archive operation moved the package but did not add its ADDED requirement
to `.cairn/specs/bootstrap-inventory/spec.md`. The accepted requirement was
copied without semantic changes before post-archive validation.
