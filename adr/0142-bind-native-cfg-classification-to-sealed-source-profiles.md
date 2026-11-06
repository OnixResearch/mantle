# ADR 0142: Bind native cfg classification to sealed source profiles

## Status

Accepted 2026-11-06 for successor source-topology authoring on the admitted
x86_64-unknown-linux-musl native route. This does not accept preparation, provider
execution, or a fixed-point proof.

## Context

The current consumer is Cargo-free Rust successor graph construction. Real
preparation passed consuming-lock repair, then exposed atomic-width predicates,
unselected dev-dependency predicates, rustix backend modes, getrandom backend
comparisons and opt-in tracing modes in the original authenticated archives.

The compiler profiles are constructed explicitly by the source author. They
supply no global opt-in mode cfgs or getrandom backend override. Bootstrap cfgs
and each crate's feature/build-script cfgs are separate from the global target
cfgs used to select Cargo dependencies. A Cargo feature named valuable does not
enable the tracing_unstable global mode.

## Decision Drivers

- Model the actual sealed native profiles, not ambient host flags.
- Preserve source-owned target/dependency semantics without selecting substitute
  versions or backends to evade a diagnostic.
- Keep cfg admission closed: unknown active predicates must still fail.
- Avoid repeated special cases for each archive's next known opt-in mode.
- Preserve active/inactive feature transitions and separate resolution scopes.

## Decision

Continue using the existing pure cfg classifier for the fixed native route.
Classify target_has_atomic widths 8/16/32/64/ptr from the original target's
64-bit ceiling. A 128-bit atomic dependency is not selected.

Review and explicitly enumerate the opt-in mode names used by the original
source dependency tables. The sealed native profiles leave these names unset:

- Native/backend modes: rustix_use_libc, miri, rustix_use_experimental_asm,
  aes_armv8 and windows_raw_dylib.
- Instrumentation/concurrency modes: tracing_unstable, tokio_unstable,
  tokio_taskdump, tokio_uring, loom, crossbeam_loom and fuzzing.

Likewise, comparisons against getrandom_backend values are false because the
global override is absent. This preserves the original default-backend clauses;
per-crate build scripts may later emit their own cfgs without retroactively
changing Cargo dependency selection.

Package features do not populate this global target-selection context either.
Feature comparisons in target tables are false even when the package enables
the corresponding Cargo feature. A real Cargo metadata probe with alloc enabled
selects the not(feature = "alloc") dependency, matching the original archive's
faster-hex lock edge. Feature expansion still controls ordinary optional edges
and per-crate compiler flags; those namespaces must not be conflated.

This is an explicit closed classification, never a catch-all unknown=false
rule or an automatic allowlist imported from arbitrary archive text. The source
inventory is diagnostic evidence; each newly admitted mode requires review.
A future native profile that enables one of these modes must update this
classification and its behavioral checks together. This interface does not
claim arbitrary user-supplied RUSTFLAGS or Miri/loom/fuzz execution support.

Select dependency kinds and optional features before requiring their target
predicate classification. Dev-only and disabled optional edges do not grant
producer authority and cannot force irrelevant cfg admission. Selecting a
previously inactive edge still requires its predicate to be admitted.

## Consequences

Mantle's Rust source-authoring owners maintain this policy with the native
profile constructor. No new policy override, environment read, execution probe,
or compatibility path is added to the pure resolver.

Behavioral coverage checks exact locked selections, target atomic boundaries,
native/backend branch selection, Cargo-feature versus global-mode separation,
and active/inactive unknown predicates. The real preparation command remains
the integration check. Its success would still not execute a provider or prove
compiler correctness, a fixed point, or release readiness.
