# Bound the GCC 4.0 configure preprocessing bridge

## Why

The selected full-source provider no longer exposes TinyCC delegation, but its historical GCC 4.0 predecessor still carries a compile-backed `-E` compatibility path for Autoconf probes. Comments describe that path as configure-only, yet the generated wrapper currently accepts any C source and has no canonical-directory, source-size, invocation-count, output, or durable audit boundary. That leaves a broader mechanism than the claim.

## What Changes

- Restrict the bridge to canonical `conftest.c` inputs in the explicitly active `libiberty`, `libcpp`, or `gcc` configure directory.
- Reject explicit preprocessing outputs, oversized probes, path escapes, missing authority, and invocation-count exhaustion before invoking a compiler.
- Record every accepted bridge invocation in a bounded build-local audit and validate its exact directory/class shape after configuration.
- Add a deterministic functional-core checker, positive and negative self-tests, checked evidence, and lifecycle coverage for the boundary.
- Keep the path provisional and ineligible for provider, compiler-correctness, or bootstrap-seed-removal claims.

## Success

The runtime wrapper fails closed outside the exact configure-probe scope; the deterministic checker rejects marker, policy, or evidence drift; focused positive and negative tests pass; ordinary quality and lifecycle gates pass; and the accepted spec continues to classify the bridge as bounded frontier debt rather than provider evidence.

## Non-Goals

This change does not replace TinyCC, prove GCC 4.0 correctness, regenerate all GCC sources, alter the selected provider identity, or rerun the already-authenticated provider fixed point unless staged product source changes require it.