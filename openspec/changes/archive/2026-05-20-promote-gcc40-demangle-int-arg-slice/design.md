## Context

The current GCC 4.0 demangle shim intentionally recognizes only a tiny subset of Itanium names. The last archived slice promoted a selected deeper nested zero-argument function and recorded schema `mantle-gcc40-native-demangle-slice-v2`.

## Goals / Non-Goals

Goals:
- Add one bounded single-`int` argument demangle shape with receipt-backed evidence.
- Keep zero-argument regressions and unsupported-shape rejections.
- Fail closed on stale v2 markers or receipt drift.

Non-goals:
- General parameter-list parsing.
- Non-`int` type decoding.
- Full `cp-demangle` or native GCC 4.0 correctness.

## Decisions

### 1. Promote only Itanium `i` as a selected argument suffix

Choice: accept a selected `i` suffix and render `(int)` for the bounded shapes used by the smoke.

Rationale: `i` is the smallest meaningful step from zero-argument `v` without pretending to parse arbitrary type lists.

Alternative: implement a parameter-list parser. Rejected as too broad for the current evidence slice.

### 2. Version the receipt and markers

Choice: move to schema `mantle-gcc40-native-demangle-slice-v3` and `*_int_arg_itanium_v3_boundary` markers.

Rationale: stale v2 evidence must not silently satisfy the promoted argument-bearing slice.

## Risks / Trade-offs

- Overclaim risk: mitigated by explicit non-claims and rejected inputs for unsupported/deeper/non-int shapes.
- Drift risk: mitigated by BLAKE3 transcript checks, marker requirements, and stale-marker denylist.
