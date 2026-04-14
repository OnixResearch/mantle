## ADDED Requirements

### Requirement: Canonical execution envelope

The build pipeline MUST normalize a canonical execution envelope before each
sandboxed build starts.

At minimum the normalized envelope MUST set stable values for:

- `HOME`
- `PATH`
- `PWD`
- `TMP`, `TEMP`, `TMPDIR`, `TEMPDIR`
- `USER`, `LOGNAME`
- `SHELL`
- `LANG`, `LC_ALL`
- `TZ`
- `TERM`
- `SOURCE_DATE_EPOCH`
- `NIX_BUILD_CORES`
- `NIX_STORE`
- explicit `umask`

In strict mode, derivation-provided environment variables MUST NOT silently
override the reproducibility-sensitive subset of that envelope except for
explicitly allowed inputs such as `SOURCE_DATE_EPOCH`.

#### Scenario: Host locale and timezone do not leak into the build

- GIVEN the host shell has `LANG=en_US.UTF-8` and `TZ=America/New_York`
- WHEN a strict build starts
- THEN the builder sees the canonical crunch-selected locale and timezone values
- AND the host locale and timezone do not leak into the sandbox

#### Scenario: Host umask does not leak into the build

- GIVEN the host process starts with a restrictive or permissive umask
- WHEN crunch dispatches a sandboxed build
- THEN crunch sets the configured build umask before the builder runs
- AND output permissions are determined by the build envelope, not the host shell state
