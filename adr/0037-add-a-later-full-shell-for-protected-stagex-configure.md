# ADR 0037: Add a later full shell for protected StageX configure

## Status

Accepted (2026-07-29)

## Context

The checked binutils 2.30 recipe runs authenticated configure scripts and GNU Make recipes.

Protected StageX execution forbids ambient `/bin/sh`. Every executable needs an exact path, BLAKE3 digest, and producer authorization.

The early source-built Bash 2.05b predecessor is intentionally small. It exposes only `echo` and a no-op `eval`. Other required builtins are stubs.

ADR 0036 keeps that early shell unchanged for the bounded GNU grep bridge. A general binutils configure path has a different requirement.

## Decision Drivers

- Do not execute ambient `/bin/sh` or `/bin/busybox` after the protected transition.
- Run the authenticated binutils configure and Make scripts instead of simulating their results.
- Reuse the authenticated Bash 2.05b source already in the StageX closure.
- Keep the early Bash and grep bridge identities unchanged.
- Bind shell generation, compilation, runtime behavior, and child executions to the protected plan.

## Decision

Mantle adds a separate full-shell stage after the protected parser generators.

The stage copies the earlier authenticated and configured Bash source. It does not replace the early Bash artifact.

TinyCC musl-v2 compiles `mkbuiltins` and links it against protected native musl. Protected execution uses `mkbuiltins` to generate the selected real builtin sources, `builtext.h`, and `builtins.c` from authenticated `.def` files.

TinyCC musl-v2 then compiles and links the full non-interactive Bash against protected native musl.

The build applies three bounded compatibility adaptations. It uses the authenticated `psize.sh` fallback value. It inlines `fmtulong.c` into `fmtumax.c` to avoid a TinyCC include-expansion defect. It also adds direct socket syscalls and single-thread semaphore operations for native-musl sources that the earlier compiler could not build.

These adaptations do not add shell execution authority. The semaphore operations support only Bash's single-thread bootstrap path. They do not claim a complete threading runtime.

The smoke boundary covers shell control flow, required builtins, an external absolute child, and a GNU Make recipe that uses the new shell. Each child has a separate exact authorization.

The binutils stage must use this protected shell through explicit `CONFIG_SHELL` and `SHELL` paths. It must not use the early Bash, ambient `/bin/sh`, or a PATH shell lookup.

## Alternatives Considered

### Expand the early Bash in place

Rejected because it would change an established predecessor and invalidate prior transition identities.

### Add Dash from a new source closure

Rejected for this checkpoint because it adds a new source and requires separate script-generator ports. Bash source and parser artifacts are already authenticated.

### Port configure and Make semantics into Rust

Rejected because it would replace the checked build behavior with a new partial interpreter.

### Use ambient `/bin/sh`

Rejected because it violates the protected execution boundary.

## Consequences

- The early Bash and native grep subset remain unchanged.
- Binutils gains a declared full-shell prerequisite.
- The full shell is a bounded, single-thread bootstrap artifact, not a general shell-correctness claim.
- Provider admission remains blocked until binutils, native TinyCC, later compiler stages, and receipt materialization complete.
