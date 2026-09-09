# Dev resume baseline

Date: 2026-09-04
Source commit: `079f02853a84114b33d8e91583c7456c16f50613`

The existing source-built fixed-point suite passed with 77 tests, 0 failures, and 3 ignored tests.

The documented first-party root Clippy command passed. Cairn validation and all three change gates also passed.

The baseline has these limits:

- Stage markers exist only in an attempt staging directory.
- The staging path contains the process ID, so a later process gets a different directory.
- Provider-cache entries hold StageX and native provider trees, but not the transition execution tree.
- `--dev-resume` does not restore a content-addressed bundle into a fresh staging directory.
- Dev reports do not list restored and executed stages separately.
- A promoted run already rejects dev cache, resume, and fast-fail options when promoted checkpoint storage is selected.

This baseline proves existing tests and current behavior only. It does not prove fresh-directory resume or a complete runtime cache cycle.
