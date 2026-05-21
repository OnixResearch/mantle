## Context

v14 narrowed the c-parse source frontier to a `config.h` plus `<stdio.h>` interaction. Direct `<stdio.h>` without a config header succeeds, full `config.h` plus `<stdio.h>` fails, and the six-undef `config-undef6.h` plus `<stdio.h>` succeeds. The diagnostic derivation is already near the size where broad matrices risk host argument limits, so v15 must stay compact.

## Goals

- Identify a narrower config macro/typedef subset that distinguishes the failing full-config stdio probe from the passing six-undef stdio probe.
- Preserve v11-v14 evidence in the receipt and parity validation.
- Keep the result diagnostic/source-frontier only.

## Non-Goals

- Fix GCC 4.0 native `c-parse.o` compilation.
- Claim native GCC 4.0 compiler/source-build correctness.
- Reintroduce broad generated-header or make-log matrices.

## Decisions

### 1. Probe the known six-undef interaction directly

**Choice:** Add compact probes that start from `config.h`, remove or isolate the six v14-normalized config definitions, and compile `<stdio.h>` under the same c-parse flags.

**Rationale:** v14 already established that full `config.h` fails and `config-undef6.h` succeeds. The smallest useful v15 evidence is a bounded distinction inside that six-definition delta, not another source include sweep.

### 2. Preserve fail-closed parity evidence

**Choice:** Bump the receipt schema to v15 and require observed v15 config/stdio fragments plus diagnostic markers in `src/bootstrap_parity.rs`.

**Rationale:** The parity report must reject stale v14 evidence and must continue to prevent `gcc.4.0` from being promoted by source-frontier diagnostics.

## Risks / Trade-offs

- The six-undef bisection may still identify a group rather than a single macro. Mitigation: record exact marker values and keep the retirement condition focused on advancing beyond the interaction.
- Diagnostic script size may grow. Mitigation: add only a small fixed set of probes and check eval script size before long builds.
