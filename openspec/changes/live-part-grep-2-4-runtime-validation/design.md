# Design: grep 2.4 runtime validation

## Context

The parent grep derivation now fails closed on missing objects, but runtime validation can be slow because Crunch must build prerequisite bootstrap inputs with bubblewrap. A prior attempt with local `.crunch-drain/grep-store` and `.crunch-drain/grep-state` reached Crunch startup/repair output and then hit the drain tool timeout before a build result was recorded.

## Decisions

### 1. Use a long-running validation runner

**Choice:** run the grep build with a background/long-lived runner and preserve full logs under this change's evidence directory.

**Rationale:** bootstrap prerequisite builds may legitimately exceed short command timeouts.

### 2. Preserve parent proof requirements

**Choice:** keep the parent V2-V5 evidence requirements: build transcript, smoke test, leakage scan, and OpenSpec validation.

**Rationale:** the parent implementation can archive only because runtime proof remains explicit and scoped here.

## Validation

Build `bootstrap/grep-2.4-musl.ncl`, smoke `grep`/`egrep`/`fgrep`, scan for leakage, and run OpenSpec validation/gates.
