# Design: Remove host git from fetchGit

## Context

A host-discovered `git` binary is an ambient dependency. Different hosts may
have different `git` versions, config files, or credential helpers, and crunch
cannot make strong hermeticity claims while fetch semantics depend on that.

## Goals / Non-Goals

**Goals:**

- stop scanning arbitrary host paths for `git`
- move `fetchGit` to a crunch-controlled implementation path
- keep the existing `fetchGit` user contract intact

**Non-Goals:**

- redesign every fetcher
- add mirror selection or signature verification in this change

## Decisions

### 1. No arbitrary host PATH discovery

**Choice:** `fetchGit` no longer scans `PATH` or common host filesystem paths for
an ambient `git` binary.

**Rationale:** host-tool discovery is the impurity we are removing.

### 2. Preserve the user-facing fetchGit contract

**Choice:** the Nickel and derivation contract stays the same: users still name a
URL, revision, and expected hash.

**Rationale:** hardening implementation details should not force a new fetcher API.

## Risks / Trade-offs

**Implementation cost**
A crunch-controlled git path is more work than a subprocess call, but it buys a
cleaner hermetic boundary.
