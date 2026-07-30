# ADR 0041: Run `ylwrap` rewrites with a bounded native runner

## Status

Accepted (2026-07-30)

## Context

Authenticated binutils 2.30 Make runs Automake `ylwrap` around Bison and Flex. `ylwrap` uses sed to rename generated files, remove the authenticated source prefix from line directives, and rename header guards.

The existing protected GNU sed completes small regular-file transformations. It does not complete the four-program `ylwrap` rewrite of a 50,168-byte `y.tab.c` within 300 seconds. Direct protected Bison completes that parser quickly, so Bison and M4 are not the blocker.

Accepting direct Bison output would bypass authenticated `ylwrap`. Authorizing ambient sed would widen execution authority. Waiting longer would remove the process bound.

A separate attempt rebuilt GNU sed 4.0.9 against native musl. TinyCC musl-v2 segfaulted while preprocessing authenticated `lib/utils.c`, so that route did not produce a usable later sed.

## Decision Drivers

- Keep authenticated Make, Bison, Flex, and `ylwrap` execution.
- Preserve the exact `ylwrap` filename, line-directive, and header-guard transformations.
- Do not authorize general sed semantics in a new helper.
- Reject substituted source prefixes, working directories, generated names, patterns, files, and outputs.
- Bound input size, output size, argument shape, and invocation count.
- Record which sed producer handled each bridge invocation.

## Decision

Mantle builds `bootstrap/stagex-ylwrap-sed-runner.c` with TinyCC musl-v2 and native musl. The regular-file sed bridge dispatches to this runner only for the exact nine-argument `ylwrap` shape:

- four `-e` programs;
- the exact `/^#/!b` gate;
- one source-prefix substitution;
- one filename-substitution program;
- one matching header-guard program;
- one generated regular file.

The runner requires a `ylwrap<digits>` current directory. The source prefix must equal its parent directory. Inputs must be regular, non-symlink files no larger than 8 MiB.

The accepted mapping is one of two closed forms:

- `y.tab.c`, `y.tab.h`, and `y.output` to one shared output stem; or
- `lex.yy.c` to one C output.

Patterns contain only escaped literal metacharacters. Output names are single-component filenames. The guard program must equal the deterministic uppercase-and-underscore mapping of the filename program.

The runner applies literal, non-overlapping substitutions only to lines that start with `#`. The path substitution changes the first match. Filename and guard substitutions change all matches. This is the exact behavior of the accepted `ylwrap` sed programs, not a general sed implementation.

The bridge records `ok:protected-sed` or `ok:ylwrap-sed`. The generated-source probe requires exactly 18 successful `ylwrap` rewrites. Each must have empty standard input, one input file, and nonempty bounded output.

Positive smokes require exact line-directive, include-name, and guard output while preserving a non-directive body line. Negative smokes reject a wrong source prefix, wrong working directory, symlink input, oversized input, broad regex, nested output name, and mismatched guard mapping.

## Alternatives Considered

### Accept direct Bison and Flex output

Rejected because it silently bypasses authenticated `ylwrap` transformations.

### Authorize host sed

Rejected because it introduces ambient host authority and loses the protected producer identity.

### Increase the protected-sed timeout

Rejected because the observed 300-second timeout already exceeds the bounded generator budget.

### Rebuild a complete later GNU sed first

Preferred in principle, but blocked because TinyCC musl-v2 segfaulted while preprocessing the authenticated GNU sed `utils.c` source.

### Add a broad native sed subset

Rejected because it would create unnecessary regex and command-language authority. The runner accepts only the observed authenticated `ylwrap` form.

## Consequences

- Authenticated Make completes the declared binutils and ld Bison/Flex targets.
- The probe validates all 14 Bison files and four Flex files with producer markers.
- The canonical sed audit distinguishes the established GNU sed producer from the bounded `ylwrap` producer.
- The helper does not replace general sed and cannot run arbitrary substitution programs.
- Generated executables and Make children are still outside the protected transition authorization graph.
- This decision does not prove full binutils compilation, binutils behavior, compiler correctness, or provider admission.
