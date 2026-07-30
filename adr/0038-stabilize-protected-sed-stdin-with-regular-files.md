# ADR 0038: Stabilize protected sed stdin with regular files

## Status

Accepted (2026-07-29)

## Context

The authenticated binutils 2.30 `intl/configure` script uses two sed processes in a pipeline to generate `config.status` substitutions.

The protected GNU sed 4.0.9 gives correct output when it reads a named regular file. Its Mes runtime loses the second input line when sed reads the same data from standard input. The generated Gawk assignment then stops at `S["LTLIBOBJS"]=`.

The later full Bash does not cause this defect. A diagnostic run with an ambient sed was not accepted as evidence.

## Decision Drivers

- Keep the authenticated configure and sed programs in control of substitution semantics.
- Do not use ambient sed, shell, or path lookup.
- Do not hard-code Autoconf substitutions in Rust or C.
- Bound input, output, invocation count, argument count, and temporary files.
- Give the generated launcher, bridge script, child tools, and protected sed separate identities and execution boundaries.
- Preserve the established protected sed artifact.

## Decision

Mantle adds a regular-file bridge in front of the protected sed artifact for authenticated configure execution.

A TinyCC musl-v2-built native launcher sets a fixed file-size limit. It then executes the exact later Bash and checked bridge script from explicit environment paths. The launcher uses the existing single-thread semaphore compatibility needed by this native-musl boundary.

The bridge copies bounded standard input into a create-new file under the declared configure scratch. If the sed invocation has no explicit input file, the bridge adds that file as sed's input. The protected sed still parses and executes every sed program.

The bridge rejects mixed standard-input and explicit-file authority. It also rejects unsupported option shapes, missing paths, oversized input, oversized output, lock exhaustion, and invocation-budget exhaustion.

The bridge permits at most 4,096 invocations and 8 MiB per input or output. It records every accepted or rejected invocation. Mantle validates contiguous raw invocation numbers and writes a deterministic canonical multiset audit.

The source identities and compiled identities are exact BLAKE3 values. The tool namespace binds `sed` to the launcher. The launcher uses only the declared full Bash, bridge script, coreutils helpers, native emitter, and protected sed.

## Alternatives Considered

### Rebuild GNU sed against native musl

Deferred because the current experimental rebuild compiles and links but segfaults on the required pipeline scripts. It is not accepted execution evidence.

### Implement the Autoconf substitution generator in Rust or C

Rejected because that would replace authenticated configure and sed semantics with a new partial implementation.

### Patch generated `config.status`

Rejected because the patch would hide the defective producer boundary and authorize generated data after the fact.

### Use ambient sed

Rejected because it violates the protected execution boundary.

## Consequences

- Authenticated `intl/configure` now creates `Makefile`, `config.intl`, and `config.h` successfully.
- The earlier protected sed and full Bash identities remain unchanged.
- The bridge proves only bounded regular-file stabilization for declared configure invocations.
- This decision does not prove general sed correctness, general shell correctness, binutils, compiler correctness, or provider admission.
