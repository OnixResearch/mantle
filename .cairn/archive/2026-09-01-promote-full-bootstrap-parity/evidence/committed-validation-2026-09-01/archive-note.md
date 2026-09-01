# Archive normalization

`cairn archive promote-full-bootstrap-parity --root . --execute` created
`.cairn/archive/1970-01-01-promote-full-bootstrap-parity/`. The directory was
renamed to `.cairn/archive/2026-09-01-promote-full-bootstrap-parity/` to match
the session date.

The archive operation moved the accepted package but did not add its ADDED
requirement to `.cairn/specs/bootstrap-inventory/spec.md`. The requirement was
copied without semantic changes from the archived delta into the canonical
specification before post-archive validation.
