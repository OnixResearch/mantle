# Design: Normalize build execution envelope

## Context

Crunch already controls the build boundary, so the right fix for ambient-state
leaks is to clamp the builder envelope before the sandboxed command starts. If
this is not done centrally, every package inherits host locale, timezone, or
permission drift for free.

## Goals / Non-Goals

**Goals:**

- define one canonical env for sandboxed builds
- set an explicit build umask
- prevent strict-mode env leakage through unsafe overrides
- add focused tests for envelope normalization

**Non-Goals:**

- redesign the broader report schema
- implement the whole determinism harness
- change fetch or closure behavior

## Decisions

### 1. Normalize the sensitive env subset centrally

**Choice:** normalize at least `HOME`, `PATH`, `PWD`, `TMP`, `TEMP`, `TMPDIR`,
`TEMPDIR`, `USER`, `LOGNAME`, `SHELL`, `LANG`, `LC_ALL`, `TZ`, `TERM`,
`SOURCE_DATE_EPOCH`, `NIX_BUILD_CORES`, and `NIX_STORE` in the build-request
path.

**Rationale:** that path already owns sandbox defaults and is the narrowest
place to enforce one consistent envelope.

### 2. Set an explicit build umask

**Choice:** set an explicit umask in the build launcher before the builder runs.

**Rationale:** output permission drift should not depend on the shell that
started crunch.

### 3. Strict mode rejects unsafe overrides

**Choice:** strict mode rejects derivation attempts to override the
reproducibility-sensitive subset of the canonical env, except for explicitly
allowed inputs such as `SOURCE_DATE_EPOCH`.

**Rationale:** once crunch claims a strict envelope, the derivation should not
be able to reintroduce host-like variability through the same channel.

## Risks / Trade-offs

**Some packages may rely on lax env semantics**
That is exactly why strict mode exists as a declared stronger contract while
practical mode can remain more permissive where needed.
