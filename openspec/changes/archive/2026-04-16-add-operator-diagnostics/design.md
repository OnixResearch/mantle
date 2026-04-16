# Design: Add operator diagnostics

## Context

Crunch already knows a lot before and after a build:

- whether required host tools or store paths are missing
- which roots are cached, substitutable, or need local build work
- where build logs were stored
- which stage or subsystem produced the failure

Operators do not get that information through one coherent supported surface.
Instead they piece it together from README notes, verbose logs, and internal
knowledge. That is expensive for ordinary development and painful for CI or
self-hosting proof triage.

## Goals / Non-Goals

**Goals:**

- provide a no-mutate preflight check for common operator errors
- preview planned build behavior without dispatching builds
- normalize failure diagnostics so human and JSON consumers see the same facts
- keep diagnostics side-effect free unless the user explicitly starts a build

**Non-Goals:**

- automatic repair of environment problems
- full graph-query tooling
- replacing saved build logs with inline-only diagnostics
- introducing a long-running daemon

## Decisions

### 1. `crunch doctor` is a pure preflight command

**Choice:** `crunch doctor` checks prerequisites and reports findings without
mutating store state or starting builds.

**Rationale:** a diagnostic command must be safe to run first, especially in
support and CI contexts.

**Implementation:** doctor checks selected command prerequisites such as
nightly toolchain visibility, `bwrap`, sandbox shell availability, writable
state or store paths where required, FUSE or `fusermount3` availability for
workflows that depend on it, and other host capabilities already known to cause
common failures. Profile selection comes from an explicit doctor profile choice
or a documented default profile when none is supplied.

### 2. Plan mode reuses build analysis but stops before dispatch

**Choice:** build-plan preview shares evaluation and cache-analysis logic with
normal builds, then stops before local build dispatch, substitution download, or
store mutation.

**Rationale:** plan output should be truthful. Reusing the real analysis path is
better than maintaining a second approximation.

**Implementation:** `crunch build --plan` evaluates roots, resolves cache or
substitution intent where possible, and reports per-root planned action such as
`cached`, `substitute`, `build`, or `preflight-error`.

### 3. Failure diagnostics use a typed envelope

**Choice:** crunch emits a typed failure envelope that records failing root,
failing phase, error class, and saved log path when available.

**Rationale:** human and machine-readable output should not disagree about what
failed.

**Implementation:** human output renders a concise summary before the detailed
log reference; JSON output exposes the same fields in a stable schema.

## Risks / Trade-offs

**[Doctor drift]**
Preflight checks can fall behind real runtime requirements.

**Mitigation:** reuse existing config and prerequisite discovery code where
possible, and add regression tests for known failure classes.

**[Plan surprises]**
A no-build preview can diverge from real execution if it uses approximations.

**Mitigation:** share as much analysis logic as possible with the real build
path and keep the initial reported states within the named action labels
`cached`, `substitute`, `build`, and `preflight-error`.

**[Too much output]**
Verbose diagnostics can overwhelm ordinary operators.

**Mitigation:** keep human summaries concise and push detail into structured
fields or referenced logs.
