# ADR 0036: Run the StageX grep bridge with a native subset runner

## Status

Accepted (2026-07-28)

## Context

The checked GNU grep 2.4 recipe emits a shell bridge after the upstream driver fails at the TinyCC handoff.

The bridge uses `test`, `read`, `exit`, `continue`, loops, and shell pattern cases. Its shebang names ambient `/bin/sh`.

Protected StageX execution forbids ambient `/bin/sh`. The source-built Bash predecessor also stubs several required builtins.

Executing the exact bridge with that Bash fails with status 127. Expanding Bash would change a major predecessor and its established boundary.

## Decision Drivers

- Preserve the exact checked bridge bytes and recipe identity.
- Do not execute ambient `/bin/sh` or any undeclared shell.
- Provide the bounded grep subset required by this bootstrap epoch.
- Keep executable identity, inputs, observations, and audit events deterministic.
- Avoid widening the source-built Bash boundary.

## Decision

Mantle retains the exact checked shell bridge as non-executable data. The protected transition never executes its shebang.

A checked C source implements the bridge's declared literal, prefix, suffix, quiet, invert, and option-parsing subset.

TinyCC musl-v2 compiles this source and links it against protected native musl. The resulting runner supplies `grep`, `egrep`, and `fgrep`.

The runner has fixed argument, pattern, line, input, output, error, and time limits. It does not call a shell.

The configured-source identity binds the authenticated GNU source, checked Nickel recipe, exact bridge bytes, and native runner source.

Protected smoke commands remain identical to the recipe's three observations. The plan closes two build events and three smoke events.

The receipt calls the executable a bridge runner. It does not call it a full GNU grep build.

## Alternatives Considered

### Execute the bridge with ambient `/bin/sh`

Rejected because it violates the one-way protected transition and grants undeclared host authority.

### Expand the source-built Bash predecessor

Rejected because required builtins would change an established predecessor, its binary identity, and its authority surface.

### Simulate bridge results in Rust

Rejected because simulation would not produce an executable prerequisite for later bootstrap consumers.

### Stop at a blocker

Rejected because the native runner preserves the checked bridge boundary without ambient shell authority.

## Consequences

- The exact recipe bridge remains reviewable and identity-bound.
- Later protected stages can invoke a native executable without shell fallback.
- The runner implements only the checked bridge subset. It does not provide general regular expressions or full GNU grep behavior.
- Full GNU grep, parser generators, binutils, native TinyCC, and provider admission remain separate claims.
