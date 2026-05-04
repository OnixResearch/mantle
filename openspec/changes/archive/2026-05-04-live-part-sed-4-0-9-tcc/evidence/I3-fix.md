# I3 sed-tcc source hardening

- Task-ID: I3
- Covers: `bootstrap.part.sed.4.0.9.tcc`
- Status: complete
- Timestamp: 2026-05-04T13:34:25Z

## Changes

- Added explicit provenance and first-consumer comments to the sed fixed-output source pin.
- Added fail-closed `test -f` checks after each required library and sed object compile.
- Removed the required archive-link path from the output contract and linked the final executable directly from the required object files.
- Added executable chmod before builder-local execution, then a positive substitution smoke (`test -> ok`) in addition to `sed --version`, using BusyBox grep explicitly.

## Verification command

Source-pin checker transcript: `evidence/I3-source-pin-recheck.log`.
