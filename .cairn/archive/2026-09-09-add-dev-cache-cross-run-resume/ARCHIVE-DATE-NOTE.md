# Archive date note

`cairn archive add-dev-cache-cross-run-resume --root . --execute` created the archive under `.cairn/archive/1970-01-01-add-dev-cache-cross-run-resume` despite `CAIRN_ARCHIVE_DATE=2026-09-09`. This is the documented Mantle archive-date defect.

The directory was renamed to `.cairn/archive/2026-09-09-add-dev-cache-cross-run-resume`, and `cairn validate --root .` passed afterward. The synced requirements remain present in `.cairn/specs/source-built-fixed-point-improved-iteration/spec.md` (both requirement IDs present after sync). No spec content or evidence was changed by the rename.
