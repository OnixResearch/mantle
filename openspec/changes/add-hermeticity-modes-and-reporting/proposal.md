# Add hermeticity modes and reporting

## Sequence

Step 1 of 6. This change establishes the operator-facing contract that later
hardening changes build on.

## Why

Crunch needs two honest execution profiles:

- practical runs that keep bootstrap and local development usable,
- strict runs that reject degraded hermeticity.

Today that distinction is implicit. Fallbacks and degraded conditions mostly
show up as warnings or log lines, so operators cannot tell what kind of claim a
successful build actually supports.

## What Changes

- add an explicit hermeticity mode selection to build-entry commands
- thread that selection into the pipeline and self-build orchestration
- define typed hermeticity audit events for degraded execution facts
- expose the selected mode and audit facts in human and JSON build reporting

## Capabilities

### New Capabilities

- `hermeticity-mode-selection`: operators can request practical or strict build behavior explicitly
- `hermeticity-audit-reporting`: build results expose degraded execution facts instead of hiding them in logs

## Impact

- **Files**: `src/main.rs`, `src/build_cmd.rs`, `src/self_build.rs`, build-report rendering, and pipeline config plumbing
- **APIs**: build-entry config gains hermeticity mode; build results gain audit-event reporting
- **Testing**: add CLI and pipeline tests for mode plumbing and report rendering

## Non-Goals

- normalize the full build environment
- change closure or fetcher semantics yet
- tighten self-build proof behavior yet
