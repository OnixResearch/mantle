# Design: Native self target-cfg predicate scope

## Problem

The native package planner currently supports only a tiny target-cfg subset. Mantle's own dependency graph uses common Cargo target dependency predicates including `not`, `any`, `all`, direct target triples, target family/env/endian/pointer-width/atomic predicates, and optional backend feature selectors. Blocking on these predicates prevents `native_package_target_planning` and downstream host graph planning from reaching the next real build blocker.

## Approach

Replace the one-off target-cfg checks with a small pure evaluator:

1. Strip the outer `cfg(...)` wrapper, or handle exact target triple table names without it.
2. Recursively evaluate `not(...)`, `any(...)`, and `all(...)` using comma splitting that respects nested parentheses and quoted strings.
3. Evaluate bounded atoms against the active target triple:
   - `unix`, `windows`
   - `target_os`, `target_arch`, `target_family`, `target_vendor`, `target_env`, `target_abi`, `target_endian`, `target_pointer_width`, `target_has_atomic`
4. Return deterministic `None` for malformed/unknown syntax so existing `unsupported-target-cfg-surface` blockers remain fail-closed.
5. Return deterministic false for known optional selector atoms that are not active in the bounded native fragment.

## Verification

- Unit tests exercise nested supported predicates and direct target triple names.
- Existing CLI target-cfg fixtures continue to prove selected registry dependency execution.
- Mantle self probe reports reduced or eliminated `unsupported-target-cfg-surface` blockers and exposes the next blocker class.
