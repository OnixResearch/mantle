# Archive synchronization note

`CAIRN_ARCHIVE_DATE=2026-09-01 cairn archive ... --execute` created the
correct dated archive directory.

The archive operation moved the change package but did not add its ADDED
requirement to `.cairn/specs/build-correctness/spec.md`. The accepted
requirement was copied without semantic changes before post-archive validation.
