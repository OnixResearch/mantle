## Context

The installed GCC 4.0 artifact currently has a driver that delegates compilation to TinyCC and a `cc1` executable that prints a bridge message and exits. A small direct-`cc1` object smoke is the lowest-risk next correctness slice because it exercises the installed frontend path without pretending the real GCC `cc1` is complete.

## Goals / Non-Goals

Goals:
- Make installed `cc1` accept a bounded GCC-shaped compile-to-object invocation.
- Verify output object exists and is non-empty during derivation build and in a logical `/crunch/store` bwrap smoke.
- Preserve partial parity classification.

Non-goals:
- Build native GCC `cc1` from upstream sources.
- Claim full C frontend correctness, optimization, diagnostics, or link behavior.

## Decisions

### 1. Bounded cc1 wrapper

Choice: install a `cc1` wrapper that parses `-quiet`, picks the first C/preprocessed input, honors `-o`, and delegates to TinyCC with `-S`.

Rationale: this matches a real compiler phase boundary better than a message-only executable while remaining compatible with the current TinyCC/Mes handoff.

Alternative: attempt a real upstream `cc1` promotion now. Rejected for this increment because generator/header/native frontend debt is broader and needs a separate boundary drain.

## Risks / Trade-offs

- The wrapper is still a bridge. Mitigation: spec and parity notes keep `gcc.4.0` partial/blocking.
- TinyCC object output shape may vary. Mitigation: smoke only requires non-empty object plus successful invocation, not exact object text.
